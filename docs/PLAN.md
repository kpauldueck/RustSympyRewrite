# Plan: SymPy, rewritten in Rust

**Goal.** Rewrite all of SymPy in Rust: as accurate, faster, smaller in memory, and more secure. Paul's homework app doesn't set the scope; it sets the **order**, so what the app uses is ported first.

**Supporting documents:**
- [research/rust-rewrites.md](research/rust-rewrites.md): lessons from other large rewrites.
- [research/sympy-module-survey.md](research/sympy-module-survey.md): every SymPy module, with its algorithms, real dependencies, traps, difficulty and port strategy.
- `oracle/sympy_inventory.json`: sizes and test counts.

## Decisions

**Settled with Paul, 2026-10-22:**
1. **Rust is the product.** It is a standalone Rust library with a first-class Rust API. Python is a bridge, not a runtime requirement (see `rsympy-py` below).
2. **Permissive licences only** (MIT OR Apache-2.0).
   - Big integers use `dashu`. `malachite` is LGPL-3.0, so it's excluded, as are GMP and FLINT.
   - mpmath is BSD, so we port its algorithms.
3. **Independent of upstream SymPy.** SymPy 1.14.0, pinned, is the answer key. We keep its BSD notice wherever we port code.
4. **Full SymPy scope.** The app's audit sets priority only.

**Settled by this plan (reversible, but everything else assumes them):**

5. **Two behaviour profiles on one core.**
   - **`sympy` (parity).** Reproduces SymPy's observable behaviour: canonical forms, `.args` order, printed strings, Float digits. This is what SymPy's test suite checks and what Python users of `rsympy-py` get.
   - **`strict` (students and services)** is the parity core plus a strict layer:
     - It never silently misreads input: it refuses by name, with opt-in vocabularies.
     - Typed decimals are exact.
     - Units are dimension-checked.
     - Every call has a budget.
     - Equivalence is proved, not guessed (see "Strict layer").

   The strict layer adds front ends, checks, refusals and its own algorithms. It **never changes** `canonicalize`, the orderings or node identity, so an expression means the same thing under both profiles and every parity test also protects strict.

   Parity alone can't deliver these: SymPy reads `f(x)` as `f*x`, `x12` as `12*x` and `\overline{AB}` as `conjugate(A)*conjugate(B)`, and `parse_expr` runs arbitrary Python through `eval`.
6. **Port faithfully, then improve.** About 85% of SymPy's code (by lines) is to be ported algorithm-for-algorithm, because tests across the whole library compare exact output shapes produced by specific heuristics. The exceptions are listed under "Strategy by module".
7. **A non-evaluating core.**
   - Raw constructors build exactly what they're given.
   - SymPy's automatic evaluation is a separate `canonicalize` layer, controlled by an explicit context rather than SymPy's global `evaluate` flag.

   This is the mistake that kept SymEngine from replacing SymPy's core. The app's radical-form checks need raw trees too.

## How big this is

| | |
|---|---|
| SymPy 1.14 non-test source | 497k lines across 70 planning units |
| SymPy test suite | 256k lines, 12.1k `test_` functions, plus test-class methods (e.g. biomechanics has about 215) and about 32k lines of doctests |
| mpmath, ported as `rsympy-mpf` | 47.7k lines |
| Survey difficulty (excluding deferred units) | 10 XL, 25 L, 19 M, 10 S |
| Conventional effort, SymPy units only | **251–671 person-weeks**, roughly 5–13 person-years for a skilled human team |

The bands are S 0.2–1, M 1–4, L 4–13 and XL 13–26 person-weeks per unit. The XL units are core, polys, printing, simplify, integrals, solvers, solvers.ode, functions.special, physics.quantum and tensor.

**Not in that range:**
- mpmath, which is XL on its own;
- `rsympy-py`;
- the strict layer;
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
| Port faithfully | core, assumptions, logic, sets, strategies, polys (+matrices, agca, numberfields), ntheory, functions.*, concrete, series, calculus, simplify, integrals, solvers, solvers.diophantine, matrices (+expressions), printing (+pretty), codegen, combinatorics, stats, geometry, tensor, vector, diffgeom, physics.* (except units), categories, liealgebras, algebras, crypto, discrete | 85% |
| Redesign | multipledispatch → generated match tables; utilities → Rust helpers; polys.domains ground types → one `Int`/`Rational`/`ModInt`; parsing and parsing.latex → a secure closed-grammar parser with parity and strict front ends; physics.units → parity and strict modes; the functions protocol → one `Head` trait; stats.sampling | 7% |
| Keep in Python, on the Rust core via `rsympy-py` | plotting, interactive, parsing.autolev, parsing.c, testing. solvers.ode also stays Python until Tier 7: it leans on integrate, which reaches parity in Tier 6 | 7% |
| Defer or drop | unify, holonomic, parsing.fortran, deprecated matrix mixins (`matrices/common.py`, `matrices/matrices.py` and their tests), benchmarks (rewritten as criterion benches) | 1% |

