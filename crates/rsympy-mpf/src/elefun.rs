//! Port of the elementary functions of `mpmath.libmp.libelefun` (the
//! constants are in `constants.rs`): powers and roots, logarithms, the
//! inverse tangent and the functions built on it, exp and the hyperbolic
//! functions, cos/sin with their argument reduction, and the fixed-point
//! kernels underneath, operation for operation as mpmath's pure-Python
//! backend runs them.
//!
//! mpmath keeps four caches here. `cos_sin_cache` and `log_taylor_cache`
//! hold values that depend only on their keys, so they only save time.
//! `atan_taylor_cache` holds values computed through `pi_fixed`, whose last
//! bits depend on the pi memo, and filling it moves that memo;
//! `log_int_cache` keeps a value computed at whatever precision came first.
//! Both change later results, so `clear_constant_cache` clears them with
//! the constant memos.

use std::cell::RefCell;
use std::collections::HashMap;

use rsympy_num::Int;

use crate::constants::{ln2_fixed, mpf_e, mpf_phi, mpf_pi, pi_fixed};
use crate::intmath::{bctable, bitcount, giant_steps, ifib, isqrt_fast, lshift, rshift};
use crate::mpf::*;

// Tuning parameters (the values for BACKEND == 'python').
const EXP_COSH_CUTOFF: i64 = 600;
const EXP_SERIES_U_CUTOFF: i64 = 1500;
const COS_SIN_CACHE_PREC: i64 = 400;
const COS_SIN_CACHE_STEP: i64 = 8;
const MAX_LOG_INT_CACHE: i64 = 2000;
const LOG_TAYLOR_PREC: i64 = 2500;
const LOG_TAYLOR_SHIFT: i64 = 9;
const LOG_AGM_MAG_PREC_RATIO: i64 = 20;
const ATAN_TAYLOR_PREC: i64 = 3000;
const ATAN_TAYLOR_SHIFT: i64 = 7;

thread_local! {
    static COS_SIN_CACHE: RefCell<HashMap<Int, (Int, Int)>> = RefCell::new(HashMap::new());
    static LOG_TAYLOR_CACHE: RefCell<HashMap<(Int, i64), (Int, Int)>> = RefCell::new(HashMap::new());
    static ATAN_TAYLOR_CACHE: RefCell<HashMap<(Int, i64), (Int, Int)>> = RefCell::new(HashMap::new());
    static LOG_INT_CACHE: RefCell<HashMap<Int, (Int, i64)>> = RefCell::new(HashMap::new());
}

/// Clears the caches whose contents change later results (see the module
/// docs); called by `clear_constant_cache`.
pub(crate) fn clear_history_caches() {
    ATAN_TAYLOR_CACHE.with(|c| c.borrow_mut().clear());
    LOG_INT_CACHE.with(|c| c.borrow_mut().clear());
}

/// `cache_prec_steps[prec]`: about the next power of two, plus 20.
fn cache_prec_steps(prec: i64) -> i64 {
    // [22, 22] + [min(2**k, LOG_TAYLOR_PREC) + 20] * 2**(k-1) for k >= 1
    assert!(
        (0..=4096).contains(&prec),
        "cache_prec_steps index {prec} out of range"
    );
    if prec < 2 {
        return 22;
    }
    let k = bitcount(&Int::from(prec - 1));
    (1i64 << k).min(LOG_TAYLOR_PREC) + 20
}

/// Python's `int ** float` (or `float ** float`) for a positive base: the
/// C library's `pow`.
fn pyf_pow(x: f64, y: f64) -> f64 {
    x.powf(y)
}

/// Converts an exponent that Python keeps as an unbounded int.
fn small(n: &Int, what: &str) -> i64 {
    n.as_i64()
        .unwrap_or_else(|| panic!("{what} {n} does not fit in i64"))
}

fn floordiv(a: &Int, b: &Int) -> Int {
    a.floor_div(b)
        .unwrap_or_else(|_| panic!("ZeroDivisionError: integer division by zero"))
}

fn sh(a: &Int, n: i64) -> Int {
    assert!(n >= 0, "negative shift count");
    a >> n as u64
}

fn shl(a: &Int, n: i64) -> Int {
    assert!(n >= 0, "negative shift count");
    a << n as u64
}

fn and3(n: &Int) -> i64 {
    (n & &Int::from(3)).as_i64().expect("n & 3 is small")
}

//----------------------------------------------------------------------
// Powers
//----------------------------------------------------------------------

/// `mpf_pow(s, t, prec, rnd)`: `s**t`; `ComplexResult` if `s` is negative
/// and `t` fractional.
pub fn mpf_pow(s: &Mpf, t: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    if s.sign == 1 && t.exp < 0 {
        return Err(MpfError::ComplexResult(
            "negative number raised to a fractional power",
        ));
    }
    if t.exp >= 0 {
        let mut n = shl(&t.man, t.exp);
        if t.sign == 1 {
            n = -n;
        }
        return mpf_pow_int(s, &n, prec, rnd);
    }
    // s**(n/2) = sqrt(s)**n
    if t.exp == -1 {
        if t.man.is_one() {
            if t.sign == 1 {
                return mpf_div(&FONE, &mpf_sqrt(s, prec + 10, rnd.reciprocal())?, prec, rnd);
            }
            return mpf_sqrt(s, prec, rnd);
        }
        if t.sign == 1 {
            return mpf_pow_int(
                &mpf_sqrt(s, prec + 10, rnd.reciprocal())?,
                &-&t.man,
                prec,
                rnd,
            );
        }
        return mpf_pow_int(&mpf_sqrt(s, prec + 10, rnd)?, &t.man, prec, rnd);
    }
    // General formula: s**t = exp(t*log(s))
    let c = mpf_log(s, prec + 10, rnd)?;
    Ok(mpf_exp(&mpf_mul(t, &c, 0, ROUND_FAST), prec, rnd))
}

/// `int_pow_fixed(y, n, prec)`: `(man, exp)` with `man * 2**exp ~= y**n`.
pub fn int_pow_fixed(y: &Int, n: i64, prec: i64) -> (Int, i64) {
    if n == 2 {
        return (y * y, 0);
    }
    let mut y = y.clone();
    let mut n = n;
    let mut bc = bitcount(&y);
    let mut exp = 0i64;
    let workprec = 2 * (prec + 4 * bitcount(&Int::from(n)) + 4);
    let (mut pm, mut pe, mut pbc) = (Int::ONE, 0i64, 1i64);
    loop {
        if n & 1 == 1 {
            pm = &pm * &y;
            pe += exp;
            pbc += bc - 2;
            pbc += bctable(&sh(&pm, pbc));
            if pbc > workprec {
                pm = sh(&pm, pbc - workprec);
                pe += pbc - workprec;
                pbc = workprec;
            }
            n -= 1;
            if n == 0 {
                break;
            }
        }
        y = &y * &y;
        exp += exp;
        bc = bc + bc - 2;
        bc += bctable(&sh(&y, bc));
        if bc > workprec {
            y = sh(&y, bc - workprec);
            exp += bc - workprec;
            bc = workprec;
        }
        n /= 2;
    }
    (pm, pe)
}

