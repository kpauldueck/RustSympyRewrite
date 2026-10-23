//! Differential tests against SymPy: replays `oracle/diff/core_exprtools.json`
//! (`factor_terms`, `gcd_terms`, `Factors`, `Term`, `_monotonic_sign`,
//! `_mask_nc` and the content/primitive methods; see
//! `oracle/gen_diff_exprtools.py`).

mod common;

use common::{Value, Vector, run_partial};
use rsympy_core::exprtools::{self, Exponent, Factors, Term};
use rsympy_core::py::PyDict;
use rsympy_core::{Expr, testing};

fn flag(v: &Vector, key: &str) -> bool {
    v.params[key]
        .as_bool()
        .unwrap_or_else(|| panic!("vector {} has no bool param {key}", v.op))
}

fn expr_of(v: &Value) -> Expr {
    match v {
        Value::Expr(e) => e.clone(),
        Value::Int(i) => Expr::integer(i.clone()),
        other => panic!("not an expression: {other:?}"),
    }
}

fn exprs_of(v: &Value) -> Vec<Expr> {
    match v {
        Value::List(items) | Value::Tuple(items) => items.iter().map(expr_of).collect(),
        other => panic!("not a sequence: {other:?}"),
    }
}

fn pair((a, b): (Expr, Expr)) -> Value {
    Value::Tuple(vec![Value::Expr(a), Value::Expr(b)])
}

fn triple((a, b, c): (Expr, Expr, Expr)) -> Value {
    Value::Tuple(vec![Value::Expr(a), Value::Expr(b), Value::Expr(c)])
}

fn opt(e: Option<Expr>) -> Value {
    e.map_or(Value::None, Value::Expr)
}

fn exponent_value(x: &Exponent) -> Value {
    match x {
        Exponent::Int(i) => Value::Int(i.clone()),
        Exponent::Expr(e) => Value::Expr(e.clone()),
    }
}

fn dict_value(d: &PyDict<Expr, Exponent>) -> Value {
    Value::Dict(
        d.iter()
            .map(|(k, x)| (Value::Expr(k.clone()), exponent_value(x)))
            .collect(),
    )
}

/// A Factors as its dict.
fn fval(f: &Factors) -> Value {
    dict_value(&f.factors)
}

/// A Term as `(coeff, numer, denom)`.
fn tval(t: &Term) -> Value {
    Value::Tuple(vec![
        Value::Expr(t.coeff.clone()),
        fval(&t.numer),
        fval(&t.denom),
    ])
}

/// A dict argument (`{expr: int-or-expr}`).
fn dict_arg(v: &Value) -> PyDict<Expr, Exponent> {
    let Value::Dict(items) = v else {
        panic!("not a dict: {v:?}")
    };
    items
        .iter()
        .map(|(k, x)| {
            let x = match x {
                Value::Int(i) => Exponent::Int(i.clone()),
                Value::Expr(e) => Exponent::Expr(e.clone()),
                other => panic!("not an exponent: {other:?}"),
            };
            (expr_of(k), x)
        })
        .collect()
}

/// `Factors(arg)` for a dict or an expression argument.
fn factors_arg(v: &Value) -> Factors {
    match v {
        Value::Dict(_) => Factors::from_dict(&dict_arg(v), false),
        _ => Factors::from_expr(&expr_of(v)),
    }
}

/// The generator's `render`: text for a result.
enum R {
    E(Expr),
    None,
    Tuple(Vec<R>),
    List(Vec<R>),
    Dict(Vec<(R, R)>),
}