Paul to confirm the last two rows (open question 7).

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

  It must also allow **foreign heads**, whose methods call back into Python, for user subclasses during the transition.
- **Binary-operator dispatch.** SymPy picks the class that handles `a*b` by `_op_priority`; this matters for matrices, quantities, vectors and `AccumBounds`. A `BinaryOp` trait with priorities is needed from day one.
- **Old assumptions as a u64 bitset `FactKB`.**
  - The 40 rules are compiled at build time, with atomic per-node caches.
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
  - **A budget.** Every loop that can blow up calls `ctx.tick()?`, so per-call budgets work without threads or signals. Retrofitting this later means touching every algorithm.
  - **The RNG,** with the `sympy.core.random` API (`seed`, `_randint`, `_randrange`, `random_complex_number`), so seeded tests replay SymPy's sequences.
  - Everything is deterministic: no hash-order iteration.
- **Floats behave like mpmath.**
  - A Float's identity includes its precision (`Float(0.5) != Float(0.5, 30)`), and the `Float(2.0) != 2` rule from 1.13 holds.
  - `rsympy-mpf` ports mpmath operation by operation at the same working precision, so it should produce the same digits. The differential tests check that rather than assume it.
- **Python numeric semantics where SymPy exposes them:** floor `//` and `%`, `Mod`, and `round` half-even.

## The oracle

There are three layers, built in this order.

1. **Golden cases (exist today).** 1,176 cases in 17 topics, generated by `oracle/gen_cases.py` and `oracle/topics/`, and scored by `crates/rsympy/tests/oracle.rs`. They run in plain `cargo test`, before `rsympy-py` exists.
   - **`source`** records where an answer came from:
     - `sympy`: a SymPy call.
     - `app`: SymPy pieces running a rule of ours or the app's. It is only as right as that rule.
     - `spec`: decided on purpose, with a note saying why.
   - **`profile`** records who must pass the case:
     - `both`;
     - `strict`: spec cases, and ops only the strict layer has;
     - `parity`: SymPy behaviour that strict deliberately changes.
   - **Generation is reproducible.**
     - SymPy, mpmath and lark are pinned.
     - `PYTHONHASHSEED=0` and Python ground types are required.
     - Caches are cleared and the RNG is seeded before each topic.
     - The generator rejects malformed or contradictory cases.
     - CI regenerates the cases and diffs them against what is committed.
   - **Known gap.** The 153 `equivalent`, `set` and `antiderivative` cases are compared by rsympy itself, so a wrong simplifier could grade itself right. The fix is a Tier 0 task: the generator records sample points and SymPy's values at them, and the runner checks against those.
2. **SymPy's own test suite**, run through `rsympy-py`.
   - **Pinned environment:** `PYTHONHASHSEED=0`, Python ground types (no gmpy2/flint), mpmath 1.3.0, pytest and hypothesis.
   - **Extras environment:** adds numpy, scipy, antlr4 4.11, matplotlib, IPython, libclang and llvmlite. Without them the ANTLR LaTeX, autolev, numpy-printer, plotting, interactive, sampling and codegen tests skip silently.
   - **Doctests** (about 32k lines) are scored separately, per docstring.
   - **Markers:**
     - `@XFAIL` tests are scored separately. An XFAIL that starts passing is a difference to explain, not a win.
     - `@slow` tests run nightly, not on every pull request.
     - Skip reasons are recorded.
   - **Test-dependency manifest.** Each SymPy test is run under `sys.setprofile`, recording which SymPy modules it actually executes. Tier exits count only tests whose traced dependencies all sit in that tier or earlier, so no tier waits on a later one.
   - **Scoring** is per test function and test method, per module, on a dashboard.
   - **Project-level suites.** `core/tests/test_args.py` (1,007 functions that instantiate every class in SymPy) and `utilities/tests/test_wester.py` are owned at project level, not counted toward core.
   - **Oracle-unstable outputs.** Some outputs depend on history or hash order:
     - `factor_cache` changes radical extraction;
     - the integrate Piecewise branches depend on hash order;
     - `_ask` shuffles facts.

     These are classified as oracle-unstable rather than counted as failures.
3. **Differential fuzzing.**
   - Random expressions are compared against SymPy on `srepr`, `.args` order, `str` and `latex`. Ordering parity can only be found this way.
   - The strict equivalence engine is checked against interval evaluation.

