//! The base of `sympy.ntheory` (Tier 1 in `docs/PLAN.md`): primality,
//! `nextprime`, `perfect_power`, `multiplicity` and `factorint`, ported
//! from SymPy 1.14 with its Python ground types.
//!
//! SymPy's number theory reads and writes global state: the random stream
//! `sympy.core.random.rng` (Pollard rho, p-1 and ECM re-seed it) and
//! `factor_cache`. Here that state is an explicit [`NtCtx`].

mod ecm;
mod external;
mod factor;
mod generate;
mod primetest;

pub use external::{
    NtError, NtResult, bit_scan1, gcdext, invert, iroot, is_square, is_strong_bpsw_prp,
    is_strong_prp, jacobi, lucas_sequence, remove, sqrtrem,
};
pub use factor::{
    FactorCache, FactorOpts, Factors, NtCtx, divisors, factorint, multiplicity, perfect_power,
    perfect_power_raw, pollard_pm1, pollard_rho, primefactors,
};
pub use generate::{nextprime, nextprime_ith, primerange};
pub use primetest::{isprime, mr};

use rsympy_num::Int;

/// `integer_nthroot(y, n)` from `sympy.core.intfunc`: `(floor(y**(1/n)),
/// exact)`.
pub fn integer_nthroot(y: &Int, n: u64) -> NtResult<(Int, bool)> {
    iroot(y, n)
}
