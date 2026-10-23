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

/// `pure_complex(v, or_real)`: `(a, b)` when `v` is `a + I*b` with Numbers
/// `a` and `b` (`b` nonzero), or `(v, 0)` for a Number `v` when `or_real`.
pub fn pure_complex(v: &Expr, or_real: bool) -> Option<(Expr, Expr)> {
    let (h, t) = v.as_coeff_Add(false);
    if t.truthy() {
        let (c, i) = t.as_coeff_Mul(false);
        if i.is_cls(crate::generated::ClassId::ImaginaryUnit) {
            return Some((h, c));
        }
    } else if or_real {
        return Some((h, crate::numbers::s::zero()));
    }
    None
}
