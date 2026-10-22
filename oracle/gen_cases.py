"""Generate golden test cases from SymPy.

Run with the pinned SymPy (see requirements.txt):

    python -m venv .venv && .venv/bin/pip install -r oracle/requirements.txt
    .venv/bin/python oracle/gen_cases.py

Writes one JSON file per topic into oracle/cases/. The Rust conformance
runner (crates/rsympy/tests/oracle.rs) reads those files; Rust CI never needs
Python. Do not edit the JSON by hand -- change this script and regenerate.

Compare modes:
  exact          our canonical str() must equal SymPy's
  equivalent     same value (numeric spot checks + simplify(a - b) == 0)
  set            same solution set, any order
  antiderivative d/dx(ours) must be equivalent to the integrand
"""

import json
import pathlib

import sympy as sp
from sympy import (
    Interval, Matrix, Rational, S, apart, cancel, cos, diff, exp, expand,
    factor, factorint, gcd, integrate, lcm, limit, log, oo, pi, series,
    simplify, sin, solveset, sqrt, symbols, tan, together, trigsimp,
)

PINNED = "1.14.0"
assert sp.__version__ == PINNED, f"need sympy=={PINNED}, have {sp.__version__}"

x, y, z, t = symbols("x y z t")
n = symbols("n", integer=True)
a = symbols("a", positive=True)

# Assumptions of every non-plain symbol used above. Argument strings lose
# assumptions, so each case records the ones its symbols carry.
DECLARED = {"n": {"integer": True}, "a": {"positive": True}}

OUT = pathlib.Path(__file__).parent / "cases"


def case(op, args, result, compare="exact", **extra):
    names = set()
    for e in args:
        if isinstance(e, sp.Basic):
            names |= {str(s) for s in e.free_symbols}
    assumptions = {k: v for k, v in DECLARED.items() if k in names}
    return {
        "op": op,
        "args": [sp.sstr(e) if isinstance(e, sp.Basic) else str(e) for e in args],
        "expected": sp.sstr(result),
        "expected_latex": sp.latex(result),
        "compare": compare,
        **({"assumptions": assumptions} if assumptions else {}),
        **extra,
    }


def numbers():
    # Inputs are strings: building them in Python would evaluate them before
    # we could record the question.
    srcs = [
        "1/3 + 1/6",
        "-7/4 * 8/21",
        "2**100",
        "(-3)**5",
        "(2/3)**-3",
        "sqrt(8)",
        "sqrt(12) + sqrt(27)",
        "sqrt(9/4)",
        "Abs(-5 + 12*I)",
        "100 % 7",
        "factorial(20)",
        "binomial(30, 12)",
    ]
    out = [case("eval", [s], sp.sympify(s)) for s in srcs]
    # SymPy leaves complex products unexpanded; expand() is what a student wants.
    for s in ["(1 + I)*(2 - 3*I)", "(3 + 4*I)/(1 - 2*I)"]:
        out.append(case("expand", [s], expand(sp.sympify(s))))
    # Integer functions.
    for m, k in [(84, 36), (2**40, 6**20), (-12, 18)]:
        out.append(case("gcd", [m, k], gcd(m, k)))
        out.append(case("lcm", [m, k], lcm(m, k)))
    for m in [360, 2**31 - 1, 600851475143, 10**12 + 39]:
        out.append(case("factorint", [m], sp.Dict(factorint(m))))
    for e, prec in [(pi, 30), (sqrt(2), 50), (exp(1), 20), (Rational(1, 7), 25)]:
        out.append(case("evalf", [e, prec], e.evalf(prec)))
    return out


