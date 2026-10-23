//! Differential tests against mpmath: replays `oracle/diff/mpf.json`
//! (written by `oracle/gen_diff.py`). Everything runs in one test, in the
//! generator's order, because mpmath memoizes constants across calls.

use std::path::Path;

use rsympy_mpf::*;
use rsympy_num::Int;
use serde_json::Value;

fn load() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../oracle/diff/mpf.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).expect("valid JSON")
}

fn mpf(v: &Value) -> Mpf {
    Mpf {
        sign: v[0].as_u64().unwrap() as u8,
        man: v[1].as_str().unwrap().parse().unwrap(),
        exp: v[2].as_i64().unwrap(),
        bc: v[3].as_i64().unwrap(),
    }
}

fn rnd(v: &Value) -> Rnd {
    match v.as_str().unwrap() {
        "n" => Rnd::Nearest,
        "f" => Rnd::Floor,
        "c" => Rnd::Ceiling,
        "u" => Rnd::Up,
        "d" => Rnd::Down,
        other => panic!("rounding mode {other}"),
    }
}

/// An expected raw mpf, or the name of the exception mpmath raised.
fn check(got: Result<Mpf, MpfError>, want: &Value, what: &str) {
    match (got, want.as_str()) {
        (Err(e), Some(class)) => assert_eq!(e.class(), class, "{what}"),
        (Ok(m), Some(class)) => panic!("{what}: got {m:?}, mpmath raised {class}"),
        (Err(e), None) => panic!("{what}: raised {e}, mpmath gave {want}"),
        (Ok(m), None) => assert_eq!(m, mpf(want), "{what}"),
    }
}

