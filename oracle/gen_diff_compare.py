"""Differential vectors for rsympy-core's canonical orderings and atom printing.

    PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python python oracle/gen_diff_compare.py

Writes oracle/diff/core_compare.json, replayed by
crates/rsympy-core/tests/diff_compare.rs. Covers `Basic.compare`,
`sort_key` and Python's `<`/`==` on sort keys, `sorted(key=default_sort_key)`,
`ordered`, `sort(key=cmp_to_key(Basic.compare))`, `as_ordered_terms` (all
monomial orders) and `as_ordered_factors`, `decompose_power`, Python's
`==`/`<`/`>` between Numbers, `complex()`/`float()` of numbers, `str()` of
atoms (with StrPrinter settings), and CPython's `list.sort` itself on ints
and strings, including comparators that are not orders, that raise, and
`reverse=True`.

Several hundred expressions; comparisons over many pairs are recorded as
matrices (one character per pair) to keep the file small.
"""

import random
from functools import cmp_to_key

from sympy import (Abs, And, Basic, Derivative, Dummy, E, Eq, EulerGamma,
                   Float, Function, GoldenRatio, I, Integer, Integral,
                   Interval, Lambda, Max, Min, Mod, Mul, Ne, Not, O, Or,
                   Piecewise, Pow, Rational, S, Symbol, Tuple, Wild, atan2,
                   Catalan, FiniteSet, cos, exp, gamma, log, nan, oo, pi, sin,
                   sqrt, symbols, zoo, TribonacciConstant, Add)
from sympy.core.exprtools import decompose_power
from sympy.core.sorting import default_sort_key, ordered
from sympy.core.symbol import Str
from sympy.printing.str import StrPrinter, sstr

from exprenc import dec, enc, vector, write

# ---------------------------------------------------------------------------
# the corpus

x, y, z = symbols("x y z")
A, B = symbols("A B", commutative=False)
xp = Symbol("x", positive=True)
xr = Symbol("x", real=True)
n = Symbol("n", integer=True)

INTEGERS = [Integer(v) for v in (0, 1, -1, 2, -2, 3, 7, 10, -12, 100, 2**64 + 1,
                                 -(10**30), 10**50)]
RATIONALS = [Rational(1, 2), Rational(-1, 2), Rational(3, 7), Rational(-22, 7),
             Rational(2**70, 3**40), Rational(1, 10**30), Rational(7, 3)]
FLOATS = [Float("1.5"), Float("0.1"), Float("-2.25", 30), Float(3), Float(0),
          Float("1e-20"), Float("12345.678", 50), Float("1.5", precision=4),
          Float("0.3", precision=2), Float("3.0", precision=5), Float("1e100"),
          Float("-1e-100"), Float("2.5", 15), Float("2.5", 16),
          Float("123456789.123456789", 25), Float("1e16"), Float("1e15"),
          Float("-7.0", 5), Float("0.000123"), Float("1e400"),
          Float("-1e-400"), Float("0.1", 100), Float("1e-5"), Float("-0.5"),
          Float(2), Float("2.0", 30)]
NUMBER_SINGLETONS = [oo, -oo, nan]
OTHER_SINGLETONS = [zoo, I, E, pi, EulerGamma, Catalan, GoldenRatio,
                    TribonacciConstant, S.true, S.false]
NUMBERS = INTEGERS + RATIONALS + FLOATS + NUMBER_SINGLETONS

SYMBOLS = [x, y, z, xp, xr, Symbol("x", integer=True, positive=True), n,
           Symbol("X"), Symbol("alpha"), Symbol("x1"), Symbol("x10"),
           Symbol("x2"), Symbol("ω"), Symbol("a_b"), A, B,
           Symbol("x", commutative=False), Symbol("x", finite=True),
           Symbol("x", zero=True), Symbol("Zz")]
DUMMIES = [Dummy("d", dummy_index=17), Dummy("d", dummy_index=5),
           Dummy("x", dummy_index=17), Dummy("x", dummy_index=17, positive=True),
           Dummy("e", dummy_index=10**6 + 3)]
