"""Generate differential test vectors for rsympy's number crates.

    PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python .venv/bin/python oracle/gen_diff.py

Writes oracle/diff/*.json. Every expected value is computed here, live, by
CPython or the pinned SymPy; the Rust tests in crates/rsympy-num/tests/ and
crates/rsympy-ntheory/tests/ replay them. CI regenerates the files and fails
if they differ from what is committed, so nobody edits them by hand.

- num.json: Python int semantics (floor division, modulo, pow with modulus,
  shifts, float conversion, math.gcd/lcm/isqrt), fractions.Fraction, and
  random.Random (MT19937 seeding, getrandbits, randint, random).
- ntheory.json: SymPy's isprime, nextprime, perfect_power, multiplicity,
  integer_nthroot and factorint. factorint calls run as one sequence from a
  fresh state, because SymPy's factor_cache and sympy.core.random.rng carry
  over from call to call and can change what factorint(n, limit=...)
  returns.
"""

import json
import math
import pathlib
import random
from fractions import Fraction

import sympy as sp
from sympy import factorint, integer_nthroot, isprime, multiplicity, nextprime, perfect_power
from sympy.external.gmpy import GROUND_TYPES
from sympy.ntheory import factor_

assert sp.__version__ == "1.14.0", sp.__version__
assert GROUND_TYPES == "python", "set SYMPY_GROUND_TYPES=python"

OUT = pathlib.Path(__file__).parent / "diff"


def fresh_state():
    sp.core.cache.clear_cache()
    factor_.factor_cache.clear()
    random.seed(0)
    sp.core.random.seed(0)


def ints(rng, count):
    """A mix of small, word-boundary and big integers of both signs."""
    edges = [0, 1, -1, 2, -2, 7, -7, 2**31 - 1, 2**31, 2**32, 2**53, 2**53 + 1,
             2**62, 2**63 - 1, 2**63, -(2**63), -(2**63) - 1, 2**64 - 1, 2**64,
             10**18, 10**19, -(10**30), 3**200, -(5**150)]
    out = list(edges)
    for _ in range(count):
        bits = rng.choice([3, 10, 31, 32, 33, 62, 63, 64, 65, 100, 200, 500, 1100])
        v = rng.getrandbits(bits)
        out.append(-v if rng.random() < 0.4 else v)
    return out


def float_or_error(f):
    try:
        return repr(f())
    except OverflowError:
        return "OverflowError"
    except ZeroDivisionError:
        return "ZeroDivisionError"