**Gates (enforced in CI):**
- No pull request lowers any module's pass count.
- Golden cases regenerate identically.
- Every failure is fixed or listed in `oracle/divergences.json` with a `category` and a `reason`. CI rejects malformed entries, unknown case ids, and entries for cases that now pass. The categories are:
  - **`bug`:** rsympy is wrong. Fix it.
  - **`intentional`:** rsympy differs on purpose. Prefer a spec case, with the SymPy case marked `profile: parity`.
  - **`sympy_bug`:** SymPy is wrong. Parity copies SymPy's bugs by default. The entry records where it can't (a crash, a hang, a hash-order output) or where strict fixes the bug.
- Only a human may add a divergence or lower a baseline. Both files get CODEOWNERS review.

**Benchmarks.**
- **Cases:** SymPy's `bench_symbench` R1–R11, about 130 cases from `bench_meijerint`, the polys benchmarks, and the app's own cases.
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
  - A test that needs an internal we won't reproduce is classified, not edited.
- **`parse_expr`.** SymPy's `parse_expr` runs Python `eval`, and the Rust parser never will. For parity, `rsympy-py` keeps SymPy's Python `parse_expr` as the fallback for custom transformations and `local_dict`/`global_dict` tricks.
- **Exceptions and deprecations.** Tests assert exception types (`raises(TypeError)`) and `SymPyDeprecationWarning`s.
  - `PORTING.md` maps each to a Rust error kind.
  - `rsympy-py` raises the matching Python class.

### rsympy-mpf
mpmath is ported in milestones, each landing with the tier that first needs it.

| Milestone | Tier | Contents |
|---|---|---|
| M1 | 1 | libmpf: `mpf` arithmetic and rounding, `to_str`, `prec_to_dps` |
| M2 | 2 | elementary functions, `mpc`, `hypsum`; `mpmath.iv` intervals for the strict equivalence engine (gates App Milestone A) |
| M3 | 3 | `polyroots`, `findroot` |
| M4 | 4 | `pslq`, `identify` (for `nsimplify`) |
| M5 | 5 | `MDNewton` (for `nsolve`), `eig`, `lu_solve` |
| M6 | 6 | `quad`, special-function kernels |

### Strict layer
This lands in Tier 4 and grows through Tier 5.
- **Front ends:**
  - a closed-grammar parser with opt-in vocabularies;
  - exact decimals;
  - quantity input with dimension checks.
- **Equivalence:**
  - Normal forms prove two answers equal.
  - Interval evaluation proves them not equal.
  - Anything else is "unknown", and the item's policy decides. There is no floating-point "probably equal".
- **Form checks:** factored, expanded, simplified fraction, slope-intercept, simplified radical.
- **Algorithms SymPy lacks or gets wrong for students:**
  - periodic equations solved exactly on a bounded interval (e.g. every solution in [0, 2π));
  - domain and singularity analysis;
  - corrections where SymPy's limits are wrong;
  - a trig-identity prover;
  - geometry checks.

## Roadmap

Tiers follow the dependency structure; inside each tier, what the app needs goes first. Units marked ⇄ can run in parallel tracks once their prerequisites land, which is where many agents help.