/// `nthroot_fixed(y, n, prec, exp1)`: Newton's method from a 50-bit
/// float estimate.
pub fn nthroot_fixed(y: &Int, n: i64, prec: i64, _exp1: i64) -> Int {
    let start = 50;
    let y1 = rshift(y, prec - n * start);
    // r = MPZ(int(y1**(1.0/n))), or the mpf route on OverflowError.
    let est = y1
        .to_f64()
        .map(|f| pyf_pow(f, 1.0 / n as f64))
        .and_then(Int::from_f64_trunc);
    let mut r = match est {
        Some(r) => r,
        None => {
            let y1 = from_int(&y1, start, ROUND_FAST);
            let fnn = from_int(&Int::from(n), 0, ROUND_FAST);
            let fnn = mpf_rdiv_int(&Int::ONE, &fnn, start, ROUND_FAST).expect("n != 0");
            let r = mpf_pow(&y1, &fnn, start, ROUND_FAST).expect("y1 >= 0");
            to_int(&r, None).expect("finite")
        }
    };
    let extra = 10;
    let extra1 = n;
    let mut prevp = start;
    for p in giant_steps(start, prec + extra, 2) {
        let (pm, pe) = int_pow_fixed(&r, n - 1, prevp);
        let r2 = rshift(&pm, (n - 1) * prevp - p - pe - extra1);
        let b = floordiv(&lshift(y, 2 * p - prec + extra1), &r2);
        r = floordiv(
            &(&b + &(&Int::from(n - 1) * &lshift(&r, p - prevp))),
            &Int::from(n),
        );
        prevp = p;
    }
    r
}

/// `mpf_nthroot(s, n, prec, rnd)`: the n-th root of a non-negative number.
pub fn mpf_nthroot(s: &Mpf, n: i64, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    let (sign, man, exp, bc) = (s.sign, &s.man, s.exp, s.bc);
    if sign == 1 {
        return Err(MpfError::ComplexResult("nth root of a negative number"));
    }
    if man.is_zero() {
        if *s == FNAN {
            return Ok(FNAN);
        }
        if *s == FZERO {
            if n > 0 {
                return Ok(FZERO);
            }
            if n == 0 {
                return Ok(FONE);
            }
            return Ok(FINF);
        }
        // Infinity
        if n == 0 {
            return Ok(FNAN);
        }
        if n < 0 {
            return Ok(FZERO);
        }
        return Ok(FINF);
    }
    let mut flag_inverse = false;
    let mut rnd = rnd;
    let mut prec = prec;
    let mut n = n;
    let extra_inverse = 5;
    if n < 2 {
        if n == 0 {
            return Ok(FONE);
        }
        if n == 1 {
            return Ok(mpf_pos(s, prec, rnd));
        }
        if n == -1 {
            return mpf_div(&FONE, s, prec, rnd);
        }
        // n < 0
        rnd = rnd.reciprocal();
        flag_inverse = true;
        prec += extra_inverse;
        n = -n;
    }
    if n > 20 && (n >= 20000 || prec < (233.0 + 28.3 * pyf_pow(n as f64, 0.62)) as i64) {
        let prec2 = prec + 10;
        let fnn = from_int(&Int::from(n), 0, ROUND_FAST);
        let nth = mpf_rdiv_int(&Int::ONE, &fnn, prec2, ROUND_FAST)?;
        let r = mpf_pow(s, &nth, prec2, rnd)?;
        let s = normalize(r.sign, r.man, r.exp, r.bc, prec, rnd);
        if flag_inverse {
            return mpf_div(&FONE, &s, prec - extra_inverse, rnd);
        }
        return Ok(s);
    }
    // Convert to a fixed-point number with prec2 bits.
    let mut prec2 = prec + 2 * n - prec.rem_euclid(n);
    // a few tests indicate that for 10 < n < 10**4 a bit more precision is
    // needed
    if n > 10 {
        prec2 += prec2.div_euclid(10);
        prec2 -= prec2.rem_euclid(n);
    }
    // Mantissa may have more bits than we need. Trim it down.
    let mut shift = bc - prec2;
    // Adjust exponents to make prec2 and exp+shift multiples of n.
    let mut sign1 = 0;
    let mut es = exp + shift;
    if es < 0 {
        sign1 = 1;
        es = -es;
    }
    if sign1 == 1 {
        shift += es.rem_euclid(n);
    } else {
        shift -= es.rem_euclid(n);
    }
    let man = rshift(man, shift);
    let extra = 10;
    let exp1 = (exp + shift - (n - 1) * prec2).div_euclid(n) - extra;
    let mut rnd_shift = 0;
    if flag_inverse {
        if matches!(rnd, Rnd::Up | Rnd::Ceiling) {
            rnd_shift = 1;
        }
    } else if matches!(rnd, Rnd::Down | Rnd::Floor) {
        rnd_shift = 1;
    }
    let man = nthroot_fixed(&(&man + rnd_shift), n, prec2, exp1);
    let s = from_man_exp(&man, exp1, prec, rnd);
    if flag_inverse {
        mpf_div(&FONE, &s, prec - extra_inverse, rnd)
    } else {
        Ok(s)
    }
}

/// `mpf_cbrt(s, prec, rnd)`.
pub fn mpf_cbrt(s: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    mpf_nthroot(s, 3, prec, rnd)
}

//----------------------------------------------------------------------
// Logarithms
//----------------------------------------------------------------------

/// `log_int_fixed(n, prec, ln2=None)`: `log(n)` as a fixed-point number,
/// cached for small `n` at the first precision that asks for more.
pub fn log_int_fixed(n: &Int, prec: i64, ln2: Option<&Int>) -> Int {
    if let Some((value, vprec)) = LOG_INT_CACHE.with(|c| c.borrow().get(n).cloned())
        && vprec >= prec
    {
        return sh(&value, vprec - prec);
    }
    let wp = prec + 10;
    let v = if wp <= LOG_TAYLOR_SHIFT {
        let ln2 = match ln2 {
            Some(v) => v.clone(),
            None => ln2_fixed(wp),
        };
        let r = bitcount(n);
        let x = shl(n, wp - r);
        &log_taylor_cached(&x, wp) + &(&Int::from(r) * &ln2)
    } else {
        let l = mpf_log(&from_int(n, 0, ROUND_FAST), wp + 5, ROUND_FAST)
            .expect("log of a positive integer");
        to_fixed(&l, wp)
    };
    if *n < MAX_LOG_INT_CACHE {
        LOG_INT_CACHE.with(|c| c.borrow_mut().insert(n.clone(), (v.clone(), wp)));
    }
    sh(&v, wp - prec)
}

/// `agm_fixed(a, b, prec)`: the AGM of two fixed-point numbers close to 1.
pub fn agm_fixed(a: &Int, b: &Int, _prec: i64) -> Int {
    let (mut a, mut b) = (a.clone(), b.clone());
    let mut i = 0;
    loop {
        let anew = &(&a + &b) >> 1;
        if i > 4 && (&a - &anew).abs() < 8 {
            return a;
        }
        b = isqrt_fast(&(&a * &b));
        a = anew;
        i += 1;
    }
}

