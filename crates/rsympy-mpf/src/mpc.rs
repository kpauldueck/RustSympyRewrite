//! Port of `mpmath.libmp.libmpc`: complex numbers as `(real, imag)` pairs
//! of raw mpfs, with arithmetic, powers and roots, exp and log, and the
//! trigonometric and hyperbolic functions and their inverses.

use rsympy_num::Int;

use crate::constants::{mpf_phi, mpf_pi};
use crate::elefun::*;
use crate::intmath::{giant_steps, lshift, rshift};
use crate::mpf::*;
use crate::pyfloat::complex_pow_float;

/// An mpc value: `(real, imag)`.
pub type Mpc = (Mpf, Mpf);

pub const MPC_ONE: Mpc = (FONE, FZERO);
pub const MPC_ZERO: Mpc = (FZERO, FZERO);
pub const MPC_TWO: Mpc = (FTWO, FZERO);
pub const MPC_HALF: Mpc = (FHALF, FZERO);

fn is_inf(x: &Mpf) -> bool {
    *x == FINF || *x == FNINF
}

fn floordiv(a: &Int, b: &Int) -> MpfResult<Int> {
    a.floor_div(b).map_err(|_| MpfError::ZeroDivision)
}

/// `mpc_is_inf(z)`: either part is infinite.
pub fn mpc_is_inf(z: &Mpc) -> bool {
    is_inf(&z.0) || is_inf(&z.1)
}

/// `mpc_is_infnan(z)`: either part is infinite or nan.
pub fn mpc_is_infnan(z: &Mpc) -> bool {
    is_inf(&z.0) || z.0 == FNAN || is_inf(&z.1) || z.1 == FNAN
}

/// `mpc_to_str(z, dps, **kwargs)`; the keyword arguments only reach the
/// imaginary part, as in mpmath.
pub fn mpc_to_str(
    z: &Mpc,
    dps: i64,
    strip_zeros: bool,
    min_fixed: Option<i64>,
    max_fixed: Option<i64>,
    show_zero_exponent: bool,
) -> String {
    let (re, im) = z;
    let rs = to_str(re, dps, true, None, None, false);
    if im.sign == 1 {
        format!(
            "{rs} - {}j",
            to_str(
                &mpf_neg(im, 0, ROUND_FAST),
                dps,
                strip_zeros,
                min_fixed,
                max_fixed,
                show_zero_exponent
            )
        )
    } else {
        format!(
            "{rs} + {}j",
            to_str(
                im,
                dps,
                strip_zeros,
                min_fixed,
                max_fixed,
                show_zero_exponent
            )
        )
    }
}

/// `mpc_to_complex(z, strict, rnd)`; `None` where Python raises
/// `OverflowError`.
pub fn mpc_to_complex(z: &Mpc, strict: bool, rnd: Rnd) -> Option<(f64, f64)> {
    Some((to_float(&z.0, strict, rnd)?, to_float(&z.1, strict, rnd)?))
}

/// `mpc_hash(z)`: `(hash(re) + 1000003*hash(im)) % 2**64`.
pub fn mpc_hash(z: &Mpc) -> u64 {
    let h = i128::from(mpf_hash(&z.0)) + 1_000_003 * i128::from(mpf_hash(&z.1));
    h.rem_euclid(1i128 << 64) as u64
}

/// `mpc_conjugate(z, prec, rnd)`.
pub fn mpc_conjugate(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    (z.0.clone(), mpf_neg(&z.1, prec, rnd))
}

/// `mpc_is_nonzero(z)`.
pub fn mpc_is_nonzero(z: &Mpc) -> bool {
    *z != MPC_ZERO
}

/// `mpc_add(z, w, prec, rnd)`.
pub fn mpc_add(z: &Mpc, w: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    (
        mpf_add(&z.0, &w.0, prec, rnd),
        mpf_add(&z.1, &w.1, prec, rnd),
    )
}

/// `mpc_add_mpf(z, x, prec, rnd)`: the imaginary part is kept as is.
pub fn mpc_add_mpf(z: &Mpc, x: &Mpf, prec: i64, rnd: Rnd) -> Mpc {
    (mpf_add(&z.0, x, prec, rnd), z.1.clone())
}

/// `mpc_sub(z, w, prec, rnd)`.
pub fn mpc_sub(z: &Mpc, w: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    (
        mpf_sub(&z.0, &w.0, prec, rnd),
        mpf_sub(&z.1, &w.1, prec, rnd),
    )
}

/// `mpc_sub_mpf(z, p, prec, rnd)`.
pub fn mpc_sub_mpf(z: &Mpc, p: &Mpf, prec: i64, rnd: Rnd) -> Mpc {
    (mpf_sub(&z.0, p, prec, rnd), z.1.clone())
}

/// `mpc_pos(z, prec, rnd)`.
pub fn mpc_pos(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    (mpf_pos(&z.0, prec, rnd), mpf_pos(&z.1, prec, rnd))
}

/// `mpc_neg(z, prec, rnd)`; `prec = 0` negates exactly.
pub fn mpc_neg(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    (mpf_neg(&z.0, prec, rnd), mpf_neg(&z.1, prec, rnd))
}

/// `mpc_shift(z, n)`.
pub fn mpc_shift(z: &Mpc, n: i64) -> Mpc {
    (mpf_shift(&z.0, n), mpf_shift(&z.1, n))
}

/// `mpc_abs(z, prec, rnd)`: `|a+bi|`, an mpf.
pub fn mpc_abs(z: &Mpc, prec: i64, rnd: Rnd) -> Mpf {
    mpf_hypot(&z.0, &z.1, prec, rnd).expect("a sum of squares is not negative")
}

