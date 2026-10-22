"""Parsing student LaTeX (op "parse_latex").

args[0] is the LaTeX a student typed. options may carry "vocabulary" (a list of
enabled vocabularies: "geometry", "vectors", "sequences") and "assumptions".

SymPy-sourced cases come from SymPy's Lark LaTeX parser, the backend the
homework app uses. Spec cases implement the semantics decided in the brief's
conventions.md: never silently misread, refuse unknown constructs by name,
typed decimals are exact, `e` is Euler's number, a superscript binds to the
atom before it, and an unparenthesised function argument runs over the
following implicit product. Every spec case records what the Lark parser
returns in `sympy_gives`.
"""

import sympy as sp
from lark import Tree
from sympy.parsing.latex import parse_latex

from caselib import case, spec_case

TOPIC = "latex_parse"
OP = "parse_latex"

GEOMETRY = {"vocabulary": ["geometry"]}
VECTORS = {"vocabulary": ["vectors"]}
SEQUENCES = {"vocabulary": ["sequences"]}


def _lark(src):
    return parse_latex(src, backend="lark")


def _lark_pi(src):
    """The Lark grammar has no \\pi. Do what the app does: parse a placeholder
    symbol in its place, then substitute pi."""
    return _lark(src.replace(r"\pi", r"\chi ")).subs(sp.Symbol("chi"), sp.pi)


def _lark_positive_x(src):
    """Parse, then re-declare x positive (the app's per-item assumptions)."""
    return _lark(src).subs(sp.Symbol("x"), sp.Symbol("x", positive=True))


def _describe(value):
    """Lark returns a Tree for an ambiguous parse; list its readings sorted."""
    if isinstance(value, Tree):
        readings = sorted(
            _describe(c) if isinstance(c, Tree) else sp.sstr(c) for c in value.children
        )
        return "ambiguous: " + " | ".join(readings)
    return value


def _lark_gives(src):
    """What the Lark parser does with src, as stable text for `sympy_gives`.

    Lark's error messages list the expected tokens in hash order, so they are
    cut off to keep the output deterministic.
    """
    try:
        return _describe(_lark(src))
    except Exception as e:  # noqa: BLE001 - recording SymPy's behaviour
        msg = " ".join(str(e).split()).split(" Expected one of")[0]
        return f"error: {type(e).__name__}: {msg[:120]}"


def _sympy(src, note=None, options=None, via=_lark):
    return case(OP, [src], via(src), options=options, note=note)


def _spec(src, expected, note, options=None, compare="exact"):
    return spec_case(
        OP, [src], expected, note, compare=compare, options=options,
        sympy_gives=lambda: _lark_gives(src),
    )


def _refused(src, thing, note, options=None):
    return _spec(src, f"refused:{thing}", note, options=options, compare="error")


def silent_misreads():
    """Inputs the app misreads silently today, plus near neighbours that must keep working."""
    return [
        _spec(r"\Delta x", "Delta_x",
              "\\Delta plus a letter is one symbol Delta_x (the app's fallback parser reads Delta*x)."),
        _spec(r"\frac{\Delta y}{\Delta x}", "Delta_y/Delta_x",
              "Each \\Delta pair is one symbol, so the Deltas never cancel to y/x."),
        _spec(r"\Delta x^{2}", "Delta_x**2",
              "The superscript binds to the atom before it, which is the whole symbol Delta_x."),
        _spec(r"\Delta v_{x}", "Delta_v_x",
              "\\Delta followed by a subscripted symbol is one symbol named Delta_<symbol>."),
        _spec(r"F\Delta t = m\Delta v", "Eq(F*Delta_t, m*Delta_v)",
              "Impulse-momentum: a letter before \\Delta multiplies the Delta symbol; never F*Delta*t."),
        _refused(r"\frac{dy}{dx}", "derivative",
                 "Leibniz notation is refused by default; never read as (d*y)/(d*x) = y/x."),
        _refused(r"\frac{d}{dx}x^{2}", "derivative",
                 "The d/dx operator is refused by default (SymPy builds an unevaluated Derivative)."),
        _refused(r"\frac{d^{2}y}{dx^{2}}", "derivative",
                 "Second-derivative Leibniz notation is refused by name like the first."),
        _sympy(r"\frac{d}{t}",
               note="d alone is an ordinary variable (distance over time); only d/d<var> is a derivative."),
    ]


