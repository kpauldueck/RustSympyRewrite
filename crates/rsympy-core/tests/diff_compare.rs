//! Differential tests against SymPy: replays `oracle/diff/core_compare.json`
//! (canonical orderings, sort keys, `list.sort`, atom printing; see
//! `oracle/gen_diff_compare.py`).

mod common;

use std::cmp::Ordering;

use common::{Value, Vector, run_partial};
use rsympy_core::compare::{self, SortKey};
use rsympy_core::error::{ErrorKind, catch, raise, reraise};
use rsympy_core::exprtools::decompose_power;
use rsympy_core::numbers;
use rsympy_core::printing::str::{FullPrec, StrPrinter, StrSettings, sstr};
use rsympy_core::{Expr, testing};
use rsympy_num::Int;
use serde_json::Value as Json;

/// Python's `repr(float)`.
fn py_float_repr(f: f64) -> String {
    if f.is_nan() {
        return "nan".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "inf" } else { "-inf" }.into();
    }
    if f == 0.0 {
        return if f.is_sign_negative() { "-0.0" } else { "0.0" }.into();
    }
    // Rust's `{:e}` gives the shortest round-trip digits, as repr does.
    let s = format!("{:e}", f.abs());
    let (mant, exp) = s.split_once('e').expect("an exponent");
    let exp: i32 = exp.parse().expect("an int");
    let digits: String = mant.replace('.', "");
    let decpt = exp + 1;
    let body = if decpt <= -4 || decpt > 16 {
        let m = if digits.len() == 1 {
            digits.clone()
        } else {
            format!("{}.{}", &digits[..1], &digits[1..])
        };
        format!(
            "{m}e{}{:02}",
            if exp < 0 { '-' } else { '+' },
            exp.unsigned_abs()
        )
    } else if decpt <= 0 {
        format!("0.{}{digits}", "0".repeat((-decpt) as usize))
    } else if decpt as usize >= digits.len() {
        format!("{digits}{}.0", "0".repeat(decpt as usize - digits.len()))
    } else {
        let d = decpt as usize;
        format!("{}.{}", &digits[..d], &digits[d..])
    };
    if f < 0.0 { format!("-{body}") } else { body }
}

fn key_value(k: &SortKey) -> Value {
    match k {
        SortKey::Int(i) => Value::Int(Int::from(*i)),
        SortKey::Str(s) => Value::Str(s.clone()),
        SortKey::Num(e) => Value::Expr(e.clone()),
        SortKey::Tuple(v) => Value::Tuple(v.iter().map(key_value).collect()),
        SortKey::Bool(b) => Value::Bool(*b),
        SortKey::Float(f) => Value::Float(py_float_repr(*f)),
    }
}

/// Argument `i`: an expression's `enc()` text, parsed here so that
/// building it inside the op can reach unported code.
fn ex(v: &Vector, i: usize) -> Expr {
    testing::parse(v.str(i))
}

/// Argument `i`: a list of `enc()` texts.
fn exprs(v: &Vector, i: usize) -> Vec<Expr> {
    match &v.args[i] {
        Value::List(items) => items
            .iter()
            .map(|a| match a {
                Value::Str(s) => testing::parse(s),
                other => panic!("not an encoded expression: {other:?}"),
            })
            .collect(),
        other => panic!("argument {i} is not a list: {other:?}"),
    }
}

/// The position in `pool` of the first unused item equal to `r`.
fn position(pool: &[Expr], used: &mut [bool], r: &Expr) -> Option<usize> {
    let i = (0..pool.len()).find(|&i| !used[i] && pool[i] == *r)?;
    used[i] = true;
    Some(i)
}

/// The generator's `positions(result, pool)`.
fn positions(result: &[Expr], pool: &[Expr]) -> Value {
    let mut used = vec![false; pool.len()];
    let out: Vec<String> = result
        .iter()
        .map(|r| match position(pool, &mut used, r) {
            Some(i) => i.to_string(),
            None => format!("?{r:?}"),
        })
        .collect();
    Value::Str(out.join(" "))
}

/// The generator's `factor_positions(result, pool)`.
fn factor_positions(result: &[Expr], pool: &[Expr]) -> Value {
    let mut used = vec![false; pool.len()];
    Value::List(
        result
            .iter()
            .map(|r| match position(pool, &mut used, r) {
                Some(i) => Value::Int(Int::from(i)),
                None => Value::Expr(r.clone()),
            })
            .collect(),
    )
}

fn order(v: &Vector) -> Option<&str> {
    v.params.get("order").and_then(Json::as_str)
}

fn reverse(v: &Vector) -> bool {
    v.params["reverse"].as_bool().expect("reverse")
}

