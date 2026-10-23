//! Port of `mpmath.libmp.libmpf`: binary floating-point numbers
//! `(-1)**sign * man * 2**exp` with an explicit precision and rounding mode
//! on every operation.
//!
//! The representation is mpmath's raw tuple `(sign, man, exp, bc)`, with
//! the same encodings for zero, infinities and NaN, so that structural
//! equality is mpmath's tuple equality.

use std::fmt;

use rsympy_num::Int;

use crate::constants::{mpf_ln2, mpf_ln10};
use crate::intmath::{bin_to_radix, bitcount, isqrt, numeral, sqrtrem};

/// mpmath's rounding modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rnd {
    /// `round_nearest`: to nearest, ties to even.
    Nearest,
    /// `round_floor`: toward -inf.
    Floor,
    /// `round_ceiling`: toward +inf.
    Ceiling,
    /// `round_up`: away from zero.
    Up,
    /// `round_down`: toward zero (mpmath's `round_fast`).
    Down,
}

/// `round_fast`.
pub const ROUND_FAST: Rnd = Rnd::Down;

impl Rnd {
    /// `shifts_down[rnd][sign]`: whether a plain right shift (floor of the
    /// magnitude) rounds in this direction for this sign.
    fn shifts_down(self, sign: u8) -> bool {
        match self {
            Rnd::Floor => sign == 0,
            Rnd::Ceiling => sign == 1,
            Rnd::Down => true,
            Rnd::Up => false,
            Rnd::Nearest => unreachable!("nearest is handled separately"),
        }
    }

    /// `reciprocal_rnd[rnd]`.
    pub fn reciprocal(self) -> Rnd {
        match self {
            Rnd::Down => Rnd::Up,
            Rnd::Up => Rnd::Down,
            Rnd::Floor => Rnd::Ceiling,
            Rnd::Ceiling => Rnd::Floor,
            Rnd::Nearest => Rnd::Nearest,
        }
    }

    /// `negative_rnd[rnd]`.
    pub fn negative(self) -> Rnd {
        match self {
            Rnd::Down => Rnd::Down,
            Rnd::Up => Rnd::Up,
            Rnd::Floor => Rnd::Ceiling,
            Rnd::Ceiling => Rnd::Floor,
            Rnd::Nearest => Rnd::Nearest,
        }
    }
}

/// Errors mpmath raises from these functions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MpfError {
    /// Python `ZeroDivisionError`.
    ZeroDivision,
    /// mpmath's `ComplexResult` (a `ValueError`).
    ComplexResult(&'static str),
    /// Python `ValueError`.
    Value(String),
    /// Python `NotImplementedError`.
    NotImplemented,
}

impl MpfError {
    /// The Python exception class.
    pub fn class(&self) -> &'static str {
        match self {
            MpfError::ZeroDivision => "ZeroDivisionError",
            MpfError::ComplexResult(_) => "ComplexResult",
            MpfError::Value(_) => "ValueError",
            MpfError::NotImplemented => "NotImplementedError",
        }
    }
}

impl fmt::Display for MpfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MpfError::ZeroDivision => f.write_str("ZeroDivisionError"),
            MpfError::ComplexResult(m) => write!(f, "ComplexResult: {m}"),
            MpfError::Value(m) => write!(f, "ValueError: {m}"),
            MpfError::NotImplemented => f.write_str("NotImplementedError"),
        }
    }
}

impl std::error::Error for MpfError {}

pub type MpfResult<T> = Result<T, MpfError>;

/// A raw mpf: `(sign, man, exp, bc)`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Mpf {
    pub sign: u8,
    pub man: Int,
    pub exp: i64,
    pub bc: i64,
}

const fn raw(sign: u8, man: i64, exp: i64, bc: i64) -> Mpf {
    Mpf {
        sign,
        man: Int::small(man),
        exp,
        bc,
    }
}

pub const FZERO: Mpf = raw(0, 0, 0, 0);
pub const FNZERO: Mpf = raw(1, 0, 0, 0);
pub const FONE: Mpf = raw(0, 1, 0, 1);
pub const FNONE: Mpf = raw(1, 1, 0, 1);
pub const FTWO: Mpf = raw(0, 1, 1, 1);
pub const FTEN: Mpf = raw(0, 5, 1, 3);
pub const FHALF: Mpf = raw(0, 1, -1, 1);
pub const FNAN: Mpf = raw(0, 0, -123, -1);
pub const FINF: Mpf = raw(0, 0, -456, -2);
pub const FNINF: Mpf = raw(1, 0, -789, -3);

impl fmt::Debug for Mpf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}, {}, {})",
            self.sign, self.man, self.exp, self.bc
        )
    }
}

impl Mpf {
    /// Zero mantissa and nonzero exponent: inf, -inf or nan.
    pub fn is_special(&self) -> bool {
        self.man.is_zero() && self.exp != 0
    }

    pub fn is_nan(&self) -> bool {
        *self == FNAN
    }
}

/// `prec_to_dps(n)`.
pub fn prec_to_dps(n: i64) -> i64 {
    1.max(((n as f64) / 3.321_928_094_887_362_6).round_ties_even() as i64 - 1)
}

/// `dps_to_prec(n)`.
pub fn dps_to_prec(n: i64) -> i64 {
    1.max((((n + 1) as f64) * 3.321_928_094_887_362_6).round_ties_even() as i64)
}

/// `repr_dps(n)`.
pub fn repr_dps(n: i64) -> i64 {
    let dps = prec_to_dps(n);
    if dps == 15 { 17 } else { dps + 3 }
}

/// Bits below the rounding position, as `h_mask[n]`: `(1 << (n-1)) - 1`.
fn low_bits_nonzero(man: &Int, n: i64) -> bool {
    // man & ((1 << (n-1)) - 1) != 0  <=>  trailing zeros < n-1
    n > 1 && man.trailing_zeros().is_some_and(|z| (z as i64) < n - 1)
}

