"""Differential vectors for rsympy-core's `sympy.core.exprtools` and the
content/primitive methods it is built on.

    PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python python oracle/gen_diff_exprtools.py

Writes oracle/diff/core_exprtools.json, replayed by
crates/rsympy-core/tests/diff_exprtools.rs. Covers `factor_terms` and
`gcd_terms` under every flag combination, `_gcd_terms`, `Factors` (built
from expressions and from dicts) and all its operations, `Term` and its
operations, `decompose_power(_rat)`, `_monotonic_sign`, `_mask_nc`,
`factor_nc`, and the `as_content_primitive`, `primitive`,
`as_powers_dict`, `extract_multiplicatively` and `extract_additively`
methods, over sums with rational and Float coefficients, rational-exponent
powers, nested Adds and Muls, non-commutative symbols, and the
expressions of sympy/core/tests/test_exprtools.py.

A Factors is recorded as its dict (values that are Python ints stay ints),
a Term as `(coeff, numer, denom)`. Results that contain Dummies made during
the call are recorded as text with each Dummy renumbered by first
appearance (see `canon`), so they do not depend on the global Dummy count;
each such vector is also replayed with a shifted count and dropped if its
answer changes.
"""

import itertools
import random
import re

from sympy import (Add, E, Float, Function, I, Integer, Integral, Interval,
                   Mul, Pow, Rational, S, Sum, Symbol, Tuple, exp, log, oo,
                   pi, root, sin, sqrt, symbols)
from sympy.core.basic import Basic
from sympy.core.exprtools import (Factors, Term, _gcd_terms, _mask_nc,
                                  _monotonic_sign, decompose_power,
                                  decompose_power_rat, factor_nc,
                                  factor_terms, gcd_terms)
from sympy.core.symbol import Dummy

from exprenc import dec, enc, vector, write

a, b, t, x, y, z = symbols("a b t x y z")
p = Symbol("p", positive=True)
q = Symbol("q", positive=True)
n = Symbol("n", negative=True)
k = Symbol("k", integer=True)
nn = Symbol("nn", nonnegative=True)
np_ = Symbol("np", nonpositive=True)
pk = Symbol("pk", positive=True, integer=True)
qk = Symbol("qk", positive=True, integer=True)
nk = Symbol("nk", negative=True, integer=True)
pe = Symbol("pe", positive=True, even=True)
pr = Symbol("pr", prime=True)
pro = Symbol("pro", prime=True, odd=True)
cm = Symbol("cm", composite=True)
cmo = Symbol("cmo", composite=True, odd=True)
ne = Symbol("ne", negative=True, even=True)
zr = Symbol("zr", zero=True)
A, B, C = symbols("A B C", commutative=False)
f = Function("f")
half, third = Rational(1, 2), Rational(1, 3)

