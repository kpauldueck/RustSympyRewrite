"""Toolkit: the compile-time checks the homework app runs on keys and figures.

Figure lint, inequality_1d keys, sequence keys, graph_plot lattice points and
trig identity steps all call these SymPy functions (app_sympy_audit.md). Args
are SymPy syntax. An arg is passed as a string, the question as typed, when
building it as a SymPy object would rewrite it first.

Ops:
  poly_coeffs        [expr, x]: Poly(expr, x).all_coeffs(), highest degree
                     first, zero coefficients included. A non-polynomial is
                     refused as "invalid:not_polynomial" (spec; SymPy raises
                     PolynomialError).
  degree             [expr, x]: degree(expr, x); degree(0) is -oo.
  real_roots         [expr, x]: real_roots(Poly(expr, x)): exact, ascending,
                     each root repeated by multiplicity; real roots with no
                     radical form come out as CRootOf(p, k).
  nroots             [expr, n]: nroots(expr, n=n), n significant digits.
                     Real roots ascending, then complex ones by real part,
                     each conjugate pair with the negative imaginary part
                     first. compare "equivalent": each root to n digits.
  singularities      [expr, x, domain], or [expr, x] where SymPy searches the
                     Complexes because x has no real=True assumption.
  continuous_domain  [expr, x, domain].
                     An isolated point where a subexpression is undefined is
                     a singularity and lies outside the continuous domain,
                     even when the hole is removable ((x**2 - 1)/(x - 1) at
                     1, tan(x)*cos(x) at pi/2). Where SymPy 1.14 misses such
                     a point, or covers only one period of a periodic
                     condition, the case is spec with the true set.
  solveset           [expr_or_relation, x, domain], an expression meaning
                     expr = 0. Here the domain is a finite Interval, as in
                     the 3.10 trig keys on [0, 2*pi). Where SymPy 1.14 loses
                     solutions the case is spec, with the true set.
  union, intersection, complement
                     [A, B]: Union(A, B), Intersection(A, B), and
                     Complement(A, B) = A minus B.
  summation          [expr, k, lo, hi], hi symbolic or oo.
  rsolve             [recurrence, "a(n)", initial_conditions] (the third arg
                     is optional): rsolve(recurrence, a(n), ics), the
                     recurrence an Eq or an expression equal to 0.
  limit              [expr, x, oo, "-"] or [expr, x, -oo, "+"]: end
                     behaviour.
  subs               [expr, x, n]: expr.subs(x, Integer(n)), exact, as the
                     graph_plot lattice check does it.
  free_symbols       [expr]: the free symbols as a list sorted by name in
                     codepoint order ("A" < "a", "x10" < "x2").
  N                  [expr, n]: N(expr, n), printed as SymPy prints the
                     Float (trailing zeros are significant and kept).
  diff               [expr, x] or [expr, x, order].
  expand_trig, trigsimp, fu
                     [expr]: sympy.expand_trig, sympy.trigsimp and
                     sympy.simplify.fu.fu. Identity-step checking: a step
                     lhs = rhs is valid when the op sends lhs - rhs to 0.
                     compare "equivalent" marks results whose exact form is
                     SymPy's heuristic choice, not a target.
"""

import sympy as sp
from sympy import (
    Function, Integer, Interval, Poly, Rational, S, cos, degree, diff, exp,
    expand_trig, limit, log, nroots, oo, pi, real_roots, sin, singularities,
    solveset, sqrt, summation, symbols, sympify, tan, trigsimp,
)
from sympy.calculus.util import continuous_domain
from sympy.simplify.fu import fu

from caselib import case, spec_case

TOPIC = "toolkit"

x, y, k, n = symbols("x y k n")
a, b, c, t = symbols("a b c t")
seq = Function("a")
TWO_PI = Interval.Ropen(0, 2 * pi)
NOT_POLY = "invalid:not_polynomial"