/// Rounds `man` (non-negative) by `n > 0` bits.
fn round_man(man: &Int, n: i64, sign: u8, rnd: Rnd) -> Int {
    if rnd == Rnd::Nearest {
        let t = man >> (n - 1) as u64;
        if t.is_odd() && (t.bit(1) || low_bits_nonzero(man, n)) {
            &(&t >> 1) + 1
        } else {
            &t >> 1
        }
    } else if rnd.shifts_down(sign) {
        man >> n as u64
    } else {
        -(&(-man) >> n as u64)
    }
}

fn strip_trailing(man: Int, exp: i64, bc: i64) -> (Int, i64, i64) {
    let mut man = man;
    let (mut exp, mut bc) = (exp, bc);
    if man.is_even() {
        let t = man.trailing_zeros().expect("nonzero") as i64;
        man = &man >> t as u64;
        exp += t;
        bc -= t;
    }
    if man.is_one() {
        bc = 1;
    }
    (man, exp, bc)
}

/// `normalize(sign, man, exp, bc, prec, rnd)`.
pub fn normalize(sign: u8, man: Int, exp: i64, bc: i64, prec: i64, rnd: Rnd) -> Mpf {
    if man.is_zero() {
        return FZERO;
    }
    let (mut man, mut exp, mut bc) = (man, exp, bc);
    let n = bc - prec;
    if n > 0 {
        man = round_man(&man, n, sign, rnd);
        exp += n;
        bc = prec;
    }
    let (man, exp, bc) = strip_trailing(man, exp, bc);
    Mpf { sign, man, exp, bc }
}

/// `normalize1`: as `normalize`, for an odd (or zero) mantissa.
pub fn normalize1(sign: u8, man: Int, exp: i64, bc: i64, prec: i64, rnd: Rnd) -> Mpf {
    if man.is_zero() {
        return FZERO;
    }
    if bc <= prec {
        return Mpf { sign, man, exp, bc };
    }
    let n = bc - prec;
    let man = round_man(&man, n, sign, rnd);
    let (man, exp, bc) = strip_trailing(man, exp + n, prec);
    Mpf { sign, man, exp, bc }
}

/// `from_man_exp(man, exp, prec, rnd)`; `prec = 0` stores it exactly.
pub fn from_man_exp(man: &Int, exp: i64, prec: i64, rnd: Rnd) -> Mpf {
    let sign = u8::from(man.is_negative());
    let man = man.abs();
    let bc = bitcount(&man);
    if prec == 0 {
        if man.is_zero() {
            return FZERO;
        }
        let (man, exp, bc) = strip_trailing(man, exp, bc);
        return Mpf { sign, man, exp, bc };
    }
    normalize(sign, man, exp, bc, prec, rnd)
}

/// `from_int(n, prec, rnd)`.
pub fn from_int(n: &Int, prec: i64, rnd: Rnd) -> Mpf {
    from_man_exp(n, 0, prec, rnd)
}

pub fn from_i64(n: i64) -> Mpf {
    from_int(&Int::from(n), 0, ROUND_FAST)
}

/// `to_man_exp(s)`.
pub fn to_man_exp(s: &Mpf) -> MpfResult<(Int, i64)> {
    if s.is_special() {
        return Err(MpfError::Value(format!(
            "mantissa and exponent are undefined for {}",
            s.man
        )));
    }
    Ok((s.man.clone(), s.exp))
}

/// `round_int(x, n, rnd)` for a signed `x`.
fn round_int(x: &Int, n: i64, rnd: Rnd) -> Int {
    match rnd {
        Rnd::Nearest => {
            if !x.is_negative() {
                let t = x >> (n - 1) as u64;
                if t.is_odd() && (t.bit(1) || low_bits_nonzero(x, n)) {
                    &(&t >> 1) + 1
                } else {
                    &t >> 1
                }
            } else {
                -round_int(&-x, n, rnd)
            }
        }
        Rnd::Floor => x >> n as u64,
        Rnd::Ceiling => -(&(-x) >> n as u64),
        Rnd::Down => {
            if !x.is_negative() {
                x >> n as u64
            } else {
                -(&(-x) >> n as u64)
            }
        }
        Rnd::Up => {
            if !x.is_negative() {
                -(&(-x) >> n as u64)
            } else {
                x >> n as u64
            }
        }
    }
}

/// `to_int(s, rnd)`; `None` rounds toward zero, like `int(float)`.
pub fn to_int(s: &Mpf, rnd: Option<Rnd>) -> MpfResult<Int> {
    if s.is_special() {
        return Err(MpfError::Value("cannot convert inf or nan to int".into()));
    }
    let signed = if s.sign == 1 { -&s.man } else { s.man.clone() };
    if s.exp >= 0 {
        return Ok(&signed << s.exp as u64);
    }
    let n = -s.exp;
    match rnd {
        None => Ok(if s.sign == 1 {
            -(&s.man >> n as u64)
        } else {
            &s.man >> n as u64
        }),
        Some(r) => Ok(round_int(&signed, n, r)),
    }
}

/// `mpf_round_int(s, rnd)`.
pub fn mpf_round_int(s: &Mpf, rnd: Rnd) -> MpfResult<Mpf> {
    if s.is_special() || s.exp >= 0 {
        return Ok(s.clone());
    }
    let mag = s.exp + s.bc;
    if mag < 1 {
        return match rnd {
            Rnd::Ceiling => Ok(if s.sign == 1 { FZERO } else { FONE }),
            Rnd::Floor => Ok(if s.sign == 1 { FNONE } else { FZERO }),
            Rnd::Nearest => Ok(if mag < 0 || s.man.is_one() {
                FZERO
            } else if s.sign == 1 {
                FNONE
            } else {
                FONE
            }),
            _ => Err(MpfError::NotImplemented),
        };
    }
    Ok(mpf_pos(s, s.bc.min(mag), rnd))
}

/// `mpf_floor(s, prec, rnd)`.
pub fn mpf_floor(s: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    let v = mpf_round_int(s, Rnd::Floor).expect("floor is implemented");
    if prec != 0 { mpf_pos(&v, prec, rnd) } else { v }
}

/// `mpf_ceil(s, prec, rnd)`.
pub fn mpf_ceil(s: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    let v = mpf_round_int(s, Rnd::Ceiling).expect("ceiling is implemented");
    if prec != 0 { mpf_pos(&v, prec, rnd) } else { v }
}