/// `log_agm(x, prec)`: `-log(x)` for `0 < x < 1` (Sasaki-Kanada).
pub fn log_agm(x: &Int, prec: i64) -> Int {
    let one = shl(&Int::ONE, prec);
    let x2 = sh(&(x * x), prec);
    // Compute jtheta2(x)**2
    let mut s = x2.clone();
    let mut a = x2.clone();
    let mut b = x2.clone();
    while !a.is_zero() {
        b = sh(&(&b * &x2), prec);
        a = sh(&(&a * &b), prec);
        s = &s + &a;
    }
    s = &s + &one;
    s = sh(&(&s * &s), prec - 2);
    s = sh(&(&s * &isqrt_fast(&shl(x, prec))), prec);
    // Compute jtheta3(x)**2
    let mut t = x.clone();
    let mut a = x.clone();
    let mut b = x.clone();
    while !a.is_zero() {
        b = sh(&(&b * &x2), prec);
        a = sh(&(&a * &b), prec);
        t = &t + &a;
    }
    t = &one + &(&t << 1);
    t = sh(&(&t * &t), prec);
    // Final formula
    let p = agm_fixed(&s, &t, prec);
    floordiv(&shl(&pi_fixed(prec), prec), &p)
}

/// `log_taylor(x, prec, r)`: fixed-point `log(x)` for `x` near 1, after
/// `r` square roots.
pub fn log_taylor(x: &Int, prec: i64, r: i64) -> Int {
    let mut x = x.clone();
    for _ in 0..r {
        x = isqrt_fast(&shl(&x, prec));
    }
    let one = shl(&Int::ONE, prec);
    let mut v = floordiv(&shl(&(&x - &one), prec), &(&x + &one));
    let sign = v.is_negative();
    if sign {
        v = -v;
    }
    let v2 = sh(&(&v * &v), prec);
    let v4 = sh(&(&v2 * &v2), prec);
    let mut s0 = v.clone();
    let mut s1 = floordiv(&v, &Int::from(3));
    v = sh(&(&v * &v4), prec);
    let mut k = 5i64;
    while !v.is_zero() {
        s0 = &s0 + &floordiv(&v, &Int::from(k));
        k += 2;
        s1 = &s1 + &floordiv(&v, &Int::from(k));
        v = sh(&(&v * &v4), prec);
        k += 2;
    }
    s1 = sh(&(&s1 * &v2), prec);
    let s = shl(&(&s0 + &s1), 1 + r);
    if sign { -s } else { s }
}

/// `log_taylor_cached(x, prec)`: fixed-point `log(x)` for `x` in (0.5, 2)
/// and `prec <= LOG_TAYLOR_PREC`.
pub fn log_taylor_cached(x: &Int, prec: i64) -> Int {
    let n = sh(x, prec - LOG_TAYLOR_SHIFT);
    let cached_prec = cache_prec_steps(prec);
    let dprec = cached_prec - prec;
    let key = (n.clone(), cached_prec);
    let (a, log_a) = match LOG_TAYLOR_CACHE.with(|c| c.borrow().get(&key).cloned()) {
        Some(v) => v,
        None => {
            let a = shl(&n, cached_prec - LOG_TAYLOR_SHIFT);
            let log_a = log_taylor(&a, cached_prec, 8);
            LOG_TAYLOR_CACHE.with(|c| c.borrow_mut().insert(key, (a.clone(), log_a.clone())));
            (a, log_a)
        }
    };
    let a = sh(&a, dprec);
    let log_a = sh(&log_a, dprec);
    let u = floordiv(&shl(&(x - &a), prec), &a);
    let mut v = floordiv(&shl(&u, prec), &(&shl(&Int::from(2), prec) + &u));
    let v2 = sh(&(&v * &v), prec);
    let v4 = sh(&(&v2 * &v2), prec);
    let mut s0 = v.clone();
    let mut s1 = floordiv(&v, &Int::from(3));
    v = sh(&(&v * &v4), prec);
    let mut k = 5i64;
    while !v.is_zero() {
        s0 = &s0 + &floordiv(&v, &Int::from(k));
        k += 2;
        s1 = &s1 + &floordiv(&v, &Int::from(k));
        v = sh(&(&v * &v4), prec);
        k += 2;
    }
    s1 = sh(&(&s1 * &v2), prec);
    let s = &(&s0 + &s1) << 1;
    &log_a + &s
}

/// `mpf_log(x, prec, rnd)`: the natural logarithm; `ComplexResult` for a
/// negative `x`.
pub fn mpf_log(x: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    let (sign, man, exp, bc) = (x.sign, &x.man, x.exp, x.bc);
    // Handle special values
    if man.is_zero() {
        if *x == FZERO {
            return Ok(FNINF);
        }
        if *x == FINF {
            return Ok(FINF);
        }
        if *x == FNAN {
            return Ok(FNAN);
        }
    }
    if sign == 1 {
        return Err(MpfError::ComplexResult("logarithm of a negative number"));
    }
    let mut wp = prec + 20;
    // Handle log(2^n) = log(n)*2.
    // Here we catch the only possible exact value, log(1) = 0
    if man.is_one() {
        if exp == 0 {
            return Ok(FZERO);
        }
        return Ok(from_man_exp(
            &(&Int::from(exp) * &ln2_fixed(wp)),
            -wp,
            prec,
            rnd,
        ));
    }
    let mag = exp + bc;
    let abs_mag = mag.abs();
    // Handle x = 1+eps, where log(x) ~ x. (mpmath measures the distance
    // from 1 this way for mag = -1 too, where it is the distance from 1/4.)
    if abs_mag <= 1 {
        // Calculate t = x-1 to measure distance from 1 in bits
        let tsign = 1 - abs_mag;
        let tman = if tsign != 0 {
            &shl(&Int::ONE, bc) - man
        } else {
            man - &shl(&Int::ONE, bc - 1)
        };
        let tbc = bitcount(&tman);
        let cancellation = bc - tbc;
        if cancellation > wp {
            let t = normalize(tsign as u8, tman, abs_mag - bc, tbc, tbc, Rnd::Nearest);
            return Ok(mpf_perturb(&t, tsign as u8, prec, rnd));
        } else {
            wp += cancellation;
        }
    }
    // Another special case: n*log(2) is a good enough approximation
    if abs_mag > 10000 && bitcount(&Int::from(abs_mag)) > wp {
        return Ok(from_man_exp(
            &(&Int::from(exp) * &ln2_fixed(wp)),
            -wp,
            prec,
            rnd,
        ));
    }
    // General case.
    let m = if wp <= LOG_TAYLOR_PREC {
        let mut m = log_taylor_cached(&lshift(man, wp - bc), wp);
        if mag != 0 {
            m = &m + &(&Int::from(mag) * &ln2_fixed(wp));
        }
        m
    } else {
        let optimal_mag = (-wp).div_euclid(LOG_AGM_MAG_PREC_RATIO);
        let n = optimal_mag - mag;
        let x = mpf_shift(x, n);
        wp += -optimal_mag;
        let mut m = -log_agm(&to_fixed(&x, wp), wp);
        m = &m - &(&Int::from(n) * &ln2_fixed(wp));
        m
    };
    Ok(from_man_exp(&m, -wp, prec, rnd))
}

