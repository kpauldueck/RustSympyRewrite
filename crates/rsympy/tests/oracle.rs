//! Conformance against SymPy.
//!
//! Loads the golden cases in `oracle/cases/*.json` (written by
//! `oracle/gen_cases.py`), runs each one through rsympy, and reports a score
//! per topic. Run with `cargo test -p rsympy --test oracle -- --nocapture` to
//! see the table.
//!
//! rsympy copies SymPy 1:1, so every case's expected answer is what SymPy
//! itself returned.
//!
//! The test fails when:
//! - a case gives a wrong answer and is not listed in `oracle/divergences.json`,
//! - a `divergences.json` entry is malformed, names no case, or names a case
//!   that now passes (delete the entry), or
//! - a topic passes fewer cases than recorded in `oracle/baseline.json`.
//!
//! `divergences.json` maps a case id to `{"category", "reason"}`. The
//! category is `bug` (rsympy is wrong; fix it), `unstable` (SymPy's own
//! answer depends on hash order or call history, so there is no single
//! answer to copy) or `unportable` (the behaviour can't exist outside
//! Python, e.g. `parse_expr` evaluating arbitrary Python code).
//!
//! `baseline.json` maps topic -> passing count. When a topic improves, raise
//! its number so it can't silently regress. Lowering a baseline or adding a
//! divergence needs a human reviewer (docs/PLAN.md).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use rsympy::ntheory::{FactorOpts, NtCtx, factorint};
use rsympy::num::Int;
use serde_json::Value;

const CATEGORIES: [&str; 3] = ["bug", "unstable", "unportable"];

/// Outcome of running one case.
enum Outcome {
    Pass,
    Fail(String),
    /// The operation isn't implemented yet.
    Unsupported,
}

/// What rsympy did with a case's input.
type Answer = Result<String, String>;

/// Runs `op` on `args` and returns rsympy's answer in SymPy's `str()` form,
/// `Err(class)` if rsympy raised an error (named by the Python exception
/// class SymPy raises, e.g. `ValueError`), or `None` if the operation isn't
/// implemented yet.
///
/// `case` gives access to `options` and `assumptions`; `session` is the
/// state SymPy carries from call to call within a topic. Every new rsympy
/// feature gets wired in here.
fn dispatch(op: &str, args: &[&str], case: &Value, session: &mut Session) -> Option<Answer> {
    let _ = case;
    match (op, args) {
        // Until the parser lands (Tier 2), only integer literals, for which
        // `sympify` gives an `Integer`.
        ("gcd", [a, b]) => Some(Ok(int_literal(a)?.gcd(&int_literal(b)?).to_string())),
        ("lcm", [a, b]) => Some(Ok(int_literal(a)?.lcm(&int_literal(b)?).to_string())),
        ("factorint", [n]) => {
            let f = factorint(&mut session.nt, &int_literal(n)?, &FactorOpts::default());
            // The generator prints `Dict(factorint(n))`, whose keys print
            // sorted.
            let items: Vec<String> = f
                .sorted()
                .iter()
                .map(|(p, e)| format!("{p}: {e}"))
                .collect();
            Some(Ok(format!("{{{}}}", items.join(", "))))
        }
        _ => None,
    }
}

/// What SymPy's global state holds while a topic runs. The generator resets
/// it before each topic (`fresh_state` in `oracle/gen_cases.py`), and so does
/// the runner.
struct Session {
    nt: NtCtx,
}

impl Session {
    fn fresh() -> Session {
        Session {
            nt: NtCtx::default(),
        }
    }
}

/// `s` as an integer, if it is a plain decimal literal.
fn int_literal(s: &str) -> Option<Int> {
    let t = s.trim();
    let digits = t.strip_prefix('-').unwrap_or(t);
    if digits.is_empty() || !digits.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    t.parse().ok()
}

