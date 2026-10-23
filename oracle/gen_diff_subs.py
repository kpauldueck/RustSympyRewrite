"""Differential vectors for rsympy-core's tree walks and substitution
(`sympy.core.basic`, `sympy.core.traversal` and the `_eval_subs`,
`as_independent`, `coeff` and `is_polynomial` methods of `sympy.core.expr`,
`add`, `mul`, `power`, `numbers` and `symbol`).

    PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python python oracle/gen_diff_subs.py

Writes oracle/diff/core_subs.json, replayed by
crates/rsympy-core/tests/diff_subs.rs. Covers `Basic.subs` (one pair, dicts,
sets, lists, a single iterable SymPy object, `simultaneous=True`, `hack2`,
strings, and the ValueError/TypeError paths), every core `_eval_subs`
(Add, Mul, Pow, Number, Infinity, Symbol), `xreplace`, `has`, `has_free`,
`has_xfree`, `atoms` (with types), `free_symbols`, `is_same`,
`preorder_traversal` (with keys and `skip()`), `postorder_traversal`,
`iterargs`, `iterfreeargs`, `walk`, `Expr.as_independent`, `Expr.coeff`
(commutative and not, `right=True`), `is_polynomial` and
`is_rational_function`, on sums, products and powers of symbols with
assumptions, numbers, non-commutative symbols and undefined functions,
including the cases of SymPy's test_subs.py, test_basic.py and
test_traversal.py.

Arguments travel as `enc()` text, which the Rust side parses inside the op
(building some inputs can reach unported code). A class used as a pattern
is written `class:<Name>`, an undefined function class `undef:<enc(f(x))>`,
a Python string `pystr:<text>`. Sets (`atoms`, `free_symbols`) are recorded
as lists sorted by `(default_sort_key, enc)`.
"""

import random

from sympy import (Add, Atom, Basic, Derivative, DiracDelta, Dummy, E, Eq,
                   Expr, FiniteSet, Float, Function, I, Integer, Integral,
                   Interval, Lambda, Mul, Number, NumberSymbol, Piecewise, Pow,
                   Rational, S, Symbol, Tuple, Wild, cos, exp, log, nan, oo, pi,
                   sin, sqrt, symbols, zoo)
from sympy.core.basic import _aresame
from sympy.core.expr import AtomicExpr
from sympy.core.function import AppliedUndef, UndefinedFunction
from sympy.core.numbers import (Half, ImaginaryUnit, Infinity, NegativeOne,
                                One, Pi, Zero)
from sympy.core.operations import AssocOp
from sympy.core.sorting import default_sort_key
from sympy.core.traversal import (iterargs, iterfreeargs, postorder_traversal,
                                  preorder_traversal, walk)

from exprenc import dec, enc, vector, write

R = random.Random(20251023)

# ---------------------------------------------------------------------------
# the corpus

x, y, z, t, w = symbols("x y z t w")
a, b, c, d = symbols("a b c d")
p = Symbol("p", positive=True)
q = Symbol("q", negative=True)
k = Symbol("k", integer=True)
nn = Symbol("nn", nonnegative=True)
r = Symbol("r", real=True)
A, B, C, L = symbols("A B C L", commutative=False)
Wl = Wild("W")
f, g = Function("f"), Function("g")
fr = Function("fr", real=True)
half = Rational(1, 2)

LEAVES = [x, y, z, a, b, p, q, k, nn, r, Integer(2), Integer(3),
          Integer(-1), half, Rational(-2, 3), Float("2.5"), pi, I, S.Zero,
          S.One]
SYMS = [x, y, z, a, b, p, q, k, nn, r]
EXPS = [Integer(2), Integer(3), Integer(-1), Integer(-2), half, -half,
        Rational(3, 2), k, y, 2 * k, k + 1, 2 * y, y + 1]
NEWS = [t, Integer(5), Rational(1, 3), t**2, t + 1, 2 * t, Float("0.5"),
        S.Zero, S.One, -t, x, x + y, oo, I]
NC_NEWS = [L, L**2, 2 * L, t, A]

CLASSES = {
    Basic: "Basic", Atom: "Atom", Expr: "Expr", AtomicExpr: "AtomicExpr",
    Number: "Number", NumberSymbol: "NumberSymbol", Function: "Function",
    AssocOp: "AssocOp", Symbol: "Symbol", Dummy: "Dummy", Wild: "Wild",
    Integer: "Integer", Rational: "Rational", Float: "Float", Add: "Add",
    Mul: "Mul", Pow: "Pow", AppliedUndef: "AppliedUndef", Tuple: "Tuple",
    exp: "Exp", sin: "Sin", cos: "Cos", ImaginaryUnit: "ImaginaryUnit",
    Pi: "Pi", Half: "Half", One: "One", Zero: "Zero",
    NegativeOne: "NegativeOne", Infinity: "Infinity", Integral: "Integral",
    Derivative: "Derivative",
}
BY_NAME = {v: k_ for k_, v in CLASSES.items()}


def rexpr(depth):
    """A random sum, product or power of LEAVES, evaluated."""
    if depth == 0 or R.random() < 0.25:
        return R.choice(LEAVES)
    kind = R.random()
    if kind < 0.4:
        return Add(*[rexpr(depth - 1) for _ in range(R.randint(2, 4))])
    if kind < 0.8:
        return Mul(*[rexpr(depth - 1) for _ in range(R.randint(2, 3))])
    return Pow(rexpr(depth - 1), R.choice(EXPS))


def rnc(depth):
    """A random expression with non-commutative factors."""
    if depth == 0 or R.random() < 0.2:
        return R.choice([A, B, C, A, B, x, y, Integer(2), Integer(-3), half])
    kind = R.random()
    if kind < 0.3:
        return Add(*[rnc(depth - 1) for _ in range(R.randint(2, 3))])
    if kind < 0.85:
        return Mul(*[rnc(depth - 1) for _ in range(R.randint(2, 4))])
    return Pow(rnc(depth - 1), R.choice([Integer(2), Integer(3), Integer(-1),
                                         half, k]))


def rfun(depth):
    """A random expression with undefined functions in it."""
    if depth == 0 or R.random() < 0.3:
        return R.choice([f(x), f(y), g(x, y), fr(x), f(x + 1), x, y,
                         Integer(2), Rational(-1, 2)])
    kind = R.random()
    if kind < 0.45:
        return Add(*[rfun(depth - 1) for _ in range(R.randint(2, 3))])
    if kind < 0.85:
        return Mul(*[rfun(depth - 1) for _ in range(2)])
    return Pow(rfun(depth - 1), R.choice([Integer(2), Integer(-1), half]))


def unique(seq):
    out = []
    for e in seq:
        if not any(type(e) is type(o) and e == o for o in out):
            out.append(e)
    return out


def subexprs(e):
    return unique(preorder_traversal(e))


def size_ok(e, limit=40):
    return sum(1 for _ in preorder_traversal(e)) <= limit


# ---------------------------------------------------------------------------
# encoding the arguments


def enc_arg(v):
    if isinstance(v, UndefinedFunction):
        return "undef:" + enc(v(x))
    if isinstance(v, type):
        return "class:" + CLASSES[v]
    if isinstance(v, PyStr):
        return "pystr:" + v.s
    if isinstance(v, Basic):
        return enc(v)
    if isinstance(v, (list, tuple)):
        return [enc_arg(i) for i in v]
    if isinstance(v, dict):
        return [[enc_arg(k_), enc_arg(x_)] for k_, x_ in v.items()]
    return v