WILDS = [Wild("w"), Wild("x"), Wild("v", positive=True)]
STRS = [Str("abc"), Str("x"), Str(""), Str("Zeta")]

ATOMS = (NUMBERS + OTHER_SINGLETONS + SYMBOLS + DUMMIES + WILDS + STRS)

f, g = Function("f"), Function("g")
fr = Function("f", real=True)
hp = Function("h", positive=True)
APPLIED = [f(x), f(y), g(x), f(x, y), f(), fr(x), hp(x), f(2), f(Rational(1, 2)),
           Function("sin")(x), Function("exp")(x), Function("F")(x), f(f(x)),
           f(x, 1), g(y, x)]

FUNCTIONS = [sin(x), cos(x), exp(x), log(x), sin(2 * x), sin(x) ** 2, Abs(x),
             exp(2), sin(1), gamma(x), Mod(x, 2), Min(x, y), Max(x, y),
             atan2(y, x), Piecewise((x, x > 0), (0, True)), sin(x + y),
             f(g(x)), exp(sin(x)), cos(y), log(2)]

ADDS = [x + y, x + 1, x - 1, 1 - x, 1 - 2 * x, 2 - 3 * x, -1 + 2 * x, pi - x,
        pi - 2 * x, x + y + z, 2 * x + 3 * y, x ** 2 + x + 1,
        x ** 2 + 2 * x * y + y ** 2, x + A, sin(x) + cos(x), exp(x) + exp(-x),
        log(x) + 1, x / 2 + y / 3, 0.5 * x + 1.5, x + 2.5, x + oo, pi * x + x,
        x + I * x, sqrt(2) * x + x, pi + 1, E + 1, x + O(x ** 2),
        x ** 3 + x ** 2 * y + x * y ** 2 + y ** 3, y ** 2 + x, x * y + 1,
        x * y * z + x + y + z, 2 * x ** 2 * y + 3 * x * y ** 2 + 1,
        Add(x, Mul(nan, y, evaluate=False), evaluate=False),
        Add(Mul(oo, x, evaluate=False), y, evaluate=False),
        Add(2, Mul(-3, x, evaluate=False), evaluate=False),
        Add(Rational(1, 2), Mul(Rational(-1, 3), y, evaluate=False),
            evaluate=False),
        Add(Float("0.5"), Mul(Float("-1.5"), x, evaluate=False), evaluate=False),
        Add(x, x ** -1, x ** -2, evaluate=False), f(x) + g(x) + x,
        A * B + B * A, x * A + y, 3 + x + y ** 2 + x * y, -x - y,
        Add(Integer(3), Integer(-5), x, evaluate=False), x ** y + x,
        Add(x ** 2, Mul(3, x, x, evaluate=False), evaluate=False)]

MULS = [2 * x, -x, -2 * x, x * y, 2 * x * y, x * y * z, 3 * x ** 2 * y,
        x ** 2 * y ** 3 * z, A * B, B * A, 2 * A * B, x * A * B, x * (y + 1),
        (x + y) * (x - y), sin(x) * cos(x), 2 * x * y * sin(x) * cos(x),
        Rational(-3, 7) * x * y, Float("-2.5") * x, pi * x, 2 * pi,
        -pi, x * sin(x) * f(x), Mul(x, y, 2, evaluate=False),
        Mul(-1, A, B, evaluate=False), oo * x, y * x ** 2,
        Mul(x, x, evaluate=False), Mul(Integer(-5), x, evaluate=False)]

POWS = [x ** 2, x ** 3, x ** -1, x ** -2, sqrt(x), x ** Rational(3, 2),
        x ** Rational(-1, 3), x ** y, 2 ** x, exp(x), Pow(E, x, evaluate=False),
        (x + 1) ** 2, (x + 1) ** (y + 1), x ** (2 * y), x ** (-y),
        x ** (y / 2), x ** Float("2.0"), x ** oo, Pow(x, 1, evaluate=False),
        Pow(2, Rational(1, 2), evaluate=False), (x * y) ** 2,
        Pow(x, -3, evaluate=False), A ** 2]

