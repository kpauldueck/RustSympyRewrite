"""Answer equivalence: the homework app's grading decision, as SymPy-parity cases.

The app decides whether a typed answer matches the key by running a fixed
recipe of SymPy calls (app_sympy_audit.md). Each recipe is an op with source
"app". Its answer is whatever SymPy returns when the generator runs the
recipe, so rsympy passes by giving the same answer from the same steps. The
inputs include the ones where SymPy's answer is surprising: typed decimals
become Floats, removable holes vanish under simplify, identities simplify
cannot prove, and nan/oo/zoo, which the last step compares structurally.

App recipes (args are strings in SymPy syntax, recorded as typed; the answer
is printed "True" or "False"):

  equivalent           [a, b]: the app's expressions_equivalent.
                         A = sympify(a); B = sympify(b)  (plain Symbols, no
                                                          assumptions)
                         d = simplify(A - B)
                         if d == 0: True
                         elif simplify(d) == 0: True
                         else: expand(A) == expand(B)
                       Every "==" is SymPy's structural equality, so the last
                       step says nan == nan and oo == oo.
  equivalent_relation  [r1, r2], each Eq(lhs, rhs) or an inequality: the
                       app's expression grader for relations.
                         R1 = sympify(r1); R2 = sympify(r2)
                       Each is moved to one side as (kind, d):
                         Eq(l, r)       -> ("==", l - r)
                         l > r, l >= r  -> (">", l - r), (">=", l - r)
                         l < r, l <= r  -> (">", r - l), (">=", r - l)
                       Different kinds give False. For "==" the answer is
                       equivalent(d1, d2) or equivalent(d1, -d2); for an
                       inequality it is equivalent(d1, d2). Here equivalent
                       is the recipe above applied to the expressions d1, d2
                       (no sympify).

Single SymPy calls (source "sympy"; the symbols' assumptions are in the case's
"assumptions" field). The app's symbols are plain; these record what simplify
and sqrt do with the same expressions once a symbol is positive or real. Some
results change (sqrt(x**2), the log sums and differences) and some do not
(log(x**2) - 2*log(Abs(x)) for real x).

  simplify             [expr]: simplify(expr).
  sqrt                 [expr]: sqrt(expr).
"""

import sympy as sp
from sympy import Abs, Symbol, expand, log, simplify, sqrt, symbols, sympify

from caselib import case, outcome

TOPIC = "equivalence"

XP, YP = symbols("x y", positive=True)
XR = Symbol("x", real=True)


def app_equivalent(a, b):
    """The app's expressions_equivalent (see the module docstring)."""
    first = simplify(a - b)
    if first == 0:
        return True
    if simplify(first) == 0:
        return True
    return expand(a) == expand(b)


def _one_side(rel):
    """(kind, d): rel holds exactly when d == 0 (kind "=="), d > 0 or d >= 0."""
    if isinstance(rel, sp.Equality):
        return "==", rel.lhs - rel.rhs
    if rel.rel_op in (">", ">="):
        return rel.rel_op, rel.lhs - rel.rhs
    return {"<": ">", "<=": ">="}[rel.rel_op], rel.rhs - rel.lhs


def app_relation(r1, r2):
    """The app's relation grader (see the module docstring)."""
    k1, d1 = _one_side(r1)
    k2, d2 = _one_side(r2)
    if k1 != k2:
        return False
    if k1 == "==":
        return app_equivalent(d1, d2) or app_equivalent(d1, -d2)
    return app_equivalent(d1, d2)


def eq(a, b, note=None):
    return outcome(
        "equivalent", [a, b], lambda: app_equivalent(sympify(a), sympify(b)), note=note, source="app",
    )


def rel(a, b, note=None):
    return outcome(
        "equivalent_relation", [a, b], lambda: app_relation(sympify(a), sympify(b)), note=note, source="app",
    )


# --- cases --------------------------------------------------------------------