/// `mpf_log_hypot(a, b, prec, rnd)`: `log(sqrt(a**2 + b**2))`.
pub fn mpf_log_hypot(a: &Mpf, b: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    // If either a or b is inf/nan/0, assume it to be a
    let (a, b) = if b.man.is_zero() { (b, a) } else { (a, b) };
    // a is inf/nan/0
    if a.man.is_zero() {
        // both are inf/nan/0
        if b.man.is_zero() {
            if *a == FZERO && *b == FZERO {
                return FNINF;
            }
            if *a == FNAN || *b == FNAN {
                return FNAN;
            }
            // at least one term is (+/- inf)^2
            return FINF;
        }
        // only a is inf/nan/0
        if *a == FZERO {
            // log(sqrt(0+b^2)) = log(|b|)
            return mpf_log(&mpf_abs(b, 0, ROUND_FAST), prec, rnd).expect("|b| >= 0");
        }
        if *a == FNAN {
            return FNAN;
        }
        return FINF;
    }
    // Exact
    let a2 = mpf_mul(a, a, 0, ROUND_FAST);
    let b2 = mpf_mul(b, b, 0, ROUND_FAST);
    let extra = 20;
    // Not exact
    let mut h2 = mpf_add(&a2, &b2, prec + extra, ROUND_FAST);
    let cancelled = mpf_add(&h2, &FNONE, 10, ROUND_FAST);
    let mag_cancelled = cancelled.exp + cancelled.bc;
    // Just redo the sum exactly if necessary
    if cancelled == FZERO || mag_cancelled < (-extra).div_euclid(2) {
        h2 = mpf_add(&a2, &b2, prec + extra - a2.exp.min(b2.exp), ROUND_FAST);
    }
    mpf_shift(&mpf_log(&h2, prec, rnd).expect("h2 >= 0"), -1)
}

//----------------------------------------------------------------------
// Inverse tangent
//----------------------------------------------------------------------

/// `atan_newton(x, prec)`.
pub fn atan_newton(x: &Int, prec: i64) -> Int {
    let two53 = 9_007_199_254_740_992.0f64;
    let r = if prec >= 100 {
        let v = sh(x, prec - 53).to_f64().expect("54 bits fit in a float");
        (v / two53).atan()
    } else {
        let v = x.to_f64().expect("small enough for a float");
        (v / 2f64.powi(prec as i32)).atan()
    };
    let mut prevp = 50i64;
    let mut r = sh(&Int::from_f64_trunc(r * two53).expect("finite"), 53 - prevp);
    let extra_p = 50;
    for wp in giant_steps(prevp, prec, 2) {
        let wp = wp + extra_p;
        r = shl(&r, wp - prevp);
        let (cos, sin) = cos_sin_fixed(&r, wp, None);
        let tan = floordiv(&shl(&sin, wp), &cos);
        let a = floordiv(
            &shl(&(&tan - &rshift(x, prec - wp)), wp),
            &(&shl(&Int::ONE, wp) + &sh(&(&tan * &tan), wp)),
        );
        r = &r - &a;
        prevp = wp;
    }
    rshift(&r, prevp - prec)
}

/// `atan_taylor_get_cached(n, prec)`.
fn atan_taylor_get_cached(n: &Int, prec: i64) -> (Int, Int) {
    // Round to next power of 2
    let prec2 = (1i64 << bitcount(&Int::from(prec - 1))) + 20;
    let dprec = prec2 - prec;
    let key = (n.clone(), prec2);
    let (a, atan_a) = match ATAN_TAYLOR_CACHE.with(|c| c.borrow().get(&key).cloned()) {
        Some(v) => v,
        None => {
            let a = shl(n, prec2 - ATAN_TAYLOR_SHIFT);
            let atan_a = atan_newton(&a, prec2);
            ATAN_TAYLOR_CACHE.with(|c| c.borrow_mut().insert(key, (a.clone(), atan_a.clone())));
            (a, atan_a)
        }
    };
    (sh(&a, dprec), sh(&atan_a, dprec))
}

/// `atan_taylor(x, prec)`.
pub fn atan_taylor(x: &Int, prec: i64) -> Int {
    let n = sh(x, prec - ATAN_TAYLOR_SHIFT);
    let (a, atan_a) = atan_taylor_get_cached(&n, prec);
    let d = x - &a;
    let one = shl(&Int::ONE, prec);
    let den = &(&sh(&(&a * &a), prec) + &sh(&(&a * &d), prec)) + &one;
    let mut v = floordiv(&shl(&d, prec), &den);
    let mut s0 = v.clone();
    let v2 = sh(&(&v * &v), prec);
    let v4 = sh(&(&v2 * &v2), prec);
    let mut s1 = floordiv(&v, &Int::from(3));
    v = sh(&(&v * &v4), prec);
    let mut k = 5i64;
    while !v.is_zero() {
        s0 = &s0 + &floordiv(&v, &Int::from(k));
        k += 2;
        s1 = &s1 + &floordiv(&v, &Int::from(k));
        v = sh(&(&v * &v4), prec);
        k += 2;
    }
    s1 = sh(&(&s1 * &v2), prec);
    let s = &s0 - &s1;
    &atan_a + &s
}

/// `atan_inf(sign, prec, rnd)`: `+-pi/2`.
fn atan_inf(sign: u8, prec: i64, rnd: Rnd) -> Mpf {
    if sign == 0 {
        return mpf_shift(&mpf_pi(prec, rnd), -1);
    }
    mpf_neg(&mpf_shift(&mpf_pi(prec, rnd.negative()), -1), 0, ROUND_FAST)
}

/// `mpf_atan(x, prec, rnd)`.
pub fn mpf_atan(x: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    let (sign, man, exp, bc) = (x.sign, &x.man, x.exp, x.bc);
    if man.is_zero() {
        if *x == FZERO {
            return FZERO;
        }
        if *x == FINF {
            return atan_inf(0, prec, rnd);
        }
        if *x == FNINF {
            return atan_inf(1, prec, rnd);
        }
        return FNAN;
    }
    let mag = exp + bc;
    // Essentially infinity
    if mag > prec + 20 {
        return atan_inf(sign, prec, rnd);
    }
    // Essentially ~ x
    if -mag > prec + 20 {
        return mpf_perturb(x, 1 - sign, prec, rnd);
    }
    let wp = prec + 30 + mag.abs();
    // For large x, use atan(x) = pi/2 - atan(1/x)
    let (x, reciprocal) = if mag >= 2 {
        (
            mpf_rdiv_int(&Int::ONE, x, wp, ROUND_FAST).expect("x != 0"),
            true,
        )
    } else {
        (x.clone(), false)
    };
    let mut t = to_fixed(&x, wp);
    if sign == 1 {
        t = -t;
    }
    let mut a = if wp < ATAN_TAYLOR_PREC {
        atan_taylor(&t, wp)
    } else {
        atan_newton(&t, wp)
    };
    if reciprocal {
        a = &(&(&pi_fixed(wp) >> 1) + 1) - &a;
    }
    if sign == 1 {
        a = -a;
    }
    from_man_exp(&a, -wp, prec, rnd)
}

