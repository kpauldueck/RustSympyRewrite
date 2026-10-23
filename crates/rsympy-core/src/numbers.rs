//! `sympy.core.numbers`: Integer, Rational, Float and the numeric
//! singletons, with the arithmetic their dunder methods define.
//!
//! Python picks `a.__add__(b)` from the left operand's class, walking up its
//! MRO. Each `*_add`/`*_mul`/... function here is one class's method, and
//! falls through to its base class's the same way.

use std::sync::LazyLock;

use rsympy_mpf::{self as mlib, Mpf, Rnd};
use rsympy_num::{Int, Rational};

use crate::ctx;
use crate::error::{ErrorKind, not_ported, raise, raise_mpf};
use crate::expr::{Data, Expr};
use crate::facts::Fact;
use crate::generated::{ClassId as C, Tag};

/// mpmath's `round_nearest`, which SymPy's numbers use (`rnd`).
pub(crate) const RND: Rnd = Rnd::Nearest;

/// `S.<Singleton>`.
pub mod s {
    use super::*;

    macro_rules! singleton {
        ($fn:ident, $static:ident, $make:expr) => {
            static $static: LazyLock<Expr> = LazyLock::new(|| $make);
            #[doc = concat!("`S.", stringify!($fn), "`.")]
            pub fn $fn() -> Expr {
                $static.clone()
            }
        };
    }

    singleton!(
        zero,
        ZERO,
        Expr::make(C::Zero, Data::Int(Int::ZERO), vec![])
    );
    singleton!(one, ONE, Expr::make(C::One, Data::Int(Int::ONE), vec![]));
    singleton!(
        negative_one,
        NEG_ONE,
        Expr::make(C::NegativeOne, Data::Int(Int::NEG_ONE), vec![])
    );
    singleton!(
        half,
        HALF,
        Expr::make(
            C::Half,
            Data::Rat(Rational::from_reduced(Int::ONE, Int::small(2))),
            vec![]
        )
    );
    singleton!(infinity, INF, Expr::make(C::Infinity, Data::None, vec![]));
    singleton!(
        negative_infinity,
        NINF,
        Expr::make(C::NegativeInfinity, Data::None, vec![])
    );
    singleton!(nan, NAN, Expr::make(C::NaN, Data::None, vec![]));
    singleton!(
        complex_infinity,
        ZOO,
        Expr::make(C::ComplexInfinity, Data::None, vec![])
    );
    singleton!(
        imaginary_unit,
        IMAG,
        Expr::make(C::ImaginaryUnit, Data::None, vec![])
    );
    singleton!(exp1, EXP1, Expr::make(C::Exp1, Data::None, vec![]));
    singleton!(pi, PI, Expr::make(C::Pi, Data::None, vec![]));
    singleton!(
        euler_gamma,
        EULER,
        Expr::make(C::EulerGamma, Data::None, vec![])
    );
    singleton!(catalan, CATALAN, Expr::make(C::Catalan, Data::None, vec![]));
    singleton!(
        golden_ratio,
        PHI,
        Expr::make(C::GoldenRatio, Data::None, vec![])
    );
    singleton!(
        tribonacci_constant,
        TRIB,
        Expr::make(C::TribonacciConstant, Data::None, vec![])
    );
    singleton!(true_, TRUE, Expr::make(C::BooleanTrue, Data::None, vec![]));
    singleton!(
        false_,
        FALSE,
        Expr::make(C::BooleanFalse, Data::None, vec![])
    );
    singleton!(
        empty_set,
        EMPTY_SET,
        Expr::make(C::EmptySet, Data::None, vec![])
    );
}

// ---------------------------------------------------------------------------
// construction

impl Expr {
    /// `Integer(i)`.
    pub fn integer(i: impl Into<Int>) -> Expr {
        let i = i.into();
        if let Some(v) = i.as_i64() {
            match v {
                0 => return s::zero(),
                1 => return s::one(),
                -1 => return s::negative_one(),
                _ => {}
            }
        }
        Expr::make(C::Integer, Data::Int(i), vec![])
    }

    /// `Rational(p, q)` for integers `p`, `q`.
    pub fn rational(p: impl Into<Int>, q: impl Into<Int>) -> Expr {
        rational_new(p.into(), q.into(), None)
    }

    /// `Rational(r)`.
    pub fn from_rational(r: &Rational) -> Expr {
        from_coprime_ints(r.numer().clone(), r.denom().clone())
    }
}

/// `Rational._new(p, q, gcd)`.
pub(crate) fn rational_new(p: Int, q: Int, gcd: Option<Int>) -> Expr {
    if q.is_zero() {
        if p.is_zero() {
            return s::nan();
        }
        return s::complex_infinity();
    }
    let (mut p, mut q) = if q.is_negative() { (-p, -q) } else { (p, q) };
    let gcd = gcd.unwrap_or_else(|| p.abs().gcd(&q));
    if gcd > Int::ONE {
        p = p.floor_div(&gcd).expect("gcd > 0");
        q = q.floor_div(&gcd).expect("gcd > 0");
    }
    from_coprime_ints(p, q)
}

/// `Rational.from_coprime_ints(p, q)`.
pub(crate) fn from_coprime_ints(p: Int, q: Int) -> Expr {
    if q.is_one() {
        return Expr::integer(p);
    }
    if p.is_one() && q == 2 {
        return s::half();
    }
    Expr::make(C::Rational, Data::Rat(Rational::from_reduced(p, q)), vec![])
}

fn int(e: &Expr) -> &Int {
    e.as_int().expect("an Integer")
}

/// `igcd(a, b)`.
fn igcd(a: &Int, b: &Int) -> Int {
    a.gcd(b)
}

/// `mpf_norm(mpf, prec)`.
pub(crate) fn mpf_norm(m: &Mpf, prec: i64) -> Mpf {
    if m.man.is_zero() {
        if m.bc == 0 {
            return mlib::FZERO;
        }
        return m.clone();
    }
    mlib::normalize(m.sign, m.man.clone(), m.exp, m.bc, prec, RND)
}

/// `Float._new(_mpf_, _prec, zero)`.
pub fn float_new(m: Mpf, prec: i64, zero: bool) -> Expr {
    if zero && m == mlib::FZERO {
        return s::zero();
    }
    if m == mlib::FNAN {
        return s::nan();
    }
    if m == mlib::FINF {
        return s::infinity();
    }
    if m == mlib::FNINF {
        return s::negative_infinity();
    }
    Expr::make(C::Float, Data::Float(mpf_norm(&m, prec), prec), vec![])
}

/// `_literal_float(s)`.
fn literal_float(s: &str) -> bool {
    let parts: Vec<&str> = s.split('e').collect();
    let (m, e) = match parts.as_slice() {
        [m] => (*m, "1"),
        [m, e] => {
            let e = e.strip_prefix(['+', '-']).unwrap_or(e);
            if e.is_empty() {
                return false;
            }
            (*m, e)
        }
        _ => return false,
    };
    let parts: Vec<&str> = m.split('.').collect();
    let (i, f) = match parts.as_slice() {
        [i] => (*i, "1"),
        [i, f] => (*i, *f),
        _ => return false,
    };
    if i.is_empty() && f.is_empty() {
        return false;
    }
    let i = i.strip_prefix(['+', '-']).unwrap_or(i);
    let i = if i.is_empty() { "1" } else { i };
    let f = if f.is_empty() { "1" } else { f };
    [i, f, e].iter().all(|n| {
        n.split('_')
            .all(|g| !g.is_empty() && g.bytes().all(|b| b.is_ascii_digit()))
    })
}