### Tier 0: Harness
- The golden-case oracle and CI, with profiles, reproducible generation and divergence validation (done, PR #1).
- Sample-point data for the 153 self-graded cases.
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
- **Parsing:** the secure `sympify` parser (parity front end) and the app's number input (`parse_number`, significant figures).
- **Infrastructure:** multipledispatch tables, strategies (the rule combinators that fu, matrices.expressions and others use), class markers, and the core-misc namespace (`sympy/__init__`, `abc`, ...).
- **Sets phase 1:** `Interval`, `FiniteSet`, `Union`, `Intersection`, `Complement`, `Range`.

**Exit:**
- Core, logic, sets-phase-1 and functions.elementary tests are at 95% or more, counting tests whose traced dependencies are in Tiers 0–2.
- The `numbers` and `numeric_input` golden topics pass.
- The `latex_parse` and `printing` topics are under way.

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
- `Matrix` over ZZ/QQ, which stores a `DomainMatrix`.
- The combinatorics permutation kernel, which the matrix tests need.
- `rsympy-mpf` M3.

**Exit:**
- Polys and exact-path matrices tests are at 95% or more, traced to Tiers 0–3.
- gcd and factor are at least 10× faster than pure SymPy on the polys benchmarks.
- The rustsmith GCD trap does not recur.

After Tier 3, **⇄ a geometry rational track** can start: `Point`, `Line`, `Segment` and `Polygon` over rationals.

### Tier 4: Simplify, printing and parsing (serve-path parity)
- **simplify basics:** `ratsimp`, `radsimp`, `logcombine`, `sqrtdenest`, and `nsimplify` (on `rsympy-mpf` M4).
- `fu`/`trigsimp`, then the `simplify()` pipeline.
- **The LaTeX printer,** with an override API because the app subclasses it.
- **LaTeX parsing.**
  - SymPy ships two LaTeX parsers that disagree: ANTLR (the default) and Lark. Parity reproduces both grammars, with node shells for what they return, including the bra-ket slice of physics.quantum.
  - Strict has its own closed grammar.
- The pretty printer.
- **⇄ physics.units,** in parity and strict modes.
- **⇄ tensor.array.**
- **The strict layer's front ends,** its equivalence engine and its form checks.

**App milestones:**
- **A:** the app's serve path runs on rsympy through `rsympy-py`. That covers the expression, factors, inequality, graph_plot and free-response graders, plus a quantity grader.
- **A2:** the app's matrix key checks over exact domains.

**Exit:**
- These golden topics pass: `equivalence` (except the function and inequality cases, which need Tier 5 solving), `forms`, `latex_print`, `units` and `matrix_keys`.
- Equivalence p99 is under 10 ms and below the app's measured SymPy latency.

### Tier 5: Calculus and solving (compile-path parity)
- The series protocol across core and functions, then Gruntz limits.
- **One cycle milestone,** ported together because these call each other: calculus.util, solveset, inequalities, sets phase 2 (`ImageSet`, `ConditionSet`) and diophantine.
- Legacy `solve`, `rsolve`, `nsolve` (on `rsympy-mpf` M5), and concrete summation.
- `Matrix` symbolic (EXRAW) paths and eigen.
- **⇄ logic's SAT solver and the new assumptions (`ask`).**
- **⇄ geometry:** ellipses, circles and planes, which need solvers.
- **⇄ the finite stats slice:** `FiniteRV`, `P`, `E`, `given`.
- **The strict layer's algorithms:** bounded-interval periodic solving, domain and singularity analysis, and limit corrections.

**App Milestone B:** figure lint and key checks run on rsympy.

**Exit:** these golden topics pass:
- the `equivalence` function and inequality cases;
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

**Exit:** integrals and series are at 90% or more. Known SymPy blowups get budgets.

### Tier 7: Domains and the long tail
Mostly parallel tracks (⇄), each gated by its upstream units:
- combinatorics beyond the permutation kernel; BSGS and `tensor_can` are needed by tensor.
- stats (discrete, then continuous).
- solvers.ode, ported hint by hint. Until now it ran as Python on the Rust core.
- matrices.expressions.
- tensor.
- physics.vector, mechanics and biomechanics.
- sympy.vector, diffgeom, control, continuum_mechanics, optics.
- physics.quantum (its LaTeX-parser slice came in Tier 4).
- secondquant, last, because its ordering keys on printed strings.
- codegen and code printers, with byte-identical output so Python-side `lambdify` keeps working.
- crypto, discrete, liealgebras, categories, algebras.
- The advanced parts of ntheory, numberfields and agca.

**Exit:** full-suite conformance of 98% or more, with every remaining failure classified.

## Working rules (humans and agents)
- **Port faithfully; don't clean up during a port.** Keep each mistranslation findable, as fish did.
- **Port SymPy's real algorithms,** including its heuristics. Output shapes are the spec.
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

## Open questions for Paul
Some of these are already pinned by `spec` golden cases, so flipping one means flipping its cases:
- Q1: `latex_parse-105` and `latex_parse-106`;
- Q4: `equivalence-077` to `080` and `equivalence-092`;
- Q5a and Q5b: `units-036`, `039`, `058`, `096` and `104`.

Q2, Q3, Q5c and Q6 aren't pinned yet.

1. **Strict profile: `\log x` means base 10**, as AP Precalculus does. The app reads it as natural log today. The parity profile keeps SymPy's natural log.
2. **Strict profile: function notation.** Is `f(x)` a function application, given that `x(x+1)` is multiplication? Possible rules:
   - `f`/`g`/`h` before parentheses are functions;
   - a `functions` vocabulary;
   - refusal.
3. **Strict profile: cube roots of negatives.** Should `\sqrt[3]{-8}` be `-2` (the real root) rather than SymPy's principal root?
4. **Strict profile: equations equivalent up to a constant factor.** The cases currently say yes; the app accepts sign changes only today.
5. **Strict profile: units.**
   - a. A bare number against a unit key is `invalid:missing_unit`.
   - b. `20 C` reads as coulombs.
   - c. Is an unparenthesised `J/kg·°C` refused as ambiguous? (`J/(kg·°C)` is accepted: `units-032`.)
6. **Strict profile: chained inequalities** (`-1 < x < 3`): parse to `And(...)`, or refuse?
7. **What stays in Python or is dropped.**
   - plotting, interactive, parsing.autolev, parsing.c and the testing package stay Python, running on the Rust core.
   - solvers.ode stays Python until Tier 7.
   - unify, holonomic, parsing.fortran, the deprecated matrix mixins and SymPy's benchmarks directory are deferred or dropped.

   Is anything there something you need in Rust?
