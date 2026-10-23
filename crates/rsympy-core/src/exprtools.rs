//! `sympy.core.exprtools` (`factor_terms`, `gcd_terms`, `Factors`, `Term`,
//! `decompose_power`, `_monotonic_sign`, `_mask_nc`, `factor_nc`) and the
//! core-hosted `sympy.simplify.radsimp` helpers `fraction`, `numer` and
//! `denom`.
//!
//! Ported from `sympy/core/exprtools.py`. A few steps need `xreplace`,
//! `subs` or `is_polynomial`, which belong to the subs unit; until it
//! lands they go through [`pending`], which is exact on the inputs that
//! need no traversal and `NotPortedYet` otherwise.

use std::sync::LazyLock;

use rsympy_num::Int;

use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::Expr;
use crate::facts::Fact;
use crate::generated::{ClassId as C, Tag};
use crate::mul::keep_coeff;
use crate::numbers::s;
use crate::ops;
use crate::py::PyDict;

/// `_eps`: `Dummy(positive=True)`, made once when SymPy imports exprtools.
/// Its index only has to differ from every other Dummy's.
static EPS: LazyLock<Expr> = LazyLock::new(|| {
    crate::symbol::dummy_with_index("Dummy_0", u64::MAX, &[(Fact::Positive, true)])
});

/// `_eps`, the positive Dummy `_monotonic_sign` uses for "just above 0".
pub fn eps() -> Expr {
    EPS.clone()
}

/// Stand-ins for the subs unit's methods. Each is exact where SymPy's
/// method has nothing to do, and `NotPortedYet` otherwise.
pub(crate) mod pending {
    use super::*;

    /// `e.xreplace(rule)`: `_xreplace` returns `e` itself for an empty
    /// rule.
    pub fn xreplace(e: &Expr, rule: &[(Expr, Expr)]) -> Expr {
        if rule.is_empty() {
            return e.clone();
        }
        not_ported("Basic.xreplace (the subs unit)")
    }

    /// `e.subs(seq)` for a sequence or a dict of pairs: `e` itself when
    /// there is nothing to replace.
    pub fn subs_seq(e: &Expr, seq: &[(Expr, Expr)]) -> Expr {
        if seq.is_empty() {
            return e.clone();
        }
        not_ported("Basic.subs (the subs unit)")
    }

    /// `e.subs(old, new)`: `e` itself when `old` occurs nowhere in it
    /// (every ported `_eval_subs` declines an `old` it does not contain,
    /// so `_subs` returns `self` at every node).
    pub fn subs(e: &Expr, old: &Expr, new: &Expr) -> Expr {
        let _ = new;
        if !e.has(old) {
            return e.clone();
        }
        not_ported("Basic.subs (the subs unit)")
    }

    /// `e.is_polynomial()`: `True` when `e` has no free symbols (the test
    /// `Expr.is_polynomial` makes before `_eval_is_polynomial`).
    pub fn is_polynomial(e: &Expr) -> bool {
        if e.free_symbols().is_empty() {
            return true;
        }
        not_ported("Expr.is_polynomial (the subs unit)")
    }
}

// ---------------------------------------------------------------------------
// Python comparisons

fn is_float_nan(e: &Expr) -> bool {
    e.as_float().is_some_and(|(m, _)| *m == rsympy_mpf::FNAN)
}

/// Whether `a <= b` or `a >= b` involves a NaN Float, which `Float._Frel`
/// compares as false against a Rational or a Float (`mpf_le`/`mpf_ge`).
fn nan_float_comparison(a: &Expr, b: &Expr) -> bool {
    if !(is_float_nan(a) || is_float_nan(b)) {
        return false;
    }
    let plain = |e: &Expr| e.is_Rational() || e.is_Float();
    if !(plain(a) && plain(b)) {
        not_ported("comparison of a NaN Float with a non-Rational");
    }
    true
}

/// `bool(a < b)` for SymPy objects.
pub(crate) fn py_lt(a: &Expr, b: &Expr) -> bool {
    crate::relational::lt(a, b).truthy()
}

/// `bool(a > b)`: `a.__gt__(b)`, which decides as `b < a` does.
pub(crate) fn py_gt(a: &Expr, b: &Expr) -> bool {
    crate::relational::lt(b, a).truthy()
}

/// `bool(a <= b)`: `not (b < a)`, but false for a NaN Float.
pub(crate) fn py_le(a: &Expr, b: &Expr) -> bool {
    if nan_float_comparison(a, b) {
        return false;
    }
    !py_lt(b, a)
}

/// `bool(a >= b)`: `not (a < b)`, but false for a NaN Float.
pub(crate) fn py_ge(a: &Expr, b: &Expr) -> bool {
    if nan_float_comparison(a, b) {
        return false;
    }
    !py_lt(a, b)
}

// ---------------------------------------------------------------------------
// Exponents: Python ints or SymPy objects

/// A value of a `Factors` dict. SymPy keeps an exponent as a SymPy object
/// or as a Python `int` (`Term` counts powers in a `defaultdict(int)`), and
/// the difference shows: `Factors.as_expr` treats an `Integer` exponent
/// differently from an `int` one, and an `int` has no SymPy methods.
#[derive(Clone, Debug)]
pub enum Exponent {
    /// A Python `int`.
    Int(Int),
    /// A SymPy object.
    Expr(Expr),
}

impl Exponent {
    /// The Python int `i`.
    pub fn int(i: i64) -> Exponent {
        Exponent::Int(Int::small(i))
    }

    /// `sympify(self)`.
    pub fn to_expr(&self) -> Expr {
        match self {
            Exponent::Int(i) => Expr::integer(i.clone()),
            Exponent::Expr(e) => e.clone(),
        }
    }

    /// `_isnumber(self)`: a Python int or a SymPy Number.
    pub fn is_number(&self) -> bool {
        match self {
            Exponent::Int(_) => true,
            Exponent::Expr(e) => e.is_Number(),
        }
    }

    /// `bool(self)`.
    pub fn truthy(&self) -> bool {
        match self {
            Exponent::Int(i) => !i.is_zero(),
            Exponent::Expr(e) => e.truthy(),
        }
    }

    /// `self == other`: an int equals an `Integer` of the same value and
    /// nothing else; SymPy objects compare structurally.
    pub fn py_eq(&self, other: &Exponent) -> bool {
        match (self, other) {
            (Exponent::Int(a), Exponent::Int(b)) => a == b,
            (Exponent::Int(i), Exponent::Expr(e)) | (Exponent::Expr(e), Exponent::Int(i)) => {
                e.is_Integer() && e.p() == *i
            }
            (Exponent::Expr(a), Exponent::Expr(b)) => a == b,
        }
    }

    /// `self + other`: int arithmetic for two ints, else SymPy's (an int
    /// operand is sympified).
    pub fn add(&self, other: &Exponent) -> Exponent {
        match (self, other) {
            (Exponent::Int(a), Exponent::Int(b)) => Exponent::Int(a + b),
            _ => Exponent::Expr(ops::add(&self.to_expr(), &other.to_expr())),
        }
    }

    /// `self - other`.
    pub fn sub(&self, other: &Exponent) -> Exponent {
        match (self, other) {
            (Exponent::Int(a), Exponent::Int(b)) => Exponent::Int(a - b),
            _ => Exponent::Expr(ops::sub(&self.to_expr(), &other.to_expr())),
        }
    }