def geometry():
    return [
        _refused(r"\overline{AB}", "\\overline",
                 "Without the geometry vocabulary \\overline is refused, never conjugate(A)*conjugate(B)."),
        _spec(r"\overline{AB}", "seg_AB",
              "Geometry vocabulary: a segment is one named length.", options=GEOMETRY),
        _sympy("AB", note="Without the geometry vocabulary a run of capitals keeps the algebra reading A*B."),
        _spec("AB", "seg_AB",
              "Geometry vocabulary: a run of 2+ capital letters names a segment.", options=GEOMETRY),
        _spec(r"AB^{2} + BC^{2} = AC^{2}", "Eq(seg_AB**2 + seg_BC**2, seg_AC**2)",
              "Pythagorean relation between segments; the superscript applies to the whole segment name.",
              options=GEOMETRY),
        _sympy(r"\frac{1}{2}bh", options=GEOMETRY,
               note="The geometry vocabulary leaves lowercase products alone, so SymPy's reading is right."),
        _spec(r"\angle ABC", "angle_ABC",
              "Geometry vocabulary: an angle is one named measure.", options=GEOMETRY),
        _spec(r"m\angle ABC = 90^{\circ}", "Eq(angle_ABC, pi/2)",
              "m\\angle reads as the angle's measure; degrees become exact radians.", options=GEOMETRY),
        _refused(r"\angle ABC", "\\angle", "Without the geometry vocabulary \\angle is refused by name."),
        _refused(r"\triangle ABC", "\\triangle",
                 "A triangle names a figure, not an expression, so it is refused even with geometry.",
                 options=GEOMETRY),
        _refused(r"AB \cong CD", "\\cong", "Congruence is a statement, not an expression.", options=GEOMETRY),
        _refused(r"AB \parallel CD", "\\parallel", "Parallelism is a statement, not an expression.",
                 options=GEOMETRY),
    ]


def vectors():
    return [
        _refused(r"\vec{v}", "\\vec", "Without the vectors vocabulary \\vec is refused by name."),
        _spec(r"\vec{v}", "vec_v", "Vectors vocabulary: an arrow vector is one symbol vec_<name>.",
              options=VECTORS),
        _refused(r"\hat{i}", "\\hat", "Without the vectors vocabulary \\hat is refused by name."),
        _spec(r"\hat{i}", "hat_i", "Vectors vocabulary: a unit vector is one symbol hat_<name>.",
              options=VECTORS),
        _refused(r"\mathbf{v}", "\\mathbf", "Without the vectors vocabulary \\mathbf is refused by name."),
        _spec(r"\mathbf{v}", "vec_v", "Vectors vocabulary: bold v is the same vector as \\vec{v}.",
              options=VECTORS),
        _refused(r"\overrightarrow{AB}", "\\overrightarrow",
                 "Without the vectors vocabulary \\overrightarrow is refused by name."),
        _spec(r"\overrightarrow{AB}", "vec_AB", "Vectors vocabulary: the vector from A to B is one symbol.",
              options=VECTORS),
        _spec(r"3\hat{i} - 4\hat{j}", "3*hat_i - 4*hat_j",
              "Component form: unit vectors multiply like symbols.", options=VECTORS),
    ]


def degrees():
    return [
        _spec(r"30^{\circ}", "pi/6", "Degrees are always on and become exact radians: 30*pi/180."),
        _spec(r"30^\circ", "pi/6", "Unbraced ^\\circ means the same as ^{\\circ}."),
        _spec(r"30\degree", "pi/6", "\\degree means the same as ^{\\circ}."),
        _spec(r"\sin(30^{\circ})", "1/2",
              "Degrees convert before the function applies; SymPy then evaluates sin(pi/6)."),
        _spec(r"(2x+10)^{\circ}", "pi*(2*x + 10)/180",
              "A degree mark on a parenthesised expression scales it by pi/180, as in angle-algebra items."),
        _refused(r"f \circ g", "\\circ",
                 "Outside a superscript \\circ is function composition, which the parser does not support."),
    ]


def refused_commands():
    return [
        _refused(r"\mathrm{m/s}", "\\mathrm", "Units are parsed only by parse_quantity, never as letters."),
        _refused(r"9.8\,\mathrm{m/s^2}", "\\mathrm",
                 "A quantity with units is refused by name here; the \\, spacing is not the problem."),
        _refused(r"5\text{ cm}", "\\text", "\\text is refused by name, never read as letters."),
        _refused(r"\lim_{x \to 0} \frac{\sin x}{x}", "\\lim",
                 "Limits are refused on purpose after the lim-collapse-to-0 incident (SymPy builds a Limit)."),
        _refused(r"x\in[0,1]", "\\in", "Set membership is refused by name; intervals go through other ops."),
        _refused(r"\begin{pmatrix}1&2\\3&4\end{pmatrix}", "\\begin{pmatrix}",
                 "Matrices are not expressions here; the refusal names the environment."),
    ]


