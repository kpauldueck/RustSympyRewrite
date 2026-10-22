# RustSympyRewrite

A rewrite of [SymPy](https://www.sympy.org) in Rust: the full library, behaving exactly like SymPy 1.14 (1:1), faster, lighter and memory-safe, and checked against SymPy itself. Work is ordered by what a real application, a homework-grading app, needs first.

Status: Tier 0 (test harness). Nothing is implemented yet. See [`docs/PLAN.md`](docs/PLAN.md) for the roadmap. The research behind it is in [`docs/research/rust-rewrites.md`](docs/research/rust-rewrites.md) (lessons from other rewrites) and [`docs/research/sympy-module-survey.md`](docs/research/sympy-module-survey.md) (every SymPy module).

## Layout

- `crates/rsympy-num`: exact numbers (integers, rationals, modular integers) and Python's RNG.
- `crates/rsympy`: the facade that will re-export the public API. For now it holds the oracle runner (`tests/oracle.rs`). The other planned crates (`rsympy-core`, `rsympy-mpf`, the domain crates, `rsympy-py`) are described in the plan.
- `oracle/`: Python that asks a pinned SymPy for the right answers and saves them as golden test cases. `gen_cases.py` runs everything and defines six general topics; the other topics are one module each in `oracle/topics/`. `inventory.py` measures SymPy itself (`sympy_inventory.json`). Every expected answer is what SymPy itself returns (or the exception it raises), computed live; nothing is hand-written. `oracle/caselib.py` documents the format.

## Checking against SymPy

```sh
cargo test -p rsympy --test oracle -- --nocapture   # per-topic score
```

To regenerate the golden cases after editing a generator in `oracle/`:

```sh
python -m venv .venv && .venv/bin/pip install -r oracle/requirements.txt
PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python .venv/bin/python oracle/gen_cases.py
```

A failing case is either fixed or listed in `oracle/divergences.json` with a category and a reason; only a human reviewer may add an entry there or lower a number in `oracle/baseline.json`.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
