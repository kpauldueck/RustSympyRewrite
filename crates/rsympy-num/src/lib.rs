//! Exact numbers for rsympy.
//!
//! - [`Int`]: arbitrary-precision integers with an inline `i64` fast path and
//!   a `dashu` fallback, using Python's floor-division and modulo semantics.
//! - [`Rational`]: reduced fractions with a positive denominator.
//! - [`PyRandom`]: CPython's MT19937 `random.Random`, so seeded SymPy code
//!   replays the same random sequences.
//!
//! `ModInt` arrives with polys (Tier 3). Floats live in `rsympy-mpf`.

mod int;
mod random;
mod rational;

pub use int::{Int, ZeroDivision};
pub use random::{EmptyRange, PyRandom};
pub use rational::Rational;