fn render(r: &R) -> String {
    match r {
        R::E(e) => testing::encode(e),
        R::None => "None".into(),
        R::Tuple(v) => format!("({})", v.iter().map(render).collect::<Vec<_>>().join(", ")),
        R::List(v) => format!("[{}]", v.iter().map(render).collect::<Vec<_>>().join(", ")),
        R::Dict(v) => format!(
            "{{{}}}",
            v.iter()
                .map(|(k, x)| format!("{}: {}", render(k), render(x)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// The generator's `canon`: the Dummies of `text` renumbered by first
/// appearance, with the automatic names `Dummy_<count>` written `_`.
fn canon(text: &str) -> String {
    const OPEN: &str = "Dummy('";
    const INDEX: &str = ", dummy_index=";
    let mut out = String::new();
    let mut seen: Vec<(String, String)> = Vec::new();
    let mut rest = text;
    while let Some(pos) = rest.find(OPEN) {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + OPEN.len() - 1..]; // from the opening quote
        let bytes = after.as_bytes();
        let mut i = 1;
        while bytes[i] != b'\'' {
            if bytes[i] == b'\\' {
                i += 1;
            }
            i += 1;
        }
        let name = &after[..=i];
        let tail = &after[i + 1..];
        let at = tail.find(INDEX).expect("a Dummy has its dummy_index");
        let assumptions = &tail[..at];
        let digits = &tail[at + INDEX.len()..];
        let end = digits.find(')').expect("closing paren");
        let key = (name.to_string(), digits[..end].to_string());
        let k = match seen.iter().position(|s| *s == key) {
            Some(k) => k,
            None => {
                seen.push(key);
                seen.len() - 1
            }
        };
        let auto = name.len() > 8
            && name.starts_with("'Dummy_")
            && name[7..name.len() - 1].bytes().all(|c| c.is_ascii_digit());
        let shown = if auto { "'_'" } else { name };
        out.push_str(&format!("Dummy({shown}{assumptions}, dummy_index={k})"));
        rest = &digits[end + 1..];
    }
    out.push_str(rest);
    out
}

fn canon_value(r: R) -> Value {
    Value::Str(canon(&render(&r)))
}

fn factors_op(name: &str, fa: &Factors, other: &Value, as_expr: bool) -> Value {
    let pair_of = |(a, b): (Factors, Factors)| Value::Tuple(vec![fval(&a), fval(&b)]);
    if as_expr {
        let w = expr_of(other);
        return match name {
            "mul" => fval(&fa.mul_expr(&w)),
            "normal" => pair_of(fa.normal_expr(&w)),
            "div" => pair_of(fa.div_expr(&w)),
            "quo" => fval(&fa.quo_expr(&w)),
            "rem" => fval(&fa.rem_expr(&w)),
            "gcd" => fval(&fa.gcd_expr(&w)),
            "lcm" => fval(&fa.lcm_expr(&w)),
            "eq" => Value::Bool(fa.py_eq_expr(&w)),
            _ => panic!("no Factors op {name}"),
        };
    }
    let mut fb = factors_arg(other);
    match name {
        "mul" => fval(&fa.mul(&fb)),
        "normal" => pair_of(fa.normal(&mut fb)),
        "div" => pair_of(fa.div(&fb)),
        "quo" => fval(&fa.quo(&fb)),
        "rem" => fval(&fa.rem(&fb)),
        "gcd" => fval(&fa.gcd(&fb)),
        "lcm" => fval(&fa.lcm(&fb)),
        "eq" => Value::Bool(fa.py_eq(&fb)),
        _ => panic!("no Factors op {name}"),
    }
}

/// `Term(coeff, Factors(numer), Factors(denom))` from the generator's
/// `Term_parts_*` arguments.
fn term_parts(v: &Vector) -> Term {
    Term::from_parts(
        &v.expr(0),
        Some(Factors::from_dict(&dict_arg(&v.args[1]), false)),
        Some(Factors::from_dict(&dict_arg(&v.args[2]), false)),
    )
}

fn dispatch(v: &Vector) -> Value {
    let e = || v.expr(0);
    let op = v.op.as_str();
    if let Some(name) = op.strip_prefix("Factors_") {
        let (name, as_expr) = match name.strip_suffix("_expr") {
            Some(n) if n != "as" && n != "dict_as" && n != "pow" => (n, true),
            _ => (name, false),
        };
        if ["mul", "normal", "div", "quo", "rem", "gcd", "lcm", "eq"].contains(&name) {
            return factors_op(name, &factors_arg(&v.args[0]), &v.args[1], as_expr);
        }
    }
    if let Some(name) = op.strip_prefix("Term_")
        && ["mul", "quo", "gcd", "lcm", "eq"].contains(&name)
    {
        let (ta, tb) = (Term::new(&v.expr(0)), Term::new(&v.expr(1)));
        return match name {
            "mul" => tval(&ta.mul(&tb)),
            "quo" => tval(&ta.quo(&tb)),
            "gcd" => tval(&ta.gcd(&tb)),
            "lcm" => tval(&ta.lcm(&tb)),
            _ => Value::Bool(ta.py_eq(&tb)),
        };
    }
    match op {
        "factor_terms" => Value::Expr(exprtools::factor_terms(
            &e(),
            flag(v, "radical"),
            flag(v, "clear"),
            flag(v, "fraction"),
            flag(v, "sign"),
        )),
        "factor_terms_seq" => {
            let seq = exprs_of(&v.args[0]);
            if v.param_str("kind") == "list" {
                // a list stays a list
                Value::List(
                    exprtools::factor_terms_seq(&seq, false, false, false, true)
                        .into_iter()
                        .map(Value::Expr)
                        .collect(),
                )
            } else {
                // sympify turns a tuple into a Tuple
                let t = rsympy_core::construct::raw(rsympy_core::ClassId::Tuple, seq);
                Value::Expr(exprtools::factor_terms(&t, false, false, false, true))
            }
        }
        "gcd_terms" => Value::Expr(exprtools::gcd_terms(
            &e(),
            flag(v, "isprimitive"),
            flag(v, "clear"),
            flag(v, "fraction"),
        )),
        "gcd_terms_seq" => Value::Expr(exprtools::gcd_terms_seq(
            &exprs_of(&v.args[0]),
            flag(v, "isprimitive"),
            flag(v, "clear"),
            flag(v, "fraction"),
        )),
        "_gcd_terms" => triple(exprtools::_gcd_terms(
            &e(),
            flag(v, "isprimitive"),
            flag(v, "fraction"),
        )),
        "_gcd_terms_seq" => triple(exprtools::_gcd_terms_seq(
            &exprs_of(&v.args[0]),
            false,
            true,
        )),
        "as_content_primitive" => {
            pair(e().as_content_primitive(flag(v, "radical"), flag(v, "clear")))
        }
        "primitive" => pair(e().primitive()),
        "as_powers_dict" => dict_value(&e().as_powers_dict()),
        "extract_multiplicatively" => opt(e().extract_multiplicatively(&v.expr(1))),
        "extract_additively" => opt(e().extract_additively(&v.expr(1))),
        "Factors" => fval(&Factors::from_expr(&e())),
        "Factors_as_expr" => Value::Expr(Factors::from_expr(&e()).as_expr()),
        "Factors_flags" => {
            let f = Factors::from_expr(&e());
            Value::Tuple(vec![Value::Bool(f.is_zero()), Value::Bool(f.is_one())])
        }
        "Factors_dict" => fval(&Factors::from_dict(&dict_arg(&v.args[0]), false)),
        "Factors_dict_as_expr" => {
            Value::Expr(Factors::from_dict(&dict_arg(&v.args[0]), false).as_expr())
        }
        "Factors_pow" => fval(&factors_arg(&v.args[0]).pow(&v.int(1))),
        "Factors_pow_factors" => {
            fval(&factors_arg(&v.args[0]).pow_factors(&Factors::from_expr(&v.expr(1))))
        }
        "Factors_pow_expr" => fval(&factors_arg(&v.args[0]).pow_expr(&v.expr(1))),
        "Factors_normal_term" | "Factors_normal_term_denom" => {
            let (ta, tb) = (Term::new(&v.expr(0)), Term::new(&v.expr(1)));
            let (fa, mut fb) = if op == "Factors_normal_term" {
                (ta.numer, tb.denom)
            } else {
                (ta.denom, tb.numer)
            };
            let (r0, r1) = fa.normal(&mut fb);
            Value::Tuple(vec![fval(&r0), fval(&r1), fval(&fb)])
        }
        "Term" => tval(&Term::new(&e())),
        "Term_as_expr" => Value::Expr(Term::new(&e()).as_expr()),
        "Term_inv" => tval(&Term::new(&e()).inv()),
        "Term_pow" => tval(&Term::new(&e()).pow(&v.int(1))),
        "Term_parts_mul" => tval(&term_parts(v).mul(&Term::new(&v.expr(3)))),
        "Term_parts_gcd" => tval(&term_parts(v).gcd(&Term::new(&v.expr(3)))),
        "Term_parts_eq" => Value::Bool(term_parts(v).py_eq(&Term::new(&v.expr(3)))),
        "decompose_power" => {
            let (b, x) = exprtools::decompose_power_int(&e());
            Value::Tuple(vec![Value::Expr(b), Value::Int(x)])
        }
        "decompose_power_rat" => pair(exprtools::decompose_power_rat(&e())),
        "_monotonic_sign" => canon_value(exprtools::monotonic_sign(&e()).map_or(R::None, R::E)),
        "_mask_nc" => {
            let name = v.params["name"].as_str();
            let (ex, reps, syms) = exprtools::mask_nc(&e(), name);
            canon_value(R::Tuple(vec![
                R::E(ex),
                R::Dict(reps.into_iter().map(|(k, x)| (R::E(k), R::E(x))).collect()),
                R::List(syms.into_iter().map(R::E).collect()),
            ]))
        }
        "factor_nc" => Value::Expr(exprtools::factor_nc(&e())),
        _ => v.unknown_op(),
    }
}

#[test]
fn canon_renumbers_dummies() {
    assert_eq!(
        canon(
            "Add(Dummy('pos', positive=True, dummy_index=1000004), Dummy('Dummy_17', dummy_index=17), Dummy('pos', positive=True, dummy_index=1000004))"
        ),
        "Add(Dummy('pos', positive=True, dummy_index=0), Dummy('_', dummy_index=1), Dummy('pos', positive=True, dummy_index=0))"
    );
}

// Unported: exp.as_base_exp and Function.__new__ (the functions unit),
// xreplace, subs, as_independent, coeff and is_polynomial with free symbols
// (the subs unit), Tuple and Interval rebuilding, the evalf sign of an Add,
// real_roots for _monotonic_sign, the polys gcd of Floats, and factor_nc of
// an Add (expand_power_exp, polys factor).
#[test]
fn exprtools_matches_sympy() {
    run_partial("exprtools", 1033, dispatch);
}
