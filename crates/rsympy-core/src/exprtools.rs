//! `sympy.core.exprtools` (`factor_terms`, `gcd_terms`, `Factors`, `Term`,
//! `decompose_power`, `_monotonic_sign`) and the core-hosted
//! `sympy.simplify.radsimp` helpers `fraction`, `numer` and `denom`.
//!
//! SKELETON in part: signatures are the contract; bodies still to be
//! ported say so.

use std::sync::LazyLock;

use crate::error::not_ported;
use crate::expr::Expr;
use crate::facts::Fact;
use crate::generated::ClassId as C;
use crate::numbers::s;
use crate::ops;

/// `_eps`: `Dummy(positive=True)`, made once when SymPy imports exprtools.
/// Its index only has to differ from every other Dummy's.
static EPS: LazyLock<Expr> = LazyLock::new(|| {
    crate::symbol::dummy_with_index("Dummy_0", u64::MAX, &[(Fact::Positive, true)])
});

/// `factor_terms(expr, radical, clear, fraction, sign)`.
pub fn factor_terms(e: &Expr, radical: bool, clear: bool, fraction: bool, sign: bool) -> Expr {
    let _ = (e, radical, clear, fraction, sign);
    not_ported("factor_terms")
}

/// `gcd_terms(terms, isprimitive, clear, fraction)` for an expression.
pub fn gcd_terms(e: &Expr, isprimitive: bool, clear: bool, fraction: bool) -> Expr {
    let _ = (e, isprimitive, clear, fraction);
    not_ported("gcd_terms")
}

/// `decompose_power(expr)`: `(base, integer exponent)`. SymPy returns a
/// Python int; an exponent beyond `i64` is `NotPortedYet` here.
pub fn decompose_power(e: &Expr) -> (Expr, i64) {
    use crate::generated::ClassId as C;
    let expr = e;
    let (mut base, exp) = expr.as_base_exp();
    let (base, e) = if exp.is_Number() {
        if exp.is_Rational() {
            if !exp.is_Integer() {
                base = crate::power::pow(&base, &Expr::rational(1, exp.q()));
            }
            (base, exp.p())
        } else {
            (expr.clone(), rsympy_num::Int::ONE)
        }
    } else {
        let (exp, tail) = exp.as_coeff_Mul(true);
        if exp.is_cls(C::NegativeOne) {
            (crate::power::pow(&base, &tail), rsympy_num::Int::NEG_ONE)
        } else if !exp.is_cls(C::One) {
            let tail = crate::mul::keep_coeff(&Expr::rational(1, exp.q()), &tail, true, false);
            (crate::power::pow(&base, &tail), exp.p())
        } else {
            (expr.clone(), rsympy_num::Int::ONE)
        }
    };
    let e = e
        .as_i64()
        .unwrap_or_else(|| not_ported("decompose_power: an exponent beyond i64"));
    (base, e)
}

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
                return Some(EPS.clone());
            }
        } else if s.is_extended_negative() == Some(true) {
            if s.is_even() == Some(true) {
                return int(-2);
            } else if s.is_integer() == Some(true) {
                return Some(s::negative_one());
            } else {
                return Some(ops::neg(&EPS));
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

    not_ported("_monotonic_sign of an Add or a symbolic denominator (needs polys)")
}

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
