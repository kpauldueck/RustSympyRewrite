//! SymPy's process-global state, kept per thread.
//!
//! - `global_parameters` (`evaluate`, `distribute`, `exp_is_pow`);
//! - `sympy.core.random.rng` and `factor_cache`, shared with ntheory as an
//!   [`NtCtx`];
//! - `Dummy._count`.
//!
//! [`reset`] puts everything back to the state the oracle records SymPy in
//! (`fresh_state()`: caches cleared, random streams seeded with 0).

use std::cell::{Cell, RefCell};

use rsympy_ntheory::NtCtx;

thread_local! {
    static EVALUATE: Cell<bool> = const { Cell::new(true) };
    static DISTRIBUTE: Cell<bool> = const { Cell::new(true) };
    static EXP_IS_POW: Cell<bool> = const { Cell::new(false) };
    static NT: RefCell<NtCtx> = RefCell::new(NtCtx::default());
    static DUMMY_COUNT: Cell<u64> = const { Cell::new(0) };
}

/// `global_parameters.evaluate`.
pub fn evaluate() -> bool {
    EVALUATE.with(Cell::get)
}

/// `global_parameters.distribute`.
pub fn distribute() -> bool {
    DISTRIBUTE.with(Cell::get)
}

/// `global_parameters.exp_is_pow`.
pub fn exp_is_pow() -> bool {
    EXP_IS_POW.with(Cell::get)
}

struct Restore<'a>(
    &'static std::thread::LocalKey<Cell<bool>>,
    bool,
    std::marker::PhantomData<&'a ()>,
);

impl Drop for Restore<'_> {
    fn drop(&mut self) {
        let v = self.1;
        self.0.with(|c| c.set(v));
    }
}

fn with_flag<T>(
    key: &'static std::thread::LocalKey<Cell<bool>>,
    value: bool,
    f: impl FnOnce() -> T,
) -> T {
    let old = key.with(|c| c.replace(value));
    let _restore = Restore(key, old, std::marker::PhantomData);
    f()
}

/// `with evaluate(value): f()`.
pub fn with_evaluate<T>(value: bool, f: impl FnOnce() -> T) -> T {
    with_flag(&EVALUATE, value, f)
}

/// `with distribute(value): f()`.
pub fn with_distribute<T>(value: bool, f: impl FnOnce() -> T) -> T {
    with_flag(&DISTRIBUTE, value, f)
}

/// Runs `f` on this thread's ntheory state (`rng` and `factor_cache`).
pub fn with_nt<T>(f: impl FnOnce(&mut NtCtx) -> T) -> T {
    NT.with(|nt| f(&mut nt.borrow_mut()))
}

/// The next `Dummy.dummy_index` (`Dummy._count += 1`).
pub(crate) fn next_dummy_index() -> u64 {
    DUMMY_COUNT.with(|c| {
        let v = c.get();
        c.set(v + 1);
        v
    })
}

/// The oracle's `fresh_state()`: default parameters, `factor_cache`
/// cleared, `sympy.core.random.seed(0)`, and mpmath's constant memo
/// cleared as in a new process.
pub fn reset() {
    EVALUATE.with(|c| c.set(true));
    DISTRIBUTE.with(|c| c.set(true));
    EXP_IS_POW.with(|c| c.set(false));
    NT.with(|nt| *nt.borrow_mut() = NtCtx::default());
    rsympy_mpf::clear_constant_cache();
}