def subscripts():
    return [
        _spec("x_1", "x_1", "Subscripts join the name with a bare underscore (SymPy names it x_{1})."),
        _spec(r"v_{0x}", "v_0x", "A letters-and-digits subscript becomes part of the name."),
        _spec(r"F_{net}", "F_net", "Multi-letter subscripts are names, not products."),
        _spec(r"a_{c}", "a_c", "A braced one-letter subscript is the same as an unbraced one."),
        _spec(r"x_{12}", "x_12", "A multi-digit braced subscript is part of the name."),
        _spec(r"v_{0}t + \frac{1}{2}at^{2}", "v_0*t + a*t**2/2",
              "Kinematics: subscripted symbols take part in implicit products."),
        _spec(r"v_x^{2}", "v_x**2", "A superscript after a subscript squares the whole symbol."),
        _spec("a_n", "a_n", "Without the sequences vocabulary a_n is an ordinary symbol."),
        _refused(r"a_{n-1}", "subscript_expression",
                 "A subscript with an operator is refused unless the sequences vocabulary is on."),
        _spec(r"a_{n-1}", "a(n - 1)",
              "Sequences vocabulary: a subscript is the argument of the sequence a.", options=SEQUENCES),
        _spec("a_n", "a(n)", "Sequences vocabulary: a_n is the sequence a at n.", options=SEQUENCES),
        _spec(r"a_{n+1} = 2a_n", "Eq(a(n + 1), 2*a(n))",
              "A recurrence relation reads as an equation between sequence terms.", options=SEQUENCES),
    ]


def binding():
    """Where exponents and function arguments attach."""
    return [
        _spec(r"400e^{\ln(0.75)t}", "400*exp(log(3/4)*t)",
              "e^{...} takes the whole braced exponent; the app reads 300.0*t today."),
        _spec(r"\ln\sqrt{x}^{2}", "log(x)",
              "The superscript binds to \\sqrt{x}, giving log(sqrt(x)**2), which SymPy simplifies to log(x)."),
        _sympy(r"\sin 2x", note="An unparenthesised argument runs over the implicit product 2x."),
        _spec(r"\ln x^{2}", "log(x**2)",
              "The superscript binds to x, not to ln x (Lark returns both readings)."),
        _spec(r"\sin x^{2}", "sin(x**2)", "The superscript binds to x, as textbooks read sin x^2."),
        _spec(r"\sin(x)^{2}", "sin(x)**2",
              "A parenthesised call is itself the atom, so the square applies to sin(x), as calculators read it."),
        _spec(r"\ln x + \ln y", "log(x) + log(y)", "An unparenthesised argument stops at + or -."),
        _spec(r"2\sin x\cos x", "2*sin(x)*cos(x)",
              "An unparenthesised argument stops at the next function name (double-angle identity)."),
        _spec(r"\sin^{2}x+\cos^{2}x", "sin(x)**2 + cos(x)**2",
              "Power-of-function notation; each argument stops at the +."),
        _sympy(r"\sqrt{x}^{2}", note="SymPy simplifies sqrt(x)**2 to x."),
        _spec(r"x^2y", "x**2*y", "An implicit product may follow a power (Lark rejects this input)."),
        _spec(r"x(x+1)", "x*(x + 1)",
              "A single letter before parentheses is multiplication by default, never a function x."),
        _sympy(r"-x^2", note="The minus sign applies after the power."),
        _sympy(r"-2^2", note="The minus sign applies after the power."),
    ]


