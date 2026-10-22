"""Plain-text and LaTeX numbers typed into a numeric answer box
(ops "parse_number" and "sig_figs").

The homework app's numeric grader does not use SymPy (it uses ast, Decimal
and Fraction), so every case here is a spec case. args[0] is the raw text a
student typed, recorded exactly (including spaces and unicode).

parse_number   -> the exact value, as SymPy syntax. Typed decimals are exact
               rationals (0.00450 is 9/2000), never Floats. Accepted forms:
               - optional sign: "-", "+", or unicode minus U+2212 (also inside
                 an exponent); spaces around the sign and around operators;
               - decimals with or without a leading digit (".5"), with a
                 trailing point ("5."), with trailing zeros;
               - comma thousands separators, only when the first group has
                 1 to 3 digits and does not start with 0, and every later
                 group has exactly 3 digits (also "{,}" as LaTeX writes it);
                 anything else with a comma is "invalid:ambiguous_comma"
                 ("0,125" is a European 0.125, never 125);
               - scientific notation: e/E with optional sign, and mantissa
                 times a power of ten written with *, \\times, x, X, U+00D7,
                 \\cdot or U+00B7 (spaces optional), exponent as ^n, ^-n,
                 ^{n}, ^(n), ** n or unicode superscript digits; the
                 exponent must be an integer; in plain text a whole digit
                 run is the exponent (10^23 is 10**23);
               - percent ("45%", "45\\%", "45 %") divides by 100;
               - integer fractions a/b and \\frac / \\dfrac with numeric
                 parts; \\sqrt{n}, \\sqrt[k]{n}, pi (\\pi, U+03C0, "pi") with
                 a numeric coefficient.
               Errors (compare "error"):
               "invalid:empty", "invalid:not_a_number" (garbage, arithmetic,
               letters, Python-only spellings like 1_000, 0x1F, inf, nan),
               "invalid:ambiguous_comma", "invalid:mixed_number" ("1 1/2" and
               "1\\frac{1}{2}": decided to refuse rather than guess 3/2 or
               read 1*1/2), "invalid:division_by_zero", "invalid:not_real"
               (\\sqrt{-4}), "invalid:unit_not_allowed" (a number followed by
               a unit; units belong to parse_quantity), and
               "refused:<command>" for a LaTeX command the number parser does
               not handle (\\ln, \\infty, \\overline), as parse_latex does.
               sympy_gives records what sympify() does with the raw text,
               which is the naive Python route and is often a silent misread.

sig_figs       -> the number of significant figures as typed (an integer).
               The mantissa is counted (sign, thousands commas, percent sign
               and the power of ten are ignored). Leading zeros never count;
               zeros between nonzero digits count; trailing zeros count only
               when a decimal point is present ("1200" -> 2, "1200." -> 4).
               A lone "0" counts as 1. Exact values (fractions, \\sqrt, pi)
               have no significant figures: "invalid:not_a_decimal". Text
               parse_number refuses is refused with the same error. SymPy has
               no such operation, so these cases have no sympy_gives.
"""

import re

from sympy import SympifyError, sympify

from caselib import spec_case

TOPIC = "numeric_input"

NUMERIC = "numeric grader (not SymPy)"


def _sympify_gives(src):
    """What sympify() makes of the raw text. Its parse errors are reported by
    their underlying exception type only (the full message repeats the input
    and is cut off mid-word by run_sympy)."""

    def go():
        try:
            return sympify(src)
        except SympifyError as e:
            return f"error: SympifyError ({type(e.base_exc).__name__})"

    return go


def num(src, expected, why, same_as=None):
    """A parse_number case. `same_as` is plain decimal/e-notation text that
    must denote the same exact value; it cross-checks the hand-written answer."""
    if same_as is not None:
        assert sympify(same_as, rational=True) == sympify(expected), (src, expected)
    return spec_case(
        "parse_number", [src], expected, f"{NUMERIC}; {why}",
        sympy_gives=_sympify_gives(src),
    )


