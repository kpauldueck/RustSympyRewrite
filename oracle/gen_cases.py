"""Generate golden test cases from SymPy.

Run with the pinned SymPy (see requirements.txt):

    python -m venv .venv && .venv/bin/pip install -r oracle/requirements.txt
    PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python .venv/bin/python oracle/gen_cases.py

(CI sets the same environment.) Each topic starts from cleared caches and a
seeded RNG, so a topic's answers do not depend on what ran before it.

Writes one JSON file per topic into oracle/cases/. The Rust conformance
runner (crates/rsympy/tests/oracle.rs) reads those files; Rust CI never needs
Python. Do not edit the JSON by hand -- change this script and regenerate.

The case format is documented in caselib.py. rsympy copies SymPy 1:1, so a
case passes only when rsympy's str() of the answer equals SymPy's exactly
(compare "exact"), or when rsympy raises the same exception class SymPy
raised (compare "error"). There is no "close enough".

Topics live in two places: the general ones below, and one module per topic
in oracle/topics/ (each defines TOPIC and generate()).
"""

import importlib
import json
import pathlib
import random

import sympy as sp
from sympy import (
    Matrix, Rational, S, apart, cancel, cos, diff, exp, expand,
    factor, factorint, gcd, integrate, lcm, limit, log, oo, pi, series,
    simplify, sin, solveset, sqrt, symbols, tan, together, trigsimp,
)

from caselib import SOURCES, case

PINNED = "1.14.0"
assert sp.__version__ == PINNED, f"need sympy=={PINNED}, have {sp.__version__}"
from sympy.external.gmpy import GROUND_TYPES  # noqa: E402

assert GROUND_TYPES == "python", "set SYMPY_GROUND_TYPES=python (gmpy2/flint change some outputs)"

x, y, z, t = symbols("x y z t")
n = symbols("n", integer=True)
a = symbols("a", positive=True)

OUT = pathlib.Path(__file__).parent / "cases"


def numbers():
    """Ops: sympify [text] = sympify(text); expand [text] = expand(sympify(text));
    gcd/lcm [m, k] = gcd(m, k) / lcm(m, k); factorint [m] = factorint(m) as a
    Dict; evalf [e, n] = e.evalf(n)."""
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
    out = [case("sympify", [s], sp.sympify(s)) for s in srcs]
    # SymPy leaves complex products unexpanded; expand() multiplies them out.
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
    """Ops: expand/factor/cancel/together [e] = f(e); apart [e, x] = apart(e, x);
    gcd [a, b] = gcd(a, b); subs [e, x, v] = e.subs(x, v)."""
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
    """Ops: solveset [e, x, domain] = solveset(e, x, domain), domain being
    S.Reals or S.Complexes (printed "Reals"/"Complexes"); linsolve [eqs, syms]
    = linsolve(list(eqs), *syms)."""
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
        out.append(case("solveset", [e, x, "Reals"], solveset(e, x, S.Reals)))
    for e in [sin(x) - S.Half, 2 * cos(x) + 1]:
        out.append(case("solveset", [e, x, "Reals"], solveset(e, x, S.Reals)))
    for e in [x**2 + 1, x**2 + 2 * x + 5]:
        out.append(
            case("solveset", [e, x, "Complexes"], solveset(e, x, S.Complexes))
        )
    for ineq in [2 * x - 3 > 5, x**2 - 4 < 0, (x - 1) / (x + 2) >= 0, abs(x - 1) <= 3]:
        out.append(case("solveset", [ineq, x, "Reals"], solveset(ineq, x, S.Reals)))
    system = [x + y - 5, 2 * x - y - 1]
    out.append(case("linsolve", [sp.Tuple(*system), sp.Tuple(x, y)], sp.linsolve(system, x, y)))
    return out


def calculus():
    """Ops: diff [e, x] or [e, x, k] = diff(e, x[, k]); limit [e, x, x0, dir] =
    limit(e, x, x0, dir); integrate [e, x] = integrate(e, x), and
    integrate [e, x, lo, hi] = integrate(e, (x, lo, hi)); series [e, x, x0, n]
    = series(e, x, x0, n); summation [e, k, lo, hi] = summation(e, (k, lo, hi))."""
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
        out.append(case("integrate", [e, x], integrate(e, x)))
    for e, lo, hi in [(x**2, 0, 3), (sin(x), 0, pi), (exp(-x), 0, oo), (1 / x**2, 1, oo)]:
        out.append(case("integrate", [e, x, lo, hi], integrate(e, (x, lo, hi))))
    for e in [exp(x), sin(x), log(1 + x), 1 / (1 - x)]:
        out.append(case("series", [e, x, 0, 6], series(e, x, 0, 6)))
    out.append(case("summation", [n**2, n, 1, 10], sp.summation(n**2, (n, 1, 10))))
    out.append(case("summation", [Rational(1, 2) ** n, n, 0, oo], sp.summation(Rational(1, 2) ** n, (n, 0, oo))))
    return out


