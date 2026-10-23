"""Generate differential test vectors for rsympy-mpf milestone M2.

    PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python .venv/bin/python oracle/gen_diff_mpf2.py

Writes oracle/diff/mpf2.json, which crates/rsympy-mpf/tests/diff_m2.rs
replays. Every expected value comes from the pinned mpmath (SymPy 1.14's
dependency), run live here with its pure-Python backend:

- libelefun: powers, roots, exp, log, the trigonometric and hyperbolic
  functions and their inverses (mpf_*), the fixed-point kernels under them,
  fibonacci and the constants;
- libmpc: every function, on all four quadrants, the axes and the branch
  cuts, with infinite and nan parts;
- libhyper: mpf_agm, mpc_agm and the generic hypergeometric summator (run
  through ctx.hypsum);
- the number context: mpf/mpc operators, the wrapped functions and their
  fallback from real to complex, the elementary functions of
  functions.py, the precision managers, _convert_param and hypsum.

mpmath memoizes its constants (a value memoized at a higher precision is
truncated, not recomputed, so the last bits depend on history) and keeps two
history-dependent caches (atan_taylor_cache, log_int_cache). Every vector in
"vectors" runs from a reset of all three and of the context; each list in
"sequences" runs in order from one reset, to test that history.

A vector is {"f": name, "a": args, "p": prec, "r": rounding, "o": result};
ctx vectors may add "tc" (trap_complex). Encodings: an mpf is
[sign, "man", exp, bc]; an mpc {"c": [mpf, mpf]}; an int {"i": "n"}; a
tuple {"t": [...]}; a float {"fb": "IEEE bits as a signed int", or "nan"}; an mpq
{"q": ["p", "q"]}; a string {"s": "..."}; a hypergeometric parameter
{"Z": int} / {"Q": mpq} / {"R": mpf} / {"C": mpc} / {"U": value}; an
exception {"e": "ClassName"}.
"""

import json
import pathlib
import random
import struct
import sys

import mpmath
import sympy as sp
from mpmath import mp
from mpmath.libmp import backend
from mpmath.libmp import libelefun as E
from mpmath.libmp import libhyper as H
from mpmath.libmp import libintmath as LI
from mpmath.libmp import libmpc as C
from mpmath.libmp import libmpf as M
from mpmath.rational import mpq

assert sp.__version__ == "1.14.0", sp.__version__
assert backend.BACKEND == "python", "mpmath must use its Python backend (no gmpy2)"

OUT = pathlib.Path(__file__).parent / "diff" / "mpf2.json"

# ifib(10**5) and friends have more digits than str() allows by default.
sys.set_int_max_str_digits(0)


# ---------------------------------------------------------------------------
# State
# ---------------------------------------------------------------------------


def _constant_memos():
    found = {}
    for mod in (mpmath.libmp.libelefun, mpmath.libmp.gammazeta):
        for name in sorted(dir(mod)):
            g = getattr(mod, name)
            for cell in getattr(g, "__closure__", None) or ():
                f = cell.cell_contents
                if callable(f) and hasattr(f, "memo_prec"):
                    found[id(f)] = f
    return list(found.values())


MEMOS = _constant_memos()


def reset():
    for f in MEMOS:
        f.memo_prec = -1
        f.memo_val = None
    E.atan_taylor_cache.clear()
    E.log_int_cache.clear()
    mp.prec = 53
    mp._prec_rounding[1] = "n"
    mp.trap_complex = False


# ---------------------------------------------------------------------------
# Encoding
# ---------------------------------------------------------------------------


def Mf(t):
    return [t[0], str(t[1]), t[2], t[3]]


def Cz(z):
    return {"c": [Mf(z[0]), Mf(z[1])]}


def I(n):
    return {"i": str(n)}


def T(*xs):
    return {"t": list(xs)}


def F(x):
    if x != x:
        # A nan's sign and payload depend on how the platform made it
        # (mpmath's is inf - inf); Python never shows them.
        return {"fb": "nan"}
    return {"fb": str(struct.unpack("<q", struct.pack("<d", x))[0])}


def V(x):
    """A ctx value (mpf, mpc, constant)."""
    if hasattr(x, "_mpc_"):
        return Cz(x._mpc_)
    return Mf(x._mpf_)


def Q(x):
    p, q = x._mpq_
    return {"q": [str(p), str(q)]}


def P(x, flag):
    if flag == "Z":
        return {"Z": I(x)}
    if flag == "Q":
        return {"Q": Q(x)}
    if flag == "R":
        return {"R": Mf(x._mpf_)}
    if flag == "C":
        return {"C": Cz(x._mpc_)}
    return {"U": V(x)}


def run(f, enc):
    try:
        return enc(f())
    except RecursionError:
        return {"e": "RecursionError"}
    except Exception as e:  # noqa: BLE001 - the class name is the answer
        return {"e": type(e).__name__}


VECTORS = []
SEQUENCES = []


def vec(name, args, f, enc, prec=None, rnd=None, **extra):
    """Run f() from a fresh state and record it."""
    reset()
    v = {"f": name, "a": args}
    if prec is not None:
        v["p"] = prec
    if rnd is not None:
        v["r"] = rnd
    v.update(extra)
    v["o"] = run(f, enc)
    VECTORS.append(v)


# ---------------------------------------------------------------------------
# Inputs
# ---------------------------------------------------------------------------

rng = random.Random(20250623)
RNDS = "nfcud"
SMALL_PRECS = [1, 2, 3, 4, 5, 7, 8, 10, 16, 24, 30, 33, 53, 54, 60, 64, 80, 100, 113, 128, 150, 200]
BIG_PRECS = [256, 300, 333, 400, 500, 600, 601, 700, 1000, 1200, 1500, 1600, 2000, 2500, 2600, 3000, 3100]
SPECIALS = [M.fzero, M.finf, M.fninf, M.fnan]


def rprec(big=0.2):
    if rng.random() < big:
        return rng.choice(BIG_PRECS)
    return rng.choice(SMALL_PRECS)


def rrnd():
    return rng.choice(RNDS)


def rbits():
    r = rng.random()
    if r < 0.85:
        return rng.choice([1, 2, 3, 5, 8, 10, 20, 30, 52, 53, 54, 64, 80, 100, 120, 200])
    return rng.choice([300, 500, 1000, 2000, 3200])


def rmpf(lo, hi, sign=None, bits=None, special=0.04):
    """A random mpf with magnitude (exp + bc) in [lo, hi]."""
    if rng.random() < special:
        return rng.choice(SPECIALS + [M.fone, M.fnone, M.ftwo, M.fhalf])
    bits = bits or rbits()
    man = rng.getrandbits(bits) | (1 << (bits - 1))
    mag = rng.randint(lo, hi)
    s = rng.random() < 0.5 if sign is None else sign
    return M.from_man_exp(-man if s else man, mag - bits)


def near(base, sign=None):
    """base * (1 +- 2^-k) for an exact mpf base."""
    k = rng.choice([1, 2, 5, 10, 30, 53, 60, 100, 200, 500, 1000, 3000])
    eps = M.from_man_exp(1, -k)
    if sign is None:
        sign = rng.random() < 0.5
    f = M.mpf_sub(M.fone, eps) if sign else M.mpf_add(M.fone, eps)
    return M.mpf_mul(base, f)


