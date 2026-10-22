"""Answer equivalence: the homework app's grading decision.

Ops (args are strings in SymPy syntax, recorded exactly as typed; they are
read with sympify, i.e. evaluated, as the app's parse_expr path does):

  equivalent           args [a, b]. The default answer is the app's current
                       algorithm, app_equivalent() below, so those cases are
                       source "sympy" with a note starting "app algorithm".
                       options: "assumptions" {symbol: {assumption: bool}}
                       (symbols are complex otherwise) and "function": true
                       (removable holes count, so the domains must match).
                       Spec cases cover the decided departures: holes under
                       "function", exact typed decimals, nan, and identities
                       SymPy's simplify cannot prove.
  equivalent_relation  args [r1, r2], each "Eq(lhs, rhs)" or an inequality.
                       Equations: lhs - rhs agree up to a nonzero constant
                       factor (decided; the app today allows only a factor of
                       -1, see app_relation). Inequalities: the same solution
                       set over the reals (solveset for one variable; the
                       app's one-side rule for the two-variable ones it gets
                       right).
  equivalent_factors   args [student, key], each a list "[f1, f2, ...]", as
                       the app's expression_factors answer type. Spec: True
                       when the factors pair up one-to-one, each pair
                       equivalent up to sign (factors 1 and -1 are units and
                       are ignored), and the products are equivalent.
                       sympy_gives is the app algorithm on the two products.
"""

import sympy as sp
from sympy import S, Symbol, expand, simplify, solveset, sympify

from caselib import case, spec_case

TOPIC = "equivalence"

APP = "app algorithm"
POS_X = {"x": {"positive": True}}
POS_XY = {"x": {"positive": True}, "y": {"positive": True}}
REAL_X = {"x": {"real": True}}


def _parse(s, assumptions=None):
    names = {n: Symbol(n, **a) for n, a in sorted((assumptions or {}).items())}
    return sympify(s, locals=names)


def app_equivalent(a, b):
    """The app's expressions_equivalent: simplify(a - b) == 0, then simplify
    that result again, then expand(a) == expand(b) as the last resort."""
    first = simplify(a - b)
    if first == 0:
        return True
    if simplify(first) == 0:
        return True
    return expand(a) == expand(b)


def _options(assumptions=None, function=False):
    out = {}
    if assumptions:
        out["assumptions"] = assumptions
    if function:
        out["function"] = True
    return out or None


def eq(a, b, want, why=None, assumptions=None):
    """An "equivalent" case answered by the app algorithm (asserted == want)."""
    got = app_equivalent(_parse(a, assumptions), _parse(b, assumptions))
    assert got == want, (a, b, got)
    note = APP if why is None else f"{APP}; {why}"
    return case("equivalent", [a, b], got, options=_options(assumptions), note=note)


def spec_eq(a, b, want, why, assumptions=None, function=False):
    """An "equivalent" case whose answer is decided; sympy_gives is the app algorithm."""

    def sympy_answer():
        return app_equivalent(_parse(a, assumptions), _parse(b, assumptions))

    if not function:
        # A spec case exists only because the app algorithm departs from it.
        assert sympy_answer() != want, (a, b)
    return spec_case(
        "equivalent", [a, b], want, why,
        options=_options(assumptions, function), sympy_gives=sympy_answer,
    )


# --- relations ----------------------------------------------------------------

def _one_side(rel):
    """(kind, d): rel holds exactly when d == 0 (kind "=="), d > 0 or d >= 0."""
    if isinstance(rel, sp.Equality):
        return "==", rel.lhs - rel.rhs
    if rel.rel_op in (">", ">="):
        return rel.rel_op, rel.lhs - rel.rhs
    return {"<": ">", "<=": ">="}[rel.rel_op], rel.rhs - rel.lhs


def app_relation(r1, r2):
    """The app's current rule: move everything to one side and compare up to
    sign (for an inequality the sign flip also flips the direction)."""
    k1, d1 = _one_side(r1)
    k2, d2 = _one_side(r2)
    if k1 != k2:
        return False
    if k1 == "==":
        return app_equivalent(d1, d2) or app_equivalent(d1, -d2)
    return app_equivalent(d1, d2)


def _constant_multiple(r1, r2):
    """The decided equation rule: lhs - rhs agree up to a nonzero constant."""
    q = simplify((r1.lhs - r1.rhs) / (r2.lhs - r2.rhs))
    return not q.free_symbols and q.is_nonzero is True and q.is_finite is True