OTHERS = [Eq(x, y), Ne(x, 1), x < y, x <= y, x > y, x >= 1, Eq(x, 2 * y),
          And(x < 1, y > 2), Or(x < 1, y > 2), Not(x < 1), Tuple(x, y),
          Tuple(1, 2, 3), Tuple(), Tuple(x, Tuple(y, z)), Tuple(x + 1, 2 * x),
          Interval(0, 1), S.EmptySet, Derivative(f(x), x), Integral(x, x),
          Lambda(x, x + 1), Tuple(Str("s"), x)]

FINITESETS = [FiniteSet(1, 2, 3), FiniteSet(x)]

CORPUS = ATOMS + APPLIED + FUNCTIONS + ADDS + MULS + POWS + OTHERS
COMPARABLE = CORPUS  # FiniteSet.compare is hash-based: never compared

ORDERS = [None, "lex", "grlex", "grevlex", "ilex", "igrlex", "igrevlex",
          "rev-lex", "rev-grlex", "rev-igrevlex", "foo"]

RNG = random.Random(20250101)


def sample(pool, k):
    return [RNG.choice(pool) for _ in range(k)]


# ---------------------------------------------------------------------------
# helpers recorded through vector()

def evector(op, args, fn, **params):
    """`vector()` with each expression argument (or list of them) passed as
    its `enc()` text, which the Rust side parses inside the op: building an
    argument can reach unported code (an Add of functions asks the
    functions' `is_commutative`), and that must count as not ported, not as
    an undecodable vector. SymPy runs on `dec(enc(a))`, the same raw tree."""
    def encode(a):
        return [enc(e) for e in a] if isinstance(a, list) else enc(a)

    def decode(s):
        return [dec(e) for e in s] if isinstance(s, list) else dec(s)

    return vector(op, [encode(a) for a in args],
                  lambda *ss: fn(*[decode(s) for s in ss]), **params)


def same(a, b):
    return type(a) is type(b) and a == b


def positions(result, pool):
    """The positions in `pool` of the items of `result` (each matched to
    the first unused structurally equal item), as a string."""
    used = [False] * len(pool)
    out = []
    for r in result:
        i = next(i for i, p in enumerate(pool) if not used[i] and same(p, r))
        used[i] = True
        out.append(str(i))
    return " ".join(out)


def factor_positions(result, pool):
    """Like `positions`, as a list, with items not in `pool` (the numbers
    `args_cnc` splits off) given as themselves."""
    used = [False] * len(pool)
    out = []
    for r in result:
        i = next((i for i, p in enumerate(pool) if not used[i] and same(p, r)), None)
        if i is None:
            out.append(r)
        else:
            used[i] = True
            out.append(i)
    return out



def compare_matrix(items):
    """`a.compare(b)` for every pair, one character per pair: < = > (or E
    when it raises), rows separated by spaces."""
    rows = []
    for a in items:
        row = ""
        for b in items:
            try:
                c = a.compare(b)
            except TypeError:
                row += "E"
            else:
                row += "<=>"[c + 1]
        rows.append(row)
    return " ".join(rows)


def key_matrix(items):
    """For every pair: `a.sort_key() == b.sort_key()` (T/F) then
    `a.sort_key() < b.sort_key()` (T/F, E if it raises)."""
    keys = [a.sort_key() for a in items]
    rows = []
    for ka in keys:
        row = ""
        for kb in keys:
            row += "T" if ka == kb else "F"
            try:
                row += "T" if ka < kb else "F"
            except TypeError:
                row += "E"
        rows.append(row)
    return " ".join(rows)


