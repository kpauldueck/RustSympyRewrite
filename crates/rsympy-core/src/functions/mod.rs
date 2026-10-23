//! `sympy.core.function` (`Application`/`Function.__new__`, the `eval`
//! protocol, `AppliedUndef`) and the `functions.elementary` kernel.
//!
//! SKELETON: signatures are the contract; bodies to be ported.
//!
//! Submodules mirror SymPy's files:
//! - `application`: `Function.__new__`, nargs checking, auto-evalf;
//! - `exponential`: `exp`, `log`;
//! - `trigonometric`: `sin`, `cos`, `tan`, `cot`, `sec`, `csc`, inverses;
//! - `complexes`: `re`, `im`, `Abs`, `sign`, `arg`, `conjugate`;
//! - `integers`: `floor`, `ceiling`;
//! - `combinatorial`: `factorial`, `binomial`, `RisingFactorial`,
//!   `FallingFactorial`;
//! - `misc`: `sqrt`, `root`, `Min`, `Max`, `Mod`.

use std::sync::Arc;

use crate::error::not_ported;
use crate::expr::{Expr, UndefData};
use crate::facts::Fact;
use crate::generated::ClassId;

/// `cls(*args)`: `Function.__new__` for a defined function class,
/// including its `eval`.
pub fn call(cls: ClassId, args: Vec<Expr>) -> Expr {
    let _ = (cls, args);
    not_ported("Function.__new__")
}

/// `Function(name, **assumptions)` (`UndefinedFunction.__new__` for a
/// string name).
pub fn undefined_function(name: &str, assumptions: &[(Fact, bool)]) -> Arc<UndefData> {
    // assumptions = Symbol(name, **assumptions).assumptions0, dropping
    // commutative unless it was given.
    let sym = crate::symbol::symbol_with(name, assumptions);
    let mut kwargs = sym.symbol_data().expect("a Symbol").assumptions0;
    if !assumptions.iter().any(|(f, _)| *f == Fact::Commutative) {
        kwargs.remove(Fact::Commutative);
    }
    // The class's is_<fact> attributes become its default_assumptions
    // (AppliedUndef contributes none of its own).
    let defaults = crate::facts::Kb::deduced(&kwargs.items(false));
    Arc::new(UndefData {
        name: Arc::from(name),
        kwargs,
        assumptions: defaults,
    })
}

/// `f(*args)` for an undefined function `f = Function(name)`.
pub fn call_undefined(f: &Arc<UndefData>, args: Vec<Expr>) -> Expr {
    let _ = (f, args);
    not_ported("AppliedUndef.__new__")
}

/// `exp(x)`.
pub fn exp(x: &Expr) -> Expr {
    call(ClassId::Exp, vec![x.clone()])
}

/// `exp.as_base_exp()`.
pub fn exp_as_base_exp(e: &Expr) -> (Expr, Expr) {
    let _ = e;
    not_ported("exp.as_base_exp")
}

/// `exp.as_numer_denom()`.
pub fn exp_as_numer_denom(e: &Expr) -> (Expr, Expr) {
    let _ = e;
    not_ported("exp.as_numer_denom")
}

/// `base._eval_power(exp)` for a class outside `core` that defines it
/// (`owner` is that class: `ExpBase`, `Abs`, `sign`, `Piecewise`, ...).
pub fn eval_power(owner: &str, base: &Expr, exp: &Expr) -> Option<Expr> {
    let _ = (base, exp);
    not_ported(format!("{owner}._eval_power"))
}

/// The `_eval_is_<fact>` handlers of function classes (and of numeric
/// singletons, which only have `Expr`'s).
pub(crate) fn handler(e: &Expr, fact: Fact) -> Option<bool> {
    if !e.is_a(crate::generated::Tag::Application) {
        return crate::numbers::singleton_handler(e, fact);
    }
    not_ported(format!("{} assumptions", e.class_name()))
}
