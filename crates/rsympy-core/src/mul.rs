//! `sympy.core.mul`: `Mul`.
//!
//! SKELETON: the signatures below are the contract other modules use; the
//! bodies are to be ported from `sympy/core/mul.py` (and `AssocOp` in
//! `sympy/core/operations.py`).

use crate::error::not_ported;
use crate::expr::Expr;
use crate::facts::Fact;

/// `Mul(*args)` under the current `global_parameters.evaluate`.
pub fn mul(args: Vec<Expr>) -> Expr {
    mul_with(args, crate::ctx::evaluate())
}

/// `Mul(*args, evaluate=evaluate)`: `AssocOp.__new__` with `Mul.flatten`.
pub fn mul_with(args: Vec<Expr>, evaluate: bool) -> Expr {
    let _ = (args, evaluate);
    not_ported("Mul.__new__")
}

/// `Mul._from_args(args, is_commutative)`: no evaluation, args as given.
/// `is_commutative` is computed from the args when not given (`None` here
/// is SymPy's `is_commutative=None` argument).
pub fn from_args(args: Vec<Expr>, is_commutative: Option<Option<bool>>) -> Expr {
    crate::construct::assoc_from_args(crate::generated::ClassId::Mul, args, is_commutative)
}

/// `_unevaluated_Mul(*args)`.
pub fn unevaluated_mul(args: Vec<Expr>) -> Expr {
    let _ = args;
    not_ported("_unevaluated_Mul")
}

/// `_keep_coeff(coeff, factors, clear, sign)`.
pub fn keep_coeff(coeff: &Expr, factors: &Expr, clear: bool, sign: bool) -> Expr {
    let _ = (coeff, factors, clear, sign);
    not_ported("_keep_coeff")
}

/// `Mul.__neg__`.
pub fn neg(a: &Expr) -> Expr {
    let _ = a;
    not_ported("Mul.__neg__")
}

/// `Mul._eval_power(expt)`.
pub fn eval_power(base: &Expr, expt: &Expr) -> Option<Expr> {
    let _ = (base, expt);
    not_ported("Mul._eval_power")
}

/// `Mul.as_coeff_Mul(rational)`.
#[allow(non_snake_case)]
pub fn as_coeff_Mul(e: &Expr, rational: bool) -> (Expr, Expr) {
    let _ = (e, rational);
    not_ported("Mul.as_coeff_Mul")
}

/// `Mul.as_coeff_mul(*deps, rational=True)` with no deps: `(coeff, factors)`.
pub fn as_coeff_mul(e: &Expr, rational: bool) -> (Expr, Vec<Expr>) {
    let _ = (e, rational);
    not_ported("Mul.as_coeff_mul")
}

/// `Mul.as_numer_denom()`.
pub fn as_numer_denom(e: &Expr) -> (Expr, Expr) {
    let _ = e;
    not_ported("Mul.as_numer_denom")
}

/// `Mul.as_base_exp()`.
pub fn as_base_exp(e: &Expr) -> (Expr, Expr) {
    let _ = e;
    not_ported("Mul.as_base_exp")
}

/// `Mul.could_extract_minus_sign()`.
pub fn could_extract_minus_sign(e: &Expr) -> bool {
    let _ = e;
    not_ported("Mul.could_extract_minus_sign")
}

/// `type(e)._prop_handler[fact](e)` for a Mul.
pub(crate) fn handler(e: &Expr, fact: Fact) -> Option<bool> {
    let _ = (e, fact);
    not_ported("Mul assumptions")
}