/// `mpc_arg(z, prec, rnd)`: the argument, an mpf.
pub fn mpc_arg(z: &Mpc, prec: i64, rnd: Rnd) -> Mpf {
    mpf_atan2(&z.1, &z.0, prec, rnd)
}

/// `mpc_floor(z, prec, rnd)`.
pub fn mpc_floor(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    (mpf_floor(&z.0, prec, rnd), mpf_floor(&z.1, prec, rnd))
}

/// `mpc_ceil(z, prec, rnd)`.
pub fn mpc_ceil(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    (mpf_ceil(&z.0, prec, rnd), mpf_ceil(&z.1, prec, rnd))
}

/// `mpc_nint(z, prec, rnd)`.
pub fn mpc_nint(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    (mpf_nint(&z.0, prec, rnd), mpf_nint(&z.1, prec, rnd))
}

/// `mpc_frac(z, prec, rnd)`.
pub fn mpc_frac(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    (mpf_frac(&z.0, prec, rnd), mpf_frac(&z.1, prec, rnd))
}

/// `mpc_mul(z, w, prec, rnd)`: the rounding applies to each part.
pub fn mpc_mul(z: &Mpc, w: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    let (a, b) = z;
    let (c, d) = w;
    let p = mpf_mul(a, c, 0, ROUND_FAST);
    let q = mpf_mul(b, d, 0, ROUND_FAST);
    let r = mpf_mul(a, d, 0, ROUND_FAST);
    let s = mpf_mul(b, c, 0, ROUND_FAST);
    let re = mpf_sub(&p, &q, prec, rnd);
    let im = mpf_add(&r, &s, prec, rnd);
    (re, im)
}

/// `mpc_square(z, prec, rnd)`.
pub fn mpc_square(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    // (a+b*I)**2 == a**2 - b**2 + 2*I*a*b
    let (a, b) = z;
    let p = mpf_mul(a, a, 0, ROUND_FAST);
    let q = mpf_mul(b, b, 0, ROUND_FAST);
    let r = mpf_mul(a, b, prec, rnd);
    let re = mpf_sub(&p, &q, prec, rnd);
    let im = mpf_shift(&r, 1);
    (re, im)
}

/// `mpc_mul_mpf(z, p, prec, rnd)`.
pub fn mpc_mul_mpf(z: &Mpc, p: &Mpf, prec: i64, rnd: Rnd) -> Mpc {
    (mpf_mul(&z.0, p, prec, rnd), mpf_mul(&z.1, p, prec, rnd))
}

/// `mpc_mul_imag_mpf(z, x, prec, rnd)`: `z * I*x`.
pub fn mpc_mul_imag_mpf(z: &Mpc, x: &Mpf, prec: i64, rnd: Rnd) -> Mpc {
    let re = mpf_neg(&mpf_mul(&z.1, x, prec, rnd), 0, ROUND_FAST);
    let im = mpf_mul(&z.0, x, prec, rnd);
    (re, im)
}

/// `mpc_mul_int(z, n, prec, rnd)`.
pub fn mpc_mul_int(z: &Mpc, n: &Int, prec: i64, rnd: Rnd) -> Mpc {
    (
        mpf_mul_int(&z.0, n, prec, rnd),
        mpf_mul_int(&z.1, n, prec, rnd),
    )
}

/// `mpc_div(z, w, prec, rnd)`.
pub fn mpc_div(z: &Mpc, w: &Mpc, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    let (a, b) = z;
    let (c, d) = w;
    let wp = prec + 10;
    // mag = c*c + d*d
    let mag = mpf_add(
        &mpf_mul(c, c, 0, ROUND_FAST),
        &mpf_mul(d, d, 0, ROUND_FAST),
        wp,
        ROUND_FAST,
    );
    // (a*c+b*d)/mag, (b*c-a*d)/mag
    let t = mpf_add(
        &mpf_mul(a, c, 0, ROUND_FAST),
        &mpf_mul(b, d, 0, ROUND_FAST),
        wp,
        ROUND_FAST,
    );
    let u = mpf_sub(
        &mpf_mul(b, c, 0, ROUND_FAST),
        &mpf_mul(a, d, 0, ROUND_FAST),
        wp,
        ROUND_FAST,
    );
    Ok((mpf_div(&t, &mag, prec, rnd)?, mpf_div(&u, &mag, prec, rnd)?))
}

/// `mpc_div_mpf(z, p, prec, rnd)`: `z/p` for a real `p`.
pub fn mpc_div_mpf(z: &Mpc, p: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    Ok((mpf_div(&z.0, p, prec, rnd)?, mpf_div(&z.1, p, prec, rnd)?))
}

/// `mpc_reciprocal(z, prec, rnd)`: `1/z`.
pub fn mpc_reciprocal(z: &Mpc, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    let (a, b) = z;
    let m = mpf_add(
        &mpf_mul(a, a, 0, ROUND_FAST),
        &mpf_mul(b, b, 0, ROUND_FAST),
        prec + 10,
        ROUND_FAST,
    );
    let re = mpf_div(a, &m, prec, rnd)?;
    let im = mpf_neg(&mpf_div(b, &m, prec, rnd)?, 0, ROUND_FAST);
    Ok((re, im))
}

/// `mpc_mpf_div(p, z, prec, rnd)`: `p/z` for a real `p`.
pub fn mpc_mpf_div(p: &Mpf, z: &Mpc, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    let (a, b) = z;
    let m = mpf_add(
        &mpf_mul(a, a, 0, ROUND_FAST),
        &mpf_mul(b, b, 0, ROUND_FAST),
        prec + 10,
        ROUND_FAST,
    );
    let re = mpf_div(&mpf_mul(a, p, 0, ROUND_FAST), &m, prec, rnd)?;
    let im = mpf_div(
        &mpf_neg(&mpf_mul(b, p, 0, ROUND_FAST), 0, ROUND_FAST),
        &m,
        prec,
        rnd,
    )?;
    Ok((re, im))
}

