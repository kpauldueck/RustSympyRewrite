//! Port of the mathematical constants in `mpmath.libmp.libelefun`: pi
//! (Chudnovsky), e, ln 2, ln 10, the golden ratio and the degree.
//!
//! mpmath memoizes each constant's fixed-point value at the highest
//! precision asked for so far (`constant_memo`) and serves lower precisions
//! by shifting that value. That makes the last bits depend on history, so
//! the memo is reproduced here: one per thread, as mpmath has one per
//! process.

use std::cell::RefCell;

use rsympy_num::Int;

use crate::intmath::{bitcount, isqrt_fast};
use crate::mpf::{Mpf, Rnd, normalize};

#[derive(Clone, Copy)]
enum Constant {
    Pi,
    E,
    Ln2,
    Ln10,
    Phi,
}

thread_local! {
    static MEMO: RefCell<[Option<(i64, Int)>; 5]> = const { RefCell::new([None, None, None, None, None]) };
}

/// `constant_memo(f)(prec)`.
fn memoized(c: Constant, prec: i64, f: fn(i64) -> Int) -> Int {
    let idx = c as usize;
    if let Some(v) = MEMO.with(|m| {
        m.borrow()[idx]
            .as_ref()
            .filter(|(mp, _)| prec <= *mp)
            .map(|(mp, val)| val >> (mp - prec) as u64)
    }) {
        return v;
    }
    let newprec = (prec as f64 * 1.05 + 10.0) as i64;
    let val = f(newprec);
    let out = &val >> (newprec - prec) as u64;
    MEMO.with(|m| m.borrow_mut()[idx] = Some((newprec, val)));
    out
}

/// Forgets every memoized constant, as a fresh Python process would have.
pub fn clear_constant_cache() {
    MEMO.with(|m| *m.borrow_mut() = [None, None, None, None, None]);
}

fn bsp_acot(q: &Int, a: i64, b: i64, hyperbolic: bool) -> (Int, Int, Int) {
    if b - a == 1 {
        let a1 = Int::from(2 * a + 3);
        let p = if hyperbolic || a & 1 == 1 {
            Int::ONE
        } else {
            Int::NEG_ONE
        };
        return (p, &a1 * &(q * q), a1);
    }
    let m = (a + b).div_euclid(2);
    let (p1, q1, r1) = bsp_acot(q, a, m, hyperbolic);
    let (p2, q2, r2) = bsp_acot(q, m, b, hyperbolic);
    (&(&q2 * &p1) + &(&r1 * &p2), &q1 * &q2, &r1 * &r2)
}

/// `acot_fixed(a, prec, hyperbolic)`.
fn acot_fixed(a: i64, prec: i64, hyperbolic: bool) -> Int {
    let n = (0.35 * prec as f64 / (a as f64).ln() + 20.0) as i64;
    let a_int = Int::from(a);
    let (p, q, _r) = bsp_acot(&a_int, 0, n, hyperbolic);
    (&(&p + &q) << prec as u64)
        .floor_div(&(&q * &a_int))
        .expect("q > 0")
}

/// `machin(coefs, prec, hyperbolic)`.
fn machin(coefs: &[(i64, i64)], prec: i64, hyperbolic: bool) -> Int {
    let extraprec = 10;
    let mut s = Int::ZERO;
    for &(a, b) in coefs {
        s = &s + &(&Int::from(a) * &acot_fixed(b, prec + extraprec, hyperbolic));
    }
    &s >> extraprec as u64
}

fn ln2_raw(prec: i64) -> Int {
    machin(&[(18, 26), (-2, 4801), (8, 8749)], prec, true)
}

fn ln10_raw(prec: i64) -> Int {
    machin(&[(46, 31), (34, 49), (20, 161)], prec, true)
}

/// `ln2_fixed(prec)`: floor(ln(2) * 2**prec), memoized.
pub fn ln2_fixed(prec: i64) -> Int {
    memoized(Constant::Ln2, prec, ln2_raw)
}

/// `ln10_fixed(prec)`.
pub fn ln10_fixed(prec: i64) -> Int {
    memoized(Constant::Ln10, prec, ln10_raw)
}

const CHUD_A: i64 = 13_591_409;
const CHUD_B: i64 = 545_140_134;
const CHUD_C: i64 = 640_320;
const CHUD_D: i64 = 12;