def _same_solution_set(r1, r2):
    (v,) = sorted(r1.free_symbols | r2.free_symbols, key=str)
    return solveset(r1, v, S.Reals) == solveset(r2, v, S.Reals)


def _same_on_grid(r1, r2):
    """Independent check of the decided inequality rule for two variables:
    the relations agree at every point of a fixed rational grid (offset by
    1/7 so no point lies on a boundary line with small integer coefficients)."""
    vs = sorted(r1.free_symbols | r2.free_symbols, key=str)
    pts = [sp.Rational(k, 2) + sp.Rational(1, 7) for k in range(-12, 13)]
    for p in pts:
        for q in pts:
            sub = dict(zip(vs, (p, q)))
            if bool(r1.subs(sub)) != bool(r2.subs(sub)):
                return False
    return True


def _check_decided(r1, r2, want):
    """Assert the decided rule gives `want`, by a route other than the app's."""
    if isinstance(r1, sp.Equality) and isinstance(r2, sp.Equality):
        assert _constant_multiple(r1, r2) == want, (r1, r2)
    elif len(r1.free_symbols | r2.free_symbols) == 2 and not isinstance(r1, sp.Equality) \
            and not isinstance(r2, sp.Equality):
        assert _same_on_grid(r1, r2) == want, (r1, r2)


def rel_app(a, b, want, why):
    """An equation (or two-variable inequality) answered by the app's rule,
    where the decided rule agrees."""
    r1, r2 = sympify(a), sympify(b)
    got = app_relation(r1, r2)
    assert got == want, (a, b, got)
    _check_decided(r1, r2, want)
    return case("equivalent_relation", [a, b], got, note=f"{APP} (one side, up to sign); {why}")


def rel_set(a, b, want, why):
    """A one-variable inequality answered by comparing solveset solution sets."""
    r1, r2 = sympify(a), sympify(b)
    got = _same_solution_set(r1, r2)
    assert got == want, (a, b, got)
    return case(
        "equivalent_relation", [a, b], got,
        note=f"same solution set over the reals (solveset); {why}",
    )


def rel_spec(a, b, want, why):
    """A decided relation case; sympy_gives is the app's one-side rule."""
    r1, r2 = sympify(a), sympify(b)
    assert app_relation(r1, r2) != want, (a, b)
    _check_decided(r1, r2, want)
    return spec_case(
        "equivalent_relation", [a, b], want, why,
        sympy_gives=lambda: app_relation(sympify(a), sympify(b)),
    )


# --- factor lists -------------------------------------------------------------

def _units_dropped(fs):
    return [f for f in fs if f not in (S.One, S.NegativeOne)]


def factors_match(student, key):
    """The decided equivalent_factors rule (see module docstring)."""
    st, ky = _units_dropped(student), _units_dropped(key)
    if len(st) != len(ky):
        return False
    remaining = list(ky)
    for f in st:
        for i, g in enumerate(remaining):
            if app_equivalent(f, g) or app_equivalent(f, -g):
                del remaining[i]
                break
        else:
            return False
    return app_equivalent(sp.Mul(*student), sp.Mul(*key))


def factors(student, key, want, why):
    st, ky = list(sympify(student)), list(sympify(key))
    assert factors_match(st, ky) == want, (student, key)
    return spec_case(
        "equivalent_factors", [student, key], want, why,
        sympy_gives=lambda: app_equivalent(sp.Mul(*st), sp.Mul(*ky)),
    )


# --- cases --------------------------------------------------------------------

