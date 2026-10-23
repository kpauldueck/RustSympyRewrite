//! Port of the parts of `mpmath.libmp.libhyper` that M2 covers: the generic
//! hypergeometric series summator and the AGM.
//!
//! `make_hyp_summator(key)` writes the source of a summation loop
//! specialised to the parameter types (`Z` integer, `Q` rational, `R` real,
//! `C` complex) and the type of `z`, then `exec`s it. [`hyp_summator`] runs
//! the code it would generate for the given types, statement for
//! statement: the same fixed-point updates in the same order, the same
//! loop exits and the same errors.

use std::collections::HashMap;

use rsympy_num::Int;

use crate::ctx::{CtxError, CtxResult};
use crate::elefun::agm_fixed;
use crate::intmath::bitcount;
use crate::mpc::*;
use crate::mpf::*;

/// A parameter of a hypergeometric series, tagged with its type as
/// `ctx._convert_param` tags it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HypParam {
    /// `'Z'`: a Python int.
    Z(Int),
    /// `'Q'`: an `mpq`, as `(p, q)`.
    Q(Int, Int),
    /// `'R'`: an mpf.
    R(Mpf),
    /// `'C'`: an mpc.
    C(Mpc),
}

impl HypParam {
    /// The type letter.
    pub fn flag(&self) -> char {
        match self {
            HypParam::Z(_) => 'Z',
            HypParam::Q(..) => 'Q',
            HypParam::R(_) => 'R',
            HypParam::C(_) => 'C',
        }
    }
}

/// The series argument: `z._mpf_` (`'R'`) or `z._mpc_` (`'C'`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HypArg {
    R(Mpf),
    C(Mpc),
}

/// The summator's value: a raw mpf, or an mpc when anything is complex.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HypValue {
    Real(Mpf),
    Complex(Mpc),
}

/// `xsign, xm, xe, xbc = x; if xsign: xm = -xm; offset = xe + wp;
/// X = xm << offset` (or `>> -offset`).
fn to_fixed_wp(x: &Mpf, wp: i64) -> Int {
    let xm = if x.sign == 1 { -&x.man } else { x.man.clone() };
    let offset = x.exp + wp;
    if offset >= 0 {
        &xm << offset as u64
    } else {
        &xm >> (-offset) as u64
    }
}

fn fdiv(a: &Int, b: &Int) -> CtxResult<Int> {
    a.floor_div(b)
        .map_err(|_| CtxError::Mpf(MpfError::ZeroDivision))
}