def _expressions():
    return [
        # Factored vs expanded.
        eq("(x + 1)**2", "x**2 + 2*x + 1"),
        eq("(x - 3)*(x + 2)", "x**2 - x - 6"),
        eq("(x - 3)*(x + 2)", "x**2 + x - 6", "sign error in the middle term"),
        eq("(x + 1)**2", "x**2 + 1", "the classic dropped middle term"),
        # Rational expressions.
        eq("1/x + 1/y", "(x + y)/(x*y)"),
        eq("1/(x + 1) - 1/(x - 1)", "-2/(x**2 - 1)"),
        eq("1/(x + 1)", "1/x + 1"),
        # Removable holes: simplify cancels the common factor, so the points
        # where only one side is undefined play no part.
        eq("(x**2 - 9)/(x + 3)", "x - 3", "the left side is undefined at x = -3"),
        eq("(x**2 - 1)/(x - 1)", "x + 1", "the left side is undefined at x = 1"),
        eq("1/(1/x + 1)", "x/(x + 1)", "the left side is also undefined at x = 0"),
        eq(
            "(x**3 - x)/(x**2 - x)", "(x**2 - 1)/(x - 1)",
            "both cancel to x + 1; the left side is undefined at 0 and 1, the right side only at 1",
        ),
        eq("(x**2 - 1)/(x - 1)", "(x**2 + 2*x - 3)/(x - 1) - 2", "both are x + 1 away from x = 1"),
        eq("tan(x)*cos(x)", "sin(x)", "the left side is undefined where cos(x) = 0"),
        # Logarithms; the app's symbols are plain (complex).
        eq("log(8)", "3*log(2)"),
        eq("log(x, 2)", "log(x)/log(2)", "change of base; sympify already reads log(x, 2) as log(x)/log(2)"),
        eq("2*log(x)", "log(x**2)", "for a plain (complex) x, simplify leaves 2*log(x) - log(x**2) as it is"),
        eq(
            "log(x) + log(y)", "log(x*y)",
            "the product rule needs x, y positive (x = y = -1 gives 2*I*pi vs 0); for plain (complex) symbols "
            "simplify does not combine the logs",
        ),
        eq(
            "log(x + y)", "log(x) + log(y)",
            "log of a sum is not a sum of logs; for plain (complex) symbols simplify does not combine or expand the logs",
        ),
        eq(
            "log(x**2)", "2*log(Abs(x))",
            "equal only for real x != 0 (x = I gives I*pi vs 0); for a plain (complex) x simplify does not expand log(x**2)",
        ),
        # Exponentials.
        eq("E**(x + y)", "E**x*E**y"),
        eq("4**x", "2**(2*x)"),
        eq("400*(3/4)**t", "300**t", "the coefficient was multiplied into the base"),
        eq(
            "400*(3/4)**t", "400*exp(t*log(3/4))",
            "(3/4)**t is exp(t*log(3/4)) by definition, but simplify does not rewrite one into the other",
        ),
        # Trig identities and near misses.
        eq("sin(x)**2 + cos(x)**2", "1"),
        eq("sin(2*x)", "2*sin(x)*cos(x)"),
        eq("cos(2*x)", "1 - 2*sin(x)**2"),
        eq("tan(x)", "sin(x)/cos(x)"),
        eq("sec(x)**2", "1 + tan(x)**2"),
        eq("sin(x + y)", "sin(x)*cos(y) + cos(x)*sin(y)"),
        eq("sin(2*x)", "2*sin(x)"),
        eq("sin(x + y)", "sin(x) + sin(y)"),
        eq("tan(2*x)", "2*tan(x)/(1 - tan(x)**2)", "the tangent double-angle identity, which simplify does not prove"),
        # Radicals and rationalised denominators.
        eq("sqrt(8)", "2*sqrt(2)", "sympify already evaluates sqrt(8) to 2*sqrt(2)"),
        eq("sqrt(12)", "3*sqrt(2)"),
        eq("1/(1 + sqrt(2))", "sqrt(2) - 1"),
        eq("sqrt(2) + sqrt(3)", "sqrt(5)"),
        eq("sqrt(x**2 + 9)", "x + 3"),
        eq("sqrt(x**2)", "x", "for a plain (complex) x, sqrt(x**2) stays unevaluated"),
        eq("sqrt(x**2)", "Abs(x)", "for complex x, sqrt(x**2) is not |x| (x = I gives I vs 1)"),
        # Absolute value.
        eq("Abs(x - 2)", "Abs(2 - x)", "sympify already writes Abs(2 - x) as Abs(x - 2)"),
        eq("Abs(x)", "x"),
        # Near misses.
        eq("x + 1", "x - 1"),
        eq("2*x", "x**2"),
        eq("x**2*x**3", "x**6", "exponents add when multiplying"),
        eq("-(x - 3)", "x - 3"),
        # Multi-variable (physics).
        eq("v0*t + a*t**2/2", "t*(2*v0 + a*t)/2"),
        # pi.
        eq("pi/2", "90*pi/180"),
        eq("pi", "22/7"),
        eq("pi", "3.14159", "3.14159 is a Float, and pi - 3.14159 stays unevaluated"),
        # Typed decimals are Floats.
        eq("0.5*x", "x/2", "0.5*x - x/2 collapses to 0 as soon as the difference is formed"),
        eq("x**0.333", "x**(1/3)", "a Float exponent and a Rational exponent"),
        eq("x**0.5", "sqrt(x)", "the Float exponent 0.5 is not the Rational 1/2"),
        eq("2**0.5", "sqrt(2)", "sympify evaluates 2**0.5 to a Float"),
        eq("log(0.5)", "-log(2)", "sympify evaluates log(0.5) to a Float"),
        eq(
            "400*exp(log(0.75)*t)", "400*(3/4)**t",
            "sympify evaluates log(0.75) to a Float, so the left side is 400*exp(-0.287682072451781*t)",
        ),
        eq("0.1 + 0.2", "0.3", "the binary Floats leave a difference of about 5.6e-17, which simplify keeps"),
        eq(
            "0.3333333333333333", "1/3",
            "the 16-digit Float differs from 1/3 by about 3.5e-17, which simplify keeps",
        ),
        # Non-finite values: the difference is nan whenever both sides are
        # infinite, so the structural expand comparison decides.
        eq("oo", "oo", "oo - oo is nan"),
        eq("oo", "-oo"),
        eq("zoo", "zoo", "zoo - zoo is nan"),
        eq("oo", "zoo"),
        eq("nan", "nan", "nan - nan is nan, but nan == nan structurally"),
        eq("nan", "0"),
        eq("oo - oo", "0", "sympify evaluates oo - oo to nan"),
    ]


