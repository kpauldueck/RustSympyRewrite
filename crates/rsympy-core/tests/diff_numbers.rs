//! Differential tests against SymPy: replays `oracle/diff/core_numbers.json`
//! (number arithmetic and assumptions; see `oracle/gen_diff_numbers.py`).

mod common;

use common::{Value, run_partial};
use rsympy_core::{Fact, ops};

#[test]
fn numbers_match_sympy() {
    // Unported: list.sort (results with several terms) and
    // Function.__new__ (Mod, Abs of non-numbers).
    run_partial("numbers", 90, |v| {
        let e = || Value::Expr;
        match v.op.as_str() {
            "add" => e()(ops::add(&v.expr(0), &v.expr(1))),
            "sub" => e()(ops::sub(&v.expr(0), &v.expr(1))),
            "mul" => e()(ops::mul(&v.expr(0), &v.expr(1))),
            "div" => e()(ops::div(&v.expr(0), &v.expr(1))),
            "mod" => e()(ops::rem(&v.expr(0), &v.expr(1))),
            "floordiv" => e()(ops::floordiv(&v.expr(0), &v.expr(1))),
            "pow" => e()(ops::pow(&v.expr(0), &v.expr(1))),
            "neg" => e()(ops::neg(&v.expr(0))),
            "abs" => e()(ops::abs(&v.expr(0))),
            "is" => {
                let fact = Fact::from_name(v.str(1)).expect("a fact name");
                Value::fuzzy(v.expr(0).is(fact))
            }
            _ => v.unknown_op(),
        }
    });
}