# sympy/core/tests/test_exprtools.py and the docstrings of exprtools,
# add.py, mul.py and power.py
TEST_EXPRS = [
    9 * (x + x * y + 1) + (3 * x + 3)**(2 + 2 * x),
    9 * (x + x * y + 1) + 3**(2 + 2 * x),
    3**(2 + 2 * x) + a * 3**(2 + 2 * x),
    x + x * A,
    (3 * x + 3)**((2 + 2 * x) / 3),
    x + (x * y + x)**(3 * x + 3),
    a * (x + x * y) + b * (x * 2 + y * x * 2),
    x / 2 + y,
    sqrt(2) + sqrt(10),
    root(-6, 3) + root(6, 3),
    1 / sqrt(a / 2 + 1),
    x / (x + 1 / x) + 1 / (x**2 + 1),
    (1 / (x**3 + x**2) + 2 / x**2) * y,
    -x - y,
    exp(-x - 2) + x,
    exp(-4 * x - 2) - x,
    exp(Mul(-1, x + 1, evaluate=False)),
    0**(x - 2) - 1,
    0**(x + 2) - 1,
    x + x * (2 + 4 * y)**3,
    x * A + x * A + x * y * A,
    -2 * x - 2 * y,
    2 * (x + 1) * (x + 4) / (5 * x**2 + 5) + (2 * x + 2) * (x + 5) / (x**2 + 1) / 5
    + (2 * x + 2) * (x + 6) / (5 * x**2 + 5),
    (2 * x + 2)**3 + (2 * x + 2)**2,
    2 + 2 * x,
    x * (2 * x + 4 * y),
    sin(x * (2 * x + 4 * y)),
    sin(x + x * A),
    x / (x + 1 / x),
    x / 2 / y + 1 / x / y,
    (x + 1)**2 * y + (x + 1) * y**2,
    x / 2 + 1, x / 2 + y / 2, x / 2 + 1 / x,
    3 + 3 * sqrt(2),
    2 * sqrt(2) + 4 * sqrt(10),
    (2 + 2 * x) * x + 2,
    sqrt(4 + 4 * sqrt(2)),
    sqrt(3 + 3 * sqrt(2)),
    (2 * x + 2)**2,
    4**((1 + y) / 2), 3**((1 + y) / 2), 3**((5 + y) / 2), 3**(2 + 2 * x),
    (2 + 2 * x)**y,
    -3 * sqrt(2) * (2 - 2 * sqrt(2)),
    2 * x + 4 * y,
    2 * x / 3 + 4 * y / 9,
    2 * x / 3 + Float("4.2") * y,
    (2 + 2 * x) * (3 + 3 * x),
    a + b + (a + b)**2 * (-2 * a - 2 * b),
    Sum(x, (y, 1, 10)),
    Sum(x, (y, 1, 10)) + x,
    Sum(x * y + x * y**2, (y, 1, 10)),
    Sum(0, (y, 1, 10)),
    Integral(x, (y, 1, 10)),
    Integral(x, (x, 0, oo)),
    Interval(0, 1),
    Tuple(x + x * y),
    Tuple(2 * x + 2, 3 * y),
    f(2 * x + 2) + 2,
    x * f(x) + 2 * x,
    (2 * x + 2)**Rational(3, 2),
    sqrt(8 * x + 8),
    (4 * x**2 + 4)**third,
    Pow(4, -half, evaluate=False) * x + x,
    Mul(2, x + 1, evaluate=False) + 2 * y,
    Add(2 * x, 2 * x, evaluate=False),
    6 * sqrt(3) + 4 * sqrt(6) + 2 * sqrt(15),
    sqrt(2) * x + sqrt(6) * y,
    2**third * x + 4**third * y,
    Pow(-2, third) * x + Pow(-16, third),
    sqrt(2) / 2 + sqrt(3) / 3,
    x * sqrt(12) + sqrt(18),
    3**Rational(2, 3) * x + 3**third * y,
    2 * sqrt(2) * x + 6 * sqrt(3) * y,
    Float("1.5") * x + 3 * y,
    Float("0.5") * x + Float("0.25") * y,
    Float("2.0") * x + 4,
    -Float("1.5") * x - Float("1.5"),
    oo * x + oo,
    x + oo,
    I * x + I,
    2 * I * x + 4 * I,
    pi * x + pi,
    2 * pi * x + 4 * pi,
    E * x + 2 * E,
    -x / 3 - y / 6,
    x**2 / 4 - y**2 / 9,
    (x / 2 + y / 3) * (x / 4 + 1),
    (3 * x + 6)**2 * (2 * y + 4)**3,
    (x + 1)**(2 * x + 4) + 3,
    x**(2 * y + 2) + x**2,
    2**(2 * x + 2) - 8,
    p * x + p * y,
    2 * p + 2 * n,
    -p - n,
]

COEFS = [Integer(1), Integer(-1), Integer(2), Integer(-2), Integer(3),
         Integer(4), Integer(6), Integer(-9), half, -half, third,
         Rational(2, 3), Rational(-3, 4), Rational(5, 6), Float("1.5"),
         Float("0.5"), Float("-2.0")]