/// `(value, number of coefficient digits)` of `decimal.Decimal(s)` as
/// `_decimal_to_Rational_prec` computes them, for a literal float string.
fn decimal_rational_prec(s: &str) -> (Expr, usize) {
    let s = s.replace('_', "");
    let (mant, exp) = match s.find('e') {
        Some(k) => (&s[..k], s[k + 1..].parse::<i64>().expect("valid exponent")),
        None => (&s[..], 0),
    };
    let (neg, mant) = match mant.strip_prefix('-') {
        Some(m) => (true, m),
        None => (false, mant.strip_prefix('+').unwrap_or(mant)),
    };
    let (ip, fp) = mant.split_once('.').unwrap_or((mant, ""));
    let digits = format!("{ip}{fp}");
    let exp = exp - fp.len() as i64;
    let stripped = digits.trim_start_matches('0');
    let ndigits = stripped.len().max(1);
    let coeff =
        Int::parse_python(if stripped.is_empty() { "0" } else { stripped }).expect("digits");
    let value = if exp >= 0 {
        let v = &coeff * &Int::small(10).pow(exp as u64);
        Expr::integer(if neg { -v } else { v })
    } else {
        let c = if neg { -coeff } else { coeff };
        Expr::rational(c, Int::small(10).pow(exp.unsigned_abs()))
    };
    (value, ndigits)
}

/// `Float(s)` for a string, with optional `dps` or `precision`.
pub fn float_from_str(text: &str, dps: Option<i64>, precision: Option<i64>) -> Expr {
    let trimmed = text.trim();
    let mut num = trimmed.replace(' ', "_").to_lowercase();
    if num.starts_with('.') && num.len() > 1 {
        num = format!("0{num}");
    } else if num.starts_with("-.") && num.len() > 2 {
        num = format!("-0.{}", &num[2..]);
    } else if num == "inf" || num == "+inf" {
        return s::infinity();
    } else if num == "-inf" {
        return s::negative_infinity();
    } else if num == "nan" {
        return s::nan();
    } else if !literal_float(&num) {
        raise(
            ErrorKind::ValueError,
            format!("string-float not recognized: {trimmed}"),
        );
    }
    let precision = match (dps, precision) {
        (Some(_), Some(_)) => raise(
            ErrorKind::ValueError,
            "Both decimal and binary precision supplied. Supply only one. ",
        ),
        (None, None) => {
            let isint = !num.contains('.');
            let (value, ndigits) = decimal_rational_prec(&num);
            let mut dps = ndigits as i64;
            if value.is_Integer() && isint {
                dps = dps.max(int(&value).num_decimal_digits() as i64);
            }
            mlib::dps_to_prec(dps.max(15))
        }
        (Some(d), None) => mlib::dps_to_prec(d),
        (None, Some(p)) => p,
    };
    let m = mlib::from_str(&num.replace('_', ""), precision, RND)
        .unwrap_or_else(|e| raise(ErrorKind::ValueError, e.to_string()));
    float_new(m, precision, false)
}

/// `Float(x)` for a Python float.
pub fn float_from_f64(x: f64) -> Expr {
    if x == 0.0 {
        return float_from_str("0", None, None);
    }
    if x == f64::INFINITY {
        return s::infinity();
    }
    if x == f64::NEG_INFINITY {
        return s::negative_infinity();
    }
    if x.is_nan() {
        return s::nan();
    }
    let prec = mlib::dps_to_prec(15);
    float_new(mlib::from_float(x, prec, RND), prec, false)
}

/// `Float(n, precision=prec)` for a Number or NumberSymbol `n`.
pub fn float_of_number(n: &Expr, prec: i64) -> Expr {
    if n.is_Float() {
        return n.clone();
    }
    float_new(as_mpf_val(n, prec), prec, false)
}

// ---------------------------------------------------------------------------
// mpf views

/// `Number._prec`: -1 except for Floats.
pub(crate) fn num_prec(e: &Expr) -> i64 {
    match e.as_float() {
        Some((_, p)) => p,
        None => -1,
    }
}

/// `self._as_mpf_val(prec)`.
pub fn as_mpf_val(e: &Expr, prec: i64) -> Mpf {
    match e.cls() {
        C::Integer | C::Zero | C::One | C::NegativeOne => mlib::from_int(int(e), prec, RND),
        C::Rational | C::Half => mlib::from_rational(&e.p(), &e.q(), prec, RND).expect("q > 0"),
        C::Float => mpf_norm(e.as_float().expect("Float").0, prec),
        C::Infinity => mlib::FINF,
        C::NegativeInfinity => mlib::FNINF,
        C::NaN => mlib::FNAN,
        C::Pi => mlib::mpf_pi(prec, RND),
        C::Exp1 => mlib::mpf_e(prec, RND),
        C::GoldenRatio => mlib::mpf_phi(prec, RND),
        _ => not_ported(format!("{}._as_mpf_val", e.class_name())),
    }
}

/// `self._as_mpf_op(prec)`.
fn as_mpf_op(e: &Expr, prec: i64) -> (Mpf, i64) {
    if let Some((m, p)) = e.as_float() {
        return (m.clone(), prec.max(p));
    }
    let prec = prec.max(num_prec(e));
    (as_mpf_val(e, prec), prec)
}

// ---------------------------------------------------------------------------
// arithmetic: the left operand is a Number (or zoo/NaN-like singleton)

fn evaluate() -> bool {
    ctx::evaluate()
}

fn is_rational_cls(e: &Expr) -> bool {
    e.is_Rational()
}

/// `Expr.__add__(self, other)`.
fn expr_add(a: &Expr, b: &Expr) -> Expr {
    crate::add::add(vec![a.clone(), b.clone()])
}

/// `Expr.__sub__(self, other)`.
fn expr_sub(a: &Expr, b: &Expr) -> Expr {
    crate::add::add(vec![a.clone(), crate::ops::neg(b)])
}

/// `Expr.__mul__(self, other)`.
fn expr_mul(a: &Expr, b: &Expr) -> Expr {
    crate::mul::mul(vec![a.clone(), b.clone()])
}

/// `Expr.__truediv__(self, other)`.
fn expr_truediv(a: &Expr, b: &Expr) -> Expr {
    let denom = crate::power::pow(b, &s::negative_one());
    if a.is_cls(C::One) {
        denom
    } else {
        crate::mul::mul(vec![a.clone(), denom])
    }
}

/// `Number.__add__`.
fn number_add(a: &Expr, b: &Expr) -> Expr {
    if b.is_Number() && evaluate() {
        match b.cls() {
            C::NaN => return s::nan(),
            C::Infinity => return s::infinity(),
            C::NegativeInfinity => return s::negative_infinity(),
            _ => {}
        }
    }
    expr_add(a, b)
}