def _poly():
    out = []
    for e in [x**3 - 3 * x + 2, (x + 1) ** 3, x**2 / 2 + 3, a * x**2 + b * x + c, S(0)]:
        out.append(case("poly_coeffs", [e, x], Poly(e, x).all_coeffs()))
    out.append(spec_case(
        "poly_coeffs", ["x + 1/x", x], NOT_POLY,
        "A negative power of x is not a polynomial term; refuse by name instead of returning coefficients.",
        compare="error", sympy_gives=lambda: Poly(x + 1 / x, x).all_coeffs(),
    ))
    out.append(case("degree", [(x + 1) ** 2 * (x - 2), x], degree((x + 1) ** 2 * (x - 2), x)))
    out.append(case("degree", [a * x**2 + x, x], degree(a * x**2 + x, x),
                    note="SymPy treats the symbolic leading coefficient a as nonzero."))
    out.append(case("degree", [S(0), x], degree(0, x), note="The zero polynomial has degree -oo."))
    out.append(case("degree", [x**2 * y**3 + x, y], degree(x**2 * y**3 + x, y)))
    out.append(spec_case(
        "degree", ["(x**2 + 1)/(x - 1)", x], NOT_POLY,
        "A rational function has no polynomial degree; refuse by name.",
        compare="error", sympy_gives=lambda: degree((x**2 + 1) / (x - 1), x),
    ))
    return out


def _roots():
    out = []
    for e, note in [
        ((x - 1) ** 2 * (x + 2), "A double root is listed twice."),
        (2 * x**3 - 3 * x**2 - 11 * x + 6, None),
        (x**2 - x - 1, None),
        (x**2 + 1, None),
        (x**5 - x - 1, "Irreducible quintic: its one real root has no radical form, so SymPy gives CRootOf."),
        (x**3 - 3 * x + 1, "Three real roots (casus irreducibilis): SymPy gives CRootOf, not radicals or cosines."),
        ((x**2 - 2) * (x**5 - x - 1), "Radical and CRootOf roots sorted together."),
        (S(5), "A nonzero constant has no roots (real_roots(5) without Poly raises PolynomialError)."),
    ]:
        out.append(case("real_roots", [e, x], real_roots(Poly(e, x)), note=note))
    for e, digits in [(x**5 - x - 1, 15), (x**3 - 3 * x + 1, 10), (x**2 - x - 1, 6)]:
        out.append(case("nroots", [e, digits], nroots(e, n=digits), compare="equivalent",
                        note="Floats to n significant digits; real roots ascending, then the complex pairs."))
    return out


def _domains():
    out = []
    for e, dom, note in [
        (1 / (x**2 - 1), S.Reals, None),
        ((x**2 - 1) / (x - 1), S.Reals, "The removable hole at 1 counts as a singularity."),
        (log(x), S.Reals, None),
        (1 / log(x), S.Reals, None),
        (tan(x), S.Reals, "SymPy writes {pi/2 + n*pi} as two ImageSets of period 2*pi."),
        (tan(x), TWO_PI, None),
        (1 / sin(x), Interval(-pi, pi), "Both closed endpoints are singular."),
        (x / (x**2 + 1), S.Reals, None),
    ]:
        out.append(case("singularities", [e, x, dom], singularities(e, x, dom), compare="set", note=note))
    out.append(case(
        "singularities", [x / (x**2 + 1), x], singularities(x / (x**2 + 1), x), compare="set",
        note="No domain and no real=True on x: SymPy searches the Complexes, so figure lint must pass Reals.",
    ))
    for e in [
        sqrt(x - 2),
        1 / (x**2 - 4),
        (x**2 - 1) / (x - 1),
        sqrt(x) / (x - 4),
        log(x - 1) + sqrt(5 - x),
    ]:
        out.append(case("continuous_domain", [e, x, S.Reals], continuous_domain(e, x, S.Reals), compare="set"))
    out.append(case("continuous_domain", [tan(x), x, TWO_PI], continuous_domain(tan(x), x, TWO_PI), compare="set"))
    out.append(case(
        "continuous_domain", ["x**(1/3)", x, S.Reals], continuous_domain(x ** Rational(1, 3), x, S.Reals),
        compare="set",
        note="SymPy answers all reals, the textbook cube root, although its x**(1/3) is the principal root for x < 0.",
    ))
    # SymPy 1.14 misses where a trig factor is undefined, and periodic
    # conditions beyond the first period. These feed function equivalence
    # (tan(x)*cos(x) vs sin(x)), so the true sets are spec.
    tc = "tan(x)*cos(x)"
    out.append(spec_case(
        "continuous_domain", [tc, x, TWO_PI],
        "Union(Interval.Ropen(0, pi/2), Interval.open(pi/2, 3*pi/2), Interval.open(3*pi/2, 2*pi))",
        "tan(x) is undefined where cos(x) = 0, so the product is too (a removable hole, as for (x**2 - 1)/(x - 1)); "
        "SymPy 1.14 answers the whole interval.",
        compare="set", sympy_gives=lambda: continuous_domain(sympify(tc), x, TWO_PI),
    ))
    out.append(spec_case(
        "singularities", [tc, x, TWO_PI], "FiniteSet(pi/2, 3*pi/2)",
        "The holes of tan(x)*cos(x) at pi/2 and 3*pi/2 count as singularities, as the hole of (x**2 - 1)/(x - 1) "
        "at 1 does; SymPy 1.14 answers EmptySet.",
        compare="set", sympy_gives=lambda: singularities(sympify(tc), x, TWO_PI),
    ))
    out.append(spec_case(
        "continuous_domain", [log(sin(x)), x, Interval(-2 * pi, 2 * pi)],
        "Union(Interval.open(-2*pi, -pi), Interval.open(0, pi))",
        "sin(x) > 0 on (-2*pi, -pi) and (0, pi); SymPy 1.14 keeps only the first period (0, pi).",
        compare="set", sympy_gives=lambda: continuous_domain(log(sin(x)), x, Interval(-2 * pi, 2 * pi)),
    ))
    return out