/// `mpf_atan2(y, x, prec, rnd)`.
pub fn mpf_atan2(y: &Mpf, x: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    if y.man.is_zero() {
        if *y == FZERO && *x != FNAN {
            if mpf_sign(x) >= 0 {
                return FZERO;
            }
            return mpf_pi(prec, rnd);
        }
        if *y == FINF || *y == FNINF {
            if *x == FINF || *x == FNINF {
                return FNAN;
            }
            // pi/2
            if *y == FINF {
                return mpf_shift(&mpf_pi(prec, rnd), -1);
            }
            // -pi/2
            return mpf_neg(&mpf_shift(&mpf_pi(prec, rnd.negative()), -1), 0, ROUND_FAST);
        }
        return FNAN;
    }
    if y.sign == 1 {
        return mpf_neg(
            &mpf_atan2(&mpf_neg(y, 0, ROUND_FAST), x, prec, rnd.negative()),
            0,
            ROUND_FAST,
        );
    }
    if x.man.is_zero() {
        if *x == FNAN {
            return FNAN;
        }
        if *x == FINF {
            return FZERO;
        }
        if *x == FNINF {
            return mpf_pi(prec, rnd);
        }
        if *y == FZERO {
            return FZERO;
        }
        return mpf_shift(&mpf_pi(prec, rnd), -1);
    }
    let tquo = mpf_atan(
        &mpf_div(y, x, prec + 4, ROUND_FAST).expect("x != 0"),
        prec + 4,
        ROUND_FAST,
    );
    if x.sign == 1 {
        mpf_add(&mpf_pi(prec + 4, ROUND_FAST), &tquo, prec, rnd)
    } else {
        mpf_pos(&tquo, prec, rnd)
    }
}

fn is_one_or_none(x: &Mpf) -> bool {
    *x == FONE || *x == FNONE
}

/// `mpf_asin(x, prec, rnd)`.
pub fn mpf_asin(x: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    if x.bc + x.exp > 0 && !is_one_or_none(x) {
        return Err(MpfError::ComplexResult(
            "asin(x) is real only for -1 <= x <= 1",
        ));
    }
    // asin(x) = 2*atan(x/(1+sqrt(1-x**2)))
    let wp = prec + 15;
    let a = mpf_mul(x, x, 0, ROUND_FAST);
    let b = mpf_add(
        &FONE,
        &mpf_sqrt(&mpf_sub(&FONE, &a, wp, ROUND_FAST), wp, ROUND_FAST)?,
        wp,
        ROUND_FAST,
    );
    let c = mpf_div(x, &b, wp, ROUND_FAST)?;
    Ok(mpf_shift(&mpf_atan(&c, prec, rnd), 1))
}

/// `mpf_acos(x, prec, rnd)`.
pub fn mpf_acos(x: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    // acos(x) = 2*atan(sqrt(1-x**2)/(1+x))
    if x.bc + x.exp > 0 {
        if !is_one_or_none(x) {
            return Err(MpfError::ComplexResult(
                "acos(x) is real only for -1 <= x <= 1",
            ));
        }
        if *x == FNONE {
            return Ok(mpf_pi(prec, rnd));
        }
    }
    let wp = prec + 15;
    let a = mpf_mul(x, x, 0, ROUND_FAST);
    let b = mpf_sqrt(&mpf_sub(&FONE, &a, wp, ROUND_FAST), wp, ROUND_FAST)?;
    let c = mpf_div(&b, &mpf_add(&FONE, x, wp, ROUND_FAST), wp, ROUND_FAST)?;
    Ok(mpf_shift(&mpf_atan(&c, prec, rnd), 1))
}

/// `mpf_asinh(x, prec, rnd)`.
pub fn mpf_asinh(x: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    let mut wp = prec + 20;
    let mag = x.exp + x.bc;
    if mag < -8 {
        if mag < -wp {
            return mpf_perturb(x, 1 - x.sign, prec, rnd);
        }
        wp += -mag;
    }
    // asinh(x) = log(x+sqrt(x**2+1))
    // use reflection symmetry to avoid cancellation
    let q = mpf_sqrt(
        &mpf_add(&mpf_mul(x, x, 0, ROUND_FAST), &FONE, wp, ROUND_FAST),
        wp,
        ROUND_FAST,
    )
    .expect("x**2 + 1 >= 0");
    let q = mpf_add(&mpf_abs(x, 0, ROUND_FAST), &q, wp, ROUND_FAST);
    if x.sign == 1 {
        mpf_neg(
            &mpf_log(&q, prec, rnd.negative()).expect("q >= 0"),
            0,
            ROUND_FAST,
        )
    } else {
        mpf_log(&q, prec, rnd).expect("q >= 0")
    }
}

/// `mpf_acosh(x, prec, rnd)`.
pub fn mpf_acosh(x: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    // acosh(x) = log(x+sqrt(x**2-1))
    let wp = prec + 15;
    if mpf_cmp(x, &FONE) == -1 {
        return Err(MpfError::ComplexResult("acosh(x) is real only for x >= 1"));
    }
    let q = mpf_sqrt(
        &mpf_add(&mpf_mul(x, x, 0, ROUND_FAST), &FNONE, wp, ROUND_FAST),
        wp,
        ROUND_FAST,
    )?;
    mpf_log(&mpf_add(x, &q, wp, ROUND_FAST), prec, rnd)
}

/// `mpf_atanh(x, prec, rnd)`.
pub fn mpf_atanh(x: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    // atanh(x) = log((1+x)/(1-x))/2
    let (sign, man, exp, bc) = (x.sign, &x.man, x.exp, x.bc);
    if man.is_zero() && exp != 0 {
        if *x == FZERO || *x == FNAN {
            return Ok(x.clone());
        }
        return Err(MpfError::ComplexResult(
            "atanh(x) is real only for -1 <= x <= 1",
        ));
    }
    let mag = bc + exp;
    if mag > 0 {
        if mag == 1 && man.is_one() {
            return Ok(if sign == 0 { FINF } else { FNINF });
        }
        return Err(MpfError::ComplexResult(
            "atanh(x) is real only for -1 <= x <= 1",
        ));
    }
    let mut wp = prec + 15;
    if mag < -8 {
        if mag < -wp {
            return Ok(mpf_perturb(x, sign, prec, rnd));
        }
        wp += -mag;
    }
    let a = mpf_add(x, &FONE, wp, ROUND_FAST);
    let b = mpf_sub(&FONE, x, wp, ROUND_FAST);
    Ok(mpf_shift(
        &mpf_log(&mpf_div(&a, &b, wp, ROUND_FAST)?, prec, rnd)?,
        -1,
    ))
}

/// `mpf_fibonacci(x, prec, rnd)`.
pub fn mpf_fibonacci(x: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    let (man, exp, bc) = (&x.man, x.exp, x.bc);
    if man.is_zero() {
        if *x == FNINF {
            return FNAN;
        }
        return x.clone();
    }
    // F(2^n) ~= 2^(2^n)
    let size = (exp + bc).abs();
    if exp >= 0 {
        // Exact
        if size < 10 || size <= bitcount(&Int::from(prec)) {
            return from_int(&ifib(&to_int(x, None).expect("finite")), prec, rnd);
        }
    }
    // Use the modified Binet formula
    let wp = prec + size + 20;
    let a = mpf_phi(wp, ROUND_FAST);
    let b = mpf_add(&mpf_shift(&a, 1), &FNONE, wp, ROUND_FAST);
    let u = mpf_pow(&a, x, wp, ROUND_FAST).expect("phi > 0");
    let v = mpf_cos_pi(x, wp, ROUND_FAST);
    let v = mpf_div(&v, &u, wp, ROUND_FAST).expect("phi**x != 0");
    let u = mpf_sub(&u, &v, wp, ROUND_FAST);
    mpf_div(&u, &b, prec, rnd).expect("b != 0")
}

//----------------------------------------------------------------------
// Exponential-type functions
//----------------------------------------------------------------------

/// The result of `exponential_series`.
pub enum ExpSeries {
    /// `type = 0`: `exp(x)`.
    Exp(Int),
    /// `type = 1` (cosh, sinh) or `type = 2` (cos, sin).
    Pair(Int, Int),
}