/// `Number.__sub__`.
fn number_sub(a: &Expr, b: &Expr) -> Expr {
    if b.is_Number() && evaluate() {
        match b.cls() {
            C::NaN => return s::nan(),
            C::Infinity => return s::negative_infinity(),
            C::NegativeInfinity => return s::infinity(),
            _ => {}
        }
    }
    expr_sub(a, b)
}

/// `Number.__mul__`.
fn number_mul(a: &Expr, b: &Expr) -> Expr {
    if b.is_Number() && evaluate() {
        match b.cls() {
            C::NaN => return s::nan(),
            C::Infinity => {
                return if a.is_zero() == Some(true) {
                    s::nan()
                } else if a.is_positive() == Some(true) {
                    s::infinity()
                } else {
                    s::negative_infinity()
                };
            }
            C::NegativeInfinity => {
                return if a.is_zero() == Some(true) {
                    s::nan()
                } else if a.is_positive() == Some(true) {
                    s::negative_infinity()
                } else {
                    s::infinity()
                };
            }
            _ => {}
        }
    }
    expr_mul(a, b)
}

/// `Number.__truediv__`.
fn number_truediv(a: &Expr, b: &Expr) -> Expr {
    if b.is_Number() && evaluate() {
        match b.cls() {
            C::NaN => return s::nan(),
            C::Infinity | C::NegativeInfinity => return s::zero(),
            _ => {}
        }
    }
    expr_truediv(a, b)
}

/// `a + b` where `a` is a Number.
pub(crate) fn add(a: &Expr, b: &Expr) -> Expr {
    let ev = evaluate();
    match a.cls() {
        C::Integer | C::Zero | C::One | C::NegativeOne => {
            if !ev {
                return expr_add(a, b);
            }
            if b.is_Integer() {
                return Expr::integer(int(a) + int(b));
            }
            if is_rational_cls(b) {
                let (bp, bq) = (b.p(), b.q());
                return rational_new(&(int(a) * &bq) + &bp, bq, Some(Int::ONE));
            }
            rational_add(a, b)
        }
        C::Rational | C::Half => rational_add(a, b),
        C::Float => {
            if b.is_Number() && ev {
                let (m, p) = a.as_float().expect("Float");
                let (rhs, prec) = as_mpf_op(b, p);
                return float_new(mlib::mpf_add(m, &rhs, prec, RND), prec, true);
            }
            number_add(a, b)
        }
        C::Infinity | C::NegativeInfinity => {
            if b.is_Number() && ev {
                let opposite = if a.is_cls(C::Infinity) {
                    C::NegativeInfinity
                } else {
                    C::Infinity
                };
                if b.is_cls(opposite) || b.is_cls(C::NaN) {
                    return s::nan();
                }
                return a.clone();
            }
            number_add(a, b)
        }
        C::NaN => a.clone(),
        _ => unreachable!("numbers::add on {}", a.class_name()),
    }
}

/// `Rational.__add__`.
fn rational_add(a: &Expr, b: &Expr) -> Expr {
    if evaluate() {
        let (p, q) = (a.p(), a.q());
        if b.is_Integer() {
            return rational_new(&p + &(&q * int(b)), q, Some(Int::ONE));
        }
        if is_rational_cls(b) {
            let (bp, bq) = (b.p(), b.q());
            return rational_new(&(&p * &bq) + &(&q * &bp), &q * &bq, None);
        }
        if b.is_Float() {
            return add(b, a);
        }
    }
    number_add(a, b)
}

/// `a - b` where `a` is a Number.
pub(crate) fn sub(a: &Expr, b: &Expr) -> Expr {
    let ev = evaluate();
    match a.cls() {
        C::Integer | C::Zero | C::One | C::NegativeOne => {
            if ev {
                if b.is_Integer() {
                    return Expr::integer(int(a) - int(b));
                }
                if is_rational_cls(b) {
                    let (bp, bq) = (b.p(), b.q());
                    return rational_new(&(int(a) * &bq) - &bp, bq, Some(Int::ONE));
                }
            }
            rational_sub(a, b)
        }
        C::Rational | C::Half => rational_sub(a, b),
        C::Float => {
            if b.is_Number() && ev {
                let (m, p) = a.as_float().expect("Float");
                let (rhs, prec) = as_mpf_op(b, p);
                return float_new(mlib::mpf_sub(m, &rhs, prec, RND), prec, true);
            }
            number_sub(a, b)
        }
        C::Infinity | C::NegativeInfinity => {
            if b.is_Number() && ev {
                if b.is_cls(a.cls()) || b.is_cls(C::NaN) {
                    return s::nan();
                }
                return a.clone();
            }
            number_sub(a, b)
        }
        C::NaN => a.clone(),
        _ => unreachable!("numbers::sub on {}", a.class_name()),
    }
}

/// `Rational.__sub__`.
fn rational_sub(a: &Expr, b: &Expr) -> Expr {
    if evaluate() {
        let (p, q) = (a.p(), a.q());
        if b.is_Integer() {
            return rational_new(&p - &(&q * int(b)), q, Some(Int::ONE));
        }
        if is_rational_cls(b) {
            let (bp, bq) = (b.p(), b.q());
            return rational_new(&(&p * &bq) - &(&q * &bp), &q * &bq, None);
        }
        if b.is_Float() {
            return add(&neg(b), a);
        }
    }
    number_sub(a, b)
}

/// `a * b` where `a` is a Number.
pub(crate) fn mul(a: &Expr, b: &Expr) -> Expr {
    let ev = evaluate();
    match a.cls() {
        C::Integer | C::Zero | C::One | C::NegativeOne => {
            if ev {
                if b.is_Integer() {
                    return Expr::integer(int(a) * int(b));
                }
                if is_rational_cls(b) {
                    let (bp, bq) = (b.p(), b.q());
                    let g = igcd(int(a), &bq);
                    return rational_new(int(a) * &bp, bq, Some(g));
                }
            }
            rational_mul(a, b)
        }
        C::Rational | C::Half => rational_mul(a, b),
        C::Float => {
            if b.is_Number() && ev {
                let (m, p) = a.as_float().expect("Float");
                let (rhs, prec) = as_mpf_op(b, p);
                return float_new(mlib::mpf_mul(m, &rhs, prec, RND), prec, true);
            }
            number_mul(a, b)
        }
        C::Infinity | C::NegativeInfinity => {
            if b.is_Number() && ev {
                if b.is_zero() == Some(true) || b.is_cls(C::NaN) {
                    return s::nan();
                }
                // oo*pos = oo, oo*other = -oo; -oo*pos = -oo, -oo*other = oo
                let pos = b.is_extended_positive() == Some(true);
                return if a.is_cls(C::Infinity) == pos {
                    s::infinity()
                } else {
                    s::negative_infinity()
                };
            }
            number_mul(a, b)
        }
        C::NaN => a.clone(),
        _ => unreachable!("numbers::mul on {}", a.class_name()),
    }
}

/// `Rational.__mul__`.
fn rational_mul(a: &Expr, b: &Expr) -> Expr {
    if evaluate() {
        let (p, q) = (a.p(), a.q());
        if b.is_Integer() {
            let g = igcd(int(b), &q);
            return rational_new(&p * int(b), q, Some(g));
        }
        if is_rational_cls(b) {
            let (bp, bq) = (b.p(), b.q());
            let g = &igcd(&p, &bq) * &igcd(&q, &bp);
            return rational_new(&p * &bp, &q * &bq, Some(g));
        }
        if b.is_Float() {
            return mul(b, a);
        }
    }
    number_mul(a, b)
}

