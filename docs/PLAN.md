# Plan: a Rust CAS to replace SymPy in Paul's homework app

**Goal.** A pure-Rust computer algebra library that can replace SymPy in Paul's homework app, so the app can move to Rust. On the app's workload it must be as accurate as SymPy, faster, lighter, and safer. Research behind the approach is in [research/rust-rewrites.md](research/rust-rewrites.md).

**What sets the scope.** The app's own SymPy usage, from an audit Paul ran on 2026-10-22, not SymPy's module list. That audit splits the work into two paths with different requirements:

- **Serve path.** This runs inside every student submit, so latency and robustness matter most.
  - Parse student LaTeX.
  - Decide whether it's equivalent to the key.
  - Check the required form (factored, expanded, simplified fraction, slope-intercept, simplified radical).
  - Check exact interval endpoints.
  - Evaluate at lattice points.
  - Print keys as LaTeX.
  - Parse numbers and physical quantities.
- **Compile path.** This runs once per content variant in CI, so breadth and exactness matter most.
  - Figure lint: roots, singularities, end behaviour, exact points on curves.
  - Interval and solveset checks of keys.
  - Sequence sums.
  - Matrix keys.
  - Geometry figure construction.
  - Probability keys.

## Decisions

**Settled 2026-10-22:**
1. **A standalone Rust library.** SymPy is only the development-time answer key.
2. **Permissive licences only.** MIT OR Apache-2.0, with no GMP, FLINT or other LGPL dependencies.
3. **Independent of upstream SymPy.** We borrow its algorithms and test cases and keep its BSD notice where we port code.
4. **Scope is high school to early college,** prioritised by the app.

**New, from the audit** (these are encoded as `spec` golden cases; see `oracle/caselib.py`):

5. **Never silently misread input.** The LaTeX parser maps each construct it knows to a named meaning and refuses everything else by name: `refused:\overline`, `refused:derivative`, and so on. Today SymPy reads `\frac{dy}{dx}` as `y/x`, `\Delta x` as `Delta*x`, and `\overline{AB}` as `conjugate(A)*conjugate(B)`.
6. **Vocabularies are opt-in per item.**
   - Geometry: `\overline{AB}` → `seg_AB`, `\angle ABC` → `angle_ABC`.
   - Vectors: `\vec{v}` → `vec_v`.
   - Sequences: `a_{n-1}` → `a(n - 1)`.

   Some readings are always on:
   - `\Delta x` → `Delta_x`.
   - Letter and digit subscripts become part of the name (`v_{0x}` → `v_0x`).
   - `30^\circ` → `pi/6`.
7. **Typed decimals are exact.** `0.75` is `3/4`, not a float. This removes the app's Float→Rational workarounds and makes `400e^{\ln(0.75)t}` ≡ `400(0.75)^t` decidable. How many significant figures were typed is reported separately (`sig_figs`).
8. **Raw trees vs canonical trees.**
   - The parser can return the tree exactly as typed. Form checks need that: `sqrt(50)` must not quietly become `5*sqrt(2)` before we check whether the radical is simplified.
   - `canonical()` applies SymPy's automatic rules.
9. **Equivalence is a decision procedure, not `simplify(a - b) == 0`.** See "Equivalence engine" below. Item options control it:
   - per-item assumptions (`x > 0`);
   - function equivalence, where removable holes count;
   - equations equivalent up to a constant factor;
   - inequalities equivalent by solution set.
10. **Every call has a budget.** Every operation is bounded in expression size, recursion depth, bigint size and step count, and returns an error instead of hanging. That replaces the app's 5-second SIGKILL pool, which is off by default.

## Architecture

```
crates/
  rsympy-num/    Integer (inline i64, big fallback), Rational, exact decimals, intervals/balls for numeric probing
  rsympy/        expressions, parser, printer, equivalence, forms, polys, solvers, matrices, units, geometry
  rsympy-py/     (planned, optional) PyO3 bindings so the Python app can migrate one call site at a time
oracle/          dev-only: SymPy 1.14.0 + Lark generate golden cases; topics in oracle/topics/
```

