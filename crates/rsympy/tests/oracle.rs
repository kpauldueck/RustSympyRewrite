//! Conformance against SymPy.
//!
//! Loads the golden cases in `oracle/cases/*.json` (written by
//! `oracle/gen_cases.py`), runs each one through rsympy, and reports a score
//! per topic. Run with `cargo test -p rsympy --test oracle -- --nocapture` to
//! see the table.
//!
//! Every case binds one or both rsympy profiles (`profile` is "both",
//! "parity" or "strict"; see `oracle/caselib.py`), and runs once under each
//! profile it binds.
//!
//! The test fails when:
//! - a case gives a wrong answer and is not listed in `oracle/divergences.json`,
//! - a `divergences.json` entry is malformed, names no case, or names a case
//!   that now passes (delete the entry), or
//! - a topic passes fewer cases than recorded in `oracle/baseline.json`.
//!
//! `divergences.json` maps a case id to `{"category", "reason"}`, where the
//! category is `bug` (rsympy is wrong, to be fixed), `intentional` (rsympy
//! differs on purpose) or `sympy_bug` (SymPy's answer is wrong and rsympy
//! does not copy it). An optional `"profile"` limits the entry to one profile.
//!
//! `baseline.json` maps profile -> topic -> passing count. When a topic
//! improves, raise its number so it can't silently regress. Lowering a
//! baseline or adding a divergence needs a human reviewer (docs/PLAN.md).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// The two behaviours rsympy offers on one core (docs/PLAN.md).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Profile {
    /// Reproduces SymPy's observable behaviour.
    Parity,
    /// For students: refuses rather than guesses, reads decimals exactly, etc.
    Strict,
}

impl Profile {
    const ALL: [Profile; 2] = [Profile::Parity, Profile::Strict];

    fn name(self) -> &'static str {
        match self {
            Profile::Parity => "parity",
            Profile::Strict => "strict",
        }
    }

    /// Whether a case whose `profile` field is `field` binds this profile.
    fn binds(self, field: &str) -> bool {
        field == "both" || field == self.name()
    }
}

const CATEGORIES: [&str; 3] = ["bug", "intentional", "sympy_bug"];

/// Outcome of running one case.
enum Outcome {
    Pass,
    Fail(String),
    /// The operation, or the case's compare mode, isn't implemented yet.
    Unsupported,
}

/// What rsympy did with a case's input.
type Answer = Result<String, String>;

/// Runs `op` on `args` and returns rsympy's answer in SymPy's `str()` form,
/// `Err(kind)` if rsympy refused the input (kinds look like
/// `refused:\overline` or `invalid:missing_unit`; see `oracle/caselib.py`),
/// or `None` if the operation isn't implemented yet.
///
/// `case` gives access to `options` and `assumptions`. Every new rsympy
/// feature gets wired in here.
fn dispatch(profile: Profile, op: &str, args: &[&str], case: &Value) -> Option<Answer> {
    let _ = (profile, op, args, case);
    None
}