/// `complex_int_pow(a, b, n)`: `(a+b*I)**n` exactly for `n >= 0`.
pub fn complex_int_pow(a: &Int, b: &Int, n: &Int) -> (Int, Int) {
    let (mut a, mut b) = (a.clone(), b.clone());
    let mut n = n.clone();
    let mut wre = Int::ONE;
    let mut wim = Int::ZERO;
    while !n.is_zero() {
        if n.is_odd() {
            let nre = &(&wre * &a) - &(&wim * &b);
            let nim = &(&wim * &a) + &(&wre * &b);
            wre = nre;
            wim = nim;
            n = &n - 1;
        }
        let na = &(&a * &a) - &(&b * &b);
        let nb = &(&a * &b) << 1;
        a = na;
        b = nb;
        n = &n >> 1;
    }
    (wre, wim)
}

/// `mpc_pow(z, w, prec, rnd)`.
pub fn mpc_pow(z: &Mpc, w: &Mpc, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    if w.1 == FZERO {
        return mpc_pow_mpf(z, &w.0, prec, rnd);
    }
    let l = mpc_log(z, prec + 10, ROUND_FAST);
    Ok(mpc_exp(&mpc_mul(&l, w, prec + 10, ROUND_FAST), prec, rnd))
}

/// `mpc_pow_mpf(z, p, prec, rnd)`.
pub fn mpc_pow_mpf(z: &Mpc, p: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    if p.exp >= 0 {
        let mut n = &p.man << p.exp as u64;
        if p.sign == 1 {
            n = -n;
        }
        return mpc_pow_int(z, &n, prec, rnd);
    }
    if p.exp == -1 {
        let sqrtz = mpc_sqrt(z, prec + 10, ROUND_FAST);
        let n = if p.sign == 1 { -&p.man } else { p.man.clone() };
        return mpc_pow_int(&sqrtz, &n, prec, rnd);
    }
    let l = mpc_log(z, prec + 10, ROUND_FAST);
    Ok(mpc_exp(
        &mpc_mul_mpf(&l, p, prec + 10, ROUND_FAST),
        prec,
        rnd,
    ))
}

/// `mpc_pow_int(z, n, prec, rnd)`.
pub fn mpc_pow_int(z: &Mpc, n: &Int, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    let (a, b) = z;
    if *b == FZERO {
        return Ok((mpf_pow_int(a, n, prec, rnd)?, FZERO));
    }
    if *a == FZERO {
        let v = mpf_pow_int(b, n, prec, rnd)?;
        let n4 = n.py_mod(&Int::from(4)).expect("4 != 0");
        return Ok(match n4.as_i64() {
            Some(0) => (v, FZERO),
            Some(1) => (FZERO, v),
            Some(2) => (mpf_neg(&v, 0, ROUND_FAST), FZERO),
            _ => (FZERO, mpf_neg(&v, 0, ROUND_FAST)),
        });
    }
    if n.is_zero() {
        return Ok(MPC_ONE);
    }
    if n.is_one() {
        return Ok(mpc_pos(z, prec, rnd));
    }
    if *n == 2 {
        return Ok(mpc_square(z, prec, rnd));
    }
    if *n == -1 {
        return mpc_reciprocal(z, prec, rnd);
    }
    if n.is_negative() {
        return mpc_reciprocal(&mpc_pow_int(z, &-n, prec + 4, ROUND_FAST)?, prec, rnd);
    }
    let (mut aman, mut aexp, abc) = (a.man.clone(), a.exp, a.bc);
    let (mut bman, mut bexp, bbc) = (b.man.clone(), b.exp, b.bc);
    if a.sign == 1 {
        aman = -aman;
    }
    if b.sign == 1 {
        bman = -bman;
    }
    let de = aexp - bexp;
    let abs_de = de.abs();
    let exact_size = n * &Int::from(abs_de + abc.max(bbc));
    if exact_size < 10000 {
        if de > 0 {
            aman = &aman << de as u64;
            aexp = bexp;
        } else {
            bman = &bman << (-de) as u64;
            bexp = aexp;
        }
        let (re, im) = complex_int_pow(&aman, &bman, n);
        // int(n*aexp): only a zero mantissa (from inf/nan parts) can come
        // with an n too large for the exponent to fit in i64.
        let scaled = |man: &Int, e: i64| {
            if man.is_zero() {
                return FZERO;
            }
            let e = (n * &Int::from(e)).as_i64().expect("exponent fits in i64");
            from_man_exp(man, e, prec, rnd)
        };
        return Ok((scaled(&re, aexp), scaled(&im, bexp)));
    }
    let l = mpc_log(z, prec + 10, ROUND_FAST);
    Ok(mpc_exp(
        &mpc_mul_int(&l, n, prec + 10, ROUND_FAST),
        prec,
        rnd,
    ))
}