    /// `-self`.
    pub fn neg(&self) -> Exponent {
        match self {
            Exponent::Int(a) => Exponent::Int(-a),
            Exponent::Expr(e) => Exponent::Expr(ops::neg(e)),
        }
    }

    /// `self*n` for a Python int `n`.
    pub fn mul_int(&self, n: &Int) -> Exponent {
        match self {
            Exponent::Int(a) => Exponent::Int(a * n),
            Exponent::Expr(e) => Exponent::Expr(ops::mul(e, &Expr::integer(n.clone()))),
        }
    }

    /// `bool(self > other)`.
    pub fn gt(&self, other: &Exponent) -> bool {
        match (self, other) {
            (Exponent::Int(a), Exponent::Int(b)) => a > b,
            _ => py_gt(&self.to_expr(), &other.to_expr()),
        }
    }

    /// `bool(self < other)`.
    pub fn lt(&self, other: &Exponent) -> bool {
        match (self, other) {
            (Exponent::Int(a), Exponent::Int(b)) => a < b,
            _ => py_lt(&self.to_expr(), &other.to_expr()),
        }
    }

    /// `bool(self <= other)`.
    pub fn le(&self, other: &Exponent) -> bool {
        match (self, other) {
            (Exponent::Int(a), Exponent::Int(b)) => a <= b,
            _ => py_le(&self.to_expr(), &other.to_expr()),
        }
    }

    /// `bool(self >= other)`.
    pub fn ge(&self, other: &Exponent) -> bool {
        match (self, other) {
            (Exponent::Int(a), Exponent::Int(b)) => a >= b,
            _ => py_ge(&self.to_expr(), &other.to_expr()),
        }
    }

    /// `self.extract_additively(c)`, which a Python int does not have.
    fn extract_additively(&self, c: &Exponent) -> Option<Expr> {
        match self {
            Exponent::Int(_) => int_has_no("extract_additively"),
            Exponent::Expr(e) => e.extract_additively(&c.to_expr()),
        }
    }

    /// `self.as_coeff_Add()`, which a Python int does not have.
    #[allow(non_snake_case)]
    fn as_coeff_Add(&self) -> (Expr, Expr) {
        match self {
            Exponent::Int(_) => int_has_no("as_coeff_Add"),
            Exponent::Expr(e) => e.as_coeff_Add(false),
        }
    }
}

fn int_has_no(attr: &str) -> ! {
    raise(
        ErrorKind::AttributeError,
        format!("'int' object has no attribute '{attr}'"),
    )
}

/// `_isnumber(i)`: `isinstance(i, (SYMPY_INTS, float)) or i.is_Number`.
pub fn isnumber(i: &Exponent) -> bool {
    i.is_number()
}

// ---------------------------------------------------------------------------
// _monotonic_sign