PI_HI = E.mpf_pi(8000)
reset()


def near_pi_multiple(halves=True):
    """k*pi/2 (or k*pi) rounded to a random number of bits."""
    k = rng.choice([1, 2, 3, 4, 5, 7, 10, 100, 1001, 12345, 10**6 + 3, 2**40 + 1])
    if rng.random() < 0.3:
        k = -k
    bits = rng.choice([10, 24, 53, 64, 100, 200, 500, 1000, 3000])
    x = M.mpf_mul(PI_HI, M.from_int(k), bits + 20)
    if halves:
        x = M.mpf_shift(x, -1)
    x = M.mpf_pos(x, bits, rng.choice(RNDS))
    if rng.random() < 0.3:
        x = M.mpf_add(x, M.from_man_exp(rng.choice([1, -1]), -rng.randint(5, 200)), bits + 300)
    return x


def rint_mpf(lo=0, hi=200):
    n = rng.randint(lo, hi)
    if rng.random() < 0.4:
        n = -n
    return M.from_int(n)


def rmpc(lo, hi, special=0.05):
    r = rng.random()
    if r < special:
        a = rng.choice(SPECIALS + [M.fone, M.fnone])
        b = rmpf(lo, hi, special=0.3)
        return (a, b) if rng.random() < 0.5 else (b, a)
    if r < 0.15:
        # on an axis
        a = rmpf(lo, hi, special=0)
        return (a, M.fzero) if rng.random() < 0.5 else (M.fzero, a)
    return (rmpf(lo, hi, special=0), rmpf(lo, hi, special=0))


def quadrants(z):
    """z in all four quadrants."""
    a, b = M.mpf_abs(z[0]), M.mpf_abs(z[1])
    return [(a, b), (M.mpf_neg(a), b), (M.mpf_neg(a), M.mpf_neg(b)), (a, M.mpf_neg(b))]


def tiny():
    return M.from_man_exp(rng.choice([1, -1, 3, -5]), -rng.choice([10, 53, 100, 300, 1000]))


def cut_points():
    """Points on and next to the branch cuts of log, sqrt, asin, acos,
    atan, asinh, acosh, atanh and the axes."""
    pts = []
    for x in [M.from_int(2), M.from_float(1.5), M.from_int(-2), M.from_float(-1.5),
              M.fone, M.fnone, M.fhalf, M.mpf_neg(M.fhalf), M.from_int(10**6),
              M.from_int(-(10**6)), M.from_man_exp(1, -60), rmpf(-5, 5), rmpf(1, 10)]:
        pts.append((x, M.fzero))
        pts.append((M.fzero, x))
        for _ in range(2):
            t = tiny()
            pts.append((x, t))
            pts.append((t, x))
    for z in [(M.fnone, M.fone), (M.fone, M.fone), (M.from_int(-1), M.from_man_exp(1, -100)),
              (M.from_man_exp(-1, -100), M.fone), (M.fzero, M.fzero)]:
        pts.append(z)
    return pts


# ---------------------------------------------------------------------------
# libelefun
# ---------------------------------------------------------------------------