/// `mpc_sqrt(z, prec, rnd)`: the principal square root.
pub fn mpc_sqrt(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    let (a, b) = z;
    if *b == FZERO {
        if *a == FZERO {
            return (a.clone(), b.clone());
        }
        // When a+bi is a negative real number, we get a real sqrt times i
        if a.sign == 1 {
            let im = mpf_sqrt(&mpf_neg(a, 0, ROUND_FAST), prec, rnd).expect("-a > 0");
            return (FZERO, im);
        }
        let re = mpf_sqrt(a, prec, rnd).expect("a > 0");
        return (re, FZERO);
    }
    let wp = prec + 20;
    let sqrt = |x: &Mpf, p: i64, r: Rnd| mpf_sqrt(x, p, r).expect("not negative");
    let div = |x: &Mpf, y: &Mpf, p: i64, r: Rnd| mpf_div(x, y, p, r).expect("w != 0");
    if a.sign == 0 {
        // case a positive
        let t = mpf_add(&mpc_abs(z, wp, ROUND_FAST), a, wp, ROUND_FAST); // t = abs(a+bi) + a
        let u = mpf_shift(&t, -1); // u = t/2
        let re = sqrt(&u, prec, rnd); // re = sqrt(u)
        let v = mpf_shift(&t, 1); // v = 2*t
        let w = sqrt(&v, wp, ROUND_FAST); // w = sqrt(v)
        let im = div(b, &w, prec, rnd); // im = b / w
        (re, im)
    } else {
        // case a negative
        let t = mpf_sub(&mpc_abs(z, wp, ROUND_FAST), a, wp, ROUND_FAST); // t = abs(a+bi) - a
        let u = mpf_shift(&t, -1); // u = t/2
        let mut im = sqrt(&u, prec, rnd); // im = sqrt(u)
        let v = mpf_shift(&t, 1); // v = 2*t
        let w = sqrt(&v, wp, ROUND_FAST); // w = sqrt(v)
        let mut re = div(b, &w, prec, rnd); // re = b/w
        if b.sign == 1 {
            re = mpf_neg(&re, 0, ROUND_FAST);
            im = mpf_neg(&im, 0, ROUND_FAST);
        }
        (re, im)
    }
}

/// `mpc_nthroot_fixed(a, b, n, prec)`: Newton's method for the n-th root of
/// the fixed-point complex number `a + b*I`.
pub fn mpc_nthroot_fixed(a: &Int, b: &Int, n: i64, prec: i64) -> MpfResult<(Int, Int)> {
    let start = 50;
    let a1 = rshift(a, prec - n * start);
    let b1 = rshift(b, prec - n * start);
    // r = (a1 + 1j * b1)**(1.0/n), and int() of its parts; OverflowError
    // (from the float conversions or the power) takes the mpc route.
    let est = match (a1.to_f64(), b1.to_f64()) {
        (Some(af), Some(bf)) => complex_pow_float(af, bf, 1.0 / n as f64).ok(),
        _ => None,
    };
    let est = match est {
        Some((re, im)) => {
            if re.is_nan() || im.is_nan() {
                return Err(MpfError::Value(
                    "cannot convert float NaN to integer".into(),
                ));
            }
            Some((
                Int::from_f64_trunc(re).expect("finite"),
                Int::from_f64_trunc(im).expect("finite"),
            ))
        }
        None => None,
    };
    let (mut re, mut im) = match est {
        Some(v) => v,
        None => {
            let a1 = from_int(&a1, start, ROUND_FAST);
            let b1 = from_int(&b1, start, ROUND_FAST);
            let fnn = from_int(&Int::from(n), 0, ROUND_FAST);
            let nth = mpf_rdiv_int(&Int::ONE, &fnn, start, ROUND_FAST)?;
            let (re, im) = mpc_pow(&(a1, b1), &(nth, FZERO), start, ROUND_FAST)?;
            (to_int(&re, None)?, to_int(&im, None)?)
        }
    };
    let extra = 10;
    let mut prevp = start;
    let extra1 = n;
    let nm1 = Int::from(n - 1);
    let nn = Int::from(n);
    for p in giant_steps(start, prec + extra, 2) {
        // this is slow for large n, unlike int_pow_fixed
        let (re2, im2) = complex_int_pow(&re, &im, &nm1);
        let re2 = rshift(&re2, (n - 1) * prevp - p - extra1);
        let im2 = rshift(&im2, (n - 1) * prevp - p - extra1);
        let r4 = rshift(&(&(&re2 * &re2) + &(&im2 * &im2)), p + extra1);
        let ap = rshift(a, prec - p);
        let bp = rshift(b, prec - p);
        let rec = rshift(&(&(&ap * &re2) + &(&bp * &im2)), p);
        let imc = rshift(&(&(&(-&ap) * &im2) + &(&bp * &re2)), p);
        let reb = floordiv(&lshift(&rec, p), &r4)?;
        let imb = floordiv(&lshift(&imc, p), &r4)?;
        re = floordiv(&(&reb + &(&nm1 * &lshift(&re, p - prevp))), &nn)?;
        im = floordiv(&(&imb + &(&nm1 * &lshift(&im, p - prevp))), &nn)?;
        prevp = p;
    }
    Ok((re, im))
}