/// The function `make_hyp_summator((p, q, param_types, ztype))` generates,
/// called as `summator(coeffs, z, prec, wp, epsshift, magnitude_check,
/// **kwargs)`; only `maxterms` of the keyword arguments is read. Returns
/// `(value, have_complex, magnitude)` and fills `magnitude_check`.
#[allow(clippy::too_many_arguments)]
pub fn hyp_summator(
    p: usize,
    coeffs: &[HypParam],
    z: &HypArg,
    prec: i64,
    wp: i64,
    epsshift: i64,
    magnitude_check: &mut HashMap<Int, Option<i64>>,
    maxterms: Option<i64>,
) -> CtxResult<(HypValue, bool, i64)> {
    let have_complex_param = coeffs.iter().any(|c| matches!(c, HypParam::C(_)));
    let have_complex_arg = matches!(z, HypArg::C(_));
    let have_complex = have_complex_param || have_complex_arg;

    // Setup code
    let max = maxterms.unwrap_or(wp * 100);
    let high = &Int::ONE << epsshift as u64;
    let low = -&high;
    let one = &Int::ONE << wp as u64;
    let mut sre = one.clone();
    let mut pre = one.clone();
    let mut sim = Int::ZERO;
    let mut pim = Int::ZERO;
    let (zre, zim) = match z {
        HypArg::C((re, im)) => (to_fixed_wp(re, wp), to_fixed_wp(im, wp)),
        HypArg::R(x) => (to_fixed_wp(x, wp), Int::ZERO),
    };

    let mut aint: Vec<Int> = Vec::new();
    let mut bint: Vec<Int> = Vec::new();
    let mut arat: Vec<(Int, Int)> = Vec::new();
    let mut brat: Vec<(Int, Int)> = Vec::new();
    let mut areal: Vec<Int> = Vec::new();
    let mut breal: Vec<Int> = Vec::new();
    let mut acomplex: Vec<(Int, Int)> = Vec::new();
    let mut bcomplex: Vec<(Int, Int)> = Vec::new();
    for (i, c) in coeffs.iter().enumerate() {
        let num = i < p;
        match c {
            HypParam::Z(v) => if num { &mut aint } else { &mut bint }.push(v.clone()),
            HypParam::Q(pp, qq) => {
                if num { &mut arat } else { &mut brat }.push((pp.clone(), qq.clone()))
            }
            HypParam::R(x) => if num { &mut areal } else { &mut breal }.push(to_fixed_wp(x, wp)),
            HypParam::C((re, im)) => if num { &mut acomplex } else { &mut bcomplex }
                .push((to_fixed_wp(re, wp), to_fixed_wp(im, wp))),
        }
    }
    let cancellable_real = areal.len().min(breal.len());
    let has_multiplier = !(aint.is_empty() && arat.is_empty() && brat.is_empty());

    // LOOP
    for n in 1..100_000_000i64 {
        let key = Int::from(n);
        if magnitude_check.contains_key(&key) {
            let mut p_mag = bitcount(&pre.abs());
            if have_complex {
                p_mag = p_mag.max(bitcount(&pim.abs()));
            }
            magnitude_check.insert(key, Some(wp - p_mag));
        }

        // Real factors
        let mul = aint
            .iter()
            .chain(arat.iter().map(|(p, _)| p))
            .chain(brat.iter().map(|(_, q)| q))
            .fold(Int::ONE, |acc, v| &acc * v);
        let div = bint
            .iter()
            .chain(brat.iter().map(|(p, _)| p))
            .chain(arat.iter().map(|(_, q)| q))
            .fold(Int::ONE, |acc, v| &acc * v);
        let div = &div * &Int::from(n);

        // Check for singular terms
        if div.is_zero() {
            if has_multiplier && mul.is_zero() {
                break;
            }
            return Err(CtxError::Mpf(MpfError::ZeroDivision));
        }

        // Update product
        if have_complex {
            for k in 0..cancellable_real {
                pre = fdiv(&(&pre * &areal[k]), &breal[k])?;
            }
            for a in &areal[cancellable_real..] {
                pre = &(&pre * a) >> wp as u64;
            }
            for b in &breal[cancellable_real..] {
                pre = fdiv(&(&pre << wp as u64), b)?;
            }
            for k in 0..cancellable_real {
                pim = fdiv(&(&pim * &areal[k]), &breal[k])?;
            }
            for a in &areal[cancellable_real..] {
                pim = &(&pim * a) >> wp as u64;
            }
            for b in &breal[cancellable_real..] {
                pim = fdiv(&(&pim << wp as u64), b)?;
            }
            if has_multiplier {
                if have_complex_arg {
                    let nre = fdiv(&(&mul * &(&(&pre * &zre) - &(&pim * &zim))), &div)?;
                    let nim = fdiv(&(&mul * &(&(&pim * &zre) + &(&pre * &zim))), &div)?;
                    pre = &nre >> wp as u64;
                    pim = &nim >> wp as u64;
                } else {
                    pre = fdiv(&(&(&(&mul * &pre) * &zre) >> wp as u64), &div)?;
                    pim = fdiv(&(&(&(&mul * &pim) * &zre) >> wp as u64), &div)?;
                }
            } else if have_complex_arg {
                let nre = fdiv(&(&(&pre * &zre) - &(&pim * &zim)), &div)?;
                let nim = fdiv(&(&(&pim * &zre) + &(&pre * &zim)), &div)?;
                pre = &nre >> wp as u64;
                pim = &nim >> wp as u64;
            } else {
                pre = fdiv(&(&(&pre * &zre) >> wp as u64), &div)?;
                pim = fdiv(&(&(&pim * &zre) >> wp as u64), &div)?;
            }
            for (cre, cim) in &acomplex {
                let nre = &(&pre * cre) - &(&pim * cim);
                let nim = &(&pim * cre) + &(&pre * cim);
                pre = &nre >> wp as u64;
                pim = &nim >> wp as u64;
            }
            for (cre, cim) in &bcomplex {
                let mag = &(cre * cre) + &(cim * cim);
                let re = &(&pre * cre) + &(&pim * cim);
                let im = &(&pim * cre) - &(&pre * cim);
                pre = fdiv(&(&re << wp as u64), &mag)?;
                pim = fdiv(&(&im << wp as u64), &mag)?;
            }
        } else {
            for k in 0..cancellable_real {
                pre = fdiv(&(&pre * &areal[k]), &breal[k])?;
            }
            for a in &areal[cancellable_real..] {
                pre = &(&pre * a) >> wp as u64;
            }
            for b in &breal[cancellable_real..] {
                pre = fdiv(&(&pre << wp as u64), b)?;
            }
            if has_multiplier {
                pre = fdiv(&(&(&(&pre * &mul) * &zre) >> wp as u64), &div)?;
            } else {
                pre = fdiv(&(&(&pre * &zre) >> wp as u64), &div)?;
            }
        }

        // Add product to sum
        if have_complex {
            sre = &sre + &pre;
            sim = &sim + &pim;
            if high > pre && pre > low && high > pim && pim > low {
                break;
            }
        } else {
            sre = &sre + &pre;
            if high > pre && pre > low {
                break;
            }
        }

        if n > max {
            return Err(CtxError::NoConvergence(
                "Hypergeometric series converges too slowly. Try increasing maxterms.".into(),
            ));
        }

        // +1 all parameters for next loop
        for v in aint.iter_mut().chain(bint.iter_mut()) {
            *v = &*v + 1;
        }
        for (pp, qq) in arat.iter_mut().chain(brat.iter_mut()) {
            *pp = &*pp + &*qq;
        }
        for v in areal.iter_mut().chain(breal.iter_mut()) {
            *v = &*v + &one;
        }
        for (cre, _) in acomplex.iter_mut().chain(bcomplex.iter_mut()) {
            *cre = &*cre + &one;
        }
    }

    if have_complex {
        let a = from_man_exp(&sre, -wp, prec, Rnd::Nearest);
        let b = from_man_exp(&sim, -wp, prec, Rnd::Nearest);
        let magn = if !sre.is_zero() {
            if !sim.is_zero() {
                (a.exp + a.bc).max(b.exp + b.bc)
            } else {
                a.exp + a.bc
            }
        } else if !sim.is_zero() {
            b.exp + b.bc
        } else {
            -wp + 1
        };
        Ok((HypValue::Complex((a, b)), true, magn))
    } else {
        let a = from_man_exp(&sre, -wp, prec, Rnd::Nearest);
        let magn = if !sre.is_zero() {
            a.exp + a.bc
        } else {
            -wp + 1
        };
        Ok((HypValue::Real(a), false, magn))
    }
}

