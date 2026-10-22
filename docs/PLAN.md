# Plan: SymPy, rewritten in Rust

**Goal.** Rewrite all of SymPy in Rust so that it behaves exactly like SymPy 1.14, 1:1, while being faster, smaller in memory and memory-safe. Paul's homework app doesn't set the scope; it sets the **order**, so what the app uses is ported first.

**Supporting documents:**
- [research/rust-rewrites.md](research/rust-rewrites.md): lessons from other large rewrites.
- [research/sympy-module-survey.md](research/sympy-module-survey.md): every SymPy module, with its algorithms, real dependencies, traps and difficulty. It was written before the 1:1 decision, so where it recommends a student-friendly or "strict" variant, this plan overrides it.
- `oracle/sympy_inventory.json`: sizes and test counts.
- The app's SymPy-usage audit (`app_sympy_audit.md`, shared by Paul in the project chat and kept out of the repo). It sets the priority order, and the oracle's `source: app` cases cite it.

## Decisions

**Settled with Paul, 2026-10-22:**
1. **Rust is the product.** It is a standalone Rust library with a first-class Rust API. Python is a bridge, not a runtime requirement (see `rsympy-py` below).
2. **Permissive licences only** (MIT OR Apache-2.0).
   - Big integers use `dashu`. `malachite` is LGPL-3.0, so it's excluded, as are GMP and FLINT.
   - mpmath is BSD, so we port its algorithms.
3. **Independent of upstream SymPy.** SymPy 1.14.0, pinned, is the answer key; we don't track later releases. `rsympy-py` vendors SymPy 1.14.0's Python source for the transition rule and the Python-only fallbacks, so it never depends on whatever `sympy` happens to be installed. We keep SymPy's BSD notice wherever we port or vendor its code.
4. **Full SymPy scope.** The app's audit sets priority only.
5. **Copy SymPy 1:1.** rsympy reproduces SymPy's observable behaviour exactly, including:
   - canonical forms and `.args` order;
   - printed strings and LaTeX;
   - Float digits;
   - which exceptions are raised;
   - SymPy's bugs.

   There are no student-friendly departures and no second "strict" mode. What the homework app does on top of SymPy stays in the app, as it does today; see "The app's own logic".

**Settled by this plan (reversible, but everything else assumes them):**

6. **Port faithfully.** About 91% of SymPy's code (by lines) is ported algorithm-for-algorithm, because SymPy's tests compare exact output shapes produced by specific heuristics. About 6% is redesigned internally with identical behaviour. The remaining 3% is Python-bound code that `rsympy-py` serves with SymPy's own Python, the benchmark scripts, and parsing.fortran. See "Strategy by module".
7. **A non-evaluating core.**
   - Raw constructors build exactly what they're given.
   - SymPy's automatic evaluation is a separate `canonicalize` layer, controlled by an explicit context that reproduces SymPy's `evaluate` flag, `UnevaluatedExpr` and `evaluate=False` behaviour exactly.

   Baking evaluation into the core is the mistake that kept SymEngine from replacing SymPy's core.

**Where "exactly" stops.** Four things can't be copied literally. Each failing case they cause is classified in the oracle rather than counted as a bug:
- **SymPy isn't deterministic everywhere.**
  - A few outputs depend on Python's hash order, SymPy's random streams or call history (for example `factor_cache`, or the Piecewise branches of `integrate`).
  - rsympy does not emulate CPython's hash values or set and dict iteration order.
  - The oracle records SymPy's answer under `PYTHONHASHSEED=0`, seeded random streams and cleared caches, and CI regenerates under a second hash seed.
  - A case whose answer changes between seeds is classified `unstable`. Every other case must match exactly.
- **Running Python code.**
  - `sympify` and `parse_expr` run token transformations and then Python's `eval`; `lambdify` builds and runs Python functions.
  - The Rust parser reproduces all of `sympy_parser`'s built-in transformations and the slice of Python's expression grammar and evaluation rules that `eval` applies to SymPy input: literals, tuples, lists, sets and dicts, operators, calls with keyword arguments, and which inputs raise which exception. `local_dict` and `global_dict` become name maps.
  - It can't run user-written transformation functions, arbitrary Python objects, or the functions `lambdify` returns; it prints the same code `lambdify` would. `rsympy-py` falls back to SymPy's Python for those, and the rest is classified `unportable`.
