//! A port of mpmath, the arbitrary-precision float library under SymPy's
//! `Float` and `evalf`, built up in the milestones listed in `docs/PLAN.md`.
//!
//! M1 (this crate so far): `libmpf`, which covers raw mpf arithmetic with
//! mpmath's rounding, conversion to and from strings, ints, rationals and
//! machine floats, and `prec_to_dps`. It also has the constants `to_str`
//! needs (`libelefun`'s pi, e, ln 2, ln 10).
//!
//! The port follows mpmath's pure-Python backend operation for operation,
//! at the same working precisions, so it produces the same digits.

mod constants;
mod intmath;
mod mpf;

pub use constants::*;
pub use intmath::*;
pub use mpf::*;