def dec_arg(v):
    if isinstance(v, str):
        if v.startswith("undef:"):
            return dec(v[6:]).func
        if v.startswith("class:"):
            return BY_NAME[v[6:]]
        if v.startswith("pystr:"):
            return v[6:]
        return dec(v)
    if isinstance(v, list):
        return [dec_arg(i) for i in v]
    return v


class PyStr:
    """A Python string argument (rather than an encoded expression)."""

    def __init__(self, s):
        self.s = s


VECTORS = []


def add(op, args, fn, **params):
    """Records `fn(*args)`; `args` are passed through `enc_arg` (Python
    calls `fn` on the decoded, fresh copies)."""
    VECTORS.append(vector(op, [enc_arg(v) for v in args],
                          lambda *vs: fn(*[dec_arg(v) for v in vs]),
                          **params))


def no_dummies(e):
    """Whether a result is free of the Dummies SymPy makes (their indices
    are random)."""
    return not any(isinstance(s, Dummy) for s in preorder_traversal(e))


def sorted_set(s):
    return sorted(s, key=lambda e: (default_sort_key(e), enc(e)))


# ---------------------------------------------------------------------------
# subs


def subs_pair(e, old, new, **kw):
    add("subs", [e, old, new], lambda e, o, n: e.subs(o, n, **kw), **kw)


def subs_dict(e, pairs, simultaneous=False):
    add("subs_dict", [e, list(map(list, pairs))],
        lambda e, ps: e.subs({o: n for o, n in ps}, simultaneous=simultaneous),
        simultaneous=simultaneous)


def subs_list(e, pairs, simultaneous=False):
    add("subs_list", [e, list(map(list, pairs))],
        lambda e, ps: e.subs([tuple(pn) for pn in ps],
                             simultaneous=simultaneous),
        simultaneous=simultaneous)


def subs_set(e, pairs):
    add("subs_set", [e, list(map(list, pairs))],
        lambda e, ps: e.subs({tuple(pn) for pn in ps}))


def subs_single(e, arg):
    add("subs_single", [e, arg], lambda e, s: e.subs(s))


# SymPy's own tests (test_subs.py, test_basic.py)
n3 = Rational(3)
ni = Symbol("n", negative=True)
a2, b2 = symbols("a b", commutative=True)
xn, yn, zn, wn = symbols("x y z w", commutative=False)
alpha = Symbol("alpha", commutative=True)
someint = Symbol("someint", commutative=True, integer=True)
Rw, Sw, Tw, Uw = symbols("R S T U", cls=Wild)
xr = Symbol("xr", real=True)
xnn, ynn = symbols("x y", nonnegative=True)
N_nc = Symbol("N", commutative=False)
c2, c3, q1p, q2p, c1, s1, s2, s3 = symbols("c2 c3 q1p q2p c1 s1 s2 s3")

