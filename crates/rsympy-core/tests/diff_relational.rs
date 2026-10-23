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
        "simplify_logic" => ex(b::simplify_logic(
            &e(),
            v.params["form"].as_str(),
            v.params["deep"].as_bool().unwrap(),
            false,
        )),
        _ => return qm(op, v),
    })
}

fn exprs(v: &Value) -> Vec<Expr> {
    match v {
        Value::List(items) => items
            .iter()
            .map(|i| match i {
                Value::Expr(e) => e.clone(),
                other => panic!("not an expression: {other:?}"),
            })
            .collect(),
        other => panic!("not a list: {other:?}"),
    }
}

fn ints(v: &Value) -> Vec<i64> {
    match v {
        Value::List(items) | Value::Tuple(items) => items
            .iter()
            .map(|i| match i {
                Value::Int(n) => n.as_i64().expect("a small int"),
                other => panic!("not an int: {other:?}"),
            })
            .collect(),
        other => panic!("not a list: {other:?}"),
    }
}

/// A minterm as `SOPform` takes it.
fn term_spec(v: &Value) -> b::TermSpec {
    match v {
        Value::Int(n) => b::TermSpec::Int(n.clone()),
        Value::List(_) | Value::Tuple(_) => b::TermSpec::Bits(ints(v)),
        Value::Dict(kv) => b::TermSpec::Dict(
            kv.iter()
                .map(|(k, x)| match (k, x) {
                    (Value::Expr(k), Value::Int(x)) => (k.clone(), x.as_i64().unwrap()),
                    other => panic!("not a variable and a bit: {other:?}"),
                })
                .collect(),
        ),
        _ => b::TermSpec::Other,
    }
}

fn term_specs(v: &Value) -> Vec<b::TermSpec> {
    match v {
        Value::List(items) => items.iter().map(term_spec).collect(),
        Value::None => vec![],
        other => panic!("not a list: {other:?}"),
    }
}

/// `SOPform` and friends.
fn qm(op: &str, v: &Vector) -> Option<Value> {
    let int_list =
        |xs: Vec<i64>| Value::List(xs.into_iter().map(|x| Value::Int(x.into())).collect());
    Some(match op {
        "qm_form" => {
            let vars = exprs(&v.args[0]);
            let (mt, dc) = (term_specs(&v.args[1]), term_specs(&v.args[2]));
            ex(match v.param_str("name") {
                "SOPform" => b::sop_form(&vars, &mt, &dc),
                _ => b::pos_form(&vars, &mt, &dc),
            })
        }
        "ANFform" => ex(b::anf_form(&exprs(&v.args[0]), &ints(&v.args[1]))),
        "anf_coeffs" => int_list(b::anf_coeffs(&ints(&v.args[0]))),
        "bool_term" => {
            let k = match &v.args[0] {
                Value::Int(n) => b::TermIndex::Int(n.clone()),
                other => b::TermIndex::Bits(ints(other)),
            };
            let vars = exprs(&v.args[1]);
            ex(match v.param_str("name") {
                "bool_minterm" => b::bool_minterm(&k, &vars),
                "bool_maxterm" => b::bool_maxterm(&k, &vars),
                _ => b::bool_monomial(&k, &vars),
            })
        }
        "term_to_integer" => Value::Int(b::term_to_integer(&ints(&v.args[0]))),
        "to_int_repr" => Value::List(
            b::to_int_repr(&exprs(&v.args[0]), &exprs(&v.args[1]))
                .into_iter()
                .map(int_list)
                .collect(),
        ),
        "gateinputcount" => Value::Int((b::gateinputcount(&v.expr(0)) as i64).into()),
        _ => return None,
    })
}

// The 2924 vectors that reach unported code:
//   1366 `_monotonic_sign` of an Add or of a symbolic denominator (polys),
//        from comparisons of sums and products of symbols
//    774 the evalf of a non-Number difference in `_n2` (pi - 1, ...)
//    518 `Function.__new__` (floor, ceiling, re, im of their arguments)
//    167 RoundFunction's `_eval_is_integer` and `_eval_is_finite`
//     36 `And`, `Or`, `Xor` of Sets (sets)
//     27 `simplify_logic` past its quick exits (`_get_truthtable` and the
//        relational substitutions need Basic.xreplace)
//     24 `Expr._eval_is_extended_positive_negative` of an Add (evalf)
//     10 `Relational._eval_simplify` (simplify)
//      1 `Relational.equals` of sides that need `Expr.equals`
//      1 `Expr.as_real_imag` under evaluate(False)
#[test]
fn relational_matches_sympy() {
    run_partial("relational", 2924, |v| {
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