def elefun_vectors():
    m = lambda t: Mf(t)  # noqa: E731

    def unary(name, f, xs, prec_big=0.2):
        for x in xs:
            p, r = rprec(prec_big), rrnd()
            vec(name, [Mf(x)], lambda: f(x, p, r), m, p, r)

    def gen_x(lo, hi, n, extra=()):
        xs = list(extra)
        for _ in range(n):
            xs.append(rmpf(lo, hi))
        return xs

    edges = SPECIALS + [M.fone, M.fnone, M.ftwo, M.fhalf, M.from_int(3), M.from_int(-7),
                        M.from_int(100), M.from_man_exp(1, -1000), M.from_man_exp(-3, -2000),
                        M.from_man_exp(1, 1000)]

    # exp: magnitudes up to 2^40 keep the result's exponent in i64.
    xs = gen_x(-3000, 6, 140, edges[:-1] + [M.from_int(-745), M.from_int(710), M.from_int(2**30),
                                            M.from_int(-(2**30)), M.from_man_exp(3, 38)])
    xs += [rint_mpf(1, 10**6) for _ in range(20)]
    xs += [rmpf(7, 40) for _ in range(20)]
    xs += [near(E.mpf_ln2(200), None) for _ in range(5)]
    unary("mpf_exp", E.mpf_exp, xs)

    # cosh/sinh/tanh
    xs = gen_x(-3000, 8, 120, edges[:-1] + [M.from_man_exp(1, -5), M.from_man_exp(1, -4), M.from_int(2**12),
                                            M.from_int(-(2**12)), M.from_man_exp(5, 25)])
    xs += [rmpf(9, 30) for _ in range(20)]
    for x in xs:
        p, r = rprec(), rrnd()
        vec("mpf_cosh_sinh", [Mf(x)], lambda: E.mpf_cosh_sinh(x, p, r), lambda t: T(Mf(t[0]), Mf(t[1])), p, r)
    unary("mpf_cosh", E.mpf_cosh, xs[::3])
    unary("mpf_sinh", E.mpf_sinh, xs[1::3])
    unary("mpf_tanh", E.mpf_tanh, xs[::2])

    # log: positive values over a huge range, near 1, exact powers of 2,
    # integers, values in [1/4, 1/2) (mag -1 taking the |mag| <= 1 path)
    xs = gen_x(-100000, 100000, 60, edges)
    xs += [rmpf(-3000, 3000, sign=False) for _ in range(80)]
    xs += [near(M.fone) for _ in range(40)]
    xs += [M.from_man_exp(1, rng.randint(-10**6, 10**6)) for _ in range(10)]
    xs += [M.from_int(rng.randint(2, 10**12)) for _ in range(15)]
    xs += [rmpf(-1, -1, sign=False, bits=b) for b in (2, 10, 53, 100, 1000, 3000)]
    xs += [rmpf(0, 1, sign=False, bits=b) for b in (2, 10, 53, 100, 1000, 3000)]
    xs += [rmpf(10001, 10**7, sign=False) for _ in range(10)]
    unary("mpf_log", E.mpf_log, xs, 0.3)

    # pow
    for _ in range(220):
        s = rmpf(-20, 20, special=0.05)
        r0 = rng.random()
        if r0 < 0.25:
            t = rint_mpf(0, 300)
        elif r0 < 0.4:
            t = M.from_man_exp(rng.choice([1, -1, 3, -3, 5, 7, -9]), -1)
        else:
            t = rmpf(-12, 12, special=0.05)
        p, r = rprec(), rrnd()
        vec("mpf_pow", [Mf(s), Mf(t)], lambda: E.mpf_pow(s, t, p, r), m, p, r)

    # nthroot / cbrt
    ns = [0, 1, -1, 2, -2, 3, -3, 4, 5, 7, 10, 11, 20, 21, 25, 50, -50, 100, 1000, 20000, 30000]
    for _ in range(200):
        s = rmpf(-3000, 3000, special=0.06) if rng.random() < 0.3 else rmpf(-60, 60, special=0.06)
        n = rng.choice(ns)
        p, r = rprec(), rrnd()
        vec("mpf_nthroot", [Mf(s), n], lambda: E.mpf_nthroot(s, n, p, r), m, p, r)
    # Newton's method with a large n, whose float start overflows
    for n, p in [(100, 3000), (300, 3000), (1000, 3000), (-200, 3000), (50, 2000), (21, 1200)]:
        s = rmpf(-100, 100, sign=False, special=0)
        r = rrnd()
        vec("mpf_nthroot", [Mf(s), n], lambda: E.mpf_nthroot(s, n, p, r), m, p, r)
    unary("mpf_cbrt", E.mpf_cbrt, gen_x(-500, 500, 60, edges + [M.from_int(27), M.from_int(-8)]))

    # atan / atan2
    xs = gen_x(-3100, 3100, 80, edges)
    xs += gen_x(-5, 5, 60)
    xs += [near(M.fone) for _ in range(10)]
    unary("mpf_atan", E.mpf_atan, xs, 0.3)
    pairs = [(a, b) for a in SPECIALS + [M.fone, M.fnone] for b in SPECIALS + [M.fone, M.fnone]]
    for _ in range(80):
        pairs.append((rmpf(-50, 50, special=0.1), rmpf(-50, 50, special=0.1)))
    for y, x in pairs:
        p, r = rprec(), rrnd()
        vec("mpf_atan2", [Mf(y), Mf(x)], lambda: E.mpf_atan2(y, x, p, r), m, p, r)

    # asin / acos / asinh / acosh / atanh
    unit = gen_x(-3000, 0, 70, edges + [near(M.fone, True), near(M.fnone, True), near(M.fone, False)])
    unit += [near(M.fone, True) for _ in range(10)] + [M.mpf_neg(near(M.fone, True)) for _ in range(5)]
    unary("mpf_asin", E.mpf_asin, unit)
    unary("mpf_acos", E.mpf_acos, unit)
    unary("mpf_atanh", E.mpf_atanh, unit + gen_x(1, 5, 10))
    unary("mpf_asinh", E.mpf_asinh, gen_x(-3000, 3000, 80, edges))
    unary("mpf_acosh", E.mpf_acosh, gen_x(0, 3000, 50, edges) + [near(M.fone, False) for _ in range(15)]
          + gen_x(-3, 0, 10))

    # cos / sin / tan, with reductions near multiples of pi/2
    xs = gen_x(-3000, 3000, 70, edges)
    xs += gen_x(-10, 12, 70)
    xs += [near_pi_multiple() for _ in range(60)]
    for x in xs:
        p, r = rprec(), rrnd()
        vec("mpf_cos_sin", [Mf(x)], lambda: E.mpf_cos_sin(x, p, r), lambda t: T(Mf(t[0]), Mf(t[1])), p, r)
    unary("mpf_cos", E.mpf_cos, xs[::2])
    unary("mpf_sin", E.mpf_sin, xs[1::2])
    unary("mpf_tan", E.mpf_tan, xs[::2] + [near_pi_multiple() for _ in range(20)])

    # cos/sin of pi*x: integers, half-integers, quarters, near them
    xs = gen_x(-3000, 3000, 30, edges)
    xs += gen_x(-10, 12, 40)
    xs += [M.from_man_exp(rng.randint(-1000, 1000), -rng.choice([0, 1, 2, 3])) for _ in range(40)]
    xs += [M.mpf_add(M.from_man_exp(rng.randint(-100, 100), -1), tiny(), 2000) for _ in range(30)]
    for x in xs:
        p, r = rprec(), rrnd()
        vec("mpf_cos_sin_pi", [Mf(x)], lambda: E.mpf_cos_sin_pi(x, p, r),
            lambda t: T(Mf(t[0]), Mf(t[1])), p, r)
    unary("mpf_cos_pi", E.mpf_cos_pi, xs[::2])
    unary("mpf_sin_pi", E.mpf_sin_pi, xs[1::2])
    for x in xs[::2]:
        p, r = rprec(), rrnd()
        vec("mpf_tan_pi", [Mf(x)], lambda: E.mpf_cos_sin(x, p, r, 3, True), m, p, r)

    # log_hypot
    pairs = [(a, b) for a in SPECIALS + [M.fone] for b in SPECIALS + [M.fone, M.ftwo]]
    for _ in range(80):
        pairs.append((rmpf(-100, 100, special=0.1), rmpf(-100, 100, special=0.1)))
    for _ in range(15):
        pairs.append((near(M.fone), rmpf(-300, -20)))
    for a, b in pairs:
        p, r = rprec(), rrnd()
        vec("mpf_log_hypot", [Mf(a), Mf(b)], lambda: E.mpf_log_hypot(a, b, p, r), m, p, r)

    # fibonacci
    xs = [M.from_int(n) for n in [0, 1, 2, 3, -1, -2, -7, 30, 100, 1000, 1023, 1024, 5000, -5000]]
    xs += [rint_mpf(0, 3000) for _ in range(30)]
    xs += gen_x(-60, 12, 50, SPECIALS)
    xs += [M.from_man_exp(rng.randint(-200, 200), -1) for _ in range(10)]
    unary("mpf_fibonacci", E.mpf_fibonacci, xs)

    # constants
    for name in ["mpf_pi", "mpf_e", "mpf_ln2", "mpf_ln10", "mpf_phi", "mpf_degree", "mpf_sqrtpi", "mpf_ln_sqrt2pi"]:
        f = getattr(E, name)
        for p in [1, 2, 10, 53, 100, 333, 1000, 3000]:
            for r in RNDS:
                vec(name, [], lambda: f(p, r), m, p, r)
    for name in ["pi_fixed", "e_fixed", "ln2_fixed", "ln10_fixed", "phi_fixed", "degree_fixed",
                 "sqrtpi_fixed", "ln_sqrt2pi_fixed"]:
        f = getattr(E, name)
        for p in [1, 5, 53, 64, 200, 1001, 3000]:
            vec(name, [p], lambda: f(p), I)