fn bs_chudnovsky(a: i64, b: i64) -> (Int, Int, Int) {
    if b - a == 1 {
        let bi = Int::from(b);
        let g = &(&Int::from(6 * b - 5) * &Int::from(2 * b - 1)) * &Int::from(6 * b - 1);
        let p = (&bi.pow(3) * &Int::from(CHUD_C).pow(3))
            .floor_div(&Int::from(24))
            .expect("24 > 0");
        let mut q = &g * &(&Int::from(CHUD_A) + &(&Int::from(CHUD_B) * &bi));
        if b & 1 == 1 {
            q = -q;
        }
        return (g, p, q);
    }
    let mid = (a + b).div_euclid(2);
    let (g1, p1, q1) = bs_chudnovsky(a, mid);
    let (g2, p2, q2) = bs_chudnovsky(mid, b);
    let p = &p1 * &p2;
    let g = &g1 * &g2;
    let q = &(&q1 * &p2) + &(&q2 * &g1);
    (g, p, q)
}

// mpmath's own truncated log2(10), not `LOG2_10`: it decides the term count.
#[allow(clippy::approx_constant)]
fn pi_raw(prec: i64) -> Int {
    let n = (prec as f64 / 3.321_928_094_8 / 14.181_647_462 + 2.0) as i64;
    let (_g, p, q) = bs_chudnovsky(0, n);
    let c = Int::from(CHUD_C);
    let sqrt_c = isqrt_fast(&(&c << (2 * prec) as u64));
    let num = &(&p * &c) * &sqrt_c;
    let den = &(&q + &(&Int::from(CHUD_A) * &p)) * &Int::from(CHUD_D);
    num.floor_div(&den).expect("den != 0")
}

/// `pi_fixed(prec)`: floor(pi * 2**prec), memoized.
pub fn pi_fixed(prec: i64) -> Int {
    memoized(Constant::Pi, prec, pi_raw)
}

/// `degree_fixed(prec)`: pi/180 (not memoized itself).
pub fn degree_fixed(prec: i64) -> Int {
    pi_fixed(prec).floor_div(&Int::from(180)).expect("180 > 0")
}

fn bspe(a: i64, b: i64) -> (Int, Int) {
    if b - a == 1 {
        return (Int::ONE, Int::from(b));
    }
    let m = (a + b).div_euclid(2);
    let (p1, q1) = bspe(a, m);
    let (p2, q2) = bspe(m, b);
    (&(&p1 * &q2) + &p2, &q1 * &q2)
}

fn e_raw(prec: i64) -> Int {
    let n = (1.1 * prec as f64 / (prec as f64).ln() + 20.0) as i64;
    let (p, q) = bspe(0, n);
    (&(&p + &q) << prec as u64).floor_div(&q).expect("q > 0")
}

/// `e_fixed(prec)`.
pub fn e_fixed(prec: i64) -> Int {
    memoized(Constant::E, prec, e_raw)
}

fn phi_raw(prec: i64) -> Int {
    let prec = prec + 10;
    let a = &isqrt_fast(&(&Int::from(5) << (2 * prec) as u64)) + &(&Int::ONE << prec as u64);
    &a >> 11
}

/// `phi_fixed(prec)`.
pub fn phi_fixed(prec: i64) -> Int {
    memoized(Constant::Phi, prec, phi_raw)
}

/// `def_mpf_constant(fixed)(prec, rnd)`.
fn mpf_constant(fixed: fn(i64) -> Int, prec: i64, rnd: Rnd) -> Mpf {
    let wp = prec + 20;
    let mut v = fixed(wp);
    if matches!(rnd, Rnd::Up | Rnd::Ceiling) {
        v = &v + 1;
    }
    let bc = bitcount(&v);
    normalize(0, v, -wp, bc, prec, rnd)
}

pub fn mpf_pi(prec: i64, rnd: Rnd) -> Mpf {
    mpf_constant(pi_fixed, prec, rnd)
}

pub fn mpf_e(prec: i64, rnd: Rnd) -> Mpf {
    mpf_constant(e_fixed, prec, rnd)
}

pub fn mpf_ln2(prec: i64, rnd: Rnd) -> Mpf {
    mpf_constant(ln2_fixed, prec, rnd)
}

pub fn mpf_ln10(prec: i64, rnd: Rnd) -> Mpf {
    mpf_constant(ln10_fixed, prec, rnd)
}

pub fn mpf_phi(prec: i64, rnd: Rnd) -> Mpf {
    mpf_constant(phi_fixed, prec, rnd)
}

pub fn mpf_degree(prec: i64, rnd: Rnd) -> Mpf {
    mpf_constant(degree_fixed, prec, rnd)
}