def _expressions():
    return [
        # Factored vs expanded.
        eq("(x + 1)**2", "x**2 + 2*x + 1", True),
        eq("(x - 3)*(x + 2)", "x**2 - x - 6", True),
        eq("(x - 3)*(x + 2)", "x**2 + x - 6", False, "sign error in the middle term"),
        eq("(x + 1)**2", "x**2 + 1", False, "the classic dropped middle term"),
        # Rational expressions.
        eq("(x**2 - 9)/(x + 3)", "x - 3", True, "removable holes are ignored without the function option"),
        eq("1/x + 1/y", "(x + y)/(x*y)", True),
        eq("1/(x + 1) - 1/(x - 1)", "-2/(x**2 - 1)", True),
        eq("1/(x + 1)", "1/x + 1", False),
        # Logarithms, with and without positivity.
        eq("log(8)", "3*log(2)", True),
        eq("log(x, 2)", "log(x)/log(2)", True, "change of base"),
        eq("log(x) + log(y)", "log(x*y)", True, assumptions=POS_XY),
        eq("2*log(x)", "log(x**2)", False, "symbols are complex unless the item sets assumptions"),
        eq("2*log(x)", "log(x**2)", True, assumptions=POS_X),
        eq("log(x + y)", "log(x) + log(y)", False, "log of a sum is not a sum of logs, even for positive x, y", assumptions=POS_XY),
        # Exponentials.
        eq("E**(x + y)", "E**x*E**y", True),
        eq("4**x", "2**(2*x)", True),
        eq("400*(3/4)**t", "300**t", False, "the coefficient was multiplied into the base"),
        # Trig identities and near misses.
        eq("sin(x)**2 + cos(x)**2", "1", True),
        eq("sin(2*x)", "2*sin(x)*cos(x)", True),
        eq("cos(2*x)", "1 - 2*sin(x)**2", True),
        eq("tan(x)", "sin(x)/cos(x)", True),
        eq("sec(x)**2", "1 + tan(x)**2", True),
        eq("sin(x + y)", "sin(x)*cos(y) + cos(x)*sin(y)", True),
        eq("sin(2*x)", "2*sin(x)", False),
        eq("sin(x + y)", "sin(x) + sin(y)", False),
        # Radicals and rationalised denominators.
        eq("sqrt(8)", "2*sqrt(2)", True),
        eq("sqrt(12)", "3*sqrt(2)", False),
        eq("1/(1 + sqrt(2))", "sqrt(2) - 1", True),
        eq("sqrt(2) + sqrt(3)", "sqrt(5)", False),
        eq("sqrt(x**2 + 9)", "x + 3", False),
        eq("sqrt(x**2)", "x", False, "symbols are complex unless the item sets assumptions"),
        eq("sqrt(x**2)", "x", True, assumptions=POS_X),
        eq("sqrt(x**2)", "Abs(x)", False, "for complex x, sqrt(x**2) is not |x| (x = I gives I vs 1)"),
        eq("sqrt(x**2)", "Abs(x)", True, assumptions=REAL_X),
        # Absolute value.
        eq("Abs(x - 2)", "Abs(2 - x)", True),
        eq("Abs(x)", "x", False),
        # Near misses.
        eq("x + 1", "x - 1", False),
        eq("2*x", "x**2", False),
        eq("x**2*x**3", "x**6", False, "exponents add when multiplying"),
        eq("-(x - 3)", "x - 3", False),
        # Multi-variable (physics).
        eq("v0*t + a*t**2/2", "t*(2*v0 + a*t)/2", True),
        # pi and e.
        eq("pi/2", "90*pi/180", True),
        eq("pi", "22/7", False),
        eq("pi", "3.14159", False, "a typed decimal is an exact rational, never pi"),
        # Typed decimals where SymPy already agrees with the exact reading.
        eq("0.5*x", "x/2", True),
        eq("x**0.333", "x**(1/3)", False, "0.333 is exactly 333/1000, not 1/3"),
        eq("0.3333333333333333", "1/3", False, "a 16-digit decimal is exact and is not 1/3"),
        # Non-finite values.
        eq("oo", "oo", True, "decided True too; oo - oo is nan, so the expand fallback decides"),
        eq("oo", "-oo", False),
        eq("zoo", "zoo", True, "decided True too; the expand fallback decides"),
        eq("oo", "zoo", False),
        eq("nan", "0", False),
        eq("oo - oo", "0", False, "oo - oo is nan"),
    ]


