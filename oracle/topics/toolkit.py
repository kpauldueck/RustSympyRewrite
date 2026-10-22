"""Toolkit: SymPy's answers on the calls behind the app's key and figure checks.

The homework app's figure lint runs real_roots and singularities, and its
graph_plot check evaluates lattice points with .subs(x, Integer(n)). Its
roadmap adds continuous_domain, solveset on trig keys over [0, 2*pi), Poly,
nroots and limit for figure end behaviour, summation and rsolve for sequence
keys, and fu, trigsimp and expand_trig for trig identity steps
(app_sympy_audit.md). This topic records what the pinned SymPy returns on the
inputs those checks see, including the inputs where SymPy's answer is not the
textbook one: those are where a port most easily diverges.

Args are SymPy syntax. An arg is a string, the question as typed, when building
it as a SymPy object would rewrite it first; the call then runs on sympify(arg).
Every case is a SymPy call, or one of the short composites spelled out below
(source "sympy"), compared exactly unless it raises:

  poly_coeffs        [expr, x]: Poly(expr, x).all_coeffs(), highest degree
                     first, zero coefficients included.
  degree             [expr, x]: degree(expr, x).
  real_roots         [expr, x]: real_roots(Poly(expr, x)).
  nroots             [expr], options {"n": digits}: nroots(expr, n=digits).
  singularities      [expr, x, domain] or [expr, x]:
                     singularities(expr, x, domain) or singularities(expr, x).
  continuous_domain  [expr, x, domain]: sympy.calculus.util.continuous_domain.
  solveset           [expr_or_relation, x, domain]: solveset(e, x, domain), an
                     expression meaning e = 0.
  union, intersection, complement
                     [A, B]: Union(A, B), Intersection(A, B) and
                     Complement(A, B).
  summation          [expr, k, lo, hi]: summation(expr, (k, lo, hi)).
  rsolve             [recurrence, "a(n)", initial_conditions] (the third arg
                     is optional): rsolve(recurrence, a(n), ics) with a =
                     Function("a"), each arg sympified. SymPy returns None when
                     the initial conditions are inconsistent.
  limit              [expr, x, point, dir]: limit(expr, x, point, dir).
  subs               [expr, x, n]: expr.subs(x, Integer(n)).
  free_symbols       [expr]: sorted(expr.free_symbols, key=str), a list in
                     codepoint order ("A" < "a", "x10" < "x2").
  N                  [expr, n]: N(expr, n).
  diff               [expr, x] or [expr, x, order]: diff(expr, x[, order]).
  expand_trig, trigsimp, fu
                     [expr]: sympy.expand_trig, sympy.trigsimp and
                     sympy.simplify.fu.fu. Several args are lhs - rhs of a
                     true trig identity, so they are 0 as functions; SymPy's
                     answer shows whether that call reaches 0.

A call that raises records the exception's class name (compare "error").
"""

import sympy as sp
from sympy import (
    Function, Integer, Interval, Poly, Rational, S, cos, degree, diff, exp,
    expand_trig, limit, log, nroots, oo, pi, real_roots, sin, singularities,
    solveset, sqrt, summation, symbols, sympify, tan, trigsimp,
)
from sympy.calculus.util import continuous_domain
from sympy.simplify.fu import fu

from caselib import case, outcome

TOPIC = "toolkit"

x, y, k, n = symbols("x y k n")
a, b, c, t = symbols("a b c t")
seq = Function("a")
TWO_PI = Interval.Ropen(0, 2 * pi)


def _poly():
    out = []
    for e in [x**3 - 3 * x + 2, (x + 1) ** 3, x**2 / 2 + 3, a * x**2 + b * x + c, S(0)]:
        out.append(case("poly_coeffs", [e, x], Poly(e, x).all_coeffs()))
    out.append(outcome(
        "poly_coeffs", ["x + 1/x", x], lambda: Poly(sympify("x + 1/x"), x).all_coeffs(),
        note="A negative power of x is not a polynomial term in x.",
    ))
    out.append(case("degree", [(x + 1) ** 2 * (x - 2), x], degree((x + 1) ** 2 * (x - 2), x)))
    out.append(case("degree", [a * x**2 + x, x], degree(a * x**2 + x, x),
                    note="SymPy treats the symbolic leading coefficient a as nonzero."))
    out.append(case("degree", [S(0), x], degree(0, x), note="The zero polynomial."))
    out.append(case("degree", [x**2 * y**3 + x, y], degree(x**2 * y**3 + x, y)))
    out.append(outcome(
        "degree", ["(x**2 + 1)/(x - 1)", x], lambda: degree(sympify("(x**2 + 1)/(x - 1)"), x),
        note="A rational function, not a polynomial in x.",
    ))
    return out