fn ordering_int(c: Ordering) -> Value {
    Value::Int(Int::from(match c {
        Ordering::Less => -1i64,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }))
}

/// `try: f() except TypeError: ...`, re-raising anything else.
fn catch_type_error<T>(f: impl FnOnce() -> T) -> Option<T> {
    match catch(f) {
        Ok(v) => Some(v),
        Err(e) if e.kind == ErrorKind::TypeError => None,
        Err(e) => reraise(e),
    }
}

fn compare_matrix(items: &[Expr]) -> String {
    let rows: Vec<String> = items
        .iter()
        .map(|a| {
            items
                .iter()
                .map(
                    |b| match catch_type_error(|| compare::compare_method(a, b)) {
                        None => 'E',
                        Some(Ordering::Less) => '<',
                        Some(Ordering::Equal) => '=',
                        Some(Ordering::Greater) => '>',
                    },
                )
                .collect()
        })
        .collect();
    rows.join(" ")
}

fn key_matrix(items: &[Expr]) -> String {
    let keys: Vec<SortKey> = items.iter().map(|a| compare::sort_key(a, None)).collect();
    let rows: Vec<String> = keys
        .iter()
        .map(|ka| {
            let mut row = String::new();
            for kb in &keys {
                row.push(if compare::key_eq(ka, kb) { 'T' } else { 'F' });
                row.push(match catch_type_error(|| compare::key_lt(ka, kb)) {
                    None => 'E',
                    Some(true) => 'T',
                    Some(false) => 'F',
                });
            }
            row
        })
        .collect();
    rows.join(" ")
}

fn joined(v: &[usize]) -> String {
    v.iter().map(usize::to_string).collect::<Vec<_>>().join(" ")
}

fn py_sort(v: &mut [usize], lt: impl FnMut(&usize, &usize) -> bool, rev: bool) {
    if rev {
        compare::py_sort_by_reverse(v, lt);
    } else {
        compare::py_sort_by(v, lt);
    }
}

fn py_sort_table(v: &Vector) -> Value {
    let values: Vec<usize> = v
        .param_str("values")
        .split_whitespace()
        .map(|s| s.parse().expect("an index"))
        .collect();
    let k = v.param_i64("k") as usize;
    let table = v.param_str("table").as_bytes();
    let mut lst: Vec<usize> = (0..values.len()).collect();
    let outcome = catch_type_error(|| {
        py_sort(
            &mut lst,
            |&i, &j| match table[values[i] * k + values[j]] {
                b'e' => raise(ErrorKind::TypeError, "the table says raise"),
                c => c == b'1',
            },
            reverse(v),
        )
    });
    let status = if outcome.is_some() { "ok" } else { "raised" };
    Value::Str(format!("{status}:{}", joined(&lst)))
}

fn py_sort_natural<T: Ord>(items: &[T], rev: bool) -> Value {
    let mut lst: Vec<usize> = (0..items.len()).collect();
    py_sort(&mut lst, |&i, &j| items[i] < items[j], rev);
    Value::Str(joined(&lst))
}

fn settings(v: &Vector) -> StrSettings {
    let mut s = StrSettings::default();
    let j = &v.params["settings"];
    for (key, val) in j.as_object().expect("settings") {
        match key.as_str() {
            "full_prec" => {
                s.full_prec = match val {
                    Json::Bool(true) => FullPrec::True,
                    Json::Bool(false) => FullPrec::False,
                    Json::String(a) if a == "auto" => FullPrec::Auto,
                    other => panic!("full_prec {other}"),
                }
            }
            "sympy_integers" => s.sympy_integers = val.as_bool().expect("bool"),
            "dps" => s.dps = val.as_i64(),
            "min" => s.min = val.as_i64(),
            "max" => s.max = val.as_i64(),
            other => panic!("unknown setting {other}"),
        }
    }
    s
}

fn str_settings(v: &Vector) -> Value {
    let p = StrPrinter::new(settings(v));
    p.set_print_level(v.param_i64("level") as usize - 1);
    Value::Str(p.print(&ex(v, 0)))
}

