//! `sympy.core.evalf`: numerical evaluation on `rsympy-mpf`.
//!
//! SKELETON: signatures are the contract; bodies to be ported.

use crate::error::not_ported;
use crate::expr::Expr;

/// `expr.evalf(n)` with default options (`n` significant digits).
pub fn evalf(e: &Expr, n: i64) -> Expr {
    let _ = (e, n);
    not_ported("evalf")
}

/// `expr._eval_evalf(prec)` (binary precision), or `None`.
pub fn eval_evalf(e: &Expr, prec: i64) -> Option<Expr> {
    let _ = (e, prec);
    not_ported("_eval_evalf")
}
