# Porting rules

The shared rulebook for everyone (people and agents) porting SymPy 1.14
into rsympy. `docs/PLAN.md` says what to build and in what order; this file
says how.

## The one rule

rsympy copies SymPy 1.14 exactly: same results, same argument order, same
errors, same quirks. When SymPy has a bug, port the bug. When you are not
sure what SymPy does, run it (see "The oracle environment") instead of
reasoning about it.

## Porting a function

1. **Read the SymPy source first**, from the pinned install:
   `/home/claude/venv/lib/python3.13/site-packages/sympy/` (or `.venv/` after
   `./ci.sh oracle` has created it). Port the code that is there, branch by
   branch, in the same order. Don't restructure, simplify or "fix" while
   porting; a faithful port keeps each mistranslation findable.
2. **Cite the source** in the doc comment: the Python name, e.g.
   ``/// `Mul.flatten(seq)`.`` Keep SymPy's local variable names where you can.
3. **Never guess.** A path you have not ported calls
   `error::not_ported("Class.method: what is missing")`, which raises
   `NotPortedYet`. A wrong answer is far worse than a missing one, because
   tests and the golden runner count `NotPortedYet` as "not yet", and a
   wrong answer as a regression.
4. **Test against live SymPy** with a differential vector file (below).
   Every branch you port should be hit by at least one vector.

## Python semantics that change results

- **Exceptions.** Python exceptions are panics carrying a `SympyError`
  (`error::raise(ErrorKind::TypeError, msg)`); `error::catch` and
  `catch_kind` are `try/except`. Use the same exception class SymPy raises.
  Messages need not match.
- **Three-valued logic.** `is_<fact>` returns `Option<bool>`. Python's
  `if x.is_positive:` is `== Some(true)`; `if x.is_positive is False:` is
  `== Some(false)`; `if not x.is_positive:` is `!= Some(true)`. Use
  `assume::fuzzy_and/fuzzy_or/fuzzy_not` for SymPy's `fuzzy_*`.
- **Truthiness of expressions.** `Basic` has no `__bool__`, so `if expr:`
  is true for any expression, except `S.false` (false), empty containers
  such as `Tuple()` (false, through `__len__`) and Relationals (which raise
  `TypeError`).
- **Dict order.** Python dicts iterate in insertion order and SymPy builds
  results by iterating them. Use `py::PyDict`, never `HashMap`, wherever
  SymPy iterates a dict.
- **Set order.** SymPy sometimes iterates a `set` of expressions. That
  order depends on Python's string hashing, which differs between hash
  seeds. If a result depends on it, SymPy itself is not deterministic:
  the oracle catches this (it regenerates under two hash seeds), and you
  must sort the same way SymPy does afterwards or report it.
- **Sorting.** `list.sort` / `sorted` with a key or `cmp_to_key` must use
  `compare::py_sort_by`, a port of CPython's sort, because SymPy's keys are
  not always total orders and a different stable algorithm gives a
  different order. Use `compare::sort_args`, `sorted_default` and
  `ordered` for SymPy's own helpers.
- **Integer division.** Python's `//` and `%` floor; use the `Int` methods
  that say so (`floor_div`, ...).
- **Floats.** `Float` arithmetic goes through `rsympy-mpf` with the
  precision rules of `core.numbers`; never through `f64` unless SymPy does.
- **Global state.** `global_parameters` (`evaluate`, `distribute`), the RNG
  and caches live in `ctx`; `ctx::with_evaluate(false, ...)` is
  `with evaluate(False):`.
- **Cache.** SymPy's `@cacheit` doesn't change results, so don't port it
  unless profiling says so.

## Code layout and ownership

- Modules follow SymPy's (see `crates/rsympy-core/src/lib.rs`). Put a port
  where SymPy has it.
- **Stub signatures are contracts.** A skeleton function's signature is what
  other modules build against. If you need it changed, say so in your
  report rather than changing it silently; adding a new function is fine.
- **Work only in the files your task names.** Additive changes to shared
  files (a new `pub fn`, a new variant, a module declaration) are fine;
  list them in your report so they can be merged.
- **Never edit by hand:** `crates/rsympy-core/src/generated.rs` (run
  `oracle/gen_core_tables.py`), anything under `oracle/cases/`,
  `oracle/diff/` (run the generator), `oracle/baseline.json`,
  `oracle/divergences.json`, `ci.sh`.
- No `unsafe`. No new dependencies without saying why; they must be pure
  Rust and MIT/Apache/BSD licensed (no GMP, FLINT, malachite).
- `cargo fmt`, and `cargo clippy --workspace --all-targets -- -D warnings`
  must be clean.

## The oracle environment

```
PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python /home/claude/venv/bin/python
```

(Python 3.13 with the pinned `sympy==1.14.0`, `mpmath==1.3.0`; `./ci.sh
oracle` builds the same thing in `.venv/`.) Always pass
`SYMPY_GROUND_TYPES=python`: rsympy copies SymPy's pure-Python ground types.

## Differential vectors

Every expected value comes from live SymPy, printed by a generator script;
nobody writes expected values by hand.

- A generator is `oracle/gen_diff_<area>.py`, writing
  `oracle/diff/core_<area>.json` through the helpers in
  `oracle/exprenc.py`. `./ci.sh oracle` runs every `gen_diff_*.py` under two
  hash seeds and fails if a committed file changes.
- Expressions travel as `exprenc.enc(e)`: `srepr(e, order='none')` (args in
  storage order) with undefined functions' assumptions spelled out. Rust
  reads them with `rsympy_core::testing::parse`, which builds the tree
  without evaluating it, so a vector can hand a function exactly the input
  SymPy's function received.
- Each vector runs from a fresh state (`exprenc.fresh_state()` in Python,
  `ctx::reset()` in Rust). Don't put `Dummy` objects in outputs (their
  indices are random in SymPy).
- The Rust side is `crates/rsympy-core/tests/diff_<area>.rs`, using
  `tests/common/mod.rs` to load the file and compare outcomes. A wrong
  answer (wrong value, wrong exception, Rust panic) always fails.
  `common::run` also fails on `NotPortedYet`; `common::run_partial(area, n,
  ...)` allows exactly `n` vectors to reach unported code. That count only
  ever goes down: lower it when your port makes vectors pass, never raise
  it to hide a regression.

## Reporting

When you finish a unit, report: what you ported (SymPy names), what you
left as `NotPortedYet` and why, the vector counts, any change to a shared
file or contract, and anything in SymPy you found surprising.