def _expression_specs():
    return [
        # Identities SymPy's simplify cannot prove.
        spec_eq(
            "400*(3/4)**t", "400*exp(t*log(3/4))", True,
            "(3/4)**t is by definition exp(t*log(3/4)), but simplify never rewrites one into the other, so the app marks a correct exponential rewrite wrong.",
        ),
        spec_eq(
            "tan(2*x)", "2*tan(x)/(1 - tan(x)**2)", True,
            "The tangent double-angle identity holds wherever both sides are defined, but simplify cannot prove it, so the app marks it wrong.",
        ),
        spec_eq(
            "log(x**2)", "2*log(Abs(x))", True,
            "For real x (x != 0, both sides undefined at 0) log(x**2) = 2*log(|x|), but simplify cannot prove it even with x real.",
            assumptions=REAL_X,
        ),
        # Typed decimals are exact.
        spec_eq(
            "400*exp(log(0.75)*t)", "400*(3/4)**t", True,
            "Typed decimals are exact (0.75 = 3/4), so this is the same function; SymPy turns log(0.75) into a Float and never matches.",
        ),
        spec_eq(
            "x**0.5", "sqrt(x)", True,
            "Typed decimals are exact, so x**0.5 is x**(1/2); SymPy never equates a Float exponent with a Rational one.",
        ),
        spec_eq(
            "0.1 + 0.2", "0.3", True,
            "Typed decimals are exact (1/10 + 2/10 = 3/10); with binary Floats SymPy says the sum differs from 0.3.",
        ),
        spec_eq(
            "log(0.5)", "-log(2)", True,
            "Typed decimals are exact, so log(0.5) = log(1/2) = -log(2); SymPy evaluates log(0.5) to a Float.",
        ),
        spec_eq(
            "2**0.5", "sqrt(2)", True,
            "Typed decimals are exact, so 2**0.5 = 2**(1/2) = sqrt(2); SymPy evaluates 2**0.5 to a Float.",
        ),
        # Non-finite.
        spec_eq(
            "nan", "nan", False,
            "nan is equivalent to nothing, itself included (decided); the app's expand fallback says nan == nan.",
        ),
        # Function equivalence: removable holes count, so domains must match.
        spec_eq(
            "(x**2 - 1)/(x - 1)", "x + 1", False,
            "As functions the left side has a hole at x = 1 that x + 1 lacks; the default (no option) answer ignores it.",
            function=True,
        ),
        spec_eq(
            "1/(1/x + 1)", "x/(x + 1)", False,
            "As functions the complex fraction is undefined at x = 0 as well as x = -1, while x/(x + 1) is defined at 0.",
            function=True,
        ),
        spec_eq(
            "(x**3 - x)/(x**2 - x)", "(x**2 - 1)/(x - 1)", False,
            "Both simplify to x + 1, but the left side has holes at 0 and 1 and the right side only at 1.",
            function=True,
        ),
        spec_eq(
            "tan(x)*cos(x)", "sin(x)", False,
            "As functions tan(x)*cos(x) is undefined where cos(x) = 0 (SymPy's continuous_domain misses this and says Reals).",
            function=True,
        ),
        spec_eq(
            "1/(x + 1) - 1/(x - 1)", "-2/(x**2 - 1)", True,
            "Both sides exclude exactly x = 1 and x = -1, so they are equal as functions.",
            function=True,
        ),
        spec_eq(
            "(x**2 - 1)/(x - 1)", "(x**2 + 2*x - 3)/(x - 1) - 2", True,
            "Both sides are x + 1 with the same single hole at x = 1, so they are equal as functions.",
            function=True,
        ),
        spec_eq(
            "(x + 1)**2", "x**2 + 2*x + 1", True,
            "Polynomials have no holes, so the function option does not change the answer.",
            function=True,
        ),
    ]


