# Prior art: large rewrites into Rust (and friends)

Research snapshot, 2026-10-22. Every claim has a source; figures marked *(self-reported)* come from the project's own authors and nobody has audited them. The "Unverified" section at the end lists what we could not confirm.

## 1. AI-driven ports, 2025–2026

### tsc-rs (Theo Browne / Ping Labs): the "Opus 5.5 TypeScript-in-Rust" port
- **What it is.** A Rust port of Microsoft's **Go** TypeScript 7 compiler, type checker, LSP and API, *not* of the original JS compiler. MIT licence; npm 0.1.0 shipped 2026-10-07.
  - Sources: https://github.com/pingdotgg/ts-rust, https://www.theregister.com/devops/2026/10/15/24000-ai-experiment-ports-typescript-compiler-to-rust/5303409, https://thevibelog.dev/blog/tsc-rs-typescript-compiler-rust-2026
- **Oracle.** tsgo, pinned to a single upstream revision.
  - The README says "All 184,773 ported Go tests pass". Coverage elsewhere quotes 181,711; we could not reconcile the two.
  - Output was also diffed against tsgo on 120 real OSS repos.
- **Harness.** `PORTING.md` + `AGENTS.md` rules, "accountability rules", saved state, code generators, Go baseline tests and gate scripts.
- **Cost and timeline** *(self-reported)*.
  - An earlier GPT-based attempt spent more than $400k in API-priced tokens, produced more than 1.3M lines of Rust, and stalled at about 84% compatibility.
  - The Opus 5.5 restart reached v0 in about 10 hours and finished in about two weeks, for about $24k API-equivalent.
  - The author has "never read a line of this code".
- **Speed** *(self-reported)*: about 2.1× faster than tsc 7 (Go) as a geometric mean. On VS Code: tsc 6 54.6 s, tsc 7 6.84 s, tsc-rs 4.20 s.
- **Problems.**
  - It inherited Go-port bugs: four of the first five issues filed reproduced upstream behaviour.
  - Monorepo errors and `tsc -b` races.
  - LSP memory grows by about 20 MiB per 1,000 edits.
  - The `--fast` mode changes output.
  - Maintenance now rests entirely on the agent plus the test suite.
- **Takeaway.** It ported from a structurally close, statically typed intermediate (Go), not from the dynamic original.

### TypeScript 7 / Project Corsa (Microsoft, TS → Go)
- **Why Go and not Rust.** A type checker builds cyclic graphs and frees almost nothing until it is done. Rust attempts turned into "write your own GC" with lots of `unsafe`. https://thenewstack.io/microsoft-typescript-devs-explain-why-they-chose-go-over-rust-c/
- **A port, not a rewrite.** A rewrite "could take years and yield an incompatible TypeScript". Memory savings came from structs laid out inline in arrays.
- **Results.** GA on 2026-07-08. Full builds are 7.7–11.9× faster (VS Code: 125.7 s → 10.6 s), and memory is 6–26% lower. https://www.digitalapplied.com/blog/typescript-7-0-ga-native-compiler-migration-playbook-2026

### Bun, Zig → Rust (agents)
- **Scale.** PR #30412 merged 2026-05-14: about 1.0M lines of Rust and 6,755 commits in about 6 days.
- **Method.** A "faithful transliteration": dependency graph, then parallel per-file translation, then compile-error loops, then test parity.
- **Unsafe debt.** The port has **13,365 `unsafe` sites** (uv has about 73), and some functions marked safe were unsound. The audit plans to make about 9.3k of them safe.
- **Status.** Not shipped; releases still come from Zig.
- Andrew Kelley's critique: tests that missed bugs in Zig can't vouch for a million lines of unreviewed Rust.
- Sources: https://www.devclass.com/ai-ml/2026/07/15/zig-creator-calls-buns-claude-rust-rewrite-unreviewed-slop/5271749, https://bun.com/bun-unsafe-audit

### Microsoft Copilot runtime, TS → Rust (agents)
- **Scale and cost.** About 430k lines of TS became about 800k lines of Rust, for about $120k in tokens. It took 14.5 weeks, with about 3 developer-weeks of human time, porting module by module over 135 releases.
- **Results.** 15.9× throughput, and memory down from 1,383 MB to 126 MB.
- **Problems.** **Dozens of regressions that compiled fine.** Agents spent far more time investigating than writing.
- Their summary: "The compiler is not an oracle." https://www.theregister.com/devops/2026/09/18/microsoft-agentically-ports-copilot-runtime-to-rust-for-120k/5297549

### Other ports
- **Google Gemini 4 Argon (C/C++ → Rust).** RE2, libgav1 and Zircon. These are vendor claims only.
- **DARPA TRACTOR / ForCLift.** "Verified lifting" from C to Rust that combines LLMs with formal semantics. https://www.cs.wisc.edu/2025/07/15/translating-legacy-code-for-a-safer-future
- **rustsmith `sympy-rust` (small SymPy precedent).**
  - It rated polys-domains a good port target (0.85) and simplify a poor one (0.49, "upstream_only", because it spans 9 modules).
  - Results: dup_mul 5× and dmp_mul 8.75× faster, but **gcd was 0.17×, i.e. slower than SymPy**.
  - A generic BigInt rational was about 3× slower than CPython's small-int fast path until an i64 fast path was added.
  - A naive Euclidean GCD blew up; it needed SymPy's subresultant PRS.
  - Source: https://github.com/John-Cusack/rustsmith/pull/98