def _roots():
    out = []
    for e, note in [
        ((x - 1) ** 2 * (x + 2), "A double root."),
        (2 * x**3 - 3 * x**2 - 11 * x + 6, None),
        (x**2 - x - 1, None),
        (x**2 + 1, None),
        (x**5 - x - 1, "Irreducible quintic: its one real root has no radical form."),
        (x**3 - 3 * x + 1, "Three real roots (casus irreducibilis): no real radical form."),
        ((x**2 - 2) * (x**5 - x - 1), "Radical and CRootOf roots in one list."),
        (S(5), "A nonzero constant (real_roots(5) without Poly raises PolynomialError)."),
    ]:
        out.append(case("real_roots", [e, x], real_roots(Poly(e, x)), note=note))
    for e, digits, note in [
        (x**5 - x - 1, 15, "One real root and two complex pairs."),
        (x**3 - 3 * x + 1, 10, "Three real roots."),
        (x**2 - x - 1, 6, None),
    ]:
        out.append(case("nroots", [e], nroots(e, n=digits), options={"n": digits}, note=note))
    return out


def _domains():
    out = []
    for e, dom, note in [
        (1 / (x**2 - 1), S.Reals, None),
        ((x**2 - 1) / (x - 1), S.Reals, "SymPy does not cancel x - 1, so the removable hole at 1 is a singularity."),
        (log(x), S.Reals, None),
        (1 / log(x), S.Reals, None),
        (tan(x), S.Reals, "The poles pi/2 + n*pi, an infinite set; SymPy writes it as two ImageSets of period 2*pi."),
        (tan(x), TWO_PI, None),
        (1 / sin(x), Interval(-pi, pi), "Both closed endpoints are poles."),
        (x / (x**2 + 1), S.Reals, None),
    ]:
        out.append(case("singularities", [e, x, dom], singularities(e, x, dom), note=note))
    out.append(case(
        "singularities", [x / (x**2 + 1), x], singularities(x / (x**2 + 1), x),
        note="No domain and no real=True on x: SymPy searches the complex plane.",
    ))
    for e in [
        sqrt(x - 2),
        1 / (x**2 - 4),
        (x**2 - 1) / (x - 1),
        sqrt(x) / (x - 4),
        log(x - 1) + sqrt(5 - x),
    ]:
        out.append(case("continuous_domain", [e, x, S.Reals], continuous_domain(e, x, S.Reals)))
    out.append(case("continuous_domain", [tan(x), x, TWO_PI], continuous_domain(tan(x), x, TWO_PI)))
    out.append(case(
        "continuous_domain", ["x**(1/3)", x, S.Reals], continuous_domain(x ** Rational(1, 3), x, S.Reals),
        note="SymPy keeps x < 0 although its x**(1/3) is the principal root, which is not real "
             "there (the subs case at x = -8 gives 2*(-1)**(1/3)).",
    ))
    tc = "tan(x)*cos(x)"
    out.append(case(
        "continuous_domain", [tc, x, TWO_PI], continuous_domain(sympify(tc), x, TWO_PI),
        note="SymPy does not exclude pi/2 and 3*pi/2, where tan(x) is undefined, although it "
             "excludes them for tan(x) alone and excludes the hole of (x**2 - 1)/(x - 1).",
    ))
    out.append(case(
        "singularities", [tc, x, TWO_PI], singularities(sympify(tc), x, TWO_PI),
        note="SymPy does not report pi/2 and 3*pi/2, where tan(x) is undefined, although it "
             "reports them for tan(x) alone and reports the hole of (x**2 - 1)/(x - 1).",
    ))
    out.append(case(
        "continuous_domain", [log(sin(x)), x, Interval(-2 * pi, 2 * pi)],
        continuous_domain(log(sin(x)), x, Interval(-2 * pi, 2 * pi)),
        note="Two periods of sin: sin(x) > 0 on (-2*pi, -pi) and on (0, pi), "
             "but SymPy keeps only (0, pi).",
    ))
    return out