/// `mpc_nthroot(z, n, prec, rnd)`: the principal n-th root.
pub fn mpc_nthroot(z: &Mpc, n: i64, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    let (a, b) = z;
    if a.sign == 0 && *b == FZERO {
        let re = mpf_nthroot(a, n, prec, rnd)?;
        return Ok((re, FZERO));
    }
    if n < 2 {
        if n == 0 {
            return Ok(MPC_ONE);
        }
        if n == 1 {
            return Ok(mpc_pos(z, prec, rnd));
        }
        if n == -1 {
            return mpc_div(&MPC_ONE, z, prec, rnd);
        }
        let inverse = mpc_nthroot(z, -n, prec + 5, rnd.reciprocal())?;
        return mpc_div(&MPC_ONE, &inverse, prec, rnd);
    }
    if n <= 20 {
        let prec2 = (1.2 * (prec + 10) as f64) as i64;
        let pf = mpc_abs(z, prec, ROUND_FAST);
        let m = pf.exp + pf.bc;
        if m > -10 && m < prec {
            let af = to_fixed(a, prec2);
            let bf = to_fixed(b, prec2);
            let (re, im) = mpc_nthroot_fixed(&af, &bf, n, prec2)?;
            let extra = 10;
            let re = from_man_exp(&re, -prec2 - extra, prec2, rnd);
            let im = from_man_exp(&im, -prec2 - extra, prec2, rnd);
            return Ok((re, im));
        }
    }
    let fnn = from_int(&Int::from(n), 0, ROUND_FAST);
    let prec2 = prec + 10 + 10;
    let nth = mpf_rdiv_int(&Int::ONE, &fnn, prec2, ROUND_FAST)?;
    let (re, im) = mpc_pow(z, &(nth, FZERO), prec2, rnd)?;
    let re = normalize(re.sign, re.man, re.exp, re.bc, prec, rnd);
    let im = normalize(im.sign, im.man, im.exp, im.bc, prec, rnd);
    Ok((re, im))
}

/// `mpc_cbrt(z, prec, rnd)`.
pub fn mpc_cbrt(z: &Mpc, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    mpc_nthroot(z, 3, prec, rnd)
}

/// `mpc_exp(z, prec, rnd)`: `exp(a) * (cos(b) + sin(b)*i)`.
pub fn mpc_exp(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    let (a, b) = z;
    if *a == FZERO {
        return mpf_cos_sin(b, prec, rnd);
    }
    if *b == FZERO {
        return (mpf_exp(a, prec, rnd), FZERO);
    }
    let mag = mpf_exp(a, prec + 4, rnd);
    let (c, s) = mpf_cos_sin(b, prec + 4, rnd);
    let re = mpf_mul(&mag, &c, prec, rnd);
    let im = mpf_mul(&mag, &s, prec, rnd);
    (re, im)
}

/// `mpc_log(z, prec, rnd)`.
pub fn mpc_log(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    let re = mpf_log_hypot(&z.0, &z.1, prec, rnd);
    let im = mpc_arg(z, prec, rnd);
    (re, im)
}

/// `mpc_cos(z, prec, rnd)`: `cos(a)*cosh(b) - sin(a)*sinh(b)*i`.
pub fn mpc_cos(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    let (a, b) = z;
    if *b == FZERO {
        return (mpf_cos(a, prec, rnd), FZERO);
    }
    if *a == FZERO {
        return (mpf_cosh(b, prec, rnd), FZERO);
    }
    let wp = prec + 6;
    let (c, s) = mpf_cos_sin(a, wp, ROUND_FAST);
    let (ch, sh) = mpf_cosh_sinh(b, wp, ROUND_FAST);
    let re = mpf_mul(&c, &ch, prec, rnd);
    let im = mpf_mul(&s, &sh, prec, rnd);
    (re, mpf_neg(&im, 0, ROUND_FAST))
}

/// `mpc_sin(z, prec, rnd)`: `sin(a)*cosh(b) + cos(a)*sinh(b)*i`.
pub fn mpc_sin(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    let (a, b) = z;
    if *b == FZERO {
        return (mpf_sin(a, prec, rnd), FZERO);
    }
    if *a == FZERO {
        return (FZERO, mpf_sinh(b, prec, rnd));
    }
    let wp = prec + 6;
    let (c, s) = mpf_cos_sin(a, wp, ROUND_FAST);
    let (ch, sh) = mpf_cosh_sinh(b, wp, ROUND_FAST);
    let re = mpf_mul(&s, &ch, prec, rnd);
    let im = mpf_mul(&c, &sh, prec, rnd);
    (re, im)
}

/// `mpc_tan(z, prec, rnd)`: `sin(2a)/M + sinh(2b)/M*i` with
/// `M = cos(2a) + cosh(2b)`.
pub fn mpc_tan(z: &Mpc, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    let (a, b) = z;
    if *b == FZERO {
        return Ok((mpf_tan(a, prec, rnd)?, FZERO));
    }
    if *a == FZERO {
        return Ok((FZERO, mpf_tanh(b, prec, rnd)));
    }
    let wp = prec + 15;
    let a = mpf_shift(a, 1);
    let b = mpf_shift(b, 1);
    let (c, s) = mpf_cos_sin(&a, wp, ROUND_FAST);
    let (ch, sh) = mpf_cosh_sinh(&b, wp, ROUND_FAST);
    // TODO: handle cancellation when c ~=  -1 and ch ~= 1
    let mag = mpf_add(&c, &ch, wp, ROUND_FAST);
    let re = mpf_div(&s, &mag, prec, rnd)?;
    let im = mpf_div(&sh, &mag, prec, rnd)?;
    Ok((re, im))
}

/// `mpc_cos_pi(z, prec, rnd)`: `cos(pi*z)`.
pub fn mpc_cos_pi(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    let (a, b) = z;
    if *b == FZERO {
        return (mpf_cos_pi(a, prec, rnd), FZERO);
    }
    let b = mpf_mul(b, &mpf_pi(prec + 5, ROUND_FAST), prec + 5, ROUND_FAST);
    if *a == FZERO {
        return (mpf_cosh(&b, prec, rnd), FZERO);
    }
    let wp = prec + 6;
    let (c, s) = mpf_cos_sin_pi(a, wp, ROUND_FAST);
    let (ch, sh) = mpf_cosh_sinh(&b, wp, ROUND_FAST);
    let re = mpf_mul(&c, &ch, prec, rnd);
    let im = mpf_mul(&s, &sh, prec, rnd);
    (re, mpf_neg(&im, 0, ROUND_FAST))
}

