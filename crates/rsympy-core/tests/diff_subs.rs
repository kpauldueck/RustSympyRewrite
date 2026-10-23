//! Differential tests against SymPy: replays `oracle/diff/core_subs.json`
//! (`Basic.subs`, `xreplace`, `has`, `atoms`, `free_symbols`, the tree
//! walks of `sympy.core.traversal`, `Expr.as_independent`, `Expr.coeff`,
//! `is_polynomial` and `is_rational_function`; see
//! `oracle/gen_diff_subs.py`).

mod common;

use common::{Value, Vector, run_partial};
use rsympy_core::basic::{self, Pattern, SubsArgs};
use rsympy_core::compare::{SortKey, default_sort_key, key_eq, key_lt, py_sort_by};
use rsympy_core::numbers;
use rsympy_core::py::PyDict;
use rsympy_core::traversal::{self, Keys};
use rsympy_core::{ClassId as C, Tag};
use rsympy_core::{Expr, testing};
use serde_json::Value as Json;

/// One argument as SymPy receives it after `sympify`: an `enc()` text
/// (parsed here, so that building it can reach unported code), a Python
/// int or a Python float.
fn to_expr(a: &Value) -> Expr {
    match a {
        Value::Str(s) => {
            assert!(
                !s.starts_with("class:") && !s.starts_with("undef:") && !s.starts_with("pystr:"),
                "not an expression: {s}"
            );
            testing::parse(s)
        }
        Value::Int(n) => Expr::integer(n.clone()),
        Value::Float(r) => numbers::float_from_f64(r.parse().expect("a float repr")),
        other => panic!("not an expression argument: {other:?}"),
    }
}

/// Argument `i` as an expression.
fn ex(v: &Vector, i: usize) -> Expr {
    to_expr(&v.args[i])
}

/// The items of list argument `i`.
fn items(v: &Vector, i: usize) -> &[Value] {
    match &v.args[i] {
        Value::List(items) => items,
        other => panic!("argument {i} of {} is not a list: {other:?}", v.op),
    }
}

/// List argument `i` as expressions.
fn exprs(v: &Vector, i: usize) -> Vec<Expr> {
    items(v, i).iter().map(to_expr).collect()
}

/// List argument `i` of `[old, new]` pairs.
fn pairs(v: &Vector, i: usize) -> Vec<(Expr, Expr)> {
    items(v, i)
        .iter()
        .map(|p| match p {
            Value::List(kv) if kv.len() == 2 => (to_expr(&kv[0]), to_expr(&kv[1])),
            other => panic!("not an (old, new) pair: {other:?}"),
        })
        .collect()
}

/// The pairs of a Python dict built from `pairs` (a repeated key keeps its
/// first position and its last value).
fn dict(pairs: Vec<(Expr, Expr)>) -> PyDict<Expr, Expr> {
    let mut d = PyDict::new();
    for (k, x) in pairs {
        d.insert(k, x);
    }
    d
}

/// A class used as a pattern (`class:<Name>` in the generator).
fn class(name: &str) -> Pattern {
    match name {
        "Basic" => Pattern::Tag(Tag::Basic),
        "Atom" => Pattern::Tag(Tag::Atom),
        "Expr" => Pattern::Tag(Tag::Expr),
        "AtomicExpr" => Pattern::Tag(Tag::AtomicExpr),
        "Number" => Pattern::Tag(Tag::Number),
        "NumberSymbol" => Pattern::Tag(Tag::NumberSymbol),
        "Function" => Pattern::Tag(Tag::Function),
        "AssocOp" => Pattern::Tag(Tag::AssocOp),
        "Symbol" => Pattern::Class(C::Symbol),
        "Dummy" => Pattern::Class(C::Dummy),
        "Wild" => Pattern::Class(C::Wild),
        "Integer" => Pattern::Class(C::Integer),
        "Rational" => Pattern::Class(C::Rational),
        "Float" => Pattern::Class(C::Float),
        "Add" => Pattern::Class(C::Add),
        "Mul" => Pattern::Class(C::Mul),
        "Pow" => Pattern::Class(C::Pow),
        "AppliedUndef" => Pattern::Class(C::AppliedUndef),
        "Tuple" => Pattern::Class(C::Tuple),
        "Exp" => Pattern::Class(C::Exp),
        "Sin" => Pattern::Class(C::Sin),
        "Cos" => Pattern::Class(C::Cos),
        "ImaginaryUnit" => Pattern::Class(C::ImaginaryUnit),
        "Pi" => Pattern::Class(C::Pi),
        "Half" => Pattern::Class(C::Half),
        "One" => Pattern::Class(C::One),
        "Zero" => Pattern::Class(C::Zero),
        "NegativeOne" => Pattern::Class(C::NegativeOne),
        "Infinity" => Pattern::Class(C::Infinity),
        "Integral" => Pattern::Class(C::Integral),
        "Derivative" => Pattern::Class(C::Derivative),
        _ => panic!("no class {name}"),
    }
}