/// `exponential_series(x, prec, type)`: Taylor series for exp (type 0),
/// cosh/sinh (1) or cos/sin (2).
pub fn exponential_series(x: &Int, prec: i64, typ: u8) -> ExpSeries {
    let (mut x, sign) = if x.is_negative() {
        (-x, 1)
    } else {
        (x.clone(), 0)
    };
    let r = (0.5 * pyf_pow(prec as f64, 0.5)) as i64;
    let xmag = bitcount(&x) - prec;
    let r = 0.max(xmag + r);
    let extra = 10 + 2 * r.max(-xmag);
    let wp = prec + extra;
    x = shl(&x, extra - r);
    let one = shl(&Int::ONE, wp);
    let alt = typ == 2;
    let mut c = if prec < EXP_SERIES_U_CUTOFF {
        let x2 = sh(&(&x * &x), wp);
        let mut a = x2.clone();
        let x4 = sh(&(&x2 * &x2), wp);
        let mut s0 = Int::ZERO;
        let mut s1 = Int::ZERO;
        let mut k = 2i64;
        while !a.is_zero() {
            a = floordiv(&a, &Int::from((k - 1) * k));
            s0 = &s0 + &a;
            k += 2;
            a = floordiv(&a, &Int::from((k - 1) * k));
            s1 = &s1 + &a;
            k += 2;
            a = sh(&(&a * &x4), wp);
        }
        s1 = sh(&(&x2 * &s1), wp);
        if alt {
            &(&s1 - &s0) + &one
        } else {
            &(&s1 + &s0) + &one
        }
    } else {
        let u = (0.3 * pyf_pow(prec as f64, 0.35)) as i64;
        let x2 = sh(&(&x * &x), wp);
        let mut a = x2.clone();
        let mut xpowers = vec![one.clone(), x2.clone()];
        for _ in 1..u {
            let next = sh(&(xpowers.last().expect("non-empty") * &x2), wp);
            xpowers.push(next);
        }
        let mut sums = vec![Int::ZERO; u as usize];
        let mut k = 2i64;
        while !a.is_zero() {
            for s in sums.iter_mut() {
                a = floordiv(&a, &Int::from((k - 1) * k));
                if alt && k & 2 != 0 {
                    *s = &*s - &a;
                } else {
                    *s = &*s + &a;
                }
                k += 2;
            }
            a = sh(&(&a * xpowers.last().expect("non-empty")), wp);
        }
        for i in 1..u as usize {
            sums[i] = sh(&(&sums[i] * &xpowers[i]), wp);
        }
        &sums.into_iter().fold(Int::ZERO, |acc, s| &acc + &s) + &one
    };
    if typ == 0 {
        let s = isqrt_fast(&(&(&c * &c) - &shl(&one, wp)));
        let mut v = if sign == 1 { &c - &s } else { &c + &s };
        for _ in 0..r {
            v = sh(&(&v * &v), wp);
        }
        ExpSeries::Exp(sh(&v, extra))
    } else {
        // Repeatedly apply the double-angle formula
        let pshift = wp - 1;
        for _ in 0..r {
            c = &sh(&(&c * &c), pshift) - &one;
        }
        // With the abs, this is the same for sinh and sin
        let mut s = isqrt_fast(&(&shl(&one, wp) - &(&c * &c)).abs());
        if sign == 1 {
            s = -s;
        }
        ExpSeries::Pair(sh(&c, extra), sh(&s, extra))
    }
}

fn series_exp(x: &Int, prec: i64) -> Int {
    match exponential_series(x, prec, 0) {
        ExpSeries::Exp(v) => v,
        ExpSeries::Pair(..) => unreachable!("type 0 gives exp"),
    }
}

fn series_pair(x: &Int, prec: i64, typ: u8) -> (Int, Int) {
    match exponential_series(x, prec, typ) {
        ExpSeries::Pair(c, s) => (c, s),
        ExpSeries::Exp(_) => unreachable!("types 1 and 2 give pairs"),
    }
}

/// `exp_basecase(x, prec)`: fixed-point `exp(x)`, best for `|x| < 1`.
pub fn exp_basecase(x: &Int, prec: i64) -> Int {
    if prec > EXP_COSH_CUTOFF {
        return series_exp(x, prec);
    }
    let mut r = pyf_pow(prec as f64, 0.5) as i64;
    let prec = prec + r;
    let mut s0 = shl(&Int::ONE, prec);
    let mut s1 = s0.clone();
    let mut k = 2i64;
    let x2 = sh(&(x * x), prec);
    let mut a = x2.clone();
    while !a.is_zero() {
        a = floordiv(&a, &Int::from(k));
        s0 = &s0 + &a;
        k += 1;
        a = floordiv(&a, &Int::from(k));
        s1 = &s1 + &a;
        k += 1;
        a = sh(&(&a * &x2), prec);
    }
    s1 = sh(&(&s1 * x), prec);
    let mut s = &s0 + &s1;
    let u = r;
    while r != 0 {
        s = sh(&(&s * &s), prec);
        r -= 1;
    }
    sh(&s, u)
}

/// `exp_expneg_basecase(x, prec)`: `(exp(x), exp(-x))`.
pub fn exp_expneg_basecase(x: &Int, prec: i64) -> (Int, Int) {
    if prec > EXP_COSH_CUTOFF {
        let (cosh, sinh) = series_pair(x, prec, 1);
        return (&cosh + &sinh, &cosh - &sinh);
    }
    let a = exp_basecase(x, prec);
    let b = floordiv(&shl(&Int::ONE, prec + prec), &a);
    (a, b)
}

/// `cos_sin_basecase(x, prec)`: fixed-point `(cos(x), sin(x))` for `x` in
/// `[0, pi/2)`.
pub fn cos_sin_basecase(x: &Int, prec: i64) -> (Int, Int) {
    if prec > COS_SIN_CACHE_PREC {
        return series_pair(x, prec, 2);
    }
    let precs = prec - COS_SIN_CACHE_STEP;
    let t = sh(x, precs);
    let n = t.clone();
    let (cos_t, sin_t) = match COS_SIN_CACHE.with(|c| c.borrow().get(&n).cloned()) {
        Some(v) => v,
        None => {
            let w = shl(&t, 10 + COS_SIN_CACHE_PREC - COS_SIN_CACHE_STEP);
            let (cos_t, sin_t) = series_pair(&w, 10 + COS_SIN_CACHE_PREC, 2);
            let v = (&cos_t >> 10, &sin_t >> 10);
            COS_SIN_CACHE.with(|c| c.borrow_mut().insert(n, v.clone()));
            v
        }
    };
    let offset = COS_SIN_CACHE_PREC - prec;
    let cos_t = sh(&cos_t, offset);
    let sin_t = sh(&sin_t, offset);
    let x = x - &shl(&t, precs);
    let mut cos = shl(&Int::ONE, prec);
    let mut sin = x.clone();
    let mut k = 2i64;
    let mut a = -sh(&(&x * &x), prec);
    while !a.is_zero() {
        a = floordiv(&a, &Int::from(k));
        cos = &cos + &a;
        k += 1;
        a = sh(&(&a * &x), prec);
        a = floordiv(&a, &Int::from(k));
        sin = &sin + &a;
        k += 1;
        a = -sh(&(&a * &x), prec);
    }
    (
        sh(&(&(&cos * &cos_t) - &(&sin * &sin_t)), prec),
        sh(&(&(&sin * &cos_t) + &(&cos * &sin_t)), prec),
    )
}

