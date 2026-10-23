"""Differential vectors for rsympy-core's relationals and Boolean algebra.

    PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python python oracle/gen_diff_relational.py

Writes oracle/diff/core_relational.json, replayed by
crates/rsympy-core/tests/diff_relational.rs. Covers Python's `<`, `<=`,
`>`, `>=` between numbers of every kind (big ints, Rationals, Floats at
several precisions, oo, -oo, nan, zoo, I, the constants), symbols with
assumptions and small sums, products and powers, with `evaluate` on and
off; the constructors `Eq`, `Ne`, `Lt`, `Le`, `Gt`, `Ge` and `Rel` (with
`evaluate=` and under `evaluate(False)`); `is_lt`, `is_le`, `is_gt`,
`is_ge`, `is_eq`, `is_neq`; the Relational properties (`lhs`, `rhs`,
`reversed`, `reversedsign`, `negated`, `canonical`, `weak`, `strict`,
`binary_symbols`, ...), `_canonical_coeff` and `bool()`; and from
`logic.boolalg` `as_Boolean`, the construction of `And`, `Or`, `Not`, `Xor`,
`Nand`, `Nor`, `Xnor`, `Implies`, `Equivalent`, `ITE`, `Exclusive`, the
operators `&`, `|`, `^`, `>>`, `<<`, `~` (also on Integers), BooleanAtom
arithmetic, `func(*args)`, the `is_*` form tests, `to_nnf`, `to_cnf`,
`to_dnf`, `to_anf`, the `distribute_*` helpers, `conjuncts`, `disjuncts`
and `_find_predicates`.

Results that SymPy returns as sets (binary_symbols, conjuncts, ...) are
recorded sorted by `default_sort_key`. Inputs avoid the cases where SymPy's
own answer depends on set iteration order (ties in `ordered`, and three or
more related relationals in one Xor or Equivalent).
"""

import operator
import random

from sympy import (And, Basic, E, Eq, Equivalent, Float, GoldenRatio, Ge, Gt,
                   I, ITE, Implies, Integer, Interval, Le, Lt, Nand, Ne, Nor,
                   Not, Or, Rational, S, Symbol, Tuple, Xor, false, nan, oo,
                   pi, sqrt, symbols, true, zoo, floor, ceiling)
from sympy.core.parameters import evaluate
from sympy.core.relational import (Rel, _canonical_coeff, is_eq, is_ge,
                                   is_gt, is_le, is_lt, is_neq)
from sympy.core.sorting import default_sort_key
from sympy.logic.boolalg import (BooleanFunction, Exclusive, Xnor,
                                 _find_predicates, as_Boolean, conjuncts,
                                 disjuncts, distribute_and_over_or,
                                 distribute_or_over_and,
                                 distribute_xor_over_and,
                                 eliminate_implications, is_anf, is_cnf,
                                 is_dnf, is_literal, is_nnf, to_anf, to_cnf,
                                 to_dnf, to_nnf)

from exprenc import vector, write

RNG = random.Random(20251023)

# ---------------------------------------------------------------------------
# the corpus

x, y, z = symbols("x y z")
a, b, c, d = symbols("a b c d")
xp = Symbol("p", positive=True)
xn = Symbol("q", negative=True)
xr = Symbol("r", real=True)
xi = Symbol("i", integer=True)
xnn = Symbol("s", nonnegative=True)
xnp = Symbol("t", nonpositive=True)
xz = Symbol("u", zero=True)
xim = Symbol("v", imaginary=True)
xinf = Symbol("w", infinite=True)
xer = Symbol("e", extended_real=True)
xep = Symbol("g", extended_positive=True)
xf = Symbol("h", finite=True)
xo = Symbol("k", odd=True)
xnz = Symbol("m", nonzero=True)

INTEGERS = [Integer(v) for v in (0, 1, -1, 2, -2, 3, -7, 10**20, -(10**20),
                                 2**64 + 1, 10**50)]