/// An argument of `has`, `atoms`, `as_independent` or `walk`: a class, an
/// undefined function (`undef:<enc(f(x))>`, standing for `f`) or an
/// expression.
fn pattern(a: &Value) -> Pattern {
    if let Value::Str(s) = a {
        if let Some(name) = s.strip_prefix("class:") {
            return class(name);
        }
        if let Some(text) = s.strip_prefix("undef:") {
            return Pattern::type_of(&testing::parse(text));
        }
    }
    Pattern::Expr(to_expr(a))
}

fn patterns(v: &Vector, i: usize) -> Vec<Pattern> {
    items(v, i).iter().map(pattern).collect()
}

/// `sympify_old`/`sympify_new` of `Basic.subs`, for the `subs_str` pairs:
/// a Python string becomes a Symbol (old) or is parsed (new).
fn str_pair(p: &Value) -> (Expr, Expr) {
    let Value::List(kv) = p else {
        panic!("not an (old, new) pair: {p:?}")
    };
    let old = match &kv[0] {
        Value::Str(s) if s.starts_with("pystr:") => basic::sympify_old_str(&s[6..]),
        a => to_expr(a),
    };
    let new = match &kv[1] {
        Value::Str(s) if s.starts_with("pystr:") => basic::sympify_new_str(&s[6..]),
        a => to_expr(a),
    };
    (old, new)
}

fn param_bool(v: &Vector, key: &str) -> bool {
    v.params.get(key).and_then(Json::as_bool).unwrap_or(false)
}

fn list(es: Vec<Expr>) -> Value {
    Value::List(es.into_iter().map(Value::Expr).collect())
}

/// A set of expressions as the generator records it: sorted by
/// `(default_sort_key(e), enc(e))`.
fn sorted_set(es: Vec<Expr>) -> Value {
    let mut keyed: Vec<(SortKey, String, Expr)> = es
        .into_iter()
        .map(|e| (default_sort_key(&e, None), testing::encode(&e), e))
        .collect();
    py_sort_by(&mut keyed, |a, b| {
        key_lt(&a.0, &b.0) || (key_eq(&a.0, &b.0) && a.1 < b.1)
    });
    list(keyed.into_iter().map(|(_, _, e)| e).collect())
}

