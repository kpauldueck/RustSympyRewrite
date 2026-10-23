//! `sympy.core.symbol`: `Symbol`, `Dummy` and `Wild`.

use std::sync::Arc;

use crate::ctx;
use crate::error::{ErrorKind, raise};
use crate::expr::{Data, Expr, SymbolData};
use crate::facts::{Fact, Kb};
use crate::generated::ClassId as C;

/// SymPy draws `Dummy._base_dummy_index` from an unseeded RNG, so it
/// differs between processes; rsympy fixes it.
const BASE_DUMMY_INDEX: u64 = 1_000_000;

/// `Symbol._canonical_assumptions(**assumptions)`: the deduced KB and the
/// assumptions as given.
fn canonical(assumptions: &[(Fact, bool)]) -> (Kb, Vec<(Fact, bool)>) {
    let orig = assumptions.to_vec();
    let mut facts: Vec<(Fact, Option<bool>)> =
        assumptions.iter().map(|&(f, v)| (f, Some(v))).collect();
    if !assumptions.iter().any(|(f, _)| *f == Fact::Commutative) {
        facts.push((Fact::Commutative, Some(true)));
    }
    (Kb::deduced(&facts), orig)
}

fn make(cls: C, name: &str, assumptions: &[(Fact, bool)], dummy_index: Option<u64>) -> Expr {
    let (kb, orig) = canonical(assumptions);
    let data = SymbolData {
        name: Arc::from(name),
        assumptions0: kb,
        assumptions_orig: orig,
        dummy_index,
    };
    Expr::make_with_kb(cls, Data::Sym(Arc::new(data)), vec![], kb)
}

/// `Symbol(name)`.
pub fn symbol(name: &str) -> Expr {
    make(C::Symbol, name, &[], None)
}

/// `Symbol(name, **assumptions)`. Raises `InconsistentAssumptions` for
/// contradictory assumptions, as SymPy does.
pub fn symbol_with(name: &str, assumptions: &[(Fact, bool)]) -> Expr {
    make(C::Symbol, name, assumptions, None)
}

/// `Dummy(name, **assumptions)`.
pub fn dummy(name: Option<&str>, assumptions: &[(Fact, bool)]) -> Expr {
    let count = ctx::next_dummy_index();
    let name = match name {
        Some(n) => n.to_string(),
        None => format!("Dummy_{count}"),
    };
    make(C::Dummy, &name, assumptions, Some(BASE_DUMMY_INDEX + count))
}

/// `Dummy(name, dummy_index=index, **assumptions)`.
pub fn dummy_with_index(name: &str, index: u64, assumptions: &[(Fact, bool)]) -> Expr {
    make(C::Dummy, name, assumptions, Some(index))
}

/// `Wild(name, **assumptions)` with no `exclude` or `properties`.
pub fn wild(name: &str, assumptions: &[(Fact, bool)]) -> Expr {
    make(C::Wild, name, assumptions, None)
}

/// `Symbol._eval_subs(old, new)`: a symbol is a power with exponent 1 when
/// `old` is a power, so `x.subs(x**2, y)` can give `sqrt(y)` for a
/// positive `x`.
pub(crate) fn eval_subs(e: &Expr, old: &Expr, new: &Expr) -> Option<Expr> {
    if old.is_Pow() {
        let p = crate::power::pow_with(e, &crate::numbers::s::one(), false);
        return crate::power::eval_subs(&p, old, new);
    }
    None
}

/// `Symbol._sanitize`: converts `(name, value)` keyword assumptions,
/// rejecting unknown names the way `_filter_assumptions` would ignore them.
pub fn parse_assumptions(kwargs: &[(&str, Option<bool>)]) -> Vec<(Fact, bool)> {
    let mut out = Vec::new();
    for &(k, v) in kwargs {
        let Some(f) = Fact::from_name(k) else {
            raise(
                ErrorKind::TypeError,
                format!("unexpected keyword argument '{k}'"),
            );
        };
        match v {
            None if f == Fact::Commutative => raise(
                ErrorKind::ValueError,
                "commutativity must be True or False.",
            ),
            None => {}
            Some(b) => out.push((f, b)),
        }
    }
    out
}