RATIONALS = [Rational(1, 2), Rational(-1, 2), Rational(1, 3), Rational(-7, 3),
             Rational(22, 7), Rational(10**30, 3), Rational(-1, 10**25)]
FLOATS = [Float("0.5"), Float("-0.5"), Float("2.0"), Float("2.0", 30), Float(0),
          Float("1e-30"), Float("0.333333333333333333333", 30),
          Float("3.14159", 6), Float("1e50"), Float("-1e50", 20),
          Float("2.5", precision=4), Float("0.1", precision=1),
          Float("7", precision=3), Float("3.1428571428571428", 17)]
SPECIAL = [oo, -oo, nan, zoo, I, pi, E, GoldenRatio, true]
NUMBERS = INTEGERS + RATIONALS + FLOATS + SPECIAL

SYMS = [x, y, xp, xn, xr, xi, xnn, xnp, xz, xim, xinf, xer, xep, xf, xo, xnz]
EXPRS = [x + 1, x - 1, 2 * x, -x, x ** 2, xp + 1, -xp, xp * xn, xr ** 2,
         xr ** 2 + 1, xp ** 2 + xn, 1 - xp, x * y, xi / 2, xp + xnn, xn - 1,
         2 * xi + 1, xp ** 3, xr + I, xinf + 1, Tuple(1, x), Tuple(1, 2),
         Tuple(Float("2.0"), 3), Tuple(2, 3, 4), floor(xr), ceiling(xr),
         floor(x)]
SMALL_NUMS = [S(0), S(1), S(-1), S(2), S.Half, Float("0.5"), Float("2.0", 30),
              oo, -oo, nan, zoo, I, Rational(-7, 3), Float("1e-30")]

RELS = [x < 1, x >= 1, x > 2, x <= 2, Eq(x, 1), Ne(x, 1), y <= 3, y > 3,
        Gt(1, x), Le(1, x), Lt(-x, -1), Eq(y, x), Ne(2 * x, y),
        Lt(x, y), Ge(y, x), Eq(1, x), Eq(x, -y), Eq(-x, y),
        Lt(x + 1, 2 * y), Le(-2 * x + 1, 0), Gt(3 - x, y), Eq(a, true),
        Ne(b, false), Eq(true, c), Ge(x, x, evaluate=False),
        Lt(2, 1, evaluate=False), Eq(x < 1, true), Eq(x, y < 1),
        Lt(x, 1 / x), Le(-x, -y), Gt(-x + y, 0)]
BATOMS = [a, b, c, d, true, false]
BNUMS = [S(0), S(1), S(2), Float(0), Float("1.0"), S.Half, nan, oo, S(-1)]
NONBOOL = [x + 1, x * y, Interval(0, 1)]
COMPOUND = [a & b, a | b, ~a, a ^ b, Implies(a, b), Equivalent(a, b),
            ITE(a, b, c), (a & b) | c, ~(a | b), (a | b) & (c | d),
            a ^ b ^ c, Equivalent(a, b, c), (a & ~b) | (~a & b),
            Implies(a & b, c), Not(a & b, evaluate=False),
            And(a, Or(b, c, evaluate=False), evaluate=False),
            Nand(a, b, evaluate=False), Nor(a, b, evaluate=False),
            Xnor(a, b, evaluate=False), Exclusive(a, b, c, evaluate=False),
            Implies(a, b, evaluate=False), ITE(a, b, c, evaluate=False),
            And(x < 1, a), Or(x < 1, Eq(y, 2)), Not(Not(a, evaluate=False),
                                                     evaluate=False),
            ITE(x < 1, a, b), Xor(a, x > 2), Equivalent(a, y <= 3)]

