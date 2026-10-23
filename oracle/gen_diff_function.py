"""Differential vectors for rsympy-core's function framework
(`sympy.core.function`: `Function.__new__`, undefined functions,
`WildFunction`, `Lambda`) and `sympy.core.mod.Mod`.

    PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python python oracle/gen_diff_function.py

Writes oracle/diff/core_function.json, replayed by
crates/rsympy-core/tests/diff_function.rs. Covers:

- undefined functions: `Function(name or Symbol, **assumptions,
  nargs=...)`, its errors, `nargs`, `class_key()`, equality and
  `isinstance`, and applying it (`f(*args)` with and without `evaluate`,
  the nargs `TypeError`, the auto-evalf of Float arguments);
- the defined classes of the class table: `class_key()`, `nargs`, and
  `cls(*args, evaluate=False)` (the nargs check of `Function._new_`);
- the inherited hooks of applied functions: `_eval_conjugate`,
  `_eval_transpose`, `_eval_adjoint`, `as_base_exp`, `as_numer_denom`,
  `fdiff`'s ArgumentIndexError, `inverse`, `_eval_subs`, `func(*args)`,
  `evalf` and `Mod._eval_rewrite_as_floor`;
- every assumption fact of applied undefined functions, raw `Mod` nodes and
  `Lambda`s;
- `Lambda`: construction (signature checks, `IdentityFunction`), calls,
  `nargs`, `variables`, `signature`, `expr`, `free_symbols`, `is_identity`;
- `WildFunction(name, nargs=...)`: `nargs` and `matches`;
- `Mod(p, q)` (and `p % q`) over integers (big and negative), rationals,
  floats, the numeric singletons, symbols with assumptions, and sums and
  products of them, evaluated, with `evaluate=False` and under
  `evaluate(False)`.

An undefined function travels as a JSON spec in the vector's params:
`{"name": "f"}` or `{"symbol": enc(Symbol)}`, `"assumptions": [[fact,
True/False/None], ...]` (keyword order) and `"nargs"` (absent when not
given; `null`, an int, or a list for a tuple). `make_function(spec)` is
what both sides build from it.

The facts of applied functions are recorded the way
oracle/gen_diff_arith_facts.py records them: with `_ask`'s shuffle of
prerequisite facts replaced by sorted order, leaving out any vector whose
answer changes under other orders.
"""

import itertools
import random
import sys
import warnings

from sympy import (Add, Dummy, E, EulerGamma, Float, Function, GoldenRatio, I,
                   Integer, Lambda, Mod, Mul, Pow, Rational, S, Symbol, Tuple,
                   Wild, WildFunction, floor, oo, pi, sin, sqrt, symbols, zoo)
from sympy.core.assumptions import _assume_defined
from sympy.core.function import Application
from sympy.core.parameters import evaluate

import gen_diff_arith_facts as facts_gen
from exprenc import dec, enc, value, vector, write
from gen_core_tables import CLASSES

x, y, z = symbols("x y z")
p = Symbol("p", positive=True)
q = Symbol("q", negative=True)
r = Symbol("r", real=True)
n = Symbol("n", integer=True)
m = Symbol("m", even=True)
o = Symbol("o", odd=True)
k = Symbol("k", integer=True, positive=True)
nn = Symbol("nn", integer=True, nonnegative=True)
A, B = symbols("A B", commutative=False)
w = Wild("w")
f = Function("f")
g = Function("g", real=True)
half = Rational(1, 2)


# ---------------------------------------------------------------------------
# undefined functions

def make_function(spec):
    """`Function(...)` from a spec (see the module docs)."""
    kw = {}
    for key, val in spec.get("assumptions", []):
        kw[key] = val
    if "nargs" in spec:
        nargs = spec["nargs"]
        kw["nargs"] = tuple(nargs) if isinstance(nargs, list) else nargs
    if "symbol" in spec:
        return Function(dec(spec["symbol"]), **kw)
    return Function(spec["name"], **kw)


def spec(name, *assumptions, **extra):
    out = {"name": name}
    if assumptions:
        out["assumptions"] = [list(a) for a in assumptions]
    out.update(extra)
    return out


