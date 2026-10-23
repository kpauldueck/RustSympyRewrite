//! Differential tests against SymPy: replays `oracle/diff/core_function.json`
//! (the function framework and `Mod`; see `oracle/gen_diff_function.py`).

mod common;

use std::sync::Arc;

use common::{Value, Vector, run_partial};
use rsympy_core::expr::UndefData;
use rsympy_core::functions::{
    self, FunctionClass, Head, Hints, NargsArg, application, head, head_of, lambda,
};
use rsympy_core::{ClassId, Expr, Fact, ctx, evalf, ops, symbol, testing};
use serde_json::Value as Json;

/// `make_function(spec)` of the generator: `Function(name or Symbol,
/// **assumptions, nargs=...)`.
fn make_function(spec: &Json) -> Arc<UndefData> {
    let kwargs: Vec<(&str, Option<bool>)> = spec
        .get("assumptions")
        .map(|a| {
            a.as_array()
                .expect("assumption pairs")
                .iter()
                .map(|kv| (kv[0].as_str().expect("a fact name"), kv[1].as_bool()))
                .collect()
        })
        .unwrap_or_default();
    let assumptions = symbol::parse_assumptions(&kwargs);
    let small = |n: &Json| n.as_i64().expect("a small nargs");
    let nargs = spec.get("nargs").map(|n| match n {
        Json::Null => NargsArg::None,
        Json::Array(items) => NargsArg::Tuple(items.iter().map(small).collect()),
        n => NargsArg::Int(small(n)),
    });
    match spec.get("symbol") {
        Some(s) => application::undefined_function_from_symbol(
            &testing::parse(s.as_str().expect("an encoded symbol")),
            &assumptions,
            nargs,
        ),
        None => application::undefined_function(
            spec["name"].as_str().expect("a name"),
            &assumptions,
            nargs,
        ),
    }
}

/// The `nargs=` keyword of a `WildFunction` vector (`None`: not given).
fn wild_nargs(v: &Vector) -> Option<NargsArg> {
    let small = |n: &Json| n.as_i64().expect("a small nargs");
    v.params.get("nargs").map(|n| match n {
        Json::Null => NargsArg::None,
        Json::Array(items) => NargsArg::Tuple(items.iter().map(small).collect()),
        n => NargsArg::Int(small(n)),
    })
}

/// The class a `cls` param names.
fn class(v: &Vector) -> ClassId {
    let name = v.param_str("cls");
    testing::class_by_name(name).unwrap_or_else(|| panic!("no class {name}"))
}

/// `evaluate=` as the vector passed it (`None`: not given).
fn evaluate_kw(v: &Vector) -> Option<bool> {
    v.params.get("evaluate").map(|e| e.as_bool().expect("a bool"))
}

/// Runs `f` under `with evaluate(False)` when the vector asks for it.
fn with_global_evaluate<T>(v: &Vector, f: impl FnOnce() -> T) -> T {
    match v.params.get("global_evaluate") {
        Some(Json::Bool(false)) => ctx::with_evaluate(false, f),
        _ => f(),
    }
}

fn exprs(v: &Vector, from: usize) -> Vec<Expr> {
    (from..v.args.len()).map(|i| v.expr(i)).collect()
}

fn opt(e: Option<Expr>) -> Value {
    e.map_or(Value::None, Value::Expr)
}

fn pair((a, b): (Expr, Expr)) -> Value {
    Value::Tuple(vec![Value::Expr(a), Value::Expr(b)])
}

fn tuple(items: Vec<Expr>) -> Value {
    Value::Tuple(items.into_iter().map(Value::Expr).collect())
}

fn class_key((a, b, name): (i32, i32, String)) -> Value {
    Value::Tuple(vec![
        Value::Int(a.into()),
        Value::Int(b.into()),
        Value::Str(name),
    ])
}