def algebra():
    out = []
    for e in [
        (x + 1) ** 2,
        (x - y) ** 3,
        (2 * x + 3) * (x - 5),
        (x + y + z) ** 2,
        (x**2 - 1) * (x**2 + x + 1),
        (sqrt(2) + x) ** 2,
    ]:
        out.append(case("expand", [e], expand(e)))
    for e in [
        x**2 - 5 * x + 6,
        x**3 - 1,
        4 * x**2 - 9 * y**2,
        x**4 - 16,
        6 * x**2 + 11 * x - 35,
        x**3 + 3 * x**2 * y + 3 * x * y**2 + y**3,
        2 * x**3 - 2 * x,
        x**2 + 1,
    ]:
        out.append(case("factor", [e], factor(e)))
    for e in [
        (x**2 - 1) / (x - 1),
        (x**2 + 5 * x + 6) / (x**2 + x - 6),
        (x**3 - 8) / (x**2 - 4),
    ]:
        out.append(case("cancel", [e], cancel(e)))
    for e in [1 / x + 1 / y, 1 / (x + 1) - 1 / (x - 1)]:
        out.append(case("together", [e], together(e)))
    for e in [
        1 / (x**2 - 1),
        (3 * x + 5) / ((x + 1) * (x + 2)),
        (x**2 + 1) / (x * (x - 1) ** 2),
    ]:
        out.append(case("apart", [e, x], apart(e, x)))
    out.append(case("gcd", [x**2 - 1, x**2 - 2 * x + 1], gcd(x**2 - 1, x**2 - 2 * x + 1)))
    out.append(case("subs", [x**2 + 3 * x, x, 2], (x**2 + 3 * x).subs(x, 2)))
    return out


def equations():
    out = []
    eqs = [
        2 * x + 3 - 11,
        x**2 - 5 * x + 6,
        x**2 + 4,
        x**2 - 2,
        x**3 - 6 * x**2 + 11 * x - 6,
        x**4 - 5 * x**2 + 4,
        (x + 1) / (x - 2) - 3,
        sqrt(x + 3) - x + 3,
        exp(2 * x) - 5,
        log(x, 2) - 3,
        abs(x - 3) - 5,
    ]
    for e in eqs:
        out.append(case("solveset", [e, x, "Reals"], solveset(e, x, S.Reals), compare="set"))
    for e in [sin(x) - S.Half, 2 * cos(x) + 1]:
        out.append(case("solveset", [e, x, "Reals"], solveset(e, x, S.Reals), compare="set"))
    for e in [x**2 + 1, x**2 + 2 * x + 5]:
        out.append(
            case("solveset", [e, x, "Complexes"], solveset(e, x, S.Complexes), compare="set")
        )
    for ineq in [2 * x - 3 > 5, x**2 - 4 < 0, (x - 1) / (x + 2) >= 0, abs(x - 1) <= 3]:
        out.append(case("solveset", [ineq, x, "Reals"], solveset(ineq, x, S.Reals), compare="set"))
    system = [x + y - 5, 2 * x - y - 1]
    out.append(case("linsolve", [sp.Tuple(*system), sp.Tuple(x, y)], sp.linsolve(system, x, y), compare="set"))
    return out


def calculus():
    out = []
    for e in [
        x**3 - 2 * x,
        sin(x) * cos(x),
        exp(x**2),
        log(x**2 + 1),
        x / (x + 1),
        tan(x),
        sqrt(1 - x**2),
        x**x,
        sp.asin(x),
    ]:
        out.append(case("diff", [e, x], diff(e, x)))
    out.append(case("diff", [sin(x) ** 2, x, 2], diff(sin(x) ** 2, x, 2)))
    for e, point, dirn in [
        (sin(x) / x, 0, "+-"),
        ((1 + 1 / x) ** x, oo, "-"),
        ((x**2 - 1) / (x - 1), 1, "+-"),
        (1 / x, 0, "+"),
        (1 / x, 0, "-"),
        ((3 * x**2 + 1) / (x**2 - 5), oo, "-"),
        ((1 - cos(x)) / x**2, 0, "+-"),
        (x * log(x), 0, "+"),
    ]:
        out.append(case("limit", [e, x, point, dirn], limit(e, x, point, dirn)))
    for e in [
        x**2 + 3 * x,
        1 / x,
        sin(x) * cos(x),
        x * exp(x),
        x * sin(x),
        1 / (x**2 - 1),
        1 / (1 + x**2),
        2 * x * cos(x**2),
        log(x),
        1 / cos(x) ** 2,
    ]:
        out.append(case("integrate", [e, x], integrate(e, x), compare="antiderivative"))
    for e, lo, hi in [(x**2, 0, 3), (sin(x), 0, pi), (exp(-x), 0, oo), (1 / x**2, 1, oo)]:
        out.append(case("integrate_definite", [e, x, lo, hi], integrate(e, (x, lo, hi))))
    for e in [exp(x), sin(x), log(1 + x), 1 / (1 - x)]:
        out.append(case("series", [e, x, 0, 6], series(e, x, 0, 6)))
    out.append(case("summation", [n**2, n, 1, 10], sp.summation(n**2, (n, 1, 10))))
    out.append(case("summation", [Rational(1, 2) ** n, n, 0, oo], sp.summation(Rational(1, 2) ** n, (n, 0, oo))))
    return out