CMP = {"<": operator.lt, "<=": operator.le, ">": operator.gt, ">=": operator.ge}
REL_CLASSES = {"Eq": Eq, "Ne": Ne, "Lt": Lt, "Le": Le, "Gt": Gt, "Ge": Ge}
BOOL_CLASSES = {"And": And, "Or": Or, "Not": Not, "Xor": Xor, "Nand": Nand,
                "Nor": Nor, "Xnor": Xnor, "Implies": Implies,
                "Equivalent": Equivalent, "ITE": ITE, "Exclusive": Exclusive}
BITOPS = {"&": operator.and_, "|": operator.or_, "^": operator.xor,
          ">>": operator.rshift, "<<": operator.lshift}
ARITH = {"+": operator.add, "-": operator.sub, "*": operator.mul,
         "/": operator.truediv, "**": operator.pow, "%": operator.mod,
         "//": operator.floordiv}

# ---------------------------------------------------------------------------
# helpers


def fix(e):
    """Gives decoded And/Or nodes the `_argset` that `LatticeOp.__new__`
    sets (`make_args` reads it); the raw decoder builds them with
    `Basic.__new__`."""
    if isinstance(e, Basic):
        for arg in e.args:
            fix(arg)
        if type(e) in (And, Or) and not hasattr(e, "_argset"):
            e._argset = frozenset(e.args)
    return e


def fixed(fn):
    return lambda *args: fn(*[fix(v) for v in args])


def under(global_eval, fn):
    """Runs `fn` with `global_parameters.evaluate` set to `global_eval`."""
    def run(*args):
        with evaluate(global_eval):
            return fn(*args)
    return run


def sorted_set(s):
    return sorted(s, key=default_sort_key)


def kwargs(ev):
    return {} if ev is None else {"evaluate": ev}


def family(r):
    """The relationals that pair up with `r` in Xor and Equivalent."""
    c = r.canonical
    return frozenset([c, c.negated.canonical])


def order_safe(args):
    """No three related relationals (whose pairing in Xor and Equivalent
    follows set order), counting those inside nested Xors."""
    flat = []
    for v in args:
        flat.extend(v.args if isinstance(v, Xor) else [v])
    seen = {}
    for v in flat:
        if v.is_Relational:
            f = family(v)
            seen[f] = seen.get(f, 0) + 1
            if seen[f] > 2:
                return False
    return True


V = []


def add(op, args, fn, **params):
    V.append(vector(op, args, fn, **params))


# ---------------------------------------------------------------------------
# comparisons


def comparisons():
    # every pair of numbers, evaluated
    for p in NUMBERS:
        for q in NUMBERS:
            for name, f in CMP.items():
                add("cmp", [p, q], under(True, f), cmp=name, eval=True)
    # numbers under evaluate(False): a sample, and everything with oo/-oo
    pairs = [(p, q) for p in NUMBERS for q in NUMBERS]
    for p, q in pairs:
        if p in (oo, -oo, nan) or q in (oo, -oo, nan) or RNG.random() < 0.15:
            name = RNG.choice(list(CMP))
            add("cmp", [p, q], under(False, CMP[name]), cmp=name, eval=False)
    # symbols and expressions against numbers and each other
    left = SYMS + EXPRS
    for p in left:
        for q in SMALL_NUMS:
            for name, f in CMP.items():
                add("cmp", [p, q], under(True, f), cmp=name, eval=True)
                add("cmp", [q, p], under(True, f), cmp=name, eval=True)
            name = RNG.choice(list(CMP))
            add("cmp", [p, q], under(False, CMP[name]), cmp=name, eval=False)
    for p in left:
        for q in RNG.sample(left, 8):
            name = RNG.choice(list(CMP))
            add("cmp", [p, q], under(True, CMP[name]), cmp=name, eval=True)
    # Booleans and relationals in comparisons
    for p in BATOMS + RELS[:6] + COMPOUND[:4]:
        for q in [S(1), x, true, x < 1]:
            name = RNG.choice(list(CMP))
            add("cmp", [p, q], under(True, CMP[name]), cmp=name, eval=True)
            add("cmp", [q, p], under(True, CMP[name]), cmp=name, eval=True)


