//! Replays the oracle's differential vectors (`oracle/diff/core_*.json`,
//! written through `oracle/exprenc.py`; see docs/PORTING.md).
//!
//! A test file loads its area with [`run`] and maps each vector's `op` to
//! the rsympy call that SymPy made:
//!
//! ```ignore
//! common::run("numbers", |v| match v.op.as_str() {
//!     "add" => Value::Expr(ops::add(&v.expr(0), &v.expr(1))),
//!     _ => v.unknown_op(),
//! });
//! ```
//!
//! Each vector starts from `ctx::reset()`. A Python exception becomes
//! `Value::Error(class name)`; `NotPortedYet` is always a failure.

#![allow(dead_code)]

use std::panic::AssertUnwindSafe;
use std::path::Path;

use rsympy_core::{Expr, catch, ctx, testing};
use rsympy_num::Int;
use serde_json::Value as Json;

/// A decoded value from a vector (see `oracle/exprenc.py`).
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Expr(Expr),
    Bool(bool),
    None,
    Int(Int),
    /// `repr(f)` of a Python float.
    Float(String),
    Str(String),
    Tuple(Vec<Value>),
    List(Vec<Value>),
    Dict(Vec<(Value, Value)>),
    /// The Python exception class raised.
    Error(String),
}

impl Value {
    /// `Some(b)`, `None` as Python's `True`/`False`/`None`.
    pub fn fuzzy(v: Option<bool>) -> Value {
        match v {
            Some(b) => Value::Bool(b),
            None => Value::None,
        }
    }
}

pub struct Vector {
    pub op: String,
    pub args: Vec<Value>,
    /// Extra settings the generator recorded (`vector(..., **params)`).
    pub params: Json,
    out: Json,
}

impl Vector {
    /// Argument `i` as an expression; a Python int becomes `Integer(i)`
    /// (what `_sympify` does with it).
    pub fn expr(&self, i: usize) -> Expr {
        match &self.args[i] {
            Value::Expr(e) => e.clone(),
            Value::Int(n) => Expr::integer(n.clone()),
            other => panic!(
                "argument {i} of {} is not an expression: {other:?}",
                self.op
            ),
        }
    }

    pub fn int(&self, i: usize) -> Int {
        match &self.args[i] {
            Value::Int(n) => n.clone(),
            other => panic!("argument {i} of {} is not an int: {other:?}", self.op),
        }
    }

    pub fn str(&self, i: usize) -> &str {
        match &self.args[i] {
            Value::Str(s) => s,
            other => panic!("argument {i} of {} is not a str: {other:?}", self.op),
        }
    }

    pub fn param_i64(&self, key: &str) -> i64 {
        self.params[key]
            .as_i64()
            .unwrap_or_else(|| panic!("vector {} has no integer param {key}", self.op))
    }

    pub fn param_str(&self, key: &str) -> &str {
        self.params[key]
            .as_str()
            .unwrap_or_else(|| panic!("vector {} has no string param {key}", self.op))
    }

    pub fn unknown_op(&self) -> Value {
        panic!("no Rust dispatch for op {:?}", self.op)
    }
}

fn decode(j: &Json) -> Value {
    let obj = j.as_object().expect("a tagged value");
    let (tag, v) = obj.iter().next().expect("one tag");
    match tag.as_str() {
        "expr" => Value::Expr(testing::parse(v.as_str().expect("expr text"))),
        "bool" => Value::Bool(v.as_bool().expect("bool")),
        "none" => Value::None,
        "int" => Value::Int(v.as_str().expect("int text").parse().expect("an int")),
        "float" => Value::Float(v.as_str().expect("float repr").to_string()),
        "str" => Value::Str(v.as_str().expect("str").to_string()),
        "tuple" => Value::Tuple(v.as_array().expect("list").iter().map(decode).collect()),
        "list" => Value::List(v.as_array().expect("list").iter().map(decode).collect()),
        "dict" => Value::Dict(
            v.as_array()
                .expect("pairs")
                .iter()
                .map(|kv| (decode(&kv[0]), decode(&kv[1])))
                .collect(),
        ),
        "error" => Value::Error(v.as_str().expect("exception name").to_string()),
        _ => panic!("unknown value tag {tag}"),
    }
}

