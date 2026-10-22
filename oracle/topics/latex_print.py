"""SymPy parity cases for printing answer keys as LaTeX (op "latex").

op "latex" is `sympy.latex(sympy.sympify(args[0]), **options)`:
  - args[0] is an expression in SymPy syntax, read with sympify, so it
    evaluates the way SymPy does before printing ((2*x + 1)/3 becomes
    2*x/3 + 1/3, log(x, 2) becomes log(x)/log(2), log(8, 2) becomes 3);
  - options, when present, are latex()'s own keyword arguments; the only one
    used here is ln_notation=True, which prints natural log as \\ln;
  - `expected` is the LaTeX string latex() returns.

The homework app displays natural log as \\ln and a quotient log(a)/log(b) as
\\log_{b} a through its own LatexPrinter subclass. That printer is app code,
not SymPy, so it has no cases here; the ln_notation cases cover the part of
that display SymPy itself can produce.
"""

import sympy as sp

from caselib import case

TOPIC = "latex_print"
OP = "latex"
LN = {"ln_notation": True}


def _sympy(src, note=None, options=None):
    return case(OP, [src], sp.latex(sp.sympify(src), **(options or {})), options=options, note=note)


def _ln(src, note=None):
    return _sympy(src, note=note, options=LN)


def fractions():
    return [
        _sympy("-3/4"),
        _sympy("3*x/4"),
        _sympy("(x + 1)/(x - 1)"),
        _sympy("x**(-2)", note="A negative power prints as a fraction."),
        _sympy("(2*x + 1)/3", note="SymPy distributes 1/3 over the sum at construction, so the key prints as two fractions."),
        _sympy("-(x + 1)/2", note="SymPy distributes -1/2 over the sum at construction."),
    ]


def radicals():
    return [
        _sympy("sqrt(8)", note="SymPy simplifies the radical at construction."),
        _sympy("(1 + sqrt(5))/2", note="SymPy distributes 1/2, so the golden ratio prints as two fractions."),
        _sympy("x**(1/3)"),
        _sympy("1/sqrt(x)"),
        _sympy("x**(3/2)", note="Only unit-numerator rational powers print as radicals."),
        _sympy("sqrt(2*x)", note="SymPy splits off the numeric factor: sqrt(2)*sqrt(x)."),
    ]


def pi_multiples():
    return [
        _sympy("pi/6"),
        _sympy("2*pi"),
        _sympy("3*pi/4"),
        _sympy("4*pi*r**3/3"),
    ]


def logs_and_exponentials():
    return [
        _sympy("log(x)", note="By default SymPy prints natural log as \\log; see the ln_notation cases for \\ln."),
        _sympy("log(x, 2)", note="SymPy stores log(x, 2) as log(x)/log(2) and prints the quotient."),
        _sympy("log(8, 2)", note="SymPy evaluates an exact integer logarithm before printing."),
        _sympy("log(x, 10)"),
        _sympy("log(x)**2", note="The power goes after the argument, not on the function name."),
        _sympy("exp(-x)"),
        _sympy("400*(3/4)**t"),
        _sympy("400*exp(t*log(3/4))"),
    ]


def trig():
    return [
        _sympy("sin(x)**2", note="Powers of trig functions print on the function name."),
        _sympy("cos(x)**2 + sin(x)**2", note="Terms print in SymPy's order: sin before cos."),
        _sympy("1/tan(x)", note="SymPy does not rewrite 1/tan as cot."),
        _sympy("asin(x)", note="SymPy prints \\operatorname{asin}, not \\arcsin or \\sin^{-1}."),
        _sympy("sin(x)*cos(x)"),
        _sympy("acos(1/2)", note="SymPy evaluates the special angle before printing."),
    ]


def sets_and_intervals():
    return [
        _sympy("Interval(0, 1)"),
        _sympy("Interval.Ropen(0, 1)"),
        _sympy("Interval(-2, oo)", note="An infinite endpoint is always open."),
        _sympy("Union(Interval.open(-oo, -2), Interval.open(3, oo))"),
        _sympy("FiniteSet(-1, 2)"),
        _sympy("EmptySet"),
        _sympy("ImageSet(Lambda(n, 2*n*pi + pi/6), S.Integers)",
               note="An ImageSet over the integers built with an ordinary Symbol n; solveset's "
                    "own ImageSets use a Dummy _n, which prints the term as '2 n \\pi' instead."),
    ]