/// `mpf_nint(s, prec, rnd)`.
pub fn mpf_nint(s: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    let v = mpf_round_int(s, Rnd::Nearest).expect("nearest is implemented");
    if prec != 0 { mpf_pos(&v, prec, rnd) } else { v }
}

/// `mpf_frac(s, prec, rnd)`.
pub fn mpf_frac(s: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    mpf_sub(s, &mpf_floor(s, 0, ROUND_FAST), prec, rnd)
}

/// `from_float(x, prec=53, rnd)`.
pub fn from_float(x: f64, prec: i64, rnd: Rnd) -> Mpf {
    if x.is_nan() {
        return FNAN;
    }
    if x == f64::INFINITY {
        return FINF;
    }
    if x == f64::NEG_INFINITY {
        return FNINF;
    }
    let (m, e) = frexp(x);
    let man = Int::from_f64_trunc(m * 9_007_199_254_740_992.0).expect("finite");
    from_man_exp(&man, e - 53, prec, rnd)
}

/// C's `frexp`: `x = m * 2**e` with `0.5 <= |m| < 1`.
pub fn frexp(x: f64) -> (f64, i64) {
    if x == 0.0 || !x.is_finite() {
        return (x, 0);
    }
    let bits = x.to_bits();
    let raw_exp = ((bits >> 52) & 0x7ff) as i64;
    if raw_exp == 0 {
        // Subnormal: scale into the normal range first.
        let (m, e) = frexp(x * 2f64.powi(64));
        return (m, e - 64);
    }
    let e = raw_exp - 1022;
    let m = f64::from_bits((bits & !(0x7ffu64 << 52)) | (1022u64 << 52));
    (m, e)
}

/// C's `ldexp(man, exp)` for an integer `man` of at most 53 bits, correctly
/// rounded into the subnormal range; `None` on overflow (Python raises
/// `OverflowError`).
fn ldexp_int(man: &Int, exp: i64) -> Option<f64> {
    if man.is_zero() {
        return Some(0.0);
    }
    let neg = man.is_negative();
    let m = man.abs();
    let bc = bitcount(&m);
    let top = bc + exp; // value in [2**(top-1), 2**top)
    if top > 1024 {
        return None;
    }
    let v = if top >= -1021 {
        // Normal result: exact.
        let f = m.to_f64().expect("<= 53 bits");
        scale2(f, exp)
    } else {
        // Subnormal: round to a multiple of 2**-1074, ties to even.
        let shift = -1074 - exp;
        let q = if shift <= 0 {
            &m << shift.unsigned_abs()
        } else if shift > bc + 1 {
            Int::ZERO
        } else {
            round_man(&m, shift, 0, Rnd::Nearest)
        };
        scale2(q.to_f64().expect("small"), -1074)
    };
    if v.is_infinite() {
        return None;
    }
    Some(if neg { -v } else { v })
}

/// `f * 2**e` by exact power-of-two steps.
fn scale2(f: f64, e: i64) -> f64 {
    let mut f = f;
    let mut e = e;
    while e > 1000 {
        f *= 2f64.powi(1000);
        e -= 1000;
    }
    while e < -1000 {
        f *= 2f64.powi(-1000);
        e += 1000;
    }
    f * 2f64.powi(e as i32)
}

/// `to_float(s, strict, rnd)`; `None` only when `strict` and out of range.
pub fn to_float(s: &Mpf, strict: bool, rnd: Rnd) -> Option<f64> {
    if s.man.is_zero() {
        return Some(if *s == FZERO {
            0.0
        } else if *s == FINF {
            f64::INFINITY
        } else if *s == FNINF {
            f64::NEG_INFINITY
        } else {
            f64::NAN
        });
    }
    let t = if s.bc > 53 {
        normalize1(s.sign, s.man.clone(), s.exp, s.bc, 53, rnd)
    } else {
        s.clone()
    };
    let man = if t.sign == 1 { -&t.man } else { t.man.clone() };
    match ldexp_int(&man, t.exp) {
        Some(v) => Some(v),
        None if strict => None,
        None => Some(if t.exp + t.bc > 0 {
            if t.sign == 1 {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            }
        } else {
            0.0
        }),
    }
}

/// `from_rational(p, q, prec, rnd)`.
pub fn from_rational(p: &Int, q: &Int, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    mpf_div(
        &from_int(p, 0, ROUND_FAST),
        &from_int(q, 0, ROUND_FAST),
        prec,
        rnd,
    )
}

/// `to_rational(s)`: `(p, q)` with `s == p/q`.
pub fn to_rational(s: &Mpf) -> MpfResult<(Int, Int)> {
    let man = if s.sign == 1 { -&s.man } else { s.man.clone() };
    if s.bc == -1 {
        return Err(MpfError::Value(format!(
            "cannot convert {man} to a rational number"
        )));
    }
    if s.exp >= 0 {
        Ok((&man << s.exp as u64, Int::ONE))
    } else {
        Ok((man, &Int::ONE << s.exp.unsigned_abs()))
    }
}

/// `to_fixed(s, prec)`.
pub fn to_fixed(s: &Mpf, prec: i64) -> Int {
    let offset = s.exp + prec;
    let man = if s.sign == 1 { -&s.man } else { s.man.clone() };
    if offset >= 0 {
        &man << offset as u64
    } else {
        &man >> offset.unsigned_abs()
    }
}

/// `mpf_eq(s, t)`.
pub fn mpf_eq(s: &Mpf, t: &Mpf) -> bool {
    if (s.man.is_zero() || t.man.is_zero()) && (*s == FNAN || *t == FNAN) {
        return false;
    }
    s == t
}