def sym_spec(sym, *assumptions):
    out = {"symbol": enc(sym)}
    if assumptions:
        out["assumptions"] = [list(a) for a in assumptions]
    return out


SPECS = [
    spec("f"),
    spec("g", ("real", True)),
    spec("h", ("positive", True)),
    spec("F", ("integer", True), ("negative", False)),
    spec("nc", ("commutative", False)),
    spec("c", ("commutative", True)),
    spec("ev", ("even", True)),
    spec("pr", ("prime", True)),
    spec("im", ("imaginary", True)),
    spec("inf", ("infinite", True)),
    spec("zr", ("zero", True)),
    spec("nz", ("nonzero", True)),
    spec("ex", ("extended_real", True)),
    spec("fin", ("finite", True)),
    spec("ir", ("irrational", True)),
    spec("tr", ("transcendental", True)),
    spec("pol", ("polar", True)),
    spec("he", ("hermitian", True), ("commutative", False)),
    spec("ah", ("antihermitian", True), ("commutative", False)),
    spec("nr", ("real", False)),
    spec("dr", ("real", None)),
    spec("f1", nargs=1),
    spec("f12", nargs=[2, 1]),
    spec("f0", nargs=0),
    spec("fN", nargs=None),
    spec("f02", nargs=[0, 2]),
    spec("fneg", nargs=-1),
    spec("f3", nargs=[3, 1, 1]),
    spec("g1", ("real", True), nargs=1),
    spec("exp"),
    spec("sin", nargs=2),
    spec("re", ("real", True)),
    sym_spec(Symbol("s")),
    sym_spec(Symbol("s", real=True)),
    sym_spec(Symbol("s", commutative=False)),
    sym_spec(Symbol("s", real=True), ("real", True)),
    sym_spec(Wild("W")),
]
# `Symbol._merge` adds assumptions without deducing their consequences, so
# this function's `_kwargs` is not what `Function('s', **kwargs)` makes of
# them: exprenc cannot encode it, so only its non-expression outcomes are
# recorded.
UNENCODABLE_SPECS = [sym_spec(Symbol("s", positive=True), ("integer", True))]
BAD_SPECS = [
    spec("f", ("commutative", None)),
    spec("f", ("real", True), ("imaginary", True)),
    spec("f", nargs=[]),
    spec("f", ("real", True), ("imaginary", True), nargs=[]),
    sym_spec(Symbol("s", real=True), ("imaginary", True)),
    sym_spec(Symbol("s", even=True), ("odd", True)),
    sym_spec(Symbol("s", positive=True), ("even", True), ("odd", True)),
    sym_spec(Integer(1)),
    sym_spec(x + 1),
]
CALL_ARGS = [
    (x,), (x, y), (), (A,), (A, x), (Integer(1),), (Float("1.5"),),
    (Float("1.5"), Float("2.5")), (Float("1.5"), x), (1.5 + 2.0 * I,),
    (1 + 2.0 * I,), (2.0 * I,), (I,), (pi,), (f(x),), (oo,), (S.NaN,),
    (x, y, z), (Float("1.5", 30),), (Float("0.1", 30), Float("2.5")),
    (S.true,),
]


_SKIPPED = []


def maybe_vector(out, *args, **params):
    """`out.append(vector(*args, **params))`, unless exprenc cannot encode
    the outcome: srepr leaves out a default argument (`Heaviside(x, 1/2)`),
    the object is broken (`Subs(x, y, z, evaluate=False)` has no `_expr`),
    or it holds a Dummy (whose index is random)."""
    try:
        vec = vector(*args, **params)
    except Exception:  # noqa: BLE001
        _SKIPPED.append(args[0])
        return
    if "Dummy(" in str(vec["out"]):
        _SKIPPED.append(args[0])
        return
    out.append(vec)


def call_with(spec_, evaluate_kw):
    def run(*args):
        fn = make_function(spec_)
        if evaluate_kw is None:
            return fn(*args)
        return fn(*args, evaluate=evaluate_kw)
    return run


def under_no_evaluate(fn):
    def run(*args):
        with evaluate(False):
            return fn(*args)
    return run