def constants_and_decimals():
    pi_note = "Lark lacks \\pi; computed as the app does (placeholder symbol, then .subs(..., pi))."
    return [
        _spec("e", "E", "A lone e is Euler's number, not a symbol."),
        _spec(r"e^{x}", "exp(x)", "e^{...} is the exponential function."),
        _spec(r"e^{\ln 5}", "5", "With e as Euler's number, SymPy simplifies exp(log(5)) to 5."),
        _sympy(r"\pi", note=pi_note, via=_lark_pi),
        _sympy(r"2\pi r", note=pi_note, via=_lark_pi),
        _sympy(r"\frac{3\pi}{4}", note=pi_note, via=_lark_pi),
        _sympy(r"\cos\left(\frac{\pi}{3}\right)", note=pi_note, via=_lark_pi),
        _spec("0.75", "3/4", "Typed decimals are exact rationals (SymPy gives Float)."),
        _spec("0.1+0.2", "3/10", "Exact decimals add exactly; no binary rounding."),
        _spec("1.5x", "3*x/2", "A decimal coefficient is exact."),
        _spec(".5", "1/2", "A decimal without a leading zero is still a number."),
        _spec("-0.5", "-1/2", "A negative typed decimal is an exact negative rational (SymPy gives Float)."),
        _spec(r"x^{0.5}", "sqrt(x)", "A decimal exponent is exact, so x^{0.5} is sqrt(x)."),
        _spec(r"(0.75)^{t}", "(3/4)**t", "A decimal base is exact."),
    ]


def implicit_multiplication():
    return [
        _sympy("2x"),
        _sympy("3(x+1)", note="SymPy distributes a number over a sum at construction."),
        _sympy("xy"),
    ]


def fractions_and_roots():
    return [
        _sympy(r"\frac12", note="Brace-less \\frac with two digits."),
        _sympy(r"\frac{x^2-1}{x-1}", note="Parsing does not cancel the removable factor."),
        _sympy(r"\frac{1}{1+\frac{1}{x}}", note="A nested fraction stays nested."),
        _sympy(r"\sqrt{50}", note="SymPy simplifies the radical at construction."),
        _sympy(r"\sqrt[3]{x}"),
        _sympy(r"\sqrt{x^{2}}", note="Without assumptions sqrt(x**2) does not simplify."),
        _sympy(r"\sqrt{x^{2}}", options={"assumptions": {"x": {"positive": True}}}, via=_lark_positive_x,
               note="With x positive, sqrt(x**2) is x."),
    ]


def operators_and_grouping():
    return [
        _sympy(r"\left(x+1\right)^2"),
        _sympy(r"2\cdot3"),
        _sympy(r"2 \times x"),
        _sympy(r"6\div 3"),
        _sympy("|x|"),
        _sympy(r"\left|x\right|"),
        _sympy(r"|2x+1| = 5"),
    ]


def functions():
    return [
        _sympy(r"\sin^{-1} x", note="^{-1} on a trig function is the inverse function, not a reciprocal."),
        _sympy(r"\arcsin x"),
        _sympy(r"\cos^{-1}\left(\frac{1}{2}\right)", note="SymPy evaluates acos(1/2) to pi/3."),
        _sympy(r"\log_2 x"),
        _sympy(r"\log_{10}(x)"),
        _sympy(r"\log_2 8", note="SymPy evaluates an exact integer logarithm."),
        _spec(r"\log x", "log(x)/log(10)",
              "Unsubscripted \\log is the common (base-10) log, as US high-school and AP Precalculus texts read it;"
              " SymPy's log is natural (\\ln is natural)."),
        _spec(r"\log(100)", "2",
              "Base-10 \\log reads as log(100, 10), which SymPy evaluates to 2 exactly as it does for"
              " \\log_{10}(100); SymPy's natural log(100) would misgrade log(100) = 2 items."),
        _sympy(r"\ln x"),
        _sympy(r"2\ln x"),
        _sympy(r"x^{-1/2}"),
        _spec(r"\sigma", "sigma", "Every lowercase Greek letter is a symbol (Lark's grammar lacks \\sigma)."),
        _sympy(r"\sin\theta", note="A Greek letter is a valid unparenthesised function argument."),
    ]


def signs_relations_infinity():
    return [
        _sympy("-2x+5"),
        _sympy("-(x-3)"),
        _sympy("x = 3"),
        _sympy(r"2x+1 \le 5"),
        _sympy(r"2x - 1 > 3", note="A strict inequality stays strict (StrictGreaterThan), as inequality_1d keys need."),
        _sympy(r"x \geq -2"),
        _sympy(r"x \ne 0"),
        _sympy(r"\infty"),
        _sympy(r"-\infty"),
    ]


def generate():
    return (
        silent_misreads()
        + geometry()
        + vectors()
        + degrees()
        + refused_commands()
        + subscripts()
        + binding()
        + constants_and_decimals()
        + implicit_multiplication()
        + fractions_and_roots()
        + operators_and_grouping()
        + functions()
        + signs_relations_infinity()
    )
