"""Differential vectors for rsympy-core's numbers (`sympy.core.numbers`).

    PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python python oracle/gen_diff_numbers.py

Writes oracle/diff/core_numbers.json, replayed by
crates/rsympy-core/tests/diff_numbers.rs. Covers number-with-number
arithmetic (which never reaches Add/Mul), integer powers, negation, abs,
and every assumption fact on numbers and numeric singletons.

This is also the template for other areas' generators: build inputs, call
SymPy through `vector(op, args, fn)`, and `write` them.
"""

import itertools

from sympy import Float, Integer, Rational, S
from sympy.core.assumptions import _assume_defined

from exprenc import vector, write

INTEGERS = [Integer(n) for n in (0, 1, -1, 2, -3, 7, 12, 2**64 + 1, -(10**30))]
RATIONALS = [Rational(1, 2), Rational(-3, 7), Rational(22, 7), Rational(2**70, 3**40)]
FLOATS = [Float("1.5"), Float("0.1"), Float("-2.25", 30), Float(3), Float(0),
          Float("1e-20"), Float("12345.678", 50)]
SINGLETONS = [S.Infinity, S.NegativeInfinity, S.NaN]
NUMBERS = INTEGERS + RATIONALS + FLOATS + SINGLETONS
OTHER_ATOMS = [S.ComplexInfinity, S.ImaginaryUnit, S.Exp1, S.Pi, S.EulerGamma,
               S.Catalan, S.GoldenRatio, S.TribonacciConstant]

BINARY = {
    "add": lambda a, b: a + b,
    "sub": lambda a, b: a - b,
    "mul": lambda a, b: a * b,
    "div": lambda a, b: a / b,
    "mod": lambda a, b: a % b,
}


def main():
    vectors = []
    for (name, fn), a, b in itertools.product(BINARY.items(), NUMBERS, NUMBERS):
        vectors.append(vector(name, [a, b], fn))
    # Python ints on the right, as in `x + 3`.
    for (name, fn), a, b in itertools.product(BINARY.items(), NUMBERS, (0, 1, -2, 10**20)):
        vectors.append(vector(name, [a, b], fn))
    for a, b in itertools.product(INTEGERS, INTEGERS):
        vectors.append(vector("floordiv", [a, b], lambda a, b: a // b))
    for a, n in itertools.product(NUMBERS, (0, 1, 2, 3, -1, -2, 5)):
        vectors.append(vector("pow", [a, Integer(n)], lambda a, n: a**n))
    for a in NUMBERS:
        vectors.append(vector("neg", [a], lambda a: -a))
        vectors.append(vector("abs", [a], abs))
    for a, fact in itertools.product(NUMBERS + OTHER_ATOMS, sorted(_assume_defined)):
        vectors.append(vector("is", [a, fact], lambda a, f: getattr(a, "is_" + f)))
    write("numbers", vectors, __doc__.split("\n\n")[0])


if __name__ == "__main__":
    main()
