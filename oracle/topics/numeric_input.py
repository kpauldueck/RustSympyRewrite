"""Text typed into a numeric answer box, read by SymPy (ops "sympify" and
"parse_latex"), as SymPy-parity cases.

The homework app's numeric grader does not call SymPy (it uses ast, Decimal
and Fraction). These cases record what SymPy's own readers do with the same
student text. That is the naive route, and it often misreads the number
without an error: a comma makes a Python tuple, a typed decimal is a Float,
Python-only spellings (1_000, 0x1F, --5) are accepted and 1/0 is zoo. These
are the inputs where a port's tokenizer and number handling are most likely to
diverge.

args[0] is the text exactly as typed, including spaces and unicode. Each case
is one SymPy call:

  sympify      sympify(args[0]) with no keyword arguments, for every input
               that has no backslash. This includes inputs with braces or
               unicode signs. One LaTeX input (3.2\\times10^{4}) also gets a
               sympify case, to record that sympify does not read LaTeX.
  parse_latex  parse_latex(args[0], backend="lark") from sympy.parsing.latex,
               the backend the app uses for LaTeX; options always records
               {"backend": "lark"}. Every input in LaTeX notation (one that
               contains a backslash or a brace) gets this case. An input with
               braces and no backslash therefore gets both cases.

The expected answer is SymPy's str() of the result, or the exception's class
name when the call raises: SympifyError when sympify cannot tokenize or parse
the text, TypeError when the parsed Python cannot be evaluated (sympify does
not wrap it), and lark's UnexpectedCharacters when the text is outside the
Lark grammar.
"""

from sympy import sympify
from sympy.parsing.latex import parse_latex

from caselib import outcome

TOPIC = "numeric_input"
BACKEND = "lark"


def _sympify(src, note=None):
    return outcome("sympify", [src], lambda: sympify(src), note=note)


def _parse_latex(src, note=None):
    return outcome(
        "parse_latex", [src], lambda: parse_latex(src, backend=BACKEND),
        options={"backend": BACKEND}, note=note,
    )


def typed(src, note=None, latex_note=None):
    """The cases for one typed input, by the rule in the module docstring.

    `note` goes on the sympify case, and on the parse_latex case unless
    `latex_note` is given."""
    out = []
    if "\\" not in src:
        out.append(_sympify(src, note))
    if "\\" in src or "{" in src:
        out.append(_parse_latex(src, latex_note or note))
    return out


def _cases(*groups):
    return [c for group in groups for c in group]


def plain_decimals():
    return _cases(
        typed("  42  ", "Spaces around the number."),
        typed("-0.5", "A typed decimal is a Float."),
        typed("−0.5", "U+2212 MINUS SIGN is not a Python operator."),
        typed("+5", "An explicit unary plus."),
        typed(".5", "A decimal with no digit before the point."),
        typed("-.5", "A negative decimal with no digit before the point."),
        typed("5.", "A trailing decimal point makes a Float."),
        typed("2.50", "A trailing zero after the point."),
        typed("0.00450", "Leading and trailing zeros around the digits."),
        typed("3.14", "A decimal with no exact binary value (and not pi)."),
    )


def commas():
    return _cases(
        typed("1,200", "A thousands comma: as Python, a comma builds a tuple, not a SymPy object."),
        typed("1,234,567", "Two thousands commas."),
        typed("-1,234.5", "A thousands comma with a decimal part: the tuple holds an Integer and a Float."),
        typed("1{,}200", "LaTeX writes a thousands comma as {,}; as Python, {,} is not valid.",
              "LaTeX writes a thousands comma as {,}."),
        typed("1,000", "A European reader takes 1,000 as 1; the group 000 is the integer 0."),
        typed("1,2", "A European decimal comma."),
        typed("0,5", "A European decimal comma after 0."),
        typed("1,2345", "A 4-digit group after the comma."),
        typed("0,125", "A European 0.125 whose 3-digit group looks like thousands grouping."),
        typed("1234,567", "A European 1234.567 with a 4-digit first group."),
    )