def simplification():
    """Ops: simplify/radsimp/trigsimp [e] = f(e); logcombine/expand_log [e] =
    f(e, **options), options being {"force": True}."""
    out = []
    for e in [
        sin(x) ** 2 + cos(x) ** 2,
        (x**2 + 2 * x + 1) / (x + 1),
        sp.sec(x) ** 2 - 1,
        2 * sin(x) * cos(x),
        exp(x) * exp(y),
        (1 - cos(x) ** 2) / sin(x),
    ]:
        out.append(case("simplify", [e], simplify(e)))
    for e in [1 / sqrt(2), 1 / (1 + sqrt(2)), 3 / (sqrt(5) - sqrt(2))]:
        out.append(case("radsimp", [e], sp.radsimp(e)))
    # force=True skips the positivity checks; with these assumption-free
    # symbols both calls would otherwise return the input unchanged.
    force = {"force": True}
    for e in [log(x) + log(y), 2 * log(x) - log(y)]:
        out.append(case("logcombine", [e], sp.logcombine(e, **force), options=dict(force)))
    for e in [log(x * y), log(x**2 / y)]:
        out.append(case("expand_log", [e], sp.expand_log(e, **force), options=dict(force)))
    out.append(case("trigsimp", [sin(x) / cos(x)], trigsimp(sin(x) / cos(x))))
    return out


def matrices():
    """Ops on a Matrix M: det = M.det(); inv = M.inv(); rref = M.rref()[0] (the
    reduced matrix, without the pivot tuple); rank = M.rank(); eigenvals =
    M.eigenvals() as a Dict."""
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
        out.append(case("eigenvals", [e], sp.Dict(e.eigenvals())))
    return out


TOPICS = {
    "numbers": numbers,
    "algebra": algebra,
    "equations": equations,
    "calculus": calculus,
    "simplify": simplification,
    "matrices": matrices,
}


def topic_modules():
    """Topic modules in oracle/topics/, sorted by file name."""
    here = pathlib.Path(__file__).parent / "topics"
    for path in sorted(here.glob("[!_]*.py")):
        mod = importlib.import_module(f"topics.{path.stem}")
        yield mod.TOPIC, mod.generate


def fresh_state():
    """Reset what makes SymPy's answers depend on history: its caches (incl.
    the factorint cache) and all three random streams it draws from: Python's
    global `random`, sympy.core.random.rng (sample points in Expr.equals,
    random_complex_number, ...) and the assumptions shuffler (sympy's seed()
    resets both of SymPy's own)."""
    sp.core.cache.clear_cache()
    from sympy.core.random import seed
    from sympy.ntheory import factor_

    factor_.factor_cache.clear()
    random.seed(0)
    seed(0)


COMPARES = ("exact", "error")


def check_schema(c):
    """Catch cases built by hand (not through caselib) that miss a field."""
    for key in ("op", "args", "expected", "compare", "source"):
        assert key in c, f"{c.get('id')}: missing {key}"
    assert c["compare"] in COMPARES, f"{c['id']}: compare {c['compare']!r}"
    assert c["source"] in SOURCES, f"{c['id']}: source {c['source']!r}"
    if c["compare"] == "error":
        assert c["expected"].isidentifier(), f"{c['id']}: error cases expect an exception class name"
    for gone in ("profile", "sympy_gives"):
        assert gone not in c, f"{c['id']}: {gone} is no longer part of the format"


def check_consistent(by_topic):
    """The same call must have one answer wherever it appears."""
    seen = {}
    clashes = []
    for cases in by_topic.values():
        for c in cases:
            key = json.dumps([c["op"], c["args"], c.get("options"), c.get("assumptions")], sort_keys=True)
            if key in seen and seen[key]["expected"] != c["expected"]:
                clashes.append(f"{seen[key]['id']} vs {c['id']}: {seen[key]['expected']!r} != {c['expected']!r}")
            seen.setdefault(key, c)
    assert not clashes, "contradictory cases:\n  " + "\n  ".join(clashes)


def main():
    OUT.mkdir(exist_ok=True)
    generators = dict(TOPICS)
    for topic, gen in topic_modules():
        assert topic not in generators, f"duplicate topic {topic}"
        generators[topic] = gen
    stale = {p.stem for p in OUT.glob("*.json")} - set(generators)
    assert not stale, f"cases with no generator: {sorted(stale)}"
    by_topic = {}
    for topic, gen in generators.items():
        fresh_state()
        cases = gen()
        for i, c in enumerate(cases):
            c["id"] = f"{topic}-{i:03d}"
            check_schema(c)
        by_topic[topic] = cases
    check_consistent(by_topic)
    total = 0
    for topic, cases in by_topic.items():
        doc = {"sympy_version": PINNED, "topic": topic, "cases": cases}
        (OUT / f"{topic}.json").write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n")
        total += len(cases)
        print(f"{topic:16s} {len(cases):4d}")
    print(f"{'total':16s} {total:4d}")


if __name__ == "__main__":
    main()