def _trig_solveset():
    out = []
    for e, note in [
        (2 * sin(x) - 1, None),
        (tan(x) - 1, "tan has period pi: two solutions in [0, 2*pi)."),
        (2 * cos(x) ** 2 - 1, "A quadratic in cos that SymPy solves correctly."),
        (sin(2 * x) - cos(x), "Double angle: cos(x)*(2*sin(x) - 1) = 0, four solutions."),
        (cos(x / 2) - S.Half, "x/2 = 5*pi/3 gives x = 10*pi/3, outside the domain."),
        (sin(x) - Rational(1, 3), "Non-special angle: exact asin values."),
        (sin(x) - 2, None),
    ]:
        out.append(case("solveset", [e, x, TWO_PI], solveset(e, x, TWO_PI), compare="set", note=note))
    out.append(case("solveset", [sin(x), x, Interval(0, 2 * pi)], solveset(sin(x), x, Interval(0, 2 * pi)),
                    compare="set", note="Closed domain: 2*pi is a solution too."))
    for e in [sin(x) > S.Half, cos(x) <= 0, 2 * cos(x) - 1 >= 0, sin(x) >= cos(x), sin(x) <= 1]:
        out.append(case("solveset", [e, x, TWO_PI], solveset(e, x, TWO_PI), compare="set"))

    def wrong(e, want, why):
        return spec_case("solveset", [e, x, TWO_PI], want, why, compare="set",
                         sympy_gives=lambda: solveset(e, x, TWO_PI))

    drop = "SymPy 1.14 drops a root of this quadratic in a trig function on a finite domain"
    out.append(wrong(2 * sin(x) ** 2 - sin(x) - 1, "FiniteSet(pi/2, 7*pi/6, 11*pi/6)",
                     f"{drop} (sin(x) = -1/2 is lost); solveset over Reals intersected with [0, 2*pi) is right."))
    out.append(wrong(2 * cos(x) ** 2 + cos(x) - 1, "FiniteSet(pi/3, pi, 5*pi/3)",
                     f"{drop} (cos(x) = -1 is lost)."))
    out.append(wrong(2 * sin(x) ** 2 + sin(x) - 1, "FiniteSet(pi/6, 5*pi/6, 3*pi/2)",
                     f"{drop} (sin(x) = -1 is lost)."))
    one_period = "SymPy 1.14 solves this inequality on one period of the function only and misses the rest of [0, 2*pi)"
    out.append(wrong(tan(x) > 1, "Union(Interval.open(pi/4, pi/2), Interval.open(5*pi/4, 3*pi/2))",
                     f"{one_period} (tan has period pi)."))
    out.append(wrong(sin(2 * x) > 0, "Union(Interval.open(0, pi/2), Interval.open(pi, 3*pi/2))",
                     f"{one_period} (sin(2*x) has period pi)."))
    out.append(wrong(tan(x) <= 0, "Union(FiniteSet(0), Interval.Lopen(pi/2, pi), Interval.open(3*pi/2, 2*pi))",
                     f"{one_period}, including the zero of tan at pi."))
    out.append(wrong(2 * sin(x) ** 2 - sin(x) - 1 > 0, "Interval.open(7*pi/6, 11*pi/6)",
                     "(2*sin(x) + 1)*(sin(x) - 1) > 0 exactly where sin(x) < -1/2; SymPy 1.14 answers Interval.open(pi/2, 2*pi), which is wrong."))
    return out


