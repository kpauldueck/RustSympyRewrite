# Plan: a Rust CAS for high-school to early-college math

**Goal.** A pure-Rust computer algebra library that a Rust app (Paul's homework app first) can use directly, with no Python at runtime. It should be as accurate as SymPy on its scope, faster, lighter and safer. The evidence behind these choices is in [research/rust-rewrites.md](research/rust-rewrites.md).

## Decisions (settled 2026-10-22)

1. **Standalone Rust library, not a SymPy accelerator.** The Rust API is the product. SymPy is only used at development time, as the answer key.
2. **Permissive licences only.** The project is MIT OR Apache-2.0. Dependencies must be permissive, so no GMP or FLINT and nothing LGPL. Homework-sized numbers don't need GMP.
3. **Independent of upstream SymPy.** We borrow SymPy's algorithms and test cases (SymPy is BSD-licensed, so keep its copyright notice wherever we port code), but we don't coordinate releases with it.
4. **Scope is high school to early college.** See below. Anything outside it gets ported only when the app needs it.

## Scope

**In scope**

| Area | Concretely |
|---|---|
| Numbers | exact integers and rationals, radicals (`sqrt(8)` becomes `2*sqrt(2)`), π, e, i, complex arithmetic, decimals at any precision |
| Algebra | expand, factor over ℚ, gcd/lcm, cancel, together, partial fractions (`apart`), collect, substitution |
| Equations | linear, quadratic, polynomial roots (rational roots, quadratic and cubic formulas, numeric fallback), rational and radical equations, simple exp/log/trig equations with general solutions, inequalities and intervals, linear systems |
| Functions | trig and inverse trig, exp/log, abs, floor/ceil, piecewise, with their textbook identities |
| Simplify | `simplify` plus the targeted tools: trigsimp, radsimp/rationalize, logcombine and expand_log, powsimp |
| Calculus | limits (including at ∞ and one-sided), derivatives (chain, product, implicit), integrals (the textbook methods: substitution, parts, partial fractions, trig substitution; definite and improper), Taylor series, finite sums and common infinite series |
| Linear algebra | matrices over ℚ and symbolic entries, det, inverse, rref, rank, solving systems, eigenvalues for small matrices |
| I/O | parse plain text (`x^2 + 2x`) and LaTeX input; print plain text and LaTeX |

**Stretch goals for early college:** first-order and linear constant-coefficient ODEs, basic probability and statistics, sequences and recurrences.

**Out of scope:** physics, tensors, Gröbner bases, the Risch algorithm, Meijer-G integration, special functions beyond the basics, code generation, plotting.

The cut is roughly 15–20% of SymPy. That is months of work, not years.

**Steps** (showing work like a textbook) are likely what a homework app wants. SymPy's `manualintegrate` already works as textbook rules, so we port integration that way from the start. That leaves the door open to step-by-step output without committing to it yet.

## Architecture

```
crates/
  rsympy-num/    Integer (i64 fast path, big fallback), Rational, exact gcd/factorint, arbitrary-precision Float
  rsympy/        Expr, canonicalisation, assumptions, parse, print, and the math modules (later split as it grows)
oracle/          Python dev tool: runs SymPy, writes golden test cases (JSON)
```

- **Expressions.** Immutable, hash-consed nodes behind `Arc`. Identical subexpressions are stored once and compare in O(1). Expression trees have no cycles, so Rust's ownership rules aren't a fight here.
  - Small integers and rationals are stored inline.
  - `Add` and `Mul` hold flat, sorted terms.
- **Construction vs. canonicalisation.**
  - Raw constructors build exactly the tree you asked for. This matters for showing steps (`x + x` must be displayable before it becomes `2*x`).
  - `canonical()` applies SymPy's automatic rules. The friendly API (`x + x` via operator overloading) is canonical by default.
  - This keeps SymPy's familiar results while avoiding the design flaw that made SymEngine too rigid.
- **Assumptions.** Symbols carry flags such as real, positive, integer and nonzero, and a small inference engine combines them. SymPy's homework answers depend on these, e.g. `sqrt(x**2)` becomes `|x|` unless `x` is positive.
- **Big numbers.** A pure-Rust bigint, `dashu` or `ibig`, chosen by benchmark, behind our own `Integer` type. Never `num-bigint`, which is very slow on large values.
- **Safety.**
  - `#![forbid(unsafe_code)]` everywhere.
  - Real parsers with no `eval`, unlike SymPy's `sympify`.
  - Limits on expression size, recursion depth and bigint size, so a malicious or typo'd input can't hang the app.
  - Fuzz the parsers.

## The oracle: how we know answers are right

- **Golden cases.** `oracle/gen_cases.py` runs a pinned SymPy (1.14.0) on a corpus of problems and writes `oracle/cases/*.json`. Each case holds the operation, the input, SymPy's answer as text, and how to compare. Rust tests load these files, so Rust CI never needs Python.
- **Comparison modes.**
  - `exact`: canonical text must match. Use it where the answer form is the point, as with expand, factor and diff.
  - `equivalent`: same value at random points plus simplification to zero. Use it for simplify, where several forms are correct.
  - `set`: the solution set matches, in any order.
  - `antiderivative`: the derivative of our answer equals the integrand. Integrals may differ by a constant.
- **Sources of cases.**
  1. Hand-picked textbook problems by topic.
  2. The relevant tests extracted from SymPy's own test suite (`core`, `polys`, `simplify`, `solvers`, `calculus`, `integrals/manualintegrate`, `series`, `matrices`, `printing`).
  3. Randomly generated expressions (differential fuzzing).
  4. Problems from Paul's app.
- **Conformance score.** `cargo test` reports pass counts per topic. **Rule:** no change may lower a topic's score.

## Phases

Each phase ends with something usable, plus an exit test.

### Phase 0: Harness (this PR)
- Cargo workspace and CI (fmt, clippy, tests).
- `forbid(unsafe)` on every crate.
- Oracle generator and first ~100 golden cases.
- A Rust conformance runner that reports the score.

**Exit:** CI is green and the runner reports 0/N.

### Phase 1: Numbers
- `Integer` and `Rational` with an i64 fast path.
- gcd, lcm and integer factoring.
- Exact rational arithmetic.
- Arbitrary-precision decimal evaluation.

**Exit:** numbers cases pass, and small-int operations don't allocate.

### Phase 2: Core expressions and I/O
- Symbols with assumptions, `Add`/`Mul`/`Pow`, constants, and elementary functions as nodes.
- `canonical()`, `subs`, numeric `evalf`.
- Plain-text and LaTeX parsing and printing.

**Exit:** arithmetic, printing and parsing cases pass, and `expand` beats SymPy by at least 10×.

### Phase 3: Polynomials
- Univariate and small multivariate polynomials over ℚ.
- expand, factor, gcd, cancel, apart.
- Roots, porting SymPy's real algorithms: subresultant PRS, Zassenhaus factoring.

**Exit:** algebra cases pass.

### Phase 4: Calculus
- diff.
- Limits: series plus a ported Gruntz algorithm.
- `manualintegrate`-style rule-based integration.
- Taylor series and sums.

**Exit:** calculus cases pass.

### Phase 5: Solving and matrices
- `solveset`-style solvers for the equation classes in scope, plus inequalities and intervals.
- Matrices.

**Exit:** equations and linear-algebra cases pass.

### Phase 6: Simplify
- trigsimp, radsimp, logcombine and `simplify` heuristics.
- This comes last because SymPy's simplify depends on everything else.

**Exit:** simplify cases pass, judged by `equivalent` and "not longer than SymPy's answer".

### Milestone: port Paul's app
After Phase 3 at the latest, start moving the app over, using whatever exists. The app's real calls become oracle cases and set the priorities.

## Working rules (for humans and agents)

- **Port faithfully first and improve later.** When a result differs from SymPy, classify it as one of:
  - our bug, so fix it;
  - intentional, so record it in `DIVERGENCES.md`;
  - a SymPy bug, so record it.
- **Port SymPy's actual algorithms**, not textbook ones. A naive GCD made one earlier Rust port slower than SymPy.
- **Agents must not edit golden cases** to make tests pass. Only the generator writes them.
- **Review numeric code adversarially.** Passing tests is not proof.

## Metrics
- Conformance per topic (target 100% in scope, with every divergence documented).
- Speed versus SymPy on the same cases.
- Peak memory.
- Fuzz crashes (target 0).
- `unsafe` count (target 0).