fn run_case(case: &Value, session: &mut Session) -> Outcome {
    let op = case["op"].as_str().expect("case.op");
    let args: Vec<&str> = case["args"]
        .as_array()
        .expect("case.args")
        .iter()
        .map(|a| a.as_str().expect("case.args[i]"))
        .collect();
    let expected = case["expected"].as_str().expect("case.expected");
    let Some(answer) = dispatch(op, &args, case, session) else {
        return Outcome::Unsupported;
    };
    // rsympy copies SymPy 1:1: the printed answer must match exactly, or the
    // same exception class must be raised.
    match (case["compare"].as_str().expect("case.compare"), answer) {
        ("error", Err(class)) if class == expected => Outcome::Pass,
        ("error", Err(class)) => Outcome::Fail(format!("raised {class}")),
        ("error", Ok(got)) => Outcome::Fail(format!("returned {got}, SymPy raises {expected}")),
        ("exact", Err(class)) => Outcome::Fail(format!("raised {class}")),
        ("exact", Ok(got)) if got == expected => Outcome::Pass,
        ("exact", Ok(got)) => Outcome::Fail(got),
        (other, _) => panic!("unknown compare mode {other:?}"),
    }
}

fn oracle_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../oracle")
}

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Checks every divergence entry's shape; returns the problems found.
fn check_divergences(divergences: &Value, ids: &BTreeSet<String>) -> Vec<String> {
    let mut problems = Vec::new();
    let Some(entries) = divergences.as_object() else {
        return vec!["divergences.json must be an object".into()];
    };
    for (id, entry) in entries {
        let category = entry["category"].as_str().unwrap_or_default();
        if !CATEGORIES.contains(&category) {
            problems.push(format!(
                "divergence {id}: category must be one of {CATEGORIES:?}"
            ));
        }
        if entry["reason"].as_str().is_none_or(|r| r.trim().is_empty()) {
            problems.push(format!("divergence {id}: needs a reason"));
        }
        if !ids.contains(id) {
            problems.push(format!("divergence {id}: no such case"));
        }
    }
    problems
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
    let docs: Vec<Value> = files.iter().map(|f| read_json(f)).collect();

    let ids: BTreeSet<String> = docs
        .iter()
        .flat_map(|d| d["cases"].as_array().expect("cases"))
        .map(|c| c["id"].as_str().expect("case.id").to_owned())
        .collect();
    let mut problems = check_divergences(&divergences, &ids);

    // topic -> (passed, total)
    let mut scores: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for doc in &docs {
        let topic = doc["topic"].as_str().expect("topic");
        let cases = doc["cases"].as_array().expect("cases");
        let mut passed = 0;
        let mut session = Session::fresh();
        for case in cases {
            let id = case["id"].as_str().expect("case.id");
            let excused = divergences.get(id).is_some();
            match run_case(case, &mut session) {
                Outcome::Pass if excused => problems.push(format!(
                    "{id} passes now; delete its divergences.json entry"
                )),
                Outcome::Pass => passed += 1,
                Outcome::Unsupported => {}
                Outcome::Fail(_) if excused => {}
                Outcome::Fail(got) => problems.push(format!(
                    "{id}: {}({}) gave {got:?}, expected {:?}",
                    case["op"].as_str().unwrap_or_default(),
                    case["args"],
                    case["expected"].as_str().unwrap_or_default(),
                )),
            }
        }
        scores.insert(topic.to_owned(), (passed, cases.len()));
    }

    println!("\n{:<16} {:>6} {:>6}", "topic", "pass", "total");
    let (mut all_pass, mut all_total) = (0, 0);
    for (topic, &(pass, total)) in &scores {
        println!("{topic:<16} {pass:>6} {total:>6}");
        all_pass += pass;
        all_total += total;
        let floor = baseline[topic.as_str()].as_u64().unwrap_or(0) as usize;
        if pass < floor {
            problems.push(format!("{topic}: {pass} passing, baseline is {floor}"));
        } else if pass > floor {
            println!("  note: {topic} beats baseline ({floor}); raise it in oracle/baseline.json");
        }
    }
    println!("{:<16} {all_pass:>6} {all_total:>6}", "all");

    assert!(
        problems.is_empty(),
        "conformance problems:\n{}",
        problems.join("\n")
    );
}