for e, o, n in [
    (x, x, n3), (2 * x, x, n3),
    (Symbol("x1"), Symbol("x1"), Float(3.0)), (Symbol("x1") * Symbol("x2"),
                                           Symbol("x1"), Float(3.0)),
    (x**x, x, 1), (x**x, x, 1.0),
    (sqrt(1 - sqrt(x)), x, 4), (sqrt(1 - x**2)**3, x, 2),
    (x**Rational(1, 3), x, 27), (x**Rational(1, 3), x, -27),
    ((-x)**Rational(1, 3), x, 27), (x**ni, x, 0), (exp(-1), E, 0),
    (x**(4.0 * y), x**(2.0 * y), ni), (2**(x + 2), 2, 3),
    ((3**(1 + xr) + 2**(1 + xr)) / (3**xr + 2**xr), 2**xr, w),
    (sqrt(x) * exp(y), sqrt(x), 1),
    (1 + x * y, x, pi),
    ((x * y * z), z * x, y), ((z * x), 1 / x, z), ((x * y / z), 1 / z, a),
    ((x * y / z), x / z, a), ((x * y / z), y / z, a), ((x * y / z), x / z, 1 / a),
    ((x * y / z), x, 1 / a), ((2 * x * y), 5 * x * y, z), ((x * y * A), x * y, a),
    ((x**2 * y**(x * Rational(3, 2))), x * y**(x / 2), 2),
    ((x * exp(x * 2)), x * exp(x), 2), (((x**(2 * y))**3), x**y, 2),
    ((x * A * B), x * A, y), ((x * y * (1 + x) * (1 + x * y)), x * y, 2),
    (((1 + A * B) * A * B), A * B, x * A * B), ((x * a / z), x / z, A),
    ((x**3 * A), x**2 * A, a), ((x**2 * A * B), x**2 * B, a),
    ((x**2 * A * B), x**2 * A, a),
    ((b * A**3 / (a**3 * c**3)), a**4 * c**3 * A**3 / b**4, z),
    ((6 * x), 2 * x, y), ((y * exp(x * Rational(3, 2))), y * exp(x), 2),
    ((A**2 * B * A**2 * B * A**2), A * B * A, C), ((x * A**3), x * A, y),
    ((x**2 * A**3), x * A, y), ((x * A**3), x * A, B),
    ((x * A * B * A * exp(x * A * B)), x * A, B),
    ((x * A * B * C * A * B), x * A * B, C), ((-I * a * b), a * b, 2),
    ((-8 * I * a), -2 * a, 1), ((-I * a), -a, 1), ((4 * x**2), 2 * x, y),
    ((2 * 4 * x**2), 2 * x, y), ((-x**3 / 9), -x / 3, z), ((-x**3 / 9), x / 3, z),
    ((-2 * x**3 / 9), x / 3, z), ((-2 * x**3 / 9), -x / 3, z),
    ((-2 * x**3 / 9), -2 * x, z), ((-2 * x**3 / 9), 2 * x, z),
    ((2 * (3 * x / 5 / 7)**2), 3 * x / 5, z), ((4 * x), -2 * x, z),
    ((2 * a2), 1, 3), ((2 * a2), 2, 3), ((2 * a2), a2, 3), (sin(2), 1, 3),
    (sin(2), 2, 3), (sin(a2), a2, 3), ((2 * xn), 1, 3), ((2 * xn), 2, 3),
    ((2 * xn), xn, 3), (sin(xn), xn, 3),
    ((a2 * b2), 2 * a2, 1), ((1.5 * a2 * b2), a2, 1), ((2 * a2 * b2), 2 * a2, 1),
    ((2 * a2 * b2), 4 * a2, 1), ((xn * yn), 2 * xn, 1), ((1.5 * xn * yn), xn, 1),
    ((2 * xn * yn), 2 * xn, 1), ((2 * xn * yn), 4 * xn, 1),
    ((a * b), a * b, d), ((a * b * a * b), a * b, d), ((a * a * b * b), a * b, d),
    ((a * b * c * d), a * b * c, t), ((a * b**c), a, d), ((a * b**c), b, d),
    ((a * b**c), c, d), ((a * b * c * b * a), a * b, d), ((a**3 * b**2 * a), a * b, d),
    ((xn * yn), xn * yn, L), ((wn * yn * xn), xn * yn, L),
    ((wn * xn * yn * zn), xn * yn, L), ((xn * yn * xn * yn), xn * yn, L),
    ((xn * xn * yn), xn * yn, L), ((xn * xn * yn * yn), xn * yn, L),
    ((wn * xn * yn), xn * yn * zn, L), ((xn * yn**zn), xn, L), ((xn * yn**zn), yn, L),
    ((xn * yn**zn), zn, L), ((wn * xn * yn * zn * xn * yn), xn * yn * zn, L),
    ((wn * xn * yn * yn * wn * xn * xn * yn * xn * yn * yn * xn * yn), xn * yn, L),
    ((xn * xn * xn), xn * xn, L), ((xn * xn * xn * yn * xn * xn * xn * xn), xn * xn, L),
    ((xn**Rational(3, 2)), xn**half, L), ((xn**half), xn**half, L),
    ((xn**Rational(-1, 2)), xn**half, L), ((xn**Rational(-1, 2)), xn**Rational(-1, 2), L),
    ((xn**(2 * someint)), xn**someint, L), ((xn**(2 * someint + 3)), xn**someint, L),
    ((xn**(3 * someint + 3)), xn**someint, L), ((xn**(3 * someint)), xn**(2 * someint), L),
    ((xn**(4 * someint)), xn**(2 * someint), L),
    ((xn**(4 * someint + 1)), xn**(2 * someint), L),
    ((xn**(4 * someint)), xn**(3 * someint), L),
    ((xn**(4 * someint + 1)), xn**(3 * someint), L),
    ((xn**(2 * alpha)), xn**alpha, L), ((xn**(2 * alpha + 2)), xn**2, L),
    (((2 * zn)**alpha), zn**alpha, yn), ((xn**(2 * someint * alpha)), xn**someint, L),
    ((xn**(2 * someint + alpha)), xn**someint, L),
    ((xn**(someint**2 + 3)), xn**someint, L), ((4**zn), 2**zn, yn),
    ((xn + yn), xn + yn, L), ((xn - yn), xn - yn, L), ((xn / yn), xn, L),
    ((xn**yn), xn, L), ((xn**yn), yn, L), (((a - c) / b), b, d),
    (((a - b) / (c * d - a * b)), c * d - a * b, t),
    ((Rw * Sw), Rw * Sw, Tw), ((Sw * Rw), Rw * Sw, Tw), ((Rw + Sw), Rw + Sw, Tw),
    ((Rw**Sw), Rw, Tw), ((Rw**Sw), Sw, Tw), ((Rw * Sw**Tw), Rw, Uw),
    ((Rw * Sw**Tw), Sw, Uw), ((Rw * Sw**Tw), Tw, Uw),
    ((a * xn * yn), xn * yn, L), ((a * b * xn * yn * xn), xn * yn, L),
    ((1 / a), a, c), ((1 / a**2), a, c), ((1 / a**2), a, -2), ((-(1 / a**2)), a, -2),
    ((1 / x), x, 0),
    ((a**2 - b - c), a**2 - b, d), ((a**2 - c), a**2 - c, d),
    ((a**2 - b - c), a**2 - c, d), ((a**2 - x - c), a**2 - c, d),
    ((a**2 - b - sqrt(a)), a**2 - sqrt(a), c), ((a + b + c + d), b + c, x),
    ((a + b + c + d), -b - c, x), (((x + 1) * y), x + 1, t),
    (((-x - 1) * y), x + 1, t), (((x - 1) * y), x + 1, t), (((-x + 1) * y), x + 1, t),
    ((0.1 + a), 0.1, Rational(1, 10)), ((-x * (-y + 1) - y * (y - 1)), -y + 1, x),
    ((I * Symbol("a")), 1, 2), ((x + 3), x + 3, a), ((x + 3), x + 2, a),
    ((x + y + 3), x + 3, a), ((x + y + 3), x + 2, a), ((x + 3.0), x + 3.0, a),
    ((x + 3.0), x + 2.0, a), ((x + y + 3.0), x + 3.0, a), ((x + y + 3.0), x + 2.0, a),
    ((a / (b * c)), b * c, d), ((a / (b**2 * c**3)), b * c, d), ((1 / (x * y)), x * y, 2),
    (((1 + x * y) / (x * y)), x * y, 1), ((x * y * z), x * y, 2),
    (((1 + x * y) / (x * y) / z), x * y, 1), (Tuple(1, True), 1, 2),
    ((x + 2.0), 2, 3), ((x + 2.0), 2.0, 3), ((x - 1), 1, y), ((x - 1), -1, y),
    ((x - oo), oo, y), ((x - oo), -oo, y),
    ((1 / (1 + x / y)), x / y, x), ((-2 * I), 2 * I, x), ((-I * x), I * x, x),
    ((-3 * I * y**4), 3 * I * y**2, x), ((-12 * x + y), -x, 1),
    ((2**(I * xr)), 2**xr, y), (((-2)**(I * xr)), (-2)**xr, y),
    ((-2 * x * sqrt(2)), 2 * x, y), ((x * A), x**2 * A, B), ((A**2), A**3, B),
    ((A**6), A**3, B), ((x + Float(2.0)), Float(2.0), 2),
    ((1 / (1 - t)), t, 1), ((t / (t - 1)), t, 1), ((-t / -(t - 1)), t, 1),
    ((1 + 2 * x * x), 2 * x * x, Symbol("s")), ((1 - 2 * x * x), 2 * x * x, Symbol("s")),
    ((2 * x * x * 2 * y * y), 2 * x * x, Symbol("s")),
    ((4**x), 2**x, y), ((9**x), 3**x, y), ((1 / sqrt(x**2)), sqrt(x**2), y),
    ((sqrt(x)**3 + sqrt(x) + x + x**2), sqrt(x), y),
    ((x**4 + x**3 + x**2 + x + sqrt(x)), x**2, y), (x, x**3, y), (x, x**Rational(1, 3), y),
    ((xnn**4 + xnn**3 + xnn**2 + xnn + sqrt(xnn)), xnn**2, ynn), (xnn, xnn**3, ynn),
    ((1 / x), x**2, 1), ((1 / (x**3)), x**2, 1), ((1 / (x**4)), x**2, 1),
    ((1 / (x**3)), x**4, 1), ((1 / (y**5)), x**5, 1), ((1 / xr), xr**2, y),
    ((x**y), x, 1), ((x**y), y, oo), ((2 * x * (y + 1)), x, 1),
]:
    subs_pair(e, o, n)

subs_pair(2 * x * (y + 1), x, 1, hack2=True)
subs_pair(2 * (y + 1 + N_nc), N_nc, 0, hack2=True)
for pw in range(1, 5):
    for kk in range(10):
        subs_pair(yn * xn**kk, xn**pw, L)
for e in [xn**(-1), xn**(-2), xn**(-3), xn**(-4), xn**(-5), xn, xn**2, xn**3,
          xn**4, xn**5]:
    subs_pair(e, xn**3, L)
    subs_pair(e, xn**(-3), L)

subs_dict(1 + x * y, [(x, pi), (y, 2)])
subs_dict(c2**2 * q2p * c3 + c1**2 * s2**2 * q2p * c3 + s1**2 * s2**2 * q2p * c3
          - c1**2 * q1p * c2 * s3 - s1**2 * q1p * c2 * s3,
          [(c1**2, 1 - s1**2), (c2**2, 1 - s2**2), (c3**3, 1 - s3**2)])
subs_dict(1 + x * y, [(x, y), (y, 2)])
subs_dict(x / y, [(x, 0), (y, 0)])
subs_dict(y / x, [(x, 0), (y, 0)])
subs_dict(x / y, [(x, 0), (y, 0)], simultaneous=True)
subs_dict(y / x, [(x, 0), (y, 0)], simultaneous=True)
subs_list(x / y, [(x, 0), (y, 0)])
subs_list(x / y, [(x, 0), (y, 0)], simultaneous=True)
subs_list(xn * yn * xn, [(xn, xn * yn), (yn, xn)], simultaneous=True)
subs_dict(x + Float(2.0), [(Float(2.0), 2)])
subs_list(x + y, [(x, 3), (y, x**2)])
subs_list(x + y, [(y, x**2), (x, 3)])
subs_list(y - 1 + 3 * x, [(y, z + 2), (1 + z, 5), (z, 2)])
subs_list(y - 2, [(y, z + 2), (z, 3)])
subs_list(6 * x**5 + x**4 - 4 * x**3 + 4 * x**2 - 2 * x + 3,
          [(x**2, y**2), (x**4, y**4)])
