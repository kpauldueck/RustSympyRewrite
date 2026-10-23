//! Differential tests against SymPy: replays `oracle/diff/core_arith.json`
//! (Add, Mul and Pow; see `oracle/gen_diff_arith.py`).

mod common;

use common::{Value, run_partial};
use rsympy_core::{Expr, add, ctx, mul, power};

fn pair((a, b): (Expr, Expr)) -> Value {
    Value::Tuple(vec![Value::Expr(a), Value::Expr(b)])
}

fn coeff_and_args((c, args): (Expr, Vec<Expr>)) -> Value {
    Value::Tuple(vec![
        Value::Expr(c),
        Value::Tuple(args.into_iter().map(Value::Expr).collect()),
    ])
}

#[test]
fn arith_matches_sympy() {
    run_partial("arith", 12668, |v| {
        let args = || (0..v.args.len()).map(|i| v.expr(i)).collect::<Vec<_>>();
        let e = || v.expr(0);
        match v.op.as_str() {
            "Add" => Value::Expr(add::add(args())),
            "Mul" if v.params.get("distribute").is_some() => {
                Value::Expr(ctx::with_distribute(false, || mul::mul(args())))
            }
            "Mul" => Value::Expr(mul::mul(args())),
            "Pow" => Value::Expr(power::pow(&v.expr(0), &v.expr(1))),
            "neg" => Value::Expr(-e()),
            "as_numer_denom" => pair(e().as_numer_denom()),
            "could_extract_minus_sign" => Value::Bool(e().could_extract_minus_sign()),
            "as_coeff_Mul" => pair(e().as_coeff_Mul(false)),
            "as_coeff_Mul_rational" => pair(e().as_coeff_Mul(true)),
            "as_coeff_Add" => pair(e().as_coeff_Add(false)),
            "as_coeff_mul" => coeff_and_args(e().as_coeff_mul()),
            "as_coeff_add" => coeff_and_args(e().as_coeff_add()),
            "as_base_exp" => pair(e().as_base_exp()),
            "primitive" => pair(add::primitive(&e())),
            "expand_power_base" => Value::Expr(power::expand_power_base_with(&e(), false)),
            "expand_power_base_force" => Value::Expr(power::expand_power_base_with(&e(), true)),
            "keep_coeff" => Value::Expr(mul::keep_coeff(&v.expr(0), &v.expr(1), true, false)),
            "keep_coeff_noclear_sign" => {
                Value::Expr(mul::keep_coeff(&v.expr(0), &v.expr(1), false, true))
            }
            "unevaluated_Add" => Value::Expr(add::unevaluated_add(args())),
            "unevaluated_Mul" => Value::Expr(mul::unevaluated_mul(args())),
            _ => v.unknown_op(),
        }
    });
}
