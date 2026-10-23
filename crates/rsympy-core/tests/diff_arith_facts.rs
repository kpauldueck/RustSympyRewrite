//! Differential tests against SymPy: replays
//! `oracle/diff/core_arith_facts.json` (the assumptions of Add, Mul and Pow;
//! see `oracle/gen_diff_arith_facts.py`).

mod common;

use common::{Value, run_partial};
use rsympy_core::Fact;

// Unported: list.sort (Add and Mul args with several terms), Function.__new__
// (functions of non-numbers), Expr.coeff, factor_terms, evalf and the
// polys behind _monotonic_sign.
#[test]
fn arith_facts_match_sympy() {
    run_partial("arith_facts", 11183, |v| match v.op.as_str() {
        "is" => {
            let fact = Fact::from_name(v.str(1)).expect("a fact name");
            Value::fuzzy(v.expr(0).is(fact))
        }
        _ => v.unknown_op(),
    });
}
