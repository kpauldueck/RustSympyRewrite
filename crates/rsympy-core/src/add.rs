//! `sympy.core.add`: `Add`.
//!
//! SKELETON: the signatures below are the contract other modules use; the
//! bodies are to be ported from `sympy/core/add.py` (and `AssocOp` in
//! `sympy/core/operations.py`).

use crate::error::not_ported;
use crate::expr::Expr;
use crate::facts::Fact;

/// `Add(*args)` under the current `global_parameters.evaluate`.
pub fn add(args: Vec<Expr>) -> Expr {
    add_with(args, crate::ctx::evaluate())
}

/// `Add(*args, evaluate=evaluate)`: `AssocOp.__new__` with `Add.flatten`.
pub fn add_with(args: Vec<Expr>, evaluate: bool) -> Expr {
    let _ = (args, evaluate);
    not_ported("Add.__new__")
}

/// `Add._from_args(args, is_commutative)`: no evaluation, args as given.
/// `is_commutative` is computed from the args when not given (`None` here
/// is SymPy's `is_commutative=None` argument).
pub fn from_args(args: Vec<Expr>, is_commutative: Option<Option<bool>>) -> Expr {
    crate::construct::assoc_from_args(crate::generated::ClassId::Add, args, is_commutative)
}

/// `_unevaluated_Add(*args)`.
pub fn unevaluated_add(args: Vec<Expr>) -> Expr {
    let _ = args;
    not_ported("_unevaluated_Add")
}

/// `Add.__neg__`.
pub fn neg(a: &Expr) -> Expr {
    let _ = a;
    not_ported("Add.__neg__")
}

/// `Add._eval_power(expt)`.
pub fn eval_power(base: &Expr, expt: &Expr) -> Option<Expr> {
    let _ = (base, expt);
    not_ported("Add._eval_power")
}

/// `Add.as_coeff_Add(rational)`.
#[allow(non_snake_case)]
pub fn as_coeff_Add(e: &Expr, rational: bool) -> (Expr, Expr) {
    let _ = (e, rational);
    not_ported("Add.as_coeff_Add")
}

/// `Add.as_coeff_add(*deps)` with no deps: `(coeff, terms)`.
pub fn as_coeff_add(e: &Expr) -> (Expr, Vec<Expr>) {
    let _ = e;
    not_ported("Add.as_coeff_add")
}

/// `Add.as_numer_denom()`.
pub fn as_numer_denom(e: &Expr) -> (Expr, Expr) {
    let _ = e;
    not_ported("Add.as_numer_denom")
}

/// `Add.could_extract_minus_sign()`.
pub fn could_extract_minus_sign(e: &Expr) -> bool {
    let _ = e;
    not_ported("Add.could_extract_minus_sign")
}

/// `type(e)._prop_handler[fact](e)` for an Add.
pub(crate) fn handler(e: &Expr, fact: Fact) -> Option<bool> {
    let _ = (e, fact);
    not_ported("Add assumptions")
}
