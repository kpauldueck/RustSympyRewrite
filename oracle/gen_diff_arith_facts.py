"""Differential vectors for the assumptions of Add, Mul and Pow (their
`_eval_is_*` handlers and `_monotonic_sign`).

    PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python python oracle/gen_diff_arith_facts.py

Writes oracle/diff/core_arith_facts.json, replayed by
crates/rsympy-core/tests/diff_arith_facts.rs: every assumption fact of
sums, products and powers of symbols with assumptions and numbers.

A few facts have no single answer in SymPy itself. `_ask` tries the
prerequisite facts in set order (which depends on Python's string hashing)
shuffled by a seeded RNG, and some handlers disagree with each other
(`Mul._eval_is_even` says `Mul(p, 0).is_even` is False), so e.g.
`Mul(p, 0, evaluate=False).is_hermitian` is True under most hash seeds and
None under others. rsympy visits prerequisites in sorted (fact) order, so
this generator replaces SymPy's shuffle with a controlled one and records
SymPy's answer for that order, which makes the file independent of the
hash seed. It then retries each vector under ORDERS random orders and
leaves it out if any answer differs: SymPy has no single answer for it.
(A vector that survives is still SymPy's answer for fact order, even if
some untried order would change it.)
"""

import itertools
import random
import sys

from sympy import Add, I, Integer, Mul, Pow, Rational, S, Symbol, sqrt
from sympy.core.assumptions import _assume_defined

from exprenc import dec, enc, fresh_state, value, vector, write
from gen_diff_arith import A, B, POOL, k, m, nn, o, p, q, r, x, y

pos2 = Symbol("pos2", positive=True)
npos = Symbol("npos", nonpositive=True)
nz = Symbol("nz", nonzero=True, real=True)
pi_ = Symbol("pi_", prime=True)
c_ = Symbol("c_", composite=True)
ext = Symbol("ext", extended_real=True)
inf = Symbol("inf", infinite=True)
im = Symbol("im", imaginary=True)
rat = Symbol("rat", rational=True)
irr = Symbol("irr", irrational=True)
fin = Symbol("fin", finite=True)
half = Rational(1, 2)

FACT_ATOMS = [x, p, q, r, k, m, o, npos, nz, pi_, c_, ext, inf, im, irr, A,
              S.Zero, Integer(-3), half, S.Infinity, I, S.Pi]
EXPONENTS = [S.NegativeOne, Integer(2), Integer(3), half, -half, k, o, p, q,
             im, inf, x]
# (nn, pos2, rat, fin and E only appear in the hand-picked list below)
ORDERS = 4

_rng = None    # None: visit in sorted order; else shuffle with it
_chose = False  # whether some batch of prerequisites had more than one fact


def _visit_order(facts):
    """Stands in for `shuffle` in `sympy.core.assumptions._ask`."""
    global _chose
    _chose = _chose or len(facts) > 1
    facts.sort()
    if _rng is not None:
        _rng.shuffle(facts)


def is_(e, fact):
    return getattr(e, "is_" + fact)


def answer(text, fact, rng):
    """The outcome of `dec(text).is_<fact>` from a fresh state, visiting the
    prerequisites in the order `rng` shuffles them into."""
    global _rng
    fresh_state()
    _rng = rng
    try:
        return value(is_(dec(text), fact))
    except Exception as exc:  # noqa: BLE001: the exception is the answer
        return {"error": type(exc).__name__}
    finally:
        _rng = None


def main():
    global _chose
    exprs = []
    for a, b in itertools.combinations(FACT_ATOMS, 2):
        exprs.append(Add(a, b, evaluate=False))
        exprs.append(Mul(a, b, evaluate=False))
    for b, e in itertools.product(FACT_ATOMS, EXPONENTS):
        exprs.append(Pow(b, e, evaluate=False))
    for args in itertools.combinations([p, q, k, m, o, Integer(2), half, I], 3):
        exprs.append(Add(*args, evaluate=False))
        exprs.append(Mul(*args, evaluate=False))
    exprs.extend(a for a in POOL if a.is_Add or a.is_Mul or a.is_Pow)
    exprs.extend([p + 1, q - 1, k + 1, 2 * k, 2 * k + 1, m + o, m * o, o * o,
                  sqrt(p), sqrt(q), p**q, k**2, (-p)**half, I * p, x * y,
                  nn + 1, npos - 2, pos2 * p, pos2 + p, rat + irr, rat * fin,
                  fin + inf, S.Exp1 * p, Pow(S.Exp1, 2, evaluate=False),
                  pi_ + 1, c_ * pi_, 2 * pi_, pi_**2,
                  Mul(2, k, Pow(2, -1), evaluate=False), k / 2, m / 2, o / 2,
                  Mul(m, Pow(4, -1), evaluate=False), 3 * x, -p, -k, A * B,
                  Pow(p, 0, evaluate=False), Pow(0, p, evaluate=False),
                  Pow(S.Exp1, x, evaluate=False), Pow(S.Exp1, I * p, evaluate=False)])
    seen = set()
    exprs = [e for e in exprs if not (e in seen or seen.add(e))]

    # (`sympy.core.assumptions` is a function: the module is shadowed)
    sys.modules["sympy.core.assumptions"].shuffle = _visit_order
    vectors, dropped = [], 0
    for e, fact in itertools.product(exprs, sorted(_assume_defined)):
        _chose = False
        vec = vector("is", [e, fact], is_)
        # if every batch had one fact, there is no other order to try
        if _chose and any(answer(enc(e), fact, random.Random(i)) != vec["out"]
                          for i in range(ORDERS)):
            dropped += 1
            continue
        vectors.append(vec)
    print(f"dropped {dropped} vectors whose answer depends on the order "
          f"_ask visits facts in")
    write("arith_facts", vectors, __doc__.split("\n\n")[0])


if __name__ == "__main__":
    main()