# ---------------------------------------------------------------------------
# constructors


def constructors():
    pool = (INTEGERS[:6] + RATIONALS[:3] + FLOATS[:6] + [oo, -oo, nan, zoo, I]
            + SYMS + EXPRS[:14] + [true, false, a, x < 1, Eq(x, 1)])
    for name, cls in REL_CLASSES.items():
        for _ in range(260):
            p, q = RNG.choice(pool), RNG.choice(pool)
            ev = RNG.choice([None, None, True, False])
            g = RNG.random() < 0.8
            if not g and ev and not (p.is_Number and q.is_Number):
                # evaluating under evaluate(False) builds unevaluated
                # arithmetic whose assumptions SymPy answers in hash
                # order (or with a TypeError deep in Mul.flatten)
                g = True
            add("rel_new", [p, q],
                under(g, lambda p, q, cls=cls, ev=ev: cls(p, q, **kwargs(ev))),
                cls=name, evaluate=ev, globl=g)
        # the wrong number of arguments
        for args in ([x], [], [x, y, z]):
            add("rel_new", args, lambda *s, cls=cls: cls(*s), cls=name,
                evaluate=None, globl=True)
    # equal by value, by structure, or not at all
    for p, q in [(S(2), Float("2.0")), (Float("2.0"), Float("2.0", 30)),
                 (S.Half, Float("0.5")), (Rational(1, 3), Float("0.333333333333333", 15)),
                 (nan, nan), (oo, oo), (oo, -oo), (zoo, oo), (zoo, zoo),
                 (x, x), (x + 1, 1 + x), (xi, S.Half), (xi, Float("0.5")),
                 (xi, Float("2.0")), (xr, I), (xp, -1), (xp, 0), (xinf, 1),
                 (xinf, oo), (x * y, y * x), (Tuple(1, 2), Tuple(1, 2)),
                 (Tuple(1, x), Tuple(1, 2)), (Tuple(1, 2), Tuple(1, 2, 3)),
                 (Tuple(1, 2), S(1)), (Tuple(1, x), x), (true, false),
                 (true, true), (a, true), (x < 1, true), (x < 1, x < 1),
                 (x < 1, y < 2), (x < 1, 1), (Interval(0, 1), Interval(0, 1)),
                 (Interval(0, 1), Interval(0, 2)), (Interval(0, 1), S(1)),
                 (xim, I), (xim, 0), (2 * xi + 1, 0), (xo, 0), (xnz, 0),
                 (x + I, x), (I * oo, oo), (zoo, I * oo), (xer, oo)]:
        for name in ("Eq", "Ne"):
            add("rel_new", [p, q], REL_CLASSES[name], cls=name,
                evaluate=None, globl=True)
    for rop in [None, "==", "eq", "!=", "<>", "ne", ">=", "ge", "<=", "le",
                ">", "gt", "<", "lt", "=<", "foo"]:
        for p, q in [(x, 1), (S(1), S(2)), (x < 1, y), (a, b), (xp, 0),
                     (Eq(x, 1), true)]:
            for ev in (None, False):
                add("Rel", [p, q],
                    lambda p, q, rop=rop, ev=ev: Rel(p, q, rop, **kwargs(ev)),
                    rop=rop, evaluate=ev)


# ---------------------------------------------------------------------------
# fuzzy comparisons and properties


def fuzzy():
    fns = {"is_lt": is_lt, "is_le": is_le, "is_gt": is_gt, "is_ge": is_ge,
           "is_eq": is_eq, "is_neq": is_neq}
    pool = (INTEGERS[:7] + RATIONALS[:4] + FLOATS[:8] + [oo, -oo, nan, zoo, I]
            + SYMS + EXPRS[:20] + [true, a, x < 1])
    for name, f in fns.items():
        for _ in range(300):
            p, q = RNG.choice(pool), RNG.choice(pool)
            add("fuzzy", [p, q], f, func=name)