/// `mpf_agm(a, b, prec, rnd)`: the arithmetic-geometric mean of
/// non-negative `a`, `b`.
pub fn mpf_agm(a: &Mpf, b: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    if a.sign == 1 || b.sign == 1 {
        return Err(MpfError::ComplexResult("agm of a negative number"));
    }
    // Handle inf, nan or zero in either operand
    if a.man.is_zero() || b.man.is_zero() {
        if *a == FNAN || *b == FNAN {
            return Ok(FNAN);
        }
        if *a == FINF {
            if *b == FZERO {
                return Ok(FNAN);
            }
            return Ok(FINF);
        }
        if *b == FINF {
            if *a == FZERO {
                return Ok(FNAN);
            }
            return Ok(FINF);
        }
        // agm(0,x) = agm(x,0) = 0
        return Ok(FZERO);
    }
    let wp = prec + 20;
    let (mut a, mut b) = (a.clone(), b.clone());
    let mut amag = a.exp + a.bc;
    let mut bmag = b.exp + b.bc;
    let mag_delta = amag - bmag;
    // Reduce to roughly the same magnitude using floating-point AGM
    let mut abs_mag_delta = mag_delta.abs();
    if abs_mag_delta > 10 {
        while abs_mag_delta > 10 {
            let na = mpf_shift(&mpf_add(&a, &b, wp, ROUND_FAST), -1);
            let nb = mpf_sqrt(&mpf_mul(&a, &b, wp, ROUND_FAST), wp, ROUND_FAST)?;
            a = na;
            b = nb;
            abs_mag_delta = abs_mag_delta.div_euclid(2);
        }
        amag = a.exp + a.bc;
        bmag = b.exp + b.bc;
    }
    // Use agm(a,b) = agm(x*a,x*b)/x to obtain a, b ~= 1
    let min_mag = amag.min(bmag);
    let max_mag = amag.max(bmag);
    let mut n = 0;
    // If too small, we lose precision when going to fixed-point
    if min_mag < -8 {
        n = -min_mag;
    }
    // If too large, we waste time using fixed-point with large numbers
    else if max_mag > 20 {
        n = -max_mag;
    }
    if n != 0 {
        a = mpf_shift(&a, n);
        b = mpf_shift(&b, n);
    }
    let af = to_fixed(&a, wp);
    let bf = to_fixed(&b, wp);
    let g = agm_fixed(&af, &bf, wp);
    Ok(from_man_exp(&g, -wp - n, prec, rnd))
}