def _trig_solveset():
    out = []
    quad = "Quadratic in a trig function on a finite domain"
    for e, note in [
        (2 * sin(x) - 1, None),
        (tan(x) - 1, "tan has period pi: two solutions in [0, 2*pi)."),
        (2 * cos(x) ** 2 - 1, "A quadratic in cos(x) with four solutions in [0, 2*pi)."),
        (sin(2 * x) - cos(x), "Double angle: cos(x)*(2*sin(x) - 1) = 0, four solutions."),
        (cos(x / 2) - S.Half, "x/2 = 5*pi/3 gives x = 10*pi/3, outside the domain."),
        (sin(x) - Rational(1, 3), "Non-special angle: asin values."),
        (sin(x) - 2, None),
        (2 * sin(x) ** 2 - sin(x) - 1, f"{quad}: (2*sin(x) + 1)*(sin(x) - 1), so sin(x) = 1 or -1/2. "
                                       "SymPy loses the sin(x) = -1/2 roots 7*pi/6 and 11*pi/6."),
        (2 * cos(x) ** 2 + cos(x) - 1, f"{quad}: (2*cos(x) - 1)*(cos(x) + 1), so cos(x) = 1/2 or -1. "
                                       "SymPy loses the cos(x) = -1 root pi."),
        (2 * sin(x) ** 2 + sin(x) - 1, f"{quad}: (2*sin(x) - 1)*(sin(x) + 1), so sin(x) = 1/2 or -1. "
                                       "SymPy loses the sin(x) = -1 root 3*pi/2."),
    ]:
        out.append(case("solveset", [e, x, TWO_PI], solveset(e, x, TWO_PI), note=note))
    out.append(case("solveset", [sin(x), x, Interval(0, 2 * pi)], solveset(sin(x), x, Interval(0, 2 * pi)),
                    note="Closed domain: 2*pi is a zero too."))
    for e, note in [
        (sin(x) > S.Half, None),
        (cos(x) <= 0, None),
        (2 * cos(x) - 1 >= 0, None),
        (sin(x) >= cos(x), None),
        (sin(x) <= 1, None),
        (tan(x) > 1, "tan has period pi, so this holds on (pi/4, pi/2) and (5*pi/4, 3*pi/2); "
                     "SymPy keeps only (pi/4, pi/2)."),
        (sin(2 * x) > 0, "sin(2*x) has period pi, so this holds on (0, pi/2) and (pi, 3*pi/2); "
                         "SymPy keeps only (0, pi/2)."),
        (tan(x) <= 0, "tan has period pi and zeros at 0 and pi, so this holds on {0}, (pi/2, pi] and "
                      "(3*pi/2, 2*pi); SymPy keeps only {0} and (pi/2, pi), dropping pi itself."),
        (2 * sin(x) ** 2 - sin(x) - 1 > 0, "(2*sin(x) + 1)*(sin(x) - 1) > 0 exactly where sin(x) < -1/2, "
                                           "on (7*pi/6, 11*pi/6); SymPy answers (pi/2, 2*pi), which "
                                           "includes x = pi, where the expression is -1."),
    ]:
        out.append(case("solveset", [e, x, TWO_PI], solveset(e, x, TWO_PI), note=note))
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
        out.append(case("union", [A, B], sp.Union(A, B)))
    for A, B in [
        (I(0, 5), I.open(3, 8)),
        (I(0, 1), I(1, 2)),
        (I(0, 2 * pi), sp.FiniteSet(pi / 6, 5 * pi / 6, 13 * pi / 6)),
        (sp.Union(I.open(-oo, -2), I.open(2, oo)), I(-3, 3)),
    ]:
        out.append(case("intersection", [A, B], sp.Intersection(A, B)))
    for A, B in [
        (S.Reals, I(0, 1)),
        (I(0, 2 * pi), sp.FiniteSet(pi / 2, 3 * pi / 2)),
        (S.Reals, sp.Union(I.open(-oo, 2), I.open(2, oo))),
    ]:
        out.append(case("complement", [A, B], sp.Complement(A, B)))
    return out


