# RustSympyRewrite

A rewrite of [SymPy](https://www.sympy.org) in Rust: the full library, as accurate as SymPy, faster, lighter and safer, and checked against SymPy itself. Work is ordered by what a real application, a homework-grading app, needs first.

Status: Tier 0 (test harness). Nothing is implemented yet. See [`docs/PLAN.md`](docs/PLAN.md) for the roadmap. The research behind it is in [`docs/research/rust-rewrites.md`](docs/research/rust-rewrites.md) (lessons from other rewrites) and [`docs/research/sympy-module-survey.md`](docs/research/sympy-module-survey.md) (every SymPy module).

## Layout

- `crates/rsympy-num`: exact numbers.
- `crates/rsympy`: expressions and the math itself.
- `oracle/`: Python scripts (`gen_cases.py` plus one module per topic in `oracle/topics/`) that ask a pinned SymPy for the right answers and save them as golden test cases. Each case says where its answer came from (`sympy`, `app`, or `spec` for answers we chose on purpose where SymPy is wrong for students) and which rsympy profile must pass it (`both`, `strict` or `parity`); `oracle/caselib.py` documents the format.

## Checking against SymPy

```sh
cargo test -p rsympy --test oracle -- --nocapture   # per-topic score
```

To regenerate the golden cases after editing `oracle/gen_cases.py`:

```sh
python -m venv .venv && .venv/bin/pip install -r oracle/requirements.txt
PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python .venv/bin/python oracle/gen_cases.py
```

A failing case is either fixed or listed in `oracle/divergences.json` with a category and a reason; only a human reviewer may add an entry there or lower a number in `oracle/baseline.json`.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
