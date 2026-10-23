"""Differential vectors for rsympy-core's Add, Mul and Pow (`sympy.core.add`,
`sympy.core.mul`, `sympy.core.power`).

    PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python python oracle/gen_diff_arith.py

Writes oracle/diff/core_arith.json, replayed by
crates/rsympy-core/tests/diff_arith.rs. Covers the constructors
(`Add(*args)`, `Mul(*args)`, `Pow(b, e)`, with and without `distribute`),
the unevaluated builders (`_unevaluated_Add`, `_unevaluated_Mul`,
`_keep_coeff`) and the structural methods (`as_numer_denom`, `primitive`,
`as_coeff_*`, `as_base_exp`, `could_extract_minus_sign`, negation,
`_eval_expand_power_base`).
"""

import itertools

import operator
import warnings

from sympy import (Add, And, E, Eq, Float, Function, I, Integer, Mul, Pow,
                   Rational, S, Symbol, pi, sqrt, symbols)
from sympy.core.add import _unevaluated_Add
from sympy.core.mul import _keep_coeff, _unevaluated_Mul
from sympy.core.parameters import distribute

from exprenc import vector, write

x, y, z = symbols("x y z")
p = Symbol("p", positive=True)
q = Symbol("q", negative=True)
r = Symbol("r", real=True)
k = Symbol("k", integer=True)
m = Symbol("m", even=True)
o = Symbol("o", odd=True)
nn = Symbol("nn", nonnegative=True)
A, B = symbols("A B", commutative=False)
f = Function("f")
half, third = Rational(1, 2), Rational(1, 3)

NUMBERS = [S.Zero, S.One, S.NegativeOne, Integer(2), Integer(-3), Integer(12),
           half, Rational(-2, 3), Float("1.5"), Float(0), Float("-2.0"),
           S.Infinity, S.NegativeInfinity, S.ComplexInfinity, S.NaN, I, pi, E]
ATOMS = [x, y, p, q, r, k, m, o, nn, A, B, f(x)]
COMPOSITES = [
    2 * x, -x, x * y, 3 * x * y, x**2, 1 / x, 2 * x**2, x + 1, x - y, 2 * x + 3,
    x**y, sqrt(x), sqrt(2), 2**third, 3 * sqrt(2), I * x, pi * x, x / 2,
    A * B, B * A, A**2, 2 * A, x * A, sqrt(6), x**k, p**half, (x + 1)**2,
    -2 * x - 2, x * (x + 1),
    # unevaluated inputs, as other code builds them
    Pow(2, 3, evaluate=False), Pow(-8, third, evaluate=False),
    Pow(4, -half, evaluate=False), Mul(2, x + 1, evaluate=False),
    Mul(-1, x, evaluate=False), Add(x, x, evaluate=False),
    Pow(x, 0, evaluate=False), Pow(x**2, 3, evaluate=False),
]
POOL = NUMBERS + ATOMS + COMPOSITES
SMALL = [S.Zero, S.One, Integer(2), half, Float("1.5"), S.Infinity, I, x, y,
         2 * x, x**2, x + 1, sqrt(2), A]
EXPONENTS = [S.Zero, S.One, S.NegativeOne, Integer(2), Integer(-2), Integer(3),
             half, -half, third, Rational(2, 3), Rational(3, 2), x, k, m,
             S.Infinity, S.NegativeInfinity, S.ComplexInfinity, S.NaN,
             Float("2.0"), Float("0.5"), I]
COEFFS = [S.One, S.NegativeOne, Integer(2), Integer(-3), half, Rational(-2, 3),
          Float("1.5"), S.Zero, S.Infinity, x]
# Booleans are not Exprs: the operators raise TypeError, the constructors
# AttributeError (with a deprecation warning) or build odd objects.
BOOLEANS = [S.true, S.false, Eq(x, 1), And(x < 1, y > 2)]
OPERANDS = [Integer(2), half, Float("1.5"), x, pi, S.Infinity, 2 * x] + BOOLEANS
OPERATORS = {"op_add": operator.add, "op_sub": operator.sub,
             "op_mul": operator.mul, "op_div": operator.truediv,
             "op_pow": operator.pow}


