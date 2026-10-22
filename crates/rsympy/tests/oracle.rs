//! Conformance against SymPy.
//!
//! Loads the golden cases in `oracle/cases/*.json` (written by
//! `oracle/gen_cases.py`), runs each one through rsympy, and reports a score
//! per topic. Run with `cargo test -p rsympy --test oracle -- --nocapture` to
//! see the table.
//!
//! The test fails when:
//! - a case gives a wrong answer and is not listed in `oracle/divergences.json`
//!   (case id -> reason), or
//! - a topic passes fewer cases than recorded in `oracle/baseline.json`.
//!
//! When a topic improves, raise its number in `baseline.json` so it can't
//! silently regress.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// Outcome of running one case.
enum Outcome {
    Pass,
    Fail(String),
    /// The operation, or the case's compare mode, isn't implemented yet.
    Unsupported,
}

/// Runs `op` on `args` and returns rsympy's answer in SymPy's `str()` form,
/// or `None` if the operation isn't implemented yet.
///
/// Every new rsympy feature gets wired in here.
fn dispatch(op: &str, args: &[&str], _assumptions: Option<&Value>) -> Option<String> {
    let _ = (op, args);
    None
}

fn run_case(case: &Value) -> Outcome {
    let op = case["op"].as_str().expect("case.op");
    let args: Vec<&str> = case["args"]
        .as_array()
        .expect("case.args")
        .iter()
        .map(|a| a.as_str().expect("case.args[i]"))
        .collect();
    let expected = case["expected"].as_str().expect("case.expected");
    let Some(got) = dispatch(op, &args, case.get("assumptions")) else {
        return Outcome::Unsupported;
    };
    match case["compare"].as_str().expect("case.compare") {
        "exact" if got == expected => Outcome::Pass,
        "exact" => Outcome::Fail(got),
        // `equivalent`, `set` and `antiderivative` need rsympy itself to do
        // the comparison; they light up once the core exists.
        _ => Outcome::Unsupported,
    }
}

fn oracle_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../oracle")
}

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn sympy_conformance() {
    let dir = oracle_dir();
    let baseline = read_json(&dir.join("baseline.json"));
    let divergences = read_json(&dir.join("divergences.json"));

    let mut files: Vec<PathBuf> = fs::read_dir(dir.join("cases"))
        .expect("oracle/cases")
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "no golden cases; run oracle/gen_cases.py"
    );

    let mut scores: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut problems = Vec::new();

    for file in &files {
        let doc = read_json(file);
        let topic = doc["topic"].as_str().expect("topic").to_owned();
        let cases = doc["cases"].as_array().expect("cases");
        let mut passed = 0;
        for case in cases {
            let id = case["id"].as_str().expect("case.id");
            match run_case(case) {
                Outcome::Pass => passed += 1,
                Outcome::Unsupported => {}
                Outcome::Fail(_) if divergences.get(id).is_some() => {}
                Outcome::Fail(got) => problems.push(format!(
                    "{id}: {}({}) gave {got:?}, SymPy gives {:?}",
                    case["op"].as_str().unwrap_or_default(),
                    case["args"],
                    case["expected"].as_str().unwrap_or_default(),
                )),
            }
        }
        scores.insert(topic, (passed, cases.len()));
    }

    println!("\n{:<12} {:>6} {:>6}", "topic", "pass", "total");
    let (mut all_pass, mut all_total) = (0, 0);
    for (topic, &(pass, total)) in &scores {
        println!("{topic:<12} {pass:>6} {total:>6}");
        all_pass += pass;
        all_total += total;
        let floor = baseline.get(topic).and_then(Value::as_u64).unwrap_or(0) as usize;
        if pass < floor {
            problems.push(format!("{topic}: {pass} passing, baseline is {floor}"));
        } else if pass > floor {
            println!("  note: {topic} beats baseline ({floor}); raise it in oracle/baseline.json");
        }
    }
    println!("{:<12} {all_pass:>6} {all_total:>6}", "all");

    assert!(
        problems.is_empty(),
        "conformance problems:\n{}",
        problems.join("\n")
    );
}
