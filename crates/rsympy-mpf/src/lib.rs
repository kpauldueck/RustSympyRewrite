//! A port of mpmath, the arbitrary-precision float library under SymPy's
//! `Float` and `evalf`, built up in the milestones listed in `docs/PLAN.md`.
//!
//! M1: `libmpf`, which covers raw mpf arithmetic with mpmath's rounding,
//! conversion to and from strings, ints, rationals and machine floats, and
//! `prec_to_dps`, with the constants `to_str` needs (`libelefun`'s pi, e,
//! ln 2, ln 10).
//!
//! M2: the rest of `libelefun` (powers, roots, exp, log, trigonometric and
//! hyperbolic functions and their inverses, `elefun`), all of `libmpc`
//! (`mpc`), the generic hypergeometric summation and AGM of `libhyper`
//! (`hyper`), and the number context SymPy's `evalf` calls through
//! (`ctx`: mpf/mpc values, `workprec`, the wrapped elementary functions).
//!
//! The port follows mpmath's pure-Python backend operation for operation,
//! at the same working precisions, so it produces the same digits.

mod constants;
pub mod ctx;
mod elefun;
pub mod hyper;
mod intmath;
mod mpc;
mod mpf;
mod pyfloat;

pub use constants::*;
pub use elefun::*;
pub use intmath::*;
pub use mpc::*;
pub use mpf::*;