- **Python-bound code.** Some of SymPy exists to drive Python itself or Python libraries:
  - the IPython/Jupyter integration (`interactive`) and the test runner (`testing`);
  - stats.sampling (numpy, scipy, pymc);
  - the matplotlib and pyglet plot backends, `experimental_lambdify`, physics.control's plots and physics.quantum's circuit plots;
  - `autowrap` (Cython, f2py) and `llvmjitcode` (llvmlite);
  - parsing.fortran (LFortran).

  `rsympy-py` serves these with SymPy's own Python. The pure-Rust API raises `NotPortedYet` for them.
- **Speed and memory** are meant to differ: that's the point of the rewrite.

## How big this is

| | |
|---|---|
| SymPy 1.14 non-test source | 497k lines across 70 planning units |
| SymPy test suite | 256k lines, 12.1k `test_` functions, plus test-class methods (e.g. biomechanics has about 215) and about 32k lines of doctests |
| mpmath, ported as `rsympy-mpf` | 47.7k lines, of which SymPy uses a large part |
| Survey difficulty | 10 XL, 26 L, 20 M, 14 S |
| Conventional effort, SymPy units only | **257–692 person-weeks**, roughly 5–14 person-years for a skilled human team |

The bands are S 0.2–1, M 1–4, L 4–13 and XL 13–26 person-weeks per unit. The XL units are core, polys, printing, simplify, integrals, solvers, solvers.ode, functions.special, physics.quantum and tensor.

**Not in that range:**
- mpmath, which is XL on its own;
- `rsympy-py`;
- tooling;
- integration.

The range is wide because nobody knows yet where in it we are. The checkpoint after Tier 2 replaces it with a measured number.

**Agent-driven ports, read carefully.** Every figure below is self-reported.
- **tsc-rs** took about two weeks. But it ported from Go, a statically typed port of the original, and an earlier attempt had spent more than $400k and stalled at 84% compatibility.
- **Bun**'s 6-day port has not shipped, and it carries 13,365 `unsafe` sites.
- **Microsoft's Copilot runtime** took 14.5 weeks with about 3 developer-weeks of human time. It was Microsoft's own TypeScript, ported by people who knew it, and it still had dozens of regressions that compiled fine.

SymPy is dynamic Python whose output shapes are fixed by heuristics, which puts it nearer the hard end. Every success had an executable oracle, so the oracle is the first deliverable.

**Checkpoint after Tier 2 (go/no-go).** Compare what Tiers 0–2 actually cost with the estimate. Measure the core's speed and memory against SymPy. If the core isn't clearly faster and smaller, or the cost is tracking past the top of the band, stop and re-plan before polys, which is the next big spend.

## Strategy by module

Details per unit are in the survey.

| Strategy | Units | Share of lines |
|---|---|---:|
| Port faithfully | Everything not listed below. That includes units the survey suggested deferring or keeping in Python: holonomic, unify, plotting's sampling, series and text backend, parsing.autolev (emits the same Python text), parsing.c (through the safe `clang` crate), the deprecated matrix API, physics.units, sandbox. solvers.ode runs as Python on the Rust core until Tier 7, because it leans on integrate | 91.1% |
| Redesign internals, same behaviour | multipledispatch → generated match tables; utilities → Rust helpers; external → Rust crates; polys.domains ground types → one `Int`/`Rational`/`ModInt`; parsing and parsing.latex → Rust parsers without `eval` that reproduce what SymPy's parsers return (see Tier 4); the functions protocol → one `Head` trait | 6.2% |
| Only in `rsympy-py` | the Python-bound code listed under "Where 'exactly' stops": interactive, testing, stats.sampling, the matplotlib and pyglet backends, `experimental_lambdify`, control and circuit plots, `autowrap`, `llvmjitcode` | 2.4% |
| Rewritten as criterion benches | the benchmarks directories | 0.2% |
| Served by `rsympy-py` until LFortran can be called from Rust | parsing.fortran | 0.1% |

## Architecture

