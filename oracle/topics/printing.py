"""SymPy's printers on everyday expressions, as SymPy-parity cases.

Every case records `expected` = sympy.sstr() of the call's result and
`expected_latex` = sympy.latex() of it (see caselib.py). This topic exists for
those two printed forms. Each case is one SymPy call:

  sympify     args [text]: sympy.sympify(text), no options. text is SymPy's
              own sstr() of an expression built in Python (x**2/2 + 3*x - 1,
              Interval(0, 1, right_open=True), ...). So each case checks that
              the printed form reads back to an expression that prints the same,
              including the printer's term order, its fraction and sign layout,
              and Interval.Ropen. The symbols are plain Symbols with no
              assumptions.
  parse_expr  args [text]: parse_expr(text, transformations=TRANSFORMATIONS)
              from sympy.parsing.sympy_parser, for plain text as a student
              types it, with no local_dict. TRANSFORMATIONS is
              standard_transformations + (implicit_multiplication_application,
              convert_xor). options records it as
              {"transformations": [function names in order]}, the same way
              units.json does. The answers depend on these transformations.
              Without convert_xor, ^ is Python's XOR ("2^10" gives 8). Without
              implicit multiplication and application, "2x", "3(x - 1)" and
              "sin x" do not parse.
"""

import sympy as sp
from sympy.parsing.sympy_parser import (
    convert_xor, implicit_multiplication_application, parse_expr,
    standard_transformations,
)

from caselib import case, outcome

TOPIC = "printing"

TRANSFORMATIONS = standard_transformations + (implicit_multiplication_application, convert_xor)
PARSE_EXPR_OPTIONS = {"transformations": [f.__name__ for f in TRANSFORMATIONS]}

x, y = sp.symbols("x y")


def round_trips():
    out = []
    for e in [
        x**2 / 2 + 3 * x - 1,
        sp.sqrt(x) / (x + 1),
        sp.sin(x) ** 2,
        sp.Rational(-3, 4) * x * y,
        sp.exp(-x) * sp.log(x),
        sp.Interval(0, 1, right_open=True),
    ]:
        src = sp.sstr(e)
        out.append(case("sympify", [src], sp.sympify(src)))
    return out


def typed():
    out = []
    for src, note in [
        ("x^2 + 2x + 1", "convert_xor reads ^ as a power, and implicit multiplication reads 2x as 2*x."),
        ("3(x - 1)", "Implicit multiplication reads 3(x - 1) as 3*(x - 1). SymPy then distributes the number over the sum."),
        ("sin x + cos(2x)", "Implicit application reads sin x as sin(x)."),
        ("2^10", "Without convert_xor, ^ is Python's XOR and 2^10 gives 8."),
        ("sqrt(16)x", "Implicit multiplication after a function call. sqrt(16) evaluates to 4."),
    ]:
        out.append(outcome(
            "parse_expr", [src],
            lambda src=src: parse_expr(src, transformations=TRANSFORMATIONS),
            options=PARSE_EXPR_OPTIONS, note=note,
        ))
    return out


def generate():
    return round_trips() + typed()