/// `mpc_sin_pi(z, prec, rnd)`: `sin(pi*z)`.
pub fn mpc_sin_pi(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    let (a, b) = z;
    if *b == FZERO {
        return (mpf_sin_pi(a, prec, rnd), FZERO);
    }
    let b = mpf_mul(b, &mpf_pi(prec + 5, ROUND_FAST), prec + 5, ROUND_FAST);
    if *a == FZERO {
        return (FZERO, mpf_sinh(&b, prec, rnd));
    }
    let wp = prec + 6;
    let (c, s) = mpf_cos_sin_pi(a, wp, ROUND_FAST);
    let (ch, sh) = mpf_cosh_sinh(&b, wp, ROUND_FAST);
    let re = mpf_mul(&s, &ch, prec, rnd);
    let im = mpf_mul(&c, &sh, prec, rnd);
    (re, im)
}

/// `mpc_cos_sin(z, prec, rnd)`: `(cos(z), sin(z))`.
pub fn mpc_cos_sin(z: &Mpc, prec: i64, rnd: Rnd) -> (Mpc, Mpc) {
    let (a, b) = z;
    if *a == FZERO {
        let (ch, sh) = mpf_cosh_sinh(b, prec, rnd);
        return ((ch, FZERO), (FZERO, sh));
    }
    if *b == FZERO {
        let (c, s) = mpf_cos_sin(a, prec, rnd);
        return ((c, FZERO), (s, FZERO));
    }
    let wp = prec + 6;
    let (c, s) = mpf_cos_sin(a, wp, ROUND_FAST);
    let (ch, sh) = mpf_cosh_sinh(b, wp, ROUND_FAST);
    let cre = mpf_mul(&c, &ch, prec, rnd);
    let cim = mpf_mul(&s, &sh, prec, rnd);
    let sre = mpf_mul(&s, &ch, prec, rnd);
    let sim = mpf_mul(&c, &sh, prec, rnd);
    ((cre, mpf_neg(&cim, 0, ROUND_FAST)), (sre, sim))
}

/// `mpc_cos_sin_pi(z, prec, rnd)`: `(cos(pi*z), sin(pi*z))`.
pub fn mpc_cos_sin_pi(z: &Mpc, prec: i64, rnd: Rnd) -> (Mpc, Mpc) {
    let (a, b) = z;
    if *b == FZERO {
        let (c, s) = mpf_cos_sin_pi(a, prec, rnd);
        return ((c, FZERO), (s, FZERO));
    }
    let b = mpf_mul(b, &mpf_pi(prec + 5, ROUND_FAST), prec + 5, ROUND_FAST);
    if *a == FZERO {
        let (ch, sh) = mpf_cosh_sinh(&b, prec, rnd);
        return ((ch, FZERO), (FZERO, sh));
    }
    let wp = prec + 6;
    let (c, s) = mpf_cos_sin_pi(a, wp, ROUND_FAST);
    let (ch, sh) = mpf_cosh_sinh(&b, wp, ROUND_FAST);
    let cre = mpf_mul(&c, &ch, prec, rnd);
    let cim = mpf_mul(&s, &sh, prec, rnd);
    let sre = mpf_mul(&s, &ch, prec, rnd);
    let sim = mpf_mul(&c, &sh, prec, rnd);
    ((cre, mpf_neg(&cim, 0, ROUND_FAST)), (sre, sim))
}

/// `mpc_cosh(z, prec, rnd)`: `cos(z*i)`.
pub fn mpc_cosh(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    let (a, b) = z;
    mpc_cos(&(b.clone(), mpf_neg(a, 0, ROUND_FAST)), prec, rnd)
}

/// `mpc_sinh(z, prec, rnd)`: `-i*sin(z*i)`.
pub fn mpc_sinh(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    let (a, b) = z;
    let (b, a) = mpc_sin(&(b.clone(), a.clone()), prec, rnd);
    (a, b)
}

/// `mpc_tanh(z, prec, rnd)`: `-i*tan(z*i)`.
pub fn mpc_tanh(z: &Mpc, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    let (a, b) = z;
    let (b, a) = mpc_tan(&(b.clone(), a.clone()), prec, rnd)?;
    Ok((a, b))
}

/// `mpc_atan(z, prec, rnd)`: `(I/2)*(log(1-I*z) - log(1+I*z))`.
pub fn mpc_atan(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    let (a, b) = z;
    // x = 1-I*z = 1 + b - I*a
    // y = 1+I*z = 1 - b + I*a
    let wp = prec + 15;
    let x = (mpf_add(&FONE, b, wp, ROUND_FAST), mpf_neg(a, 0, ROUND_FAST));
    let y = (mpf_sub(&FONE, b, wp, ROUND_FAST), a.clone());
    let l1 = mpc_log(&x, wp, ROUND_FAST);
    let l2 = mpc_log(&y, wp, ROUND_FAST);
    let (a, b) = mpc_sub(&l1, &l2, prec, rnd);
    // (I/2) * (a+b*I) = (-b/2 + a/2*I)
    let mut v = (
        mpf_neg(&mpf_shift(&b, -1), 0, ROUND_FAST),
        mpf_shift(&a, -1),
    );
    // Subtraction at infinity gives correct real part but
    // wrong imaginary part (should be zero)
    if v.1 == FNAN && mpc_is_inf(z) {
        v = (v.0, FZERO);
    }
    v
}

fn beta_crossover() -> Mpf {
    from_float(0.6417, 53, ROUND_FAST)
}

fn alpha_crossover() -> Mpf {
    from_float(1.5, 53, ROUND_FAST)
}

