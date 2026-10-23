#!/usr/bin/env bash
# Local CI: every check the project gates on, in one command.
#
#   ./ci.sh                 run everything (Rust checks, then the oracle)
#   ./ci.sh rust            formatting, lints and tests only
#   ./ci.sh oracle          regenerate the answer key and check it is unchanged
#   ./ci.sh install-hook    run ./ci.sh automatically before every `git push`
#
# Needs: a Rust toolchain with rustfmt and clippy (https://rustup.rs), and
# Python 3.13. The first oracle run creates .venv/ with the pinned SymPy;
# set PYTHON=/path/to/python to use an existing environment instead.
#
# The oracle step reruns every generator (oracle/gen_*.py), which rewrite
# oracle/cases/, oracle/diff/ and the crates' generated*.rs from live SymPy,
# under two hash seeds, and fails if either run changes a committed file.
# Those files are generated, never hand-edited, so nothing is lost.

set -euo pipefail
cd "$(dirname "$0")"

step() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
fail() { printf '\n\033[31mFAILED: %s\033[0m\n' "$*" >&2; exit 1; }

rust_checks() {
    command -v cargo >/dev/null || fail "cargo not found; install Rust from https://rustup.rs"
    cargo fmt --version >/dev/null 2>&1 || fail "rustfmt missing; run: rustup component add rustfmt"
    cargo clippy --version >/dev/null 2>&1 || fail "clippy missing; run: rustup component add clippy"

    step "cargo fmt --check"
    cargo fmt --all --check || fail "formatting; run: cargo fmt --all"

    step "cargo clippy"
    cargo clippy --workspace --all-targets -- -D warnings || fail "clippy warnings"

    step "cargo test"
    cargo test --workspace || fail "tests"
}

python_env() {
    if [[ -n "${PYTHON:-}" ]]; then
        echo "$PYTHON"
        return
    fi
    if [[ ! -x .venv/bin/python ]]; then
        local py
        py="$(command -v python3.13 || command -v python3 || true)"
        [[ -n "$py" ]] || fail "Python 3.13 not found"
        step "creating .venv with the pinned SymPy" >&2
        "$py" -m venv .venv >&2
        .venv/bin/pip install --quiet -r oracle/requirements.txt >&2
    fi
    echo .venv/bin/python
}

oracle_checks() {
    local py
    py="$(python_env)"
    "$py" -c 'import sys; assert sys.version_info[:2] == (3, 13), sys.version' \
        || fail "the oracle needs Python 3.13 (set PYTHON=/path/to/python3.13)"
    export SYMPY_GROUND_TYPES=python
    for seed in 0 1; do
        step "regenerating the answer key (PYTHONHASHSEED=$seed)"
        PYTHONHASHSEED=$seed "$py" oracle/gen_cases.py
        PYTHONHASHSEED=$seed "$py" oracle/gen_diff.py
        PYTHONHASHSEED=$seed "$py" oracle/gen_core_tables.py
        for gen in oracle/gen_diff_*.py oracle/gen_tables_*.py; do
            [[ -e "$gen" ]] || continue
            PYTHONHASHSEED=$seed "$py" "$gen"
        done
        local generated=(oracle/cases oracle/diff ':(glob)crates/**/generated*.rs')
        git diff --exit-code --stat -- "${generated[@]}" \
            || fail "regenerated answers differ from the committed ones (seed $seed); see git diff"
        if [[ -n "$(git ls-files --others --exclude-standard -- "${generated[@]}")" ]]; then
            git ls-files --others --exclude-standard -- "${generated[@]}"
            fail "generators wrote files that are not committed (seed $seed)"
        fi
    done
}

install_hook() {
    local hook
    hook="$(git rev-parse --git-path hooks)/pre-push"
    printf '#!/bin/sh\n# Installed by ./ci.sh install-hook\nexec ./ci.sh\n' > "$hook"
    chmod +x "$hook"
    echo "Installed $hook: ./ci.sh now runs before every git push (skip once with git push --no-verify)."
}

case "${1:-all}" in
    all) rust_checks; oracle_checks ;;
    rust) rust_checks ;;
    oracle) oracle_checks ;;
    install-hook) install_hook; exit 0 ;;
    *) fail "unknown step '$1' (use: all, rust, oracle, install-hook)" ;;
esac

printf '\n\033[32mAll checks passed.\033[0m\n'