subs_list(x**y, [(x, 1), (y, oo)])
subs_list(x**y, [(y, oo), (x, 1)])
subs_dict(x**y, [(x, 1), (y, oo)])
subs_list(x**y, [(x, 1), (y, oo)], simultaneous=True)
subs_list(x, [(x, y)])
subs_single(x, Tuple(Tuple(x, y)))
subs_set(x * Symbol("s"), [(x, 2), (Symbol("s"), 3)])
add("subs_str", [2 * x + y + z, [[PyStr("x"), 1], [PyStr("y"), 2]]],
    lambda e, ps: e.subs({o: n for o, n in ps}))
add("subs_str", [Symbol("N"), [[PyStr("N"), 3]]],
    lambda e, ps: e.subs({o: n for o, n in ps}))
add("subs_str", [Symbol("var1"), [[PyStr("var1"), PyStr("x")]]],
    lambda e, ps: e.subs({o: n for o, n in ps}))
add("subs_str", [1 / x, [[PyStr("x"), PyStr("1/2")]]],
    lambda e, ps: e.subs({o: n for o, n in ps}))
add("subs_str", [Symbol("text"), [[PyStr("text"), Tuple()]]],
    lambda e, ps: e.subs({o: n for o, n in ps}))

# the single-argument forms and their errors
for e in [x + y, 2 * x, x**2 * y]:
    for arg in [x, x + y, Integer(3), Tuple(), Tuple(Tuple(x, 2)),
                Tuple(Tuple(x, 2), Tuple(y, 3)), Tuple(Tuple(y, x), Tuple(x, 2)),
                Tuple(x), Tuple(Tuple(x, 1, 2)), Tuple(Tuple(x)), FiniteSet(Tuple(x, 1)),
                FiniteSet(Tuple(x, 1), Tuple(y, 2)), S.EmptySet, Interval(0, 1),
                Tuple(x, y), Tuple(Tuple(x, y), z)]:
        subs_single(e, arg)

# random single substitutions
NEWS_ALL = NEWS
for depth, count in ((1, 40), (2, 140), (3, 120)):
    for _ in range(count):
        e = rexpr(depth)
        if not size_ok(e):
            continue
        subs = subexprs(e)
        cands = list(subs)
        if e.is_Add or e.is_Mul:
            args = list(e.args)
            for _ in range(3):
                kk = R.randint(1, len(args))
                cands.append(e.func(*R.sample(args, kk)))
        cands += [-s for s in R.sample(subs, min(2, len(subs)))]
        for s in subs:
            if s.is_Pow:
                bb, ex = s.args
                cands += [bb**m for m in (1, 2, 3, -1)]
                cands += [bb**(ex * 2), bb**(ex / 2), Pow(R.choice(SYMS), ex)]
        cands += R.sample(LEAVES, 3)
        cands = unique(cands)
        for old in R.sample(cands, min(len(cands), 6)):
            subs_pair(e, old, R.choice(NEWS_ALL))

# non-commutative ones
for _ in range(120):
    e = rnc(R.choice((2, 3)))
    if not size_ok(e):
        continue
    subs = subexprs(e)
    cands = list(subs)
    if e.is_Mul:
        args = list(e.args)
        for _ in range(3):
            i = R.randrange(len(args))
            j = R.randint(i + 1, len(args))
            cands.append(Mul(*args[i:j]))
    for s in subs:
        if s.is_Pow:
            bb, ex = s.args
            cands += [bb**m for m in (1, 2, 3, -1)]
    cands = unique(cands)
    for old in R.sample(cands, min(len(cands), 5)):
        subs_pair(e, old, R.choice(NC_NEWS))

# undefined functions
for _ in range(60):
    e = rfun(R.choice((1, 2)))
    if not size_ok(e):
        continue
    cands = unique(subexprs(e) + [f(x), x, y, f])
    for old in R.sample(cands, min(len(cands), 3)):
        if isinstance(old, UndefinedFunction):
            continue
        subs_pair(e, old, R.choice([t, 2, f(t), g(t, t)]))

# dicts, lists, sets and simultaneous substitution
for _ in range(220):
    e = rexpr(R.choice((2, 3)))
    if not size_ok(e):
        continue
    subs = subexprs(e)
    olds = unique(R.sample(subs, min(len(subs), R.randint(2, 4))))
    pairs = [(o, R.choice(NEWS_ALL + olds)) for o in olds]
    choice = R.random()
    if choice < 0.3:
        subs_dict(e, pairs)
    elif choice < 0.55:
        subs_list(e, pairs)
    elif choice < 0.75:
        subs_dict(e, pairs, simultaneous=True)
    elif choice < 0.9:
        subs_list(e, pairs, simultaneous=True)
    else:
        subs_set(e, pairs)
for _ in range(40):
    e = rnc(3)
    if not size_ok(e):
        continue
    subs = subexprs(e)
    olds = unique(R.sample(subs, min(len(subs), 3)))
    pairs = [(o, R.choice(NC_NEWS + olds)) for o in olds]
    if R.random() < 0.5:
        subs_list(e, pairs, simultaneous=True)
    else:
        subs_dict(e, pairs)
# infinities first (and the shifting indices of that loop)
for e in [x + y + z, x * y + z, x + 2 * y, x**y + z, x - y]:
    subs_dict(e, [(x, oo), (y, 1), (z, 2)])
    subs_dict(e, [(x, 1), (y, oo), (z, -oo)])
    subs_dict(e, [(x, zoo), (y, 2), (z, oo)])
    subs_dict(e, [(x, nan), (y, oo), (z, 1)])
    subs_dict(e, [(x + y, 2), (x, oo), (y, -oo)])

# ---------------------------------------------------------------------------
# xreplace

for e, rule in [
    (x + y, {x: 2}), (x * y + 1, {x * y: z}), (x + y + z, {x + y: t}),
    (x**2 + 1, {x**2: t, x: y}), (2 * x * y, {2: 3}), (x - oo, {oo: t}),
    (x - oo, {-oo: t}), (x + 2.0, {2.0: 2}), (f(x) + x, {x: y}),
    (f(x) + x, {f(x): y}), (A * B + B * A, {A: B, B: A}), (x, {x: y}),
    (x, {y: x}), (Integer(2), {Integer(2): x}), (x + y, {}),
    (sin(x) + cos(x), {sin(x): y}), (Tuple(x, Tuple(y, x)), {x: z}),
    (Tuple(x, Tuple(y, x)), {Tuple(y, x): z}),
]:
    add("xreplace", [e, rule], lambda e, ps: e.xreplace({o: n for o, n in ps}))
for _ in range(200):
    e = rexpr(R.choice((2, 3)))
    if not size_ok(e):
        continue
    subs = subexprs(e)
    olds = unique(R.sample(subs, min(len(subs), R.randint(1, 3))))
    rule = {o: R.choice(NEWS_ALL) for o in olds}
    add("xreplace", [e, rule], lambda e, ps: e.xreplace({o: n for o, n in ps}))

# ---------------------------------------------------------------------------
# has, has_free, has_xfree, atoms, free_symbols, is_same

