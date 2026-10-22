# Plan: SymPy in Rust

**Goal.** Match SymPy's accuracy while being faster, using less memory and being more secure. The evidence behind these choices is in [research/rust-rewrites.md](research/rust-rewrites.md).

## What we're up against

These numbers come from measuring SymPy 1.14:

| | |
|---|---|
| Non-test source | ~497k lines of Python |
| Tests | ~256k lines, ~12.1k `def test_` functions |
| Biggest modules (src LOC) | polys 77k, physics 62k, core 35k, functions 29k, solvers 29k, matrices 25k, printing 24k |
| Modules that import mpmath (outside tests) | 55 |
| `sympify`/`parse_expr` | ends in `eval()` (`parsing/sympy_parser.py`), a known security hole |

A big-bang port of 500k lines with SymPy's quirks is how you end up at "84% for $400k". The plan below is designed to avoid that.

## Strategy in one paragraph

We build a Rust core (expressions, numbers, polys, evalf) behind a PyO3 module that presents SymPy's Python API. **SymPy itself is the oracle.** We pin a SymPy commit and run its own test suite against our shim. Modules we haven't ported yet keep running as *SymPy's Python code on top of our core*, so there is a working, testable product at every step, the way fish did it. Rust takes the compute; glue and UI (plotting, interactive, most of physics) stay Python until there's a measured reason to move them. The core gets a **non-evaluating** representation with SymPy's auto-evaluation layered on top as a compatibility mode. That is the design mistake that sank SymEngine as a replacement, and we don't port it.

## Key decisions

1. **Compatibility target.** The SymPy Python API is the contract and the test suite is the spec. We don't promise bug-for-bug compatibility. Every diff gets one of three labels:
   - **bug**: ours, fix it.
   - **intentional**: documented in `DIVERGENCES.md`.
   - **upstream bug**: recorded, matched by default, and fixed later behind a flag.

   This is uv's model.
2. **Expression representation.** Hash-consed, immutable DAG nodes behind `Arc`, with a precomputed hash and interning through a concurrent weak table. Equality is then pointer equality, and memory is shared across subexpressions.
   - There are no cycles, so Rust ownership is fine. TypeScript's reason for picking Go doesn't apply to an immutable expression tree.
   - Symbols come from an append-only interned table (Symbolica-style) and carry their assumptions.
   - Add and Mul use flat, sorted, packed coefficient/term vectors.
   - Small integers and rationals are stored inline; only big ones point to a bigint.
3. **Evaluation.**
   - Raw constructors never evaluate.
   - `canonicalize` reproduces SymPy's `Add.flatten`/`Mul.flatten`/`Pow.__new__` rules, and the Python shim calls it by default (`evaluate=True`).
   - Every evaluation step has a work budget, because SymPy constructors can do unbounded work.
4. **Numbers.**
   - We define our own `Integer`: an i64 fast path plus a bigint behind a trait. Without the fast path we lose to CPython, as rustsmith measured.
   - The default backend is a permissively licensed pure-Rust bigint, either `dashu` or `ibig`, chosen by benchmark.
   - A `gmp`/`flint` cargo feature gives the fastest build (LGPL).
   - Arbitrary-precision floats replace mpmath for evalf: `astro-float` or our own, with FLINT/Arb behind the same feature.
   - We do not touch `num-bigint`.
5. **Polys.** We port SymPy's *actual* algorithms: dense DUP/DMP, sparse `PolyElement`, heuristic GCD plus subresultant PRS, Zassenhaus and Wang factoring, Gröbner bases, and `DomainMatrix`. Under the `flint` feature, whole layers delegate to FLINT, as SymPy already does via python-flint.
6. **Python extensibility.** Users subclass `Function` and `Expr` in Python, and SymPy's own long tail does too.
   - A `PyNode` variant wraps a Python object and calls its `eval`, `_eval_*`, `fdiff` and printing hooks through the boundary.
   - This is slow but correct, and it's what lets unported modules run on the new core.
7. **One extension module.** All Rust crates link into a single `.so`. This avoids Symbolica's per-module global-state problem.
   - Builds use free-threaded CPython 3.14t from day one.
   - Algorithms run entirely on one side of the boundary, never chatty per-node calls.