fn run_case(profile: Profile, case: &Value) -> Outcome {
    let op = case["op"].as_str().expect("case.op");
    let args: Vec<&str> = case["args"]
        .as_array()
        .expect("case.args")
        .iter()
        .map(|a| a.as_str().expect("case.args[i]"))
        .collect();
    let expected = case["expected"].as_str().expect("case.expected");
    let Some(answer) = dispatch(profile, op, &args, case) else {
        return Outcome::Unsupported;
    };
    match (case["compare"].as_str().expect("case.compare"), answer) {
        ("error", Err(kind)) if kind == expected => Outcome::Pass,
        ("error", Err(kind)) => Outcome::Fail(format!("refused with {kind}")),
        ("error", Ok(got)) => Outcome::Fail(format!("accepted, gave {got}")),
        (_, Err(kind)) => Outcome::Fail(format!("refused with {kind}")),
        ("exact", Ok(got)) if got == expected => Outcome::Pass,
        ("exact", Ok(got)) => Outcome::Fail(got),
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

/// Checks every divergence entry's shape; returns the problems found.
fn check_divergences(divergences: &Value, ids: &BTreeMap<String, String>) -> Vec<String> {
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
        match (ids.get(id), entry.get("profile").map(Value::as_str)) {
            (None, _) => problems.push(format!("divergence {id}: no such case")),
            (Some(_), None) => {}
            (Some(bound), Some(Some(p)))
                if Profile::ALL.iter().any(|x| x.name() == p && x.binds(bound)) => {}
            (Some(_), Some(_)) => problems.push(format!(
                "divergence {id}: profile must be one the case binds"
            )),
        }
    }
    problems
}

/// Whether `divergences` excuses a failure of case `id` under `profile`.
fn excused(divergences: &Value, id: &str, profile: Profile) -> bool {
    divergences
        .get(id)
        .is_some_and(|e| e["profile"].as_str().is_none_or(|p| p == profile.name()))
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

    // case id -> the profile field it binds
    let ids: BTreeMap<String, String> = docs
        .iter()
        .flat_map(|d| d["cases"].as_array().expect("cases"))
        .map(|c| {
            let id = c["id"].as_str().expect("case.id").to_owned();
            let profile = c["profile"].as_str().expect("case.profile").to_owned();
            (id, profile)
        })
        .collect();
    let mut problems = check_divergences(&divergences, &ids);

    // (profile, topic) -> (passed, total)
    let mut scores: BTreeMap<(Profile, String), (usize, usize)> = BTreeMap::new();
    for doc in &docs {
        let topic = doc["topic"].as_str().expect("topic");
        for profile in Profile::ALL {
            let (mut passed, mut total) = (0, 0);
            for case in doc["cases"].as_array().expect("cases") {
                if !profile.binds(case["profile"].as_str().expect("case.profile")) {
                    continue;
                }
                total += 1;
                let id = case["id"].as_str().expect("case.id");
                let excused = excused(&divergences, id, profile);
                match run_case(profile, case) {
                    Outcome::Pass if excused => problems.push(format!(
                        "{id} ({}) passes now; delete its divergences.json entry",
                        profile.name()
                    )),
                    Outcome::Pass => passed += 1,
                    Outcome::Unsupported => {}
                    Outcome::Fail(_) if excused => {}
                    Outcome::Fail(got) => problems.push(format!(
                        "{id} ({}): {}({}) gave {got:?}, expected {:?}",
                        profile.name(),
                        case["op"].as_str().unwrap_or_default(),
                        case["args"],
                        case["expected"].as_str().unwrap_or_default(),
                    )),
                }
            }
            scores.insert((profile, topic.to_owned()), (passed, total));
        }
    }

    println!(
        "\n{:<16} {:<7} {:>6} {:>6}",
        "topic", "profile", "pass", "total"
    );
    let mut all: BTreeMap<Profile, (usize, usize)> = BTreeMap::new();
    for ((profile, topic), &(pass, total)) in &scores {
        if total == 0 {
            continue;
        }
        println!("{topic:<16} {:<7} {pass:>6} {total:>6}", profile.name());
        let sum = all.entry(*profile).or_default();
        sum.0 += pass;
        sum.1 += total;
        let floor = baseline[profile.name()][topic.as_str()]
            .as_u64()
            .unwrap_or(0) as usize;
        if pass < floor {
            problems.push(format!(
                "{topic} ({}): {pass} passing, baseline is {floor}",
                profile.name()
            ));
        } else if pass > floor {
            println!("  note: beats baseline ({floor}); raise it in oracle/baseline.json");
        }
    }
    for (profile, (pass, total)) in &all {
        println!("{:<16} {:<7} {pass:>6} {total:>6}", "all", profile.name());
    }

    assert!(
        problems.is_empty(),
        "conformance problems:\n{}",
        problems.join("\n")
    );
}