/// `a / b` where `a` is a Number.
pub(crate) fn truediv(a: &Expr, b: &Expr) -> Expr {
    let ev = evaluate();
    match a.cls() {
        C::Integer | C::Zero | C::One | C::NegativeOne | C::Rational | C::Half => {
            if ev {
                let (p, q) = (a.p(), a.q());
                if b.is_Integer() {
                    if !p.is_zero() && int(b).is_zero() {
                        return s::complex_infinity();
                    }
                    let g = igcd(&p, int(b));
                    return rational_new(p, &q * int(b), Some(g));
                }
                if is_rational_cls(b) {
                    let (bp, bq) = (b.p(), b.q());
                    let g = &igcd(&p, &bp) * &igcd(&q, &bq);
                    return rational_new(&p * &bq, &q * &bp, Some(g));
                }
                if b.is_Float() {
                    let inv = expr_truediv(&s::one(), b);
                    return crate::ops::mul(a, &inv);
                }
            }
            number_truediv(a, b)
        }
        C::Float => {
            // `other != 0`: Float equality is structural in SymPy 1.14, so
            // only S.Zero (the one zero Rational) equals 0
            if b.is_Number() && !b.is_cls(C::Zero) && ev {
                let (m, p) = a.as_float().expect("Float");
                let (rhs, prec) = as_mpf_op(b, p);
                let q = mlib::mpf_div(m, &rhs, prec, RND)
                    .unwrap_or_else(|e| raise(ErrorKind::ZeroDivisionError, e.to_string()));
                return float_new(q, prec, true);
            }
            number_truediv(a, b)
        }
        C::Infinity | C::NegativeInfinity => {
            if b.is_Number() && ev {
                if matches!(b.cls(), C::Infinity | C::NegativeInfinity | C::NaN) {
                    return s::nan();
                }
                let nonneg = b.is_extended_nonnegative() == Some(true);
                let same = a.is_cls(C::Infinity) == nonneg;
                return if same {
                    s::infinity()
                } else {
                    s::negative_infinity()
                };
            }
            number_truediv(a, b)
        }
        C::NaN => a.clone(),
        _ => unreachable!("numbers::truediv on {}", a.class_name()),
    }
}

/// `-a` for a Number or numeric singleton.
pub(crate) fn neg(a: &Expr) -> Expr {
    match a.cls() {
        C::Integer | C::Zero | C::One | C::NegativeOne => Expr::integer(-int(a)),
        C::Rational | C::Half => Expr::rational(-a.p(), a.q()),
        C::Float => {
            let (m, p) = a.as_float().expect("Float");
            if *m == mlib::FZERO {
                return a.clone();
            }
            float_new(mlib::mpf_neg(m, 0, RND), p, true)
        }
        C::Infinity => s::negative_infinity(),
        C::NegativeInfinity => s::infinity(),
        C::NaN | C::ComplexInfinity => a.clone(),
        _ => unreachable!("numbers::neg on {}", a.class_name()),
    }
}

/// `abs(a)` for a Number or numeric singleton, `None` where SymPy falls
/// back to `Abs(a)`.
pub(crate) fn abs(a: &Expr) -> Option<Expr> {
    Some(match a.cls() {
        C::Integer | C::Zero | C::One | C::NegativeOne => {
            if int(a).is_negative() {
                Expr::integer(-int(a))
            } else {
                a.clone()
            }
        }
        C::Rational | C::Half => Expr::rational(a.p().abs(), a.q()),
        C::Float => {
            let (m, p) = a.as_float().expect("Float");
            float_new(mlib::mpf_abs(m, 0, RND), p, true)
        }
        C::Infinity | C::NegativeInfinity | C::ComplexInfinity => s::infinity(),
        C::Exp1 | C::Pi => a.clone(),
        C::ImaginaryUnit => s::one(),
        _ => return None,
    })
}

/// `a % b` where `a` is a Number, `None` where SymPy falls back to
/// `Mod(a, b)`.
pub(crate) fn rem(a: &Expr, b: &Expr) -> Option<Expr> {
    let ev = evaluate();
    match a.cls() {
        C::Integer | C::Zero | C::One | C::NegativeOne if ev && b.is_Integer() => {
            let r = int(a)
                .py_mod(int(b))
                .unwrap_or_else(|_| raise(ErrorKind::ZeroDivisionError, "integer modulo by zero"));
            Some(Expr::integer(r))
        }
        C::Integer | C::Zero | C::One | C::NegativeOne | C::Rational | C::Half if ev => {
            if is_rational_cls(b) {
                let (p, q, bp, bq) = (a.p(), a.q(), b.p(), b.q());
                let n = (&p * &bq).floor_div(&(&bp * &q)).unwrap_or_else(|_| {
                    raise(
                        ErrorKind::ZeroDivisionError,
                        "integer division or modulo by zero",
                    )
                });
                return Some(Expr::rational(&(&p * &bq) - &(&(&n * &bp) * &q), &q * &bq));
            }
            if let Some((_, bprec)) = b.as_float() {
                // calculate mod with Rationals, *then* round the answer
                let r = rem(a, &float_to_rational(b)).expect("Rational % Rational");
                return Some(float_of_number(&r, bprec));
            }
            None
        }
        C::Infinity | C::NegativeInfinity => Some(s::nan()),
        C::Float if ev => float_mod(a, b),
        _ => None,
    }
}

/// `Float.__mod__(self, other)` under `evaluate`.
fn float_mod(a: &Expr, b: &Expr) -> Option<Expr> {
    let (m, prec) = a.as_float().expect("Float");
    if is_rational_cls(b) && !b.q().is_one() {
        // calculate mod with Rationals, *then* round the result
        let r = rem(&float_to_rational(a), b).expect("Rational % Rational");
        return Some(float_of_number(&r, prec));
    }
    if let Some((_, bprec)) = b.as_float() {
        let r = truediv(a, b);
        if int_valued(&r) {
            return Some(float_from_str("0", None, Some(prec.max(bprec))));
        }
    }
    if b.is_Number() {
        let (rhs, prec) = as_mpf_op(b, prec);
        let r = mlib::mpf_mod(m, &rhs, prec, RND).unwrap_or_else(|e| raise_mpf(e));
        return Some(float_new(r, prec, true));
    }
    None
}

/// `Rational(f)` for a Float: its exact value (`_as_integer_ratio`).
pub(crate) fn float_to_rational(f: &Expr) -> Expr {
    let (m, _) = f.as_float().expect("Float");
    let mut p = if m.sign == 1 {
        -m.man.clone()
    } else {
        m.man.clone()
    };
    let q = if m.exp < 0 {
        Int::ONE << (-m.exp) as u64
    } else {
        p = p << m.exp as u64;
        Int::ONE
    };
    Expr::rational(p, q)
}

// ---------------------------------------------------------------------------
// _eval_power