/// `mpf_agm1(a, prec, rnd)`: `agm(1, a)`.
pub fn mpf_agm1(a: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
    mpf_agm(&FONE, a, prec, rnd)
}

/// `mpf_min_max(seq)`.
pub fn mpf_min_max(seq: &[Mpf]) -> (Mpf, Mpf) {
    let mut min = seq[0].clone();
    let mut max = seq[0].clone();
    for x in &seq[1..] {
        if mpf_lt(x, &min) {
            min = x.clone();
        }
        if mpf_gt(x, &max) {
            max = x.clone();
        }
    }
    (min, max)
}

/// `mpc_agm(a, b, prec, rnd)`: the complex AGM. As in mpmath, the result
/// is the last iterate, not rounded to `prec`.
pub fn mpc_agm(a: &Mpc, b: &Mpc, prec: i64, _rnd: Rnd) -> MpfResult<Mpc> {
    if mpc_is_infnan(a) || mpc_is_infnan(b) {
        return Ok((FNAN, FNAN));
    }
    if *a == MPC_ZERO || *b == MPC_ZERO {
        return Ok((FZERO, FZERO));
    }
    if mpc_neg(a, 0, ROUND_FAST) == *b {
        return Ok((FZERO, FZERO));
    }
    let wp = prec + 20;
    let eps = mpf_shift(&FONE, -wp + 10);
    let (mut a, mut b) = (a.clone(), b.clone());
    loop {
        let a1 = mpc_shift(&mpc_add(&a, &b, wp, ROUND_FAST), -1);
        let b1 = mpc_sqrt(&mpc_mul(&a, &b, wp, ROUND_FAST), wp, ROUND_FAST);
        a = a1;
        b = b1;
        let size = mpf_min_max(&[mpc_abs(&a, 10, ROUND_FAST), mpc_abs(&b, 10, ROUND_FAST)]).1;
        let err = mpc_abs(&mpc_sub(&a, &b, 10, ROUND_FAST), 10, ROUND_FAST);
        if size == FZERO || mpf_lt(&err, &mpf_mul(&eps, &size, 0, ROUND_FAST)) {
            return Ok(a);
        }
    }
}

/// `mpc_agm1(a, prec, rnd)`: `agm(1, a)`.
pub fn mpc_agm1(a: &Mpc, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
    mpc_agm(&MPC_ONE, a, prec, rnd)
}