/// `repr(float)`, in Python's layout.
fn py_float_repr(f: f64) -> String {
    if f.is_nan() {
        return "nan".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if f == 0.0 {
        return if f.is_sign_negative() {
            "-0.0".into()
        } else {
            "0.0".into()
        };
    }
    let r = format!("{f:e}");
    let (mant, exp) = r.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let (sign, mant) = mant.strip_prefix('-').map_or(("", mant), |m| ("-", m));
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    if (-4..16).contains(&exp) {
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

fn arithmetic(doc: &Value) {
    for row in doc["ops"].as_array().unwrap() {
        let (s, t) = (mpf(&row["s"]), mpf(&row["t"]));
        let prec = row["prec"].as_i64().unwrap();
        let r = rnd(&row["rnd"]);
        let ctx = format!("s={s:?} t={t:?} prec={prec} rnd={r:?}");
        check(
            Ok(mpf_add(&s, &t, prec, r)),
            &row["add"],
            &format!("add {ctx}"),
        );
        check(
            Ok(mpf_sub(&s, &t, prec, r)),
            &row["sub"],
            &format!("sub {ctx}"),
        );
        if !row["add_exact"].is_null() {
            check(
                Ok(mpf_add(&s, &t, 0, ROUND_FAST)),
                &row["add_exact"],
                &format!("add exact {ctx}"),
            );
        }
        check(
            Ok(mpf_mul(&s, &t, prec, r)),
            &row["mul"],
            &format!("mul {ctx}"),
        );
        check(mpf_div(&s, &t, prec, r), &row["div"], &format!("div {ctx}"));
        assert_eq!(
            i64::from(mpf_cmp(&s, &t)),
            row["cmp"].as_i64().unwrap(),
            "cmp {ctx}"
        );
        assert_eq!(mpf_eq(&s, &t), row["eq"].as_bool().unwrap(), "eq {ctx}");
        assert_eq!(mpf_lt(&s, &t), row["lt"].as_bool().unwrap(), "lt {ctx}");
        check(mpf_sqrt(&s, prec, r), &row["sqrt"], &format!("sqrt {ctx}"));
        check(Ok(mpf_pos(&s, prec, r)), &row["pos"], &format!("pos {ctx}"));
        check(
            Ok(mpf_floor(&s, 0, ROUND_FAST)),
            &row["floor"],
            &format!("floor {ctx}"),
        );
        check(
            Ok(mpf_ceil(&s, 0, ROUND_FAST)),
            &row["ceil"],
            &format!("ceil {ctx}"),
        );
        check(
            Ok(mpf_nint(&s, 0, ROUND_FAST)),
            &row["nint"],
            &format!("nint {ctx}"),
        );
        assert_eq!(mpf_hash(&s), row["hash"].as_i64().unwrap(), "hash {ctx}");
        let n = Int::from(row["pow_n"].as_i64().unwrap());
        check(
            mpf_pow_int(&s, &n, prec, r),
            &row["pow"],
            &format!("pow {n} {ctx}"),
        );
        if let Some(m) = row.get("mod") {
            check(mpf_mod(&s, &t, prec, r), m, &format!("mod {ctx}"));
        }
        if let Some(v) = row.get("to_int") {
            assert_eq!(
                to_int(&s, None).unwrap().to_string(),
                v.as_str().unwrap(),
                "to_int {ctx}"
            );
            let v = row["to_int_rnd"].as_str().unwrap();
            assert_eq!(
                to_int(&s, Some(r)).unwrap().to_string(),
                v,
                "to_int rnd {ctx}"
            );
        }
    }
}

fn strings(doc: &Value) {
    for row in doc["to_str"].as_array().unwrap() {
        let s = mpf(&row["s"]);
        let dps = row["dps"].as_i64().unwrap();
        let ctx = format!("s={s:?} dps={dps}");
        assert_eq!(
            to_str(&s, dps, true, None, None, false),
            row["str"].as_str().unwrap(),
            "to_str {ctx}"
        );
        assert_eq!(
            to_str(&s, dps, false, None, None, false),
            row["str_nostrip"].as_str().unwrap(),
            "to_str nostrip {ctx}"
        );
        let f = to_float(&s, false, ROUND_FAST).unwrap();
        assert_eq!(
            py_float_repr(f),
            row["float"].as_str().unwrap(),
            "to_float {ctx}"
        );
    }
    for row in doc["from_str"].as_array().unwrap() {
        let lit = row[0].as_str().unwrap();
        let prec = row[1].as_i64().unwrap();
        let r = rnd(&row[2]);
        check(
            from_str(lit, prec, r),
            &row[3],
            &format!("from_str({lit:?}, {prec}, {r:?})"),
        );
    }
    for row in doc["from_float"].as_array().unwrap() {
        let f: f64 = row[0].as_str().unwrap().parse().unwrap();
        let prec = row[1].as_i64().unwrap();
        check(
            Ok(from_float(f, prec, Rnd::Nearest)),
            &row[2],
            &format!("from_float({f:e}, {prec})"),
        );
    }
    for row in doc["prec_dps"].as_array().unwrap() {
        let n = row[0].as_i64().unwrap();
        assert_eq!(prec_to_dps(n), row[1].as_i64().unwrap(), "prec_to_dps({n})");
        assert_eq!(dps_to_prec(n), row[2].as_i64().unwrap(), "dps_to_prec({n})");
        assert_eq!(repr_dps(n), row[3].as_i64().unwrap(), "repr_dps({n})");
    }
}

fn constants(doc: &Value) {
    for row in doc["constants"].as_array().unwrap() {
        let name = row[0].as_str().unwrap();
        let prec = row[1].as_i64().unwrap();
        let r = rnd(&row[2]);
        let got = match name {
            "pi" => mpf_pi(prec, r),
            "e" => mpf_e(prec, r),
            "ln2" => mpf_ln2(prec, r),
            "ln10" => mpf_ln10(prec, r),
            "phi" => mpf_phi(prec, r),
            "degree" => mpf_degree(prec, r),
            other => panic!("constant {other}"),
        };
        check(Ok(got), &row[3], &format!("{name}({prec}, {r:?})"));
    }
}

#[test]
fn mpmath_vectors_in_order() {
    let doc = load();
    clear_constant_cache();
    arithmetic(&doc);
    strings(&doc);
    constants(&doc);
}