BASES = [x, y, z, a, p, n, k, nn]
NUMBASES = [Integer(2), Integer(3), Integer(5), Integer(6), Integer(10),
            Integer(12), Integer(-6), Integer(18)]
EXPS = [Integer(2), Integer(3), Integer(-1), Integer(-2), half, third,
        Rational(2, 3), Rational(3, 2), -half, Rational(-1, 3)]
RADEXPS = [half, third, Rational(2, 3), -half]


class Gen:
    """Random expressions from a fixed seed."""

    def __init__(self, seed, coefs=COEFS, nc=False):
        self.rng = random.Random(seed)
        self.coefs = coefs
        self.nc = nc

    def monomial(self):
        rng = self.rng
        factors = []
        for _ in range(rng.randint(1, 3)):
            r = rng.random()
            if r < 0.45:
                factors.append(rng.choice(BASES))
            elif r < 0.7:
                factors.append(rng.choice(BASES)**rng.choice(EXPS))
            else:
                factors.append(rng.choice(NUMBASES)**rng.choice(RADEXPS))
        if self.nc and rng.random() < 0.6:
            factors.append(rng.choice([A, B, A * B, B * A, A**2, C]))
        return Mul(*factors)

    def term(self):
        return self.rng.choice(self.coefs) * self.monomial()

    def sum(self, depth=0):
        rng = self.rng
        terms = [self.term() for _ in range(rng.randint(2, 4))]
        if depth < 2 and rng.random() < 0.4:
            terms.append(rng.choice(self.coefs) * self.sum(depth + 1)
                         * rng.choice([S.One, self.monomial()]))
        if rng.random() < 0.3:
            terms.append(rng.choice(self.coefs))
        return Add(*terms)

    def expr(self):
        rng = self.rng
        r = rng.random()
        if r < 0.35:
            return self.sum()
        if r < 0.5:
            return rng.choice(self.coefs) * self.sum() * self.sum()
        if r < 0.65:
            return self.sum()**rng.choice(EXPS + [x, 2 * x + 2, x / 2 + half])
        if r < 0.75:
            return self.sum() / self.sum()
        if r < 0.85:
            return self.term() + self.sum()**rng.choice(EXPS)
        return Add(*[self.term() * self.sum() for _ in range(rng.randint(2, 3))])


def unique(seq):
    seen = set()
    out = []
    for e in seq:
        if e not in seen:
            seen.add(e)
            out.append(e)
    return out


RANDOM = unique([Gen(1).expr() for _ in range(130)])
RANDOM_NC = unique([Gen(2, nc=True).expr() for _ in range(25)])
RANDOM_INT = unique([Gen(3, coefs=COEFS[:10]).expr() for _ in range(40)])

ATOMS = [S.Zero, S.One, S.NegativeOne, Integer(2), Integer(-6), half,
         Rational(-3, 4), Float("1.5"), Float("-2.0"), Float(0), S.Infinity,
         S.NegativeInfinity, S.ComplexInfinity, S.NaN, I, pi, E, x, p, n, A]
SIMPLE = [2 * x, -x, -3 * x, x * y, 6 * x * y, x**2, 1 / x, x / 2, -x / 3,
          sqrt(2), 2**third, 3 * sqrt(2), sqrt(x), x**half * y, x**(2 * y),
          x**(2 * y / 3), x**(-2 * y), (x + 1)**2, 2**(2 * x + 2), I * x,
          -I, 2 * I, pi * x, A * B, 2 * A, x * A, -2 * x * A * B,
          Mul._from_args([S.NegativeInfinity, x]), Pow(x, 0, evaluate=False),
          Mul(2, x + 1, evaluate=False), exp(2 * y / 3), exp(-x),
          sqrt(I) * I, I**Rational(3, 2), (-2.0)**x, 27 * x / y**9,
          3 * x / 2, x * x / y, x**sqrt(2)**3, (x**2)**half,
          x**(x + 3), x**(2 + y)]