/// `mpf_hash(s)`, Python 3's numeric hash.
pub fn mpf_hash(s: &Mpf) -> i64 {
    const HASH_BITS: i64 = 61;
    let modulus = Int::from((1i64 << 61) - 1);
    if s.man.is_zero() {
        if *s == FNAN {
            return 0;
        }
        if *s == FINF {
            return 314_159;
        }
        if *s == FNINF {
            return -314_159;
        }
    }
    let mut h = s.man.py_mod(&modulus).expect("modulus > 0");
    let e = if s.exp >= 0 {
        s.exp % HASH_BITS
    } else {
        HASH_BITS - 1 - ((-1 - s.exp) % HASH_BITS)
    };
    h = (&h << e as u64).py_mod(&modulus).expect("modulus > 0");
    let mut h = h.as_i64().expect("< 2**61");
    if s.sign == 1 {
        h = -h;
    }
    if h == -1 { -2 } else { h }
}

/// `mpf_sign(s)`: -1, 0 or 1 (0 for nan).
pub fn mpf_sign(s: &Mpf) -> i32 {
    if s.man.is_zero() {
        if *s == FINF {
            return 1;
        }
        if *s == FNINF {
            return -1;
        }
        return 0;
    }
    if s.sign == 1 { -1 } else { 1 }
}

/// `mpf_cmp(s, t)`.
pub fn mpf_cmp(s: &Mpf, t: &Mpf) -> i32 {
    if s.man.is_zero() || t.man.is_zero() {
        if *s == FZERO {
            return -mpf_sign(t);
        }
        if *t == FZERO {
            return mpf_sign(s);
        }
        if s == t {
            return 0;
        }
        if *t == FNAN {
            return 1;
        }
        if *s == FINF {
            return 1;
        }
        if *t == FNINF {
            return 1;
        }
        return -1;
    }
    if s.sign != t.sign {
        return if s.sign == 0 { 1 } else { -1 };
    }
    if s.exp == t.exp {
        if s.man == t.man {
            return 0;
        }
        let gt = s.man > t.man;
        return if gt == (s.sign == 1) { -1 } else { 1 };
    }
    let a = s.bc + s.exp;
    let b = t.bc + t.exp;
    if s.sign == 1 {
        if a < b {
            return 1;
        }
        if a > b {
            return -1;
        }
    } else {
        if a < b {
            return -1;
        }
        if a > b {
            return 1;
        }
    }
    let delta = mpf_sub(s, t, 5, Rnd::Floor);
    if delta.sign == 1 { -1 } else { 1 }
}

pub fn mpf_lt(s: &Mpf, t: &Mpf) -> bool {
    !(s.is_nan() || t.is_nan()) && mpf_cmp(s, t) < 0
}

pub fn mpf_le(s: &Mpf, t: &Mpf) -> bool {
    !(s.is_nan() || t.is_nan()) && mpf_cmp(s, t) <= 0
}

pub fn mpf_gt(s: &Mpf, t: &Mpf) -> bool {
    !(s.is_nan() || t.is_nan()) && mpf_cmp(s, t) > 0
}

pub fn mpf_ge(s: &Mpf, t: &Mpf) -> bool {
    !(s.is_nan() || t.is_nan()) && mpf_cmp(s, t) >= 0
}

/// `mpf_pos(s, prec, rnd)`: round `s` to `prec` bits (0 = exactly).
pub fn mpf_pos(s: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    if prec != 0 {
        if s.is_special() {
            return s.clone();
        }
        return normalize1(s.sign, s.man.clone(), s.exp, s.bc, prec, rnd);
    }
    s.clone()
}

/// `mpf_neg(s, prec, rnd)`.
pub fn mpf_neg(s: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    if s.man.is_zero() {
        if s.exp != 0 {
            if *s == FINF {
                return FNINF;
            }
            if *s == FNINF {
                return FINF;
            }
        }
        return s.clone();
    }
    if prec == 0 {
        return Mpf {
            sign: 1 - s.sign,
            ..s.clone()
        };
    }
    normalize1(1 - s.sign, s.man.clone(), s.exp, s.bc, prec, rnd)
}

/// `mpf_abs(s, prec, rnd)`.
pub fn mpf_abs(s: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    if s.is_special() {
        if *s == FNINF {
            return FINF;
        }
        return s.clone();
    }
    if prec == 0 {
        return Mpf {
            sign: 0,
            ..s.clone()
        };
    }
    normalize1(0, s.man.clone(), s.exp, s.bc, prec, rnd)
}

fn add_impl(s: &Mpf, t: &Mpf, prec: i64, rnd: Rnd, sub: bool) -> Mpf {
    let (ssign, sman, sexp, sbc) = (s.sign, &s.man, s.exp, s.bc);
    let (tsign, tman, texp, tbc) = (t.sign ^ u8::from(sub), &t.man, t.exp, t.bc);
    if !sman.is_zero() && !tman.is_zero() {
        let offset = sexp - texp;
        if offset != 0 {
            if offset > 0 {
                if offset > 100 && prec != 0 {
                    let delta = sbc + sexp - tbc - texp;
                    if delta > prec + 4 {
                        let off = prec + 4;
                        let mut m = sman << off as u64;
                        m = if tsign == ssign { &m + 1 } else { &m - 1 };
                        let bc = bitcount(&m);
                        return normalize1(ssign, m, sexp - off, bc, prec, rnd);
                    }
                }
                let (man, sign) = if ssign == tsign {
                    (tman + &(sman << offset as u64), ssign)
                } else {
                    let m = if ssign == 1 {
                        tman - &(sman << offset as u64)
                    } else {
                        &(sman << offset as u64) - tman
                    };
                    if m.is_negative() { (-m, 1) } else { (m, 0) }
                };
                let bc = bitcount(&man);
                return normalize1(sign, man, texp, bc, if prec != 0 { prec } else { bc }, rnd);
            } else {
                if offset < -100 && prec != 0 {
                    let delta = tbc + texp - sbc - sexp;
                    if delta > prec + 4 {
                        let off = prec + 4;
                        let mut m = tman << off as u64;
                        m = if ssign == tsign { &m + 1 } else { &m - 1 };
                        let bc = bitcount(&m);
                        return normalize1(tsign, m, texp - off, bc, prec, rnd);
                    }
                }
                let (man, sign) = if ssign == tsign {
                    (sman + &(tman << offset.unsigned_abs()), ssign)
                } else {
                    let m = if tsign == 1 {
                        sman - &(tman << offset.unsigned_abs())
                    } else {
                        &(tman << offset.unsigned_abs()) - sman
                    };
                    if m.is_negative() { (-m, 1) } else { (m, 0) }
                };
                let bc = bitcount(&man);
                return normalize1(sign, man, sexp, bc, if prec != 0 { prec } else { bc }, rnd);
            }
        }
        let (man, sign) = if ssign == tsign {
            (tman + sman, ssign)
        } else {
            let m = if ssign == 1 { tman - sman } else { sman - tman };
            if m.is_negative() { (-m, 1) } else { (m, 0) }
        };
        let bc = bitcount(&man);
        return normalize(sign, man, texp, bc, if prec != 0 { prec } else { bc }, rnd);
    }
    // Zeros and special numbers.
    let t = if sub {
        mpf_neg(t, 0, ROUND_FAST)
    } else {
        t.clone()
    };
    if sman.is_zero() {
        if sexp != 0 {
            if *s == t || !t.man.is_zero() || t.exp == 0 {
                return s.clone();
            }
            return FNAN;
        }
        if !t.man.is_zero() {
            let p = if prec != 0 { prec } else { t.bc };
            return normalize1(t.sign, t.man.clone(), t.exp, t.bc, p, rnd);
        }
        return t;
    }
    if t.exp != 0 {
        return t;
    }
    let p = if prec != 0 { prec } else { sbc };
    normalize1(ssign, sman.clone(), sexp, sbc, p, rnd)
}