def undefined_vectors():
    out = []
    for sp_, args in itertools.product(SPECS + BAD_SPECS, CALL_ARGS):
        out.append(vector("undef_call", list(args), call_with(sp_, None), spec=sp_))
    for sp_, args in itertools.product(SPECS, CALL_ARGS[:10]):
        out.append(vector("undef_call", list(args), call_with(sp_, False),
                          spec=sp_, evaluate=False))
        out.append(vector("undef_call", list(args), under_no_evaluate(call_with(sp_, None)),
                          spec=sp_, global_evaluate=False))
    for sp_ in SPECS + BAD_SPECS + UNENCODABLE_SPECS:
        out.append(vector("undef_nargs", [], lambda s=sp_: str(make_function(s).nargs),
                          spec=sp_))
        out.append(vector("undef_class_key", [], lambda s=sp_: make_function(s).class_key(),
                          spec=sp_))
        out.append(vector("undef_name", [], lambda s=sp_: make_function(s).__name__,
                          spec=sp_))
        for args in [(x,), (x, y), (), (x, y, z)]:
            out.append(vector("undef_instance_nargs", list(args),
                              lambda *a, s=sp_: str(make_function(s)(*a).nargs), spec=sp_))
    pairs = SPECS[:8] + SPECS[20:26] + SPECS[32:] + UNENCODABLE_SPECS
    for s1, s2 in itertools.product(pairs, pairs):
        out.append(vector("undef_eq", [], lambda a=s1, b=s2: make_function(a) == make_function(b),
                          spec=s1, other=s2))
    for s1, s2 in itertools.product(pairs, pairs):
        out.append(vector("undef_isinstance", [x],
                          lambda e, a=s1, b=s2: isinstance(make_function(a)(e), make_function(b)),
                          spec=s1, other=s2))
        out.append(vector("undef_func_eq", [x],
                          lambda e, a=s1, b=s2: make_function(a)(e).func == make_function(b),
                          spec=s1, other=s2))
    return out


# ---------------------------------------------------------------------------
# defined classes

def defined_classes():
    """The `Application` subclasses of the class table, by `__name__`."""
    return [cls for name, cls in CLASSES
            if isinstance(cls, type) and issubclass(cls, Application)
            and name != "AppliedUndef"]


def defined_vectors():
    out = []
    for cls in defined_classes():
        name = cls.__name__
        out.append(vector("class_key", [], lambda c=cls: c.class_key(), cls=name))
        out.append(vector("class_nargs", [], lambda c=cls: str(c.nargs), cls=name))
        if cls is WildFunction:
            # (wild_vectors covers its constructor)
            continue
        for args in [(), (x,), (x, y), (x, y, z), (Float("1.5"),)]:
            maybe_vector(out, "defined_call", list(args),
                         lambda *a, c=cls: c(*a, evaluate=False), cls=name, evaluate=False)
    return out


# ---------------------------------------------------------------------------
# the hooks applied functions inherit