8. **Security.**
   - The parser is a real parser with no `eval`, `exec` or attribute access.
   - Resource limits cover expression size, recursion depth and bigint bits.
   - `#![forbid(unsafe_code)]` applies everywhere except the FFI crate.
   - We run `cargo-fuzz` on the parser and evaluators, `cargo-deny` and `cargo-vet` on dependencies, and Miri in CI.

## Repo layout

```
crates/
  sympy-numbers/   Integer, Rational, Float, GF(p); bigint backends behind traits
  sympy-core/      Expr DAG, interner, canonicalize, assumptions, subs/xreplace, diff, expand
  sympy-polys/     domains, DUP/DMP, sparse rings, gcd, factor, groebner, DomainMatrix
  sympy-evalf/     arbitrary-precision evaluation (mpmath replacement subset)
  sympy-parse/     sympify / parse_expr, no eval
  sympy-py/        PyO3 module: the only crate with unsafe; SymPy-compatible classes
oracle/            pinned SymPy, test runner against the shim, differential fuzzer, dashboard
bench/             sympy_benchmarks (asv) + memory benchmarks + cross-CAS comparisons
PORTING.md         mapping rules for humans and agents
DIVERGENCES.md     every intentional difference from SymPy
```

## The oracle (built first, before any port)

- **Pinned upstream.** SymPy is a git submodule pinned to one commit. We rebase quarterly, and the dashboard tracks the diff.
- **Test suite against the shim.**
  - We run SymPy's own pytest suite with `import sympy` redirected to our package, whose unported pieces fall back to SymPy's Python.
  - Per module, we track pass, fail, xfail, and diff-by-category.
  - **Gate:** no PR may lower any module's pass count.
- **Differential fuzzing.**
  - Generate random expressions, run them through SymPy and through us, and compare `srepr` after canonicalization.
  - For operations where structure legitimately differs (e.g. `simplify`), compare numeric values at random high-precision points instead.
  - Fuzzing finds what 12k hand-written tests miss.
- **Printing parity.** SymPy tests compare `str()` output, so the printer must reproduce SymPy's `sort_key` ordering independently of our internal order.

## Phases

The exit criteria are the contract; durations are rough.

### Phase 0: Harness (2–4 weeks)
- Cargo workspace, maturin build, CI, `forbid(unsafe)`, `cargo-deny` and fuzz targets.
- Oracle runner and per-module dashboard, starting with SymPy run against itself as the baseline.
- Benchmarks:
  - `sympy_benchmarks` (asv), plus RSS and allocation tracking.
  - The same workloads on SymEngine and python-flint as ceilings to aim at.
- `PORTING.md` mapping rules covering:
  - Python class → Rust type
  - `_eval_*` dispatch → trait or enum match
  - `@cacheit` → interning or memo
  - The forbidden patterns
- **Exit:** a Rust "hello expr" is importable, and the oracle reports a baseline for every module.

### Phase 1: Numbers and domains
- `Integer`, `Rational`, `GF(p)`, bigint backends, and `ntheory` basics (gcd, primality, factorint).
- These ship as a new SymPy **ground type**. `SYMPY_GROUND_TYPES` already switches between python, gmpy and flint, so adding `rust` is a tiny integration.
- This delivers value to stock SymPy users immediately.
- **Exit:**
  - `polys/domains` and `ntheory` tests pass with the `rust` ground type.
  - Small-int operations are at least as fast as gmpy2.

### Phase 2: Polys and DomainMatrix
- The 77k-line module and SymPy's best-tested numeric core (~1.4k tests).
- Plugs in under SymPy's existing `DMP` and `DomainMatrix` classes, the same seam python-flint uses.
- **Exit:**
  - The polys and matrices domain tests pass.
  - gcd, factor and groebner are within 2× of FLINT, and at least 10× faster than pure SymPy.
  - No regression against rustsmith's gcd trap.

### Phase 3: Core expressions
- `Basic`, `Expr`, `Symbol`, `Add`, `Mul`, `Pow`, `Number`, and the old assumptions system (`FactKB` and the `_eval_is_*` inference).
- Also: `subs`, `xreplace`, `expand`, `diff`, `as_coeff_*`, `sort_key`, the `str` and `srepr` printers, and the no-`eval` `sympify`.
- `PyNode` bridging for Python subclasses.
- This is the hard phase. Auto-evaluation semantics and assumptions are where SymPy's behaviour actually lives.
- **Exit:**
  - Core tests are at 98% or more, with every remaining diff classified.
  - A random-expression fuzz run has zero unclassified diffs.
  - `expand` and `subs` microbenchmarks are at least 10× faster, and memory per node is at least 3× lower.