/// `mpf_add(s, t, prec, rnd)`; `prec = 0` adds exactly.
pub fn mpf_add(s: &Mpf, t: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    add_impl(s, t, prec, rnd, false)
}

/// `mpf_sub(s, t, prec, rnd)`.
pub fn mpf_sub(s: &Mpf, t: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    add_impl(s, t, prec, rnd, true)
}

/// `mpf_sum(xs, prec, rnd, absolute)`.
pub fn mpf_sum(xs: &[Mpf], prec: i64, rnd: Rnd, absolute: bool) -> Mpf {
    let mut man = Int::ZERO;
    let mut exp: i64 = 0;
    let max_extra_prec = if prec != 0 { prec * 2 } else { 1_000_000 };
    let mut special: Option<Mpf> = None;
    for x in xs {
        if !x.man.is_zero() {
            let xman = if x.sign == 1 && !absolute {
                -&x.man
            } else {
                x.man.clone()
            };
            let delta = x.exp - exp;
            if x.exp >= exp {
                if delta > max_extra_prec
                    && (man.is_zero() || delta - bitcount(&man.abs()) > max_extra_prec)
                {
                    man = xman;
                    exp = x.exp;
                } else {
                    man = &man + &(&xman << delta as u64);
                }
            } else {
                let delta = -delta;
                if delta - x.bc > max_extra_prec {
                    if man.is_zero() {
                        man = xman;
                        exp = x.exp;
                    }
                } else {
                    man = &(&man << delta as u64) + &xman;
                    exp = x.exp;
                }
            }
        } else if x.exp != 0 {
            let x = if absolute {
                mpf_abs(x, 0, ROUND_FAST)
            } else {
                x.clone()
            };
            let base = special.clone().unwrap_or(FZERO);
            special = Some(mpf_add(&base, &x, 1, ROUND_FAST));
        }
    }
    // `if special:` -- a tuple is always truthy.
    if let Some(sp) = special {
        return sp;
    }
    from_man_exp(&man, exp, prec, rnd)
}

/// `mpf_mul(s, t, prec, rnd)` (mpmath's Python backend).
pub fn mpf_mul(s: &Mpf, t: &Mpf, prec: i64, rnd: Rnd) -> Mpf {
    let sign = s.sign ^ t.sign;
    let man = &s.man * &t.man;
    if !man.is_zero() {
        let mut bc = s.bc + t.bc - 1;
        if (&man >> bc as u64).is_one() {
            bc += 1;
        }
        if prec != 0 {
            return normalize1(sign, man, s.exp + t.exp, bc, prec, rnd);
        }
        return Mpf {
            sign,
            man,
            exp: s.exp + t.exp,
            bc,
        };
    }
    let s_special = s.is_special();
    let t_special = t.is_special();
    if !s_special && !t_special {
        return FZERO;
    }
    if *s == FNAN || *t == FNAN {
        return FNAN;
    }
    let (s, t) = if t_special { (t, s) } else { (s, t) };
    if *t == FZERO {
        return FNAN;
    }
    if mpf_sign(s) * mpf_sign(t) == 1 {
        FINF
    } else {
        FNINF
    }
}

/// `mpf_mul_int(s, n, prec, rnd)`.
pub fn mpf_mul_int(s: &Mpf, n: &Int, prec: i64, rnd: Rnd) -> Mpf {
    if s.man.is_zero() {
        return mpf_mul(s, &from_int(n, 0, ROUND_FAST), prec, rnd);
    }
    if n.is_zero() {
        return FZERO;
    }
    let mut sign = s.sign;
    let n = if n.is_negative() {
        sign ^= 1;
        -n
    } else {
        n.clone()
    };
    let man = &s.man * &n;
    let mut bc = s.bc + bitcount(&n) - 1;
    if (&man >> bc as u64).is_one() {
        bc += 1;
    }
    normalize(sign, man, s.exp, bc, prec, rnd)
}

/// `mpf_shift(s, n)`: `s * 2**n`, exactly.
pub fn mpf_shift(s: &Mpf, n: i64) -> Mpf {
    if s.man.is_zero() {
        return s.clone();
    }
    Mpf {
        exp: s.exp + n,
        ..s.clone()
    }
}

/// `mpf_frexp(x)`.
pub fn mpf_frexp(x: &Mpf) -> MpfResult<(Mpf, i64)> {
    if x.man.is_zero() {
        if *x == FZERO {
            return Ok((FZERO, 0));
        }
        return Err(MpfError::Value(String::new()));
    }
    Ok((mpf_shift(x, -x.bc - x.exp), x.bc + x.exp))
}