def hook_vectors():
    out = []
    applied = [f(x), g(x), Function("h", positive=True)(x),
               Function("im", imaginary=True)(x), Function("nc", commutative=False)(x),
               Function("he", hermitian=True, commutative=False)(x),
               Function("ah", antihermitian=True, commutative=False)(x),
               Function("nr", real=False)(x), f(A), f(x, y), f(), f(Float("1.5")),
               Function("f1", nargs=1)(x), Mod(x, y, evaluate=False),
               # (not Mod(A, 2): Function._eval_is_commutative says it is
               # not commutative, Mod._eval_is_nonnegative that it is
               # nonnegative, hence commutative, so its facts depend on the
               # order _ask tries them in)
               Mod(2, 3, evaluate=False), Mod(n, 2, evaluate=False), Lambda(x, x**2), Lambda((x, y), x + y),
               S.IdentityFunction]
    for e in applied:
        out.append(vector("eval_conjugate", [e], lambda e: e._eval_conjugate()))
        out.append(vector("eval_transpose", [e], lambda e: e._eval_transpose()))
        out.append(vector("eval_adjoint", [e], lambda e: e._eval_adjoint()))
        out.append(vector("as_base_exp", [e], lambda e: e.as_base_exp()))
        out.append(vector("as_numer_denom", [e], lambda e: e.as_numer_denom()))
        out.append(vector("eval_subs", [e, x, y], lambda e, a, b: e._eval_subs(a, b)))
        out.append(vector("eval_subs", [e, f(x), g(x)], lambda e, a, b: e._eval_subs(a, b)))
        if e.is_Function and not isinstance(e, Lambda):
            for i in (0, 1, 2, 3):
                maybe_vector(out, "fdiff", [e, i], lambda e, i: e.fdiff(i))
            out.append(vector("inverse", [e], lambda e: e.inverse()))
            for args in [(y,), (y, z), (), (Integer(5), Integer(3))]:
                out.append(vector("func", [e] + list(args), lambda e, *a: e.func(*a)))
        for prec in (15, 30):
            out.append(vector("evalf", [e], lambda e, d=prec: e.evalf(d), n=prec))
    for a, b in [(x, y), (x, 3), (Integer(7), Integer(3)), (x, half), (Float("1.5"), x)]:
        out.append(vector("rewrite_floor", [Mod(a, b, evaluate=False)],
                          under_no_evaluate(lambda e: e._eval_rewrite_as_floor(*e.args)),
                          global_evaluate=False))
        out.append(vector("rewrite_floor", [Mod(a, b, evaluate=False)],
                          lambda e: e._eval_rewrite_as_floor(*e.args)))
    return out


# ---------------------------------------------------------------------------
# facts

def fact_exprs():
    exprs = []
    fact_specs = [s for s in SPECS if "nargs" not in s or s["nargs"] is None]
    for sp_, args in itertools.product(fact_specs, [(x,), (A,), (Integer(1),), (x, A), ()]):
        exprs.append(make_function(sp_)(*args))
    mod_p = [x, p, q, n, m, o, k, r, nn, Integer(2), Integer(-3), half, Float("1.5"),
             I, A, S.Zero]
    mod_q = [Integer(2), Integer(-3), half, Float("-2.5"), x, p, q, n, k, S.Zero, A]
    for a, b in itertools.product(mod_p, mod_q):
        exprs.append(Mod(a, b, evaluate=False))
    exprs.extend([Lambda(x, x**2), Lambda((x, y), x + y), Lambda((), 1), S.IdentityFunction,
                  sin(x, evaluate=False)])
    seen = set()
    return [e for e in exprs if not (e in seen or seen.add(e))]


def fact_vectors():
    # (`sympy.core.assumptions` is a function: the module is shadowed)
    module = sys.modules["sympy.core.assumptions"]
    shuffle = module.shuffle
    module.shuffle = facts_gen._visit_order
    out, dropped = [], 0
    try:
        for e, fact in itertools.product(fact_exprs(), sorted(_assume_defined)):
            facts_gen._chose = False
            vec = vector("is", [e, fact], facts_gen.is_)
            if facts_gen._chose and any(
                    facts_gen.answer(enc(e), fact, random.Random(i)) != vec["out"]
                    for i in range(facts_gen.ORDERS)):
                dropped += 1
                continue
            out.append(vec)
    finally:
        module.shuffle = shuffle
    print(f"dropped {dropped} fact vectors whose answer depends on the order "
          f"_ask visits facts in")
    return out


# ---------------------------------------------------------------------------
# Lambda

SIGNATURES = [x, Tuple(x), Tuple(x, y), Tuple(Tuple(x, y), z), Tuple(),
              Tuple(x, Tuple(y, Tuple(z))), w, Symbol("s", real=True), p,
              Integer(1), Tuple(x, x), Tuple(x, Tuple(x)), Tuple(x, Integer(1)),
              x + 1, f(x), Tuple(f(x)), Tuple(Tuple())]
BODIES = [x, x**2, x + y, Tuple(x, y), y, Integer(1), f(x), Tuple(x), x * y * z,
          Lambda(y, x + y), Mod(x, 3, evaluate=False), Tuple(Tuple(x, y), z),
          Float("1.5") * x]
