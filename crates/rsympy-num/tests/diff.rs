//! Differential tests against CPython: replays `oracle/diff/num.json`
//! (written by `oracle/gen_diff.py`).

use std::path::Path;

use rsympy_num::{Int, PyRandom, Rational};
use serde_json::Value;

fn load() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../oracle/diff/num.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).expect("valid JSON")
}

fn int(v: &Value) -> Int {
    v.as_str().expect("string").parse().expect("integer")
}

fn s(v: &Value) -> &str {
    v.as_str().expect("string")
}

/// `repr(float)`: Rust's shortest round-trip digits, in Python's layout.
fn py_float_repr(f: f64) -> String {
    if f.is_infinite() {
        return if f > 0.0 { "inf".into() } else { "-inf".into() };
    }
    let r = format!("{f:e}");
    let (mant, exp) = r.split_once('e').expect("exponent");
    let exp: i32 = exp.parse().expect("int exponent");
    let (sign, mant) = mant.strip_prefix('-').map_or(("", mant), |m| ("-", m));
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    if (-5..16).contains(&exp) {
        let point = exp + 1;
        if point <= 0 {
            format!("{sign}0.{}{digits}", "0".repeat((-point) as usize))
        } else if point as usize >= digits.len() {
            format!(
                "{sign}{digits}{}.0",
                "0".repeat(point as usize - digits.len())
            )
        } else {
            let (a, b) = digits.split_at(point as usize);
            format!("{sign}{a}.{b}")
        }
    } else {
        let (a, b) = digits.split_at(1);
        let m = if b.is_empty() {
            a.to_string()
        } else {
            format!("{a}.{b}")
        };
        format!(
            "{sign}{m}e{}{:02}",
            if exp < 0 { '-' } else { '+' },
            exp.abs()
        )
    }
}

#[test]
fn int_binary_ops() {
    let doc = load();
    for row in doc["binops"].as_array().unwrap() {
        let (a, b) = (int(&row["a"]), int(&row["b"]));
        let ctx = format!("a={a} b={b}");
        assert_eq!((&a + &b).to_string(), s(&row["add"]), "{ctx}");
        assert_eq!((&a - &b).to_string(), s(&row["sub"]), "{ctx}");
        assert_eq!((&a * &b).to_string(), s(&row["mul"]), "{ctx}");
        assert_eq!((&a & &b).to_string(), s(&row["and"]), "{ctx}");
        assert_eq!(a.gcd(&b).to_string(), s(&row["gcd"]), "{ctx}");
        assert_eq!(a.lcm(&b).to_string(), s(&row["lcm"]), "{ctx}");
        assert_eq!(a.cmp(&b) as i64, row["cmp"].as_i64().unwrap(), "{ctx}");
        if b.is_zero() {
            assert!(a.floor_div(&b).is_err());
            continue;
        }
        assert_eq!(
            a.floor_div(&b).unwrap().to_string(),
            s(&row["floordiv"]),
            "{ctx}"
        );
        assert_eq!(a.py_mod(&b).unwrap().to_string(), s(&row["mod"]), "{ctx}");
        let e = int(&row["powmod_e"]);
        assert_eq!(
            a.pow_mod(&e, &b).unwrap().to_string(),
            s(&row["powmod"]),
            "{ctx} e={e}"
        );
    }
}

#[test]
fn int_unary_ops() {
    let doc = load();
    for row in doc["unops"].as_array().unwrap() {
        let a = int(&row["a"]);
        let ctx = format!("a={a}");
        assert_eq!((-&a).to_string(), s(&row["neg"]), "{ctx}");
        assert_eq!(a.abs().to_string(), s(&row["abs"]), "{ctx}");
        assert_eq!(a.bit_length(), row["bit_length"].as_u64().unwrap(), "{ctx}");
        let k = row["shift"].as_u64().unwrap();
        assert_eq!((&a << k).to_string(), s(&row["shl"]), "{ctx} k={k}");
        assert_eq!((&a >> k).to_string(), s(&row["shr"]), "{ctx} k={k}");
        let f = a
            .to_f64()
            .map_or("OverflowError".to_string(), py_float_repr);
        assert_eq!(f, s(&row["float"]), "{ctx}");
        assert_eq!(
            a.abs().isqrt().unwrap().to_string(),
            s(&row["isqrt"]),
            "{ctx}"
        );
        let n = row["root_n"].as_u64().unwrap();
        let r = a.abs().nth_root_floor(n);
        assert_eq!(r.to_string(), s(&row["root"]), "{ctx} n={n}");
        assert_eq!(
            r.pow(n) == a.abs(),
            row["root_exact"].as_bool().unwrap(),
            "{ctx} n={n}"
        );
        let e = row["pow_e"].as_u64().unwrap();
        assert_eq!(a.pow(e).to_string(), s(&row["pow"]), "{ctx} e={e}");
    }
}

#[test]
fn fractions() {
    let doc = load();
    for row in doc["fractions"].as_array().unwrap() {
        let f = Rational::new(int(&row["p"]), int(&row["q"])).unwrap();
        let g = Rational::new(int(&row["r"]), int(&row["s"])).unwrap();
        let ctx = format!("f={f} g={g}");
        assert_eq!(f.to_string(), s(&row["f"]), "{ctx}");
        assert_eq!((&f + &g).to_string(), s(&row["add"]), "{ctx}");
        assert_eq!((&f - &g).to_string(), s(&row["sub"]), "{ctx}");
        assert_eq!((&f * &g).to_string(), s(&row["mul"]), "{ctx}");
        let div = f
            .div(&g)
            .map_or("ZeroDivisionError".to_string(), |q| q.to_string());
        assert_eq!(div, s(&row["div"]), "{ctx}");
        assert_eq!(f.floor().to_string(), s(&row["floor"]), "{ctx}");
        assert_eq!(f.ceil().to_string(), s(&row["ceil"]), "{ctx}");
        let fl = f.to_f64();
        let fl = if fl.is_infinite() {
            "OverflowError".to_string()
        } else {
            py_float_repr(fl)
        };
        assert_eq!(fl, s(&row["float"]), "{ctx}");
        assert_eq!(f.cmp(&g) as i64, row["cmp"].as_i64().unwrap(), "{ctx}");
    }
}

#[test]
fn mersenne_twister() {
    let doc = load();
    for row in doc["random"].as_array().unwrap() {
        let seed = int(&row["seed"]);
        let mut r = PyRandom::new(&seed);
        for d in row["draws"].as_array().unwrap() {
            let got = match s(&d[0]) {
                "getrandbits" => r.getrandbits(d[1].as_u64().unwrap()).to_string(),
                "randint" => r
                    .randint(&int(&d[1][0]), &int(&d[1][1]))
                    .unwrap()
                    .to_string(),
                "random" => py_float_repr(r.random()),
                other => panic!("unknown draw {other}"),
            };
            assert_eq!(got, s(&d[2]), "seed={seed} draw={d}");
        }
    }
}