/// `base._eval_power(expt)` for numeric bases. `None` leaves an
/// unevaluated `Pow`.
pub(crate) fn eval_power(base: &Expr, expt: &Expr) -> Option<Expr> {
    match base.cls() {
        C::Zero => zero_eval_power(expt),
        C::One => Some(base.clone()),
        C::NegativeOne => negative_one_eval_power(base, expt),
        C::Integer => integer_eval_power(base, expt),
        C::Rational | C::Half => rational_eval_power(base, expt),
        C::Float => float_eval_power(base, expt),
        C::Infinity => infinity_eval_power(expt),
        C::NegativeInfinity => negative_infinity_eval_power(expt),
        C::ComplexInfinity => complex_infinity_eval_power(expt),
        C::NaN => None,
        C::ImaginaryUnit => imaginary_unit_eval_power(expt),
        C::Exp1 => {
            if ctx::exp_is_pow() {
                not_ported("Exp1._eval_power_exp_is_pow");
            }
            Some(crate::functions::exp(expt))
        }
        _ => None,
    }
}

fn pow(b: &Expr, e: &Expr) -> Expr {
    crate::power::pow(b, e)
}

fn pow_uneval(b: &Expr, e: &Expr) -> Expr {
    crate::power::pow_with(b, e, false)
}

/// `Zero._eval_power`.
fn zero_eval_power(expt: &Expr) -> Option<Expr> {
    if expt.is_extended_positive() == Some(true) {
        return Some(s::zero());
    }
    if expt.is_extended_negative() == Some(true) {
        return Some(s::complex_infinity());
    }
    if expt.is_extended_real() == Some(false) {
        return Some(s::nan());
    }
    if expt.is_zero() == Some(true) {
        return Some(s::one());
    }
    let (coeff, terms) = expt.as_coeff_Mul(false);
    if coeff.is_negative() == Some(true) {
        return Some(pow(&s::complex_infinity(), &terms));
    }
    if !coeff.is_cls(C::One) {
        return Some(pow(&s::zero(), &terms));
    }
    None
}

/// `NegativeOne._eval_power`.
fn negative_one_eval_power(base: &Expr, expt: &Expr) -> Option<Expr> {
    if expt.is_odd() == Some(true) {
        return Some(s::negative_one());
    }
    if expt.is_even() == Some(true) {
        return Some(s::one());
    }
    if expt.is_Number() {
        if expt.is_Float() {
            return Some(pow(&float_from_f64(-1.0), expt));
        }
        if matches!(expt.cls(), C::NaN | C::Infinity | C::NegativeInfinity) {
            return Some(s::nan());
        }
        if expt.is_cls(C::Half) {
            return Some(s::imaginary_unit());
        }
        if expt.is_Rational() {
            let (p, q) = (expt.p(), expt.q());
            if q == 2 {
                return Some(pow(&s::imaginary_unit(), &Expr::integer(p)));
            }
            let (i, r) = p.div_mod(&q).expect("q > 0");
            if !i.is_zero() {
                let a = pow(base, &Expr::integer(i));
                let b = pow(base, &Expr::rational(r, q));
                return Some(crate::ops::mul(&a, &b));
            }
        }
    }
    None
}

/// `Rational._eval_power`.
fn rational_eval_power(base: &Expr, expt: &Expr) -> Option<Expr> {
    if expt.is_Number() {
        if expt.is_Float() {
            let (_, prec) = expt.as_float().expect("Float");
            let f = float_of_number(base, prec);
            return Some(pow(&f, expt));
        }
        let (p, q) = (base.p(), base.q());
        if expt.is_extended_negative() == Some(true) {
            let ne = crate::ops::neg(expt);
            if ne.is_cls(C::One) {
                return Some(Expr::rational(q, p));
            }
            if base.is_negative() == Some(true) {
                let a = pow(&s::negative_one(), expt);
                let b = pow(&Expr::rational(q, -p), &ne);
                return Some(crate::ops::mul(&a, &b));
            }
            return Some(pow(&Expr::rational(q, p), &ne));
        }
        if expt.is_cls(C::Infinity) {
            if p > q {
                return Some(s::infinity());
            }
            if p < -&q {
                let ioo = crate::ops::mul(&s::infinity(), &s::imaginary_unit());
                return Some(crate::ops::add(&s::infinity(), &ioo));
            }
            return Some(s::zero());
        }
        if expt.is_Integer() {
            let n = int(expt)
                .as_u64()
                .unwrap_or_else(|| not_ported("huge Rational power"));
            return Some(rational_new(p.pow(n), q.pow(n), Some(Int::ONE)));
        }
        if expt.is_Rational() {
            let (ep, eq) = (expt.p(), expt.q());
            let mut intpart = ep.floor_div(&eq).expect("q > 0");
            let one_over = |k: &Int| -> Expr {
                let k = k
                    .as_u64()
                    .unwrap_or_else(|| not_ported("huge Rational power"));
                rational_new(Int::ONE, q.pow(k), Some(Int::ONE))
            };
            let (ratfracpart, tail) = if !intpart.is_zero() {
                intpart = &intpart + 1;
                let remfracpart = &(&intpart * &eq) - &ep;
                (Expr::rational(remfracpart, eq.clone()), one_over(&intpart))
            } else {
                let remfracpart = &eq - &ep;
                (Expr::rational(remfracpart, eq.clone()), one_over(&Int::ONE))
            };
            let qpart = pow(&Expr::integer(q.clone()), &ratfracpart);
            if !p.is_one() {
                let ppart = pow(&Expr::integer(p.clone()), expt);
                return Some(crate::ops::mul(&crate::ops::mul(&ppart, &qpart), &tail));
            }
            return Some(crate::ops::mul(&qpart, &tail));
        }
    }
    if base.is_extended_negative() == Some(true) && expt.is_even() == Some(true) {
        return Some(pow(&crate::ops::neg(base), expt));
    }
    None
}