def _sequences():
    out = []
    for e, lo, hi, note in [
        ("2 + 3*(k - 1)", 1, n, "Arithmetic partial sum."),
        ("a1 + (k - 1)*d", 1, n, "Arithmetic partial sum with symbolic first term and difference."),
        ("3*2**(k - 1)", 1, n, "Geometric partial sum (3*2**n - 3 in textbook form)."),
        ("100*(-1/2)**(k - 1)", 1, n, "Geometric partial sum with a negative ratio."),
        ("a*r**(k - 1)", 1, n, "Symbolic ratio: r = 1 is a special case."),
        ("5*(1/3)**(k - 1)", 1, oo, "Convergent infinite geometric series."),
        ("2**k", 0, oo, "Divergent geometric series."),
    ]:
        out.append(case("summation", [e, k, lo, hi], summation(sympify(e), (k, lo, hi)), note=note))
    an = seq(n)
    for rec, ics, note in [
        ("Eq(a(n), a(n - 1) + 3)", "{a(1): 2}", "Arithmetic: a(1) = 2, d = 3."),
        ("Eq(a(n), a(n - 1) + d)", "{a(1): a1}", "Arithmetic with symbolic d and a(1)."),
        ("Eq(a(n), 2*a(n - 1))", "{a(0): 5}", "Geometric: a(0) = 5, r = 2."),
        ("Eq(a(n), -3*a(n - 1))", "{a(1): 2}", "Geometric with a negative ratio counted from a(1)."),
        ("Eq(a(n), r*a(n - 1))", "{a(1): a1}", "Geometric with symbolic r."),
        ("Eq(a(n + 1), a(n) + 4)", "{a(0): 1}", "Forward-shift form."),
        ("Eq(a(n), 4*a(n - 1)/5 + 100)", "{a(0): 0}", "Affine recurrence (repeated drug dose)."),
        ("Eq(a(n), 2*a(n - 1))", "{a(0): 1, a(1): 3}", "Inconsistent initial conditions: a(1) = 2*a(0) = 2, not 3."),
    ]:
        out.append(case("rsolve", [rec, "a(n)", ics], sp.rsolve(sympify(rec), an, sympify(ics)), note=note))
    out.append(case("rsolve", ["Eq(a(n), a(n - 1) + 3)", "a(n)"], sp.rsolve(sympify("Eq(a(n), a(n - 1) + 3)"), an),
                    note="No initial condition: SymPy names the arbitrary constant C0."))
    return out


def _end_behaviour():
    out = []
    for e, pt, note in [
        ((6 * x**2 - x) / (3 - 2 * x**2), -oo, "Equal degrees: ratio of leading coefficients, 6/(-2)."),
        ((2 * x + 1) / (x**2 + 1), -oo, "Higher denominator degree: horizontal asymptote y = 0."),
        (x**3 / (x**2 + 1), -oo, "Numerator degree one higher: unbounded, sign from the leading terms."),
        ((2 * x**2 - x + 3) / (x - 1) - 2 * x, oo, "What is left after subtracting the slant asymptote's slope term."),
        (-2 * x**3 + x, -oo, None),
        (400 * Rational(3, 4) ** x, -oo, "Decay function with a coefficient at -oo."),
        (3 * 2**x - 4, -oo, "Shifted exponential: horizontal asymptote y = -4."),
        (100 / (1 + 9 * exp(-x / 2)), oo, "Logistic: carrying capacity."),
        (100 / (1 + 9 * exp(-x / 2)), -oo, "Logistic: left asymptote."),
        (sqrt(x**2 + 1) / x, -oo, "sqrt(x**2) = -x for x < 0."),
        (x - sqrt(x**2 + x), oo, None),
        (sin(x), oo, "Oscillates, with no limit."),
    ]:
        dirn = "-" if pt == oo else "+"
        out.append(case("limit", [e, x, pt, dirn], limit(e, x, pt, dirn), note=note))
    for src, note in [
        ("2**x", "SymPy answers oo for a bare b**x at -oo although 2**x -> 0 there; with a "
                 "coefficient and a shift (3*2**x - 4, above) it gets -4."),
        ("(1/2)**x", "SymPy answers 0 for a bare b**x with b < 1 at -oo although (1/2)**x grows "
                     "without bound there; with a coefficient (400*(3/4)**x, above) it gets oo."),
    ]:
        out.append(case("limit", [src, x, -oo, "+"], limit(sympify(src), x, -oo, "+"), note=note))
    out.append(case("limit", ["(1/2)**x", x, oo, "-"], limit(sympify("(1/2)**x"), x, oo, "-"),
                    note="The same bare b**x at oo: exponential decay."))
    return out