def simplification():
    out = []
    for e in [
        sin(x) ** 2 + cos(x) ** 2,
        (x**2 + 2 * x + 1) / (x + 1),
        sp.sec(x) ** 2 - 1,
        2 * sin(x) * cos(x),
        exp(x) * exp(y),
        (1 - cos(x) ** 2) / sin(x),
    ]:
        out.append(case("simplify", [e], simplify(e), compare="equivalent"))
    for e in [1 / sqrt(2), 1 / (1 + sqrt(2)), 3 / (sqrt(5) - sqrt(2))]:
        out.append(case("radsimp", [e], sp.radsimp(e)))
    for e in [log(x) + log(y), 2 * log(x) - log(y)]:
        out.append(case("logcombine", [e], sp.logcombine(e, force=True)))
    for e in [log(x * y), log(x**2 / y)]:
        out.append(case("expand_log", [e], sp.expand_log(e, force=True)))
    out.append(case("trigsimp", [sin(x) / cos(x)], trigsimp(sin(x) / cos(x)), compare="equivalent"))
    return out


def matrices():
    out = []
    mats = [
        Matrix([[1, 2], [3, 4]]),
        Matrix([[2, 0, 1], [1, 3, 2], [1, 1, 2]]),
        Matrix([[x, 1], [1, x]]),
    ]
    for m in mats:
        out.append(case("det", [m], m.det()))
        out.append(case("inv", [m], m.inv()))
    m = Matrix([[1, 2, 3], [2, 4, 6], [1, 0, 1]])
    out.append(case("rref", [m], m.rref()[0]))
    out.append(case("rank", [m], S(m.rank())))
    for e in [Matrix([[2, 1], [1, 2]]), Matrix([[4, 1], [2, 3]])]:
        out.append(case("eigenvals", [e], sp.Dict(e.eigenvals()), compare="set"))
    return out


def printing():
    out = []
    for e in [
        x**2 / 2 + 3 * x - 1,
        sqrt(x) / (x + 1),
        sin(x) ** 2,
        Rational(-3, 4) * x * y,
        exp(-x) * log(x),
        Interval(0, 1, right_open=True),
    ]:
        out.append(case("print", [e], e))
    # Parsing: plain-text input as a student would type it.
    from sympy.parsing.sympy_parser import (
        convert_xor, implicit_multiplication_application, parse_expr,
        standard_transformations,
    )

    tr = standard_transformations + (implicit_multiplication_application, convert_xor)
    for src in ["x^2 + 2x + 1", "3(x - 1)", "sin x + cos(2x)", "2^10", "sqrt(16)x"]:
        out.append(case("parse", [src], parse_expr(src, transformations=tr)))
    return out


TOPICS = {
    "numbers": numbers,
    "algebra": algebra,
    "equations": equations,
    "calculus": calculus,
    "simplify": simplification,
    "matrices": matrices,
    "printing": printing,
}


def main():
    OUT.mkdir(exist_ok=True)
    total = 0
    for topic, gen in TOPICS.items():
        cases = gen()
        for i, c in enumerate(cases):
            c["id"] = f"{topic}-{i:03d}"
        doc = {"sympy_version": PINNED, "topic": topic, "cases": cases}
        (OUT / f"{topic}.json").write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n")
        total += len(cases)
        print(f"{topic:10s} {len(cases):4d}")
    print(f"{'total':10s} {total:4d}")


if __name__ == "__main__":
    main()