/// `Integer._eval_power`.
fn integer_eval_power(base: &Expr, expt: &Expr) -> Option<Expr> {
    let p = int(base);
    if expt.is_cls(C::Infinity) {
        if *p > Int::ONE {
            return Some(s::infinity());
        }
        let ioo = crate::ops::mul(&s::imaginary_unit(), &s::infinity());
        return Some(crate::ops::add(&s::infinity(), &ioo));
    }
    if expt.is_cls(C::NegativeInfinity) {
        let inv = rational_new(Int::ONE, p.clone(), Some(Int::ONE));
        return Some(pow(&inv, &s::infinity()));
    }
    if !expt.is_Number() && base.is_negative() == Some(true) && expt.is_even() == Some(true) {
        return Some(pow(&crate::ops::neg(base), expt));
    }
    if expt.is_Float() {
        return rational_eval_power(base, expt);
    }
    if !expt.is_Rational() {
        return None;
    }
    if expt.is_cls(C::Half) && base.is_negative() == Some(true) {
        let r = pow(&crate::ops::neg(base), expt);
        return Some(crate::ops::mul(&s::imaginary_unit(), &r));
    }
    if expt.is_negative() == Some(true) {
        let ne = crate::ops::neg(expt);
        if base.is_negative() == Some(true) {
            let a = pow(&s::negative_one(), expt);
            let b = pow(&rational_new(Int::ONE, -p, Some(Int::ONE)), &ne);
            return Some(crate::ops::mul(&a, &b));
        }
        return Some(pow(&rational_new(Int::ONE, p.clone(), Some(Int::ONE)), &ne));
    }
    let (ep, eq) = (expt.p(), expt.q());
    let eq_u = eq.as_u64().unwrap_or_else(|| not_ported("huge root"));
    let (x, exact) = rsympy_ntheory::integer_nthroot(&p.abs(), eq_u)
        .unwrap_or_else(|e| raise(ErrorKind::ValueError, e.to_string()));
    if exact {
        let k = ep
            .abs()
            .as_u64()
            .unwrap_or_else(|| not_ported("huge power"));
        let mut result = Expr::integer(x.pow(k));
        if base.is_negative() == Some(true) {
            result = crate::ops::mul(&result, &pow(&s::negative_one(), expt));
        }
        return Some(result);
    }
    let b_pos = p.abs();
    let dict: Vec<(Int, Int)> =
        ctx::with_nt(|nt| match rsympy_ntheory::perfect_power(nt, &b_pos) {
            Some((b, e)) => vec![(b, Int::from(e))],
            None => {
                let opts = rsympy_ntheory::FactorOpts {
                    limit: Some(Int::small(1 << 15)),
                    use_trial: true,
                    use_rho: false,
                    use_pm1: false,
                    use_ecm: true,
                };
                rsympy_ntheory::factorint(nt, &b_pos, &opts)
                    .iter()
                    .map(|(k, v)| (k.clone(), Int::from(v)))
                    .collect()
            }
        });
    let mut out_int = Int::ONE;
    let mut out_rad = s::one();
    let mut sqr_int = Int::ONE;
    let mut sqr_gcd = Int::ZERO;
    let mut sqr_dict: Vec<(Int, Int)> = Vec::new();
    for (prime, exponent) in dict {
        let exponent = &exponent * &ep;
        let (div_e, div_m) = exponent.div_mod(&eq).expect("q > 0");
        if div_e.is_positive() {
            let k = div_e.as_u64().unwrap_or_else(|| not_ported("huge power"));
            out_int = &out_int * &prime.pow(k);
        }
        if div_m.is_positive() {
            let g = igcd(&div_m, &eq);
            if !g.is_one() {
                let e = rational_new(
                    div_m.floor_div(&g).expect("g > 0"),
                    eq.floor_div(&g).expect("g > 0"),
                    Some(Int::ONE),
                );
                out_rad = crate::ops::mul(&out_rad, &pow(&Expr::integer(prime), &e));
            } else {
                sqr_dict.push((prime, div_m));
            }
        }
    }
    for (_, ex) in &sqr_dict {
        if sqr_gcd.is_zero() {
            sqr_gcd = ex.clone();
        } else {
            sqr_gcd = igcd(&sqr_gcd, ex);
            if sqr_gcd.is_one() {
                break;
            }
        }
    }
    for (k, v) in &sqr_dict {
        let e = v.floor_div(&sqr_gcd).expect("gcd > 0");
        let e = e.as_u64().unwrap_or_else(|| not_ported("huge power"));
        sqr_int = &sqr_int * &k.pow(e);
    }
    if sqr_int == b_pos && out_int.is_one() && out_rad.is_cls(C::One) {
        return None;
    }
    let rad = pow(&Expr::integer(sqr_int), &Expr::rational(sqr_gcd, eq));
    let mut result = crate::ops::mul(&crate::ops::mul(&Expr::integer(out_int), &out_rad), &rad);
    if base.is_negative() == Some(true) {
        result = crate::ops::mul(&result, &pow(&s::negative_one(), expt));
    }
    Some(result)
}

/// `Float._eval_power`.
fn float_eval_power(base: &Expr, expt: &Expr) -> Option<Expr> {
    let (m, prec) = base.as_float().expect("Float");
    if *m == mlib::FZERO {
        if expt.is_extended_positive() == Some(true) {
            return Some(base.clone());
        }
        if expt.is_extended_negative() == Some(true) {
            return Some(s::complex_infinity());
        }
    }
    if expt.is_Number() {
        if expt.is_Integer() {
            let y = mlib::mpf_pow_int(m, int(expt), prec, RND)
                .unwrap_or_else(|e| raise(ErrorKind::ZeroDivisionError, e.to_string()));
            return Some(float_new(y, prec, true));
        }
        if expt.is_Rational()
            && expt.p().is_one()
            && expt.q().is_odd()
            && base.is_negative() == Some(true)
        {
            let a = pow_uneval(&s::negative_one(), expt);
            let b = float_eval_power(&neg(base), expt)?;
            return Some(crate::ops::mul(&a, &b));
        }
        not_ported("mpf_pow for a non-integer Float power");
    }
    None
}

/// `Infinity._eval_power`.
fn infinity_eval_power(expt: &Expr) -> Option<Expr> {
    if expt.is_extended_positive() == Some(true) {
        return Some(s::infinity());
    }
    if expt.is_extended_negative() == Some(true) {
        return Some(s::zero());
    }
    if expt.is_cls(C::NaN) || expt.is_cls(C::ComplexInfinity) {
        return Some(s::nan());
    }
    if expt.is_extended_real() == Some(false) && expt.is_number() {
        not_ported("oo ** complex");
    }
    None
}

/// `NegativeInfinity._eval_power`.
fn negative_infinity_eval_power(expt: &Expr) -> Option<Expr> {
    if !expt.is_number() {
        return None;
    }
    if matches!(expt.cls(), C::NaN | C::Infinity | C::NegativeInfinity) {
        return Some(s::nan());
    }
    if expt.is_Integer() && expt.is_extended_positive() == Some(true) {
        return Some(if expt.is_odd() == Some(true) {
            s::negative_infinity()
        } else {
            s::infinity()
        });
    }
    let inf_part = pow(&s::infinity(), expt);
    let s_part = pow(&s::negative_one(), expt);
    if inf_part.is_cls(C::Zero) && s_part.is_finite() == Some(true) {
        return Some(inf_part);
    }
    if inf_part.is_cls(C::ComplexInfinity)
        && s_part.is_finite() == Some(true)
        && s_part.is_zero() != Some(true)
    {
        return Some(s::complex_infinity());
    }
    Some(crate::ops::mul(&s_part, &inf_part))
}

/// `ComplexInfinity._eval_power`.
fn complex_infinity_eval_power(expt: &Expr) -> Option<Expr> {
    if expt.is_cls(C::ComplexInfinity) {
        return Some(s::nan());
    }
    if expt.is_Number() {
        if expt.is_zero() == Some(true) {
            return Some(s::nan());
        }
        return Some(if expt.is_positive() == Some(true) {
            s::complex_infinity()
        } else {
            s::zero()
        });
    }
    None
}

