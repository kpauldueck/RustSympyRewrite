"""Building blocks for golden cases. See gen_cases.py for how they are run.

rsympy copies SymPy 1:1 (docs/PLAN.md), so every expected answer is whatever
the pinned SymPy actually returns, computed live by the generator. Nothing is
hand-written, and nothing is "what SymPy should have said".

A case is one JSON object:

  id              "<topic>-<nnn>", assigned by gen_cases.py
  op              the SymPy call: a function or method name such as "factor",
                  "parse_latex" or "convert_to", or, for source "app", the
                  name of an app recipe documented in its topic module
  args            list of strings; SymPy syntax unless the op says otherwise
                  (parse_latex takes LaTeX, sympify takes raw text)
  options         optional dict of the call's keyword arguments
                  (e.g. {"backend": "lark"})
  assumptions     optional {symbol: {assumption: bool}}, taken from the
                  Symbol objects in args (strings carry none)
  expected        the answer as SymPy's str(); for compare="error", the class
                  name of the exception SymPy raises (e.g. "ValueError")
  expected_latex  SymPy's latex() of the answer, when it is an expression
  compare         "exact" (str() must match SymPy's) or "error" (same
                  exception class)
  source          "sympy": one SymPy call (the default)
                  "app":   a fixed recipe of SymPy calls that the homework app
                           runs (e.g. its answer-equivalence check). The answer
                           is still SymPy's; the case checks the app's logic
                           gives the same result on rsympy.
  note            optional; why the case is interesting
"""

import sympy as sp

SOURCES = ("sympy", "app")


def _symbols_in(arg):
    if isinstance(arg, (sp.Basic, sp.MatrixBase)):
        return arg.free_symbols
    if isinstance(arg, (list, tuple)):
        return set().union(*(_symbols_in(a) for a in arg)) if arg else set()
    return set()


def _assumptions(args):
    out = {}
    for s in sorted(_symbols_in(args), key=str):
        orig = getattr(s, "_assumptions_orig", None) or {}
        if orig:
            out[str(s)] = {k: bool(v) for k, v in sorted(orig.items())}
    return out


def _text(value):
    if isinstance(value, str):
        return value
    if isinstance(value, sp.MatrixBase):
        # sstr() pretty-prints matrices over several lines; keep one line.
        return str(value)
    return sp.sstr(value)


def _has_latex(value):
    if isinstance(value, sp.MatrixBase):
        return True
    return isinstance(value, sp.Basic) and not isinstance(value, sp.logic.boolalg.BooleanAtom)


def case(op, args, result, *, options=None, note=None, source="sympy"):
    """A case whose expected answer is `result`, which SymPy just computed."""
    if source not in SOURCES:
        raise ValueError(f"unknown source {source!r}")
    out = {"op": op, "args": [_text(a) for a in args]}
    if options:
        out["options"] = options
    assumptions = _assumptions(args)
    if assumptions:
        out["assumptions"] = assumptions
    out["expected"] = _text(result)
    if _has_latex(result):
        out["expected_latex"] = sp.latex(result)
    out["compare"] = "exact"
    out["source"] = source
    if note:
        out["note"] = note
    return out


def outcome(op, args, fn, *, options=None, note=None, source="sympy"):
    """Run the SymPy call `fn` (no arguments) and record what it does.

    If it returns, this is `case(op, args, fn(), ...)`. If it raises,
    the case has compare="error" and expected = the exception's class name, so
    rsympy must fail the same way.
    """
    try:
        result = fn()
    except Exception as e:  # noqa: BLE001 - recording SymPy's behaviour
        out = case(op, args, "", options=options, note=note, source=source)
        out.pop("expected_latex", None)
        out["expected"] = type(e).__name__
        out["compare"] = "error"
        return out
    return case(op, args, result, options=options, note=note, source=source)