```
crates/
  rsympy-num/    Int (inline i64, dashu fallback) with Python floor-div/mod semantics, Rational, ModInt,
                 CPython-compatible MT19937 (tests replay SymPy's random sequences)
  rsympy-mpf/    port of mpmath, in milestones (see "rsympy-mpf")
  rsympy-core/   nodes, canonicalize, orderings, assumptions, Head trait, Ctx, hook traits
  rsympy-*/      domain crates (polys, simplify, solvers, series, integrals, printing, parsing, ...)
                 that depend on core and implement its hook traits
  rsympy/        facade: re-exports the public API and installs every hook at startup
  rsympy-py/     PyO3 module with SymPy's Python API (see "rsympy-py")
oracle/          dev-only Python: pinned SymPy, golden-case generators, module inventory
```

**Crate layout and cycles.** SymPy's modules depend on each other in cycles. The survey verified these, and they are real, not just lazy imports:
- core ↔ functions.elementary (via `Pow.__new__`, evalf and assumptions);
- core ↔ simplify (via `fraction` and `clear_coefficients`);
- core → polys.numberfields (via `Expr.equals` and sign tests that fall back to `minimal_polynomial`);
- sets ↔ solvers;
- calculus.util ↔ solveset;
- limits ↔ simplify;
- integrals ↔ simplify ↔ solve.

Rust crates can't be cyclic, so cycles are broken by **late-bound hooks**:
- `rsympy-core` defines a hook trait for each call it makes upward;
- domain crates implement those traits;
- the facade installs the implementations.

Small helpers that core calls constantly (`fraction`, `clear_coefficients` and the other core-hosted simplify helpers in Tier 2) move into core instead.

Once hooks exist, splitting into crates is cheap, and it keeps builds parallel and incremental; one crate of this size would build slowly. So we start split, track clean and incremental build times, and merge crates if hook plumbing costs more than it saves.

**Core design requirements from the survey.** Each of these is expensive to retrofit:
- **Exact orderings.** Two orderings must match SymPy exactly:
  - `Basic.compare` defines `.args` order. It needs each node kind's SymPy class name, including singleton names like `Half` and `NegativeOne`.
  - `sort_key`/`class_key` defines printed order. `Atom.sort_key` uses `str()`, so atom printing ships with core.
- **A virtual class hierarchy.** Tests, `class_key` and many algorithms ask `isinstance(e, Function)` and similar.
  - Each node kind carries its SymPy class path.
  - A table gives each kind's method resolution order and `class_key`.
- **One `Head` trait for functions.** It covers:
  - `eval` (auto-evaluation at special values) and the auto-evalf-when-all-args-are-Floats rule;
  - `nargs` and `inverse`;
  - the fuzzy assumption handlers (about 419 `_eval_is_*`);
  - about 528 rewrites, about 163 derivative hooks (`fdiff`), and about 127 series hooks (`_eval_nseries`, `_eval_as_leading_term`);
  - `_eval_evalf`, `_eval_subs`, `_eval_conjugate`/`transpose`/`adjoint`, `_eval_simplify`;
  - an open expand-hint registry;
  - per-printer hooks (`_latex`, `_sympystr`, ...).

  It must also allow **foreign heads**, whose methods call back into Python. Users' own `Function` and `Basic` subclasses are part of SymPy's API and need them permanently; SymPy classes not yet ported need them during the transition.
- **Binary-operator dispatch.** SymPy picks the class that handles `a*b` by `_op_priority`; this matters for matrices, quantities, vectors and `AccumBounds`. A `BinaryOp` trait with priorities is needed from day one.
- **Old assumptions as a u64 bitset `FactKB`.**
  - The 43 rules of `_assume_rules` are compiled at build time, with atomic per-node caches.
  - This is mostly for memory and determinism. Assumptions are about 20% of SymPy's runtime on app-like workloads, so even free assumptions cap the speed-up at about 1.25× (Amdahl's law). Speed has to come from everywhere at once.
- **Extension hooks in canonicalisation.**
  - Constructor post-processors keyed on argument kind (`MatrixExpr`, `Vector`, `TensExpr`, quantum states).
  - An open kind lattice for `Mul`.
  - Noncommutative `Mul` (a commutative part plus an ordered noncommutative part).
