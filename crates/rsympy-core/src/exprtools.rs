//! `sympy.core.exprtools` (`factor_terms`, `gcd_terms`, `Factors`, `Term`,
//! `decompose_power`, `_monotonic_sign`) and the core-hosted
//! `sympy.simplify.radsimp` helpers `fraction`, `numer` and `denom`.
//!
//! SKELETON: signatures are the contract; bodies to be ported.

use crate::error::not_ported;
use crate::expr::Expr;

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

/// `_monotonic_sign(expr)`.
pub fn monotonic_sign(e: &Expr) -> Option<Expr> {
    let _ = e;
    not_ported("_monotonic_sign")
}

/// `fraction(expr, exact)`: `(numerator, denominator)`.
pub fn fraction(e: &Expr, exact: bool) -> (Expr, Expr) {
    let _ = (e, exact);
    not_ported("fraction")
}

/// `numer(expr)`.
pub fn numer(e: &Expr) -> Expr {
    fraction(e, false).0
}

/// `denom(expr)`.
pub fn denom(e: &Expr) -> Expr {
    fraction(e, false).1
}