LAMBDA_ARGS = [(), (Integer(1),), (Integer(1), Integer(2)),
               (Tuple(Integer(1), Integer(2)), Integer(3)),
               (Tuple(Integer(1), Integer(2), Integer(3)), Integer(3)),
               (Tuple(Integer(1), Integer(2)),), (x,), (y, x),
               (Tuple(y, x), z), (Integer(1), Integer(2), Integer(3)),
               (Tuple(Integer(1), Tuple(Integer(2), Integer(3))),),
               (Integer(1), Tuple(Integer(2), Tuple(Integer(3), Integer(4)))),
               (Integer(1), Tuple(Integer(2), Integer(3))), (Integer(5),),
               (Float("1.5"),), (Integer(7), Integer(3))]


def lambda_vectors():
    out = []
    lambdas = []
    for sig, body in itertools.product(SIGNATURES, BODIES):
        vec = vector("lambda", [sig, body], Lambda)
        out.append(vec)
        if "expr" in vec["out"]:
            lambdas.append(dec(vec["out"]["expr"]))
    lambdas.extend([Lambda(x, Mod(x, 3)), Lambda((x, y), Mod(x, y)),
                    Lambda(y, Lambda(x, x + y)), Lambda(x, Float("2.5") * x)])
    seen = set()
    lambdas = [e for e in lambdas if not (e in seen or seen.add(e))]
    for lam in lambdas:
        # (IdentityFunction's variable is a Dummy, which maybe_vector drops)
        out.append(vector("lambda_nargs", [lam], lambda lam: str(lam.nargs)))
        maybe_vector(out, "lambda_variables", [lam], lambda lam: lam.variables)
        maybe_vector(out, "lambda_bound_symbols", [lam], lambda lam: lam.bound_symbols)
        maybe_vector(out, "lambda_signature", [lam], lambda lam: lam.signature)
        maybe_vector(out, "lambda_expr", [lam], lambda lam: lam.expr)
        out.append(vector("lambda_free_symbols", [lam],
                          lambda lam: sorted(lam.free_symbols, key=str)))
        out.append(vector("lambda_is_identity", [lam], lambda lam: lam.is_identity))
        # every call of the right arity, and two of the wrong one
        arity = len(lam.signature)
        right = [a for a in LAMBDA_ARGS if len(a) == arity]
        wrong = [a for a in LAMBDA_ARGS if len(a) != arity][:2]
        for args in right + wrong:
            maybe_vector(out, "lambda_call", [lam] + list(args), lambda lam, *a: lam(*a))
    return out


# ---------------------------------------------------------------------------
# WildFunction

def wild_vectors():
    out = []
    nargs_options = [{}, {"nargs": 1}, {"nargs": [1, 2]}, {"nargs": []}, {"nargs": None},
                     {"nargs": 0}, {"nargs": [2, 2, 3]}]

    def make(name, opts):
        if "nargs" in opts:
            nargs = opts["nargs"]
            return WildFunction(name, nargs=tuple(nargs) if isinstance(nargs, list) else nargs)
        return WildFunction(name)

    targets = [f(x), f(x, y), f(), x, Mod(x, y, evaluate=False), sin(x, evaluate=False),
               Lambda(x, x**2), x + y, Function("f1", nargs=1)(x)]
    for opts in nargs_options:
        out.append(vector("wild_nargs", ["F"], lambda s, o=opts: str(make(s, o).nargs), **opts))
        for t in targets:
            out.append(vector("wild_matches", ["F", t],
                              lambda s, t, o=opts: (lambda d: None if d is None else
                                                    list(d.values()))(make(s, o).matches(t)),
                              **opts))
    return out


# ---------------------------------------------------------------------------
# Mod