/// `mpf_exp(x, prec, rnd)`.
pub fn mpf_exp(x: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    let (sign, exp, bc) = (x.sign, x.exp, x.bc);
    if !x.man.is_zero() {
        let mag = bc + exp;
        let wp = prec + 14;
        let man = if sign == 1 { -&x.man } else { x.man.clone() };
        // TODO: the best cutoff depends on both x and the precision.
        if prec > 600 && exp >= 0 {
            // Need about log2(exp(n)) ~= 1.45*mag extra precision
            let e = mpf_e(wp + (1.45 * mag as f64) as i64, ROUND_FAST);
            return mpf_pow_int(&e, &shl(&man, exp), prec, rnd).expect("e > 0");
        }
        if mag < -wp {
            return mpf_perturb(&FONE, sign, prec, rnd);
        }
        let (t, n) = if mag > 1 {
            // |x| >= 2
            let wpmod = wp + mag;
            let offset = exp + wpmod;
            let t = lshift(&man, offset);
            let lg2 = ln2_fixed(wpmod);
            let (n, t) = t.div_mod(&lg2).expect("ln2 > 0");
            (sh(&t, mag), small(&n, "exp: quotient by ln 2"))
        } else {
            let offset = exp + wp;
            (lshift(&man, offset), 0)
        };
        let man = exp_basecase(&t, wp);
        return from_man_exp(&man, n - wp, prec, rnd);
    }
    if exp == 0 {
        return FONE;
    }
    if *x == FNINF {
        return FZERO;
    }
    x.clone()
}

/// `mpf_cosh_sinh(x, prec, rnd)`: `(cosh(x), sinh(x))`.
pub fn mpf_cosh_sinh(x: &Mpf, prec: i64, rnd: Rnd) -> (Mpf, Mpf) {
    match cosh_sinh_impl(x, prec, rnd, false) {
        CoshSinh::Pair(c, s) => (c, s),
        CoshSinh::Tanh(_) => unreachable!("tanh=0 gives a pair"),
    }
}

/// `mpf_tanh(x, prec, rnd)`: `mpf_cosh_sinh(x, prec, rnd, tanh=1)`.
pub fn mpf_tanh(x: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    match cosh_sinh_impl(x, prec, rnd, true) {
        CoshSinh::Tanh(t) => t,
        CoshSinh::Pair(..) => unreachable!("tanh=1 gives one value"),
    }
}

/// `mpf_cosh(x, prec, rnd)`.
pub fn mpf_cosh(x: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    mpf_cosh_sinh(x, prec, rnd).0
}

/// `mpf_sinh(x, prec, rnd)`.
pub fn mpf_sinh(x: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    mpf_cosh_sinh(x, prec, rnd).1
}

enum CoshSinh {
    Pair(Mpf, Mpf),
    Tanh(Mpf),
}

/// `mpf_cosh_sinh(x, prec, rnd, tanh)`.
fn cosh_sinh_impl(x: &Mpf, prec: i64, rnd: Rnd, tanh: bool) -> CoshSinh {
    let (sign, man, exp, bc) = (x.sign, &x.man, x.exp, x.bc);
    if man.is_zero() && exp != 0 {
        if tanh {
            if *x == FINF {
                return CoshSinh::Tanh(FONE);
            }
            if *x == FNINF {
                return CoshSinh::Tanh(FNONE);
            }
            return CoshSinh::Tanh(FNAN);
        }
        if *x == FINF {
            return CoshSinh::Pair(FINF, FINF);
        }
        if *x == FNINF {
            return CoshSinh::Pair(FINF, FNINF);
        }
        return CoshSinh::Pair(FNAN, FNAN);
    }
    let mag = exp + bc;
    let mut wp = prec + 14;
    if mag < -4 {
        // Extremely close to 0, sinh(x) ~= x and cosh(x) ~= 1
        if mag < -wp {
            if tanh {
                return CoshSinh::Tanh(mpf_perturb(x, 1 - sign, prec, rnd));
            }
            let cosh = mpf_perturb(&FONE, 0, prec, rnd);
            let sinh = mpf_perturb(x, sign, prec, rnd);
            return CoshSinh::Pair(cosh, sinh);
        }
        // Fix for cancellation when computing sinh
        wp += -mag;
    }
    // Does exp(-2*x) vanish?
    if mag > 10 {
        // 3*(1<<(mag-1)) > wp
        let vanishes = mag > 62 || 3 * (1i128 << (mag - 1)) > wp as i128;
        if vanishes {
            // XXX: rounding
            if tanh {
                let one = if sign == 0 { FONE } else { FNONE };
                return CoshSinh::Tanh(mpf_perturb(&one, 1 - sign, prec, rnd));
            }
            let c = mpf_shift(&mpf_exp(&mpf_abs(x, 0, ROUND_FAST), prec, rnd), -1);
            let mut s = c.clone();
            if sign == 1 {
                s = mpf_neg(&s, 0, ROUND_FAST);
            }
            return CoshSinh::Pair(c, s);
        }
    }
    // |x| > 1
    let (t, n) = if mag > 1 {
        let wpmod = wp + mag;
        let offset = exp + wpmod;
        let t = lshift(man, offset);
        let lg2 = ln2_fixed(wpmod);
        let (n, t) = t.div_mod(&lg2).expect("ln2 > 0");
        (sh(&t, mag), small(&n, "cosh_sinh: quotient by ln 2"))
    } else {
        let offset = exp + wp;
        (lshift(man, offset), 0)
    };
    let (a, b) = exp_expneg_basecase(&t, wp);
    // TODO: optimize division precision
    let cosh = &a + &sh(&b, 2 * n);
    let mut sinh = &a - &sh(&b, 2 * n);
    if sign == 1 {
        sinh = -sinh;
    }
    if tanh {
        let man = floordiv(&shl(&sinh, wp), &cosh);
        return CoshSinh::Tanh(from_man_exp(&man, -wp, prec, rnd));
    }
    let cosh = from_man_exp(&cosh, n - wp - 1, prec, rnd);
    let sinh = from_man_exp(&sinh, n - wp - 1, prec, rnd);
    CoshSinh::Pair(cosh, sinh)
}

/// `mod_pi2(man, exp, mag, wp)`: reduces `man * 2**exp` modulo pi/2,
/// returning `(t, n, wp)`.
pub fn mod_pi2(man: &Int, exp: i64, mag: i64, wp: i64) -> (Int, Int, i64) {
    if mag > 0 {
        let mut i = 0;
        loop {
            let cancellation_prec = 20i64 << i;
            let wpmod = wp + mag + cancellation_prec;
            let pi2 = pi_fixed(wpmod - 1);
            let pi4 = &pi2 >> 1;
            let offset = wpmod + exp;
            let t = lshift(man, offset);
            let (n, y) = t.div_mod(&pi2).expect("pi > 0");
            let small_ = if y > pi4 { &pi2 - &y } else { y.clone() };
            if !sh(&small_, wp + mag - 10).is_zero() {
                let t = sh(&y, mag);
                let wp = wpmod - mag;
                return (t, n, wp);
            }
            i += 1;
        }
    } else {
        let wp = wp + -mag;
        let offset = exp + wp;
        (lshift(man, offset), Int::ZERO, wp)
    }
}

