//! Differential tests against SymPy: replays `oracle/diff/ntheory.json`
//! (written by `oracle/gen_diff.py`).

use std::path::Path;

use rsympy_ntheory::{
    FactorOpts, NtCtx, factorint, integer_nthroot, isprime, multiplicity, nextprime, perfect_power,
};
use rsympy_num::Int;
use serde_json::Value;

fn load() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../oracle/diff/ntheory.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).expect("valid JSON")
}

fn int(v: &Value) -> Int {
    v.as_str().expect("string").parse().expect("integer")
}

#[test]
fn primality() {
    let doc = load();
    for row in doc["isprime"].as_array().unwrap() {
        let n = int(&row[0]);
        assert_eq!(isprime(&n), row[1].as_bool().unwrap(), "isprime({n})");
    }
    for row in doc["nextprime"].as_array().unwrap() {
        let n = int(&row[0]);
        assert_eq!(nextprime(&n), int(&row[1]), "nextprime({n})");
    }
}

#[test]
fn powers_and_roots() {
    let doc = load();
    let mut nt = NtCtx::default();
    for row in doc["perfect_power"].as_array().unwrap() {
        let n = int(&row[0]);
        let want = (!row[1].is_null()).then(|| (int(&row[1][0]), row[1][1].as_u64().unwrap()));
        assert_eq!(perfect_power(&mut nt, &n), want, "perfect_power({n})");
    }
    for row in doc["multiplicity"].as_array().unwrap() {
        let (p, n) = (int(&row[0]), int(&row[1]));
        assert_eq!(
            multiplicity(&p, &n).unwrap(),
            row[2].as_u64().unwrap(),
            "multiplicity({p}, {n})"
        );
    }
    for row in doc["integer_nthroot"].as_array().unwrap() {
        let (y, n) = (int(&row[0]), row[1].as_u64().unwrap());
        let (r, exact) = integer_nthroot(&y, n).unwrap();
        assert_eq!(
            (r, exact),
            (int(&row[2]), row[3].as_bool().unwrap()),
            "integer_nthroot({y}, {n})"
        );
    }
}

/// The calls run in order on one context, as SymPy ran them: the cache and
/// random stream carry over.
#[test]
fn factorint_sequence() {
    let doc = load();
    let mut nt = NtCtx::default();
    for row in doc["factorint"].as_array().unwrap() {
        let n = int(&row["n"]);
        let opts = match row["limit"].as_str() {
            Some(l) => FactorOpts::with_limit(l.parse().unwrap()),
            None => FactorOpts::default(),
        };
        let got: Vec<(String, u64)> = factorint(&mut nt, &n, &opts)
            .iter()
            .map(|(p, e)| (p.to_string(), e))
            .collect();
        let want: Vec<(String, u64)> = row["factors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| (f[0].as_str().unwrap().to_string(), f[1].as_u64().unwrap()))
            .collect();
        assert_eq!(got, want, "factorint({n}, limit={:?})", opts.limit);
    }
}

#[test]
fn squares_and_jacobi() {
    let doc = load();
    for row in doc["is_square"].as_array().unwrap() {
        let n = int(&row[0]);
        assert_eq!(
            rsympy_ntheory::is_square(&n),
            row[1].as_bool().unwrap(),
            "is_square({n})"
        );
    }
    for row in doc["jacobi"].as_array().unwrap() {
        let (x, y) = (int(&row[0]), int(&row[1]));
        let want = row[2].as_i64().unwrap();
        assert_eq!(
            i64::from(rsympy_ntheory::jacobi(&x, &y).unwrap()),
            want,
            "jacobi({x}, {y})"
        );
    }
}
