//! SymPy's expression core (Tier 2 of `docs/PLAN.md`), ported from SymPy
//! 1.14 to behave 1:1.
//!
//! Layout follows SymPy's modules:
//!
//! | module | SymPy source |
//! |---|---|
//! | [`expr`] | `Basic`/`Expr` nodes, equality, hashing |
//! | [`facts`], [`assume`] | `core.facts`, `core.assumptions` (`is_*`) |
//! | [`numbers`] | `core.numbers` |
//! | [`symbol`] | `core.symbol` |
//! | [`compare`] | `Basic.compare`, `sort_key`, `core.sorting`, `list.sort` |
//! | [`add`], [`mul`], [`power`] | `core.add`, `core.mul`, `core.power` |
//! | [`ops`] | Python's binary operators on `Expr` (`__add__`, ...) |
//! | [`methods`] | `Expr` methods dispatched on class (`as_coeff_Mul`, ...) |
//! | [`exprtools`] | `core.exprtools` and `simplify.radsimp.fraction` |
//! | [`functions`] | `core.function` and the `functions.elementary` kernel |
//! | [`evalf`] | `core.evalf` |
//! | [`printing`] | `printing.str`, `printing.repr`, `printing.precedence` |
//! | [`parsing`], [`sympify`] | `parsing.sympy_parser`, `core.sympify` |
//! | [`py`] | Python built-ins results depend on (`dict` order, `repr(str)`) |
//! | [`testing`] | the oracle's expression encoding, for differential tests |
//!
//! Errors follow Python: see [`error`].

// The skeleton stage: many contract functions have no callers yet.
#![allow(dead_code)]
// Expr caches assumptions in an atomic, but Hash and Eq never read it.
#![allow(clippy::mutable_key_type)]

pub mod add;
pub mod assume;
pub mod compare;
pub mod construct;
pub mod ctx;
pub mod error;
pub mod evalf;
pub mod expr;
pub mod exprtools;
pub mod facts;
pub mod functions;
mod generated;
mod generated_compare;
pub mod methods;
pub mod mul;
pub mod numbers;
pub mod ops;
pub mod parsing;
pub mod power;
pub mod printing;
pub mod py;
pub mod symbol;
pub mod sympify;
pub mod testing;

pub use error::{ErrorKind, SympyError, SympyResult, catch};
pub use expr::Expr;
pub use facts::Fact;
pub use generated::{ClassId, Flag, Tag};
pub use numbers::s;