/// What `mpf_cos_sin` returns for its `which` argument.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Which {
    CosSin,
    Cos,
    Sin,
    Tan,
}

enum CosSin {
    Pair(Mpf, Mpf),
    One(Mpf),
}

impl CosSin {
    fn pair(self) -> (Mpf, Mpf) {
        match self {
            CosSin::Pair(c, s) => (c, s),
            CosSin::One(_) => unreachable!("which=0 gives a pair"),
        }
    }

    fn one(self) -> Mpf {
        match self {
            CosSin::One(v) => v,
            CosSin::Pair(..) => unreachable!("which>0 gives one value"),
        }
    }
}

fn select(which: Which, c: Mpf, s: Mpf) -> CosSin {
    match which {
        Which::CosSin => CosSin::Pair(c, s),
        Which::Cos => CosSin::One(c),
        Which::Sin | Which::Tan => CosSin::One(s),
    }
}

/// `mpf_cos_sin(x, prec, rnd, which, pi)`.
fn cos_sin_impl(x: &Mpf, prec: i64, rnd: Rnd, which: Which, pi: bool) -> MpfResult<CosSin> {
    let (sign, exp, bc) = (x.sign, x.exp, x.bc);
    let mut man = x.man.clone();
    if man.is_zero() {
        let (c, s) = if exp != 0 {
            (FNAN, FNAN)
        } else {
            (FONE, FZERO)
        };
        return Ok(select(which, c, s));
    }
    let mag = bc + exp;
    let mut wp = prec + 10;
    // Extremely small?
    if mag < 0 && mag < -wp {
        let x = if pi {
            mpf_mul(x, &mpf_pi(wp, ROUND_FAST), 0, ROUND_FAST)
        } else {
            x.clone()
        };
        let c = mpf_perturb(&FONE, 1, prec, rnd);
        let s = mpf_perturb(&x, 1 - sign, prec, rnd);
        return Ok(match which {
            Which::CosSin => CosSin::Pair(c, s),
            Which::Cos => CosSin::One(c),
            Which::Sin => CosSin::One(s),
            Which::Tan => CosSin::One(mpf_perturb(&x, sign, prec, rnd)),
        });
    }
    let (t, n) = if pi {
        if exp >= -1 {
            let (c, s) = if exp == -1 {
                let neg = man.bit(1) ^ (sign == 1);
                (FZERO, if neg { FNONE } else { FONE })
            } else if exp == 0 {
                (FNONE, FZERO)
            } else {
                (FONE, FZERO)
            };
            return Ok(match which {
                Which::Tan => CosSin::One(mpf_div(&s, &c, prec, rnd)?),
                _ => select(which, c, s),
            });
        }
        // Subtract nearest half-integer (= mod by pi/2)
        let n = &sh(&man, -exp - 2) + 1;
        let n = &n >> 1;
        man = &man - &shl(&n, -exp - 1);
        let mag2 = bitcount(&man) + exp;
        wp = prec + 10 - mag2;
        let offset = exp + wp;
        let t = lshift(&man, offset);
        let t = sh(&(&t * &pi_fixed(wp)), wp);
        (t, n)
    } else {
        let (t, n, w) = mod_pi2(&man, exp, mag, wp);
        wp = w;
        (t, n)
    };
    let (mut c, mut s) = cos_sin_basecase(&t, wp);
    match and3(&n) {
        1 => (c, s) = (-s, c),
        2 => (c, s) = (-c, -s),
        3 => (c, s) = (s, -c),
        _ => {}
    }
    if sign == 1 {
        s = -s;
    }
    Ok(match which {
        Which::CosSin => CosSin::Pair(
            from_man_exp(&c, -wp, prec, rnd),
            from_man_exp(&s, -wp, prec, rnd),
        ),
        Which::Cos => CosSin::One(from_man_exp(&c, -wp, prec, rnd)),
        Which::Sin => CosSin::One(from_man_exp(&s, -wp, prec, rnd)),
        Which::Tan => CosSin::One(from_rational(&s, &c, prec, rnd)?),
    })
}

/// `mpf_cos_sin(x, prec, rnd)`: `(cos(x), sin(x))`.
pub fn mpf_cos_sin(x: &Mpf, prec: i64, rnd: Rnd) -> (Mpf, Mpf) {
    cos_sin_impl(x, prec, rnd, Which::CosSin, false)
        .expect("only tan divides")
        .pair()
}

/// `mpf_cos(x, prec, rnd)`.
pub fn mpf_cos(x: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    cos_sin_impl(x, prec, rnd, Which::Cos, false)
        .expect("only tan divides")
        .one()
}

/// `mpf_sin(x, prec, rnd)`.
pub fn mpf_sin(x: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    cos_sin_impl(x, prec, rnd, Which::Sin, false)
        .expect("only tan divides")
        .one()
}

/// `mpf_tan(x, prec, rnd)`.
pub fn mpf_tan(x: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    Ok(cos_sin_impl(x, prec, rnd, Which::Tan, false)?.one())
}

/// `mpf_cos_sin_pi(x, prec, rnd)`: `(cos(pi*x), sin(pi*x))`.
pub fn mpf_cos_sin_pi(x: &Mpf, prec: i64, rnd: Rnd) -> (Mpf, Mpf) {
    cos_sin_impl(x, prec, rnd, Which::CosSin, true)
        .expect("only tan divides")
        .pair()
}

/// `mpf_cos_pi(x, prec, rnd)`.
pub fn mpf_cos_pi(x: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    cos_sin_impl(x, prec, rnd, Which::Cos, true)
        .expect("only tan divides")
        .one()
}

/// `mpf_sin_pi(x, prec, rnd)`.
pub fn mpf_sin_pi(x: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    cos_sin_impl(x, prec, rnd, Which::Sin, true)
        .expect("only tan divides")
        .one()
}

/// `mpf_cos_sin(x, prec, rnd, which=3, pi=True)`: `tan(pi*x)`, which
/// mpmath reaches only by calling `mpf_cos_sin` directly.
pub fn mpf_tan_pi(x: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    Ok(cos_sin_impl(x, prec, rnd, Which::Tan, true)?.one())
}

// Low-overhead fixed-point versions

/// `cos_sin_fixed(x, prec, pi2=None)`.
pub fn cos_sin_fixed(x: &Int, prec: i64, pi2: Option<&Int>) -> (Int, Int) {
    let pi2 = match pi2 {
        Some(p) => p.clone(),
        None => pi_fixed(prec - 1),
    };
    let (n, t) = x.div_mod(&pi2).expect("pi > 0");
    let (c, s) = cos_sin_basecase(&t, prec);
    match and3(&n) {
        0 => (c, s),
        1 => (-s, c),
        2 => (-c, -s),
        _ => (s, -c),
    }
}

/// `exp_fixed(x, prec, ln2=None)`.
pub fn exp_fixed(x: &Int, prec: i64, ln2: Option<&Int>) -> Int {
    let ln2 = match ln2 {
        Some(l) => l.clone(),
        None => ln2_fixed(prec),
    };
    let (n, t) = x.div_mod(&ln2).expect("ln2 > 0");
    let n = small(&n, "exp_fixed: quotient by ln 2");
    let v = exp_basecase(&t, prec);
    if n >= 0 { shl(&v, n) } else { sh(&v, -n) }
}
