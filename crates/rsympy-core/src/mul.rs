//! `sympy.core.mul`: `Mul`.
//!
//! SKELETON: the signatures below are the contract other modules use; the
//! bodies are to be ported from `sympy/core/mul.py` (and `AssocOp` in
//! `sympy/core/operations.py`).

use crate::error::not_ported;
use crate::expr::Expr;
use crate::facts::Fact;
use crate::generated::ClassId as C;
use crate::numbers::s;
use crate::ops;

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
    let (mut c, mut args) = as_coeff_mul(a, true);
    if !args[0].is_cls(C::ComplexInfinity) {
        c = ops::neg(&c);
    }
    if !c.is_cls(C::One) {
        if args[0].is_Number() {
            args[0] = if c.is_cls(C::NegativeOne) {
                ops::neg(&args[0])
            } else {
                ops::mul(&args[0], &c)
            };
        } else {
            args.insert(0, c);
        }
    }
    from_args(args, Some(a.is_commutative()))
}

/// `AssocOp._new_rawargs(*args)` for a Mul: `_from_args`, recomputing
/// commutativity only when `e` is non-commutative.
pub fn new_rawargs(e: &Expr, args: Vec<Expr>) -> Expr {
    crate::add::assoc_new_rawargs(e, args)
}

/// `Mul._eval_power(expt)`.
pub fn eval_power(base: &Expr, expt: &Expr) -> Option<Expr> {
    let _ = (base, expt);
    not_ported("Mul._eval_power")
}

/// `Mul.as_coeff_Mul(rational)`.
#[allow(non_snake_case)]
pub fn as_coeff_Mul(e: &Expr, rational: bool) -> (Expr, Expr) {
    let (coeff, args) = (e.arg(0), &e.args()[1..]);
    if coeff.is_Number() {
        if !rational || coeff.is_Rational() {
            if args.len() == 1 {
                return (coeff.clone(), args[0].clone());
            }
            return (coeff.clone(), new_rawargs(e, args.to_vec()));
        } else if coeff.is_extended_negative() == Some(true) {
            let mut v = vec![ops::neg(coeff)];
            v.extend_from_slice(args);
            return (s::negative_one(), new_rawargs(e, v));
        }
    }
    (s::one(), e.clone())
}

/// `Mul.as_coeff_mul(*deps, rational=True)` with no deps: `(coeff, factors)`.
pub fn as_coeff_mul(e: &Expr, rational: bool) -> (Expr, Vec<Expr>) {
    let args = e.args();
    if args[0].is_Number() {
        if !rational || args[0].is_Rational() {
            return (args[0].clone(), args[1..].to_vec());
        } else if args[0].is_extended_negative() == Some(true) {
            let mut v = vec![ops::neg(&args[0])];
            v.extend_from_slice(&args[1..]);
            return (s::negative_one(), v);
        }
    }
    (s::one(), args.to_vec())
}

/// `Mul.as_numer_denom()`.
pub fn as_numer_denom(e: &Expr) -> (Expr, Expr) {
    let _ = e;
    not_ported("Mul.as_numer_denom")
}

/// `Mul.as_base_exp()`.
pub fn as_base_exp(e: &Expr) -> (Expr, Expr) {
    let mut e1: Option<Expr> = None;
    let mut bases = Vec::new();
    let mut nc = 0;
    for m in e.args() {
        let (b, x) = m.as_base_exp();
        if b.is_commutative() != Some(true) {
            nc += 1;
        }
        match &e1 {
            None => e1 = Some(x),
            Some(first) => {
                if x != *first || nc > 1 || !x.is_Integer() {
                    return (e.clone(), s::one());
                }
            }
        }
        bases.push(b);
    }
    (e.func(bases), e1.expect("a Mul has args"))
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
