"""Inventory of the pinned SymPy: size, tests and internal dependencies per module.

    .venv/bin/python oracle/inventory.py > oracle/sympy_inventory.json

For each top-level subpackage of sympy (and each subpackage of the big ones,
listed in SPLIT) it reports non-test source lines, test functions, and which
other modules its non-test code imports. The port order in docs/PLAN.md is
built from this: a module can be ported once everything it imports has been.
"""

import ast
import json
import pathlib
import sys

import sympy

PINNED = "1.14.0"
assert sympy.__version__ == PINNED, f"need sympy=={PINNED}, have {sympy.__version__}"

ROOT = pathlib.Path(sympy.__file__).parent
# Subpackages large enough to plan separately.
SPLIT = {"polys", "physics", "functions", "solvers", "matrices", "printing", "integrals", "parsing", "stats"}


def module_of(rel_parts):
    """Map a path inside sympy/ (as parts) to its planning unit."""
    if len(rel_parts) == 1:
        return "core-misc"  # top-level files like sympy/abc.py, sympy/this.py
    top = rel_parts[0]
    if top in SPLIT and len(rel_parts) > 2 and (ROOT / top / rel_parts[1]).is_dir() and rel_parts[1] != "tests":
        return f"{top}.{rel_parts[1]}"
    return top


def target_of(name, current_pkg_parts):
    """Planning unit for an imported dotted name, or None if outside sympy."""
    parts = name.split(".")
    if parts[0] != "sympy":
        return None
    parts = parts[1:]
    if not parts:
        return "core-misc"
    path = ROOT.joinpath(*parts)
    if path.is_dir() or path.with_suffix(".py").exists():
        rel = parts if path.is_dir() else parts[:-1] + [parts[-1] + ".py"]
        if path.is_dir():
            rel = parts + ["__init__.py"]
        return module_of(rel)
    # `from sympy import foo` style: attribute of a package; walk up.
    while parts and not ROOT.joinpath(*parts).is_dir() and not ROOT.joinpath(*parts).with_suffix(".py").exists():
        parts = parts[:-1]
    return module_of(parts + ["__init__.py"]) if parts else "core-misc"


def imports_in(path, rel_parts):
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    pkg = ["sympy", *rel_parts[:-1]]
    out = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for a in node.names:
                t = target_of(a.name, pkg)
                if t:
                    out.add(t)
        elif isinstance(node, ast.ImportFrom):
            if node.level:
                base = pkg[: len(pkg) - node.level + 1]
                name = ".".join(base + ([node.module] if node.module else []))
            else:
                name = node.module or ""
            t = target_of(name, pkg)
            if t:
                # `from sympy import X` pulls in the root namespace; resolve X when it is a module.
                if name == "sympy":
                    for a in node.names:
                        t2 = target_of(f"sympy.{a.name}", pkg)
                        out.add(t2 if t2 else "core-misc")
                else:
                    out.add(t)
    return out


def main():
    units = {}
    for path in sorted(ROOT.rglob("*.py")):
        rel = path.relative_to(ROOT).parts
        is_test = "tests" in rel
        unit = module_of(rel if not is_test else tuple(p for p in rel if p != "tests") or rel)
        if is_test:
            unit = module_of(rel[: rel.index("tests")] + ("x.py",))
        u = units.setdefault(unit, {"src_lines": 0, "test_lines": 0, "test_functions": 0, "files": 0, "imports": set()})
        text = path.read_text(encoding="utf-8")
        if is_test:
            u["test_lines"] += text.count("\n")
            u["test_functions"] += sum(1 for line in text.splitlines() if line.startswith("def test_"))
        else:
            u["src_lines"] += text.count("\n")
            u["files"] += 1
            try:
                u["imports"] |= imports_in(path, rel)
            except SyntaxError as e:  # pragma: no cover
                print(f"skip {path}: {e}", file=sys.stderr)
    for name, u in units.items():
        u["imports"] = sorted(i for i in u["imports"] if i != name)
    json.dump({"sympy_version": PINNED, "units": dict(sorted(units.items()))}, sys.stdout, indent=1)
    print()


if __name__ == "__main__":
    main()