## 2. Human-driven native rewrites

- **fish 4.0, C++ → Rust.** https://fishshell.com/blog/rustport/
  - About 57k lines of C++ became about 75k lines of Rust over roughly 2 years.
  - autocxx let C++ and Rust live in **one binary**, so there was a working fish at every commit.
  - They ported self-contained builtins first and moved tangled subsystems in big chunks.
  - They avoided cleanup during the port so that mistranslations stayed findable.
  - The "second 90%" was testing.
- **uv.** A "drop-in for common pip workflows", not a clone. Every divergence is documented and classified as intentional, a side effect or a bug, and they refuse bug-for-bug compatibility where it is unsafe. https://docs.astral.sh/uv/pip/compatibility/
- **Ruff.** The formatter is more than 99.9% line-identical to Black on Django and Zulip, and known deltas are listed. https://docs.astral.sh/ruff/faq
- **Biome.** Reached more than 96% Prettier compatibility, measured by diffing against Prettier. https://v1.biomejs.dev/blog/biome-wins-prettier-challenge
- **Rolldown 1.0.** Keeps Rollup's plugin API and is 10–30× faster. Hook filters avoid Rust→JS crossings. https://voidzero.dev/posts/announcing-rolldown-1-0
- **pydantic-core (Rust + PyO3).** Rust owns the hot path and the Python API is kept; it is 4–50× faster than v1. https://pydantic.dev/articles/pydantic-v2

## 3. CAS prior art

- **SymEngine (C++).** SymPy only has an opt-in `USE_SYMENGINE` shim, used mostly by `physics.mechanics`. It never replaced SymPy's core because it is "too inflexible": it **bakes in automatic evaluation**. Julia moved off both SymPy and SymEngine for the same reason. https://oscarbenjamin.github.io/blog/czi/post1.html
- **Oscar Benjamin's diagnosis (SymPy lead).**
  - Evaluation at construction time is "a fundamental design flaw", because constructors can do unbounded work.
  - Too much of SymPy runs at the `Expr` layer instead of the `polys` domains.
  - His fix: a non-evaluating core, FLINT for polynomials, and Arb-style evalf.
- **FLINT inside SymPy.** https://oscarbenjamin.github.io/blog/czi/post2.html
  - Factoring a degree-500 polynomial takes 8 s in pure SymPy and 80 ms with FLINT. A `minpoly` case takes more than 5 min versus 300 ms.
  - SymPy ≥1.13 uses python-flint automatically when it is installed, for ZZ/QQ/GF(p), dense univariate polys, ZZ/QQ matrices and number fields.
  - Advice: swap whole layers, not individual functions.
- **Symbolica (Rust + PyO3).**
  - 1.0 shipped 2025-11-23. It is **source-available, not OSS**: commercial or multicore use needs a paid licence, so we cannot vendor it.
  - It uses an append-only global symbol table.
  - Every PyO3 `.so` has its own global state, so they had to merge their extensions into one module.
  - https://symbolica.io/posts/stable_release/
- **egg / egglog.** E-graphs plus Datalog, up to 9.27× faster than egg. The Herbie case study shows the tension between sound and unsound rulesets. https://arxiv.org/pdf/2304.04332
- **Rust bigints.** https://github.com/Rjected/bigint-benchmark-rs, https://malachite.rs/performance
  - rug (GMP, LGPL) is the fastest.
  - malachite is 1.3–1.8× slower.
  - dashu and ibig are in the middle.
  - num-bigint is catastrophic on large numbers: 482 s versus 2.8 s for 10M digits of e.
  - Below about 1000 bits they are all about equal, so **the small-int fast path is what matters for a CAS**.

## 4. Lessons

**What worked**
- A single executable oracle, with the upstream revision pinned.
- Porting the upstream tests and using them as the spec.
- A working product at every step.
- Documented divergences.
- Keeping the public API while replacing the hot path.

**What failed**
- Agents with a weak harness: 84% compatibility for more than $400k.
- Transliterated memory models, which leave `unsafe` debt.
- Trusting "it compiles".
- Rust ownership fighting cyclic graphs.
- Porting a design flaw: SymEngine and auto-evaluation.

**Using agents**
- Explicit mapping rules (`PORTING.md`), gate scripts and per-module conformance counters.
- Parallel workers.
- Expect agents to spend most of their time investigating rather than writing.
- Adversarial review of numeric and `unsafe` code.

## Unverified

- The tsc-rs harness details and the two test counts.
- Findings of the TRACTOR benchmark paper.
- Malachite's licence.
- Whether SymPy 1.15 has shipped, or uses `fmpz_mpoly` by default.
- Any Symbolica-vs-SymPy benchmark.
- All AI-port cost and speed figures, which are self-reported.
