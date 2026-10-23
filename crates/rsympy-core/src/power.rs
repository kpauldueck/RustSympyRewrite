//! `sympy.core.power`: `Pow`.
//!
//! SKELETON: the signatures below are the contract other modules use; the
//! bodies are to be ported from `sympy/core/power.py`.

use crate::error::not_ported;
use crate::expr::Expr;
use crate::facts::Fact;

/// `Pow(b, e)` under the current `global_parameters.evaluate`.
pub fn pow(b: &Expr, e: &Expr) -> Expr {
    pow_with(b, e, crate::ctx::evaluate())
}

/// `Pow(b, e, evaluate=evaluate)`: `Pow.__new__`.
pub fn pow_with(b: &Expr, e: &Expr, evaluate: bool) -> Expr {
    let _ = (b, e, evaluate);
    not_ported("Pow.__new__")
}

/// `Expr.__new__(Pow, b, e)` plus `obj.is_commutative = ...`: the node
/// `Pow.__new__` builds when nothing evaluates.
pub fn from_args(b: &Expr, e: &Expr) -> Expr {
    let c = match b.is_commutative() {
        Some(true) => e.is_commutative(),
        other => other,
    };
    crate::construct::node_with_commutative(
        crate::generated::ClassId::Pow,
        vec![b.clone(), e.clone()],
        c,
    )
}

/// `Pow._eval_power(expt)` (a power of a power).
pub fn eval_power(base: &Expr, expt: &Expr) -> Option<Expr> {
    let _ = (base, expt);
    not_ported("Pow._eval_power")
}

/// `Pow.as_base_exp()`.
pub fn as_base_exp(e: &Expr) -> (Expr, Expr) {
    let (b, x) = (e.arg(0), e.arg(1));
    if b.is_Rational() && b.p().is_one() && !b.q().is_one() {
        return (Expr::integer(b.q()), crate::ops::neg(x));
    }
    (b.clone(), x.clone())
}

/// `Pow.as_numer_denom()`.
pub fn as_numer_denom(e: &Expr) -> (Expr, Expr) {
    let _ = e;
    not_ported("Pow.as_numer_denom")
}

/// `Pow._eval_expand_power_base()` with default hints.
pub fn expand_power_base(e: &Expr) -> Expr {
    let _ = e;
    not_ported("Pow._eval_expand_power_base")
}

/// `type(e)._prop_handler[fact](e)` for a Pow.
pub(crate) fn handler(e: &Expr, fact: Fact) -> Option<bool> {
    let _ = (e, fact);
    not_ported("Pow assumptions")
}