/// `acos_asin(z, prec, rnd, n)`: the complex acos (`n = 0`) or asin
/// (`n = 1`), after Hull, Fairgrieve and Tang.
pub fn acos_asin(z: &Mpc, prec: i64, rnd: Rnd, n: u8) -> MpfResult<Mpc> {
    let (a, b) = z;
    let wp = prec + 10;
    let neg = |x: &Mpf| mpf_neg(x, 0, ROUND_FAST);
    // special cases with real argument
    if *b == FZERO {
        let am = mpf_sub(&FONE, &mpf_abs(a, 0, ROUND_FAST), wp, ROUND_FAST);
        // case abs(a) <= 1
        if am.sign == 0 {
            if n == 0 {
                return Ok((mpf_acos(a, prec, rnd)?, FZERO));
            }
            return Ok((mpf_asin(a, prec, rnd)?, FZERO));
        }
        // cases abs(a) > 1
        if a.sign == 1 {
            // case a < -1
            let pi = mpf_pi(prec, rnd);
            let c = mpf_acosh(&neg(a), prec, rnd)?;
            if n == 0 {
                return Ok((pi, neg(&c)));
            }
            return Ok((neg(&mpf_shift(&pi, -1)), c));
        }
        // case a > 1
        let c = mpf_acosh(a, prec, rnd)?;
        if n == 0 {
            return Ok((FZERO, c));
        }
        let pi = mpf_pi(prec, rnd);
        return Ok((mpf_shift(&pi, -1), neg(&c)));
    }
    let mut asign = 0;
    let mut bsign = 0;
    let mut a = a.clone();
    let mut b = b.clone();
    if a.sign == 1 {
        a = neg(&a);
        asign = 1;
    }
    if b.sign == 1 {
        b = neg(&b);
        bsign = 1;
    }
    let add = |x: &Mpf, y: &Mpf| mpf_add(x, y, wp, ROUND_FAST);
    let sub = |x: &Mpf, y: &Mpf| mpf_sub(x, y, wp, ROUND_FAST);
    let mul = |x: &Mpf, y: &Mpf| mpf_mul(x, y, wp, ROUND_FAST);
    let div = |x: &Mpf, y: &Mpf| mpf_div(x, y, wp, ROUND_FAST);
    let sqrt = |x: &Mpf| mpf_sqrt(x, wp, ROUND_FAST);
    let am = sub(&FONE, &a);
    let ap = add(&FONE, &a);
    let r = mpf_hypot(&ap, &b, wp, ROUND_FAST)?;
    let s = mpf_hypot(&am, &b, wp, ROUND_FAST)?;
    let alpha = mpf_shift(&add(&r, &s), -1);
    let beta = div(&a, &alpha)?;
    let b2 = mul(&b, &b);
    let mut re;
    // case beta <= beta_crossover
    if sub(&beta_crossover(), &beta).sign == 0 {
        if n == 0 {
            re = mpf_acos(&beta, wp, ROUND_FAST)?;
        } else {
            re = mpf_asin(&beta, wp, ROUND_FAST)?;
        }
    } else {
        // to compute the real part in this region use the identity
        // asin(beta) = atan(beta/sqrt(1-beta**2))
        let ax = add(&alpha, &a);
        // case a <= 1
        if am.sign == 0 {
            let c = div(&b2, &add(&r, &ap))?;
            let d = add(&s, &am);
            re = mpf_shift(&mul(&ax, &add(&c, &d)), -1);
            if n == 0 {
                re = mpf_atan(&div(&sqrt(&re)?, &a)?, wp, ROUND_FAST);
            } else {
                re = mpf_atan(&div(&a, &sqrt(&re)?)?, wp, ROUND_FAST);
            }
        } else {
            let c = div(&ax, &add(&r, &ap))?;
            let d = div(&ax, &sub(&s, &am))?;
            re = mpf_shift(&add(&c, &d), -1);
            re = mul(&b, &sqrt(&re)?);
            if n == 0 {
                re = mpf_atan(&div(&re, &a)?, wp, ROUND_FAST);
            } else {
                re = mpf_atan(&div(&a, &re)?, wp, ROUND_FAST);
            }
        }
    }
    // to compute alpha + sqrt(alpha**2 - 1), if alpha <= alpha_crossover
    // replace it with 1 + Am1 + sqrt(Am1*(alpha+1)))
    let mut im;
    if sub(&alpha_crossover(), &alpha).sign == 0 {
        let c1 = div(&b2, &add(&r, &ap))?;
        // case a < 1
        let am1 = if neg(&am).sign == 1 {
            let c2 = add(&s, &am);
            let c2 = div(&b2, &c2)?;
            mpf_shift(&add(&c1, &c2), -1)
        } else {
            let c2 = sub(&s, &am);
            mpf_shift(&add(&c1, &c2), -1)
        };
        // im = log(1 + Am1 + sqrt(Am1*(alpha+1)))
        im = mul(&am1, &add(&alpha, &FONE));
        im = mpf_log(&add(&FONE, &add(&am1, &sqrt(&im)?)), wp, ROUND_FAST)?;
    } else {
        // im = log(alpha + sqrt(alpha*alpha - 1))
        im = sqrt(&sub(&mul(&alpha, &alpha), &FONE))?;
        im = mpf_log(&add(&alpha, &im), wp, ROUND_FAST)?;
    }
    if asign == 1 {
        if n == 0 {
            re = sub(&mpf_pi(wp, ROUND_FAST), &re);
        } else {
            re = neg(&re);
        }
    }
    if bsign == 0 && n == 0 {
        im = neg(&im);
    }
    if bsign == 1 && n == 1 {
        im = neg(&im);
    }
    let re = normalize(re.sign, re.man, re.exp, re.bc, prec, rnd);
    let im = normalize(im.sign, im.man, im.exp, im.bc, prec, rnd);
    Ok((re, im))
}

