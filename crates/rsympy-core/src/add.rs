//! `sympy.core.add`: `Add`.
//!
//! SKELETON: the signatures below are the contract other modules use; the
//! bodies are to be ported from `sympy/core/add.py` (and `AssocOp` in
//! `sympy/core/operations.py`).

use crate::error::not_ported;
use crate::expr::Expr;
use crate::facts::Fact;
use crate::generated::ClassId as C;
use crate::numbers::s;
use crate::ops;

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
    if crate::ctx::distribute() {
        // Expr.__neg__
        return crate::mul::from_args(vec![s::negative_one(), a.clone()], Some(a.is_commutative()));
    }
    add(a.args().iter().map(ops::neg).collect())
}

/// `AssocOp._new_rawargs(*args)`: `_from_args` with `self`'s
/// commutativity, recomputed when `self` is non-commutative.
pub(crate) fn assoc_new_rawargs(e: &Expr, args: Vec<Expr>) -> Expr {
    let c = e.is_commutative();
    let is_commutative = if c == Some(false) { None } else { Some(c) };
    crate::construct::assoc_from_args(e.cls(), args, is_commutative)
}

/// `Add._new_rawargs(*args)`.
pub fn new_rawargs(e: &Expr, args: Vec<Expr>) -> Expr {
    assoc_new_rawargs(e, args)
}

/// `Add._eval_power(expt)`.
pub fn eval_power(base: &Expr, expt: &Expr) -> Option<Expr> {
    let _ = (base, expt);
    not_ported("Add._eval_power")
}

/// `Add.as_coeff_Add(rational)`.
#[allow(non_snake_case)]
pub fn as_coeff_Add(e: &Expr, rational: bool) -> (Expr, Expr) {
    let (coeff, args) = (e.arg(0), &e.args()[1..]);
    if coeff.is_Number() && !rational || coeff.is_Rational() {
        return (coeff.clone(), new_rawargs(e, args.to_vec()));
    }
    (s::zero(), e.clone())
}

/// `Add.as_coeff_add(*deps)` with no deps: `(coeff, terms)`.
pub fn as_coeff_add(e: &Expr) -> (Expr, Vec<Expr>) {
    let (coeff, notrat) = e.arg(0).as_coeff_add();
    if !coeff.is_cls(C::Zero) {
        let mut v = notrat;
        v.extend_from_slice(&e.args()[1..]);
        return (coeff, v);
    }
    (s::zero(), e.args().to_vec())
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