SUMS = [x + 1, 2 * x + 2, 2 * x + 4 * y, x / 2 + y / 3, -x - y, -2 * x - 4,
        x + y + z, 3 * x * y + 6 * x, x**2 + 2 * x + 1, sqrt(2) + 2,
        Float("1.5") * x + 3]
EXPRS = unique(ATOMS + SIMPLE + SUMS + TEST_EXPRS + RANDOM + RANDOM_NC
               + RANDOM_INT)

FLAGS4 = list(itertools.product([False, True], repeat=4))
FLAGS3 = list(itertools.product([False, True], repeat=3))
FLAGS2 = list(itertools.product([False, True], repeat=2))


def fresh(v):
    """A fresh copy of `v` (vector() only copies its top-level Basics)."""
    if isinstance(v, Basic):
        return dec(enc(v))
    if isinstance(v, dict):
        return {fresh(kk): fresh(vv) for kk, vv in v.items()}
    if isinstance(v, (list, tuple)):
        return type(v)(fresh(i) for i in v)
    return v


def fout(fs):
    """A Factors as its dict."""
    return dict(fs.factors)


def tout(tm):
    """A Term as `(coeff, numer, denom)`."""
    return (tm.coeff, fout(tm.numer), fout(tm.denom))


def factors(v):
    """The Factors of a vector argument: from a dict or an expression."""
    return Factors(fresh(v) if isinstance(v, dict) else v)


def render(v):
    """Text for a result, written the same way by diff_exprtools.rs."""
    if isinstance(v, Basic):
        return enc(v)
    if v is None:
        return "None"
    if isinstance(v, bool):
        return repr(v)
    if isinstance(v, int):
        return str(v)
    if isinstance(v, tuple):
        return "(" + ", ".join(render(i) for i in v) + ")"
    if isinstance(v, list):
        return "[" + ", ".join(render(i) for i in v) + "]"
    if isinstance(v, dict):
        return "{" + ", ".join(f"{render(kk)}: {render(vv)}"
                               for kk, vv in v.items()) + "}"
    raise TypeError(type(v))


_DUMMY = re.compile(r"Dummy\(('(?:[^'\\]|\\.)*')((?:, \w+=\w+)*), "
                    r"dummy_index=(\d+)\)")


def canon(text):
    """Renumbers the Dummies in `text` by first appearance, and writes the
    automatic names `Dummy_<count>` as `_`."""
    seen = {}

    def sub(m):
        name, rest, index = m.groups()
        key = (name, index)
        if key not in seen:
            seen[key] = len(seen)
        if re.fullmatch(r"'Dummy_\d+'", name):
            name = "'_'"
        return f"Dummy({name}{rest}, dummy_index={seen[key]})"

    return _DUMMY.sub(sub, text)


DROPPED = []
UNENCODABLE = []


def vec(op, args, fn, **params):
    """`vector(...)`, or None when SymPy's answer has no lossless encoding
    (a Sum or Integral inside an Add or Mul does not rebuild unevaluated)."""
    try:
        return vector(op, args, fn, **params)
    except (AssertionError, AttributeError):
        UNENCODABLE.append(op)
        return None


def dummy_vector(op, args, fn, **params):
    """A vector whose result may hold new Dummies: recorded through
    `canon`, and dropped unless a shifted Dummy count gives the same text."""
    def run(*a):
        return canon(render(fn(*a)))
    v1 = vec(op, args, run, **params)
    Dummy._count += 1000
    v2 = vec(op, args, run, **params)
    if v1 is None or v2 is None:
        return None
    if v1["out"] != v2["out"]:
        DROPPED.append(op)
        return None
    return v1