/// Loads `oracle/diff/core_<area>.json`.
pub fn load(area: &str) -> Vec<Json> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../oracle/diff")
        .join(format!("core_{area}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}; run oracle/gen_diff_{area}.py", path.display()));
    let body: Json = serde_json::from_str(&text).expect("valid JSON");
    body["vectors"].as_array().expect("vectors").clone()
}

/// Replays every vector of `area` through `f` and fails listing the
/// vectors whose outcome differs from SymPy's, including any that reach
/// code not ported yet.
pub fn run(area: &str, f: impl Fn(&Vector) -> Value) {
    run_partial(area, 0, f);
}

/// Like [`run`], but exactly `not_ported` vectors are expected to reach
/// unported code (`NotPortedYet`). A wrong answer always fails. The count
/// is a ratchet: when porting makes more vectors pass, the test fails
/// until the number is lowered to match.
pub fn run_partial(area: &str, not_ported: usize, f: impl Fn(&Vector) -> Value) {
    let vectors = load(area);
    let mut failures = Vec::new();
    let mut unported = Vec::new();
    for (i, j) in vectors.iter().enumerate() {
        ctx::reset();
        let decoded = catch(|| {
            let args = j["args"]
                .as_array()
                .expect("args")
                .iter()
                .map(decode)
                .collect();
            let v = Vector {
                op: j["op"].as_str().expect("op").to_string(),
                args,
                params: j.get("params").cloned().unwrap_or(Json::Null),
                out: j["out"].clone(),
            };
            let expected = decode(&v.out);
            (v, expected)
        });
        let (v, expected) = match decoded {
            Ok(x) => x,
            Err(e) => {
                failures.push(format!("#{i}: cannot decode: {e}"));
                continue;
            }
        };
        let got = match std::panic::catch_unwind(AssertUnwindSafe(|| catch(|| f(&v)))) {
            Ok(Ok(got)) => got,
            Ok(Err(e)) if e.kind == rsympy_core::ErrorKind::NotPortedYet => {
                unported.push(format!("#{i} {}: not ported: {}", v.op, e.message));
                continue;
            }
            Ok(Err(e)) => Value::Error(e.kind.python_name().to_string()),
            Err(payload) => {
                let msg = payload
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| payload.downcast_ref::<&str>().copied())
                    .unwrap_or("?");
                failures.push(format!("#{i} {}({:?}): Rust panic: {msg}", v.op, v.args));
                continue;
            }
        };
        if got != expected {
            failures.push(format!(
                "#{i} {}({:?}):\n    expected {expected:?}\n         got {got:?}",
                v.op, v.args
            ));
        }
    }
    if unported.len() != not_ported {
        failures.extend(unported.iter().cloned());
        failures.push(format!(
            "expected exactly {not_ported} vectors to reach unported code, but {} did; \
             update the count passed to run_partial",
            unported.len()
        ));
    }
    if !failures.is_empty() {
        // RSYMPY_DIFF_ALL=1 lists every failure instead of the first 30.
        let limit = if std::env::var_os("RSYMPY_DIFF_ALL").is_some() {
            usize::MAX
        } else {
            30
        };
        let mut by_kind: Vec<(String, usize)> = Vec::new();
        for f in &failures {
            let kind = f.split(['(', ':']).next().unwrap_or("").trim().to_string();
            let kind = kind
                .split_once(' ')
                .map_or(kind.clone(), |(_, op)| op.to_string());
            match by_kind.iter_mut().find(|(k, _)| *k == kind) {
                Some((_, n)) => *n += 1,
                None => by_kind.push((kind, 1)),
            }
        }
        let summary: Vec<_> = by_kind.iter().map(|(k, n)| format!("{k}: {n}")).collect();
        let shown: Vec<_> = failures.iter().take(limit).cloned().collect();
        panic!(
            "{} of {} vectors in core_{area}.json differ from SymPy ({}):\n{}",
            failures.len(),
            vectors.len(),
            summary.join(", "),
            shown.join("\n")
        );
    }
    eprintln!(
        "core_{area}.json: {} of {} vectors match SymPy, {} not ported yet",
        vectors.len() - unported.len(),
        vectors.len(),
        unported.len()
    );
}