def _relations():
    return [
        # Equations: the app compares up to sign; decided rule allows any nonzero constant.
        rel_app("Eq(y, 2*x + 1)", "Eq(2*x - y + 1, 0)", True, "the sides differ by a factor of -1"),
        rel_app("Eq(x, 3)", "Eq(3, x)", True, "swapping sides is a factor of -1"),
        rel_app("Eq(y, 2*x + 1)", "Eq(y, 2*x - 1)", False, "a different line"),
        rel_app(
            "Eq(y, 2*x + 1)", "Eq(2*y, 4*x + 1)", False,
            "only some terms were doubled, so lhs - rhs are not constant multiples (a different line)",
        ),
        rel_app(
            "Eq(x**2, 4)", "Eq(x, 2)", False,
            "squaring lost the root x = -2",
        ),
        rel_app(
            "Eq(x**2, 4)", "Eq(Abs(x), 2)", False,
            "same roots, but lhs - rhs are not constant multiples, so not equivalent under the decided rule either",
        ),
        rel_app(
            "Eq(x*y, 1)", "Eq(y, 1/x)", False,
            "the sides differ by the factor x, which is not a constant, so the decided rule also says False",
        ),
        rel_app("Eq(x, 3)", "x >= 3", False, "an equation is never equivalent to an inequality"),
        rel_spec(
            "Eq(y, 2*x + 1)", "Eq(2*y, 4*x + 2)", True,
            "lhs - rhs differ by the constant factor 2 (decided rule); the app today only allows a factor of -1.",
        ),
        rel_spec(
            "Eq(y, 2*x + 1)", "Eq(x, (y - 1)/2)", True,
            "Solving for x instead of y multiplies lhs - rhs by -1/2 (decided rule); the app today only allows a factor of -1.",
        ),
        rel_spec(
            "Eq(2*x + 3, 11)", "Eq(x, 4)", True,
            "lhs - rhs differ by the constant factor 2 (decided rule); the app today only allows a factor of -1.",
        ),
        rel_spec(
            "Eq(x/2 + y/3, 1)", "Eq(3*x + 2*y, 6)", True,
            "Clearing denominators multiplies lhs - rhs by 6 (decided rule); the app today only allows a factor of -1.",
        ),
        # Inequalities, one variable: solution sets.
        rel_set("x > 3", "3 < x", True, "the app's one-side rule agrees"),
        rel_set("x > 3", "-x < -3", True, "the app's one-side rule agrees"),
        rel_set("x > 3", "2*x > 6", True, "the app's one-side rule (up to sign only) says False"),
        rel_set("2*x - 3 > 5", "x > 4", True, "the app's one-side rule (up to sign only) says False"),
        rel_set("x**2 < 4", "Abs(x) < 2", True, "the app's one-side rule says False"),
        rel_set("x > 3", "-x > -3", False, "the sign was not flipped when multiplying by -1"),
        rel_set("x > 3", "x < 3", False, "wrong direction"),
        rel_set("x > 3", "x >= 3", False, "strict vs non-strict"),
        rel_set("x**2 < 4", "x < 2", False, "lost the lower bound -2"),
        # Inequalities, two variables (half-planes).
        rel_app("y > 2*x + 1", "2*x - y < -1", True, "both are y - 2*x - 1 > 0"),
        rel_app(
            "y > 2*x + 1", "-y > -2*x - 1", False,
            "the direction was not flipped when multiplying by -1, giving the other half-plane",
        ),
        rel_spec(
            "y > 2*x + 1", "2*y > 4*x + 2", True,
            "Multiplying both sides by 2 keeps the same half-plane (same solution set); the app's one-side rule says False.",
        ),
    ]


def _factor_lists():
    return [
        factors("[x + 3, x - 2]", "[x - 2, x + 3]", True, "Order of the factors does not matter."),
        factors(
            "[2 - x, -x - 3]", "[x - 2, x + 3]", True,
            "Each factor matches up to sign and the two sign changes cancel in the product.",
        ),
        factors(
            "[2 - x, x + 3]", "[x - 2, x + 3]", False,
            "Each factor matches up to sign, but the product has the wrong sign.",
        ),
        factors(
            "[-1, 2 - x, x + 3]", "[x - 2, x + 3]", True,
            "The unit -1 is ignored when pairing factors and fixes the sign of the product.",
        ),
        factors(
            "[x**2 - 4, x + 3]", "[x - 2, x + 2, x + 3]", False,
            "The products are equal but x**2 - 4 is not one of the key's factors.",
        ),
        factors(
            "[x + 3, x - 2]", "[x + 2, x - 3]", False,
            "Sign error inside both factors: neither factor matches and the products differ.",
        ),
        factors(
            "[-2, 1 - x, x + 1]", "[2, x - 1, x + 1]", True,
            "Numeric factors pair up to sign like the others (-2 with 2, 1 - x with x - 1) and the products agree.",
        ),
        factors(
            "[2*x - 2, x + 1]", "[2, x - 1, x + 1]", False,
            "Units are only 1 and -1, so 2*x - 2 does not pair with x - 1 and the lists have different lengths (decided).",
        ),
        factors(
            "[x**2 + 2*x + 1, x - 5]", "[(x + 1)**2, x - 5]", True,
            "Factors are compared by equivalence, not by form; whether the student's form is factored is a separate form check.",
        ),
    ]


def generate():
    return _expressions() + _expression_specs() + _relations() + _factor_lists()