/// The traversal order named by the `keys` param.
fn with_keys<R>(v: &Vector, f: impl FnOnce(Keys<'_>) -> R) -> R {
    let dsk = |e: &Expr| default_sort_key(e, None);
    let keys: [&dyn Fn(&Expr) -> SortKey; 1] = [&dsk];
    match v.param_str("keys") {
        "none" => f(Keys::None),
        "true" => f(Keys::Default),
        "default_sort_key" => f(Keys::With(&keys)),
        other => panic!("unknown keys {other}"),
    }
}

fn dispatch(v: &Vector) -> Value {
    match v.op.as_str() {
        "subs" => {
            let (e, old, new) = (ex(v, 0), ex(v, 1), ex(v, 2));
            Value::Expr(basic::subs_with_hints(
                &e,
                SubsArgs::Pair(old, new),
                false,
                param_bool(v, "hack2"),
            ))
        }
        "subs_dict" => {
            let e = ex(v, 0);
            let d = dict(pairs(v, 1));
            let seq = d.into_iter().collect();
            Value::Expr(basic::subs(
                &e,
                SubsArgs::Unordered(seq),
                param_bool(v, "simultaneous"),
            ))
        }
        "subs_list" => {
            let e = ex(v, 0);
            Value::Expr(basic::subs(
                &e,
                SubsArgs::Sequence(pairs(v, 1)),
                param_bool(v, "simultaneous"),
            ))
        }
        "subs_set" => {
            // the generator's sets have distinct olds, so the set's
            // (hash-dependent) order does not matter after sorting
            let e = ex(v, 0);
            Value::Expr(basic::subs(&e, SubsArgs::Unordered(pairs(v, 1)), false))
        }
        "subs_single" => {
            let (e, arg) = (ex(v, 0), ex(v, 1));
            Value::Expr(basic::subs_single(&e, &arg, false))
        }
        "subs_str" => {
            let e = ex(v, 0);
            let d = dict(items(v, 1).iter().map(str_pair).collect());
            Value::Expr(basic::subs(
                &e,
                SubsArgs::Unordered(d.into_iter().collect()),
                false,
            ))
        }
        "xreplace" => {
            let e = ex(v, 0);
            let rule = dict(pairs(v, 1));
            Value::Expr(e.xreplace(&rule))
        }
        "has" => {
            let e = ex(v, 0);
            Value::Bool(basic::has(&e, &patterns(v, 1)))
        }
        "has_free" => {
            let e = ex(v, 0);
            Value::Bool(basic::has_free(&e, &patterns(v, 1)))
        }
        "has_xfree" => {
            let e = ex(v, 0);
            Value::Bool(basic::has_xfree(&e, &exprs(v, 1)))
        }
        "free_symbols" => sorted_set(ex(v, 0).free_symbols()),
        "atoms" => {
            let e = ex(v, 0);
            sorted_set(basic::atoms(&e, &patterns(v, 1)))
        }
        "is_same" => Value::Bool(basic::is_same(&ex(v, 0), &ex(v, 1))),
        "preorder" => {
            let e = ex(v, 0);
            list(with_keys(v, |k| {
                traversal::preorder_traversal(&e, k).collect()
            }))
        }
        "postorder" => {
            let e = ex(v, 0);
            list(with_keys(v, |k| traversal::postorder_traversal(&e, k)))
        }
        "preorder_skip" => {
            let e = ex(v, 0);
            let cls = pattern(&v.args[1]);
            let mut pt = traversal::preorder_traversal(&e, Keys::None);
            let mut out = Vec::new();
            while let Some(i) = pt.next() {
                if cls.isinstance(&i) {
                    traversal::PreorderTraversal::skip(&mut pt);
                }
                out.push(i);
            }
            list(out)
        }
        "preorder_skip_eq" => {
            let (e, node) = (ex(v, 0), ex(v, 1));
            let mut pt = traversal::preorder_traversal(&e, Keys::None);
            let mut out = Vec::new();
            while let Some(i) = pt.next() {
                if i == node {
                    traversal::PreorderTraversal::skip(&mut pt);
                }
                out.push(i);
            }
            list(out)
        }
        "iterargs" => list(traversal::iterargs(&ex(v, 0)).collect()),
        "iterfreeargs" => list(traversal::iterfreeargs(&ex(v, 0)).collect()),
        "iterfreeargs_first" => Value::Expr(
            traversal::iterfreeargs(&ex(v, 0))
                .next()
                .expect("the expression itself"),
        ),
        "walk" => {
            let e = ex(v, 0);
            let ts = patterns(v, 1);
            list(traversal::walk(&e, &|n: &Expr| {
                ts.iter().any(|t| t.isinstance(n))
            }))
        }
        "as_independent" => {
            let e = ex(v, 0);
            let deps = patterns(v, 1);
            let as_add = v.params.get("as_Add").and_then(Json::as_bool);
            let (i, d) = e.as_independent(&deps, as_add);
            Value::Tuple(vec![Value::Expr(i), Value::Expr(d)])
        }
        "coeff" => {
            let (e, x) = (ex(v, 0), ex(v, 1));
            Value::Expr(e.coeff(&x, v.param_i64("n"), param_bool(v, "right")))
        }
        "is_polynomial" => {
            let e = ex(v, 0);
            Value::fuzzy(e.is_polynomial(&exprs(v, 1)))
        }
        "is_rational_function" => {
            let e = ex(v, 0);
            Value::fuzzy(e.is_rational_function(&exprs(v, 1)))
        }
        _ => v.unknown_op(),
    }
}

#[test]
fn subs_matches_sympy() {
    run_partial("subs", 239, dispatch);
}