def _interval_algebra():
    out = []
    I = Interval
    for A, B in [
        (I.Ropen(0, 1), I(1, 2)),
        (I.open(0, 1), I.open(1, 2)),
        (sp.FiniteSet(1), I.open(1, 2)),
        (sp.Union(I.open(-oo, 2), I.open(2, oo)), sp.FiniteSet(2)),
    ]:
        out.append(case("union", [A, B], sp.Union(A, B), compare="set"))
    for A, B in [
        (I(0, 5), I.open(3, 8)),
        (I(0, 1), I(1, 2)),
        (I(0, 2 * pi), sp.FiniteSet(pi / 6, 5 * pi / 6, 13 * pi / 6)),
        (sp.Union(I.open(-oo, -2), I.open(2, oo)), I(-3, 3)),
    ]:
        out.append(case("intersection", [A, B], sp.Intersection(A, B), compare="set"))
    for A, B in [
        (S.Reals, I(0, 1)),
        (I(0, 2 * pi), sp.FiniteSet(pi / 2, 3 * pi / 2)),
        (S.Reals, sp.Union(I.open(-oo, 2), I.open(2, oo))),
    ]:
        out.append(case("complement", [A, B], sp.Complement(A, B), compare="set"))
    return out


def _sequences():
    out = []
    for e, lo, hi, note in [
        ("2 + 3*(k - 1)", 1, n, "Arithmetic partial sum."),
        ("a1 + (k - 1)*d", 1, n, "Arithmetic partial sum with symbolic first term and difference."),
        ("3*2**(k - 1)", 1, n, "Geometric partial sum; SymPy leaves 3*2**(n + 1)/2 - 3, not 3*2**n - 3."),
        ("100*(-1/2)**(k - 1)", 1, n, "Geometric partial sum with a negative ratio."),
        ("a*r**(k - 1)", 1, n, "Symbolic ratio: SymPy returns a Piecewise with an r = 1 branch."),
        ("5*(1/3)**(k - 1)", 1, oo, "Convergent infinite geometric series."),
        ("2**k", 0, oo, "Divergent geometric series."),
    ]:
        res = summation(sympify(e), (k, lo, hi))
        out.append(case("summation", [e, k, lo, hi], res, compare="exact" if res.is_number else "equivalent",
                        note=note))
    an = seq(n)
    for rec, ics, note in [
        ("Eq(a(n), a(n - 1) + 3)", "{a(1): 2}", "Arithmetic: a(1) = 2, d = 3."),
        ("Eq(a(n), a(n - 1) + d)", "{a(1): a1}", "Arithmetic with symbolic d and a(1)."),
        ("Eq(a(n), 2*a(n - 1))", "{a(0): 5}", "Geometric: a(0) = 5, r = 2."),
        ("Eq(a(n), -3*a(n - 1))", "{a(1): 2}", "Geometric with a negative ratio counted from a(1)."),
        ("Eq(a(n), r*a(n - 1))", "{a(1): a1}", "Geometric with symbolic r; SymPy gives a1*r**n/r."),
        ("Eq(a(n + 1), a(n) + 4)", "{a(0): 1}", "Forward-shift form."),
        ("Eq(a(n), 4*a(n - 1)/5 + 100)", "{a(0): 0}", "Affine recurrence (repeated drug dose)."),
    ]:
        res = sp.rsolve(sympify(rec), an, sympify(ics))
        out.append(case("rsolve", [rec, "a(n)", ics], res, compare="equivalent", note=note))
    out.append(case("rsolve", ["Eq(a(n), a(n - 1) + 3)", "a(n)"], sp.rsolve(sympify("Eq(a(n), a(n - 1) + 3)"), an),
                    compare="equivalent", note="No initial condition: SymPy names the arbitrary constant C0."))
    out.append(spec_case(
        "rsolve", ["Eq(a(n), 2*a(n - 1))", "a(n)", "{a(0): 1, a(1): 3}"], "invalid:inconsistent_initial_conditions",
        "a(1) = 3 contradicts a(1) = 2*a(0) = 2; refuse by name (SymPy returns None).",
        compare="error", sympy_gives=lambda: sp.rsolve(sympify("Eq(a(n), 2*a(n - 1))"), an, {seq(0): 1, seq(1): 3}),
    ))
    return out