def num_err(src, kind, why):
    return spec_case(
        "parse_number", [src], kind, f"{NUMERIC}; {why}", compare="error",
        sympy_gives=_sympify_gives(src),
    )


def _count_sig(mantissa):
    """Reference sig-fig count for a plain decimal mantissa (digits, one
    optional point). Used only to cross-check the hand-written answers."""
    assert re.fullmatch(r"\d*\.?\d*", mantissa) and re.search(r"\d", mantissa), mantissa
    digits = mantissa.replace(".", "").lstrip("0")
    if not digits:
        return 1  # the value zero
    if "." not in mantissa:
        digits = digits.rstrip("0")
    return len(digits)


def sf(src, n, why, mantissa):
    """A sig_figs case; `mantissa` is the digits the rule counts, for the
    cross-check against _count_sig."""
    assert _count_sig(mantissa) == n, (src, mantissa, n)
    return spec_case("sig_figs", [src], str(n), f"{NUMERIC}; {why}")


def sf_err(src, kind, why):
    return spec_case("sig_figs", [src], kind, f"{NUMERIC}; {why}", compare="error")


def _plain():
    return [
        num("  42  ", "42", "surrounding whitespace is ignored"),
        num("-0.5", "-1/2", "typed decimals are exact rationals, never Floats", "-0.5"),
        num("−0.5", "-1/2", "unicode minus U+2212 is a minus sign (sympify cannot read it)"),
        num("+5", "5", "an explicit plus sign is allowed"),
        num(".5", "1/2", "a decimal may start with the point"),
        num("-.5", "-1/2", "a negative decimal may start with the point", "-.5"),
        num("5.", "5", "a trailing decimal point is allowed (it only matters for sig_figs)"),
        num("2.50", "5/2", "trailing zeros do not change the value"),
        num("0.00450", "9/2000", "leading and trailing zeros; exact", "0.00450"),
        num("3.14", "157/50", "3.14 is the decimal typed, not pi"),
    ]


def _commas():
    return [
        num("1,200", "1200", "comma thousands separator (groups of exactly 3); sympify reads a tuple"),
        num("1,234,567", "1234567", "several thousands groups"),
        num("-1,234.5", "-2469/2", "thousands separator with a decimal part", "-1234.5"),
        num("1{,}200", "1200", "LaTeX writes a thousands comma as {,}"),
        num("1,000", "1000", "decided: a 3-digit group is a thousands separator even though a "
            "European reader would take 1,000 as 1"),
        num_err("1,2", "invalid:ambiguous_comma", "decided: a group that is not 3 digits could be a "
                "decimal comma, so it is refused rather than guessed"),
        num_err("0,5", "invalid:ambiguous_comma", "European decimal comma; refused, never read as 5 or 0.5"),
        num_err("1,2345", "invalid:ambiguous_comma", "4-digit group"),
        num_err("0,125", "invalid:ambiguous_comma", "a 3-digit group after a leading 0 is a European "
                "decimal (0.125), never thousands grouping; sympify reads a tuple"),
        num_err("1234,567", "invalid:ambiguous_comma", "a 4-digit first group is not thousands grouping "
                "(1234.567 in European notation)"),
    ]