def kernel_vectors():
    """The fixed-point kernels, on the inputs their callers give them."""
    def fixed(x, prec):
        return M.to_fixed(x, prec)

    # int_pow_fixed(y, n, prec): y ~ 2^prec
    for _ in range(60):
        prec = rng.choice([10, 50, 53, 64, 100, 200, 500, 1000])
        y = (1 << prec) + rng.getrandbits(prec) - (1 << (prec - 1))
        n = rng.choice([2, 3, 4, 5, 7, 10, 19, 20, 33])
        vec("int_pow_fixed", [I(y), n, prec], lambda: E.int_pow_fixed(y, n, prec), lambda t: T(I(t[0]), I(t[1])))

    # nthroot_fixed(y, n, prec, exp1): what mpf_nthroot passes
    for _ in range(50):
        n = rng.choice([2, 3, 4, 5, 7, 10, 15, 20])
        prec = rng.choice([53, 100, 200, 500, 1000]) // n * n
        y = rng.getrandbits(prec + rng.randint(-n, n)) | 1
        vec("nthroot_fixed", [I(y), n, prec, 0], lambda: E.nthroot_fixed(y, n, prec, 0), I)

    # log_int_fixed(n, prec), each from an empty cache
    for _ in range(40):
        n = rng.choice([2, 3, 7, 10, 1999, 2000, 2001, rng.randint(2, 10**30)])
        prec = rng.choice([1, 5, 9, 10, 53, 100, 500, 3000])
        vec("log_int_fixed", [I(n), prec], lambda: E.log_int_fixed(n, prec), I)

    # agm_fixed(a, b, prec), log_agm(x, prec)
    for _ in range(30):
        prec = rng.choice([53, 100, 500, 1000, 3000])
        a = (1 << prec) + rng.getrandbits(prec) // 4
        b = (1 << prec) - rng.getrandbits(prec) // 3
        vec("agm_fixed", [I(a), I(b), prec], lambda: E.agm_fixed(a, b, prec), I)
    for _ in range(30):
        prec = rng.choice([53, 100, 500, 1000, 3000])
        x = rng.getrandbits(prec - rng.randint(1, prec // 2)) | 1
        vec("log_agm", [I(x), prec], lambda: E.log_agm(x, prec), I)

    # log_taylor(x, prec, r), log_taylor_cached(x, prec): x in (1/2, 2)
    for _ in range(40):
        prec = rng.choice([20, 53, 100, 500, 1000, 2500])
        x = (1 << prec) + rng.getrandbits(prec) - (1 << (prec - 1))
        r = rng.choice([0, 1, 3, 8])
        vec("log_taylor", [I(x), prec, r], lambda: E.log_taylor(x, prec, r), I)
        vec("log_taylor_cached", [I(x), prec], lambda: E.log_taylor_cached(x, prec), I)

    # atan_newton(x, prec), atan_taylor(x, prec): x in [0, 1)
    for _ in range(40):
        prec = rng.choice([20, 53, 99, 100, 500, 1000, 2999])
        x = rng.getrandbits(prec)
        vec("atan_newton", [I(x), prec], lambda: E.atan_newton(x, prec), I)
        vec("atan_taylor", [I(x), prec], lambda: E.atan_taylor(x, prec), I)

    # exponential_series(x, prec, type), exp_basecase, exp_expneg_basecase,
    # cos_sin_basecase (x in [0, pi/2)), cos_sin_fixed, exp_fixed
    for _ in range(60):
        prec = rng.choice([20, 53, 100, 400, 401, 600, 601, 1000, 1499, 1500, 3000])
        x = rng.getrandbits(prec - rng.randint(0, 20))
        if rng.random() < 0.3:
            x = -x
        typ = rng.choice([0, 1, 2])
        vec("exponential_series", [I(x), prec, typ], lambda: E.exponential_series(x, prec, typ),
            (lambda t: T(I(t[0]), I(t[1]))) if typ else I)
        vec("exp_basecase", [I(x), prec], lambda: E.exp_basecase(x, prec), I)
        vec("exp_expneg_basecase", [I(x), prec], lambda: E.exp_expneg_basecase(x, prec),
            lambda t: T(I(t[0]), I(t[1])))
        xa = abs(x) % ((E.pi_fixed(prec) >> 1) or 1)
        vec("cos_sin_basecase", [I(xa), prec], lambda: E.cos_sin_basecase(xa, prec), lambda t: T(I(t[0]), I(t[1])))
        xb = x * rng.choice([1, 3, 1000])
        vec("cos_sin_fixed", [I(xb), prec], lambda: E.cos_sin_fixed(xb, prec), lambda t: T(I(t[0]), I(t[1])))
        vec("exp_fixed", [I(xb), prec], lambda: E.exp_fixed(xb, prec), I)

    # mod_pi2(man, exp, mag, wp)
    for _ in range(40):
        x = near_pi_multiple() if rng.random() < 0.5 else rmpf(-5, 3000, special=0)
        s, man, exp, bc = x
        wp = rng.choice([20, 63, 110, 300, 1010])
        vec("mod_pi2", [I(man), exp, exp + bc, wp], lambda: E.mod_pi2(man, exp, exp + bc, wp),
            lambda t: T(I(t[0]), I(t[1]), I(t[2])))

    # ifib
    for n in [0, 1, 2, 3, 10, 50, 64, 93, 94, 1000, 4097, -1, -2, -3, -50, -51, 10**5]:
        vec("ifib", [I(n)], lambda: LI.ifib(n), I)


# ---------------------------------------------------------------------------
# libmpc
# ---------------------------------------------------------------------------


def mpc_vectors():
    c = Cz
    m = Mf

    def unary(name, f, zs, enc=c, big=0.15):
        for z in zs:
            p, r = rprec(big), rrnd()
            vec(name, [Cz(z)], lambda: f(z, p, r), enc, p, r)

    def zgen(lo, hi, n):
        zs = cut_points()
        for _ in range(n):
            for q in quadrants(rmpc(lo, hi, special=0)):
                zs.append(q)
            zs.append(rmpc(lo, hi))
        return zs

    zs_small = zgen(-8, 8, 25)
    zs_wide = zgen(-300, 40, 12)

    # predicates and conversions
    for z in zs_small[:60] + [(M.finf, M.fzero), (M.fzero, M.fnan), (M.fninf, M.fninf)]:
        vec("mpc_is_inf", [Cz(z)], lambda: C.mpc_is_inf(z), bool)
        vec("mpc_is_infnan", [Cz(z)], lambda: C.mpc_is_infnan(z), bool)
        vec("mpc_is_nonzero", [Cz(z)], lambda: C.mpc_is_nonzero(z), bool)
        vec("mpc_hash", [Cz(z)], lambda: C.mpc_hash(z), I)
        dps = rng.choice([1, 5, 15, 30])
        vec("mpc_to_str", [Cz(z), dps], lambda: C.mpc_to_str(z, dps), lambda s: {"s": s})
        vec("mpc_to_complex", [Cz(z)], lambda: C.mpc_to_complex(z), lambda w: T(F(w.real), F(w.imag)))

    # arithmetic
    for _ in range(140):
        z, w = rng.choice(zs_small + zs_wide), rng.choice(zs_small + zs_wide)
        x = rmpf(-20, 20, special=0.1)
        n = rng.choice([0, 1, -1, 2, 3, -5, 10**20 + 7])
        p, r = rprec(), rrnd()
        p0 = rng.choice([0, p])
        A = [Cz(z), Cz(w)]
        vec("mpc_add", A, lambda: C.mpc_add(z, w, p, r), c, p, r)
        vec("mpc_sub", A, lambda: C.mpc_sub(z, w, p0, r), c, p0, r)
        vec("mpc_mul", A, lambda: C.mpc_mul(z, w, p, r), c, p, r)
        vec("mpc_div", A, lambda: C.mpc_div(z, w, p, r), c, p, r)
        vec("mpc_add_mpf", [Cz(z), Mf(x)], lambda: C.mpc_add_mpf(z, x, p, r), c, p, r)
        vec("mpc_sub_mpf", [Cz(z), Mf(x)], lambda: C.mpc_sub_mpf(z, x, p0, r), c, p0, r)
        vec("mpc_mul_mpf", [Cz(z), Mf(x)], lambda: C.mpc_mul_mpf(z, x, p, r), c, p, r)
        vec("mpc_mul_imag_mpf", [Cz(z), Mf(x)], lambda: C.mpc_mul_imag_mpf(z, x, p, r), c, p, r)
        vec("mpc_div_mpf", [Cz(z), Mf(x)], lambda: C.mpc_div_mpf(z, x, p, r), c, p, r)
        vec("mpc_mpf_div", [Mf(x), Cz(z)], lambda: C.mpc_mpf_div(x, z, p, r), c, p, r)
        vec("mpc_mul_int", [Cz(z), I(n)], lambda: C.mpc_mul_int(z, n, p, r), c, p, r)
        vec("mpc_pos", [Cz(z)], lambda: C.mpc_pos(z, p, r), c, p, r)
        vec("mpc_neg", [Cz(z)], lambda: C.mpc_neg(z, p0, r), c, p0, r)
        vec("mpc_conjugate", [Cz(z)], lambda: C.mpc_conjugate(z, p, r), c, p, r)
        k = rng.randint(-100, 100)
        vec("mpc_shift", [Cz(z), k], lambda: C.mpc_shift(z, k), c)
        vec("mpc_square", [Cz(z)], lambda: C.mpc_square(z, p, r), c, p, r)
        vec("mpc_reciprocal", [Cz(z)], lambda: C.mpc_reciprocal(z, p, r), c, p, r)
        vec("mpc_abs", [Cz(z)], lambda: C.mpc_abs(z, p, r), m, p, r)
        vec("mpc_arg", [Cz(z)], lambda: C.mpc_arg(z, p, r), m, p, r)
    for name in ["mpc_floor", "mpc_ceil", "mpc_nint", "mpc_frac"]:
        unary(name, getattr(C, name), rng.sample(zs_small + zs_wide, 40))

    # powers
    for _ in range(50):
        a, b = rng.randint(-10**6, 10**6), rng.randint(-10**6, 10**6)
        n = rng.choice([0, 1, 2, 3, 7, 30, 101])
        vec("complex_int_pow", [I(a), I(b), I(n)], lambda: C.complex_int_pow(a, b, n), lambda t: T(I(t[0]), I(t[1])))
    for _ in range(160):
        z = rng.choice(zs_small)
        n = rng.choice([0, 1, -1, 2, -2, 3, 4, 5, 7, 13, -13, 100, 1000, 3000, -3000])
        p, r = rprec(), rrnd()
        vec("mpc_pow_int", [Cz(z), I(n)], lambda: C.mpc_pow_int(z, n, p, r), c, p, r)
        t = rng.choice([rint_mpf(0, 20), M.from_man_exp(rng.choice([1, -1, 3, -7]), -1), rmpf(-6, 6, special=0.05)])
        vec("mpc_pow_mpf", [Cz(z), Mf(t)], lambda: C.mpc_pow_mpf(z, t, p, r), c, p, r)
        w = rng.choice(zs_small) if rng.random() < 0.7 else (t, M.fzero)
        if w[0] in SPECIALS[1:] or w[1] in SPECIALS[1:] or M.mpf_cmp(C.mpc_abs(w, 10), M.from_int(100)) > 0:
            w = (t, rmpf(-6, 6, special=0))
        vec("mpc_pow", [Cz(z), Cz(w)], lambda: C.mpc_pow(z, w, p, r), c, p, r)

    # roots
    unary("mpc_sqrt", C.mpc_sqrt, zs_small + zs_wide)
    for _ in range(140):
        z = rng.choice(zs_small + zs_wide)
        n = rng.choice([0, 1, -1, 2, 3, -3, 4, 5, 7, 10, 19, 20, 21, 50, -50])
        p, r = rprec(), rrnd()
        vec("mpc_nthroot", [Cz(z), n], lambda: C.mpc_nthroot(z, n, p, r), c, p, r)
    unary("mpc_cbrt", C.mpc_cbrt, rng.sample(zs_small + zs_wide, 60))
    for _ in range(40):
        n = rng.choice([2, 3, 5, 7, 10, 20])
        prec = rng.choice([53, 100, 300, 1000])
        a = rng.getrandbits(prec + rng.randint(-40, 40)) * rng.choice([1, -1])
        b = rng.getrandbits(prec + rng.randint(-40, 40)) * rng.choice([1, -1])
        vec("mpc_nthroot_fixed", [I(a), I(b), n, prec], lambda: C.mpc_nthroot_fixed(a, b, n, prec),
            lambda t: T(I(t[0]), I(t[1])))
    # the OverflowError path: the float estimate overflows
    for z in [(M.from_int(2**40), M.from_int(3)), (M.from_int(-(2**35)), M.from_int(2**36))]:
        for n in [19, 20]:
            for p in [53, 100]:
                vec("mpc_nthroot", [Cz(z), n], lambda: C.mpc_nthroot(z, n, p, "n"), c, p, "n")

    # exp, log, trig
    zs_exp = zgen(-300, 6, 10) + [(rmpf(-3000, 6), rmpf(-3000, 3000)) for _ in range(20)]
    unary("mpc_exp", C.mpc_exp, zs_exp)
    unary("mpc_log", C.mpc_log, zs_small + zs_wide + [(near(M.fone), tiny()) for _ in range(10)])
    zs_trig = zgen(-200, 6, 12) + [(near_pi_multiple(), rmpf(-50, 5)) for _ in range(15)]
    for name in ["mpc_cos", "mpc_sin", "mpc_tan", "mpc_cos_pi", "mpc_sin_pi", "mpc_cosh", "mpc_sinh",
                 "mpc_tanh", "mpc_expj", "mpc_expjpi"]:
        unary(name, getattr(C, name), rng.sample(zs_trig, 70))
    for name in ["mpc_cos_sin", "mpc_cos_sin_pi"]:
        unary(name, getattr(C, name), rng.sample(zs_trig, 60), lambda t: T(Cz(t[0]), Cz(t[1])))

    # inverse functions, on and around their branch cuts
    zs_inv = zgen(-60, 60, 10) + zgen(-5, 5, 15)
    for name in ["mpc_atan", "mpc_acos", "mpc_asin", "mpc_asinh", "mpc_acosh", "mpc_atanh"]:
        unary(name, getattr(C, name), zs_inv, big=0.1)
    unary("mpc_fibonacci", C.mpc_fibonacci, zgen(-6, 5, 6))
    for x in [M.fone, rmpf(-5, 5), M.fzero, M.fnan]:
        for name in ["mpf_expj", "mpf_expjpi"]:
            vec(name, [Mf(x)], lambda: getattr(C, name)(x, 53, "n"), m, 53, "n")


# ---------------------------------------------------------------------------
# libhyper: AGM
# ---------------------------------------------------------------------------


def agm_vectors():
    m, c = Mf, Cz
    pairs = [(a, b) for a in SPECIALS + [M.fone, M.fnone] for b in SPECIALS + [M.fone, M.ftwo]]
    for _ in range(70):
        pairs.append((rmpf(-100, 100, sign=False, special=0.05), rmpf(-100, 100, sign=False, special=0.05)))
    for _ in range(15):
        pairs.append((rmpf(-10, 10), rmpf(-10, 10)))
    for a, b in pairs:
        p, r = rprec(), rrnd()
        vec("mpf_agm", [Mf(a), Mf(b)], lambda: H.mpf_agm(a, b, p, r), m, p, r)
        vec("mpf_agm1", [Mf(a)], lambda: H.mpf_agm1(a, p, r), m, p, r)
    zs = cut_points() + [rmpc(-10, 10) for _ in range(40)]
    for _ in range(80):
        a, b = rng.choice(zs), rng.choice(zs)
        if rng.random() < 0.1:
            b = C.mpc_neg(a)
        p, r = rprec(0.1), rrnd()
        if mpc_agm_stops(a, b, p):
            vec("mpc_agm", [Cz(a), Cz(b)], lambda: H.mpc_agm(a, b, p, r), c, p, r)
        if mpc_agm_stops(C.mpc_one, a, p):
            vec("mpc_agm1", [Cz(a)], lambda: H.mpc_agm1(a, p, r), c, p, r)


def mpc_agm_stops(a, b, prec, cap=5000):
    """Whether mpc_agm(a, b, prec) returns: its loop, copied from libhyper,
    with an iteration cap. For some inputs (agm(a, a) with a < 0, or
    agm(-2^-100+i, 2^-300+i)) mpmath iterates forever."""
    if C.mpc_is_infnan(a) or C.mpc_is_infnan(b) or C.mpc_zero in (a, b) or C.mpc_neg(a) == b:
        return True
    wp = prec + 20
    eps = M.mpf_shift(M.fone, -wp + 10)
    for _ in range(cap):
        a, b = C.mpc_shift(C.mpc_add(a, b, wp), -1), C.mpc_sqrt(C.mpc_mul(a, b, wp), wp)
        size = H.mpf_min_max([C.mpc_abs(a, 10), C.mpc_abs(b, 10)])[1]
        err = C.mpc_abs(C.mpc_sub(a, b, 10), 10)
        if size == M.fzero or M.mpf_lt(err, M.mpf_mul(eps, size)):
            return True
    return False


# ---------------------------------------------------------------------------
# The context
# ---------------------------------------------------------------------------


def ctx_vec(name, args, f, enc, prec=53, rnd="n", tc=False):
    """Run f() under mp.prec = prec, rounding rnd, trap_complex tc."""
    def g():
        mp.prec = prec
        mp._prec_rounding[1] = rnd
        mp.trap_complex = tc
        return f()

    extra = {"tc": True} if tc else {}
    vec(name, args, g, enc, prec, rnd, **extra)


def mkv(x):
    if isinstance(x, tuple) and len(x) == 2 and isinstance(x[0], tuple):
        return mp.make_mpc(x)
    return mp.make_mpf(x)


def rvalue(lo=-10, hi=10):
    if rng.random() < 0.5:
        return mkv(rmpf(lo, hi, special=0.06))
    return mkv(rmpc(lo, hi))


def ctx_prec():
    return rng.choice([1, 2, 10, 24, 53, 53, 53, 64, 100, 113, 200, 333, 1000, 2000, 3000])


def ctx_vectors():
    v = V

    def vargs(*xs):
        return [V(x) for x in xs]

    # the wrapped libmp functions, real-to-complex fallback included
    fallback = {
        "sqrt": [-2, -1, M.from_float(-0.25)], "cbrt": [-8, -2], "ln": [-1, -10, M.from_float(-0.5)],
        "asin": [2, -3, M.from_float(1.5)], "acos": [2, -3], "acosh": [0, M.from_float(0.5), -2],
        "atanh": [2, -2, M.from_float(1.5)], "expj": [1, 0, -2], "expjpi": [M.fhalf, 1, M.from_float(0.25)],
        "agm1": [-1, -3], "tan": [], "exp": [], "atan": [], "sin": [], "cos": [], "sinh": [], "cosh": [],
        "tanh": [], "asinh": [], "sinpi": [], "cospi": [], "floor": [], "ceil": [], "nint": [], "frac": [],
        "fib": [],
    }
    for name, extra in fallback.items():
        f = getattr(mp, name)
        xs = [mkv(M.from_int(e) if isinstance(e, int) else e) for e in extra]
        for _ in range(14):
            xs.append(rvalue(-6, 6))
        xs += [mkv(s) for s in SPECIALS]
        for x in xs:
            p, r = ctx_prec(), rng.choice("nnnfcud")
            ctx_vec("ctx." + name, vargs(x), lambda: f(x), v, p, r)
        for x in xs[:len(extra)] + xs[-6:-4]:
            ctx_vec("ctx." + name, vargs(x), lambda: f(x), v, 53, "n", tc=True)
        # keyword precision: f(x, prec=..., rounding=...)
        for x in xs[:3]:
            p2, r2 = ctx_prec(), rrnd()
            ctx_vec("ctx." + name + "_kw", vargs(x) + [p2, r2], lambda: f(x, prec=p2, rounding=r2), v, 53, "n")

    # operators
    ints = [0, 1, -1, 2, 3, -7, 10**30]
    for _ in range(120):
        x, y = rvalue(-20, 20), rvalue(-20, 20)
        if rng.random() < 0.1:
            y = x
        p, r = ctx_prec(), rng.choice("nnfcud")
        for op in ["add", "sub", "mul", "div", "pow", "fmod"]:
            fn = {"add": lambda: x + y, "sub": lambda: x - y, "mul": lambda: x * y, "div": lambda: x / y,
                  "pow": lambda: x ** y, "fmod": lambda: mp.fmod(x, y)}[op]
            ctx_vec("ctx." + op, vargs(x, y), fn, v, p, r)
        for op in ["eq", "lt", "le", "gt", "ge"]:
            fn = {"eq": lambda: x == y, "lt": lambda: x < y, "le": lambda: x <= y, "gt": lambda: x > y,
                  "ge": lambda: x >= y}[op]
            ctx_vec("ctx." + op, vargs(x, y), fn, bool, p, r)
        n = rng.choice(ints)
        ctx_vec("ctx.mul_int", vargs(x) + [I(n)], lambda: x * n, v, p, r)
        ctx_vec("ctx.rdiv_int", [I(n)] + vargs(x), lambda: n / x, v, p, r)
        ctx_vec("ctx.pow_int", vargs(x) + [I(n % 50 - 20)], lambda: x ** (n % 50 - 20), v, p, r)
        ctx_vec("ctx.neg", vargs(x), lambda: -x, v, p, r)
        ctx_vec("ctx.pos", vargs(x), lambda: +x, v, p, r)
        ctx_vec("ctx.abs", vargs(x), lambda: abs(x), v, p, r)
        ctx_vec("ctx.truthy", vargs(x), lambda: bool(x), bool, p, r)
    for x in [mkv(M.from_int(-2)), mkv(M.from_float(-0.5))]:
        for y in [mkv(M.fhalf), mkv(M.from_float(1.5)), mkv(M.from_float(0.3))]:
            ctx_vec("ctx.pow", vargs(x, y), lambda: x ** y, v)
            ctx_vec("ctx.pow", vargs(x, y), lambda: x ** y, v, tc=True)

    # functions.py
    zs = [rvalue(-6, 6) for _ in range(40)] + [mkv(s) for s in SPECIALS]
    zs += [mkv(M.fzero), mkv(C.mpc_zero), mkv((M.fzero, M.fone)), mkv(M.fone), mkv(M.fnone)]
    for name in ["cot", "sec", "csc", "coth", "sech", "csch", "acot", "asec", "acsc", "acoth", "asech",
                 "acsch", "sinc", "sincpi", "sign", "arg", "fabs", "re", "im", "conj", "log10",
                 "degrees", "radians", "isnan", "isinf", "isfinite", "isnormal", "isnpint", "mag",
                 "nint_distance", "frexp", "cos_sin", "cospi_sinpi"]:
        f = getattr(mp, name)
        enc = {"isnan": bool, "isinf": bool, "isfinite": bool, "isnormal": bool, "isnpint": bool,
               "mag": lambda t: I(t) if isinstance(t, int) else V(t),
               "nint_distance": lambda t: T(I(t[0]), I(t[1]) if isinstance(t[1], int) else V(t[1])),
               "frexp": lambda t: T(V(t[0]), I(t[1])),
               "cos_sin": lambda t: T(V(t[0]), V(t[1])),
               "cospi_sinpi": lambda t: T(V(t[0]), V(t[1]))}.get(name, v)
        for x in zs:
            p, r = ctx_prec(), rng.choice("nnnfcud")
            ctx_vec("ctx." + name, vargs(x), lambda: f(x), enc, p, r)
    for x in zs + [mkv(M.from_int(7)), mkv((M.from_int(3), M.fzero)), mkv((M.from_int(3), M.fone))]:
        for g in [False, True]:
            ctx_vec("ctx.isint", vargs(x) + [g], lambda: mp.isint(x, g), bool)
    # nint_distance near integers and half-integers
    for _ in range(30):
        x = M.mpf_add(M.from_man_exp(rng.randint(-1000, 1000), -rng.choice([0, 1])), tiny(), 3000)
        x = mkv(x) if rng.random() < 0.6 else mkv((x, rng.choice([M.fzero, tiny()])))
        ctx_vec("ctx.nint_distance", vargs(x), lambda: mp.nint_distance(x),
                lambda t: T(I(t[0]), I(t[1]) if isinstance(t[1], int) else V(t[1])))

    # two-argument functions
    for _ in range(60):
        x, y = rvalue(-8, 8), rvalue(-8, 8)
        p, r = ctx_prec(), rng.choice("nnfcud")
        ctx_vec("ctx.log", vargs(x, y), lambda: mp.log(x, y), v, p, r)
        ctx_vec("ctx.log", vargs(x) + [None], lambda: mp.log(x), v, p, r)
        ctx_vec("ctx.power", vargs(x, y), lambda: mp.power(x, y), v, p, r)
        ctx_vec("ctx.hypot", vargs(x, y), lambda: mp.hypot(x, y), v, p, r)
        ctx_vec("ctx.atan2", vargs(x, y), lambda: mp.atan2(x, y), v, p, r)
        ctx_vec("ctx.agm", vargs(x, y), lambda: mp.agm(x, y), v, p, r)
        ctx_vec("ctx.agm", vargs(x) + [None], lambda: mp.agm(x), v, p, r)
        k = rng.randint(-60, 60)
        ctx_vec("ctx.ldexp", vargs(x) + [k], lambda: mp.ldexp(x, k), v, p, r)
    for x in [mkv(M.from_int(-8)), mkv(M.from_int(27)), mkv(M.from_int(-2)), rvalue(), rvalue(), rvalue()]:
        for n in [1, 2, 3, 4, 5, -3, 7]:
            for k in [0, 1, 2, 3, -1, 5]:
                p, r = ctx_prec(), rng.choice("nnfcud")
                ctx_vec("ctx.root", vargs(x) + [n, k], lambda: mp.root(x, n, k), v, p, r)
    for x in [mkv(M.fone), mkv(M.ftwo)]:
        for b in [mkv(M.fone), mkv((M.fone, M.fzero))]:
            ctx_vec("ctx.agm", vargs(x, b), lambda: mp.agm(x, b), v)

    # constants and constructors
    for name in ["pi", "e", "ln2", "ln10", "phi", "degree", "eps"]:
        for p in [1, 10, 53, 200]:
            for r in "nfcud":
                ctx_vec("ctx.const", [{"s": name}], lambda: +getattr(mp, name), v, p, r)
    nums = [(I(5), 5), (I(-(10**40)), -(10**40)), (F(0.1), 0.1), (F(-2.5), -2.5), (T(F(1.5), F(-0.25)), 1.5 - 0.25j),
            ({"q": ["1", "3"]}, mpq(1, 3)), ({"q": ["-7", "4"]}, mpq(-7, 4)), ({"s": "1.25"}, "1.25"),
            ({"s": "0.1"}, "0.1"), ({"s": "-3e-400"}, "-3e-400"), ({"s": "inf"}, "inf"), ({"s": "zz"}, "zz"),
            ({"s": "2/3"}, "2/3")]
    for x in [rvalue(-100, 100) for _ in range(10)]:
        nums.append((V(x), x))
    for enc_x, x in nums:
        for p, r in [(53, "n"), (10, "f"), (100, "c"), (4, "u"), (2, "d")]:
            ctx_vec("ctx.convert", [enc_x], lambda: mp.convert(x), v, p, r)
            ctx_vec("ctx.mpf", [enc_x], lambda: mp.mpf(x), v, p, r)
            ctx_vec("ctx.mpc", [enc_x, I(3)], lambda: mp.mpc(x, 3), v, p, r)
            ctx_vec("ctx.convert_param", [enc_x], lambda: mp._convert_param(x), lambda t: P(t[0], t[1]), p, r)
    for x in [mkv(M.from_man_exp(3, -4)), mkv(M.from_man_exp(-5, -5)), mkv(M.from_man_exp(7, 3)),
              mkv((M.from_man_exp(3, -2), M.fzero)), mkv((M.finf, M.fzero)), mkv(M.fnan)]:
        ctx_vec("ctx.convert_param", [V(x)], lambda: mp._convert_param(x), lambda t: P(t[0], t[1]))

    # precision managers
    for n in [1, 5, 53, 100, 333]:
        for fname in ["exp", "sqrt", "ln"]:
            x = mkv(M.from_float(2.5))
            f = getattr(mp, fname)

            def wp():
                with mp.workprec(n):
                    y = f(x)
                return T(V(y), I(mp.prec), I(mp.dps))

            def ep():
                with mp.extraprec(n):
                    y = f(x)
                return T(V(y), I(mp.prec), I(mp.dps))

            def wd():
                with mp.workdps(n):
                    y = f(x)
                return T(V(y), I(mp.prec), I(mp.dps))

            def ed():
                with mp.extradps(n):
                    y = f(x)
                return T(V(y), I(mp.prec), I(mp.dps))

            for nm, g in [("workprec", wp), ("extraprec", ep), ("workdps", wd), ("extradps", ed)]:
                p = rng.choice([10, 53, 70])
                ctx_vec("ctx." + nm, [n, {"s": fname}] + vargs(x), g, lambda t: t, p, "n")
    for n in [0, 1, 2, 3, 4, 15, 16, 17, 53, 54, 100]:
        def sp_():
            mp.prec = n
            return T(I(mp.prec), I(mp.dps))

        def sd_():
            mp.dps = n
            return T(I(mp.prec), I(mp.dps))

        ctx_vec("ctx.set_prec", [n], sp_, lambda t: t)
        ctx_vec("ctx.set_dps", [n], sd_, lambda t: t)


def hypsum_vectors():
    """ctx.hypsum on the parameters _convert_param makes."""
    def conv(x):
        return mp._convert_param(x)

    def mk(x):
        if isinstance(x, tuple) and len(x) == 2 and not isinstance(x[0], tuple):
            return x  # (p, q) rational
        if isinstance(x, (int,)):
            return x
        return mkv(x)

    specs = []
    for _ in range(90):
        p = rng.choice([0, 1, 1, 2, 2, 3])
        q = rng.choice([0, 1, 1, 2]) if p <= 1 else rng.choice([1, 2, 3])
        params = []
        for i in range(p + q):
            r = rng.random()
            if r < 0.25:
                params.append(rng.randint(-6, 8) if i < p else rng.randint(1, 8))
            elif r < 0.45:
                params.append((rng.randint(-20, 20), rng.choice([2, 3, 4, 7])))
            elif r < 0.75:
                params.append(rmpf(-6, 4, special=0))
            else:
                params.append((rmpf(-4, 3, special=0), rmpf(-4, 3, special=0)))
        if p == q + 1:
            z = rmpf(-8, -1, special=0) if rng.random() < 0.6 else (rmpf(-8, -1, special=0), rmpf(-8, -1, special=0))
        elif p > q + 1:
            params[0] = -rng.randint(0, 6)
            z = rmpf(-3, 3, special=0)
        else:
            z = rmpf(-6, 5, special=0) if rng.random() < 0.6 else (rmpf(-6, 4, special=0), rmpf(-6, 4, special=0))
        specs.append((p, q, params, z, {}))
    # denominators near nonpositive integers, poles, terminating series
    for b in [M.from_float(-3.0000001), M.mpf_add(M.from_int(-5), M.from_man_exp(1, -80), 200),
              (M.from_float(-2.0000000001), M.from_man_exp(1, -60)), M.from_float(-0.99999)]:
        specs.append((1, 1, [M.fone, b], M.from_float(0.75), {}))
        specs.append((2, 1, [M.fhalf, M.from_float(1.25), b], M.from_float(-0.3), {}))
    for a, b in [(-3, -5), (-3, -2), (-5, -5), (2, -1), (-1, 0)]:
        specs.append((1, 1, [a, b], M.from_float(0.5), {}))
    specs.append((1, 1, [-1, 1], M.fone, {"zeroprec": 100}))
    specs.append((1, 1, [-1, 1], M.fone, {}))
    specs.append((1, 1, [-1, 1], M.fone, {"maxprec": 200}))
    specs.append((2, 1, [1, 1, 2], M.from_float(0.999), {"maxterms": 50}))
    specs.append((1, 1, [M.fhalf, M.from_float(1.5)], M.from_int(-30), {"accurate_small": False}))
    specs.append((0, 1, [M.from_float(2.5)], M.from_float(-40.0), {}))
    # 0F1(;1;-z) = J0(2 sqrt(z)) next to its first zero: the result's
    # smallness makes hypsum double its working precision, here against
    # maxprec on both sides of the steps 50, 105, 215, 435
    with mp.workprec(400):
        z0 = -(mp.besseljzero(0, 1) / 2) ** 2
    for bits in [30, 60, 100, 190]:
        z = M.mpf_pos(z0._mpf_, bits, "n")
        for kw in [{}, {"maxprec": 49}, {"maxprec": 104}, {"maxprec": 105}, {"maxprec": 214},
                   {"maxprec": 215}, {"maxprec": 434}, {"zeroprec": 20}, {"zeroprec": 80}]:
            specs.append((0, 1, [M.fone], z, kw))
            specs.append((0, 1, [(2, 2)], (z, M.fzero), kw))

    for p, q, params, z, kw in specs:
        coeffs, flags = [], []
        for x in params:
            cx, fl = conv(mk(x))
            coeffs.append(cx)
            flags.append(fl)
        if "U" in flags:
            continue
        zv = mkv(z)
        prec = ctx_prec() if rng.random() < 0.8 else rng.choice([2000, 3000])
        enc_params = [P(cx, fl) for cx, fl in zip(coeffs, flags)]
        opts = {k: kw[k] for k in ["accurate_small", "maxprec", "maxterms", "zeroprec"] if k in kw}
        flagstr = "".join(flags)
        ctx_vec("ctx.hypsum", [p, q, enc_params, V(zv), opts],
                lambda: mp.hypsum(p, q, flagstr, coeffs, zv, **kw), V, prec, "n")


# ---------------------------------------------------------------------------
# History: in-order sequences from one reset
# ---------------------------------------------------------------------------


def seq_vectors():
    """Calls whose results depend on what ran before them in mpmath (the
    constant memos, atan_taylor_cache, log_int_cache), run in order."""
    def step(name, args, f, enc, prec=None, rnd=None):
        v = {"f": name, "a": args}
        if prec is not None:
            v["p"] = prec
        if rnd is not None:
            v["r"] = rnd
        v["o"] = run(f, enc)
        return v

    for _ in range(12):
        reset()
        seq = []
        for _ in range(rng.randint(5, 25)):
            r0 = rng.random()
            p, r = rng.choice([10, 53, 100, 300, 1000, 2000, 3000]), rrnd()
            if r0 < 0.15:
                seq.append(step("mpf_pi", [], lambda: E.mpf_pi(p, r), Mf, p, r))
            elif r0 < 0.25:
                n = rng.choice([2, 3, 10, 1999])
                seq.append(step("log_int_fixed", [I(n), p], lambda: E.log_int_fixed(n, p), I))
            elif r0 < 0.4:
                x = rmpf(-5, 5, special=0)
                seq.append(step("mpf_atan", [Mf(x)], lambda: E.mpf_atan(x, p, r), Mf, p, r))
            elif r0 < 0.55:
                x = near_pi_multiple() if rng.random() < 0.5 else rmpf(-3, 200, special=0)
                seq.append(step("mpf_cos_sin", [Mf(x)], lambda: E.mpf_cos_sin(x, p, r),
                                lambda t: T(Mf(t[0]), Mf(t[1])), p, r))
            elif r0 < 0.65:
                x = rmpf(-200, 200, sign=False, special=0)
                seq.append(step("mpf_log", [Mf(x)], lambda: E.mpf_log(x, p, r), Mf, p, r))
            elif r0 < 0.75:
                x = rmpf(-5, 30, special=0)
                seq.append(step("mpf_exp", [Mf(x)], lambda: E.mpf_exp(x, p, r), Mf, p, r))
            elif r0 < 0.85:
                xi = rng.getrandbits(p)
                seq.append(step("atan_taylor", [I(xi), p], lambda: E.atan_taylor(xi, p), I))
            else:
                z = rmpc(-5, 5)
                seq.append(step("mpc_log", [Cz(z)], lambda: C.mpc_log(z, p, r), Cz, p, r))
        SEQUENCES.append(seq)
    reset()


def main():
    elefun_vectors()
    kernel_vectors()
    mpc_vectors()
    agm_vectors()
    ctx_vectors()
    hypsum_vectors()
    seq_vectors()
    reset()
    doc = {"sympy_version": sp.__version__, "mpmath_version": mpmath.__version__,
           "vectors": VECTORS, "sequences": SEQUENCES}
    OUT.write_text(json.dumps(doc, separators=(",", ":")) + "\n")
    print("mpf2", {"vectors": len(VECTORS), "sequences": len(SEQUENCES),
                   "sequence steps": sum(len(s) for s in SEQUENCES)})


if __name__ == "__main__":
    main()