- **Expressions.**
  - Immutable, hash-consed nodes behind `Arc`. Identical subexpressions are stored once and compare in O(1).
  - Small numbers are stored inline. `Add` and `Mul` are flat and sorted.
  - Symbols carry assumptions: real, positive, integer, nonzero.
- **Parser.**
  - A hand-written recursive-descent LaTeX parser over a closed grammar, plus a plain-text number and quantity parser.
  - No `eval`, no fall-through to implicit multiplication of unknown commands, and every refusal names what it refused.
- **Equivalence engine** (the serve path's hot spot), in four steps:
  1. **Canonical match.** If the canonical forms match, the answer is true.
  2. **Rational normal form.** Expressions in ℚ(symbols) are reduced to a normal form: cancelled numerator/denominator, expanded, with ℚ coefficients. This decides equivalence exactly for every polynomial or rational-function answer.
  3. **Numeric probing.** For radicals, exp, log and trig, evaluate both sides at several random points inside the shared domain, using interval/ball arithmetic at increasing precision.
     - A certified difference anywhere means **not equivalent**, with certainty.
     - Agreement everywhere means **equivalent**, with a stated confidence.

     WeBWorK grades formulas the same way, by random-point evaluation.
  4. **Targeted rewriting** (log/exp rules under assumptions, trig rewritten as exponentials) upgrades a probed "equivalent" to a proof where it can.

  Two options sit on top:
  - With `function: true`, the domains (singularities) must also match.
  - Non-finite values (`oo`, `zoo`, `nan`) are compared structurally.
- **Python bridge (optional, planned).** The app is Python today. A thin PyO3 crate lets it swap SymPy calls for rsympy one at a time, parser first, while still in Python. It's the same ship-of-Theseus approach that worked for fish and pydantic. The final Rust app doesn't need it.

## The oracle

- **Golden cases.** `oracle/gen_cases.py` plus the topic modules in `oracle/topics/` write `oracle/cases/<topic>.json`, and `crates/rsympy/tests/oracle.rs` scores rsympy per topic.
  - CI regenerates the cases and fails on any diff, so nobody hand-edits the answer key.
  - Any drop in a topic's score fails the build (`oracle/baseline.json`).
  - Wrong answers fail unless the case is listed in `oracle/divergences.json`.
- **Two kinds of expected answer.**
  - `source: sympy`: SymPy computed the answer.
  - `source: spec`: we chose the answer on purpose, because SymPy is wrong for students, lacks the operation, or the app needs different behaviour. Every spec case records why (`note`) and what SymPy does instead (`sympy_gives`).
- **Compare modes:** `exact`, `equivalent`, `set`, `antiderivative`, `error`.
- **Still to add:**
  - Cases harvested from the app's own grader tests and content keys.
  - Random-expression differential fuzzing for the equivalence engine.

## Phases

The order follows the app: serve path first, compile path second, everything else later. Each phase has an exit test in the oracle.

### Phase 0: Harness (PR #1)
- Workspace, CI and the oracle.
- General topics, plus app-driven topics: `latex_parse`, `latex_print`, `equivalence`, `forms`, `numeric_input`, `units`, `toolkit`, `matrix_keys`, `geometry`, `probability`.

### Phase 1: Numbers and number input
- `Integer` and `Rational` with an i64 fast path.
- gcd, lcm, factorint.
- Exact decimals and significant figures.
- `parse_number`: scientific notation in every spelling, percent, thousands separators, `\frac`, `\sqrt`, `2\pi`.

This is enough to port the app's `numeric` grader right away.

**Exit:** `numbers` and `numeric_input` pass.

### Phase 2: Expressions, parser and printer
- Raw and canonical trees, assumptions, `subs`, `free_symbols`, `N`.
- The LaTeX parser with vocabularies.
- The LaTeX printer with the course conventions (`\ln`, `\log_b a`).

**Exit:** `latex_parse`, `latex_print` and `printing` pass. Parse throughput is at least 10× SymPy+Lark.

### Phase 3: Polynomials, equivalence and forms (completes the serve path)
- Multivariate polynomials and rational functions over ℚ: gcd, cancel, factor, expand. Port SymPy's real algorithms (subresultant PRS, Zassenhaus); a naive GCD made one earlier Rust port slower than SymPy.
- The equivalence engine.
- Form checks.
- Exact-endpoint vocabulary.

**Exit:** `algebra`, `equivalence` and `forms` pass. The 99th-percentile equivalence check stays under 10 ms on app-sized inputs.

**Milestone A:** the app's `expression`, `expression_factors`, `inequality_1d/2d`, `graph_plot` and `free_response` deterministic graders can run on rsympy.

### Phase 4: Units and quantities
- A closed unit vocabulary.
- Dimensions and conversions.
- `parse_quantity`.
- Tolerance and instrument-resolution comparisons.

Phase 4 is small and independent, so it can run alongside Phase 3 once Phase 2 lands.

**Exit:** `units` passes.

### Phase 5: Compile-path toolkit
- Real roots (exact isolation plus `nroots`), singularities, continuous domain.
- `solveset` over intervals, including trig equations and inequalities on [0, 2π), plus interval algebra.
- Sums and `rsolve` for arithmetic and geometric sequences.
- Limits for end behaviour.
- `diff`.
- Trig identity rewriting (the `fu`/TR rules).
- Matrices: det, inv, powers, eigenvectors and Markov steady states.

**Exit:** `toolkit`, `matrix_keys`, `equations` and `matrices` pass.

### Phase 6: Geometry and probability
- Exact `Point`, `Line`, `Segment`, `Ray`, `Circle`, `Triangle`, `Polygon`.
- Intersections, distances, angles, triangle centres, transformations, similarity.
- Finite probability (`P`, `given`, `E`) and counting.

**Exit:** `geometry` and `probability` pass.

**Milestone B:** the content gate's figure lint and key checks run on rsympy.

### Later, when content needs it
Integration (rule-based, in the style of `manualintegrate`, which also yields steps), Taylor series, general `simplify`, first-order ODEs. The `calculus` and `simplify` topics already track these.

## Working rules (humans and agents)

- **Port SymPy's algorithms faithfully;** improve them only behind a passing oracle.
- **Classify every disagreement with SymPy** as one of:
  - our bug, so fix it;
  - intentional, so make it a spec case or add it to `divergences.json`;
  - a SymPy bug, so record it.
- **Agents never edit golden JSON.** Only the generators write it.
- **Numeric and parser code gets adversarial review,** plus fuzzing. Passing tests is not proof.
- **No `unsafe`** (it is forbidden workspace-wide).

## Metrics
- **Conformance:** per-topic oracle scores (target 100% in scope, with every divergence documented).
- **Speed:** p50 and p99 serve-path latency versus SymPy on the app's own cases.
- **Memory:** peak memory.
- **Fuzzing:** crash count (target 0).

## Open questions for Paul

The golden cases already take a position on each of these. Where a case pins the current choice, it is marked `spec`; flipping a choice means flipping its cases.

1. **`\log x` means base 10.** That's the AP Precalculus convention, and it is what the cases assume today. The app currently reads it as natural log, so existing keys written with `\log` would change meaning.
2. **Function notation.** `x(x+1)` is read as multiplication, so `f(x)` would become `f*x`, which is a silent misread. Options:
   - treat `f`, `g` and `h` before parentheses as functions;
   - add a `functions` vocabulary;
   - refuse it.

   No case is pinned yet.
3. **Cube roots of negatives.** `\sqrt[3]{-8}` is `2*(-1)**(1/3)` in SymPy (the principal root), but students expect `-2`. We need a real-root rule. No case is pinned.
4. **Equations up to a constant factor.** Should `2x + 4 = 0` match `x + 2 = 0`? The cases currently say yes; the app accepts sign changes only today.
5. **Units.**
   - A bare number against a unit key is `invalid:missing_unit` (a free retry), per the audit.
   - `20 C` reads as coulombs, not Celsius.
   - Is `J/kg·°C` refused as ambiguous?
6. **Chained inequalities** (`-1 < x < 3`): parse them to `And(...)`, or refuse? `inequality_1d` keys need an answer.
7. **Python bridge.** Would you use it to migrate the app gradually, or port the app in one go after Milestone A?