def _scientific():
    return [
        num("3.2e4", "32000", "e notation", "3.2e4"),
        num("3.2E4", "32000", "capital E"),
        num("3.2e+4", "32000", "explicit plus in the exponent"),
        num("3.2e-4", "1/3125", "negative exponent; exact", "3.2e-4"),
        num("3.2*10^4", "32000", "caret power of ten (works in the app today)"),
        num("3.2*10**4", "32000", "Python power"),
        num("3.2*10^{4}", "32000", "braced exponent after a plain *; sympify reads {4} as a set"),
        num("3.2\\times10^{4}", "32000", "LaTeX \\times with braced exponent (fails in the app today)"),
        num("3.2\\times 10^4", "32000", "LaTeX \\times with a space and bare exponent"),
        num("3.2 \\times 10 ^ { 4 }", "32000", "spaces everywhere inside the power of ten"),
        num("3.2×10^4", "32000", "unicode multiplication sign U+00D7"),
        num("3.2 x 10^4", "32000", "letter x used as a times sign between mantissa and 10^n"),
        num("3.2 X 10^4", "32000", "capital X as a times sign"),
        num("1.6x10^-19", "1/6250000000000000000", "x with no spaces and an unbraced negative "
            "exponent, as students type it", "1.6e-19"),
        num("2.5*10^(-3)", "1/400", "parenthesised exponent, as calculators write it; "
            "sympify gives a Float", "2.5e-3"),
        num("1.5 \\cdot 10^{3}", "1500", "LaTeX \\cdot"),
        num("1.5·10^3", "1500", "middle dot U+00B7"),
        num("6.02\\times10^{23}", "602000000000000000000000", "Avogadro-sized exponent stays exact", "6.02e23"),
        num("−1.6×10^{−19}", "-1/6250000000000000000", "unicode minus in the sign and the exponent", "-1.6e-19"),
        num("3.2×10⁴", "32000", "unicode superscript exponent digits"),
        num("10^{-3}", "1/1000", "a bare power of ten"),
        num("10^23", "100000000000000000000000", "plain text: the whole digit run is the exponent "
            "(never 10^2 times 3)"),
        num_err("3.2\\times10^{4.5}", "invalid:not_a_number", "a power of ten needs an integer exponent"),
        num_err("3.2e", "invalid:not_a_number", "e with no exponent"),
    ]


def _percent():
    return [
        num("45%", "9/20", "percent divides by 100"),
        num("45\\%", "9/20", "LaTeX escaped percent (fails in the app today)"),
        num("45 %", "9/20", "space before the percent sign"),
        num("12.5%", "1/8", "decimal percent; exact"),
    ]


def _fractions():
    return [
        num("3/4", "3/4", "integer fraction"),
        num("-3/4", "-3/4", "negative fraction"),
        num("6/8", "3/4", "the value, not the form (form checks are separate)"),
        num("\\frac{3}{4}", "3/4", "LaTeX fraction"),
        num("-\\frac{3}{4}", "-3/4", "sign before \\frac"),
        num("\\frac{-3}{4}", "-3/4", "sign inside the numerator"),
        num("\\dfrac{1}{2}", "1/2", "\\dfrac is read as \\frac"),
        num_err("1/0", "invalid:division_by_zero", "refused, never zoo"),
        num_err("1 1/2", "invalid:mixed_number", "decided: a mixed number is refused rather than "
                "guessed as 3/2; the student is asked to write 3/2 or 1.5"),
        num_err("-2 3/4", "invalid:mixed_number", "negative mixed number (is it -11/4 or -5/4?)"),
        num_err("1\\frac{1}{2}", "invalid:mixed_number", "LaTeX mixed number; implicit "
                "multiplication would silently read it as 1/2"),
    ]


def _radicals_and_pi():
    return [
        num("\\sqrt{2}", "sqrt(2)", "exact square root"),
        num("\\sqrt{8}", "2*sqrt(2)", "the value is exact; SymPy's canonical form pulls out the square"),
        num("2\\sqrt{3}", "2*sqrt(3)", "coefficient times a root"),
        num("\\frac{1}{\\sqrt{2}}", "sqrt(2)/2", "SymPy's canonical form rationalises the denominator"),
        num("\\sqrt[3]{2}", "2**(1/3)", "cube root"),
        num_err("\\sqrt{-4}", "invalid:not_real", "a numeric answer is a real number"),
        num("2\\pi", "2*pi", "LaTeX pi with a coefficient"),
        num("\\frac{3\\pi}{4}", "3*pi/4", "pi inside a fraction"),
        num("2π", "2*pi", "unicode pi U+03C0"),
        num("2pi", "2*pi", "plain-text pi"),
    ]