fn dispatch(v: &Vector) -> Value {
    let e = || v.expr(0);
    match v.op.as_str() {
        // undefined functions
        "undef_call" => with_global_evaluate(v, || {
            let f = make_function(&v.params["spec"]);
            Value::Expr(functions::call_undefined_with(&f, exprs(v, 0), evaluate_kw(v)))
        }),
        "undef_nargs" => Value::Str(make_function(&v.params["spec"]).nargs().to_string()),
        "undef_class_key" => class_key(make_function(&v.params["spec"]).class_key()),
        "undef_name" => Value::Str(make_function(&v.params["spec"]).name().to_string()),
        "undef_instance_nargs" => {
            let f = make_function(&v.params["spec"]);
            let applied = functions::call_undefined(&f, exprs(v, 0));
            Value::Str(application::nargs_of(&applied).to_string())
        }
        "undef_eq" => {
            let f = make_function(&v.params["spec"]);
            let g = make_function(&v.params["other"]);
            Value::Bool(f == g)
        }
        "undef_isinstance" => {
            let f = make_function(&v.params["spec"]);
            let applied = functions::call_undefined(&f, exprs(v, 0));
            let g = make_function(&v.params["other"]);
            Value::Bool(g.is_instance(&applied))
        }
        "undef_func_eq" => {
            let f = make_function(&v.params["spec"]);
            let applied = functions::call_undefined(&f, exprs(v, 0));
            let g = make_function(&v.params["other"]);
            Value::Bool(FunctionClass::of(&applied) == Some(FunctionClass::Undefined(g)))
        }
        // defined classes
        "class_key" => class_key(head(class(v)).class_key()),
        "class_nargs" => Value::Str(head(class(v)).nargs().to_string()),
        "defined_call" => Value::Expr(functions::call_with(
            class(v),
            exprs(v, 0),
            evaluate_kw(v),
        )),
        // inherited hooks
        "eval_conjugate" => opt(functions::eval_conjugate_of(&e())),
        "eval_transpose" => opt(functions::eval_transpose_of(&e())),
        "eval_adjoint" => opt(functions::eval_adjoint_of(&e())),
        "as_base_exp" => pair(e().as_base_exp()),
        "as_numer_denom" => pair(e().as_numer_denom()),
        "eval_subs" => opt(functions::eval_subs_of(&e(), &v.expr(1), &v.expr(2))),
        "fdiff" => {
            let i = v.int(1).as_i64().expect("a small index");
            let e = e();
            Value::Expr(head_of(&e).fdiff(&e, i as usize))
        }
        "inverse" => {
            let e = e();
            let inv = head_of(&e).inverse(&e, 1);
            panic!("inverse returned a class: {inv:?}")
        }
        "func" => Value::Expr(e().func(exprs(v, 1))),
        "evalf" => Value::Expr(evalf::evalf(&e(), v.param_i64("n"))),
        "rewrite_floor" => with_global_evaluate(v, || {
            let e = e();
            let rv = functions::eval_rewrite_as_of(&e, "floor", e.args(), &Hints::default());
            opt(rv.expect("Mod has _eval_rewrite_as_floor"))
        }),
        "is" => {
            let fact = Fact::from_name(v.str(1)).expect("a fact name");
            Value::fuzzy(e().is(fact))
        }
        // Lambda
        "lambda" => Value::Expr(lambda::lambda(&v.expr(0), &v.expr(1))),
        "lambda_nargs" => Value::Str(lambda::nargs(&e()).to_string()),
        "lambda_variables" | "lambda_bound_symbols" => tuple(lambda::variables(&e())),
        "lambda_signature" => Value::Expr(lambda::signature(&e())),
        "lambda_expr" => Value::Expr(lambda::expr(&e())),
        "lambda_free_symbols" => {
            let mut syms = lambda::free_symbols(&e());
            syms.sort_by_key(|s| s.to_string());
            Value::List(syms.into_iter().map(Value::Expr).collect())
        }
        "lambda_is_identity" => Value::Bool(lambda::is_identity(&e())),
        "lambda_call" => Value::Expr(lambda::call(&e(), &exprs(v, 1))),
        // WildFunction
        "wild_nargs" => {
            let w = application::wild_function(v.str(0), wild_nargs(v));
            Value::Str(application::wild_function_nargs(&w).to_string())
        }
        "wild_matches" => {
            let w = application::wild_function(v.str(0), wild_nargs(v));
            match application::wild_function_matches(&w, &v.expr(1), None) {
                None => Value::None,
                Some(d) => Value::List(d.values().cloned().map(Value::Expr).collect()),
            }
        }
        // Mod
        "Mod" => with_global_evaluate(v, || {
            Value::Expr(functions::call_with(
                ClassId::Mod,
                exprs(v, 0),
                evaluate_kw(v),
            ))
        }),
        "op_mod" => Value::Expr(ops::rem(&v.expr(0), &v.expr(1))),
        _ => v.unknown_op(),
    }
}

#[test]
fn function_matches_sympy() {
    run_partial("function", 0, dispatch);
}