def matrices_and_piecewise():
    return [
        _sympy("Matrix([[1, 2], [3, 4]])"),
        _sympy("Matrix([[1], [2]])"),
        _sympy("Matrix([[1/2, 0], [0, 2]])"),
        _sympy("Piecewise((x, x >= 0), (-x, True))"),
        _sympy("Piecewise((x**2, x < 1), (2*x - 1, True))"),
    ]


def signs_and_ordering():
    return [
        _sympy("-x**2 + 3*x - 2", note="A negative leading term prints with a spaced minus sign."),
        _sympy("-x + 1",
               note="A two-term sum of a positive Number or NumberSymbol and a negative Number "
                    "times one factor prints the constant first (special case in "
                    "Expr.as_ordered_terms); longer sums such as -x + y + 1, and -x + sqrt(2), "
                    "keep the leading negative term."),
        _sympy("-sqrt(2)/2"),
        _sympy("3 + x**2 + x", note="Polynomial terms print by descending degree."),
        _sympy("x*y**2 + x**2*y"),
        _sympy("x*sqrt(3)", note="The numeric radical prints before the symbol."),
        _sympy("(x - 2)*(x - 3)", note="Factors print in SymPy's canonical order, not as typed."),
        _sympy("x*(x + 1)"),
        _sympy("2*(x + 1)**2"),
    ]


def relations_and_symbols():
    return [
        _sympy("Eq(y, 2*x + 1)"),
        _sympy("x <= 3"),
        _sympy("Ne(x, 0)"),
        _sympy("And(x > -1, x < 3)", note="A compound inequality prints with \\wedge."),
        _sympy("Abs(x - 3)"),
        _sympy("-oo"),
        _sympy("x_1"),
        _sympy("v_0x"),
        _sympy("F_net"),
        _sympy("Delta_x", note="A Greek-letter name before the underscore prints as the Greek command, and the rest as a subscript."),
    ]


def ln_notation_logs():
    return [
        _ln("log(x)"),
        _ln("log(x)**2", note="The power goes after the argument, as with \\log."),
        _ln("log(x)/2"),
        _ln("1/log(x)", note="A log only in the denominator."),
        _ln("log(10)", note="The natural log of 10 is an unevaluated log(10), not a base-10 log."),
        _ln("log(x, 2)", note="SymPy stores log(x, 2) as log(x)/log(2), so both logs print as \\ln."),
        _ln("log(x)/log(2)", note="The same tree as log(x, 2)."),
        _ln("log(x, 10)", note="Base 10 is stored as log(x)/log(10) like any other base."),
        _ln("3*log(x)/log(5)", note="A numeric coefficient over a log of the base."),
        _ln("log(x + 1, 3)"),
        _ln("log(x, b)", note="A symbolic base: log(x)/log(b)."),
        _ln("log(x, 2) + 1", note="The change-of-base quotient inside a sum."),
        _ln("log(8)/log(2)", note="Typed as a quotient, unlike log(8, 2), this stays unevaluated."),
        _ln("log(2)/log(x)", note="The symbolic log is in the denominator."),
        _ln("log(x)/(2*log(3))", note="A numeric factor and a log share the denominator."),
        _ln("x*log(y, 2)", note="A symbolic factor in front of a change-of-base quotient."),
        _ln("log(x, 1/2)", note="SymPy rewrites log(1/2) as -log(2) at construction, so the sign moves out front."),
        _ln("400*exp(t*log(3/4))", note="A natural log inside an exponent."),
        _ln("exp(x)", note="ln_notation changes only log; exp still prints as e^{x}."),
    ]


def underscore_symbol_names():
    """Multi-letter names split at the underscore: how latex() treats the prefix."""
    return [
        _sympy("Delta_y/Delta_x", note="A Greek-letter name before the underscore prints as \\Delta with a subscript."),
        _sympy("seg_AB", note="seg is neither a Greek letter nor a printer modifier, so latex() emits the bare letters seg (no command, no \\text{}) with AB as a subscript."),
        _sympy("angle_ABC", note="angle is neither a Greek letter nor a printer modifier, so latex() emits the bare letters angle (no command, no \\text{}) with ABC as a subscript."),
        _sympy("vec_v", note="SymPy turns vec into \\vec{} only as a name suffix (vvec), not as a prefix before an underscore."),
        _sympy("3*hat_i - 4*hat_j", note="SymPy turns hat into \\hat{} only as a name suffix (ihat), not as a prefix before an underscore."),
    ]


def generate():
    return (
        fractions()
        + radicals()
        + pi_multiples()
        + logs_and_exponentials()
        + trig()
        + sets_and_intervals()
        + matrices_and_piecewise()
        + signs_and_ordering()
        + relations_and_symbols()
        + ln_notation_logs()
        + underscore_symbol_names()
    )