HAS_CORPUS = [x + y + 1, 2 * x * y, x**2 + x, A * B * C, A * B + C, f(x) + 1,
              f(x, y) * g(z), sin(x) + cos(y), exp(x) * y, x**y, Tuple(x, y),
              Integral(x * y, (x, 0, 1)), Lambda(x, x + y), Derivative(f(x), x),
              Eq(x, y + 1), pi * x + I, Float("1.5") * x, Integer(3), x,
              Rational(1, 2) + x, x * y * z + x * z,
              A * B * A * C, x + y + z + 1, 2 * x + 3 * y]
PATTERNS = [x, y, z, x + 1, x + y, y + 1, x * y, 2 * x, x * z, A * B, B * C,
            A * C, f(x), f, g, sin(x), Symbol, Number, Integer, Rational, Add,
            Mul, Pow, AppliedUndef, Function, Atom, Basic, Expr, Float, Tuple,
            exp, Integral, I, pi, Integer(1), Integer(2), half, x**2, t, Dummy,
            Wild, Half, One, NumberSymbol, AssocOp, AtomicExpr, Zero, Pi,
            ImaginaryUnit]


for e in HAS_CORPUS:
    add("has", [e, []], lambda e, ps: e.has(*ps))
    for pt in PATTERNS:
        add("has", [e, [pt]], lambda e, ps: e.has(*ps))
    for _ in range(4):
        ps = R.sample(PATTERNS, 2)
        add("has", [e, ps], lambda e, ps: e.has(*ps))
    for pt in [x, y, x + 1, x * y, f(x), Symbol, Integral]:
        add("has_free", [e, [pt]], lambda e, ps: e.has_free(*ps))
    add("has_xfree", [e, [x, y]], lambda e, ps: e.has_xfree(set(ps)))
    add("has_xfree", [e, [x + 1]], lambda e, ps: e.has_xfree(set(ps)))
    add("free_symbols", [e], lambda e: sorted_set(e.free_symbols))
    add("atoms", [e, []], lambda e, ts: sorted_set(e.atoms(*ts)))
    for ts in [[Symbol], [Number], [Integer], [Rational], [Add], [Mul],
               [Pow], [AppliedUndef], [Function], [Symbol, Number], [I],
               [x], [Integer(5)], [half], [f], [f(t)], [Tuple], [Basic],
               [Atom], [NumberSymbol], [Dummy]]:
        add("atoms", [e, ts], lambda e, ts: sorted_set(e.atoms(*ts)))

for _ in range(150):
    e = rexpr(R.choice((2, 3)))
    if not size_ok(e):
        continue
    subs = subexprs(e)
    cands = subs + [x + 1, x * y, 2 * x, x + y + 1]
    if e.is_Add or e.is_Mul:
        args = list(e.args)
        cands.append(e.func(*R.sample(args, R.randint(1, len(args)))))
    for pt in R.sample(cands, min(4, len(cands))):
        add("has", [e, [pt]], lambda e, ps: e.has(*ps))
    add("free_symbols", [e], lambda e: sorted_set(e.free_symbols))
    add("atoms", [e, []], lambda e, ts: sorted_set(e.atoms(*ts)))
    add("atoms", [e, [R.choice([Symbol, Number, Pow, Mul, Add, Rational])]],
        lambda e, ts: sorted_set(e.atoms(*ts)))

for aa, bb in [(x + 2, x + 2.0), (x + 2.0, x + 2.0), (S(2), S(2.0)), (x, x),
               (x, Symbol("x", positive=True)), (Tuple(cos(x), S(1)),
                                                 Tuple(cos(x), S(1.0))),
               (Float(1), Float(1)), (Float(1, 3), Float(1, 15)), (x * y, y * x),
               (half, Float(0.5)), (Tuple(), Tuple(Tuple())), (Integer(0), S.Zero),
               (x + y, x + y + 0), (A * B, B * A), (f(x), f(x)), (f(x), g(x))]:
    add("is_same", [aa, bb], lambda aa, bb: _aresame(aa, bb))

# ---------------------------------------------------------------------------
# traversals

TRAV = [x + y * z, z + w * (x + y), (x + y) * z, x**2 + 2 * x + 1,
        A * B * C + x, f(x, g(y)) + 1, Tuple(x, Tuple(y, z), 1),
        Integral(x**2, (x, 0, 1)), sin(x) * cos(y) + exp(x),
        Eq(x + 1, y), 2 * x * y**3 / z, Integral(f(x), (f(x), 1)),
        Lambda(x, x + y), x, Integer(2)]


def skipping(e, cls):
    out = []
    pt = preorder_traversal(e)
    for i in pt:
        out.append(i)
        if isinstance(i, cls):
            pt.skip()
    return out


for e in TRAV + [rexpr(3) for _ in range(30)]:
    if not size_ok(e, 60):
        continue
    add("preorder", [e], lambda e: list(preorder_traversal(e)), keys="none")
    add("preorder", [e], lambda e: list(preorder_traversal(e, keys=True)),
        keys="true")
    add("preorder", [e],
        lambda e: list(preorder_traversal(e, keys=default_sort_key)),
        keys="default_sort_key")
    add("postorder", [e], lambda e: list(postorder_traversal(e)), keys="none")
    add("postorder", [e], lambda e: list(postorder_traversal(e, keys=True)),
        keys="true")
    add("postorder", [e],
        lambda e: list(postorder_traversal(e, keys=default_sort_key)),
        keys="default_sort_key")
    for cls in (Add, Mul, Pow):
        add("preorder_skip", [e, cls], lambda e, cls: skipping(e, cls))
    add("iterargs", [e], lambda e: list(iterargs(e)))
    add("iterfreeargs", [e], lambda e: list(iterfreeargs(e)))
    add("iterfreeargs_first", [e], lambda e: next(iter(iterfreeargs(e))))
    for ts in [[Add], [Add, Mul], [Mul, Pow], [Basic]]:
        add("walk", [e, ts], lambda e, ts: list(walk(e, *ts)))

# ---------------------------------------------------------------------------
# as_independent

IND_CORPUS = [x + y + 1, 2 * x * y, x * y * z + x + 3, sin(x) * y, x**2 * y + y,
              A * B * x * y, x * A * y * B, x * y * A * B + A, 3 * x * A * B * y,
              f(x) * y + x, Integer(0), Integer(3), x, y, pi * x, x + A * B,
              -2 * x * y, Float("1.5") * x * y, Integral(x, (x, 0, 1)),
              x * exp(y) + 2, Eq(x, y), Tuple(x, y), x**y * y**x]
DEPS = [[x], [y], [x, y], [z], [Symbol], [A], [x * y], [x**2], [f(x)], [Add],
        [Mul], [Number], [x, A], [Integral], [I], []]
for e in IND_CORPUS + [rexpr(R.choice((2, 3))) for _ in range(60)]:
    if not size_ok(e):
        continue
    for deps in (DEPS if e in IND_CORPUS else R.sample(DEPS, 5)):
        for as_add in (None, True, False):
            add("as_independent", [e, deps],
                lambda e, ds, as_add=as_add: (
                    e.as_independent(*ds) if as_add is None
                    else e.as_independent(*ds, as_Add=as_add)),
                as_Add=as_add)

# ---------------------------------------------------------------------------
# coeff