def _end_behaviour():
    out = []
    for e, pt, note in [
        ((6 * x**2 - x) / (3 - 2 * x**2), -oo, "Equal degrees: ratio of leading coefficients, 6/(-2)."),
        ((2 * x + 1) / (x**2 + 1), -oo, "Higher denominator degree: horizontal asymptote y = 0."),
        (x**3 / (x**2 + 1), -oo, "Numerator degree one higher: unbounded, sign from the leading terms."),
        ((2 * x**2 - x + 3) / (x - 1) - 2 * x, oo, "What is left after subtracting the slant asymptote's slope term."),
        (-2 * x**3 + x, -oo, None),
        (400 * Rational(3, 4) ** x, -oo, "Decay function at -oo (with a coefficient SymPy is right)."),
        (3 * 2**x - 4, -oo, "Shifted exponential: horizontal asymptote y = -4."),
        (100 / (1 + 9 * exp(-x / 2)), oo, "Logistic: carrying capacity."),
        (100 / (1 + 9 * exp(-x / 2)), -oo, "Logistic: left asymptote."),
        (sqrt(x**2 + 1) / x, -oo, "sqrt(x**2) = -x for x < 0."),
        (x - sqrt(x**2 + x), oo, None),
        (sin(x), oo, "Oscillates: SymPy returns the accumulation bounds, not a number."),
    ]:
        dirn = "-" if pt == oo else "+"
        out.append(case("limit", [e, x, pt, dirn], limit(e, x, pt, dirn), note=note))
    for src, want in [("2**x", "0"), ("(1/2)**x", "oo")]:
        e = sympify(src)
        out.append(spec_case(
            "limit", [src, x, -oo, "+"], want,
            f"SymPy 1.14 gets a bare b**x wrong at -oo (it answers {limit(e, x, -oo, '+')}); "
            "with a coefficient or a shift, or written as exp(x*log(b)), it is right.",
            sympy_gives=lambda e=e: limit(e, x, -oo, "+"),
        ))
    out.append(case("limit", ["(1/2)**x", x, oo, "-"], limit(sympify("(1/2)**x"), x, oo, "-"),
                    note="Exponential decay at oo; for a bare b**x SymPy is right at oo, wrong only at -oo."))
    return out


def _points():
    out = []
    wave = 3 * sin(pi * x / 2) + 1
    for v in [-1, 0, 1]:
        out.append(case("subs", [wave, x, v], wave.subs(x, Integer(v)), note="graph_plot lattice point."))
    for e, var, v, cmp, note in [
        ("-2*sin(pi*(x - 1)/4) + 3", x, 0, "exact", "Exact radical lattice value."),
        (4 * cos(pi * x / 6), x, 1, "exact", None),
        (400 * Rational(3, 4) ** t, t, 2, "exact", None),
        (1 / (x - 2), x, 2, "exact", "A pole evaluates to complex infinity."),
        ((x**2 - 1) / (x - 1), x, 1, "exact", "A removable hole evaluates to nan (0/0), not 2."),
        ("log(x, 2)", x, 8, "equivalent", "SymPy leaves log(8)/log(2) unevaluated; it equals 3."),
        ("x**(1/3)", x, -8, "exact", "The principal cube root, not -2 (real_root(x, 3) gives -2)."),
    ]:
        out.append(case("subs", [e, var, v], sympify(e).subs(var, Integer(v)), compare=cmp, note=note))
    return out


