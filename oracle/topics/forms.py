"""Form checks: is the student's answer written in the required form?

Ops (all spec: SymPy has no such checks): is_factored, is_expanded,
is_simplified_fraction, is_slope_intercept, is_simplified_radical. args[0] is
SymPy syntax read UNEVALUATED (as sympify(s, evaluate=False) reads it), since
evaluation would rewrite the very form being checked (sqrt(8) -> 2*sqrt(2),
6*(x + 1) -> 6*x + 6). Expected is "True"/"False".

Textbook definitions (conventions.md), with the decisions made here:
  factored        product of irreducible factors over the integers: one
                  rational constant factor allowed, every other factor a
                  primitive polynomial (content 1, so 2*x + 4 is not factored)
                  irreducible over the rationals (x**2 + 1, x**2 - 2 count),
                  powers and repeats of such factors allowed.
  expanded        no product of sums and no power of a sum anywhere; a number
                  times a sum counts as a product of sums. Combining like
                  terms is NOT required (x**2 + x + x + 1 is expanded), and a
                  power of a monomial counts as a monomial.
  simplified      a single fraction p/q (a polynomial counts, q = 1) with no
  fraction        fraction nested inside p or q and gcd(p, q) = 1 over the
                  integers, so a common integer factor or a common factor up
                  to sign also fails.
  slope-intercept y = m*x + b with m, b exact numbers; m*x may be x or x/2,
                  b may be omitted when 0, m*x may be omitted when m = 0
                  (horizontal line), term order and the side y is on do not
                  matter, and there must be exactly one x term and at most
                  one constant.
  simplified      no perfect-square factor under sqrt (perfect n-th power
  radical         under an n-th root), no radical in a denominator, no
                  fraction under a radical.
"""

from sympy import sympify

from caselib import spec_case

TOPIC = "forms"

CONFIRM = "confirm against the app's form_checker tests"


def form(op, src, want, why):
    sympify(src, evaluate=False)  # the question must parse as SymPy syntax
    return spec_case(op, [src], want, f"{why}; {CONFIRM}.")


def _factored():
    op = "is_factored"
    return [
        form(op, "(x - 2)*(x - 3)", True, "A product of irreducible linear factors"),
        form(op, "x**2 - 5*x + 6", False, "A reducible quadratic left as a sum is not factored"),
        form(op, "2*x + 4", False, "The common factor 2 is not pulled out, and over the integers every non-constant factor must be primitive"),
        form(op, "2*x + 3", True, "A primitive linear polynomial is a single irreducible factor (contrast 2*x + 4)"),
        form(op, "6*(x + 1)", True, "A constant factor times a primitive irreducible factor is factored"),
        form(op, "(x + 1)**2", True, "A power of an irreducible factor is factored"),
        form(op, "(x + 1)*(x + 1)", True, "A repeated factor written out twice is still a product of irreducible factors"),
        form(op, "x*(x**2 - 1)", False, "x**2 - 1 still factors as (x - 1)*(x + 1), so this is not fully factored"),
        form(op, "x**2 + 1", True, "Irreducible over the rationals, so it counts as factored"),
        form(op, "x**2 - 2", True, "Irreducible over the rationals (splitting it with sqrt(2) is factoring over the reals), so it counts as factored"),
        form(op, "(2*x + 2)*(x - 1)", False, "The factor 2*x + 2 still has the common factor 2"),
        form(op, "x*(x + 1) + 2*(x + 1)", False, "A sum of products (a factor-by-grouping step left unfinished) is not factored"),
        form(op, "-(x - 2)*(x + 3)", True, "A leading -1 is an allowed constant factor (sympify with evaluate=False reads this as (2 - x)*(x + 3), also factored)"),
        form(op, "x**3 - 8", False, "A difference of cubes left unfactored"),
        form(op, "(x - 2)*(x**2 + 2*x + 4)", True, "The quadratic factor is irreducible over the rationals"),
        form(op, "(x**2 - 4)*(x**2 + 4)", False, "x**2 - 4 still factors"),
        form(op, "x**2 + 2*x*y + y**2", False, "A multi-variable perfect square left unfactored"),
        form(op, "(x + 2)/2", True, "A rational constant factor (1/2) is allowed"),
        form(op, "3*x**2*y", True, "A monomial is a constant times powers of irreducible factors"),
    ]


def _expanded():
    op = "is_expanded"
    return [
        form(op, "x**2 + 2*x + 1", True, "A sum of monomials"),
        form(op, "(x + 1)**2", False, "A power of a sum is not expanded"),
        form(op, "(x + 1)*(x - 1)", False, "A product of sums is not expanded"),
        form(op, "2*(x + 3)", False, "A number times a sum has not been distributed"),
        form(op, "x*y + x*z", True, "A multi-variable sum of monomials"),
        form(op, "3*x**2 - 4*x*y + y**2", True, "A multi-variable sum of monomials with negative coefficients"),
        form(op, "x**2 + x + x + 1", True, "Decided: expansion only forbids products and powers of sums, so uncombined like terms still count as expanded"),
        form(op, "(2*x)**2 + 4*x + 1", True, "Decided: a power of a monomial is a monomial, so no sum is multiplied or raised to a power"),
        form(op, "-2*(x - 1)", False, "The -2 has not been distributed over the sum"),
        form(op, "(x + 1)**2 - 2*x", False, "A power of a sum inside a larger sum is still not expanded"),
        form(op, "(x + 1)/2", False, "Dividing a sum by 2 multiplies it by 1/2, which has not been distributed"),
        form(op, "x/2 + 1/2", True, "A sum of monomials with rational coefficients"),
    ]


