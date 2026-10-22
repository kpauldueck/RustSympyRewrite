"""Building blocks for golden cases. See gen_cases.py for how they are run.

A case is one JSON object:

  id              "<topic>-<nnn>", assigned by gen_cases.py
  op              operation name, e.g. "factor", "parse_latex", "convert_to"
  args            list of strings; SymPy syntax unless the op says otherwise
                  (parse_latex takes LaTeX, parse_number takes raw text)
  options         optional dict of op options (vocabulary, tolerance, ...)
  assumptions     optional {symbol: {assumption: bool}}, taken from the
                  Symbol objects in args (strings carry none)
  expected        the answer as SymPy's str(), or for compare="error" the
                  error kind (see below)
  expected_latex  SymPy's latex() of the answer, when it is an expression
  compare         exact | equivalent | set | antiderivative | error
  source          "sympy": SymPy computed `expected` (the default)
                  "spec":   we decided `expected` on purpose, because SymPy is
                            wrong for students, has no such operation, or the
                            homework app needs different behaviour. Spec cases
                            carry `note` (why) and, where SymPy has a
                            comparable call, `sympy_gives` (what it does).

Error kinds for compare="error" are "refused:<thing>", where <thing> is the
LaTeX command or construct the parser refuses by name (e.g. "refused:\\overline",
"refused:derivative"), or "invalid:<reason>" for inputs that are wrong rather
than unsupported (e.g. "invalid:dimension_mismatch").
"""

import sympy as sp


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


def case(op, args, result, compare="exact", *, options=None, note=None):
    """A case whose expected answer SymPy computed (`result`)."""
    out = {"op": op, "args": [_text(a) for a in args]}
    if options:
        out["options"] = options
    assumptions = _assumptions(args)
    if assumptions:
        out["assumptions"] = assumptions
    out["expected"] = _text(result)
    if _has_latex(result):
        out["expected_latex"] = sp.latex(result)
    out["compare"] = compare
    out["source"] = "sympy"
    if note:
        out["note"] = note
    return out


def spec_case(op, args, expected, note, compare="exact", *, options=None, sympy_gives=None):
    """A case whose expected answer we chose on purpose.

    `expected` is SymPy syntax (canonicalised through sympify so it prints the
    way SymPy would), a plain str for compare="error", or a Python
    bool/int/list. `sympy_gives` is a zero-argument callable that runs SymPy's
    nearest equivalent; its result (or exception) is recorded for reference.
    """
    if compare == "error" or not isinstance(expected, str):
        exp_text = _text(expected)
        exp_latex = None
    else:
        parsed = sp.sympify(expected)
        exp_text = sp.sstr(parsed)
        exp_latex = sp.latex(parsed)
    out = {"op": op, "args": [_text(a) for a in args]}
    if options:
        out["options"] = options
    assumptions = _assumptions(args)
    if assumptions:
        out["assumptions"] = assumptions
    out["expected"] = exp_text
    if exp_latex is not None:
        out["expected_latex"] = exp_latex
    out["compare"] = compare
    out["source"] = "spec"
    out["note"] = note
    if sympy_gives is not None:
        out["sympy_gives"] = run_sympy(sympy_gives)
    return out


def run_sympy(fn):
    """Run a SymPy call, returning its str() or 'error: <Type>: <message>'."""
    try:
        return _text(fn())
    except Exception as e:  # noqa: BLE001 - recording SymPy's behaviour
        msg = " ".join(str(e).split())[:120]
        return f"error: {type(e).__name__}: {msg}"