def _with_assumptions():
    """What simplify and sqrt do once the symbols are positive or real."""
    out = []
    for e, note in [
        (log(XP) + log(YP) - log(XP * YP), None),
        (2 * log(XP) - log(XP**2), None),
        (log(XP + YP) - log(XP) - log(YP), "simplify combines the logs but nothing cancels"),
        (log(XR**2) - 2 * log(Abs(XR)), "log(x**2) = 2*log(Abs(x)) for real x != 0, but simplify leaves the difference unchanged"),
    ]:
        out.append(outcome("simplify", [e], lambda e=e: simplify(e), note=note))
    for s in [XP, XR]:
        out.append(case("sqrt", [s**2], sqrt(s**2)))
    return out


def _relations():
    return [
        # Equations.
        rel("Eq(y, 2*x + 1)", "Eq(2*x - y + 1, 0)", "the one-side forms differ by a factor of -1"),
        rel("Eq(x, 3)", "Eq(3, x)", "swapping sides is a factor of -1"),
        rel("Eq(y, 2*x + 1)", "Eq(y, 2*x - 1)", "a different line"),
        rel("Eq(y, 2*x + 1)", "Eq(2*y, 4*x + 1)", "only some terms were doubled (a different line)"),
        rel("Eq(y, 2*x + 1)", "Eq(2*y, 4*x + 2)", "the one-side forms differ by the factor 2"),
        rel("Eq(y, 2*x + 1)", "Eq(x, (y - 1)/2)", "solving for x instead multiplies the one-side form by -1/2"),
        rel("Eq(2*x + 3, 11)", "Eq(x, 4)", "the one-side forms 2*x - 8 and x - 4 differ by the factor 2"),
        rel("Eq(x/2 + y/3, 1)", "Eq(3*x + 2*y, 6)", "clearing denominators multiplies the one-side form by 6"),
        rel("Eq(x**2, 4)", "Eq(x, 2)", "squaring lost the root x = -2"),
        rel("Eq(x**2, 4)", "Eq(Abs(x), 2)", "the same roots, but x**2 - 4 and Abs(x) - 2 are different expressions"),
        rel("Eq(x*y, 1)", "Eq(y, 1/x)", "the one-side forms differ by the factor x"),
        rel("Eq(x, 3)", "x >= 3", "an equation and an inequality"),
        # Inequalities, one variable.
        rel("x > 3", "3 < x", "sympify keeps 3 < x as StrictLessThan(3, x)"),
        rel("x > 3", "-x < -3"),
        rel("x > 3", "2*x > 6", "the same solution set, but the one-side forms differ by the factor 2"),
        rel("2*x - 3 > 5", "x > 4", "the same solution set, but the one-side forms are 2*x - 8 and x - 4"),
        rel("x**2 < 4", "Abs(x) < 2", "the same real solution set, but 4 - x**2 and 2 - Abs(x) are different expressions"),
        rel("x > 3", "-x > -3", "the direction was not flipped when multiplying by -1"),
        rel("x > 3", "x < 3", "wrong direction"),
        rel("x > 3", "x >= 3", "strict vs non-strict"),
        rel("x**2 < 4", "x < 2", "lost the lower bound -2"),
        # Inequalities, two variables (half-planes).
        rel("y > 2*x + 1", "2*x - y < -1", "both are y - 2*x - 1 > 0 on one side"),
        rel("y > 2*x + 1", "-y > -2*x - 1", "the direction was not flipped when multiplying by -1"),
        rel("y > 2*x + 1", "2*y > 4*x + 2", "the same half-plane, but the one-side forms differ by the factor 2"),
    ]


def generate():
    return _expressions() + _with_assumptions() + _relations()