COEFF_CORPUS = [
    3 * x + 2 * x * y + x**2 + 5, x + y + 1, 2 * x**2 + x**2 * y, x * y,
    (x + 1) * y + x, x + 2 * y * x**2 + x**3 * z, a * x + b * x**2 + c,
    3 + 2 * x + 4 * x**2, x * sin(y) + 2 * x, 2 * A * B + 3 * B * A,
    x * A + y * A * B + A * B * x, A * B * C + A * C, 2 * A + x * A,
    A * x * B + A * B, A * B + B * A + A, x * A * B * C + y * A * B,
    (A + B) * C + A * C, A * B * A + A * B, x + A * B + A, Integer(0),
    Integer(5), x, x**2, 2 * x, 3 * x * y**2, x * y + x * z + y * z,
    (1 + x)**2 + (1 + x), x**k + x**2, 2 * pi * x + pi, 3 * sqrt(2) * x + x,
    x + x * y + x * y**2, -x + y, f(x) * x + f(x), Float("2.5") * x + y,
    A * B * C * A * B + A * B, C * A * B + A * B * C + B * A, A**2 * B + A * B,
    x * A**2 + y * A, A * x + B * x, (x + y) * A + x * B,
]
XS = [x, y, x**2, x * y, x + 1, A, A * B, B, C, B * C, A**2, Integer(1),
      Integer(2), Integer(0), sin(y), pi, f(x), z, x + y, A + B, 2 * x]
for e in COEFF_CORPUS:
    for xx in XS:
        for n in (1, 2, 0):
            if n != 1 and not (xx in (x, A, x * y)):
                continue
            for right in (False, True):
                add("coeff", [e, xx], lambda e, xx, n=n, right=right:
                    e.coeff(xx, n, right=right), n=n, right=right)
for _ in range(150):
    e = rexpr(R.choice((2, 3)))
    if not size_ok(e):
        continue
    xx = R.choice(subexprs(e) + SYMS)
    n = R.choice((1, 1, 1, 2, 0, -1))
    add("coeff", [e, xx], lambda e, xx, n=n: e.coeff(xx, n), n=n, right=False)

# ---------------------------------------------------------------------------
# is_polynomial, is_rational_function

POLY_CORPUS = [x**2 + 2 * x + 1, x**2 + 1 / x, x**y, 2**x, x**half + 1,
               x * y + y**2, (x + 1)**3, sin(x) + 1, sin(y) * x**2,
               (x + 1) / (x - 1), x**k, x**-2, f(x) + 1, f(y) * x, pi * x + E,
               Integer(3), x, A * B + A, oo, x + oo, zoo, nan, sqrt(2) * x,
               x**Float("2.0"), (x**2 + 1)**2, 1 / (x**2 + 1), x**(2 * k),
               exp(x), log(x) * x, Integral(x, (x, 0, 1)), x**3 * y**-1,
               Eq(x, 1), Tuple(x)]
for e in POLY_CORPUS + [rexpr(R.choice((2, 3))) for _ in range(80)]:
    if not size_ok(e):
        continue
    for syms in [[], [x], [y], [x, y], [z], [k], [x + 1], [sin(x)], [f(x)]]:
        add("is_polynomial", [e, syms], lambda e, ss: e.is_polynomial(*ss))
        add("is_rational_function", [e, syms],
            lambda e, ss: e.is_rational_function(*ss))

# ---------------------------------------------------------------------------
# more of SymPy's tests (test_subs.py, test_expr.py, test_basic.py and
# test_traversal.py; Basic() instances become Tuples, which rsympy has)

subs_list(sin(x), [(sin(x), 2), (x, 1)])
subs_dict(sin(x), [(sin(x), 2), (x, 1)])
subs_list(sin(x), [(x, 1), (sin(x), 2)])
subs_dict(sin(2 * x) + sqrt(sin(2 * x)) * cos(2 * x) * sin(exp(x) * x),
          [(sin(2 * x), c), (sqrt(sin(2 * x)), a), (cos(2 * x), b),
           (exp(x), Symbol("e")), (x, d)])
for e, o, n in [
    (exp(xn * yn - zn), xn * yn, L),
    (a2 * exp(xn * yn - wn * zn) + b2 * exp(xn * yn + wn * zn), zn, 0),
    (wn * exp(a2 * b2 - c) * xn * yn / 4, xn * yn, L),
    (Rw * xn * yn * exp(xn * yn), xn * yn, L),
    (a2 * xn * yn * yn * xn - xn * yn * zn * exp(a2 * b2), xn * yn, L),
    (a + b + exp(a + b), a + b, c), (c + b + exp(c + b), c + b, a),
    (a**2 - b - c, Add(*(a**2 - b - c).args[:2]), d),
    (2 * sin(x) * cos(x), x, n3), (2 * sin(x) * cos(x), sin(x), cos(x)),
    (exp((x - 1) * log(2)) * 2**x, 2**x, w), (abs(x), x, -7.7),
    (Eq(f(x)**2, x), f(x), 4), (Eq(f(x)**2, 16), f(x), 3),
    (Eq(f(x)**2, 16), f(x), 4), (x * exp(x), x, y), (x * exp(x), exp(x), y),
    (exp(2 * x + 3 * y), x, t), (exp(x)**2, exp(x), y),
    (exp(x) * exp(y), exp(x + y), z), (x**(2 * y) * x**3, x**y, t),
]:
    subs_pair(e, o, n)
xc = Symbol("xc", complex=True)
yc = Symbol("yc", complex=True)
yr = Symbol("yr", real=True)
for xx, yy in [(xr, yr), (xc, yc)]:
    subs_list(6 * xx**5 + xx**4 - 4 * xx**3 + 4 * xx**2 - 2 * xx + 3,
              [(xx**2, yy**2), (xx**4, yy**4)])
ee = c * yn * xn * yn * xn**(Rw * Sw - a2 * b2) - 4 * a2 * b2 * yn * xn * yn
subs_list(ee, [(xn * yn, L), (a2 * b2, Symbol("K")), (Rw * Sw, Uw)])
subs_list(x < -oo, [(-oo, 1), (oo, 2)])
subs_list(x < -oo, [(oo, 2), (-oo, 1)])
subs_list(x < oo, [(-oo, 2), (oo, 1)])
subs_list(x < oo, [(oo, 1), (-oo, 2)])
add("subs_str", [1 / x, [[PyStr("x"), PyStr("(1/2)")]]],
    lambda e, ps: e.subs({o: n for o, n in ps}))
add("subs_str", [Symbol("s"), [[PyStr("s"), 1]]],
    lambda e, ps: e.subs({o: n for o, n in ps}))
add("subs_str", [Symbol("var1"), [[PyStr("var1"), PyStr("x")]]],
    lambda e, ps: e.subs(*ps[0]))
Xf, Yf = Function("x")(t), Function("y")(t)
subs_dict(Xf * Yf, [(Xf, 1 + Xf), (Yf, Xf)])
Wa, Wb = Wild("a"), Wild("b")
subs_dict(Wa / Wb * sin(Wb * x), [(Wa, 3), (Wb, 4)])
subs_set(Wa / Wb * sin(Wb * x), [(Wa, 3), (Wb, 4)])
subs_dict(x * exp(x), [(x, y), (exp(x), y)])
subs_dict(z * exp(z), [(z, y), (exp(z), y)])
subs_list(x * exp(x), [(x, y), (exp(x), y)])
subs_list(x * exp(x), [(exp(x), y), (x, y)])
subs_dict(exp(x / 2 + y), [(exp(y + 1), 2), (x, 2)])

# Basic(...) stand-ins
B1 = Tuple()
B2 = Tuple(B1)
B3 = Tuple(B2)
B21 = Tuple(B2, B1)
for e, o, n in [(B21, B2, B1), (B21, B2, B21), (B3, B2, B1), (B21, B1, x),
                (Tuple(x, B21), B1, y)]:
    subs_pair(e, o, n)