/// `mpc_acos(z, prec, rnd)`.
pub fn mpc_acos(z: &Mpc, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    acos_asin(z, prec, rnd, 0)
}

/// `mpc_asin(z, prec, rnd)`.
pub fn mpc_asin(z: &Mpc, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    acos_asin(z, prec, rnd, 1)
}

/// `mpc_asinh(z, prec, rnd)`: `I * asin(-I z)`.
pub fn mpc_asinh(z: &Mpc, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    let (a, b) = z;
    let (a, b) = mpc_asin(&(b.clone(), mpf_neg(a, 0, ROUND_FAST)), prec, rnd)?;
    Ok((mpf_neg(&b, 0, ROUND_FAST), a))
}

/// `mpc_acosh(z, prec, rnd)`.
pub fn mpc_acosh(z: &Mpc, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    // acosh(z) = -I * acos(z)   for Im(acos(z)) <= 0
    //            +I * acos(z)   otherwise
    let (a, b) = mpc_acos(z, prec, rnd)?;
    if b.sign == 1 || b == FZERO {
        Ok((mpf_neg(&b, 0, ROUND_FAST), a))
    } else {
        Ok((b, mpf_neg(&a, 0, ROUND_FAST)))
    }
}

/// `mpc_atanh(z, prec, rnd)`: `(log(1+z) - log(1-z))/2`.
pub fn mpc_atanh(z: &Mpc, prec: i64, _rnd: Rnd) -> Mpc {
    let wp = prec + 15;
    let a = mpc_add(z, &MPC_ONE, wp, ROUND_FAST);
    let b = mpc_sub(&MPC_ONE, z, wp, ROUND_FAST);
    let a = mpc_log(&a, wp, ROUND_FAST);
    let b = mpc_log(&b, wp, ROUND_FAST);
    let mut v = mpc_shift(&mpc_sub(&a, &b, wp, ROUND_FAST), -1);
    // Subtraction at infinity gives correct imaginary part but
    // wrong real part (should be zero)
    if v.0 == FNAN && mpc_is_inf(z) {
        v = (FZERO, v.1);
    }
    v
}

/// `mpc_fibonacci(z, prec, rnd)`.
pub fn mpc_fibonacci(z: &Mpc, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    let (re, im) = z;
    if *im == FZERO {
        return Ok((mpf_fibonacci(re, prec, rnd), FZERO));
    }
    let size = (re.exp + re.bc).abs().max((re.exp + re.bc).abs());
    let wp = prec + size + 20;
    let a = mpf_phi(wp, ROUND_FAST);
    let b = mpf_add(&mpf_shift(&a, 1), &FNONE, wp, ROUND_FAST);
    let u = mpc_pow(&(a, FZERO), z, wp, ROUND_FAST)?;
    let v = mpc_cos_pi(z, wp, ROUND_FAST);
    let v = mpc_div(&v, &u, wp, ROUND_FAST)?;
    let u = mpc_sub(&u, &v, wp, ROUND_FAST);
    mpc_div_mpf(&u, &b, prec, rnd)
}

/// `mpf_expj(x, prec, rnd)`: always `ComplexResult`.
pub fn mpf_expj(_x: &Mpf, _prec: i64, _rnd: Rnd) -> MpfResult<Mpf> {
    Err(MpfError::ComplexResult(""))
}

/// `mpc_expj(z, prec, rnd)`: `exp(i*z)`.
pub fn mpc_expj(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    let (re, im) = z;
    if *im == FZERO {
        return mpf_cos_sin(re, prec, rnd);
    }
    if *re == FZERO {
        return (mpf_exp(&mpf_neg(im, 0, ROUND_FAST), prec, rnd), FZERO);
    }
    let ey = mpf_exp(&mpf_neg(im, 0, ROUND_FAST), prec + 10, ROUND_FAST);
    let (c, s) = mpf_cos_sin(re, prec + 10, ROUND_FAST);
    let re = mpf_mul(&ey, &c, prec, rnd);
    let im = mpf_mul(&ey, &s, prec, rnd);
    (re, im)
}

/// `mpf_expjpi(x, prec, rnd)`: always `ComplexResult`.
pub fn mpf_expjpi(_x: &Mpf, _prec: i64, _rnd: Rnd) -> MpfResult<Mpf> {
    Err(MpfError::ComplexResult(""))
}

/// `mpc_expjpi(z, prec, rnd)`: `exp(i*pi*z)`.
pub fn mpc_expjpi(z: &Mpc, prec: i64, rnd: Rnd) -> Mpc {
    let (re, im) = z;
    if *im == FZERO {
        return mpf_cos_sin_pi(re, prec, rnd);
    }
    let mut wp = prec + 10;
    if !im.man.is_zero() {
        wp += 0.max(im.exp + im.bc);
    }
    let im = mpf_neg(
        &mpf_mul(&mpf_pi(wp, ROUND_FAST), im, wp, ROUND_FAST),
        0,
        ROUND_FAST,
    );
    if *re == FZERO {
        return (mpf_exp(&im, prec, rnd), FZERO);
    }
    let ey = mpf_exp(&im, prec + 10, ROUND_FAST);
    let (c, s) = mpf_cos_sin_pi(re, prec + 10, ROUND_FAST);
    let re = mpf_mul(&ey, &c, prec, rnd);
    let im = mpf_mul(&ey, &s, prec, rnd);
    (re, im)
}
