//! Differential tests against SymPy: replays
//! `oracle/diff/core_relational.json` (Python's comparison operators,
//! the Relational classes and the fuzzy comparisons of `core.relational`,
//! and `logic.boolalg`; see `oracle/gen_diff_relational.py`).

mod common;

use common::{Value, Vector, run_partial};
use rsympy_core::ClassId as C;
use rsympy_core::logic::boolalg::{self as b, BitOp};
use rsympy_core::relational::{self as r, Fuzzy};
use rsympy_core::{Expr, compare, ctx, ops};

fn ex(e: Expr) -> Value {
    Value::Expr(e)
}

fn fuzzy(v: Fuzzy) -> Value {
    match v {
        Fuzzy::Py(Some(t)) => Value::Bool(t),
        Fuzzy::Py(None) => Value::None,
        Fuzzy::Sym(e) => Value::Expr(e),
    }
}

/// A set SymPy returns, recorded sorted by `default_sort_key`.
fn sorted_set(mut v: Vec<Expr>) -> Value {
    compare::sorted_default(&mut v);
    Value::List(v.into_iter().map(Value::Expr).collect())
}

fn args(v: &Vector) -> Vec<Expr> {
    (0..v.args.len()).map(|i| v.expr(i)).collect()
}

/// `params[key]` as Python's `True`/`False`/`None`.
fn opt_bool(v: &Vector, key: &str) -> Option<bool> {
    v.params.get(key).and_then(|j| j.as_bool())
}

fn rel_class(name: &str) -> C {
    match name {
        "Eq" => C::Equality,
        "Ne" => C::Unequality,
        "Lt" => C::StrictLessThan,
        "Le" => C::LessThan,
        "Gt" => C::StrictGreaterThan,
        "Ge" => C::GreaterThan,
        _ => panic!("unknown relational class {name}"),
    }
}

fn bool_class(name: &str) -> C {
    match name {
        "And" => C::And,
        "Or" => C::Or,
        "Not" => C::Not,
        "Xor" => C::Xor,
        "Nand" => C::Nand,
        "Nor" => C::Nor,
        "Xnor" => C::Xnor,
        "Implies" => C::Implies,
        "Equivalent" => C::Equivalent,
        "ITE" => C::ITE,
        "Exclusive" => C::Exclusive,
        _ => panic!("unknown Boolean class {name}"),
    }
}

/// Python's `p <op> q`.
fn compare_op(op: &str, p: &Expr, q: &Expr) -> Expr {
    match op {
        "<" => ops::lt(p, q),
        "<=" => ops::le(p, q),
        ">" => ops::gt(p, q),
        ">=" => ops::ge(p, q),
        _ => panic!("unknown comparison {op}"),
    }
}

fn arith_op(op: &str, p: &Expr, q: &Expr) -> Expr {
    match op {
        "+" => ops::add(p, q),
        "-" => ops::sub(p, q),
        "*" => ops::mul(p, q),
        "/" => ops::div(p, q),
        "**" => ops::pow(p, q),
        "%" => ops::rem(p, q),
        "//" => ops::floordiv(p, q),
        _ => panic!("unknown arithmetic operator {op}"),
    }
}

fn bit_op(op: &str) -> BitOp {
    match op {
        "&" => BitOp::And,
        "|" => BitOp::Or,
        "^" => BitOp::Xor,
        ">>" => BitOp::Rshift,
        "<<" => BitOp::Lshift,
        _ => panic!("unknown bitwise operator {op}"),
    }
}

/// `prop(name)(e)` in the generator.
fn prop(name: &str, e: &Expr) -> Value {
    match name {
        "lhs" => ex(r::lhs(e)),
        "rhs" => ex(r::rhs(e)),
        "rel_op" => Value::Str(r::rel_op(e).to_string()),
        "gts" => ex(r::gts(e)),
        "lts" => ex(r::lts(e)),
        "reversed" => ex(r::reversed(e)),
        "reversedsign" => ex(r::reversedsign(e)),
        "negated" => ex(r::negated(e)),
        "canonical" => ex(r::canonical(e)),
        "weak" => ex(r::weak(e)),
        "strict" => ex(r::strict(e)),
        "binary_symbols" => sorted_set(r::binary_symbols(e)),
        "canonical_coeff" => ex(r::canonical_coeff(e)),
        "bool" => Value::Bool(e.truthy()),
        "func" => ex(e.func(e.args().to_vec())),
        "is_Relational" => Value::Bool(e.is_Relational()),
        _ => panic!("unknown property {name}"),
    }
}