def py_sort_table(values, k, table, reverse):
    """`sorted(range(n), key=K)` where `K(i) < K(j)` is looked up in
    `table` ('1' true, '0' false, 'e' raises TypeError); the list as the
    sort leaves it, even when it raises."""
    class K:
        __slots__ = ("v",)

        def __init__(self, v):
            self.v = v

        def __lt__(self, other):
            c = table[self.v * k + other.v]
            if c == "e":
                raise TypeError("the table says raise")
            return c == "1"

    lst = list(range(len(values)))
    try:
        lst.sort(key=lambda i: K(values[i]), reverse=reverse)
        status = "ok"
    except TypeError:
        status = "raised"
    return status + ":" + " ".join(map(str, lst))


def py_sort_natural(items, reverse):
    """`sorted(range(n), key=items.__getitem__, reverse=reverse)` for ints
    or strings."""
    return " ".join(map(str, sorted(range(len(items)), key=items.__getitem__,
                                    reverse=reverse)))


def str_at_level(e, level, settings):
    p = StrPrinter(settings)
    p._print_level = level - 1
    return p._print(e)


def sort_args(items):
    items = list(items)
    items.sort(key=cmp_to_key(Basic.compare))
    return items


# ---------------------------------------------------------------------------
# list.sort cases

def table_consistent(k, rank):
    return "".join("1" if rank[a] < rank[b] else "0"
                   for a in range(k) for b in range(k))


def table_random(k, p_true=0.5, p_err=0.0):
    out = []
    for _ in range(k * k):
        r = RNG.random()
        out.append("e" if r < p_err else ("1" if r < p_err + p_true else "0"))
    return "".join(out)


def table_flipped(k, flips):
    rank = [RNG.randrange(k) for _ in range(k)]
    t = list(table_consistent(k, rank))
    for _ in range(flips):
        i = RNG.randrange(k * k)
        t[i] = "1" if t[i] == "0" else "0"
    return "".join(t)


