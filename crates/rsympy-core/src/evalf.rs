//! `sympy.core.evalf`: numerical evaluation on `rsympy-mpf`.
//!
//! SKELETON: signatures are the contract; bodies to be ported.

use crate::error::not_ported;
use crate::expr::Expr;

/// `expr.evalf(n)` with default options (`n` significant digits).
///
/// Only the path for a class `evalf_table` has no entry for (an
/// `AppliedUndef`, `Mod`, ...) whose `_eval_evalf` returns `None` is
/// ported: the object comes back unchanged.
pub fn evalf(e: &Expr, n: i64) -> Expr {
    if crate::generated_function::FUNCTION_INFO[e.cls() as usize].in_evalf_table {
        not_ported("evalf");
    }
    let prec = rsympy_mpf::dps_to_prec(n);
    // result = evalf(self, prec + 4, options): with no evalf_table entry
    // for type(x), that is "xe = x._eval_evalf(prec)", and
    // NotImplementedError when xe is None
    if crate::functions::eval_evalf_of(e, prec + 4).is_some() {
        not_ported("evalf: the as_real_imag of an _eval_evalf result");
    }
    // except NotImplementedError: fall back to the ordinary evalf
    match crate::functions::eval_evalf_of(e, prec) {
        None => e.clone(),
        Some(v) if !v.is_number() => v,
        Some(_) => not_ported("evalf: normalizing a numeric _eval_evalf result"),
    }
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