fn form(op: &str, v: &Vector) -> Option<Value> {
    let e = || v.expr(0);
    Some(match op {
        "is_literal" => Value::Bool(b::is_literal(&e())),
        "is_nnf" => Value::Bool(b::is_nnf(&e(), v.params["simplified"].as_bool().unwrap())),
        "to_nnf" => ex(b::to_nnf(&e(), v.params["simplify"].as_bool().unwrap())),
        "is_cnf" => Value::Bool(b::is_cnf(&e())),
        "is_dnf" => Value::Bool(b::is_dnf(&e())),
        "is_anf" => Value::Bool(b::is_anf(&e())),
        "to_cnf" | "to_dnf" => {
            let simplify = v.params["simplify"].as_bool().unwrap();
            let force = opt_bool(v, "force").unwrap_or(false);
            ex(if op == "to_cnf" {
                b::to_cnf(&e(), simplify, force)
            } else {
                b::to_dnf(&e(), simplify, force)
            })
        }
        "to_anf" => ex(b::to_anf(&e(), v.params["deep"].as_bool().unwrap())),
        "eliminate_implications" => ex(b::eliminate_implications(&e())),
        "distribute_and_over_or" => ex(b::distribute_and_over_or(&e())),
        "distribute_or_over_and" => ex(b::distribute_or_over_and(&e())),
        "distribute_xor_over_and" => ex(b::distribute_xor_over_and(&e())),
        "conjuncts" => sorted_set(b::conjuncts(&e())),
        "disjuncts" => sorted_set(b::disjuncts(&e())),
        "find_predicates" => sorted_set(b::find_predicates(&e())),
        _ => return None,
    })
}

// Unported: Relational.equals and simplify (Expr.equals, simplify), the
// evalf of a non-Number difference in _n2 (pi - 1), Float comparisons
// with comparable non-Numbers (evalf), is_eq's clear_coefficients path,
// floor/ceiling's _eval_is_eq (rewrite), Interval comparisons (sets) and
// simplify_logic.
#[test]
fn relational_matches_sympy() {
    run_partial("relational", 2899, |v| {
        let p = || v.expr(0);
        let q = || v.expr(1);
        match v.op.as_str() {
            "cmp" => {
                let eval = v.params["eval"].as_bool().unwrap();
                let op = v.param_str("cmp");
                ex(ctx::with_evaluate(eval, || compare_op(op, &p(), &q())))
            }
            "rel_new" => {
                let cls = rel_class(v.param_str("cls"));
                let evaluate = opt_bool(v, "evaluate");
                let globl = v.params["globl"].as_bool().unwrap();
                ex(ctx::with_evaluate(globl, || {
                    r::new_rel_args(cls, &args(v), evaluate)
                }))
            }
            "Rel" => {
                let rop = v.params["rop"].as_str();
                let evaluate = opt_bool(v, "evaluate");
                ex(r::rel(&p(), &q(), rop, evaluate))
            }
            "fuzzy" => fuzzy(match v.param_str("func") {
                "is_lt" => Fuzzy::Py(r::is_lt(&p(), &q())),
                "is_gt" => Fuzzy::Py(r::is_gt(&p(), &q())),
                "is_le" => r::is_le_exact(&p(), &q()),
                "is_ge" => r::is_ge_exact(&p(), &q()),
                "is_eq" => r::is_eq_exact(&p(), &q()),
                "is_neq" => Fuzzy::Py(r::is_neq(&p(), &q())),
                other => panic!("unknown fuzzy comparison {other}"),
            }),
            "prop" => prop(v.param_str("name"), &p()),
            "equals" => Value::fuzzy(r::equals(&p(), &q())),
            "simplify" => ex(r::simplify(&p())),
            "bool_new" => {
                let cls = bool_class(v.param_str("cls"));
                let evaluate = opt_bool(v, "evaluate");
                let globl = v.params["globl"].as_bool().unwrap();
                let keep_true = opt_bool(v, "remove_true") == Some(false);
                ex(ctx::with_evaluate(globl, || {
                    if keep_true {
                        b::xor_with(args(v), false)
                    } else {
                        b::new_boolfunc(cls, args(v), evaluate)
                    }
                }))
            }
            "as_Boolean" => ex(b::as_boolean(&p())),
            "binary_check" => Value::List(
                b::binary_check_and_simplify(&args(v))
                    .into_iter()
                    .map(Value::Expr)
                    .collect(),
            ),
            "bitop" => ex(b::bit_op(&p(), &q(), bit_op(v.param_str("sym")))),
            "invert" => ex(ops::invert(&p())),
            "arith" => ex(arith_op(v.param_str("sym"), &p(), &q())),
            "neg" => ex(ops::neg(&p())),
            "abs" => ex(ops::abs(&p())),
            op => form(op, v).unwrap_or_else(|| v.unknown_op()),
        }
    });
}