def main():
    vectors = []

    def add(v):
        if v is not None:
            vectors.append(v)

    # factor_terms, every flag combination
    for e in EXPRS:
        for radical, clear, fraction, sign in FLAGS4:
            add(vec("factor_terms", [e],
                       lambda e, r=radical, c=clear, fr=fraction, s=sign:
                       factor_terms(e, radical=r, clear=c, fraction=fr, sign=s),
                       radical=radical, clear=clear, fraction=fraction,
                       sign=sign))
    for seq in [[x + x * y], [2 * x + 2, 3 * y + 3 * x * y], []]:
        for typ in (list, tuple):
            add(vec("factor_terms_seq", [typ(seq)],
                       lambda s: factor_terms(fresh(s)), kind=typ.__name__))

    # gcd_terms and _gcd_terms
    for e in EXPRS:
        for isprimitive, clear, fraction in FLAGS3:
            add(vec("gcd_terms", [e],
                       lambda e, i=isprimitive, c=clear, fr=fraction:
                       gcd_terms(e, isprimitive=i, clear=c, fraction=fr),
                       isprimitive=isprimitive, clear=clear,
                       fraction=fraction))
        for isprimitive, fraction in FLAGS2:
            add(vec("_gcd_terms", [e],
                       lambda e, i=isprimitive, fr=fraction:
                       _gcd_terms(e, isprimitive=i, fraction=fr),
                       isprimitive=isprimitive, fraction=fraction))
    term_lists = [list(Add.make_args(e)) for e in TEST_EXPRS[:30] + RANDOM[:30]]
    term_lists += [[S.Zero, S.Zero], [2 * x + 4], [], [x], [2 * x, 4 * y],
                   [x / 2, 1 / x], [x * A, 2 * x * A], [A, B]]
    for terms in term_lists:
        for isprimitive, clear, fraction in FLAGS3:
            add(vec("gcd_terms_seq", [terms],
                       lambda s, i=isprimitive, c=clear, fr=fraction:
                       gcd_terms(fresh(s), isprimitive=i, clear=c,
                                 fraction=fr),
                       isprimitive=isprimitive, clear=clear,
                       fraction=fraction))
        add(vec("_gcd_terms_seq", [terms],
                   lambda s: _gcd_terms(fresh(s))))

    # as_content_primitive, primitive, as_powers_dict
    for e in EXPRS:
        for radical, clear in FLAGS2:
            add(vec("as_content_primitive", [e],
                       lambda e, r=radical, c=clear:
                       e.as_content_primitive(radical=r, clear=c),
                       radical=radical, clear=clear))
        add(vec("primitive", [e], lambda e: e.primitive()))
        add(vec("as_powers_dict", [e], lambda e: dict(e.as_powers_dict())))

    # extract_multiplicatively, extract_additively
    mfactors = [Integer(2), Integer(-1), Integer(3), half, Integer(-2),
                Float("2.0"), x, 2 * x, -x, x + 1, 2 * x + 2, x**2, sqrt(2),
                I, S.Infinity, S.NegativeInfinity, S.Zero, x * y, 4 * x * y]
    mexprs = unique(ATOMS + SIMPLE + SUMS + [2 * x * y + 4 * x, 6 * x**2,
                                              4 * (x + 1), x**3 * y,
                                              (2 * x + 2)**2])
    for e in mexprs:
        for c in mfactors:
            add(vec("extract_multiplicatively", [e, c],
                       lambda e, c: e.extract_multiplicatively(c)))
    anumbers = [S.Zero, S.One, Integer(3), Integer(-3), Integer(5), half,
                Rational(-2, 3), Float("0.1"), Float("2.5"), Float("-1.5"),
                S.Infinity, S.NegativeInfinity, S.NaN, I]
    for e in anumbers + [x + 3, 2 * x - 3, x + Float("1.5"), x, 2 * x + 3,
                         x + half]:
        for c in anumbers:
            add(vec("extract_additively", [e, c],
                       lambda e, c: e.extract_additively(c)))
    for e, c in [(2 * x + 3, x + 1), (2 * x + 3, 3 * x), (x, x), (x + y, y)]:
        add(vec("extract_additively", [e, c],
                   lambda e, c: e.extract_additively(c)))

    # Factors from expressions and dicts
    fexprs = unique(ATOMS + SIMPLE + SUMS + TEST_EXPRS[:40] + RANDOM[:60]
                    + RANDOM_NC[:10])
    for e in fexprs:
        add(vec("Factors", [e], lambda e: fout(Factors(e))))
        add(vec("Factors_as_expr", [e], lambda e: Factors(e).as_expr()))
        add(vec("Factors_flags", [e],
                   lambda e: (Factors(e).is_zero, Factors(e).is_one)))
    dicts = [
        {}, {x: Integer(2), y: Integer(3)}, {x: 2, y: 3},
        {S.NegativeOne: Integer(3)}, {S.NegativeOne: S.One, I: Integer(5)},
        {I: Rational(3, 2)}, {S.NegativeOne: Rational(3, 2)},
        {I: S.One, S.NegativeOne: third}, {I: 1}, {Float("-1.0"): 2, I: 1},
        {S.NegativeOne: Rational(-3, 2)}, {S.One: 5}, {S.One: x},
        {S.NegativeOne: x}, {I: x}, {S.NegativeOne: 1, S.One: 2},
        {S.NegativeOne: Float("0.5")}, {I: 2}, {I: half}, {I: 3, x: 1},
        {S.NegativeOne: 2, x: -1}, {x: 5, y: 3, z: 7},
        {y: 4, z: 3, t: 10}, {x: 4, y: 7, t: 7}, {z: 1, t: 3},
        {S(2): x}, {S(3): x}, {S(2): 2 * x + 2}, {S(8): S.One},
        {x: x + 6}, {x: y + 1}, {x: Integer(2), y: -1}, {x: 0},
        {S.Zero: S.One}, {S.Zero: 1, x: 1}, {x: half, y: -third},
        {x: Float("0.5")}, {S.NegativeOne: I}, {I: Float("2.0")},
    ]
    for d in dicts:
        add(vec("Factors_dict", [d], lambda d: fout(Factors(fresh(d)))))
        add(vec("Factors_dict_as_expr", [d],
                   lambda d: Factors(fresh(d)).as_expr()))

    # Factors operations, Factors and expression operands
    operands = [x, 2 * x, x**2, x * y**2, x * y / z, 1 / x, S.Zero, S.One,
                Integer(2), Integer(-6), half, x**(2 + y), x**2, x**y,
                2**x, 2**y, 2**(2 * x + 2), Integer(8), x**(x + 3), x**-3,
                x**3, x**4, x**(y - 3), x**(y + 3), x**(y + 4), sqrt(2) * x,
                -I, I, 3 * x / 2, I * x * y, x**half, x**(-half), (x + 1)**2,
                -x, Float("1.5") * x, x**(2 * y), A * x, oo]
    fdicts = [{x: 5, y: 3, z: 7}, {y: 4, z: 3, t: 10}, {x: 4, y: 7, t: 7},
              {z: 1, t: 3}, {x: Integer(2), y: 1}, {x: x + 3}, {x: 3},
              {x: Float("0.2")}, {x: -2, y: Integer(-1)}]
    pairs = list(itertools.product(operands[:28], operands[:28]))
    pairs = pairs[::3] + list(itertools.product(fdicts, fdicts))
    pairs += [(a_, b_) for a_ in operands for b_ in fdicts]
    ops = {
        "mul": lambda fa, fb: fout(fa.mul(fb)),
        "normal": lambda fa, fb: tuple(fout(r) for r in fa.normal(fb)),
        "div": lambda fa, fb: tuple(fout(r) for r in fa.div(fb)),
        "quo": lambda fa, fb: fout(fa.quo(fb)),
        "rem": lambda fa, fb: fout(fa.rem(fb)),
        "gcd": lambda fa, fb: fout(fa.gcd(fb)),
        "lcm": lambda fa, fb: fout(fa.lcm(fb)),
        "eq": lambda fa, fb: fa == fb,
    }
    for name, op in ops.items():
        for u, w in pairs:
            add(vec(f"Factors_{name}", [u, w],
                       lambda u, w, op=op: op(factors(u), factors(w))))
        for u, w in list(itertools.product(operands, operands))[::7]:
            if isinstance(w, Basic):
                add(vec(f"Factors_{name}_expr", [u, w],
                           lambda u, w, op=op: op(factors(u), w)))
    for u in operands + fdicts:
        for e in [0, 1, 2, 3, -1]:
            add(vec("Factors_pow", [u, e],
                       lambda u, e: fout(factors(u).pow(e))))
        for e in [S(2), S.Zero, x, Integer(-1)]:
            add(vec("Factors_pow_factors", [u, e],
                       lambda u, e: fout(factors(u).pow(Factors(e)))))
        add(vec("Factors_pow_expr", [u, Integer(2)],
                   lambda u, e: fout(factors(u).pow(e))))
    # normal against the defaultdict(int) of a Term, which looking up a
    # missing key changes
    tterms = [4 * x * y**2 / z / t**3, 2 * x**3 * y**5 / t**3, x / y,
              x**2 * y / z**3, 1 / x, x * y * z, S.One, 2 * x**(2 * y),
              x**(y + 1) / y**2, (x + 1) / (2 * x + 2)**2]
    for u, w in itertools.product(tterms, tterms):
        add(vec("Factors_normal_term", [u, w],
                   lambda u, w: (lambda fa, fb: (
                       tuple(fout(r) for r in fa.normal(fb)) + (fout(fb),)))(
                           Term(u).numer, Term(w).denom)))
        add(vec("Factors_normal_term_denom", [u, w],
                   lambda u, w: (lambda fa, fb: (
                       tuple(fout(r) for r in fa.normal(fb)) + (fout(fb),)))(
                           Term(u).denom, Term(w).numer)))

    # Term
    texprs = unique(tterms + [2 * x**3 * y**5 * t**7, (2 * x + 2)**3,
                              (2 * x + 2) * (3 * x + 6)**2, x, S.One, S.Zero,
                              Integer(2), Rational(-3, 4), x**(2 * y),
                              sqrt(x), x**(-half), exp(2 * x),
                              Float("1.5") * x, x * A, A, I * x, -x / 3,
                              (x / 2 + 1)**-2, sqrt(2) * x, oo * x]
                    + SIMPLE + RANDOM[:30])
    for e in texprs:
        add(vec("Term", [e], lambda e: tout(Term(e))))
        add(vec("Term_as_expr", [e], lambda e: Term(e).as_expr()))
        add(vec("Term_inv", [e], lambda e: tout(Term(e).inv())))
        for m in [0, 1, 3, -1, -3]:
            add(vec("Term_pow", [e, m], lambda e, m: tout(Term(e).pow(m))))
    tpairs = list(itertools.product(texprs[:24], texprs[:24]))
    tops = {
        "mul": lambda ta, tb: tout(ta.mul(tb)),
        "quo": lambda ta, tb: tout(ta.quo(tb)),
        "gcd": lambda ta, tb: tout(ta.gcd(tb)),
        "lcm": lambda ta, tb: tout(ta.lcm(tb)),
        "eq": lambda ta, tb: ta == tb,
    }
    for name, op in tops.items():
        for u, w in tpairs[::2]:
            add(vec(f"Term_{name}", [u, w],
                       lambda u, w, op=op: op(Term(u), Term(w))))
    for c, nd, dd in [(Integer(4), {x: 1, y: 2}, {z: 1, t: 3}),
                      (Integer(2), {x: 3, y: 5}, {t: 3}), (half, {}, {x: 2}),
                      (x, {y: 1}, {}), (Float("1.5"), {x: 1}, {y: 1}),
                      (S.Zero, {}, {})]:
        for other in [4 * x * y**2 / z / t**3, 2 * x, S.One, x / y]:
            add(vec("Term_parts_mul", [c, nd, dd, other],
                       lambda c, nd, dd, o: tout(Term(
                           c, Factors(fresh(nd)), Factors(fresh(dd))).mul(
                               Term(o)))))
            add(vec("Term_parts_gcd", [c, nd, dd, other],
                       lambda c, nd, dd, o: tout(Term(
                           c, Factors(fresh(nd)), Factors(fresh(dd))).gcd(
                               Term(o)))))
            add(vec("Term_parts_eq", [c, nd, dd, other],
                       lambda c, nd, dd, o: Term(
                           c, Factors(fresh(nd)), Factors(fresh(dd)))
                       == Term(o)))

    # decompose_power, decompose_power_rat
    for e in unique(ATOMS + SIMPLE + [x**(2 * y), x**(2 * y / 3),
                                      x**(y * Rational(2, 3)), x**(-y),
                                      x**(Float("2.0") * y), exp(-3 * x / 2),
                                      x**Float("1.5"), x**(I * y),
                                      x**Rational(-4, 3)]):
        add(vec("decompose_power", [e], decompose_power))
        add(vec("decompose_power_rat", [e], decompose_power_rat))

    # _monotonic_sign
    msyms = [x, p, n, k, nn, np_, pk, qk, nk, pe, pr, pro, cm, cmo, ne, zr]
    mexprs = list(msyms) + [-s for s in msyms] + [
        p + 1, pk - 1, p - 1, n + 1, nk - 1, n - 1, -p + 1, -pk - 1, -p - 1,
        -n + 1, -nk - 1, -n - 1, n**3, n**3 + log(2) * n - 1, -p**3, 1 / p,
        p / (p + 1), nn / (nn + 1), -1 / p, np_ / (-np_ + 1), -2 / pk / qk,
        -2 / (pk - 1) / qk, (pk - 1) * qk + 1, -(pk - 1) * qk - 1,
        sqrt(2) + 1, pi - 3, E - 3, sqrt(2) - 2, 1 - sqrt(3), pi + 1,
        2 * pi, -pi, Integer(3), Integer(-2), half, S.Zero, I, x + 1,
        x**2 + 1, p**2 + 2 * p + 1, p * q + 1, p + q + 1, -p - q - 2,
        p**half + 1, 2**p + 1, 1 / (p + q) + 1, p / q + 1, pk * qk - 1,
        # (fixed indices: an automatic dummy_index is random per process)
        Dummy("d", positive=True, dummy_index=1) + 1,
        Dummy("dp", prime=True, dummy_index=2),
        Dummy("dco", composite=True, odd=True, dummy_index=3),
        -Dummy("dn", negative=True, dummy_index=4) + 1,
    ]
    for e in mexprs:
        add(dummy_vector("_monotonic_sign", [e], _monotonic_sign))

    # _mask_nc, factor_nc
    F_, G_ = symbols("F G", commutative=False, cls=Function)
    ncexprs = [A**2 - x**2, A**2 - B**2, x * A + y * B, A + B + C, A, x,
               x + 1, 2 * A * B - B * A, A * B * C, x * A * B + A,
               (A + 1)**2, (A + x)**2, sin(A) * B + B * sin(A),
               F_(x) * G_(y) - G_(y) * F_(x), A * x + A * y, (x + A) * B]
    for e in ncexprs:
        for name in [None, "d"]:
            add(dummy_vector("_mask_nc", [e], lambda e, nm=name: _mask_nc(e, nm),
                             name=name))
    fnc = ncexprs + [x * (1 + y), n * (x + 1), x * y * z, sin(x * y),
                     (x + 1)**2, x**(y + 1), Tuple(x, y), S.One, x,
                     -n * (2 * x**2 + 2 * x), 2 * x * A]
    for e in fnc:
        add(vec("factor_nc", [e], factor_nc))

    write("exprtools", vectors, __doc__.split("\n\n")[0])
    if UNENCODABLE:
        print(f"skipped {len(UNENCODABLE)} vectors with unencodable answers: "
              f"{sorted(set(UNENCODABLE))}")
    if DROPPED:
        print(f"dropped {len(DROPPED)} vectors whose Dummies depend on the "
              f"Dummy count: {sorted(set(DROPPED))}")


if __name__ == "__main__":
    main()