- **Node kinds hosted in core.**
  - `Order` and `AccumBounds`, which `Add` and `Mul` absorb.
  - A matrix node kind.
  - `Permutation` as an atom.
  - Unevaluated `Integral`, `Limit`, `Sum` and `Product`.
  - Side-table memos for the types that SymPy mutates lazily.
- **An explicit context, `Ctx`.**
  - `evaluate`, `distribute` and `exp_is_pow` become context, not globals.
  - **The RNG,** with the `sympy.core.random` API (`seed`, `_randint`, `_randrange`, `random_complex_number`), so seeded tests replay SymPy's sequences.
  - **An optional budget.** Every loop that can blow up calls `ctx.tick()?`. The budget is unlimited by default, so results are SymPy's. It lets a caller like the app stop a runaway call cleanly instead of killing a process, and retrofitting it later would mean touching every algorithm.
  - Everything is deterministic: no hash-order iteration.
- **Floats behave like mpmath.**
  - A Float's identity includes its precision (`Float(0.5) != Float(0.5, 30)`), and the `Float(2.0) != 2` rule from 1.13 holds.
  - `rsympy-mpf` ports mpmath operation by operation at the same working precision, so it should produce the same digits. The differential tests check that rather than assume it.
- **Python numeric semantics where SymPy exposes them:** floor `//` and `%`, `Mod`, Python's `round` on Python numbers, and SymPy's own `Expr.round`, which rounds the decimal value (`Float(2.675).round(2)` is `2.68`, where Python's `round(2.675, 2)` is `2.67`).

## The oracle

There are three layers, built in this order.

1. **Golden cases (exist today).** 1,248 cases in 17 topics. They come from `oracle/gen_cases.py` and `oracle/topics/` and are scored by `crates/rsympy/tests/oracle.rs`. They run in plain `cargo test`, before `rsympy-py` exists.
   - Every expected answer is what the pinned SymPy returns, computed live by the generator. When SymPy raises, the expected answer is the exception's class name.
   - **`source`** is either:
     - `sympy`: one SymPy call, with its keyword arguments in `options`; or
     - `app`: a fixed recipe of SymPy calls that the homework app runs, or plans to, such as its answer-equivalence check. Each recipe is written out in its topic module and implemented again in the Rust runner. These cases check that rsympy gives SymPy's answers along the app's call paths. They don't run the app's own code, so when the app changes a recipe, the topic module and the runner change with it.
   - **Generation is reproducible.**
     - Python 3.13, SymPy, mpmath and lark are pinned.
     - The generator refuses to run without Python ground types, and CI sets `PYTHONHASHSEED=0`.
     - Before each topic, caches (including `factor_cache`) are cleared and all three random streams are seeded: Python's `random`, `sympy.core.random.rng` and the assumptions shuffler.
     - The generator rejects malformed or contradictory cases.
     - CI regenerates the cases, diffs them against what is committed, then regenerates under a second hash seed and diffs again.
   - **Exact comparison only.**
     - A case passes when rsympy's `str()` of the answer is character-for-character SymPy's, or when rsympy raises the same exception class.
     - Cases also record SymPy's `latex()` of the answer (`expected_latex`). From Tier 4, when the LaTeX printer lands, the runner checks that too.
     - There is no "equivalent" or "same set in any order" mode, so rsympy never grades its own answers.
2. **SymPy's own test suite**, run through `rsympy-py`.
   - **Pinned environment:** `PYTHONHASHSEED=0`, Python ground types (no gmpy2/flint), mpmath 1.3.0, pytest and hypothesis.
   - **Extras environment:** adds numpy, scipy, pymc, antlr4 4.11, matplotlib, IPython, libclang, llvmlite, Cython, lxml, z3, matchpy, numexpr, numba, jax, tensorflow, torch and lfortran, as far as they install together. Without them the ANTLR LaTeX, autolev, array-library printer and lambdify, plotting, interactive, sampling, autowrap, mathml, SMT and codegen tests skip silently. Packages that can't be installed alongside the rest are listed, and their tests are recorded as skipped by environment, not as passing.
   - **Doctests** (about 32k lines) are scored separately, per docstring.
   - **Markers:**
     - `@XFAIL` tests are scored separately. An XFAIL that starts passing is a difference to explain, not a win.
     - `@slow` tests run nightly, not on every pull request.
     - Skip reasons are recorded.
   - **Test-dependency manifest.** Each SymPy test is run under `sys.setprofile`, recording every SymPy function it executes (file and qualified name), not just modules; code that runs at import time doesn't count. A checked-in map assigns each SymPy function to the tier that ports it, because many units are split across tiers: `simplify()` calls `hyperexpand` on every input, `Matrix.det` calls `dotprodsimp`, `apart` calls legacy `solve`, and logic's SAT code comes later than `boolalg`. Tier exits count only tests whose traced functions all sit in that tier or earlier, so no tier waits on a later one.
   - **Scoring** is per test function and test method, per module, on a dashboard.
   - **Project-level suites.** `core/tests/test_args.py` (1,007 functions that instantiate every class in SymPy) and `utilities/tests/test_wester.py` are owned at project level, not counted toward core.
3. **Differential fuzzing.** Random expressions are compared against SymPy on `srepr`, `.args` order, `str` and `latex`. Ordering parity can only be found this way.

**Gates.**
- **Enforced by CI today:**
  - Golden cases regenerate identically, under two hash seeds.
  - Every failing golden case is fixed or listed in `oracle/divergences.json` with a `category` and a `reason`. CI rejects malformed entries, unknown case ids, and entries for cases that now pass.
  - No golden topic's pass count drops below `oracle/baseline.json`.
- **Added when the dashboard exists:** no pull request lowers any SymPy test module's pass count. SymPy-suite classifications live next to the golden ones, keyed by pytest node id, under the same categories.
- **Divergence categories:**
  - **`bug`:** rsympy is wrong. Fix it.
  - **`unstable`:** SymPy's own answer depends on hash order, random streams or history.
  - **`unportable`:** the behaviour needs a Python interpreter, or CPython itself.

  There is no category for "rsympy is better than SymPy".
- **Enforced by review:** only a human may add a divergence or lower a baseline. Both files have CODEOWNERS entries, which bind once branch protection requires code-owner review.

**Benchmarks.**
- **Cases:** SymPy's `bench_symbench` (R1–R8, R10, R11; there is no R9), the 152 cases in `bench_meijerint`, the polys benchmarks, and the app's own cases.
- **Measures:** p50/p99 latency and peak memory, against SymPy 1.14.
- **Comparisons:**
  - Pure SymPy is the target to beat.
  - SymPy+python-flint is a reference ceiling. Expect to lose to it on large polynomial work: FLINT is decades of tuned C, and our licence rule excludes it.
- **Rule:** no speed claim without a measurement.

## Workstreams that span tiers

### rsympy-py
This starts in Tier 0 and grows every tier. It presents SymPy's Python API over the Rust core, runs SymPy's suite, and is how the app migrates.
- **Transition rule.** Until a hook's Rust implementation lands, `rsympy-py` serves it with SymPy's own Python code running on Rust objects. That lets a tier ship before the later tiers it calls into. The pure-Rust API instead returns a named `NotPortedYet` error, never a wrong answer.
- **Internal-API policy.** Many SymPy tests touch internals (`_assumptions`, `_eval_*`, private helpers).
  - `rsympy-py` provides shims for the internals tests use.
  - Tests are selected, never altered in meaning.
  - Under 1:1 these internals are behaviour too. A missing shim is a `bug`; only a test that needs CPython itself (frames, bytecode, `__dict__` layout, reference counts) is classified `unportable`.
- **Python-only behaviour.** `rsympy-py` falls back to its vendored SymPy Python for:
  - `parse_expr` with user-written transformations or arbitrary Python objects in its namespaces;
  - running the functions `lambdify` builds;
  - the Python-bound code listed under "Where 'exactly' stops".
- **Exceptions and deprecations.** Tests assert exception types (`raises(TypeError)`) and `SymPyDeprecationWarning`s.
  - `PORTING.md` maps each to a Rust error kind.
  - `rsympy-py` raises the matching Python class.

### rsympy-mpf
mpmath is ported in milestones, each landing with the tier that first needs it. The scope is what SymPy calls, not all of mpmath.

| Milestone | Tier | Contents |
|---|---|---|
| M1 | 1 | libmpf: `mpf` arithmetic and rounding, `to_str`, `prec_to_dps` |
| M2 | 2 | elementary functions, `mpc`, `hypsum` |
| M3 | 3 | `polyroots`, `findroot` |
| M4 | 4 | `pslq`, `identify` (for `nsimplify`) |
| M5 | 5 | `MDNewton` (for `nsolve`), `eig`, `lu_solve` |
| M6 | 6 | `quad`, special-function kernels |

### The app's own logic
The homework app wraps SymPy in its own rules, for example:
- its answer-equivalence check;
- its form checks;
- its unit and number input handling;
- its LaTeX workarounds.

Those rules stay in the app. When the app moves to Rust, they move with it, now calling rsympy instead of SymPy. The golden cases with `source: app` replay those recipes, and the matrix calls on the app's roadmap, against SymPy's answers.

The student-facing rules drafted before the 1:1 decision (base-10 `\log`, exact decimals, refusing `\overline`, unit-input rules) are in git history at commit 727fa5e if the app wants them for its own code.

## Roadmap

Tiers follow the dependency structure; inside each tier, what the app needs goes first. Units marked ⇄ can run in parallel tracks once their prerequisites land, which is where many agents help.

A golden topic or SymPy test module **passes** when every case either passes or carries an `unstable` or `unportable` divergence. A `bug` entry means it doesn't pass.

### Tier 0: Harness
- The golden-case oracle and CI, with reproducible generation and divergence validation (done, PR #1).
- The `rsympy-py` skeleton, the SymPy test-runner dashboard, the doctest layer and the marker policy.
- The test-dependency manifest.
- The pinned and extras oracle environments.
- Criterion benchmarks, plus a measurement of the app's current SymPy latency. That measurement is the baseline for the Tier 4 target.
- `PORTING.md` mapping rules:
  - Python class → Rust kind/`Head`;
  - `_eval_*` → trait method;
  - `@cacheit` → memo;
  - dispatch tables;
  - exception types and deprecation warnings.

**Exit:** SymPy, run against itself through the dashboard, reproduces SymPy's own pass, skip and xfail counts.

### Tier 1: Numbers
- `rsympy-num`, plus ntheory's base: `isprime`, `nextprime`, `perfect_power`, `multiplicity`, and `factorint(limit=)` reproduced exactly, because `sqrt` simplification depends on it.
- `rsympy-mpf` M1.
- The CPython-compatible RNG.

**Exit:**
- The ntheory-base tests pass.
- Differential tests of `rsympy-num` against Python ints, and of `rsympy-mpf` against mpmath, pass.

### Tier 2: Core kernel
In order:
1. Nodes, hash-consing, the class table, orderings, `FactKB`.
2. `Add`/`Mul`/`Pow` canonicalisation, `_eval_power`, `exprtools`, binary-operator dispatch.
3. `logic.boolalg` together with Relational.
4. `subs`/`xreplace`/`replace`/`match`/`Wild`.
5. The `Head` trait, `Lambda`, `Derivative`, `Subs`, `expand`, `evalf` (on `rsympy-mpf` M2).

Alongside that:
- **The functions.elementary kernel:** `exp`, `log`, `re`/`im`/`Abs`/`sign`, `floor`/`ceiling`, `Min`/`Max`, `Piecewise`, trig special values.
- **Other functions core reaches:** combinatorial numbers, plus the special functions core itself reaches:
  - `gamma`, `KroneckerDelta`, `Heaviside`, `DiracDelta`;
  - `chebyshevt` and the other orthogonal polynomials, which numberfields uses for trig minimal polynomials.
- **Core-hosted simplify helpers,** which core calls itself: `fraction`, `numer`, `denom`, `collect`, `collect_const`, `signsimp`, `powsimp`, `powdenest`, `factor_terms`, `clear_coefficients`.
- **Unevaluated containers:** `Integral`, `Limit`, `Sum`, `Product` (objects only; the engines come later), and tensor.indexed (`Indexed`, `IndexedBase`).
- **Absorbing node kinds:** `Order` and `AccumBounds`.
- **Printers:** `StrPrinter` and `ReprPrinter`.
- **Parsing:** `sympify`/`parse_expr` with every transformation in `sympy_parser` (the standard ones plus implicit multiplication and application, `convert_xor`, `rationalize` and the rest), as described under "Where 'exactly' stops".
- **Infrastructure:** multipledispatch tables, strategies (the rule combinators that fu, matrices.expressions and others use), class markers, and the core-misc namespace (`sympy/__init__`, `abc`, ...).
- **Sets phase 1:** `Interval`, `FiniteSet`, `Union`, `Intersection`, `Complement`, `Range`.

**Exit:**
- Core, `logic.boolalg`, sets-phase-1 and functions.elementary tests are at 95% or more, counting tests whose traced functions are in Tiers 0–2.
- The `printing` golden topic passes, and so do `numbers` apart from its polynomial `gcd`/`lcm` cases (Tier 3) and the `sympify` cases in `numeric_input` and `forms`.

Then the **go/no-go checkpoint** (see "How big this is").

### Tier 3: Polynomials and linear algebra over exact domains
Order:
1. Dense layer.
2. galoistools.
3. euclidtools/heugcd/sqf.
4. factortools (Zassenhaus, Wang).
5. Sparse rings.
6. `DomainMatrix` and `solve_lin_sys`.
7. `Poly`/`DMP` API.
8. partfrac.
9. `roots` (polyroots.py) and `CRootOf`.
10. `AlgebraicField` and `minimal_polynomial`.
11. agca (the parts that Gröbner and other units use).
12. Gröbner.

In parallel:
- `Matrix` over ZZ/QQ, which stores a `DomainMatrix`, together with the helpers those paths call (`dotprodsimp`).
- The linear-system slice of legacy `solve` (`solve` → `_solve_system` → `solve_linear_system`), which `apart`'s undetermined-coefficients path calls.
- The combinatorics permutation kernel, which the matrix tests need.
- `rsympy-mpf` M3.

**Exit:**
- Polys and exact-path matrices tests are at 95% or more, traced to Tiers 0–3.
- The `algebra` and `forms` golden topics pass, as do `numbers` in full and the `matrices` cases over ZZ/QQ.
- gcd and factor are at least 10× faster than pure SymPy on the polys benchmarks.
- The rustsmith GCD trap does not recur.

After Tier 3, **⇄ a geometry rational track** can start: `Point`, `Line`, `Segment` and `Polygon` over rationals.

### Tier 4: Simplify, printing and parsing (the app's serve path)
- **simplify basics:** `ratsimp`, `radsimp`, `logcombine`, `sqrtdenest`, and `nsimplify` (on `rsympy-mpf` M4).
- `fu`/`trigsimp`, then the `simplify()` pipeline. That includes `hyperexpand`'s entry point and the `hyper`/`meijerg` node kinds, because `simplify()` calls `hyperexpand` on every input; its formula tables come in Tiers 5 and 6.
- **Node shells the printers and parsers need,** including `ImageSet` and `ConditionSet`. Their evaluation comes in Tier 5.
- **The LaTeX printer,** including the subclassing API the app uses.
- **The LaTeX parsers.** SymPy has two, Lark (the app's) and ANTLR (the default). Reproducing their grammars isn't enough, because the result depends on the parser engine:
  - for Lark, the Earley parse with explicit ambiguity, SymPy's `_ambig` resolution and Lark 1.3.1's derivation order;
  - for ANTLR 4.11, its prediction and error recovery.

  Both must raise the same exceptions (`UnexpectedCharacters`, `LaTeXParsingError`, ...) on the same inputs. Each needs node shells for what it returns, including the bra-ket slice of physics.quantum.
- The pretty printer.
- **⇄ physics.units.**
- **⇄ tensor.array.**

**App milestones:**
- **A:** the app's serve path runs on rsympy through `rsympy-py`, with the app's own logic unchanged.
- **A2:** the app's matrix key checks over exact domains.

**Exit:**
- These golden topics pass: `simplify`, `equivalence`, `latex_parse`, `latex_print`, `numeric_input` and `units`. The `matrix_keys` cases over ZZ/QQ pass too.
- The app's answer check is faster at p99 than the app's measured SymPy latency.

### Tier 5: Calculus and solving (the app's compile path)
- The series protocol across core and functions, then Gruntz limits.
- **One cycle milestone,** ported together because these call each other: calculus.util, solveset, inequalities, sets phase 2 (`ImageSet`, `ConditionSet`) and diophantine.
- Legacy `solve`, `rsolve`, `nsolve` (on `rsympy-mpf` M5), and concrete summation, including `eval_sum_hyper` and the `hyper` formula tables of `hyperexpand` it uses. Meijer G stays with meijerint in Tier 6.
- `Matrix` symbolic (EXRAW) paths and eigen.
- **⇄ logic's SAT solver and the new assumptions (`ask`).**
- **⇄ geometry:** ellipses, circles and planes, which need solvers.
- **⇄ the finite stats slice:** `FiniteRV` and the finite distributions (`Die`, `Coin`, `Bernoulli`, `Binomial`, `Hypergeometric`), `P`, `E`, `given`, `density`.

**App Milestone B:** figure lint and key checks run on rsympy.

**Exit:** these golden topics pass:
- `equations`, and `calculus` apart from its integration cases;
- the rest of `matrix_keys`;
- `toolkit`;
- `geometry`;
- `probability`.

### Tier 6: Integration and special functions
The `Integral` object already exists from Tier 2; this tier adds the engines.
- ratint, trigintegrate, manualintegrate.
- heurisch, then Risch (needs `Poly` over towers of fraction fields).
- meijerint together with hyperexpand.
- Transforms and Laplace.
- Formal series.
- functions.special symbolic rules, in step with `rsympy-mpf` M6.

**Exit:** integrals and series are at 90% or more, and the `calculus` golden topic passes in full.

### Tier 7: Domains and the long tail
Mostly parallel tracks (⇄), each gated by its upstream units:
- combinatorics beyond the permutation kernel; BSGS and `tensor_can` are needed by tensor.
- stats (discrete, then continuous).
- solvers.ode, ported hint by hint. Until now it ran as Python on the Rust core.
- holonomic.
- unify.
- matrices.expressions and the deprecated matrix API.
- tensor.
- physics.vector, mechanics and biomechanics.
- sympy.vector, diffgeom, control, continuum_mechanics, optics, hep.
- physics.quantum (its LaTeX-parser slice came in Tier 4).
- secondquant, last, because its ordering keys on printed strings.
- physics' top-level modules: wigner, hydrogen, paulialgebra, sho, qho_1d, pring and physics.matrices (wigner earlier if the Tier 4 bra-ket slice needs Clebsch-Gordan).
- codegen and code printers, with byte-identical output so Python-side `lambdify` keeps working.
- The rest of utilities (iterables, enumerative, lambdify's printers) and the rest of printing (mathml, preview, dot, tree, tableform).
- plotting (sampling, series, text backend), parsing.autolev, parsing.c, parsing.mathematica, parsing.maxima, `ast_parser` and `sym_expr`.
- crypto, discrete, liealgebras, categories, algebras, sandbox.
- The advanced parts of ntheory, numberfields and agca.

**Exit:** full-suite conformance of 98% or more. Every remaining failure is classified, and none is classified `bug`.

## Working rules (humans and agents)
- **Port faithfully; don't clean up during a port.** Keep each mistranslation findable, as fish did.
- **Port SymPy's real algorithms,** including its heuristics and its bugs. Output shapes are the spec.
- **Agents never edit golden JSON or SymPy's tests.**
  - Generators write golden cases.
  - Tests are selected, never altered in meaning.
  - Agents don't add divergences or lower baselines.
- **Numeric kernels, canonicalisation and the FFI crate get adversarial review** plus fuzzing. "Compiles" and "passes tests" are not proof.
- **`unsafe` is forbidden outside `rsympy-py`.**
- **One worker per unit.** Its gate is the unit's dashboard row; `PORTING.md` is the shared rulebook.

## Metrics
- **Conformance:** per module, on SymPy's suite, its doctests and the golden cases.
- **Speed and memory:** against SymPy 1.14 on the benchmark set.
- **Fuzzing:** crashes and parity diffs found per CPU-hour.
- **Build time:** clean and incremental.
- **Cost:** actual cost against the estimate, per unit.
- **Safety:** `unsafe` count outside FFI (target 0).