/// `mpf_div(s, t, prec, rnd)`.
pub fn mpf_div(s: &Mpf, t: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    if s.man.is_zero() || t.man.is_zero() {
        if *s == FZERO {
            if *t == FZERO {
                return Err(MpfError::ZeroDivision);
            }
            if *t == FNAN {
                return Ok(FNAN);
            }
            return Ok(FZERO);
        }
        if *t == FZERO {
            return Err(MpfError::ZeroDivision);
        }
        let s_special = s.is_special();
        let t_special = t.is_special();
        if s_special && t_special {
            return Ok(FNAN);
        }
        if *s == FNAN || *t == FNAN {
            return Ok(FNAN);
        }
        if !t_special {
            if *t == FZERO {
                return Ok(FNAN);
            }
            return Ok(if mpf_sign(s) * mpf_sign(t) == 1 {
                FINF
            } else {
                FNINF
            });
        }
        return Ok(FZERO);
    }
    let sign = s.sign ^ t.sign;
    if t.man.is_one() {
        return Ok(normalize1(
            sign,
            s.man.clone(),
            s.exp - t.exp,
            s.bc,
            prec,
            rnd,
        ));
    }
    let extra = (prec - s.bc + t.bc + 5).max(5);
    let (quot, rem) = (&s.man << extra as u64).div_mod(&t.man).expect("t.man > 0");
    if !rem.is_zero() {
        let quot = &(&quot << 1) + 1;
        let extra = extra + 1;
        let bc = bitcount(&quot);
        return Ok(normalize1(sign, quot, s.exp - t.exp - extra, bc, prec, rnd));
    }
    let bc = bitcount(&quot);
    Ok(normalize(sign, quot, s.exp - t.exp - extra, bc, prec, rnd))
}

/// `mpf_rdiv_int(n, t, prec, rnd)`: `n / t`.
pub fn mpf_rdiv_int(n: &Int, t: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    if n.is_zero() || t.man.is_zero() {
        return mpf_div(&from_int(n, 0, ROUND_FAST), t, prec, rnd);
    }
    let mut sign = t.sign;
    let n = if n.is_negative() {
        sign ^= 1;
        -n
    } else {
        n.clone()
    };
    let extra = prec + t.bc + 5;
    let (quot, rem) = (&n << extra as u64).div_mod(&t.man).expect("t.man > 0");
    if !rem.is_zero() {
        let quot = &(&quot << 1) + 1;
        let extra = extra + 1;
        let bc = bitcount(&quot);
        return Ok(normalize1(sign, quot, -t.exp - extra, bc, prec, rnd));
    }
    let bc = bitcount(&quot);
    Ok(normalize(sign, quot, -t.exp - extra, bc, prec, rnd))
}

/// `mpf_mod(s, t, prec, rnd)`.
pub fn mpf_mod(s: &Mpf, t: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    if s.is_special() || t.is_special() {
        return Ok(FNAN);
    }
    if s.sign == t.sign && t.exp > s.exp + s.bc {
        return Ok(s.clone());
    }
    if t.man.is_one() && s.exp > t.exp + t.bc {
        return Ok(FZERO);
    }
    let base = s.exp.min(t.exp);
    let sman = if s.sign == 1 { -&s.man } else { s.man.clone() };
    let tman = if t.sign == 1 { -&t.man } else { t.man.clone() };
    let man = (&sman << (s.exp - base) as u64)
        .py_mod(&(&tman << (t.exp - base) as u64))
        .map_err(|_| MpfError::ZeroDivision)?;
    let (man, sign) = if man.is_negative() {
        (-man, 1)
    } else {
        (man, 0)
    };
    let bc = bitcount(&man);
    Ok(normalize(sign, man, base, bc, prec, rnd))
}

/// `mpf_pow_int(s, n, prec, rnd)`.
pub fn mpf_pow_int(s: &Mpf, n: &Int, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    if s.is_special() {
        if *s == FINF {
            return Ok(if n.is_positive() {
                s.clone()
            } else if n.is_zero() {
                FNAN
            } else {
                FZERO
            });
        }
        if *s == FNINF {
            return Ok(if n.is_positive() {
                if n.is_odd() { FNINF } else { FINF }
            } else if n.is_zero() {
                FNAN
            } else {
                FZERO
            });
        }
        return Ok(FNAN);
    }
    if n.is_zero() {
        return Ok(FONE);
    }
    if n.is_one() {
        return Ok(mpf_pos(s, prec, rnd));
    }
    if *n == 2 {
        if s.man.is_zero() {
            return Ok(FZERO);
        }
        let man = &s.man * &s.man;
        if man.is_one() {
            return Ok(Mpf {
                sign: 0,
                man: Int::ONE,
                exp: s.exp + s.exp,
                bc: 1,
            });
        }
        let mut bc = s.bc + s.bc - 2;
        bc += bitcount(&(&man >> bc as u64));
        return Ok(normalize1(0, man, s.exp + s.exp, bc, prec, rnd));
    }
    if *n == -1 {
        return mpf_div(&FONE, s, prec, rnd);
    }
    if n.is_negative() {
        let inverse = mpf_pow_int(s, &-n, prec + 5, rnd.reciprocal())?;
        return mpf_div(&FONE, &inverse, prec, rnd);
    }
    let result_sign = s.sign & u8::from(n.is_odd());
    if s.man.is_one() {
        let e = n
            .as_i64()
            .and_then(|n| s.exp.checked_mul(n))
            .expect("exponent fits in i64");
        return Ok(Mpf {
            sign: result_sign,
            man: Int::ONE,
            exp: e,
            bc: 1,
        });
    }
    let nn = n.as_i64().expect("power fits in i64");
    if s.bc.saturating_mul(nn) < 1000 {
        let man = s.man.pow(nn as u64);
        let bc = bitcount(&man);
        return Ok(normalize1(result_sign, man, s.exp * nn, bc, prec, rnd));
    }
    let rounds_down = rnd == Rnd::Nearest || rnd.shifts_down(result_sign);
    let workprec = prec + 4 * bitcount(n) + 4;
    let (mut pm, mut pe, mut pbc) = (Int::ONE, 0i64, 1i64);
    let (mut man, mut exp, mut bc) = (s.man.clone(), s.exp, s.bc);
    let mut n = nn;
    loop {
        if n & 1 == 1 {
            pm = &pm * &man;
            pe += exp;
            pbc += bc - 2;
            pbc += bitcount(&(&pm >> pbc as u64));
            if pbc > workprec {
                let d = (pbc - workprec) as u64;
                pm = if rounds_down {
                    &pm >> d
                } else {
                    -(&(-&pm) >> d)
                };
                pe += pbc - workprec;
                pbc = workprec;
            }
            n -= 1;
            if n == 0 {
                break;
            }
        }
        man = &man * &man;
        exp += exp;
        bc = bc + bc - 2;
        bc += bitcount(&(&man >> bc as u64));
        if bc > workprec {
            let d = (bc - workprec) as u64;
            man = if rounds_down {
                &man >> d
            } else {
                -(&(-&man) >> d)
            };
            exp += bc - workprec;
            bc = workprec;
        }
        n /= 2;
    }
    Ok(normalize(result_sign, pm, pe, pbc, prec, rnd))
}