def _symbols_and_numbers():
    out = []
    for src, note in [
        ("2*pi*r", "pi is a constant, not a symbol."),
        ("E**x + e", "E is Euler's number; a lower-case e in SymPy syntax is a symbol."),
        ("Integral(x*y, (x, 0, 1))", "The integration variable is bound."),
        ("A*a + B + b", "Codepoint order: capitals first."),
        ("x10 + x2 + x1", "Plain string order, not natural order."),
        ("3 + sqrt(2)", None),
    ]:
        out.append(case("free_symbols", [src], sorted(sympify(src).free_symbols, key=str), note=note))
    for src, digits, note in [
        ("pi", 10, None),
        ("atan2(3, 4)", 5, "A trailing zero is a significant digit and is kept."),
        ("5", 3, "An integer to 3 significant digits."),
        ("12345", 3, "SymPy switches to scientific notation when the digits run out."),
        ("CRootOf(x**5 - x - 1, 0)", 12, "Numeric value of the quintic's real root."),
    ]:
        out.append(case("N", [src, digits], sp.N(sympify(src), digits), note=note))
    return out


def _calculus_and_trig():
    out = []
    for e in [3 * sin(pi * x / 2) + 1, 100 / (1 + 9 * exp(-x / 2)), 400 * Rational(3, 4) ** x]:
        out.append(case("diff", [e, x], diff(e, x)))
    out.append(case("diff", [x * exp(x), x, 2], diff(x * exp(x), x, 2)))
    for src, cmp, note in [
        ("sin(2*x)", "exact", None),
        ("tan(x + y)", "exact", None),
        ("sin(3*x)", "exact", None),
        ("cos(x + pi/2)", "exact", "Auto-evaluates to -sin(x) before expand_trig runs."),
        ("tan(x - pi/4)", "equivalent",
         "SymPy first rewrites tan(x - pi/4) as -cot(x + pi/4), so the result is in cot; (tan(x) - 1)/(tan(x) + 1) is the textbook form."),
    ]:
        out.append(case("expand_trig", [src], expand_trig(sympify(src)), compare=cmp, note=note))
    for src, cmp, note in [
        ("sin(x)**2 + cos(x)**2", "exact", None),
        ("cos(x)/(1 - sin(x)) - (1 + sin(x))/cos(x)", "exact", "Identity step: lhs - rhs reduces to 0."),
        ("tan(x)*sin(x) + cos(x) - sec(x)", "exact", "Identity step: lhs - rhs reduces to 0."),
        ("(sin(x) + cos(x))**2", "equivalent", "SymPy's form is 2*sin(x + pi/4)**2; 1 + sin(2*x) is the textbook one."),
    ]:
        out.append(case("trigsimp", [src], trigsimp(sympify(src)), compare=cmp, note=note))
    for src in ["tan(x) + cot(x) - sec(x)*csc(x)", "(sec(x) - cos(x))/sin(x) - tan(x)"]:
        e = sympify(src)
        out.append(spec_case(
            "trigsimp", [src], "0",
            "lhs - rhs of a true identity; the step checker needs 0, and SymPy's trigsimp stops at a nonzero form.",
            sympy_gives=lambda e=e: trigsimp(e),
        ))
    for src, cmp, note in [
        ("sin(x)**2 + cos(x)**2", "exact", None),
        ("cos(x)**4 - sin(x)**4", "exact", None),
        ("(sin(x) + cos(x))**2", "exact", None),
        ("sin(x - y) + sin(x + y)", "exact", None),
        ("cos(x)/(1 - sin(x)) - (1 + sin(x))/cos(x)", "equivalent",
         "fu does not combine fractions and returns this unchanged; trigsimp reduces it to 0."),
    ]:
        out.append(case("fu", [src], fu(sympify(src)), compare=cmp, note=note))
    return out


def generate():
    return (
        _poly()
        + _roots()
        + _domains()
        + _trig_solveset()
        + _interval_algebra()
        + _sequences()
        + _end_behaviour()
        + _points()
        + _symbols_and_numbers()
        + _calculus_and_trig()
    )