/// `_monotonic_sign(expr)`: the closest number to zero that `expr` can
/// reach, with `_eps` standing for "just above 0" (None if unknown).
pub fn monotonic_sign(e: &Expr) -> Option<Expr> {
    if e.is_extended_real() != Some(true) {
        return None;
    }

    let neg = ops::neg(e);
    if neg.is_Symbol() {
        let rv = monotonic_sign(&neg);
        return rv.map(|rv| ops::neg(&rv));
    }

    if !e.is_Add() && e.as_numer_denom().1.is_number() {
        let s = e;
        let int = |n: i64| Some(Expr::integer(n));
        if s.is_prime() == Some(true) {
            return if s.is_odd() == Some(true) {
                int(3)
            } else {
                int(2)
            };
        } else if s.is_composite() == Some(true) {
            return if s.is_odd() == Some(true) {
                int(9)
            } else {
                int(4)
            };
        } else if s.is_positive() == Some(true) {
            if s.is_even() == Some(true) {
                return if s.is_prime() == Some(false) {
                    int(4)
                } else {
                    int(2)
                };
            } else if s.is_integer() == Some(true) {
                return Some(s::one());
            } else {
                return Some(eps());
            }
        } else if s.is_extended_negative() == Some(true) {
            if s.is_even() == Some(true) {
                return int(-2);
            } else if s.is_integer() == Some(true) {
                return Some(s::negative_one());
            } else {
                return Some(ops::neg(&eps()));
            }
        }
        if s.is_zero() == Some(true)
            || s.is_extended_nonpositive() == Some(true)
            || s.is_extended_nonnegative() == Some(true)
        {
            return Some(s::zero());
        }
        return None;
    }

    // univariate polynomial
    let free = e.free_symbols();
    if free.len() == 1 {
        if pending::is_polynomial(e) {
            let x = &free[0];
            let mut x0 = monotonic_sign(x);
            if x0
                .as_ref()
                .is_some_and(|v| *v == eps() || *v == ops::neg(&eps()))
            {
                x0 = Some(s::zero());
            }
            if x0.is_some() {
                not_ported(
                    "_monotonic_sign of a univariate polynomial (needs diff and real_roots)",
                );
            }
        } else {
            let (n, d) = e.as_numer_denom();
            let mut den = None;
            if n.is_number() {
                den = monotonic_sign(&d);
            } else if !d.is_number() && monotonic_sign(&n).is_some() {
                den = monotonic_sign(&d);
            }
            if let Some(den) = den
                && (den.is_positive() == Some(true) || den.is_negative() == Some(true))
            {
                let v = ops::mul(&n, &den);
                let dummy = |name: &str, fact: Fact| {
                    Some(crate::symbol::dummy(Some(name), &[(fact, true)]))
                };
                if v.is_positive() == Some(true) {
                    return dummy("pos", Fact::Positive);
                } else if v.is_nonnegative() == Some(true) {
                    return dummy("nneg", Fact::Nonnegative);
                } else if v.is_negative() == Some(true) {
                    return dummy("neg", Fact::Negative);
                } else if v.is_nonpositive() == Some(true) {
                    return dummy("npos", Fact::Nonpositive);
                }
            }
        }
        return None;
    }

    // multivariate
    let (c, a) = e.as_coeff_Add(false);
    let mut v: Option<Expr> = None;
    if !pending::is_polynomial(&a) {
        // F/A or A/F where A is a number and F is a signed, rational monomial
        let (n, d) = a.as_numer_denom();
        if !(n.is_number() || d.is_number()) {
            return None;
        }
        if (a.is_Mul() || a.is_Pow())
            && a.is_rational() == Some(true)
            && a.atoms_where(|p| p.is_cls(C::Pow))
                .iter()
                .filter(|p| p.is_Pow())
                .all(|p| p.arg(1).is_Integer())
            && (a.is_positive() == Some(true) || a.is_negative() == Some(true))
        {
            let mut vv = s::one();
            for ai in a.mul_args() {
                if ai.is_number() {
                    vv = ops::mul(&vv, &ai);
                    continue;
                }
                let mut reps = Vec::new();
                for x in ai.free_symbols() {
                    let r = monotonic_sign(&x)?;
                    reps.push((x, r));
                }
                vv = ops::mul(&vv, &pending::subs_seq(&ai, &reps));
            }
            v = Some(vv);
        }
    } else if c.truthy() {
        // signed linear expression
        if !a
            .atoms_where(|p| p.is_cls(C::Pow))
            .iter()
            .any(|p| !p.is_number())
            && (a.is_nonpositive() == Some(true) || a.is_nonnegative() == Some(true))
        {
            let mut p = Vec::new();
            for i in a.free_symbols() {
                let vi = monotonic_sign(&i)?;
                let vi = if vi.truthy() {
                    vi
                } else if i.is_nonnegative() == Some(true) {
                    eps()
                } else {
                    ops::neg(&eps())
                };
                p.push((i, vi));
            }
            v = Some(pending::xreplace(&a, &p));
        }
    }
    if let Some(v) = v {
        let rv = ops::add(&v, &c);
        if v.is_nonnegative() == Some(true) && rv.is_positive() == Some(true) {
            return Some(pending::subs(&rv, &eps(), &s::zero()));
        }
        if v.is_nonpositive() == Some(true) && rv.is_negative() == Some(true) {
            return Some(pending::subs(&rv, &eps(), &s::zero()));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// decompose_power

/// `decompose_power(expr)`: `(base, integer exponent)`, the exponent a
/// Python int.
pub fn decompose_power_int(e: &Expr) -> (Expr, Int) {
    let expr = e;
    let (mut base, exp) = expr.as_base_exp();
    if exp.is_Number() {
        if exp.is_Rational() {
            if !exp.is_Integer() {
                base = crate::power::pow(&base, &Expr::rational(1, exp.q()));
            }
            (base, exp.p())
        } else {
            (expr.clone(), Int::ONE)
        }
    } else {
        let (exp, tail) = exp.as_coeff_Mul(true);
        if exp.is_cls(C::NegativeOne) {
            (crate::power::pow(&base, &tail), Int::NEG_ONE)
        } else if !exp.is_cls(C::One) {
            let tail = keep_coeff(&Expr::rational(1, exp.q()), &tail, true, false);
            (crate::power::pow(&base, &tail), exp.p())
        } else {
            (expr.clone(), Int::ONE)
        }
    }
}

/// `decompose_power(expr)` with the exponent as an `i64`; an exponent
/// beyond `i64` is `NotPortedYet` here.
pub fn decompose_power(e: &Expr) -> (Expr, i64) {
    let (base, e) = decompose_power_int(e);
    let e = e
        .as_i64()
        .unwrap_or_else(|| not_ported("decompose_power: an exponent beyond i64"));
    (base, e)
}

/// `decompose_power_rat(expr)`: `(base, Rational exponent)`; a non-Rational
/// exponent only has its integer coefficient separated.
pub fn decompose_power_rat(e: &Expr) -> (Expr, Expr) {
    let (base, exp) = e.as_base_exp();
    if !exp.is_Rational() {
        let (base, exp_i) = decompose_power_int(e);
        return (base, Expr::integer(exp_i));
    }
    (base, exp)
}

// ---------------------------------------------------------------------------
// Factors

/// `Factors`: an efficient representation of `f_1*f_2*...*f_n` as a dict
/// `{base: exponent}`.
#[derive(Clone)]
pub struct Factors {
    /// `self.factors`, in insertion order.
    pub factors: PyDict<Expr, Exponent>,
    /// Whether `factors` is a `defaultdict(int)` (what `Term` builds, and
    /// what `dict.copy()` of one keeps): looking up a missing key then
    /// inserts and returns 0. Only `Factors.normal` looks keys up that way.
    pub defaultdict: bool,
}

impl std::fmt::Debug for Factors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Factors(")?;
        f.debug_map().entries(self.factors.iter()).finish()?;
        f.write_str(")")
    }
}

impl Default for Factors {
    fn default() -> Self {
        Factors::new()
    }
}

fn one_exp() -> Exponent {
    Exponent::Expr(s::one())
}

/// `del d[k]`.
fn del(d: &mut PyDict<Expr, Exponent>, k: &Expr) {
    if d.remove(k).is_none() {
        raise(ErrorKind::KeyError, format!("{k:?}"));
    }
}

/// `d[k]`.
fn getitem(d: &PyDict<Expr, Exponent>, k: &Expr) -> Exponent {
    d.get(k)
        .cloned()
        .unwrap_or_else(|| raise(ErrorKind::KeyError, format!("{k:?}")))
}

impl Factors {
    /// `Factors()` (also `Factors(None)` and `Factors(S.One)`).
    pub fn new() -> Factors {
        Factors {
            factors: PyDict::new(),
            defaultdict: false,
        }
    }

    /// `Factors(S.Zero)`: `{0: 1}`.
    fn zero() -> Factors {
        let mut factors = PyDict::new();
        factors.insert(s::zero(), one_exp());
        Factors {
            factors,
            defaultdict: false,
        }
    }

    /// `Factors(expr)` for a SymPy object (a Python int or float is
    /// sympified first, so pass its SymPy number).
    pub fn from_expr(factors: &Expr) -> Factors {
        if factors.is_cls(C::One) {
            // factors in (None, S.One)
            return Factors::new();
        }
        if factors.is_cls(C::Zero) {
            // factors is S.Zero or factors == 0
            return Factors::zero();
        }
        if factors.is_Number() {
            let mut n = factors.clone();
            let mut d = PyDict::new();
            if py_lt(&n, &s::zero()) {
                d.insert(s::negative_one(), one_exp());
                n = ops::neg(&n);
            }
            if !n.is_cls(C::One) {
                if n.is_Float() || n.is_Integer() || n.is_cls(C::Infinity) {
                    d.insert(n, one_exp());
                } else if n.is_Rational() {
                    // since we're processing Numbers, the denominator is
                    // stored with a negative exponent; all other factors
                    // are left .
                    if !n.p().is_one() {
                        d.insert(Expr::integer(n.p()), one_exp());
                    }
                    d.insert(Expr::integer(n.q()), Exponent::Expr(s::negative_one()));
                } else {
                    raise(
                        ErrorKind::ValueError,
                        format!("Expected Float|Rational|Integer, not {n:?}"),
                    );
                }
            }
            return Factors {
                factors: d,
                defaultdict: false,
            };
        }
        if factors.args().is_empty() {
            let mut d = PyDict::new();
            d.insert(factors.clone(), one_exp());
            return Factors {
                factors: d,
                defaultdict: false,
            };
        }
        if !factors.is_a(Tag::Expr) {
            // `factors.copy()` (Basic.copy) and then indexing it as a dict
            not_ported("Factors of a non-Expr Basic with args");
        }
        let (mut c, nc) = factors.args_cnc(true);
        let i = c.iter().filter(|a| a.is_cls(C::ImaginaryUnit)).count();
        c.retain(|a| !a.is_cls(C::ImaginaryUnit));
        let mut d = crate::mul::from_args(c, None).as_powers_dict();
        // Handle all rational Coefficients
        let keys: Vec<Expr> = d.keys().cloned().collect();
        for f in keys {
            if f.is_Rational() && !f.is_Integer() {
                let (p, q) = (Expr::integer(f.p()), Expr::integer(f.q()));
                let vp = d
                    .get(&p)
                    .cloned()
                    .unwrap_or(Exponent::Expr(s::zero()))
                    .add(&getitem(&d, &f));
                d.insert(p, vp);
                let vq = d
                    .get(&q)
                    .cloned()
                    .unwrap_or(Exponent::Expr(s::zero()))
                    .sub(&getitem(&d, &f));
                d.insert(q, vq);
                d.remove(&f);
            }
        }
        if i > 0 {
            let im = s::imaginary_unit();
            let v = d
                .get(&im)
                .cloned()
                .unwrap_or(Exponent::Expr(s::zero()))
                .add(&Exponent::Int(Int::from(i)));
            d.insert(im, v);
        }
        if !nc.is_empty() {
            d.insert(crate::mul::mul_with(nc, false), one_exp());
        }
        Factors {
            factors: d,
            defaultdict: false,
        }
    }

    /// `Factors(d)` for a dict (`defaultdict` when `d` is a
    /// `defaultdict(int)`): a copy, with numeric powers of -1, 1 and I
    /// made canonical.
    pub fn from_dict(d: &PyDict<Expr, Exponent>, defaultdict: bool) -> Factors {
        let mut factors = d.clone();

        // tidy up -/+1 and I exponents if Rational

        let handle: Vec<Expr> = factors
            .keys()
            .filter(|k| k.is_cls(C::ImaginaryUnit) || k.is_cls(C::NegativeOne) || k.is_cls(C::One))
            .cloned()
            .collect();
        if !handle.is_empty() {
            let mut i1 = s::one();
            for k in &handle {
                if !getitem(&factors, k).is_number() {
                    continue;
                }
                let v = factors.remove(k).expect("a handled key");
                i1 = ops::mul(&i1, &ops::pow(k, &v.to_expr()));
            }
            if !i1.is_cls(C::One) {
                // at worst, -1.0*I*(-1)**e
                let parts = if i1.is_Mul() {
                    i1.args().to_vec()
                } else {
                    vec![i1.clone()]
                };
                for a in parts {
                    if a.is_cls(C::NegativeOne) {
                        factors.insert(a, one_exp());
                    } else if a.is_cls(C::ImaginaryUnit) {
                        factors.insert(s::imaginary_unit(), one_exp());
                    } else if a.is_Pow() {
                        let b = a.arg(0).clone();
                        let v = factors
                            .get(&b)
                            .cloned()
                            .unwrap_or(Exponent::Expr(s::zero()))
                            .add(&Exponent::Expr(a.arg(1).clone()));
                        factors.insert(b, v);
                    } else if crate::numbers::equal_valued(&a, &s::one()) {
                        factors.insert(a, one_exp());
                    } else if crate::numbers::equal_valued(&a, &s::negative_one()) {
                        factors.insert(ops::neg(&a), one_exp());
                        factors.insert(s::negative_one(), one_exp());
                    } else {
                        raise(
                            ErrorKind::ValueError,
                            format!("unexpected factor in i1: {a:?}"),
                        );
                    }
                }
            }
        }
        Factors {
            factors,
            defaultdict,
        }
    }

    /// `self.gens`: the bases (a frozenset in SymPy; here in dict order).
    pub fn gens(&self) -> Vec<Expr> {
        self.factors.keys().cloned().collect()
    }

    /// `self.is_zero`.
    pub fn is_zero(&self) -> bool {
        self.factors.len() == 1 && self.factors.contains_key(&s::zero())
    }

    /// `self.is_one`.
    pub fn is_one(&self) -> bool {
        self.factors.is_empty()
    }

    /// `self.as_expr()`: the underlying expression.
    pub fn as_expr(&self) -> Expr {
        let mut args = Vec::new();
        for (factor, exp) in self.factors.iter() {
            if !exp.py_eq(&Exponent::int(1)) {
                match exp {
                    Exponent::Expr(x) if x.is_Integer() => {
                        let (b, e) = factor.as_base_exp();
                        let e = keep_coeff(x, &e, true, false);
                        args.push(ops::pow(&b, &e));
                    }
                    _ => args.push(ops::pow(factor, &exp.to_expr())),
                }
            } else {
                args.push(factor.clone());
            }
        }
        crate::mul::mul(args)
    }

    /// `self.mul(other)`: Factors of `self*other`.
    pub fn mul(&self, other: &Factors) -> Factors {
        if self.is_zero() || other.is_zero() {
            return Factors::zero();
        }
        let mut factors = self.factors.clone();

        for (factor, exp) in other.factors.iter() {
            let mut exp = exp.clone();
            if let Some(cur) = factors.get(factor) {
                exp = cur.add(&exp);

                if !exp.truthy() {
                    factors.remove(factor);
                    continue;
                }
            }

            factors.insert(factor.clone(), exp);
        }

        Factors::from_dict(&factors, false)
    }

    /// `self.mul(other)` for a SymPy object `other` (`Factors(other)`).
    pub fn mul_expr(&self, other: &Expr) -> Factors {
        self.mul(&Factors::from_expr(other))
    }

    /// `self.normal(other)`: `self` and `other` with their gcd removed.
    /// Looking up a factor missing from a defaultdict `other` inserts 0
    /// into it, as in SymPy.
    pub fn normal(&self, other: &mut Factors) -> (Factors, Factors) {
        let mut self_factors = self.factors.clone();
        let mut other_factors = other.factors.clone();

        for (factor, self_exp) in self.factors.iter() {
            let other_exp = match other.factors.get(factor) {
                Some(v) => v.clone(),
                None if other.defaultdict => {
                    other.factors.insert(factor.clone(), Exponent::int(0));
                    Exponent::int(0)
                }
                None => continue,
            };

            let exp = self_exp.sub(&other_exp);

            if !exp.truthy() {
                del(&mut self_factors, factor);
                del(&mut other_factors, factor);
            } else if exp.is_number() {
                if exp.gt(&Exponent::int(0)) {
                    self_factors.insert(factor.clone(), exp);
                    del(&mut other_factors, factor);
                } else {
                    del(&mut self_factors, factor);
                    other_factors.insert(factor.clone(), exp.neg());
                }
            } else {
                let r = self_exp.extract_additively(&other_exp);
                if let Some(r) = r {
                    if r.truthy() {
                        self_factors.insert(factor.clone(), Exponent::Expr(r));
                        del(&mut other_factors, factor);
                    } else {
                        // should be handled already
                        del(&mut self_factors, factor);
                        del(&mut other_factors, factor);
                    }
                } else {
                    let mut other_exp = other_exp;
                    let (sc, sa) = self_exp.as_coeff_Add();
                    if sc.truthy() {
                        let (oc, oa) = other_exp.as_coeff_Add();
                        let diff = ops::sub(&sc, &oc);
                        if py_gt(&diff, &s::zero()) {
                            let v = getitem(&self_factors, factor).sub(&Exponent::Expr(oc));
                            self_factors.insert(factor.clone(), v);
                            other_exp = Exponent::Expr(oa);
                        } else if py_lt(&diff, &s::zero()) {
                            let sce = Exponent::Expr(sc);
                            let v = getitem(&self_factors, factor).sub(&sce);
                            self_factors.insert(factor.clone(), v);
                            let v = getitem(&other_factors, factor).sub(&sce);
                            other_factors.insert(factor.clone(), v);
                            other_exp = Exponent::Expr(ops::sub(&oa, &diff));
                        } else {
                            self_factors.insert(factor.clone(), Exponent::Expr(sa));
                            other_exp = Exponent::Expr(oa);
                        }
                    }
                    if other_exp.truthy() {
                        other_factors.insert(factor.clone(), other_exp);
                    } else {
                        del(&mut other_factors, factor);
                    }
                }
            }
        }

        (
            Factors::from_dict(&self_factors, false),
            Factors::from_dict(&other_factors, false),
        )
    }

    /// `self.normal(other)` for a SymPy object `other`.
    pub fn normal_expr(&self, other: &Expr) -> (Factors, Factors) {
        let mut other = Factors::from_expr(other);
        if other.is_zero() {
            return (Factors::new(), Factors::zero());
        }
        if self.is_zero() {
            return (Factors::zero(), Factors::new());
        }
        self.normal(&mut other)
    }

    /// `self.div(other)`: `self` and `other` with their gcd removed (`/`
    /// gives the first).
    pub fn div(&self, other: &Factors) -> (Factors, Factors) {
        let mut quo = self.factors.clone();
        let mut rem: PyDict<Expr, Exponent> = PyDict::new();

        for (factor, exp) in other.factors.iter() {
            let mut exp = exp.clone();
            if let Some(q) = quo.get(factor).cloned() {
                let d = q.sub(&exp);
                if d.is_number() {
                    if d.le(&Exponent::int(0)) {
                        del(&mut quo, factor);
                    }

                    if d.ge(&Exponent::int(0)) {
                        if d.truthy() {
                            quo.insert(factor.clone(), d);
                        }

                        continue;
                    }

                    exp = d.neg();
                } else {
                    let r = q.extract_additively(&exp);
                    if let Some(r) = r {
                        if r.truthy() {
                            quo.insert(factor.clone(), Exponent::Expr(r));
                        } else {
                            // should be handled already
                            del(&mut quo, factor);
                        }
                    } else {
                        let mut other_exp = exp;
                        let (sc, sa) = getitem(&quo, factor).as_coeff_Add();
                        if sc.truthy() {
                            let (oc, oa) = other_exp.as_coeff_Add();
                            let diff = ops::sub(&sc, &oc);
                            if py_gt(&diff, &s::zero()) {
                                let v = getitem(&quo, factor).sub(&Exponent::Expr(oc));
                                quo.insert(factor.clone(), v);
                                other_exp = Exponent::Expr(oa);
                            } else if py_lt(&diff, &s::zero()) {
                                let v = getitem(&quo, factor).sub(&Exponent::Expr(sc));
                                quo.insert(factor.clone(), v);
                                other_exp = Exponent::Expr(ops::sub(&oa, &diff));
                            } else {
                                quo.insert(factor.clone(), Exponent::Expr(sa));
                                other_exp = Exponent::Expr(oa);
                            }
                        }
                        if other_exp.truthy() {
                            rem.insert(factor.clone(), other_exp);
                        } else if rem.contains_key(factor) {
                            raise(ErrorKind::AssertionError, "");
                        }
                    }
                    continue;
                }
            }

            rem.insert(factor.clone(), exp);
        }

        (
            Factors::from_dict(&quo, false),
            Factors::from_dict(&rem, false),
        )
    }

    /// `self.div(other)` for a SymPy object `other`.
    pub fn div_expr(&self, other: &Expr) -> (Factors, Factors) {
        let other = Factors::from_expr(other);
        if other.is_zero() {
            raise(ErrorKind::ZeroDivisionError, "");
        }
        if self.is_zero() {
            return (Factors::zero(), Factors::new());
        }
        self.div(&other)
    }

    /// `self.quo(other)`: the numerator Factors of `self/other`.
    pub fn quo(&self, other: &Factors) -> Factors {
        self.div(other).0
    }

    /// `self.quo(other)` for a SymPy object `other`.
    pub fn quo_expr(&self, other: &Expr) -> Factors {
        self.div_expr(other).0
    }

    /// `self.rem(other)`: the denominator Factors of `self/other`.
    pub fn rem(&self, other: &Factors) -> Factors {
        self.div(other).1
    }

    /// `self.rem(other)` for a SymPy object `other`.
    pub fn rem_expr(&self, other: &Expr) -> Factors {
        self.div_expr(other).1
    }

    /// `self.pow(other)` for a Python int `other`.
    pub fn pow(&self, other: &Int) -> Factors {
        if other.is_negative() {
            raise(
                ErrorKind::ValueError,
                format!("expected non-negative integer, got {other}"),
            );
        }
        let mut factors = PyDict::new();

        if !other.is_zero() {
            for (factor, exp) in self.factors.iter() {
                factors.insert(factor.clone(), exp.mul_int(other));
            }
        }

        Factors::from_dict(&factors, false)
    }

    /// `self.pow(other)` for a Factors `other`: its expression, which must
    /// be a non-negative Integer.
    pub fn pow_factors(&self, other: &Factors) -> Factors {
        let other = other.as_expr();
        if other.is_Integer() {
            return self.pow(&other.p());
        }
        raise(
            ErrorKind::ValueError,
            format!("expected non-negative integer, got {other:?}"),
        )
    }

    /// `self.pow(other)` for a SymPy object: only Python ints are accepted.
    pub fn pow_expr(&self, other: &Expr) -> Factors {
        raise(
            ErrorKind::ValueError,
            format!("expected non-negative integer, got {other:?}"),
        )
    }

    /// `self.gcd(other)`: the common factors with the smaller exponent.
    pub fn gcd(&self, other: &Factors) -> Factors {
        let mut factors = PyDict::new();

        for (factor, exp) in self.factors.iter() {
            let exp = exp.to_expr();
            if let Some(oexp) = other.factors.get(factor) {
                let lt = ops::sub(&exp, &oexp.to_expr()).is_negative();
                if lt == Some(true) {
                    factors.insert(factor.clone(), Exponent::Expr(exp));
                } else if lt == Some(false) {
                    factors.insert(factor.clone(), oexp.clone());
                }
            }
        }

        Factors::from_dict(&factors, false)
    }

    /// `self.gcd(other)` for a SymPy object `other`.
    pub fn gcd_expr(&self, other: &Expr) -> Factors {
        let other = Factors::from_expr(other);
        if other.is_zero() {
            return Factors::from_dict(&self.factors, self.defaultdict);
        }
        self.gcd(&other)
    }

    /// `self.lcm(other)`: the union of the factors with the larger
    /// exponent.
    pub fn lcm(&self, other: &Factors) -> Factors {
        let mut factors = self.factors.clone();

        for (factor, exp) in other.factors.iter() {
            let mut exp = exp.clone();
            if let Some(cur) = factors.get(factor) {
                // max(exp, factors[factor])
                if cur.gt(&exp) {
                    exp = cur.clone();
                }
            }

            factors.insert(factor.clone(), exp);
        }

        Factors::from_dict(&factors, false)
    }

    /// `self.lcm(other)` for a SymPy object `other`.
    pub fn lcm_expr(&self, other: &Expr) -> Factors {
        let other = Factors::from_expr(other);
        if self.is_zero() || other.is_zero() {
            return Factors::zero();
        }
        self.lcm(&other)
    }

    /// `self == other`: dict equality of the factors.
    pub fn py_eq(&self, other: &Factors) -> bool {
        self.factors.len() == other.factors.len()
            && self
                .factors
                .iter()
                .all(|(k, v)| other.factors.get(k).is_some_and(|w| v.py_eq(w)))
    }

    /// `self == other` for a SymPy object `other`.
    pub fn py_eq_expr(&self, other: &Expr) -> bool {
        self.py_eq(&Factors::from_expr(other))
    }
}

// ---------------------------------------------------------------------------
// Term

/// `Term`: an efficient representation of `coeff*(numer/denom)`.
#[derive(Clone, Debug)]
pub struct Term {
    pub coeff: Expr,
    pub numer: Factors,
    pub denom: Factors,
}

/// `a.gcd(b)` for a Term's coefficient.
fn coeff_gcd(a: &Expr, b: &Expr) -> Expr {
    if !a.is_Number() {
        raise(
            ErrorKind::AttributeError,
            format!("'{}' object has no attribute 'gcd'", a.class_name()),
        );
    }
    crate::numbers::number_gcd(a, b)
}

/// `a.lcm(b)` for a Term's coefficient.
fn coeff_lcm(a: &Expr, b: &Expr) -> Expr {
    if !a.is_Number() {
        raise(
            ErrorKind::AttributeError,
            format!("'{}' object has no attribute 'lcm'", a.class_name()),
        );
    }
    crate::numbers::number_lcm(a, b)
}

impl Term {
    /// `Term(term)`: splits a commutative expression into its coefficient
    /// and the factors of its numerator and denominator.
    pub fn new(term: &Expr) -> Term {
        if term.is_commutative() != Some(true) {
            raise(
                ErrorKind::NonCommutativeExpression,
                "commutative expression expected",
            );
        }

        let (mut coeff, factors) = term.as_coeff_mul();
        let mut numer: PyDict<Expr, Exponent> = PyDict::new();
        let mut denom: PyDict<Expr, Exponent> = PyDict::new();

        for factor in factors {
            let (mut base, exp) = decompose_power_int(&factor);

            if base.is_Add() {
                let (cont, b) = base.primitive();
                base = b;
                coeff = ops::mul(&coeff, &ops::pow(&cont, &Expr::integer(exp.clone())));
            }

            if exp > 0 {
                let v = numer.setdefault_with(base, || Exponent::int(0));
                *v = v.add(&Exponent::Int(exp));
            } else {
                let v = denom.setdefault_with(base, || Exponent::int(0));
                *v = v.add(&Exponent::Int(-exp));
            }
        }

        Term {
            coeff,
            numer: Factors::from_dict(&numer, true),
            denom: Factors::from_dict(&denom, true),
        }
    }

    /// `Term(coeff, numer, denom)` with either Factors given (an empty
    /// `Factors()` for one left out).
    pub fn from_parts(coeff: &Expr, numer: Option<Factors>, denom: Option<Factors>) -> Term {
        Term {
            coeff: coeff.clone(),
            numer: numer.unwrap_or_default(),
            denom: denom.unwrap_or_default(),
        }
    }

    /// `self.as_expr()`.
    pub fn as_expr(&self) -> Expr {
        ops::mul(
            &self.coeff,
            &ops::div(&self.numer.as_expr(), &self.denom.as_expr()),
        )
    }

    /// `self.mul(other)`.
    pub fn mul(&self, other: &Term) -> Term {
        let coeff = ops::mul(&self.coeff, &other.coeff);
        let numer = self.numer.mul(&other.numer);
        let mut denom = self.denom.mul(&other.denom);

        let (numer, denom) = numer.normal(&mut denom);

        Term {
            coeff,
            numer,
            denom,
        }
    }

    /// `self.inv()`.
    pub fn inv(&self) -> Term {
        Term {
            coeff: ops::div(&s::one(), &self.coeff),
            numer: self.denom.clone(),
            denom: self.numer.clone(),
        }
    }

    /// `self.quo(other)`.
    pub fn quo(&self, other: &Term) -> Term {
        self.mul(&other.inv())
    }

    /// `self.pow(other)` for a Python int `other`.
    pub fn pow(&self, other: &Int) -> Term {
        if other.is_negative() {
            self.inv().pow(&-other)
        } else {
            Term {
                coeff: ops::pow(&self.coeff, &Expr::integer(other.clone())),
                numer: self.numer.pow(other),
                denom: self.denom.pow(other),
            }
        }
    }

    /// `self.gcd(other)`.
    pub fn gcd(&self, other: &Term) -> Term {
        Term {
            coeff: coeff_gcd(&self.coeff, &other.coeff),
            numer: self.numer.gcd(&other.numer),
            denom: self.denom.gcd(&other.denom),
        }
    }

    /// `self.lcm(other)`.
    pub fn lcm(&self, other: &Term) -> Term {
        Term {
            coeff: coeff_lcm(&self.coeff, &other.coeff),
            numer: self.numer.lcm(&other.numer),
            denom: self.denom.lcm(&other.denom),
        }
    }

    /// `self == other`.
    pub fn py_eq(&self, other: &Term) -> bool {
        self.coeff == other.coeff
            && self.numer.py_eq(&other.numer)
            && self.denom.py_eq(&other.denom)
    }
}

// ---------------------------------------------------------------------------
// _gcd_terms, gcd_terms

/// An item of the `terms` list `gcd_terms` hands to `_gcd_terms`: its
/// `mask` leaves a list (the commutative part) for a non-commutative term
/// with no non-commutative factors.
enum TermItem {
    E(Expr),
    List(Vec<Expr>),
}

/// `_gcd_terms(terms, isprimitive, fraction)` for an expression: the terms
/// of an Add (a Tuple's elements): `(cont, numer, denom)`.
pub fn _gcd_terms(terms: &Expr, isprimitive: bool, fraction: bool) -> (Expr, Expr, Expr) {
    let terms = if terms.is_cls(C::Tuple) {
        terms.args().to_vec()
    } else {
        terms.add_args()
    };
    gcd_terms_items(
        terms.into_iter().map(TermItem::E).collect(),
        isprimitive,
        fraction,
    )
}

/// `_gcd_terms(terms, isprimitive, fraction)` for a list of terms.
pub fn _gcd_terms_seq(terms: &[Expr], isprimitive: bool, fraction: bool) -> (Expr, Expr, Expr) {
    gcd_terms_items(
        terms.iter().cloned().map(TermItem::E).collect(),
        isprimitive,
        fraction,
    )
}

fn gcd_terms_items(items: Vec<TermItem>, isprimitive: bool, fraction: bool) -> (Expr, Expr, Expr) {
    // terms = list(map(Term, [t for t in terms if t]))
    let kept: Vec<TermItem> = items
        .into_iter()
        .filter(|t| match t {
            TermItem::E(e) => e.truthy(),
            TermItem::List(v) => !v.is_empty(),
        })
        .collect();
    let mut terms: Vec<Term> = kept
        .into_iter()
        .map(|t| match t {
            TermItem::E(e) => Term::new(&e),
            TermItem::List(_) => raise(
                ErrorKind::AttributeError,
                "'list' object has no attribute 'is_commutative'",
            ),
        })
        .collect();

    // there is some simplification that may happen if we leave this
    // here rather than duplicate it before the mapping of Term onto
    // the terms
    if terms.is_empty() {
        return (s::zero(), s::zero(), s::one());
    }

    let mut cont;
    let mut numer;
    let denom;
    if terms.len() == 1 {
        cont = terms[0].coeff.clone();
        numer = terms[0].numer.as_expr();
        denom = terms[0].denom.as_expr();
    } else {
        let mut c = terms[0].clone();
        for term in &terms[1..] {
            c = c.gcd(term);
        }

        for term in terms.iter_mut() {
            *term = term.quo(&c);
        }

        let d;
        let numers: Vec<Expr>;
        if fraction {
            let mut dd = terms[0].denom.clone();

            for term in &terms[1..] {
                dd = dd.lcm(&term.denom);
            }

            numers = terms
                .iter()
                .map(|term| {
                    let numer = term.numer.mul(&dd.quo(&term.denom));
                    ops::mul(&term.coeff, &numer.as_expr())
                })
                .collect();
            d = dd;
        } else {
            numers = terms.iter().map(Term::as_expr).collect();
            d = Term::new(&s::one()).numer;
        }

        cont = c.as_expr();
        numer = crate::add::add(numers);
        denom = d.as_expr();
    }

    if !isprimitive && numer.is_Add() {
        let (_cont, n) = numer.primitive();
        numer = n;
        cont = ops::mul(&cont, &_cont);
    }

    (cont, numer, denom)
}

/// `gcd_terms`'s `mask(terms)`: replaces the non-commutative part of each
/// term with a new Dummy, returning the terms and the replacements that
/// restore them.
fn mask(terms: Vec<Expr>) -> (Vec<TermItem>, Vec<(Expr, Expr)>) {
    enum Cnc {
        Commutative(Expr),
        Split(Vec<Expr>, Vec<Expr>),
    }
    let args: Vec<Cnc> = terms
        .into_iter()
        .map(|a| {
            if a.is_commutative() == Some(true) {
                Cnc::Commutative(a)
            } else {
                if !a.is_a(Tag::Expr) {
                    raise(
                        ErrorKind::AttributeError,
                        format!("'{}' object has no attribute 'args_cnc'", a.class_name()),
                    );
                }
                let (c, nc) = a.args_cnc(true);
                Cnc::Split(c, nc)
            }
        })
        .collect();
    let mut reps = Vec::new();
    let items = args
        .into_iter()
        .map(|x| match x {
            Cnc::Commutative(a) => TermItem::E(a),
            Cnc::Split(mut c, nc) => {
                if nc.is_empty() {
                    TermItem::List(c)
                } else {
                    let nc = crate::mul::mul(nc);
                    let d = crate::symbol::dummy(None, &[]);
                    reps.push((d.clone(), nc));
                    c.push(d);
                    TermItem::E(crate::mul::mul(c))
                }
            }
        })
        .collect();
    (items, reps)
}

/// `gcd_terms(terms, isprimitive, clear, fraction)` for an expression:
/// the gcd of the terms of every Add in it, put together.
pub fn gcd_terms(e: &Expr, isprimitive: bool, clear: bool, fraction: bool) -> Expr {
    if e.is_Add() {
        return gcd_terms_addlike(e.args().to_vec(), isprimitive, clear, fraction);
    }

    if e.is_Atom() {
        return e.clone();
    }

    if e.is_Mul() {
        let (c, args) = e.as_coeff_mul();
        let m = crate::mul::mul(
            args.iter()
                .map(|i| gcd_terms(i, isprimitive, clear, fraction))
                .collect(),
        );
        return keep_coeff(&c, &m, clear, false);
    }

    // don't treat internal args like terms of an Add
    fn handle(a: &Expr, isprimitive: bool, clear: bool, fraction: bool) -> Expr {
        if !a.is_a(Tag::Expr) {
            if a.args().is_empty() {
                return a.clone();
            }
            return a.func(
                a.args()
                    .iter()
                    .map(|i| handle(i, isprimitive, clear, fraction))
                    .collect(),
            );
        }
        gcd_terms(a, isprimitive, clear, fraction)
    }

    // (SymPy's Dict, which gets its values handled, is not ported)
    e.func(
        e.args()
            .iter()
            .map(|i| handle(i, isprimitive, clear, fraction))
            .collect(),
    )
}

/// `gcd_terms(terms, ...)` for a list (or tuple or set) of terms, treated
/// as the terms of a sum.
pub fn gcd_terms_seq(terms: &[Expr], isprimitive: bool, clear: bool, fraction: bool) -> Expr {
    gcd_terms_addlike(terms.to_vec(), isprimitive, clear, fraction)
}

fn gcd_terms_addlike(terms: Vec<Expr>, isprimitive: bool, clear: bool, fraction: bool) -> Expr {
    let (terms, reps) = mask(terms);
    let (cont, numer, denom) = gcd_terms_items(terms, isprimitive, fraction);
    let mut numer = pending::xreplace(&numer, &reps);
    let (mut coeff, factors) = cont.as_coeff_Mul(false);
    if !clear {
        let (c, _coeff) = coeff.as_coeff_Mul(false);
        if !c.is_Integer() && !clear && numer.is_Add() {
            let (n, d) = c.as_numer_denom();
            let _numer = ops::div(&numer, &d);
            if _numer
                .args()
                .iter()
                .any(|a| a.as_coeff_Mul(false).0.is_Integer())
            {
                numer = _numer;
                coeff = ops::mul(&n, &_coeff);
            }
        }
    }
    keep_coeff(
        &coeff,
        &ops::div(&ops::mul(&factors, &numer), &denom),
        clear,
        false,
    )
}

// ---------------------------------------------------------------------------
// factor_terms

/// `factor_terms(expr, radical, clear, fraction, sign)`: removes common
/// factors from the terms of every Add in `expr` without changing its
/// structure otherwise.
pub fn factor_terms(e: &Expr, radical: bool, clear: bool, fraction: bool, sign: bool) -> Expr {
    let flags = FactorTerms {
        radical,
        clear,
        fraction,
        sign,
    };
    flags.run(e)
}

struct FactorTerms {
    radical: bool,
    clear: bool,
    fraction: bool,
    sign: bool,
}

impl FactorTerms {
    /// `do(expr)`.
    fn run(&self, expr: &Expr) -> Expr {
        // (no ported atom or Expr is `iterable`, and a non-Expr Basic takes
        // the args branch below either way)
        if expr.is_Atom() {
            return expr.clone();
        }

        if expr.is_Pow() || expr.is_Function() || !expr.is_a(Tag::Expr) {
            let args = expr.args();
            let newargs: Vec<Expr> = args.iter().map(|i| self.run(i)).collect();
            if newargs.as_slice() == args {
                return expr.clone();
            }
            return expr.func(newargs);
        }

        if expr.is_cls(C::Sum) || expr.is_cls(C::Integral) {
            return self.factor_sum_int(expr);
        }

        let (mut cont, mut p) = expr.as_content_primitive(self.radical, self.clear);
        if p.is_Add() {
            let mut list_args: Vec<Expr> = p.add_args().iter().map(|a| self.run(a)).collect();
            // get a common negative (if there) which gcd_terms does not remove
            if !list_args.iter().any(|a| {
                a.as_coeff_Mul(false)
                    .0
                    .extract_multiplicatively(&s::negative_one())
                    .is_none()
            }) {
                cont = ops::neg(&cont);
                list_args = list_args.iter().map(ops::neg).collect();
            }
            // watch out for exp(-(x+2)) which gcd_terms will change to exp(-x-2)
            let mut special = Vec::new();
            for a in list_args.iter_mut() {
                let (_b, e) = a.as_base_exp();
                if e.is_Mul() && e != crate::mul::mul(e.args().to_vec()) {
                    let d = crate::symbol::dummy(None, &[]);
                    special.push((d.clone(), a.clone()));
                    *a = d;
                }
            }
            // rebuild p not worrying about the order which gcd_terms will fix
            p = crate::add::from_args(list_args, None);
            p = pending::xreplace(&gcd_terms(&p, true, self.clear, self.fraction), &special);
        } else if !p.args().is_empty() {
            p = p.func(p.args().iter().map(|a| self.run(a)).collect());
        }
        keep_coeff(&cont, &p, self.clear, self.sign)
    }

    /// `_factor_sum_int(expr, **kwargs)`: a Sum or Integral with the
    /// factors that do not depend on its variables taken out.
    fn factor_sum_int(&self, expr: &Expr) -> Expr {
        let result = expr.arg(0); // expr.function
        if result.is_cls(C::Zero) {
            return s::zero();
        }
        // f = factor_terms(result, **kwargs); i, d = f.as_independent(*wrt)
        let _f = self.run(result);
        not_ported("_factor_sum_int: Expr.as_independent (the subs unit)")
    }
}

/// `factor_terms(seq, ...)` for a list or tuple: each element's
/// `factor_terms`, in a sequence of the same type.
pub fn factor_terms_seq(
    seq: &[Expr],
    radical: bool,
    clear: bool,
    fraction: bool,
    sign: bool,
) -> Vec<Expr> {
    let flags = FactorTerms {
        radical,
        clear,
        fraction,
        sign,
    };
    seq.iter().map(|e| flags.run(e)).collect()
}

// ---------------------------------------------------------------------------
// _mask_nc, factor_nc

/// `_mask_nc(eq, name)`: `eq` with its non-commutative objects replaced by
/// Dummies, the replacements that restore them, and the non-commutative
/// symbols left in the result.
pub fn mask_nc(eq: &Expr, name: Option<&str>) -> (Expr, Vec<(Expr, Expr)>, Vec<Expr>) {
    let name = match name {
        Some(n) if !n.is_empty() => n,
        _ => "mask",
    };
    // Make Dummy() append sequential numbers to the name
    let mut names = 0usize;
    let mut dummy = |assumptions: &[(Fact, bool)]| {
        let d = crate::symbol::dummy(Some(&format!("{name}{names}")), assumptions);
        names += 1;
        d
    };

    let expr = eq;
    if expr.is_commutative() == Some(true) {
        return (eq.clone(), vec![], vec![]);
    }

    // identify nc-objects; symbols and other
    let mut rep: Vec<(Expr, Expr)> = Vec::new();
    let mut nc_obj: Vec<Expr> = Vec::new();
    let mut nc_syms: Vec<Expr> = Vec::new();
    // (`rep` is still empty during the traversal, so no subtree is skipped
    // for matching one of its keys, and the order of the traversal does not
    // matter for the sets it fills)
    fn visit(a: &Expr, nc_obj: &mut Vec<Expr>, nc_syms: &mut Vec<Expr>) {
        if a.is_commutative() != Some(true) {
            if a.is_symbol() {
                if !nc_syms.contains(a) {
                    nc_syms.push(a.clone());
                }
                return;
            } else if !(a.is_Add() || a.is_Mul() || a.is_Pow()) {
                if !nc_obj.contains(a) {
                    nc_obj.push(a.clone());
                }
                return;
            }
        }
        for b in a.args() {
            visit(b, nc_obj, nc_syms);
        }
    }
    visit(expr, &mut nc_obj, &mut nc_syms);

    // If there is only one nc symbol or object, it can be factored regularly
    // but polys is going to complain, so replace it with a Dummy.
    if nc_obj.len() == 1 && nc_syms.is_empty() {
        let o = nc_obj.pop().expect("one nc object");
        rep.push((o, dummy(&[])));
    } else if nc_syms.len() == 1 && nc_obj.is_empty() {
        let o = nc_syms.pop().expect("one nc symbol");
        rep.push((o, dummy(&[])));
    }

    // Any remaining nc-objects will be replaced with an nc-Dummy and
    // identified as an nc-Symbol to watch out for
    crate::compare::sorted_default(&mut nc_obj);
    for n in nc_obj {
        let nc = dummy(&[(Fact::Commutative, false)]);
        rep.push((n, nc.clone()));
        if !nc_syms.contains(&nc) {
            nc_syms.push(nc);
        }
    }
    let expr = pending::subs_seq(expr, &rep);

    crate::compare::sorted_default(&mut nc_syms);
    (
        expr,
        rep.into_iter().map(|(k, v)| (v, k)).collect(),
        nc_syms,
    )
}

/// `factor_nc(expr)`: the factored form of `expr`, handling
/// non-commutative factors.
pub fn factor_nc(e: &Expr) -> Expr {
    if !e.is_a(Tag::Expr) || e.args().is_empty() {
        return e.clone();
    }
    if !e.is_Add() {
        return e.func(e.args().iter().map(factor_nc).collect());
    }
    not_ported("factor_nc of an Add (needs expand_power_exp and the polys gcd and factor)")
}

// ---------------------------------------------------------------------------
// fraction, numer, denom (sympy.simplify.radsimp)

/// `fraction(expr, exact)`: `(numerator, denominator)`.
pub fn fraction(e: &Expr, exact: bool) -> (Expr, Expr) {
    let mut numer = Vec::new();
    let mut denom = Vec::new();
    let terms = if e.is_Mul() {
        e.args().to_vec()
    } else {
        vec![e.clone()]
    };
    for term in terms {
        if term.is_commutative() == Some(true) && (term.is_Pow() || term.is_cls(C::Exp)) {
            let (b, ex) = term.as_base_exp();
            if ex.is_negative() == Some(true) {
                if ex.is_cls(C::NegativeOne) {
                    denom.push(b);
                } else if exact {
                    not_ported("fraction(exact=True): Expr.is_constant");
                } else {
                    denom.push(crate::power::pow(&b, &ops::neg(&ex)));
                }
            } else if ex.is_positive() == Some(true) {
                numer.push(term);
            } else if !exact && ex.is_Mul() {
                let (n, d) = term.as_numer_denom(); // this will cause evaluation
                if n != s::one() {
                    numer.push(n);
                }
                denom.push(d);
            } else {
                numer.push(term);
            }
        } else if term.is_Rational() && !term.is_Integer() {
            if !term.p().is_one() {
                numer.push(Expr::integer(term.p()));
            }
            denom.push(Expr::integer(term.q()));
        } else {
            numer.push(term);
        }
    }
    (
        crate::mul::mul_with(numer, !exact),
        crate::mul::mul_with(denom, !exact),
    )
}

/// `numer(expr)`.
pub fn numer(e: &Expr) -> Expr {
    fraction(e, false).0
}

/// `denom(expr)`.
pub fn denom(e: &Expr) -> Expr {
    fraction(e, false).1
}