def _simplified_fraction():
    op = "is_simplified_fraction"
    return [
        form(op, "(x + 1)/(x - 1)", True, "Numerator and denominator share no factor"),
        form(op, "(x**2 - 1)/(x - 1)", False, "Numerator and denominator share the factor x - 1"),
        form(op, "(x**2 + 5*x + 6)/(x**2 + x - 6)", False, "Numerator and denominator share the factor x + 3"),
        form(op, "(x + 2)/(x - 2)", True, "The fully reduced form of (x**2 + 5*x + 6)/(x**2 + x - 6)"),
        form(op, "(2*x + 4)/6", False, "Numerator and denominator share the integer factor 2"),
        form(op, "(x + 2)/3", True, "The fully reduced form of (2*x + 4)/6"),
        form(op, "6/8", False, "A numeric fraction whose numerator and denominator share the factor 2 (3/4 is the reduced form)"),
        form(op, "(1/x)/(1 + 1/x)", False, "A complex fraction (fractions nested inside the denominator) is not simplified"),
        form(op, "x/(x**2 + x)", False, "Numerator and denominator share the factor x"),
        form(op, "(x - 1)/(1 - x)", False, "Numerator and denominator share a factor up to sign (the fraction is -1)"),
        form(op, "(x**2 + 1)/(x + 1)", True, "An improper fraction whose numerator and denominator share no factor is simplified"),
        form(op, "x**2/x", False, "Read unevaluated, numerator and denominator share the factor x"),
        form(op, "x + 1", True, "A polynomial is a fraction with denominator 1"),
        form(op, "1/x + 1/y", False, "Decided: a sum of fractions is not a single simplified fraction"),
        form(op, "(2*x + 1)/(4*x + 2)", False, "Numerator and denominator share the factor 2*x + 1"),
        form(op, "(x + 1)/(2*(x - 1))", True, "A factored denominator with no factor in common with the numerator is fine"),
    ]


def _slope_intercept():
    op = "is_slope_intercept"
    return [
        form(op, "Eq(y, 2*x + 1)", True, "y = m*x + b"),
        form(op, "Eq(y, x)", True, "m*x may be just x and b = 0 may be omitted"),
        form(op, "Eq(y, -x)", True, "m = -1 written as -x"),
        form(op, "Eq(y, 5)", True, "Decided: a horizontal line y = b is slope-intercept form with the 0*x term omitted"),
        form(op, "Eq(y, 3 - 2*x)", True, "Decided: term order is not part of the form, so y = b + m*x counts"),
        form(op, "Eq(2*x + 1, y)", True, "Decided: which side y is on does not matter"),
        form(op, "Eq(y, x/2 + 3)", True, "A fractional slope written x/2 is m*x with m = 1/2"),
        form(op, "Eq(y, 0.5*x + 2)", True, "A typed decimal slope is an exact number"),
        form(op, "Eq(y, sqrt(2)*x + 1)", True, "m may be any exact number, including an irrational one"),
        form(op, "Eq(2*y, 4*x + 2)", False, "y is not isolated"),
        form(op, "Eq(y - 1, 2*(x - 3))", False, "Point-slope form, not slope-intercept form"),
        form(op, "Eq(2*x - y + 1, 0)", False, "General (standard) form, not slope-intercept form"),
        form(op, "Eq(y, 2*(x + 1))", False, "The right side m*(x + c) has not been distributed into m*x + b"),
        form(op, "Eq(y, 2*x + 3*x - 1)", False, "Like terms are not combined: exactly one x term is allowed"),
        form(op, "Eq(y, (4*x + 2)/2)", False, "A fraction of a sum is not m*x + b"),
        form(op, "Eq(y, x**2 + 1)", False, "Not a line"),
        form(op, "Eq(x, 3)", False, "A vertical line has no slope-intercept form"),
    ]


def _simplified_radical():
    op = "is_simplified_radical"
    return [
        form(op, "sqrt(8)", False, "8 has the perfect-square factor 4"),
        form(op, "2*sqrt(2)", True, "The simplified form of sqrt(8)"),
        form(op, "sqrt(4)", False, "The radicand is a perfect square"),
        form(op, "sqrt(7)", True, "7 is square-free"),
        form(op, "1/sqrt(2)", False, "A radical in the denominator"),
        form(op, "sqrt(2)/2", True, "The rationalised form of 1/sqrt(2)"),
        form(op, "sqrt(1/2)", False, "A fraction under the radical"),
        form(op, "3*sqrt(2)/2", True, "A rational coefficient times a square-free radical with an integer denominator"),
        form(op, "sqrt(3/4)", False, "A fraction under the radical"),
        form(op, "sqrt(3)/2", True, "The simplified form of sqrt(3/4)"),
        form(op, "sqrt(50)/5", False, "50 has the perfect-square factor 25"),
        form(op, "2/(1 + sqrt(3))", False, "A radical in a binomial denominator must be rationalised"),
        form(op, "sqrt(3) - 1", True, "The rationalised form of 2/(1 + sqrt(3))"),
        form(op, "(1 + sqrt(2))/2", True, "A sum with a square-free radical over an integer"),
        form(op, "sqrt(x**3)", False, "x**3 has the perfect-square factor x**2"),
        form(op, "x*sqrt(x)", True, "The simplified form of sqrt(x**3)"),
        form(op, "16**(1/3)", False, "Decided: the rule extends to n-th roots, and 16 has the perfect-cube factor 8"),
        form(op, "2*2**(1/3)", True, "Decided: the rule extends to n-th roots, and 2 is cube-free"),
    ]


def generate():
    return (
        _factored() + _expanded() + _simplified_fraction()
        + _slope_intercept() + _simplified_radical()
    )