/// `ImaginaryUnit._eval_power`.
fn imaginary_unit_eval_power(expt: &Expr) -> Option<Expr> {
    if expt.is_Integer() {
        let e = int(expt).py_mod(&Int::small(4)).expect("4 > 0");
        return Some(match e.as_i64() {
            Some(0) => s::one(),
            Some(1) => s::imaginary_unit(),
            Some(2) => s::negative_one(),
            _ => crate::ops::neg(&s::imaginary_unit()),
        });
    }
    if expt.is_Rational() {
        // divmod(expt, 2) on a Rational: (floor(expt/2), expt - 2*floor)
        let (p, q) = (expt.p(), expt.q());
        let two_q = &q * &Int::small(2);
        let i = p.floor_div(&two_q).expect("q > 0");
        let r = Expr::rational(&p - &(&i * &two_q), q);
        let rv = pow_uneval(&s::imaginary_unit(), &r);
        if i.is_odd() {
            return Some(crate::mul::mul_with(vec![s::negative_one(), rv], false));
        }
        return Some(rv);
    }
    None
}

// ---------------------------------------------------------------------------
// assumption handlers

/// The `_eval_is_*` handlers of Rational and Integer (and the Expr
/// positivity handlers, which for a Rational reduce to its sign).
pub(crate) fn rational_handler(e: &Expr, fact: Fact) -> Option<bool> {
    let p = e.p();
    match fact {
        Fact::Positive => Some(p.is_positive()),
        Fact::Zero => Some(p.is_zero()),
        // Expr._eval_is_extended_positive_negative: evalf(2) of a nonzero
        // rational has its sign.
        Fact::ExtendedPositive => Some(p.is_positive()),
        Fact::ExtendedNegative => Some(p.is_negative()),
        Fact::Odd => Some(p.is_odd()),
        Fact::Prime => Some(rsympy_ntheory::isprime(&p)),
        Fact::Composite => {
            if p > Int::ONE {
                e.is_prime().map(|b| !b)
            } else {
                Some(false)
            }
        }
        _ => None,
    }
}

/// `Float._eval_is_*`.
pub(crate) fn float_handler(e: &Expr, fact: Fact) -> Option<bool> {
    let (m, _) = e.as_float().expect("Float");
    let inf = *m == mlib::FINF || *m == mlib::FNINF;
    let sign = mlib::mpf_sign(m);
    match fact {
        Fact::Finite => Some(!inf),
        Fact::Infinite => Some(inf),
        Fact::Integer => {
            if *m == mlib::FZERO {
                return Some(true);
            }
            if !int_valued(e) {
                return Some(false);
            }
            None
        }
        Fact::Negative => Some(!inf && sign < 0),
        Fact::Positive => Some(!inf && sign > 0),
        Fact::ExtendedNegative => Some(*m == mlib::FNINF || (!inf && sign < 0)),
        Fact::ExtendedPositive => Some(*m == mlib::FINF || (!inf && sign > 0)),
        Fact::Zero => Some(*m == mlib::FZERO),
        _ => None,
    }
}

/// The handlers numeric singletons inherit (`Expr._eval_is_extended_*`).
pub(crate) fn singleton_handler(e: &Expr, fact: Fact) -> Option<bool> {
    match (e.cls(), fact) {
        (C::NaN, _) => None,
        (_, Fact::ExtendedPositive | Fact::ExtendedNegative) if e.is_a(Tag::NumberSymbol) => {
            // Every NumberSymbol is a positive real constant.
            Some(fact == Fact::ExtendedPositive)
        }
        _ => None,
    }
}

/// `int_valued(x)`: whether a Float or Rational has an integer value.
pub fn int_valued(x: &Expr) -> bool {
    if x.is_Integer() {
        return true;
    }
    match x.as_float() {
        Some((m, _)) => m.exp >= 0,
        None => false,
    }
}

// ---------------------------------------------------------------------------
// comparisons and float(), as Python's operators call them on two Numbers

/// `isinstance(b, type(a))` for a strict subclass: Python's `a == b` and
/// `a < b` try `b`'s reflected method first when `type(b)` is a proper
/// subclass of `type(a)` (Zero, One and NegativeOne under Integer; Integer
/// and the constants under Rational).
fn is_proper_subclass(b: &Expr, a: &Expr) -> bool {
    b.cls() != a.cls()
        && match a.cls() {
            C::Integer => b.is_a(Tag::Integer),
            C::Rational => b.is_a(Tag::Rational),
            _ => false,
        }
}

/// Python's `a == b` for two Numbers (`Integer.__eq__`, `Rational.__eq__`,
/// `Float.__eq__`, `Infinity.__eq__`, ...).
pub fn number_eq(a: &Expr, b: &Expr) -> bool {
    debug_assert!(a.is_Number() && b.is_Number());
    if is_proper_subclass(b, a) {
        number_eq_method(b, a)
    } else {
        number_eq_method(a, b)
    }
}

/// `a.__eq__(b)` for two Numbers.
fn number_eq_method(a: &Expr, b: &Expr) -> bool {
    match a.cls() {
        // Integer.__eq__
        C::Integer | C::Zero | C::One | C::NegativeOne if b.is_Integer() => int(a) == int(b),
        // Rational.__eq__: `other` is a Number and never a NumberSymbol here.
        C::Integer | C::Zero | C::One | C::NegativeOne | C::Rational | C::Half => {
            b.is_Rational() && a.p() == b.p() && a.q() == b.q()
        }
        // Float.__eq__ -> Basic.__eq__: both are Numbers, so their
        // `_hashable_content()`s decide: `(_mpf_, _prec)` against another
        // Float's, a Rational's `(p, q)` or a singleton's `()`.
        C::Float => {
            a.ptr_eq(b)
                || match (a.as_float(), b.as_float()) {
                    (Some((m, p)), Some((n, q))) => m == n && p == q,
                    _ => false,
                }
        }
        // `other is S.Infinity or other == float('inf')`: no other Number
        // equals float('inf') (it sympifies to S.Infinity, which only
        // Infinity itself equals).
        C::Infinity => b.is_cls(C::Infinity),
        C::NegativeInfinity => b.is_cls(C::NegativeInfinity),
        // `other is S.NaN`
        C::NaN => b.is_cls(C::NaN),
        _ => unreachable!("number_eq on {}", a.class_name()),
    }
}

/// `bool(a < b)` for two Numbers, as Python's `<` dispatches it.
pub fn number_lt(a: &Expr, b: &Expr) -> bool {
    debug_assert!(a.is_Number() && b.is_Number());
    if is_proper_subclass(b, a) {
        number_rich(b, a, Rel::Gt)
    } else {
        number_rich(a, b, Rel::Lt)
    }
}

/// `bool(a > b)` for two Numbers, as Python's `>` dispatches it.
pub fn number_gt(a: &Expr, b: &Expr) -> bool {
    debug_assert!(a.is_Number() && b.is_Number());
    if is_proper_subclass(b, a) {
        number_rich(b, a, Rel::Lt)
    } else {
        number_rich(a, b, Rel::Gt)
    }
}

/// `__lt__` or `__gt__`.
#[derive(Clone, Copy, PartialEq)]
enum Rel {
    Lt,
    Gt,
}

impl Rel {
    /// The reflected comparison (`__lt__` <-> `__gt__`).
    fn reflected(self) -> Rel {
        match self {
            Rel::Lt => Rel::Gt,
            Rel::Gt => Rel::Lt,
        }
    }

    fn mpf(self, s: &Mpf, t: &Mpf) -> bool {
        match self {
            Rel::Lt => mlib::mpf_lt(s, t),
            Rel::Gt => mlib::mpf_gt(s, t),
        }
    }
}