PROPS = ["lhs", "rhs", "rel_op", "gts", "lts", "reversed", "reversedsign",
         "negated", "canonical", "weak", "strict", "binary_symbols",
         "canonical_coeff", "bool", "func", "is_Relational"]


def prop(name):
    def get(e):
        if name == "binary_symbols":
            return sorted_set(e.binary_symbols)
        if name == "canonical_coeff":
            return _canonical_coeff(e)
        if name == "bool":
            return bool(e)
        if name == "func":
            return e.func(*e.args)
        return getattr(e, name)
    return get


def properties():
    items = (RELS + [Lt(x, oo, evaluate=False), Eq(2, x, evaluate=False),
                     Ge(-x - 1, 2 * y), Lt(-x, y), Gt(x, -y), Le(1, 2 * x),
                     Lt(x, -1), Ne(-x, -y), Eq(Eq(2, x, evaluate=False), true),
                     Lt(Ge(1, x, evaluate=False), 3, evaluate=False),
                     Eq(x < 1, y < 2), Lt(S(2), S(1), evaluate=False),
                     Lt(S(1), S(2), evaluate=False), Eq(2, 1, evaluate=False),
                     Ne(true, x), Eq(-x + 1, 0), Lt(Rational(-1, 2) * x + 3, 1),
                     Gt(-3 * x, 2), Le(x / 2 + Rational(1, 3), y),
                     Eq(x * y, -1), Lt(-x * y, 0)]
             + BATOMS + COMPOUND[:3] + [S(1), x + 1])
    for e in items:
        for name in PROPS:
            if name == "func" and e.is_Atom:
                continue  # Symbol() and friends: not this module's
            add("prop", [e], prop(name), name=name)
    for e in RELS[:10]:
        add("equals", [e, e.reversed], lambda p, q: p.equals(q))
        add("simplify", [e], lambda p: p.simplify())
    # Relational.equals without Expr.equals: the same or the reversed
    # relation, another class, or not a Relational at all
    for p, q in [(x < 1, x < 1), (x < 1, 1 > x), (x < 1, x <= 1),
                 (x < 1, 1 >= x), (x < 1, Eq(x, 1)), (Eq(x, 1), Ne(x, 1)),
                 (Eq(x, 1), Eq(1, x)), (Ne(x, y), Ne(y, x)), (x < 1, x),
                 (Eq(x, 1), S(1)), (x > 2, Ge(x, 2)), (x >= 2, 2 <= x),
                 (Lt(2, 1, evaluate=False), Gt(1, 2, evaluate=False)),
                 (Eq(a, true), Ne(a, true)), (y <= 3, Lt(y, 3)),
                 (x < 1, y > 3), (Eq(x, 1), x < 1), (Ne(x, 1), x >= 1)]:
        add("equals", [p, q], lambda p, q: p.equals(q))


# ---------------------------------------------------------------------------
# Boolean construction


def random_args(pool, lo, hi):
    while True:
        args = [RNG.choice(pool) for _ in range(RNG.randint(lo, hi))]
        if order_safe(args):
            return args