def num_vectors():
    rng = random.Random(20261023)
    xs = ints(rng, 120)
    pairs = [(rng.choice(xs), rng.choice(xs)) for _ in range(600)]
    binops = []
    for a, b in pairs:
        row = {"a": str(a), "b": str(b), "add": str(a + b), "sub": str(a - b), "mul": str(a * b),
               "and": str(a & b), "gcd": str(math.gcd(a, b)), "lcm": str(math.lcm(a, b)),
               "cmp": (a > b) - (a < b)}
        if b:
            row["floordiv"] = str(a // b)
            row["mod"] = str(a % b)
            e = abs(rng.choice(xs)) % 5000
            row["powmod_e"] = str(e)
            row["powmod"] = str(pow(a, e, b))
        binops.append(row)
    unops = []
    for a in xs:
        k = rng.randrange(0, 130)
        n = rng.randrange(1, 12)
        unops.append({
            "a": str(a), "neg": str(-a), "abs": str(abs(a)), "bit_length": a.bit_length(),
            "shift": k, "shl": str(a << k), "shr": str(a >> k),
            "float": float_or_error(lambda: float(a)),
            "isqrt": str(math.isqrt(abs(a))),
            "root_n": n, "root": str(integer_nthroot(abs(a), n)[0]),
            "root_exact": integer_nthroot(abs(a), n)[1],
            "pow_e": k % 12, "pow": str(a ** (k % 12)),
        })
    fracs = []
    for _ in range(300):
        p, q, r, s = (rng.choice(xs) for _ in range(4))
        if not q or not s:
            continue
        f, g = Fraction(p, q), Fraction(r, s)
        row = {"p": str(p), "q": str(q), "r": str(r), "s": str(s), "f": str(f),
               "add": str(f + g), "sub": str(f - g), "mul": str(f * g),
               "div": str(f / g) if g else "ZeroDivisionError",
               "floor": str(math.floor(f)), "ceil": str(math.ceil(f)),
               "float": float_or_error(lambda: float(f)), "cmp": (f > g) - (f < g)}
        fracs.append(row)
    rand = []
    for seed in [0, 1, 42, 1234, 2**32 - 1, 2**32, 2**40 + 3, -5, 10**30, 12345678901234567890123]:
        r = random.Random(seed)
        draws = []
        for k in [1, 5, 31, 32, 33, 64, 100, 0]:
            draws.append(["getrandbits", k, str(r.getrandbits(k))])
        for a, b in [(0, 10), (1, 6), (6, 10**20), (-(10**12), 10**12), (5, 5)]:
            draws.append(["randint", [str(a), str(b)], str(r.randint(a, b))])
        draws.append(["random", None, repr(r.random())])
        rand.append({"seed": str(seed), "draws": draws})
    return {"binops": binops, "unops": unops, "fractions": fracs, "random": rand}


def factorint_sequence():
    """factorint calls in order, sharing SymPy's state as a session would."""
    calls = [
        (360, None), (2147483647, None), (600851475143, None), (1000000000039, None),
        (2**64 + 1, None), (10**20 + 1, None), (2**67 - 1, None), (3**40 * 7, None),
        (1407633717262338957430697921446883, 10000),
        (12, 2**15), (2**15 * 3**7, 2**15), (1000003 * 1000033, 2**15),
        (32767**2 * 5, 2**15), (99991 * 99989, 2**15), (2**100 - 1, 2**15),
        ((2**31 - 1) * (2**61 - 1), None), (1000000007 * 998244353, None),
        (10**24 + 7, None), ((10**12 + 39) * (10**12 + 61), None),
        (-360, None), (0, None), (1, None), (9, 5), (97, 1), (10**30 + 57, 2**15),
        (5**30 * 3, 100), (123456789 * 987654321, 2**15), (2**89 - 1, None),
        (600851475143, 2**15), (1000000007 * 998244353, 2**15),
        ((10**13 + 37) * (10**13 + 51) * 3, None),
        ((10**11 + 3) * (10**15 + 37), None), ((2**41 - 1) * (10**17 + 3), None),
        (2**128 + 1, None), ((10**11 + 3) * (10**15 + 37), 2**15),
    ]
    out = []
    for n, limit in calls:
        f = factorint(n, limit=limit)
        out.append({"n": str(n), "limit": None if limit is None else str(limit),
                    "factors": [[str(p), e] for p, e in f.items()]})
    return out


def ntheory_vectors():
    fresh_state()
    rng = random.Random(7)
    prime_inputs = [-7, 0, 1, 2, 3, 4, 5, 25, 49, 2809, 8321, 31621, 65077, 65537, 341531,
                    4296595241, 350269456337, 55245642489451, 7999252175582851,
                    585226005592931977, 2**64 - 59, 2**64 + 13, 2**89 - 1, 2**127 - 1,
                    561, 1105, 3215031751, 3825123056546413051,
                    318665857834031151167461, 3317044064679887385961981,
                    10**30 + 57, 10**50 + 151, (2**61 - 1) * (2**31 - 1)]
    prime_inputs += [rng.getrandbits(rng.choice([16, 32, 48, 64, 80, 128])) | 1 for _ in range(150)]
    isp = [[str(n), isprime(n)] for n in prime_inputs]
    nxt = [[str(n), str(nextprime(n))] for n in [-3, 0, 1, 2, 3, 13, 89, 1000, 65536, 10**9,
                                                 2**31, 2**64, 10**30, 2**127]]
    pp_inputs = [4, 8, 16, 17, 27, 64, 81, 100, 1000, 1024, 3**8, 5**12, 10**6, 999999,
                 2**10 * 3**10, 2**64, 3**41, (2**61 - 1)**3, 7**30 * 11**15, 6**25,
                 -8, -4, -64, -(3**15), -(2**60), 10**40, 12**20 + 1, 2**127 - 1,
                 (10**12 + 39)**5, 2**3 * 3**6, 2**2 * 3**3, 5**7 * 7**14, 1, 0, 2, 3]
    pp = []
    for n in pp_inputs:
        r = perfect_power(n)
        pp.append([str(n), None if r is False else [str(r[0]), r[1]]])
    mult = []
    for p, n in [(2, 1024), (3, 3**40 * 2), (5, 7), (7, 7**100), (10, 10**25 * 3), (2, -96),
                 (3, 10**50), (6, 6**30 * 5), (97, 97**12)]:
        mult.append([str(p), str(n), multiplicity(p, n)])
    roots = []
    for y, n in [(0, 3), (1, 5), (8, 3), (9, 3), (10**30, 3), (2**200 + 1, 2), (3**100, 50),
                 (12345678901234567890, 7), (2**100, 101), (5, 1)]:
        r, e = integer_nthroot(y, n)
        roots.append([str(y), n, str(r), e])
    from sympy.external.ntheory import is_square, jacobi
    sq_inputs = [0, 1, 2, 4, 15, 16, 17, 793, 1408, 10**20, 10**20 + 1, (2**70 + 3)**2,
                 (2**70 + 3)**2 - 1, -4] + [rng.getrandbits(90)**2 for _ in range(10)]
    sq_inputs += [rng.getrandbits(rng.choice([8, 40, 90])) for _ in range(60)]
    squares = [[str(n), is_square(n)] for n in sq_inputs]
    jac = []
    for _ in range(80):
        y = rng.getrandbits(rng.choice([5, 30, 70])) | 1
        x = rng.getrandbits(rng.choice([5, 30, 70])) - 2**20
        jac.append([str(x), str(y), jacobi(x, y)])
    fresh_state()
    seq = factorint_sequence()
    return {"is_square": squares, "jacobi": jac, "isprime": isp, "nextprime": nxt, "perfect_power": pp, "multiplicity": mult,
            "integer_nthroot": roots, "factorint": seq}


def main():
    OUT.mkdir(exist_ok=True)
    for name, gen in [("num", num_vectors), ("ntheory", ntheory_vectors)]:
        data = gen()
        doc = {"sympy_version": sp.__version__, **data}
        (OUT / f"{name}.json").write_text(json.dumps(doc, indent=0) + "\n")
        print(name, {k: len(v) for k, v in data.items()})


if __name__ == "__main__":
    main()
