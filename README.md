# RustSympyRewrite

A computer algebra system in pure Rust, covering high-school through early-college math: algebra, polynomials, equation solving, trig, calculus, and basic linear algebra. It is modelled on [SymPy](https://www.sympy.org) and checked against it.

Status: Phase 0 (test harness). Nothing is implemented yet. See [`docs/PLAN.md`](docs/PLAN.md) for the roadmap and [`docs/research/rust-rewrites.md`](docs/research/rust-rewrites.md) for the research behind it.

## Layout

- `crates/rsympy-num`: exact numbers.
- `crates/rsympy`: expressions and the math itself.
- `oracle/`: Python scripts (`gen_cases.py` plus one module per topic in `oracle/topics/`) that ask a pinned SymPy for the right answers and save them as golden test cases. Cases marked `spec` are answers we chose on purpose where SymPy is wrong for students; `oracle/caselib.py` documents the format.

## Checking against SymPy

```sh
cargo test -p rsympy --test oracle -- --nocapture   # per-topic score
```

To regenerate the golden cases after editing `oracle/gen_cases.py`:

```sh
python -m venv .venv && .venv/bin/pip install -r oracle/requirements.txt
.venv/bin/python oracle/gen_cases.py
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