def booleans():
    pool = BATOMS * 3 + RELS[:24] + BNUMS + NONBOOL + COMPOUND
    simple = BATOMS * 4 + RELS[:12] + BNUMS[:5] + COMPOUND[:8]
    for name, cls in BOOL_CLASSES.items():
        lo, hi = {"Implies": (1, 3), "ITE": (2, 4), "Not": (0, 2)}.get(name, (0, 4))
        n = {"And": 260, "Or": 260, "Xor": 260, "Equivalent": 200,
             "ITE": 200, "Implies": 160, "Not": 120}.get(name, 100)
        for i in range(n):
            args = random_args(pool if i % 2 else simple, lo, hi)
            if name in ("Implies", "ITE") and i % 3:
                # mostly the right arity
                args = random_args(pool if i % 2 else simple,
                                   *(2, 2) if name == "Implies" else (3, 3))
            if name == "Not" and i % 4:
                args = random_args(pool, 1, 1)
            ev = RNG.choice([None, None, None, True, False])
            g = RNG.random() < 0.85
            params = dict(cls=name, evaluate=ev, globl=g)
            kw = kwargs(ev)
            if name == "Xor" and RNG.random() < 0.3:
                kw["remove_true"] = False
                params["remove_true"] = False
            add("bool_new", args,
                under(g, lambda *s, cls=cls, kw=kw: cls(*s, **kw)), **params)
    # ITE with Eq/Ne conditions on binary symbols
    conds = [Eq(a, true), Eq(true, a), Eq(a, false), Eq(false, a), Ne(a, true),
             Ne(false, a), Eq(a, b), Ne(a, b), Eq(x, 1), Eq(a, c), Ne(c, true)]
    for cond in conds:
        for bc in [(a, b), (b, c), (true, false), (false, true), (b, b),
                   (a, true), (c, a), (S(1), S(0)), (x < 1, b)]:
            add("bool_new", [cond, *bc], ITE, cls="ITE", evaluate=None,
                globl=True)
    # as_Boolean and binary_check_and_simplify
    for e in (BATOMS + BNUMS + NONBOOL + RELS[:5] + COMPOUND[:5]
              + [xz, xnz, xp, xi, Symbol("j", zero=False), Float("0.0", 30),
                 S(-2), I, zoo]):
        add("as_Boolean", [e], as_Boolean)
    for _ in range(40):
        args = random_args(BATOMS + BNUMS + RELS[:4] + NONBOOL, 0, 3)
        add("binary_check", args,
            lambda *s: BooleanFunction.binary_check_and_simplify(*s))


def operators():
    pool = (BATOMS + RELS[:8] + COMPOUND[:10] + [S(0), S(1), S(2), S(-3),
            S(5), Integer(2**70), S.Half, Float("1.0"), x + 1, oo, xi, I,
            Tuple(1, 2), Interval(0, 1)])
    for p in pool:
        for q in RNG.sample(pool, 12):
            for name, f in BITOPS.items():
                add("bitop", [p, q], f, sym=name)
    for p, q in [(S(6), S(3)), (S(-6), S(3)), (S(6), S(-3)), (S(1), S(70)),
                 (S(-5), S(2)), (S(5), S(0)), (Integer(2**70), S(3)),
                 (Integer(-2**70), S(65)), (S(0), S(0)), (S(-1), S(-1))]:
        for name, f in BITOPS.items():
            add("bitop", [p, q], f, sym=name)
            add("bitop", [q, p], f, sym=name)
    for p in pool + [S(-6), Integer(-2**70)]:
        add("invert", [p], operator.invert)
    # BooleanAtom arithmetic
    for p in [true, false]:
        for q in [S(1), x, true, Float("1.0"), oo, S.Half, x < 1]:
            for name, f in ARITH.items():
                add("arith", [p, q], f, sym=name)
                add("arith", [q, p], f, sym=name)
        add("neg", [p], operator.neg)
        add("abs", [p], abs)


# ---------------------------------------------------------------------------
# normal forms