def _points():
    out = []
    wave = 3 * sin(pi * x / 2) + 1
    for v in [-1, 0, 1]:
        out.append(case("subs", [wave, x, v], wave.subs(x, Integer(v)), note="graph_plot lattice point."))
    for e, var, v, note in [
        ("-2*sin(pi*(x - 1)/4) + 3", x, 0, "A radical lattice value."),
        (4 * cos(pi * x / 6), x, 1, None),
        (400 * Rational(3, 4) ** t, t, 2, None),
        (1 / (x - 2), x, 2, "A pole."),
        ((x**2 - 1) / (x - 1), x, 1, "A removable hole: 0/0."),
        ("log(x, 2)", x, 8, "log(x, 2) is log(x)/log(2); log(8)/log(2) equals 3."),
        ("x**(1/3)", x, -8, "The principal cube root of -8 (real_root(-8, 3) is -2)."),
    ]:
        out.append(case("subs", [e, var, v], sympify(e).subs(var, Integer(v)), note=note))
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
        ("atan2(3, 4)", 5, "The fifth significant digit is a trailing zero."),
        ("5", 3, "An integer to 3 significant digits."),
        ("12345", 3, "More integer digits than significant digits."),
        ("CRootOf(x**5 - x - 1, 0)", 12, "Numeric value of the quintic's real root."),
    ]:
        out.append(case("N", [src, digits], sp.N(sympify(src), digits), note=note))
    return out


def _calculus_and_trig():
    out = []
    for e in [3 * sin(pi * x / 2) + 1, 100 / (1 + 9 * exp(-x / 2)), 400 * Rational(3, 4) ** x]:
        out.append(case("diff", [e, x], diff(e, x)))
    out.append(case("diff", [x * exp(x), x, 2], diff(x * exp(x), x, 2)))
    for src, note in [
        ("sin(2*x)", None),
        ("tan(x + y)", None),
        ("sin(3*x)", None),
        ("cos(x + pi/2)", "Auto-evaluates to -sin(x) before expand_trig runs."),
        ("tan(x - pi/4)", "Auto-evaluation rewrites tan(x - pi/4) as -cot(x + pi/4) before expand_trig runs."),
    ]:
        out.append(case("expand_trig", [src], expand_trig(sympify(src)), note=note))
    step = "lhs - rhs of the identity"
    for src, note in [
        ("sin(x)**2 + cos(x)**2", None),
        ("cos(x)/(1 - sin(x)) - (1 + sin(x))/cos(x)", f"{step} cos(x)/(1 - sin(x)) = (1 + sin(x))/cos(x)."),
        ("tan(x)*sin(x) + cos(x) - sec(x)", f"{step} tan(x)*sin(x) + cos(x) = sec(x)."),
        ("(sin(x) + cos(x))**2", "Equals 1 + sin(2*x)."),
        ("tan(x) + cot(x) - sec(x)*csc(x)", f"{step} tan(x) + cot(x) = sec(x)*csc(x)."),
        ("(sec(x) - cos(x))/sin(x) - tan(x)", f"{step} (sec(x) - cos(x))/sin(x) = tan(x)."),
    ]:
        out.append(case("trigsimp", [src], trigsimp(sympify(src)), note=note))
    for src, note in [
        ("sin(x)**2 + cos(x)**2", None),
        ("cos(x)**4 - sin(x)**4", None),
        ("(sin(x) + cos(x))**2", None),
        ("sin(x - y) + sin(x + y)", None),
        ("cos(x)/(1 - sin(x)) - (1 + sin(x))/cos(x)", f"{step} cos(x)/(1 - sin(x)) = (1 + sin(x))/cos(x)."),
    ]:
        out.append(case("fu", [src], fu(sympify(src)), note=note))
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