/// `mpf_perturb(x, eps_sign, prec, rnd)`.
pub fn mpf_perturb(x: &Mpf, eps_sign: u8, prec: i64, rnd: Rnd) -> Mpf {
    if rnd == Rnd::Nearest {
        return mpf_pos(x, prec, rnd);
    }
    let eps = Mpf {
        sign: eps_sign,
        man: Int::ONE,
        exp: x.exp + x.bc - prec - 1,
        bc: 1,
    };
    let away = if x.sign == 1 {
        matches!(rnd, Rnd::Down | Rnd::Ceiling) ^ (eps_sign == 1)
    } else {
        matches!(rnd, Rnd::Up | Rnd::Ceiling) ^ (eps_sign == 1)
    };
    if away {
        mpf_add(x, &eps, prec, rnd)
    } else {
        mpf_pos(x, prec, rnd)
    }
}

/// `to_digits_exp(s, dps)`: `(sign, digits, exponent)`, rounded toward zero.
pub fn to_digits_exp(s: &Mpf, dps: i64) -> (&'static str, String, i64) {
    let (sign, s) = if s.sign == 1 {
        ("-", mpf_neg(s, 0, ROUND_FAST))
    } else {
        ("", s.clone())
    };
    if s.man.is_zero() {
        return ("", "0".into(), 0);
    }
    let log2_10 = 10f64.ln() / 2f64.ln();
    let bitprec = (dps as f64 * log2_10) as i64 + 10;
    let exp_from_1 = s.exp + s.bc;
    let (s, mut exponent) = if exp_from_1.abs() > 3500 {
        let expprec = bitcount(&Int::from(s.exp.abs())) + 5;
        let tmp = from_i64(s.exp);
        let tmp = mpf_mul(&tmp, &mpf_ln2(expprec, ROUND_FAST), 0, ROUND_FAST);
        let tmp =
            mpf_div(&tmp, &mpf_ln10(expprec, ROUND_FAST), expprec, ROUND_FAST).expect("ln10 != 0");
        let b = to_int(&tmp, None).expect("finite");
        let p = mpf_pow_int(&FTEN, &b, bitprec, ROUND_FAST).expect("10**b is finite");
        let s = mpf_div(&s, &p, bitprec, ROUND_FAST).expect("10**b != 0");
        (s, b.as_i64().expect("decimal exponent fits in i64"))
    } else {
        (s, 0)
    };
    let fixprec = (bitprec - s.exp - s.bc).max(0);
    let fixdps = (fixprec as f64 / log2_10 + 0.5) as i64;
    let sf = to_fixed(&s, fixprec);
    let sd = bin_to_radix(&sf, fixprec, 10, fixdps);
    let digits = numeral(&sd, dps);
    exponent += digits.len() as i64 - fixdps - 1;
    (sign, digits, exponent)
}

/// `to_str(s, dps, strip_zeros, min_fixed, max_fixed, show_zero_exponent)`.
/// `None` bounds take mpmath's defaults; `i64::MIN`/`i64::MAX` stand for
/// `-inf`/`+inf`.
pub fn to_str(
    s: &Mpf,
    dps: i64,
    strip_zeros: bool,
    min_fixed: Option<i64>,
    max_fixed: Option<i64>,
    show_zero_exponent: bool,
) -> String {
    if s.man.is_zero() {
        if *s == FZERO {
            let mut t = String::from(if dps != 0 { "0.0" } else { ".0" });
            if show_zero_exponent {
                t.push_str("e+0");
            }
            return t;
        }
        if *s == FINF {
            return "+inf".into();
        }
        if *s == FNINF {
            return "-inf".into();
        }
        if *s == FNAN {
            return "nan".into();
        }
        panic!("to_str: malformed mpf {s:?}");
    }
    let min_fixed = min_fixed.unwrap_or_else(|| (-(dps.div_euclid(3))).min(-5));
    let max_fixed = max_fixed.unwrap_or(dps);
    let (sign, digits, mut exponent) = to_digits_exp(s, dps + 3);
    let mut digits: Vec<u8> = digits.into_bytes();
    if dps == 0 {
        if b"56789".contains(&digits[0]) {
            exponent += 1;
        }
        digits = b".0".to_vec();
    } else {
        let d = dps as usize;
        if digits.len() > d && b"56789".contains(&digits[d]) {
            digits.truncate(d);
            let mut i = d as i64 - 1;
            while i >= 0 && digits[i as usize] == b'9' {
                i -= 1;
            }
            if i >= 0 {
                let iu = i as usize;
                digits[iu] += 1;
                for c in &mut digits[iu + 1..] {
                    *c = b'0';
                }
            } else {
                digits = std::iter::once(b'1')
                    .chain(std::iter::repeat_n(b'0', d - 1))
                    .collect();
                exponent += 1;
            }
        } else {
            digits.truncate(d);
        }
        let split;
        if min_fixed < exponent && exponent < max_fixed {
            if exponent < 0 {
                let mut z = vec![b'0'; (-exponent) as usize];
                z.extend_from_slice(&digits);
                digits = z;
                split = 1;
            } else {
                split = exponent as usize + 1;
                if split > d {
                    digits.extend(std::iter::repeat_n(b'0', split - d));
                }
            }
            exponent = 0;
        } else {
            split = 1;
        }
        let mut out = digits[..split].to_vec();
        out.push(b'.');
        out.extend_from_slice(&digits[split..]);
        digits = out;
        if strip_zeros {
            while digits.last() == Some(&b'0') {
                digits.pop();
            }
            if digits.last() == Some(&b'.') {
                digits.push(b'0');
            }
        }
    }
    let digits = String::from_utf8(digits).expect("ASCII digits");
    if exponent == 0 && dps != 0 && !show_zero_exponent {
        return format!("{sign}{digits}");
    }
    if exponent >= 0 {
        return format!("{sign}{digits}e+{exponent}");
    }
    format!("{sign}{digits}e{exponent}")
}

