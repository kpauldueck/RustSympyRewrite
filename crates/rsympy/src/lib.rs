//! rsympy: SymPy 1.14, rewritten in Rust to behave exactly like it.
//!
//! So far: exact numbers (`num`) and the number-theory base (`ntheory`).
//! `docs/PLAN.md` describes the tiers, and `tests/oracle.rs` measures how
//! many SymPy golden cases we reproduce.

pub use rsympy_ntheory as ntheory;
pub use rsympy_num as num;