def no_distribute(fn):
    def run(*args):
        with distribute(False):
            return fn(*args)
    return run


def main():
    vectors = []
    for a, b in itertools.product(POOL, POOL):
        vectors.append(vector("Add", [a, b], lambda *a: Add(*a)))
        vectors.append(vector("Mul", [a, b], lambda *a: Mul(*a)))
    for args in itertools.combinations_with_replacement(SMALL, 3):
        vectors.append(vector("Add", list(args), lambda *a: Add(*a)))
        vectors.append(vector("Mul", list(args), lambda *a: Mul(*a)))
    for a, b in itertools.product(SMALL, POOL):
        vectors.append(vector("Mul", [a, b], no_distribute(lambda *a: Mul(*a)),
                              distribute=False))
    for b, e in itertools.product(POOL, EXPONENTS):
        vectors.append(vector("Pow", [b, e], Pow))
    for name, op in OPERATORS.items():
        for a, b in itertools.product(OPERANDS, OPERANDS):
            if a in BOOLEANS or b in BOOLEANS:
                vectors.append(vector(name, [a, b], op))
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        for a, b in itertools.product(BOOLEANS[:2], [Integer(2), x]):
            for cls in (Add, Mul, Pow):
                vectors.append(vector(cls.__name__, [a, b], cls))
                vectors.append(vector(cls.__name__, [b, a], cls))

    # results of the constructors are the inputs of the structural methods
    exprs = list(POOL)
    for a, b in itertools.product(SMALL, SMALL):
        exprs.extend([Add(a, b), Mul(a, b), Pow(a, b) if a != 0 else a])
    seen = set()
    exprs = [e for e in exprs if not (e in seen or seen.add(e))]
    for e in exprs:
        vectors.append(vector("neg", [e], lambda e: -e))
        vectors.append(vector("as_numer_denom", [e], lambda e: e.as_numer_denom()))
        vectors.append(vector("could_extract_minus_sign", [e],
                              lambda e: e.could_extract_minus_sign()))
        vectors.append(vector("as_coeff_Mul", [e], lambda e: e.as_coeff_Mul()))
        vectors.append(vector("as_coeff_Mul_rational", [e],
                              lambda e: e.as_coeff_Mul(rational=True)))
        vectors.append(vector("as_coeff_Add", [e], lambda e: e.as_coeff_Add()))
        vectors.append(vector("as_coeff_mul", [e], lambda e: e.as_coeff_mul()))
        vectors.append(vector("as_coeff_add", [e], lambda e: e.as_coeff_add()))
        vectors.append(vector("as_base_exp", [e], lambda e: e.as_base_exp()))
        if e.is_Add:
            vectors.append(vector("primitive", [e], lambda e: e.primitive()))
        if e.is_Pow:
            vectors.append(vector("expand_power_base", [e],
                                  lambda e: e._eval_expand_power_base()))
            vectors.append(vector("expand_power_base_force", [e],
                                  lambda e: e._eval_expand_power_base(force=True)))
    for c, e in itertools.product(COEFFS, exprs[::3]):
        vectors.append(vector("keep_coeff", [c, e], _keep_coeff))
        vectors.append(vector("keep_coeff_noclear_sign", [c, e],
                              lambda c, e: _keep_coeff(c, e, clear=False, sign=True)))
    for args in itertools.combinations_with_replacement(SMALL, 3):
        vectors.append(vector("unevaluated_Add", list(args), _unevaluated_Add))
        vectors.append(vector("unevaluated_Mul", list(args), _unevaluated_Mul))
    write("arith", vectors, __doc__.split("\n\n")[0])


if __name__ == "__main__":
    main()