def forms():
    pool = (BATOMS + RELS[:10] + COMPOUND + NONBOOL[:2]
            + [S(1), S(0), Not(x + 1), Not(x < 1, evaluate=False),
               Xor(a, b, c, d), Equivalent(a, b, c, d), (a & b) | (c & d),
               (a | b) & (c | ~d), ITE(a & b, c, ~d), Implies(a | b, c & d),
               (a ^ b) & c, ~(a ^ b ^ c), ~Equivalent(a, b),
               ~Implies(a, b), ~ITE(a, b, c), Or(a, And(b, ~a, evaluate=False),
                                                 evaluate=False),
               And(a, ~a, evaluate=False), Or(a, ~a, b, evaluate=False),
               Nand(a, b, c, evaluate=False), Xor(a, true, remove_true=False),
               Xor(a & b, c, remove_true=False), Basic.__new__(And, a, b, x + 1),
               Or(a & b, a & ~b, evaluate=False)])
    for e in pool:
        add("is_literal", [e], is_literal)
        for simp in (True, False):
            add("is_nnf", [e], fixed(lambda v, simp=simp: is_nnf(v, simp)),
                simplified=simp)
            add("to_nnf", [e], fixed(lambda v, simp=simp: to_nnf(v, simp)),
                simplify=simp)
        add("is_cnf", [e], fixed(is_cnf))
        add("is_dnf", [e], fixed(is_dnf))
        add("is_anf", [e], fixed(is_anf))
        add("to_cnf", [e], fixed(to_cnf), simplify=False)
        add("to_dnf", [e], fixed(to_dnf), simplify=False)
        for deep in (True, False):
            add("to_anf", [e], fixed(lambda v, deep=deep: to_anf(v, deep)),
                deep=deep)
        add("eliminate_implications", [e], fixed(eliminate_implications))
        add("distribute_and_over_or", [e], fixed(distribute_and_over_or))
        add("distribute_or_over_and", [e], fixed(distribute_or_over_and))
        add("distribute_xor_over_and", [e], fixed(distribute_xor_over_and))
        add("conjuncts", [e], fixed(lambda v: sorted_set(conjuncts(v))))
        add("disjuncts", [e], fixed(lambda v: sorted_set(disjuncts(v))))
        add("find_predicates", [e],
            fixed(lambda v: sorted_set(_find_predicates(v))))
    # simplify=True needs simplify_logic; more than 8 predicates raises first
    many = And(*symbols("a0:9"))
    for e in [a & b, (a | b) & c, many | x, many]:
        for f, name in ((to_cnf, "to_cnf"), (to_dnf, "to_dnf")):
            add(name, [e], fixed(lambda v, f=f: f(v, True)), simplify=True)
            add(name, [e], fixed(lambda v, f=f: f(v, True, True)),
                simplify=True, force=True)


# ---------------------------------------------------------------------------
# SOPform, POSform, ANFform and friends


QM = random.Random(20251024)
QMVARS = symbols("w x y z v u")


def spec(term, variables):
    """A term as SOPform takes it: an int, a list of bits or a dict."""
    r = QM.random()
    if r < 0.45:
        return term
    bits = [int(b) for b in bin(term)[2:].rjust(len(variables), "0")]
    if r < 0.9:
        return bits
    # a dict fixing a random subset of the variables (at least one)
    keep = sorted(QM.sample(range(len(variables)),
                            QM.randint(1, len(variables))))
    return {variables[i]: bits[i] for i in keep}