def values_pattern(kind, size, k):
    if kind == "random":
        return [RNG.randrange(k) for _ in range(size)]
    if kind == "sorted":
        return sorted(RNG.randrange(k) for _ in range(size))
    if kind == "reversed":
        return sorted((RNG.randrange(k) for _ in range(size)), reverse=True)
    if kind == "runs":
        out = []
        while len(out) < size:
            m = RNG.randrange(1, 80)
            run = sorted(RNG.randrange(k) for _ in range(m))
            if RNG.random() < 0.5:
                run.reverse()
            out.extend(run)
        return out[:size]
    if kind == "sawtooth":
        period = RNG.randrange(2, 40)
        return [(i % period) * k // period for i in range(size)]
    if kind == "interleaved":
        # two sorted halves whose blocks interleave: galloping territory
        half = size // 2
        a = sorted(RNG.randrange(k) for _ in range(half))
        b = sorted(RNG.randrange(k) for _ in range(size - half))
        return a + b
    raise ValueError(kind)


def sort_vectors():
    vectors = []
    small = [0, 1, 2, 3, 5, 8, 13, 31, 32, 33, 63, 64, 65, 100, 127, 128, 129,
             200, 255, 256, 257]
    big = [500, 1000, 2048]
    kinds = ["random", "sorted", "reversed", "runs", "sawtooth", "interleaved"]
    for size in small + big:
        for kind in kinds:
            k = RNG.choice([1, 2, 3, 5, 8, 16])
            values = values_pattern(kind, size, k)
            rank = list(range(k))
            RNG.shuffle(rank)
            tables = [
                ("consistent", table_consistent(k, list(range(k)))),
                ("shuffled", table_consistent(k, rank)),
                ("random", table_random(k)),
                ("flipped", table_flipped(k, max(1, k // 3))),
            ]
            if size >= 8:
                tables.append(("raises", table_random(k, 0.5, 0.02)))
                tables.append(("always", "1" * (k * k)))
                tables.append(("never", "0" * (k * k)))
            if size in big:
                tables = [tables[i] for i in (1, 2, 4)]
            for ti, (name, table) in enumerate(tables):
                for reverse in ((False,) if size in big or ti % 2 else (False, True)):
                    params = {"values": " ".join(map(str, values)), "k": k,
                              "table": table, "reverse": reverse,
                              "kind": f"{kind}/{name}"}
                    vectors.append(vector(
                        "py_sort_table", [], lambda v=values, k=k, t=table,
                        r=reverse: py_sort_table(v, k, t, r), **params))
    # natural order on ints (big ones too) and on strings
    for size in [0, 1, 2, 7, 40, 64, 65, 300, 1200]:
        for reverse in (False, True):
            ints = [RNG.choice([RNG.randrange(-5, 5), RNG.randrange(-10**30, 10**30),
                                RNG.randrange(2**62, 2**64)]) for _ in range(size)]
            vectors.append(vector("py_sort_ints", [ints],
                                  lambda v, r=reverse: py_sort_natural(v, r),
                                  reverse=reverse))
            alphabet = ["", "a", "ab", "b", "B", "abc", "é", "ω", "x1", "x10",
                        "x2", "Zeta", "ÿ", "Ā", "😀", "à"]
            strs = [RNG.choice(alphabet) for _ in range(size)]
            vectors.append(vector("py_sort_strs", [strs],
                                  lambda v, r=reverse: py_sort_natural(v, r),
                                  reverse=reverse))
    return vectors


# ---------------------------------------------------------------------------

def main():
    vectors = []

    # compare: each expression against an equal copy, and targeted pairs
    # within each class.
    for a in COMPARABLE:
        vectors.append(evector("compare", [a, a], lambda a, b: a.compare(b)))
    groups = [NUMBERS, OTHER_SINGLETONS, SYMBOLS, DUMMIES + WILDS + STRS,
              APPLIED, FUNCTIONS, ADDS, MULS, POWS, OTHERS]
    for grp in groups:
        for a in grp:
            for b in sample(grp, 4):
                vectors.append(evector("compare", [a, b], lambda a, b: a.compare(b)))
    for _ in range(400):
        a, b = sample(COMPARABLE, 2)
        vectors.append(evector("compare", [a, b], lambda a, b: a.compare(b)))
    # compare matrices over whole classes and random mixes
    for grp in groups:
        for i in range(0, len(grp), 12):
            vectors.append(evector("compare_matrix", [grp[i:i + 12]], compare_matrix))
    for _ in range(120):
        vectors.append(evector("compare_matrix", [sample(COMPARABLE, 10)],
                               compare_matrix))

    # sort_key itself, and its == / < over many pairs
    for a in CORPUS + FINITESETS:
        vectors.append(evector("sort_key", [a], lambda a: a.sort_key()))
        vectors.append(evector("class_key", [a], lambda a: a.class_key()))
    for a in ADDS + MULS[:8]:
        for order in ("grlex", "rev-lex", "igrevlex"):
            vectors.append(evector("sort_key", [a],
                                   lambda a, o=order: a.sort_key(order=o),
                                   order=order))
    for grp in groups + [FINITESETS]:
        for i in range(0, len(grp), 12):
            vectors.append(evector("key_matrix", [grp[i:i + 12]], key_matrix))
    for _ in range(150):
        vectors.append(evector("key_matrix", [sample(CORPUS + FINITESETS, 10)],
                               key_matrix))

    # sorted(key=default_sort_key), ordered, sort(key=cmp_to_key(compare)):
    # each result as positions in the input.
    def sorted_default(v):
        return positions(sorted(v, key=default_sort_key), v)

    def ordered_(v):
        return positions(list(ordered(v)), v)

    def sort_args_(v):
        return positions(sort_args(v), v)

    for _ in range(200):
        items = sample(CORPUS + FINITESETS, RNG.randrange(0, 12))
        vectors.append(evector("sorted_default", [items], sorted_default))
        items = sample(CORPUS + FINITESETS, RNG.randrange(0, 12))
        vectors.append(evector("ordered", [items], ordered_))
        items = sample(COMPARABLE, RNG.randrange(0, 12))
        vectors.append(evector("sort_args", [items], sort_args_))
    for grp in groups:
        vectors.append(evector("sorted_default", [grp], sorted_default))
        vectors.append(evector("ordered", [grp], ordered_))
        vectors.append(evector("sort_args", [grp], sort_args_))
    # ties: equal sort keys keep their order
    ties = [x, xp, xr, Symbol("x", commutative=False), x, xp]
    for _ in range(10):
        RNG.shuffle(ties)
        vectors.append(evector("ordered", [list(ties)], ordered_))
        vectors.append(evector("sorted_default", [list(ties)], sorted_default))

    # as_ordered_terms (as positions in Add.make_args) and
    # as_ordered_factors (positions in Mul.make_args, or the new numbers
    # args_cnc splits off)
    for a in ADDS + MULS + POWS + ATOMS[:20] + APPLIED[:4] + FUNCTIONS[:6]:
        for order in ORDERS:
            vectors.append(evector(
                "as_ordered_terms", [a],
                lambda e, o=order: positions(e.as_ordered_terms(order=o),
                                             Add.make_args(e)),
                order=order))
    for a in MULS + ADDS[:10] + POWS[:5] + ATOMS[:10] + OTHERS[:3]:
        for order in (None, "lex", "grlex"):
            vectors.append(evector(
                "as_ordered_factors", [a],
                lambda e, o=order: factor_positions(e.as_ordered_factors(order=o),
                                                    Mul.make_args(e)),
                order=order))

    # decompose_power: (enc(base), exponent)
    def decompose_power_(e):
        base, k = decompose_power(e)
        return enc(base), k

    for a in POWS + MULS[:6] + SYMBOLS[:4] + APPLIED[:3] + FUNCTIONS[:4] + [
            x ** (3 * y), x ** (Rational(-2, 3) * y), x ** (-2 * y),
            x ** (y * z), x ** (2 ** 70), x ** -(2 ** 70)]:
        vectors.append(evector("decompose_power", [a], decompose_power_))

    # Python's ==, <, > between Numbers
    nums = (INTEGERS[:8] + [INTEGERS[10], INTEGERS[11]] + RATIONALS[:5]
            + [FLOATS[i] for i in (0, 1, 2, 3, 4, 7, 12, 13, 19, 20)]
            + NUMBER_SINGLETONS)
    for a in nums:
        for b in nums:
            vectors.append(evector("number_eq", [a, b], lambda a, b: a == b))
            vectors.append(evector("number_lt", [a, b], lambda a, b: bool(a < b)))
            vectors.append(evector("number_gt", [a, b], lambda a, b: bool(a > b)))

    # complex() and float()
    for a in NUMBERS + OTHER_SINGLETONS[:-2] + [sqrt(2), sin(1), 1 + I, 2 * pi]:
        vectors.append(evector("complex", [a], lambda a: (complex(a).real,
                                                          complex(a).imag)))
    for a in NUMBERS:
        vectors.append(evector("float", [a], float))

    # str() of atoms, and Float printing under StrPrinter settings
    for a in ATOMS:
        vectors.append(evector("str", [a], str))
    for a in APPLIED[:3] + FUNCTIONS[:3] + ADDS[:3] + MULS[:3] + POWS[:3] + OTHERS[:4]:
        vectors.append(evector("str", [a], str))
    for a in FLOATS:
        for full_prec in (True, False, "auto"):
            for level in (1, 2):
                for extra in ({}, {"dps": 3}, {"dps": 20}, {"min": -2, "max": 3},
                              {"min": 0, "max": 0}, {"dps": 0}):
                    settings = dict(full_prec=full_prec, **extra)
                    vectors.append(evector(
                        "str_settings", [a],
                        lambda e, s=settings, lv=level: str_at_level(e, lv, s),
                        settings=settings, level=level))
    for a in INTEGERS + RATIONALS + FLOATS[:3] + OTHER_SINGLETONS:
        for level in (1, 2):
            settings = {"sympy_integers": True}
            vectors.append(evector(
                "str_settings", [a],
                lambda e, s=settings, lv=level: str_at_level(e, lv, s),
                settings=settings, level=level))

    vectors.extend(sort_vectors())
    write("compare", vectors, __doc__.split("\n\n")[0])


if __name__ == "__main__":
    main()