def _refused():
    return [
        num_err("", "invalid:empty", "nothing typed"),
        num_err("   ", "invalid:empty", "only whitespace"),
        num_err("abc", "invalid:not_a_number", "letters; sympify makes a Symbol"),
        num_err("1.2.3", "invalid:not_a_number", "two decimal points"),
        num_err("--5", "invalid:not_a_number", "double sign; sympify silently gives 5"),
        num_err("4 2", "invalid:not_a_number", "two numbers separated by a space are neither 42 nor 4*2"),
        num_err("2+3", "invalid:not_a_number", "a numeric answer is one number, not arithmetic "
                "(sympify evaluates it to 5)"),
        num_err("2 x 3", "invalid:not_a_number", "x is a times sign only before a power of ten; "
                "2 x 3 is arithmetic, never silently 6"),
        num_err("1_000", "invalid:not_a_number", "Python digit grouping; sympify silently gives 1000"),
        num_err("0x1F", "invalid:not_a_number", "hexadecimal; sympify silently gives 31"),
        num_err("inf", "invalid:not_a_number", "Python float spelling of infinity"),
        num_err("nan", "invalid:not_a_number", "Python float spelling of not-a-number; sympify gives nan"),
        num_err("\\infty", "refused:\\infty", "infinity is not a numeric answer; refused by name"),
        num_err("\\ln 2", "refused:\\ln", "functions belong to expression answers; refused by name"),
        num_err("0.\\overline{3}", "refused:\\overline", "repeating-decimal bar is not supported; "
                "refused by name, never read as conjugate"),
        num_err("12 m/s", "invalid:unit_not_allowed", "decided: a unit after the number is refused "
                "by the numeric grader, never dropped; quantity items use parse_quantity"),
    ]


def _sig_figs():
    return [
        sf("0.00450", 3, "leading zeros never count; the trailing zero after the point does", "0.00450"),
        sf("1200", 2, "trailing zeros without a decimal point do not count", "1200"),
        sf("1200.", 4, "a trailing decimal point makes the trailing zeros significant", "1200."),
        sf("1.200e3", 4, "count the mantissa of e notation", "1.200"),
        sf("1.2\\times10^{3}", 2, "count the mantissa of \\times notation", "1.2"),
        sf("3.20×10^4", 3, "trailing zero in the mantissa counts", "3.20"),
        sf("101", 3, "a zero between nonzero digits counts", "101"),
        sf("5.0e-3", 2, "negative exponent; the trailing zero of the mantissa counts", "5.0"),
        sf("10.0", 3, "zeros after the point count, and so the zero before it", "10.0"),
        sf("0.10", 2, "leading zero no, trailing zero yes", "0.10"),
        sf(".5", 1, "leading-point decimal", ".5"),
        sf("-0.0450", 3, "the sign is ignored", "0.0450"),
        sf("1,200", 2, "thousands commas are ignored", "1200"),
        sf("12.5%", 3, "count the digits before the percent sign", "12.5"),
        sf("0", 1, "decided: a lone zero has one significant figure", "0"),
        sf_err("3/4", "invalid:not_a_decimal", "a fraction is exact and has no significant figures"),
        sf_err("\\sqrt{2}", "invalid:not_a_decimal", "an exact root has no significant figures"),
        sf_err("2\\pi", "invalid:not_a_decimal", "an exact multiple of pi has no significant figures"),
        sf_err("1,2", "invalid:ambiguous_comma", "same refusal as parse_number"),
        sf_err("abc", "invalid:not_a_number", "same refusal as parse_number"),
    ]


def generate():
    return (
        _plain()
        + _commas()
        + _scientific()
        + _percent()
        + _fractions()
        + _radicals_and_pi()
        + _refused()
        + _sig_figs()
    )