### Phase 4: Functions and evalf
- Elementary functions, then special functions, plus evalf with correct rounding (Arb-style ball arithmetic where SymPy relies on mpmath's heuristics).
- `functions` has ~565 tests; evalf correctness gets its own fuzz oracle against mpmath at higher precision.
- **Exit:** the functions and evalf tests pass, and mpmath is no longer imported on the core path.

### Phase 5: The algorithms people actually use
Order: `matrices` → `series` → `simplify` → `solvers` (solve, solveset, ode) → `integrals` (risch, heurisch, meijerg, manual) → `sets`, `logic`, `assumptions`.
- Port each module **faithfully first**, with no cleanup during the port (fish's rule), so the oracle can localize every mistranslation.
- Leave `simplify` until last. It spans nine modules and was the worst-rated port target.
- A new e-graph/egglog `simplify` is a separate research track with its own oracle (numeric equivalence plus "result no longer than SymPy's"). It is not part of parity.
- **Exit:** per-module pass rates at least 98%, and the top-20 asv benchmarks at least 10× faster.

### Phase 6: The long tail stays Python
`physics`, `stats`, `geometry`, `combinatorics`, `plotting`, `printing` (LaTeX, MathML, code), `codegen`, `parsing` (LaTeX and Mathematica), `vector`, `tensor`.
- These run unmodified on the Rust core via the shim.
- We port pieces only when a profile says so.
- `physics.mechanics` is the obvious first candidate, since it is the one that already pushed SymPy toward SymEngine.

## Using AI agents

This is where the research is most concrete:

- **Setup**
  - Write `PORTING.md` before agents start. tsc-rs had one; the $400k attempt apparently didn't.
  - Use one worker per module, with the oracle gate as the only definition of done.
  - Workers must not edit tests or the pinned upstream.
- **Port from typed intermediates.** tsc-rs worked partly because it ported from Go rather than JS. For gnarly Python (core, assumptions), consider having agents first produce type-annotated, `mypy --strict`-clean Python, checked by the oracle, and port *that*.
- **Review.** "Compiles" and "passes tests" are not enough: Copilot shipped regressions that compiled, and Bun had unsound functions that passed. Numeric kernels, canonicalization and the FFI crate get adversarial review: a second agent or a human trying to break them, with fuzz cases committed.
- **Budget.** Expect most agent time to go to investigation rather than generation, and size budgets accordingly.

## Success metrics, tracked on the dashboard
- **Accuracy:** per-module SymPy test pass %, fuzz diffs (target: 0 unclassified), and evalf rounding errors (target: 0).
- **Speed:** geometric mean over asv against SymPy, SymEngine, and python-flint ceilings.
- **Memory:** bytes per node, plus peak RSS on benchmark workloads.
- **Security:** `unsafe` count outside FFI (target: 0), fuzz hours without crashes, and no `eval` anywhere in the codebase.

## Biggest risks
1. **Auto-evaluation and assumptions semantics.** There are thousands of implicit rules. Mitigation: canonicalize as a separate, oracle-tested layer, plus fuzzing.
2. **Python-subclass extensibility.** If `PyNode` is slow or leaky, the long-tail-stays-Python strategy collapses. Prototype it in Phase 0, not Phase 3.
3. **mpmath replacement.** Accuracy here is subtle. Keep mpmath as the oracle for longer than feels necessary.
4. **Upstream drift.** SymPy keeps moving, including Benjamin's own non-evaluating-core work. Pin, rebase quarterly, and watch SymPy's own redesign: if upstream ships a non-evaluating core, align with its API rather than inventing a parallel one.
5. **Licensing.** The default build stays BSD-compatible, and GMP/FLINT stay behind a feature. Never copy Symbolica code.

## Open questions for Paul
- **Who it's for.** Is the goal a drop-in accelerator for SymPy users (this plan), or a standalone Rust CAS with Python bindings, where the Rust API is first-class?
- **Licensing.** Is an LGPL feature (GMP/FLINT) acceptable, or must every build be permissive?
- **Upstream.** Should we coordinate with upstream SymPy (Oscar Benjamin et al.) or stay an independent fork? Upstreaming Phase 1 as a ground type is cheap goodwill.