/// Python's `float(x)` literal check, for the strings `from_str` accepts.
fn is_float_literal(x: &str) -> bool {
    let b = x.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let rest = &x[i..];
    if ["inf", "infinity", "nan"].contains(&rest) {
        return true;
    }
    // digits with single underscores between digits
    let digit_run = |i: &mut usize| -> usize {
        let start = *i;
        let mut last_us = false;
        while *i < b.len() && (b[*i].is_ascii_digit() || (b[*i] == b'_' && *i > start && !last_us))
        {
            last_us = b[*i] == b'_';
            *i += 1;
        }
        if last_us {
            *i -= 1;
        }
        *i - start
    };
    let int_digits = digit_run(&mut i);
    let mut frac_digits = 0;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        frac_digits = digit_run(&mut i);
    }
    if int_digits + frac_digits == 0 {
        return false;
    }
    if i < b.len() && b[i] == b'e' {
        i += 1;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        if digit_run(&mut i) == 0 {
            return false;
        }
    }
    i == b.len()
}

/// `str_to_man_exp(x, base=10)`.
pub fn str_to_man_exp(x: &str) -> MpfResult<(Int, i64)> {
    let x = x.to_lowercase();
    let x = x.trim_end_matches('l');
    if !is_float_literal(x.trim()) {
        return Err(MpfError::Value(format!(
            "could not convert string to float: '{x}'"
        )));
    }
    let (mut mant, mut exp) = match x.split_once('e') {
        None => (x.to_string(), 0i64),
        Some((m, e)) => {
            let e: Int = e
                .replace('_', "")
                .parse()
                .map_err(|_| MpfError::Value(x.into()))?;
            (
                m.to_string(),
                e.as_i64().ok_or_else(|| MpfError::Value(x.into()))?,
            )
        }
    };
    if let Some((a, b)) = mant.clone().split_once('.') {
        let b = b.trim_end_matches('0');
        exp -= b.len() as i64;
        mant = format!("{a}{b}");
    }
    let mant = mant.replace('_', "");
    let man = match mant.trim() {
        "" | "+" | "-" => Int::ZERO,
        m => m.parse::<Int>().map_err(|_| MpfError::Value(x.into()))?,
    };
    Ok((man, exp))
}

/// `from_str(x, prec, rnd)`.
pub fn from_str(x: &str, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    let x = x.to_lowercase();
    let x = x.trim();
    match x {
        "inf" | "+inf" => return Ok(FINF),
        "-inf" => return Ok(FNINF),
        "nan" => return Ok(FNAN),
        _ => {}
    }
    if let Some((p, q)) = x.split_once('/') {
        let p: Int = p
            .trim_end_matches('l')
            .parse()
            .map_err(|_| MpfError::Value(x.into()))?;
        let q: Int = q
            .trim_end_matches('l')
            .parse()
            .map_err(|_| MpfError::Value(x.into()))?;
        return from_rational(&p, &q, prec, rnd);
    }
    let (man, exp) = str_to_man_exp(x)?;
    if exp.abs() > 400 {
        let s = from_int(&man, prec + 10, ROUND_FAST);
        let p = mpf_pow_int(&FTEN, &Int::from(exp), prec + 10, ROUND_FAST)?;
        return Ok(mpf_mul(&s, &p, prec, rnd));
    }
    if exp >= 0 {
        Ok(from_int(
            &(&man * &Int::from(10).pow(exp as u64)),
            prec,
            rnd,
        ))
    } else {
        from_rational(&man, &Int::from(10).pow(exp.unsigned_abs()), prec, rnd)
    }
}

/// `mpf_sqrt(s, prec, rnd)`: correctly rounded.
pub fn mpf_sqrt(s: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    if s.sign == 1 {
        return Err(MpfError::ComplexResult("square root of a negative number"));
    }
    if s.man.is_zero() {
        return Ok(s.clone());
    }
    let (mut man, mut exp, mut bc) = (s.man.clone(), s.exp, s.bc);
    if exp & 1 == 1 {
        exp -= 1;
        man = &man << 1;
        bc += 1;
    } else if man.is_one() {
        return Ok(normalize1(s.sign, man, exp.div_euclid(2), bc, prec, rnd));
    }
    let mut shift = 4.max(2 * prec - bc + 4);
    shift += shift & 1;
    let man = if matches!(rnd, Rnd::Floor | Rnd::Down) {
        isqrt(&(&man << shift as u64))
    } else {
        let (m, rem) = sqrtrem(&(&man << shift as u64));
        if rem.is_zero() {
            m
        } else {
            shift += 2;
            &(&m << 1) + 1
        }
    };
    Ok(from_man_exp(&man, (exp - shift).div_euclid(2), prec, rnd))
}

/// `mpf_hypot(x, y, prec, rnd)`.
pub fn mpf_hypot(x: &Mpf, y: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    if *y == FZERO {
        return Ok(mpf_abs(x, prec, rnd));
    }
    if *x == FZERO {
        return Ok(mpf_abs(y, prec, rnd));
    }
    let h2 = mpf_add(
        &mpf_mul(x, x, 0, ROUND_FAST),
        &mpf_mul(y, y, 0, ROUND_FAST),
        prec + 4,
        ROUND_FAST,
    );
    mpf_sqrt(&h2, prec, rnd)
}
