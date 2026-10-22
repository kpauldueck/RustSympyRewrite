"""Printing answer keys as LaTeX (op "latex").

args[0] is an expression in SymPy syntax (read with sympify, so it evaluates
the way SymPy does: (2*x + 1)/3 becomes 2*x/3 + 1/3 before printing).
`expected` is the LaTeX string itself.

SymPy-sourced cases record sympy.latex() for typical answer keys. Spec cases
with options {"conventions": "course"} record the homework app's display
conventions, which differ from SymPy's printer:
  - natural log prints as \\ln (the shape of SymPy's ln_notation=True);
  - a quotient log(a)/log(b) prints as \\log_{b}{\\left(a \\right)};
  - base 10 prints as a bare \\log{\\left(a \\right)}, matching parse_latex,
    where an unsubscripted \\log is base 10 (so keys round-trip);
  - the parse_latex vocabulary symbols print back as the LaTeX they came from.
"""

import sympy as sp

from caselib import case, run_sympy

TOPIC = "latex_print"
OP = "latex"
COURSE = {"conventions": "course"}


def _sympy(src, note=None):
    return case(OP, [src], sp.latex(sp.sympify(src)), note=note)


def _course(src, expected, note):
    """Spec case for the course conventions. Built by hand because spec_case
    would sympify `expected`, and here `expected` is a LaTeX string."""
    return {
        "op": OP,
        "args": [src],
        "options": COURSE,
        "expected": expected,
        "compare": "exact",
        "source": "spec",
        "profile": "strict",
        "note": note,
        "sympy_gives": run_sympy(lambda: sp.latex(sp.sympify(src))),
    }


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
        _sympy("log(x)", note="SymPy prints natural log as \\log; see the course cases for \\ln."),
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
               note="The shape solveset gives for a trig equation's solutions."),
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
        _sympy("-x + 1", note="SymPy puts the positive constant first."),
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
        _sympy("Delta_x", note="SymPy subscripts the part after the underscore; see the course case."),
    ]


def course_logs():
    natural = "Course convention: natural log prints as \\ln (the shape of SymPy's ln_notation=True)."
    base_b = "Course convention: log(a)/log(b) prints as \\log_{b}{\\left(a \\right)}."
    return [
        _course("log(x)", r"\ln{\left(x \right)}", natural),
        _course("log(x)**2", r"\ln{\left(x \right)}^{2}", natural),
        _course("log(x)/2", r"\frac{\ln{\left(x \right)}}{2}", natural),
        _course("1/log(x)", r"\frac{1}{\ln{\left(x \right)}}",
                "Near miss: a log only in the denominator is not a change of base; it stays a natural log."),
        _course("log(10)", r"\ln{\left(10 \right)}",
                "Near miss: the natural log of 10 is not the base-10 log, so it prints \\ln, never a bare \\log."),
        _course("log(x, 2)", r"\log_{2}{\left(x \right)}", base_b),
        _course("log(x)/log(2)", r"\log_{2}{\left(x \right)}",
                "A typed quotient of logs is the same expression as log(x, 2), so it prints the same way."),
        _course("log(x, 10)", r"\log{\left(x \right)}",
                "Decision: base 10 prints as a bare \\log, because parse_latex reads a bare \\log as base 10"
                " and natural log always prints as \\ln."),
        _course("3*log(x)/log(5)", r"3 \log_{5}{\left(x \right)}",
                "A numeric coefficient stays in front of the \\log_{b} term."),
        _course("log(x + 1, 3)", r"\log_{3}{\left(x + 1 \right)}", base_b),
        _course("log(x, b)", r"\log_{b}{\left(x \right)}", "A symbolic base is written as the subscript."),
        _course("log(x, 2) + 1", r"\log_{2}{\left(x \right)} + 1", "The \\log_{b} form works inside a sum."),
        _course("log(8)/log(2)", r"\log_{2}{\left(8 \right)}",
                "An unevaluated quotient log(8)/log(2) prints as \\log_{2} 8; printing never evaluates."),
        _course("log(2)/log(x)", r"\log_{x}{\left(2 \right)}",
                "Any quotient of two logs reads as a change of base, whichever is symbolic."),
        _course("log(x)/(2*log(3))", r"\frac{\log_{3}{\left(x \right)}}{2}",
                "The other factors of the quotient stay as a fraction around the \\log_{b} term."),
        _course("x*log(y, 2)", r"x \log_{2}{\left(y \right)}",
                "A symbolic factor stays in front of the \\log_{b} term."),
        _course("log(x, 1/2)", r"- \log_{2}{\left(x \right)}",
                "SymPy rewrites log(1/2) as -log(2) at construction, so base 1/2 displays as -\\log_{2}."),
        _course("400*exp(t*log(3/4))", r"400 e^{t \ln{\left(\frac{3}{4} \right)}}",
                "Natural log inside an exponent also prints as \\ln."),
        _course("exp(x)", r"e^{x}", "Course conventions change only logs and vocabulary symbols; exp prints as e^{x}."),
    ]


def course_symbols():
    note = ("Decision: course printing inverts the parse_latex vocabulary so a key round-trips"
            " through LaTeX; SymPy prints the raw symbol name.")
    return [
        _course("Delta_y/Delta_x", r"\frac{\Delta y}{\Delta x}", note),
        _course("seg_AB", r"\overline{AB}", note),
        _course("angle_ABC", r"\angle ABC", note),
        _course("vec_v", r"\vec{v}", note),
        _course("3*hat_i - 4*hat_j", r"3 \hat{i} - 4 \hat{j}", note),
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
        + course_logs()
        + course_symbols()
    )