fn dispatch(v: &Vector) -> Value {
    match v.op.as_str() {
        "compare" => ordering_int(compare::compare_method(&ex(v, 0), &ex(v, 1))),
        "compare_matrix" => Value::Str(compare_matrix(&exprs(v, 0))),
        "sort_key" => key_value(&compare::sort_key(&ex(v, 0), order(v))),
        "class_key" => {
            let (a, b, name) = compare::class_key(&ex(v, 0));
            Value::Tuple(vec![
                Value::Int(Int::from(a as i64)),
                Value::Int(Int::from(b as i64)),
                Value::Str(name),
            ])
        }
        "key_matrix" => Value::Str(key_matrix(&exprs(v, 0))),
        "sorted_default" => {
            let pool = exprs(v, 0);
            let mut items = pool.clone();
            compare::sorted_default(&mut items);
            positions(&items, &pool)
        }
        "ordered" => {
            let pool = exprs(v, 0);
            positions(&compare::ordered(pool.clone()), &pool)
        }
        "sort_args" => {
            let pool = exprs(v, 0);
            let mut items = pool.clone();
            compare::sort_args(&mut items);
            positions(&items, &pool)
        }
        "as_ordered_terms" => {
            let e = ex(v, 0);
            positions(&compare::as_ordered_terms(&e, order(v)), &e.add_args())
        }
        "as_ordered_factors" => {
            let e = ex(v, 0);
            factor_positions(&compare::as_ordered_factors(&e, order(v)), &e.mul_args())
        }
        "decompose_power" => {
            let (b, e) = decompose_power(&ex(v, 0));
            Value::Tuple(vec![
                Value::Str(testing::encode(&b)),
                Value::Int(Int::from(e)),
            ])
        }
        "number_eq" => Value::Bool(numbers::number_eq(&ex(v, 0), &ex(v, 1))),
        "number_lt" => Value::Bool(numbers::number_lt(&ex(v, 0), &ex(v, 1))),
        "number_gt" => Value::Bool(numbers::number_gt(&ex(v, 0), &ex(v, 1))),
        "complex" => {
            let (re, im) = compare::py_complex(&ex(v, 0));
            Value::Tuple(vec![
                Value::Float(py_float_repr(re)),
                Value::Float(py_float_repr(im)),
            ])
        }
        "float" => Value::Float(py_float_repr(compare::py_float(&ex(v, 0)))),
        "str" => Value::Str(sstr(&ex(v, 0))),
        "str_settings" => str_settings(v),
        "py_sort_table" => py_sort_table(v),
        "py_sort_ints" => {
            let items: Vec<Int> = match &v.args[0] {
                Value::List(l) => l
                    .iter()
                    .map(|a| match a {
                        Value::Int(i) => i.clone(),
                        other => panic!("{other:?}"),
                    })
                    .collect(),
                other => panic!("{other:?}"),
            };
            py_sort_natural(&items, reverse(v))
        }
        "py_sort_strs" => {
            let items: Vec<String> = match &v.args[0] {
                Value::List(l) => l
                    .iter()
                    .map(|a| match a {
                        Value::Str(s) => s.clone(),
                        other => panic!("{other:?}"),
                    })
                    .collect(),
                other => panic!("{other:?}"),
            };
            py_sort_natural(&items, reverse(v))
        }
        _ => v.unknown_op(),
    }
}

#[test]
fn float_repr_matches_python() {
    let cases = [
        (0.5, "0.5"),
        (1.0, "1.0"),
        (1e16, "1e+16"),
        (1234567890123456.0, "1234567890123456.0"),
        (0.0001, "0.0001"),
        (0.00001, "1e-05"),
        (-1.5e-7, "-1.5e-07"),
        (std::f64::consts::PI, "3.141592653589793"),
        (1e300, "1e+300"),
        (123.456, "123.456"),
    ];
    for (f, s) in cases {
        assert_eq!(py_float_repr(f), s);
    }
}

/// 440 vectors reach code that is not ported yet:
/// - 253 `StrictLessThan` / `StrictGreaterThan._eval_relation` (is_lt,
///   is_gt): `<` or `>` between a Number and `oo` / `-oo` goes through
///   `Expr.__lt__` (the `number_lt` / `number_gt` vectors, and sort keys
///   that tie up to such a Number);
/// - 50 `exp.as_base_exp`, 22 `AppliedUndef.__new__`: the sort key of
///   `E**x` builds `exp(x)` with `Function("exp")`;
/// - 36 `evalf`: `complex()` of a non-atom (`sqrt(2)`, `sin(1)`, ...);
/// - 30 `as_real_imag` of a Mul (`complex()` of e.g. `2*I`);
/// - 19 `StrPrinter._print_<Class>` of non-atoms (Function, Relational,
///   Pow, Mul, Add);
/// - 14 `_nodes of a Derivative`, 12 `factor_terms`, 2 `decompose_power`
///   exponents beyond `i64`, and `EulerGamma._as_mpf_val` /
///   `Catalan._as_mpf_val` (one each).
#[test]
fn compare_matches_sympy() {
    run_partial("compare", 440, dispatch);
}