subs_list(B21, [(B2, B1), (B1, B2)])
subs_dict(B21, [(B1, B2), (B2, B1)])
for e, rule in [(B21, {B2: B1}), (B21, {B2: B21}), (B3, {B2: B1}),
                (B21, {B1: B2, B2: B1})]:
    add("xreplace", [e, rule], lambda e, ps: e.xreplace({o: n for o, n in ps}))
for ps in [[B1], [B3, B1], [Basic], [B21, B3], []]:
    add("has", [B21, ps], lambda e, ps: e.has(*ps))
add("atoms", [B21, []], lambda e, ts: sorted_set(e.atoms(*ts)))
add("free_symbols", [B21], lambda e: sorted_set(e.free_symbols))
BE = Tuple(B21, B3)
add("preorder", [BE], lambda e: list(preorder_traversal(e)), keys="none")


def skipping_eq(e, node):
    out = []
    pt = preorder_traversal(e)
    for i in pt:
        out.append(i)
        if i == node:
            pt.skip()
    return out


add("preorder_skip_eq", [BE, B2], skipping_eq)
add("preorder_skip_eq", [x + y * (z + 1), y * (z + 1)], skipping_eq)
add("preorder_skip_eq", [x + y * (z + 1), z + 1], skipping_eq)
for e in [Piecewise((x, x < 1), (x**2, True)), Integral(x**2, (x, 0, 1))]:
    for keys, kw in [("none", {}), ("true", {"keys": True}),
                     ("default_sort_key", {"keys": default_sort_key})]:
        add("preorder", [e], lambda e, kw=kw: list(preorder_traversal(e, **kw)),
            keys=keys)
        add("postorder", [e],
            lambda e, kw=kw: list(postorder_traversal(e, **kw)), keys=keys)

# test_expr.py: atoms, is_polynomial, is_rational_function
kk = Symbol("k", nonnegative=True, integer=True)
for e, ts in [
    (x, []), (1 + x, []), (1 + 2 * cos(x), [Symbol]),
    (1 + 2 * cos(x), [Symbol, Number]), (2 * (x**(y**x)), []),
    (S.Half, []), (S.Half, [Symbol]), (I * pi, [NumberSymbol]),
    (I * pi, [NumberSymbol, I]), (I * pi, [I, NumberSymbol]),
    (exp(exp(x)), [exp]), (1 + x * (2 + y) + exp(3 + z), [Add]),
    (f(x) + sin(x) + 2, [AppliedUndef]),
    (f(x) + sin(x) + 2, [AppliedUndef, Function]),
    (f(x) + sin(x) + 2, [Function]), (f(x) + sin(x) + 2, [AppliedUndef, Number]),
    (f(x) + sin(x) + 2, [Function, Number]),
]:
    add("atoms", [e, ts], lambda e, ts: sorted_set(e.atoms(*ts)))
for e, ss in [
    (Rational(2), [x, y, z]), (S.Pi, [x, y, z]), (x, [x]), (x, [y]),
    (x**2, [x]), (x**2, [y]), (x**(-2), [x]), (x**(-2), [y]), (2**x, [x]),
    (2**x, [y]), (x**kk, [x]), (x**kk, [kk]), (x**x, [x]), (kk**kk, [kk]),
    (kk**x, [kk]), (x**(-kk), [x]), ((2 * x)**kk, [x]),
    (x**2 + 3 * x - 8, [x]), (x**2 + 3 * x - 8, [y]), (x**2 + 3 * x - 8, []),
    (sqrt(x), [x]), (sqrt(x)**3, [x]), (x**2 + 3 * x * sqrt(y) - 8, [x]),
    (x**2 + 3 * x * sqrt(y) - 8, [y]),
    ((x**2) * (y**2) + x * (y**2) + y * x + exp(2), []),
    ((x**2) * (y**2) + x * (y**2) + y * x + exp(x), []),
    ((x**2) * (y**2) + x * (y**2) + y * x + exp(2), [x, y]),
    ((x**2) * (y**2) + x * (y**2) + y * x + exp(x), [x, y]),
    (1 / f(x) + 1, [f(x)]),
]:
    add("is_polynomial", [e, ss], lambda e, ss: e.is_polynomial(*ss))
for e, ss in [
    (Integer(1), []), (Integer(1), [x]), (Rational(17, 54), []),
    (Rational(17, 54), [x]), (12 / x, []), (12 / x, [x]), (x / y, []),
    (x / y, [x]), (x / y, [x, y]), (x**2 + 1 / x / y, []),
    (x**2 + 1 / x / y, [x]), (x**2 + 1 / x / y, [x, y]), (sin(y) / x, []),
    (sin(y) / x, [y]), (sin(y) / x, [x]), (sin(y) / x, [x, y]),
] + [(i / dd, []) for i in (nan, oo, -oo, zoo) for dd in (1, x)] + [
    (i, []) for i in (nan, oo, -oo, zoo)]:
    add("is_rational_function", [e, ss],
        lambda e, ss: e.is_rational_function(*ss))

# test_expr.py: as_independent
n1, n2, n3c = symbols("n1 n2 n3", commutative=False)
for e, deps, as_add in [
    (S.Zero, [x], True), (S.Zero, [x], False),
    (2 * x * sin(x) + y + x, [x], None), (2 * x * sin(x) + y + x, [y], None),
    (2 * x * sin(x) + y + x, [x, y], None), (x * sin(x) * cos(y), [x], None),
    (x * sin(x) * cos(y), [y], None), (x * sin(x) * cos(y), [x, y], None),
    (sin(x), [x], None), (sin(x), [y], None), (2 * sin(x), [x], None),
    (2 * sin(x), [y], None), (n1 + n1 * n2, [n2], None),
    (n2 * n1 + n1 * n2, [n2], None), (n1 * n2 * n1, [n2], None),
    (n1 * n2 * n1, [n1], None), (3 * x, [x], True), (3 * x, [x], False),
    (3 + x, [x], True), (3 + x, [x], False), (3 * x, [Symbol], None),
    (n1 * x * y, [x], None), ((x + n1) * (x - y), [x], None),
    ((x + n1) * (x - y), [y], None),
    (DiracDelta(x - n1) * DiracDelta(x - y), [x], None),
    (x * y * n1 * n2 * n3c, [n2], None), (x * y * n1 * n2 * n3c, [n1], None),
    (x * y * n1 * n2 * n3c, [n3c], None),
    (DiracDelta(x - n1) * DiracDelta(y - n1) * DiracDelta(x - n2), [y], None),
    (Add(x, -x, 2, -3, evaluate=False), [x], None),
    (Mul(x, 1 / x, 2, -3, evaluate=False), [x], None),
    (x * y, [z], True),
]:
    add("as_independent", [e, deps],
        lambda e, ds, as_add=as_add: (
            e.as_independent(*ds) if as_add is None
            else e.as_independent(*ds, as_Add=as_add)),
        as_Add=as_add)