def quine_mccluskey():
    from sympy.logic.boolalg import (ANFform, POSform, SOPform, anf_coeffs,
                                     bool_maxterm, bool_minterm,
                                     bool_monomial, gateinputcount,
                                     simplify_logic, term_to_integer,
                                     to_int_repr)
    forms = {"SOPform": SOPform, "POSform": POSform}
    for i in range(420):
        n = QM.choice([1, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 6])
        variables = list(QMVARS[:n])
        everything = list(range(2 ** n))
        QM.shuffle(everything)
        k = QM.randint(0, 2 ** n)
        minterms = everything[:k]
        rest = everything[k:]
        dontcares = rest[:QM.randint(0, min(len(rest), 2 ** n // 3))]
        name = "SOPform" if i % 2 else "POSform"
        mt = [spec(t, variables) for t in minterms]
        dc = [spec(t, variables) for t in dontcares]
        if QM.random() < 0.3:
            dc = None
        add("qm_form", [variables, mt, dc], forms[name], name=name)
    # inputs SOPform rejects or reads oddly
    v3 = list(QMVARS[:3])
    for mt, dc in [([[0, 1]], None), ([8], None), ([-1], None), ([S(3)], None),
                   (["101"], None), ([1, 3], [3]), ([1, 3], [[0, 1, 1]]),
                   ([{QMVARS[5]: 1}], None), ([], [1]), ([], None),
                   ([[1, 0, 2]], None), ([[1, 3, 0]], None),
                   ([{QMVARS[0]: 1, QMVARS[1]: 0, QMVARS[2]: 1}], [5]),
                   ([(1, 0, 1), 2], [{QMVARS[0]: 0}])]:
        for name in ("SOPform", "POSform"):
            add("qm_form", [v3, mt, dc], forms[name], name=name)
    add("qm_form", [[x, x], [{x: 1}], None], SOPform, name="SOPform")
    add("qm_form", [[], [0], None], SOPform, name="SOPform")
    add("qm_form", [[], [0], None], POSform, name="POSform")
    # ANFform and anf_coeffs
    for i in range(80):
        n = QM.choice([0, 1, 2, 3, 4])
        values = [QM.randint(0, 1) for _ in range(2 ** n)]
        if i % 10 == 9:
            values = values[:-1] or [0, 1, 1]
        add("ANFform", [list(QMVARS[:n]), values], ANFform)
        add("anf_coeffs", [values], anf_coeffs)
    add("ANFform", [[x], [0, 2]], ANFform)
    add("anf_coeffs", [[]], anf_coeffs)
    add("anf_coeffs", [[1, 3, 0, 2]], anf_coeffs)
    # the k-th minterm, maxterm and monomial
    fns = {"bool_minterm": bool_minterm, "bool_maxterm": bool_maxterm,
           "bool_monomial": bool_monomial}
    for name, f in fns.items():
        for k in [0, 1, 5, 7, 8, -1, [1, 0, 1], [0, 0], [1, 1, 1, 1],
                  [1, 3, 0], [2, 1, 0]]:
            add("bool_term", [k, v3], f, name=name)
        add("bool_term", [0, []], f, name=name)
    for term in [[1, 0, 0], [0], [1], [], [0, 0, 1, 1], [2], [1, 2], [-1],
                 [-1, 1], [1, -1], [10]]:
        add("term_to_integer", [term], term_to_integer)
    clauses = [a | b, b, ~a | c, Or(a, ~b, c), ~c]
    for cl in [clauses, clauses[:2], [a & b], [d], [a | d]]:
        add("to_int_repr", [cl, [a, b, c]],
            lambda cl, sy: [sorted(st) for st in to_int_repr(cl, sy)])
    add("to_int_repr", [[a | b], [a, b, a]],
        lambda cl, sy: [sorted(st) for st in to_int_repr(cl, sy)])
    for e in (BATOMS + RELS[:3] + COMPOUND + [x + 1, S(1),
              Nand(a, b, c, evaluate=False), Xnor(a, b, evaluate=False),
              Basic.__new__(And, a, x + 1)]):
        add("gateinputcount", [e], gateinputcount)
    # simplify_logic as far as it goes without xreplace
    for e in [a & b, a | b, ~a, a, true, x < 1, (a | b) & c, (a & b) | c,
              a & (x < 1), x + 1]:
        for form in [None, "cnf", "dnf", "foo"]:
            for deep in (True, False):
                add("simplify_logic", [e], fixed(
                    lambda v, form=form, deep=deep:
                    simplify_logic(v, form, deep)), form=form, deep=deep)


def main():
    comparisons()
    constructors()
    fuzzy()
    properties()
    booleans()
    operators()
    forms()
    quine_mccluskey()
    write("relational", V, __doc__.split("\n\n")[0])


if __name__ == "__main__":
    main()