MOD_NUMBERS = [
    S.Zero, S.One, S.NegativeOne, Integer(2), Integer(-2), Integer(3), Integer(-3),
    Integer(5), Integer(-5), Integer(7), Integer(10), Integer(-12), Integer(100),
    Integer(2**64 + 1), Integer(-(2**65)), Integer(10**30 + 7), half, -half,
    Rational(3, 7), Rational(-22, 7), Rational(5, 3), Rational(2**70, 3**40),
    Float("1.5"), Float("-1.5"), Float("0.3"), Float("2.0"), Float("-2.0"), Float(0),
    Float("1e-20"), Float("12345.678", 30), Float("-0.75", 50), Float(3), S.Infinity,
    S.NegativeInfinity, zoo, S.NaN, I, pi, E, EulerGamma, GoldenRatio, 2 * I, 1 + I,
    sqrt(2),
]
MOD_SYMBOLS = [x, y, p, q, n, m, o, k, r, nn, A]
MOD_COMPOSITES = [
    x + 1, 2 * x, x * y, n + 1, 2 * n, 2 * n + 1, 3 * n, n * m, x + y, x**2, n**2,
    x / 2, Float("1.5") * x, Float("0.6") * x, -x, -n, 3 * x * y, p + 1, k + 1,
    k + 2, 2 * k, k**2, n * o, 6 * n, 4 * m, o + 1, x + n, x + 2 * y + 3,
    Mod(x, 3, evaluate=False), -Mod(x, 3, evaluate=False), Mod(x, 6, evaluate=False),
    Mod(n, 4, evaluate=False), Mod(x, 3, evaluate=False) + 1,
    Mod(x, 3, evaluate=False) + Mod(y, 3, evaluate=False) + 2,
    2 * Mod(n, 3, evaluate=False), n * Mod(n, 3, evaluate=False),
    Mod(x, -3, evaluate=False), Mul(4, n, evaluate=False), Mul(6, n, m, evaluate=False),
    Pow(2, 100, evaluate=False), Pow(3, 2, evaluate=False), Pow(-2, 5, evaluate=False),
    Pow(x, 2, evaluate=False), Pow(n, 3, evaluate=False), Pow(Pow(2, 3, evaluate=False), 2,
                                                               evaluate=False),
    f(x), I * x, pi * x, sqrt(2) * x,
]
MOD_DIVISORS = [
    S.Zero, S.One, S.NegativeOne, Integer(2), Integer(-2), Integer(3), Integer(-3),
    Integer(4), Integer(6), Integer(7), Integer(97), Integer(10**30 + 7), half,
    Rational(-2, 3), Float("0.5"), Float("0.3"), Float("-2.5"), S.Infinity, S.NaN, I,
    pi, x, y, p, q, n, k, m, 2 * x, x + 1, k + 1, 3 * x, Float("0.3") * x,
    Mod(x, 3, evaluate=False), -x, A,
]


def mod_vectors():
    out = []
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        for a, b in itertools.product(MOD_NUMBERS, MOD_NUMBERS):
            out.append(vector("Mod", [a, b], Mod))
        for a, b in itertools.product(MOD_SYMBOLS + MOD_COMPOSITES, MOD_DIVISORS):
            out.append(vector("Mod", [a, b], Mod))
        for a, b in itertools.product(MOD_NUMBERS[::3], MOD_SYMBOLS + MOD_COMPOSITES[::4]):
            out.append(vector("Mod", [a, b], Mod))
        for a, b in itertools.product(MOD_NUMBERS[::4] + MOD_SYMBOLS[:4],
                                      MOD_DIVISORS[::3]):
            out.append(vector("Mod", [a, b], lambda a, b: Mod(a, b, evaluate=False),
                              evaluate=False))
            out.append(vector("Mod", [a, b], under_no_evaluate(Mod), global_evaluate=False))
        for a, b in itertools.product(MOD_NUMBERS[::2] + MOD_SYMBOLS[:3]
                                      + MOD_COMPOSITES[:4],
                                      MOD_NUMBERS[1::5] + MOD_SYMBOLS[:3]):
            out.append(vector("op_mod", [a, b], lambda a, b: a % b))
        for args in [(), (x,), (x, y, z)]:
            out.append(vector("Mod", list(args), Mod))
    return out


def main():
    # (calling Lambdas builds Add(Tuple, ...) and the like, which warn)
    warnings.simplefilter("ignore")
    vectors = (undefined_vectors() + defined_vectors() + hook_vectors()
               + fact_vectors() + lambda_vectors() + wild_vectors() + mod_vectors())
    print(f"left out {len(_SKIPPED)} vectors whose outcome exprenc cannot encode")
    write("function", vectors, __doc__.split("\n\n")[0])


if __name__ == "__main__":
    main()
