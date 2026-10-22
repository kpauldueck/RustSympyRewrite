//! Exact numbers for rsympy.
//!
//! Tier 1 (see `docs/PLAN.md`) fills this in: `Int` with an inline `i64`
//! fast path and a `dashu` fallback, using Python's floor-division and modulo
//! semantics, `Rational`, `ModInt`, and a CPython-compatible MT19937.
//! Floats live in `rsympy-mpf`.
