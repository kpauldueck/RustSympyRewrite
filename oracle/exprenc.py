"""Shared helpers for rsympy-core's differential vector generators.

A generator `oracle/gen_diff_<area>.py` builds vectors with `vector(...)`
and writes them with `write(area, vectors)` to `oracle/diff/core_<area>.json`;
`crates/rsympy-core/tests/diff_<area>.rs` replays them (see
`crates/rsympy-core/tests/common/mod.rs` and docs/PORTING.md).

Values are tagged JSON:

    {"expr": "<enc(e)>"}   a SymPy object
    {"bool": true}         {"none": null}
    {"int": "123"}         a Python int (as a string: it may be big)
    {"str": "..."}         {"float": "<repr(f)>"}
    {"tuple": [...]}       {"list": [...]}
    {"dict": [[k, v], ...]}  in iteration order
    {"error": "TypeError"} the exception class SymPy raised

`enc(e)` is `srepr(e, order='none')` (args in storage order, not printing
order) with undefined functions' assumptions spelled out, so that
`dec(enc(e)) == e` holds; `enc` checks this for every value it encodes.
"""

import json
import pathlib
import random

import sympy as sp
from sympy.core.function import AppliedUndef, UndefinedFunction
from sympy.external.gmpy import GROUND_TYPES
from sympy.printing.repr import ReprPrinter

assert sp.__version__ == "1.14.0", sp.__version__
assert GROUND_TYPES == "python", "set SYMPY_GROUND_TYPES=python"

DIFF = pathlib.Path(__file__).parent / "diff"


def _mpmath_constant_memos():
    """The memo holders of mpmath's @constant_memo constants (pi, e, ln2,
    euler, ...): the decorated function, reached through the wrapper's
    closure."""
    import mpmath.libmp as libmp

    found = {}
    for mod in (libmp.libelefun, libmp.gammazeta):
        for name in dir(mod):
            g = getattr(mod, name)
            for cell in getattr(g, "__closure__", None) or ():
                f = cell.cell_contents
                if callable(f) and hasattr(f, "memo_prec"):
                    found[id(f)] = f
    return list(found.values())


_MEMOS = _mpmath_constant_memos()


def fresh_state():
    """Reset everything that makes SymPy's answers depend on history: its
    caches, the factorint cache, all random streams, and mpmath's memoized
    constants (a constant memoized at a higher precision is truncated, not
    recomputed, so its last bits depend on what ran before)."""
    sp.core.cache.clear_cache()
    from sympy.core.random import seed
    from sympy.ntheory import factor_

    factor_.factor_cache.clear()
    random.seed(0)
    seed(0)
    for f in _MEMOS:
        f.memo_prec = -1
        f.memo_val = None


class _Encoder(ReprPrinter):
    def _print_UndefinedFunction(self, expr):
        kw = "".join(f", {k}={v!r}" for k, v in sorted(expr._kwargs.items()))
        return f"Function({expr.__name__!r}{kw})"


def _namespace():
    ns = {}
    exec("from sympy import *", ns)
    from sympy.core.symbol import Str
    from sympy.core.relational import (Equality, GreaterThan, LessThan,
                                       StrictGreaterThan, StrictLessThan,
                                       Unequality)
    from sympy.functions.elementary.piecewise import ExprCondPair

    ns.update(Str=Str, Equality=Equality, Unequality=Unequality,
              StrictLessThan=StrictLessThan, LessThan=LessThan,
              StrictGreaterThan=StrictGreaterThan, GreaterThan=GreaterThan,
              ExprCondPair=ExprCondPair, true=sp.true, false=sp.false,
              EmptySet=sp.S.EmptySet)
    return ns


_NS = _namespace()


def _raw(cls):
    """`cls(*args)` without evaluation, as rsympy's testing::parse builds
    it: `_from_args` for Add and Mul, `Pow(b, e, evaluate=False)`, and
    `Basic.__new__` for everything else."""
    if cls in (sp.Add, sp.Mul):
        return lambda *args: cls._from_args(args)
    if cls is sp.Pow:
        return lambda b, e: sp.Pow(b, e, evaluate=False)
    return lambda *args: sp.Basic.__new__(cls, *args)


_RAW_NS = {
    name: _raw(obj)
    if isinstance(obj, type) and issubclass(obj, sp.Basic)
    and not issubclass(obj, sp.Atom) and obj is not sp.Function else obj
    for name, obj in _NS.items()
}


def dec(text):
    """Rebuilds an object from `enc` output without evaluating it."""
    return eval(text, dict(_RAW_NS))


def enc(e):
    """The lossless encoding of a SymPy object (see the module docs)."""
    text = _Encoder({"order": "none"}).doprint(e)
    back = dec(text)
    if back != e or type(back) != type(e):
        raise AssertionError(f"encoding does not round-trip: {text}")
    return text


def value(v):
    """Tags a Python value for JSON."""
    if isinstance(v, bool):
        return {"bool": v}
    if v is None:
        return {"none": None}
    if isinstance(v, int):
        return {"int": str(v)}
    if isinstance(v, float):
        return {"float": repr(v)}
    if isinstance(v, str):
        return {"str": v}
    if isinstance(v, (sp.Basic, UndefinedFunction)) and not isinstance(v, type):
        return {"expr": enc(v)}
    if isinstance(v, tuple):
        return {"tuple": [value(x) for x in v]}
    if isinstance(v, list):
        return {"list": [value(x) for x in v]}
    if isinstance(v, dict):
        return {"dict": [[value(k), value(x)] for k, x in v.items()]}
    raise TypeError(f"no encoding for {type(v).__name__}: {v!r}")


def vector(op, args, fn, **params):
    """Runs `fn(*args)` from a fresh state and records the outcome.

    `args` are SymPy objects or plain values; `params` are extra JSON-able
    settings the Rust side needs (e.g. a precision), recorded verbatim.
    """
    vec = {"op": op, "args": [value(a) for a in args]}
    if params:
        vec["params"] = params
    fresh_state()
    # Call fn on fresh copies, as the Rust side does: SymPy objects cache
    # the facts they work out (obj._assumptions), so reusing an object
    # would make this answer depend on earlier vectors.
    args = [dec(enc(a)) if isinstance(a, sp.Basic) else a for a in args]
    try:
        out = fn(*args)
    except Exception as exc:  # noqa: BLE001: the exception is the answer
        vec["out"] = {"error": type(exc).__name__}
    else:
        vec["out"] = value(out)
    return vec


def write(area, vectors, doc):
    """Writes oracle/diff/core_<area>.json; `doc` says what it tests."""
    path = DIFF / f"core_{area}.json"
    body = {"doc": doc, "sympy": sp.__version__, "vectors": vectors}
    path.write_text(json.dumps(body, indent=1, ensure_ascii=False) + "\n")
    print(f"wrote {path} ({len(vectors)} vectors)")


__all__ = ["dec", "enc", "fresh_state", "value", "vector", "write",
           "AppliedUndef"]