# test_expr.py: has, has_free, has_xfree, free_symbols
Wp = Wild("p")
for e, ps in [
    (sin(x), [x]), (sin(x), [sin]), (sin(x), [y]), (sin(x), [cos]),
    (f(x), [x]), (f(x), [f]), (f(x), [y]), (f(x), [g]),
    (Derivative(f(x), x), [x]), (Derivative(f(x), x), [f]),
    (Derivative(f(x), x), [Derivative]), (Derivative(f(x), x), [y]),
    (Derivative(f(x), x), [g]), (Derivative(f(x), x), [sin]),
    (x**2, [Symbol]), (x**2, [Wild]), (2 * Wp, [Wild]), (x, []),
    (S(1), [Wild]), (x, [Wild]),
    (x**2 * y + sin(2**t + log(z)), [x]), (x**2 * y + sin(2**t + log(z)), [t]),
    (x**2 * y + sin(2**t + log(z)), [w]),
    (x**2 * y + sin(2**t + log(z)), [x, y, z, t, w]),
    (Integer(4400), [x]), (4400 * x**4400, [x]), (4400 * y**4400, [x]),
    (4400 * y**4400, [x, y]), (4400 * y**4400, [x, z]),
    (Integral(x**2 + sin(x * y * z), (x, 0, x + y + z)), [x + y]),
    (Integral(x**2 + sin(x * y * z), (x, 0, x + y + z)), [x * z]),
    (Integral(x**2 + sin(x * y * z), (x, 0, x + y + z)), [2 * x + y]),
    (Integral(x**2 + sin(x * y * z), (x, 0, x + y + z)), [2 * x * y]),
    (Tuple(x, y), [x]), (Tuple(x, y), [z]), (Tuple(f(x), g(x)), [x]),
    (Tuple(f(x), g(x)), [y]), (Tuple(f(x), g(x)), [f]),
    (Tuple(f(x), g(x)), [f(x)]), (Tuple(True), [S.true]), (Tuple(True), [1]),
]:
    add("has", [e, ps], lambda e, ps: e.has(*ps))
HI = x * g(x) * sin(x) * exp(x * y) * A * B * C * cos(x * A * B)
for pt in [x, x * y, x * sin(x), x * sin(y), x * A, x * A * B, x * A * C,
           x * A * B * C, x * A * C * B, x * sin(x) * A * B * C,
           x * sin(x) * A * C * B, x * sin(y) * A * B * C, x * g(x),
           x + sin(x)]:
    add("has", [HI, [pt]], lambda e, ps: e.has(*ps))
for e, ps in [
    (x, [x]), (x, [y]), (x + y, [x]), (x + y, [x, z]), (f(x), [x]),
    (f(x), [f(x)]), (Integral(f(x), (f(x), 1, y)), [y]),
    (Integral(f(x), (f(x), 1, y)), [x]), (Integral(f(x), (f(x), 1, y)), [f(x)]),
    (x + 1 + y, [x + 1]), (x + 2 + y, [x + 1]), (2 + 3 * x * y, [3 * x]),
]:
    add("has_free", [e, ps], lambda e, ps: e.has_free(*ps))
for e, s_ in [(x + 1, [x]), ((x + 1)**2, [x + 1]), (x + y + 1, [x + 1])]:
    add("has_xfree", [e, s_], lambda e, ps: e.has_xfree(set(ps)))
for e in [S.One, x, Integral(x, (x, 1, y))]:
    add("free_symbols", [e], lambda e: sorted_set(e.free_symbols))

# test_expr.py: coeff
nc_n, nc_m, nc_o, nc_l = symbols("n m o l", commutative=False)
for e, xx, n, right in [
    (x + 1, x + 1, 1, False), (3 * x, 0, 1, False),
    (z * (1 + x) * x**2, 1 + x, 1, False),
    (1 + 2 * x * x**(1 + x), x * x**(1 + x), 1, False),
    (1 + 2 * x**(y + z), x**(y + z), 1, False),
    (3 + 2 * x + 4 * x**2, 1, 1, False), (3 + 2 * x + 4 * x**2, -1, 1, False),
    (3 + 2 * x + 4 * x**2, x, 1, False), (3 + 2 * x + 4 * x**2, x**2, 1, False),
    (3 + 2 * x + 4 * x**2, x**3, 1, False), (-x / 8 + x * y, x, 1, False),
    (-x / 8 + x * y, -x, 1, False), (4 * x, 2 * x, 1, False),
    (2 * x, 2 * x, 1, False), (-oo * x, x * oo, 1, False),
    (10 * x, x, 0, False), (10 * x, 10 * x, 0, False),
    (n1 * n2, n1, 1, False), (n1 * n2, n2, 1, False),
    (n1 * n2 + x * n1, n1, 1, False), (n2 * n1 + x * n1, n1, 1, False),
    (n2 * n1 + x * n1**2, n1, 1, False), (n1**x, n1, 1, False),
    (n1 * n2 + n2 * n1, n1, 1, False), (2 * (n1 + n2) * n2, n1 + n2, 1, True),
    (2 * (n1 + n2) * n2, n1 + n2, 1, False),
    (2 * f(x) + 3 * Derivative(f(x), x), f(x), 1, False),
    (z * (x + y)**2, z, 1, False), (z * (x + y)**2, x + y, 1, False),
    (z * (x + y)**2 + z * (2 * x + 2 * y)**2, z, 1, False),
    (x + y + 3 * z, 1, 1, False), (-x + 2 * y, -1, 1, False),
    (x - 2 * y, -1, 1, False), (-x - 2 * y, 2, 1, False),
    (x + sqrt(2) * x, sqrt(2), 1, False), (z * (x + y)**2, (x + y)**2, 1, False),
    (2 + 2 * x + (x + 1) * y, x + 1, 1, False), (x + 2 * y + 3, 1, 1, False),
    (x + 2 * y + 3, x, 0, False), (x**2 + 2 * y + 3 * x, x**2, 0, False),
    (x, 0, 0, False), (x, x, 0, False), (nc_n, nc_n, 1, False),
    (y, nc_n, 1, False), (3 * nc_n, nc_n, 1, False),
    (2 + nc_n, x * nc_m, 1, False), (2 * x * nc_n * nc_m, x, 1, False),
    (2 + nc_n, x * nc_m * nc_n + y, 1, False),
    (2 * x * nc_n * nc_m, 3 * nc_n, 1, False),
    (nc_n * nc_m + nc_m * nc_n * nc_m, nc_n, 1, False),
    (nc_n * nc_m + nc_m * nc_n * nc_m, nc_n, 1, True),
    (nc_n * nc_m + nc_m * nc_n, nc_n, 1, False),
    (nc_n * nc_m + nc_o * nc_m * nc_n, nc_m * nc_n, 1, False),
    (nc_n * nc_m + nc_o * nc_m * nc_n, nc_m * nc_n, 1, True),
    (nc_n * nc_m + nc_n * nc_m * nc_n, nc_n * nc_m, 1, True),
    (x * y, z, 0, False), (x * nc_n + y * nc_n + z * nc_m, nc_n, 1, False),
    (nc_n * nc_m + nc_n * nc_o + nc_o * nc_l, nc_n, 1, True),
    (x * nc_n * nc_m * nc_n + y * nc_n * nc_m * nc_o + z * nc_l, nc_m, 1, True),
    (x * nc_n * nc_m * nc_n + x * nc_n * nc_m * nc_o + z * nc_l, nc_m, 1, True),
    (x * nc_n * nc_m * nc_n + x * nc_n * nc_m * nc_o + z * nc_l, nc_m, 1, False),
]:
    add("coeff", [e, xx], lambda e, xx, n=n, right=right:
        e.coeff(xx, n, right=right), n=n, right=right)

# ---------------------------------------------------------------------------
# keep only seed-independent, Dummy-free answers


def keep(v):
    out = v["out"]
    if "expr" in out:
        return no_dummies(dec(out["expr"]))
    return True


KEPT = [v for v in VECTORS if keep(v)]
assert len(KEPT) == len(VECTORS), "a result holds a Dummy"
VECTORS = KEPT

write("subs", VECTORS,
      "Basic.subs/_subs/_eval_subs/xreplace, has/atoms/free_symbols, "
      "traversals, as_independent, coeff, is_polynomial "
      "(oracle/gen_diff_subs.py)")