/// `bool(a.__lt__(b))` / `bool(a.__gt__(b))` for two Numbers: each class's
/// method, falling back to `Expr.__lt__` / `Expr.__gt__`.
fn number_rich(a: &Expr, b: &Expr, op: Rel) -> bool {
    match a.cls() {
        // Integer.__lt__ / __gt__
        C::Integer | C::Zero | C::One | C::NegativeOne if b.is_Integer() => match op {
            Rel::Lt => int(a) < int(b),
            Rel::Gt => int(a) > int(b),
        },
        C::Integer | C::Zero | C::One | C::NegativeOne | C::Rational | C::Half => {
            rational_rich(a, b, op)
        }
        // Float.__lt__ / __gt__ (`other` is never a NumberSymbol here)
        C::Float => match float_frel(a, b, op) {
            Some(v) => v,
            None => expr_rich(a, b, op),
        },
        // Infinity, NegativeInfinity and NaN: `__lt__ = Expr.__lt__`, ...
        C::Infinity | C::NegativeInfinity | C::NaN => expr_rich(a, b, op),
        _ => unreachable!("number comparison on {}", a.class_name()),
    }
}

/// `Rational.__lt__` / `__gt__`: `_Rrel(other, '__gt__')` (resp.
/// `'__lt__'`), else `Expr.__lt__` / `Expr.__gt__`.
fn rational_rich(s: &Expr, other: &Expr, op: Rel) -> bool {
    // _Rrel; `other` is a Number and never a NumberSymbol.
    let attr = op.reflected();
    if other.is_Float() {
        // op = getattr(o, attr); return op(s)
        return number_rich(other, s, attr);
    }
    if other.is_Rational() {
        let s2 = Expr::integer(&s.p() * &other.q());
        let o2 = Expr::integer(&s.q() * &other.p());
        return number_rich(&o2, &s2, attr);
    }
    if other.is_number() && other.is_extended_real() == Some(true) {
        // return Integer(s.p), s.q*o: `o` is oo or -oo here and s.q > 0,
        // so s.q*o is o.
        return expr_rich(&Expr::integer(s.p()), other, op);
    }
    // rv is None: Expr.__lt__(self, other)
    expr_rich(s, other, op)
}

/// `Float._Frel(other, op)` for a Number `other` (`None` where it returns
/// None).
fn float_frel(a: &Expr, other: &Expr, op: Rel) -> Option<bool> {
    let (m, _) = a.as_float().expect("Float");
    if other.is_Rational() {
        let q = mlib::from_int(&other.q(), 0, mlib::ROUND_FAST);
        let smpf = mlib::mpf_mul(m, &q, 0, mlib::ROUND_FAST);
        let ompf = mlib::from_int(&other.p(), 0, mlib::ROUND_FAST);
        return Some(op.mpf(&smpf, &ompf));
    }
    if let Some((n, _)) = other.as_float() {
        return Some(op.mpf(m, n));
    }
    // `other.is_comparable and other not in (S.Infinity,
    // S.NegativeInfinity)`: the Numbers left are oo and -oo (excluded) and
    // nan (`is_comparable = False`).
    None
}

/// `bool(Expr.__lt__(a, b))`: `bool(StrictLessThan(a, b))` (resp.
/// `StrictGreaterThan`), whose `_Inequality.__new__` first rejects
/// non-real arguments and NaN, then asks `is_lt` / `is_gt`.
fn expr_rich(lhs: &Expr, rhs: &Expr, op: Rel) -> bool {
    for me in [lhs, rhs] {
        if me.is_extended_real() == Some(false) {
            raise(
                ErrorKind::TypeError,
                format!("Invalid comparison of non-real {}", me.class_name()),
            );
        }
        if me.is_cls(C::NaN) {
            raise(ErrorKind::TypeError, "Invalid NaN comparison");
        }
    }
    match op {
        Rel::Lt => not_ported("StrictLessThan._eval_relation (is_lt) on numbers"),
        Rel::Gt => not_ported("StrictGreaterThan._eval_relation (is_gt) on numbers"),
    }
}

/// `float(n)` for a Number: `Number.__float__`,
/// `mlib.to_float(self._as_mpf_val(53))`.
pub fn number_float(n: &Expr) -> f64 {
    debug_assert!(n.is_Number());
    mlib::to_float(&as_mpf_val(n, 53), false, mlib::ROUND_FAST).expect("not strict")
}

/// `int(n)` for the classes of `core.numbers` that define `__int__`
/// (`Integer`, `Rational`, `Float` and the `NumberSymbol`s); `None` for
/// any other class (`Expr.__int__` applies).
pub(crate) fn number_int(n: &Expr) -> Option<Int> {
    Some(match n.cls() {
        // Integer.__int__
        C::Integer | C::Zero | C::One | C::NegativeOne => int(n).clone(),
        // Rational.__int__: truncates toward zero
        C::Rational | C::Half => {
            let (p, q) = (n.p(), n.q());
            if p.is_negative() {
                -(-p).floor_div(&q).expect("q > 0")
            } else {
                p.floor_div(&q).expect("q > 0")
            }
        }
        // Float.__int__ (to_int rounds down)
        C::Float => {
            let (m, _) = n.as_float().expect("Float");
            if *m == mlib::FZERO {
                return Some(Int::ZERO);
            }
            mlib::to_int(m, None).unwrap_or_else(|e| raise_mpf(e))
        }
        C::Exp1 => Int::from(2),
        C::Pi => Int::from(3),
        C::GoldenRatio | C::TribonacciConstant => Int::from(1),
        C::EulerGamma | C::Catalan => Int::from(0),
        _ => return None,
    })
}

/// `equal_valued(x, y)`: `x == y`, except that a Float equals a Rational
/// (or another Float) of the same value whatever its precision.
pub fn equal_valued(x: &Expr, y: &Expr) -> bool {
    // Handle everything except Float/Rational first
    let (mut x, mut y) = (x, y);
    if !x.is_Float() && !y.is_Float() {
        return x.py_eq(y);
    } else if x.is_Float() && y.is_Float() {
        // Compare values without regard for precision
        return x.as_float().expect("Float").0 == y.as_float().expect("Float").0;
    } else if x.is_Float() {
        std::mem::swap(&mut x, &mut y);
    }
    if !x.is_Rational() {
        return false;
    }

    // Now y is Float and x is Rational.
    let m = y.as_float().expect("Float").0;
    let (p, q) = (x.p(), x.q());
    let man = if m.sign == 1 {
        -m.man.clone()
    } else {
        m.man.clone()
    };
    let exp = m.exp;
    if exp == 0 {
        // y odd integer
        q.is_one() && man == p
    } else if exp > 0 {
        // y even integer
        if !q.is_one() {
            return false;
        }
        if p.bit_length() != man.bit_length() + exp as u64 {
            return false;
        }
        (&man << exp as u64) == p
    } else {
        // y non-integer. Need p == man and q == 2**-exp
        if p != man {
            return false;
        }
        let neg_exp = (-exp) as u64;
        if q.bit_length() - 1 != neg_exp {
            return false;
        }
        (Int::ONE << neg_exp) == q
    }
}