def scientific():
    return _cases(
        typed("3.2e4", "E notation."),
        typed("3.2E4", "Capital E."),
        typed("3.2e+4", "An explicit plus in the exponent."),
        typed("3.2e-4", "A negative exponent."),
        typed("3.2*10^4", "sympify converts ^ to ** by default (convert_xor=True)."),
        typed("3.2*10**4", "Python's ** power."),
        typed("3.2*10^{4}", "As Python, {4} is a set, and a power with a set exponent raises TypeError.",
              "A braced exponent after a plain *."),
        typed(r"3.2\times10^{4}", "LaTeX \\times with a braced exponent."),
        [_sympify(r"3.2\times10^{4}", "The Python tokenizer has no backslash token, so sympify does not read LaTeX.")],
        typed(r"3.2\times 10^4", "LaTeX \\times with a space and an unbraced exponent."),
        typed(r"3.2 \times 10 ^ { 4 }", "Spaces everywhere inside the power of ten."),
        typed("3.2×10^4", "U+00D7 MULTIPLICATION SIGN is not a Python operator."),
        typed("3.2 x 10^4", "The letter x as a times sign; sympify's default transformations have no "
              "implicit multiplication."),
        typed("3.2 X 10^4", "Capital X as a times sign."),
        typed("1.6x10^-19", "x as a times sign with no spaces and an unbraced negative exponent."),
        typed("2.5*10^(-3)", "A parenthesised exponent, as calculators write it."),
        typed(r"1.5 \cdot 10^{3}", "LaTeX \\cdot as the times sign."),
        typed("1.5·10^3", "U+00B7 MIDDLE DOT as the times sign."),
        typed(r"6.02\times10^{23}", "A large power of ten; the Float prints in e notation."),
        typed("−1.6×10^{−19}", "U+2212 minus signs in the mantissa and the braced exponent."),
        typed("3.2×10⁴", "A unicode superscript exponent digit."),
        typed("10^{-3}", "As Python, {-3} is a set, so the power raises TypeError.",
              "A bare power of ten with a braced negative exponent."),
        typed("10^23", "The whole digit run after ^ is the exponent."),
        typed(r"3.2\times10^{4.5}", "A power of ten with a non-integer exponent."),
        typed("3.2e", "E notation with no exponent digits."),
    )


def percent():
    return _cases(
        typed("45%", "% is Python's modulo operator, here with no right operand."),
        typed(r"45\%", "LaTeX's escaped percent sign."),
        typed("45 %", "A space before the percent sign."),
        typed("12.5%", "A decimal percent."),
    )


def fractions():
    return _cases(
        typed("3/4", "Integer division gives an exact Rational."),
        typed("-3/4", "A negative fraction."),
        typed("6/8", "An unreduced fraction."),
        typed(r"\frac{3}{4}", "A LaTeX fraction of integers."),
        typed(r"-\frac{3}{4}", "The sign before \\frac."),
        typed(r"\frac{-3}{4}", "The sign inside the numerator."),
        typed(r"\dfrac{1}{2}", "\\dfrac, the display-style fraction."),
        typed("1/0", "Division by zero."),
        typed("1 1/2", "A mixed number: two numbers separated by a space."),
        typed("-2 3/4", "A negative mixed number."),
        typed(r"1\frac{1}{2}", "A LaTeX mixed number: the integer and the fraction are adjacent, an implicit product."),
    )


def radicals_and_pi():
    return _cases(
        typed(r"\sqrt{2}", "An exact square root."),
        typed(r"\sqrt{8}", "A square root with a square factor, taken out at construction."),
        typed(r"2\sqrt{3}", "A coefficient times a root."),
        typed(r"\frac{1}{\sqrt{2}}", "A root in the denominator."),
        typed(r"\sqrt[3]{2}", "A cube root."),
        typed(r"\sqrt{-4}", "The square root of a negative number."),
        typed(r"2\pi", "The Lark grammar has no \\pi command."),
        typed(r"\frac{3\pi}{4}", "\\pi inside a fraction; the Lark grammar has no \\pi command."),
        typed("2π", "U+03C0 is a valid Python name, but nothing joins the number to it."),
        typed("2pi", "A number followed by a name, with no implicit multiplication."),
    )


def not_numbers():
    """Text a numeric answer box should not take as a number."""
    return _cases(
        typed("", "Nothing typed."),
        typed("   ", "Only spaces."),
        typed("abc", "Letters become a Symbol."),
        typed("1.2.3", "Two decimal points."),
        typed("--5", "Python reads --5 as -(-5)."),
        typed("4 2", "Two numbers separated by a space."),
        typed("2+3", "Arithmetic is evaluated."),
        typed("2 x 3", "x as a times sign between two integers."),
        typed("1_000", "Python's underscore digit grouping."),
        typed("0x1F", "A Python hexadecimal literal."),
        typed("inf", "Python's float spelling of infinity; inf is not a SymPy name."),
        typed("nan", "Python's float spelling of not-a-number; nan is a SymPy name."),
        typed(r"\infty", "LaTeX infinity."),
        typed(r"\ln 2", "A function applied to a number."),
        typed(r"0.\overline{3}", "A repeating-decimal bar; the Lark grammar reads \\overline as complex conjugation."),
        typed("12 m/s", "A number followed by a unit."),
    )


def generate():
    return (
        plain_decimals()
        + commas()
        + scientific()
        + percent()
        + fractions()
        + radicals_and_pi()
        + not_numbers()
    )
