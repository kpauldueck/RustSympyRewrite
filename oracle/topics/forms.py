"""Form-check inputs: how SymPy reads and rewrites answers written in a form.

The homework app checks whether a student's answer is written in a required
form (factored, expanded, simplified fraction, slope-intercept, simplified
radical). Those checks sit on SymPy: the app's checker uses factor (factored),
expand (expanded), numer/denom/gcd (simplified fraction) and the Mul/Add
structure of the parsed expression (slope-intercept), and its simplified-radical
spike reads input with sympify(..., evaluate=False). This topic records what
the pinned SymPy does on the inputs those checks see. It does not decide
whether an input is "in the form": SymPy has no such check.

args[0] is the input in SymPy syntax. Every case is one SymPy call:

  sympify   options {"evaluate": False}: sympify(s, evaluate=False), the
            unevaluated tree a form check needs (evaluation rewrites the very
            form being checked). Recorded for every input. Its str() and
            latex() show quirks of unevaluated trees, e.g. x - 2 is stored as
            Add(x, Mul(-1, 2)) and prints as x - 1*2.
  sympify   no options: sympify(s), the evaluated parse. Recorded for every
            input except the simplified-fraction ones, whose evaluated parse
            the fraction() parts below already show. The app's parse paths
            evaluate, so an evaluated tree is what its form checks see today,
            and evaluation is what turns e.g. a factored 6*(x + 1) into
            6*x + 6. factor and expand alone do not pin it: factor(6*(x + 1))
            and factor(6*x + 6) give the same answer.
  factor    factor(sympify(s)), for the factored-form inputs.
  expand    expand(sympify(s)), for the expanded-form inputs.
  fraction  fraction(sympify(s)), for the simplified-fraction inputs; SymPy
            returns a Python tuple (numer, denom), recorded as its str().
  gcd       gcd(numer, denom) on the two parts fraction() just returned
            (args are their str()), whenever denom is not 1.
"""

import sympy as sp

from caselib import case

TOPIC = "forms"

# Inputs, grouped by the form each one probes. FACTORED, EXPANDED and
# FRACTIONS are (input, note on the group's SymPy call, or None); LINES and
# RADICALS have no group call beyond sympify.
FACTORED = [
    ("(x - 2)*(x - 3)", None),
    ("x**2 - 5*x + 6", None),
    ("2*x + 4", "factor pulls out the integer content."),
    ("2*x + 3", None),
    ("6*(x + 1)", "factor pulls the integer content 6 back out of the evaluated 6*x + 6."),
    ("(x + 1)**2", None),
    ("(x + 1)*(x + 1)", None),
    ("x*(x**2 - 1)", None),
    ("x**2 + 1", None),
    ("x**2 - 2", "Irreducible over the rationals, factor's default domain."),
    ("(2*x + 2)*(x - 1)", "factor pulls the content 2 out of the factor 2*x + 2."),
    ("x*(x + 1) + 2*(x + 1)", None),
    ("-(x - 2)*(x + 3)", "factor moves the sign of the evaluated (2 - x)*(x + 3) back out in front."),
    ("x**3 - 8", None),
    ("(x - 2)*(x**2 + 2*x + 4)", None),
    ("(x**2 - 4)*(x**2 + 4)", None),
    ("x**2 + 2*x*y + y**2", None),
    ("(x + 2)/2", "factor pulls the rational content 1/2 back out of the evaluated x/2 + 1."),
    ("3*x**2*y", None),
]

EXPANDED = [
    ("x**2 + 2*x + 1", None),
    ("(x + 1)**2", None),
    ("(x + 1)*(x - 1)", None),
    ("2*(x + 3)", None),
    ("x*y + x*z", None),
    ("3*x**2 - 4*x*y + y**2", None),
    ("x**2 + x + x + 1", None),
    ("(2*x)**2 + 4*x + 1", None),
    ("-2*(x - 1)", None),
    ("(x + 1)**2 - 2*x", "The 2*x cancels against the expanded square."),
    ("(x + 1)/2", None),
    ("x/2 + 1/2", None),
]

FRACTIONS = [
    ("(x + 1)/(x - 1)", None),
    ("(x**2 - 1)/(x - 1)", "Evaluation does not cancel a common polynomial factor."),
    ("(x**2 + 5*x + 6)/(x**2 + x - 6)", None),
    ("(x + 2)/(x - 2)", None),
    ("(2*x + 4)/6", "Evaluation distributes 1/6 over the sum (x/3 + 2/3). fraction() does not "
                    "look inside a sum, so the denominator is 1."),
    ("(x + 2)/3", "Evaluation distributes 1/3 over the sum (x/3 + 2/3). fraction() does not "
                  "look inside a sum, so the denominator is 1."),
    ("6/8", "Evaluation reduces the integer quotient to 3/4."),
    ("(1/x)/(1 + 1/x)", "fraction() does not clear the 1/x nested in the denominator."),
    ("x/(x**2 + x)", None),
    ("(x - 1)/(1 - x)", "Numerator and denominator differ only in sign."),
    ("(x**2 + 1)/(x + 1)", None),
    ("x**2/x", "Evaluation cancels x**2/x to x before fraction() sees it."),
    ("x + 1", None),
    ("1/x + 1/y", "fraction() does not combine a sum of fractions."),
    ("(2*x + 1)/(4*x + 2)", None),
    ("(x + 1)/(2*(x - 1))", "Evaluation distributes the 2 inside the denominator."),
]

LINES = [
    "Eq(y, 2*x + 1)",
    "Eq(y, x)",
    "Eq(y, -x)",
    "Eq(y, 5)",
    "Eq(y, 3 - 2*x)",
    "Eq(2*x + 1, y)",
    "Eq(y, x/2 + 3)",
    "Eq(y, 0.5*x + 2)",
    "Eq(y, sqrt(2)*x + 1)",
    "Eq(2*y, 4*x + 2)",
    "Eq(y - 1, 2*(x - 3))",
    "Eq(2*x - y + 1, 0)",
    "Eq(y, 2*(x + 1))",
    "Eq(y, 2*x + 3*x - 1)",
    "Eq(y, (4*x + 2)/2)",
    "Eq(y, x**2 + 1)",
    "Eq(x, 3)",
]

RADICALS = [
    "sqrt(8)",
    "2*sqrt(2)",
    "sqrt(4)",
    "sqrt(7)",
    "1/sqrt(2)",
    "sqrt(2)/2",
    "sqrt(1/2)",
    "3*sqrt(2)/2",
    "sqrt(3/4)",
    "sqrt(3)/2",
    "sqrt(50)/5",
    "2/(1 + sqrt(3))",
    "sqrt(3) - 1",
    "(1 + sqrt(2))/2",
    "sqrt(x**3)",
    "x*sqrt(x)",
    "16**(1/3)",
    "2*2**(1/3)",
]

# Notes for sympify(s) (evaluated), keyed by input.
EVALUATED = {
    # factored-form inputs
    "(x - 2)*(x - 3)": "Evaluation reorders the factors: (x - 3)*(x - 2).",
    "6*(x + 1)": "Evaluation distributes the 6 over the sum: 6*x + 6.",
    "(x + 1)*(x + 1)": "Evaluation collects the repeated factor into a power.",
    "x*(x + 1) + 2*(x + 1)": "Evaluation distributes 2*(x + 1) but not x*(x + 1).",
    "-(x - 2)*(x + 3)": "Evaluation folds the leading minus into the first factor: (2 - x)*(x + 3).",
    "(x + 2)/2": "Evaluation distributes 1/2 over the sum: x/2 + 1.",
    # expanded-form inputs
    "2*(x + 3)": "Evaluation distributes a number over a sum.",
    "x**2 + x + x + 1": "Evaluation combines the repeated x terms.",
    "(2*x)**2 + 4*x + 1": "Evaluation turns (2*x)**2 into 4*x**2.",
    "-2*(x - 1)": "Evaluation distributes -2 over the sum: 2 - 2*x.",
    "(x + 1)/2": "Evaluation distributes 1/2 over the sum: x/2 + 1/2.",
    # slope-intercept inputs
    "Eq(y, 0.5*x + 2)": "A typed decimal slope stays a Float.",
    "Eq(y - 1, 2*(x - 3))": "Evaluation distributes the 2 on the right side.",
    "Eq(y, 2*(x + 1))": "Evaluation distributes the 2 over x + 1.",
    "Eq(y, 2*x + 3*x - 1)": "Evaluation combines the like x terms.",
    "Eq(y, (4*x + 2)/2)": "Evaluation distributes 1/2 over the sum.",
    # radical inputs
    "sqrt(8)": "Evaluation pulls the square factor 4 out of the radical.",
    "sqrt(4)": "Evaluation takes the root of a perfect square.",
    "1/sqrt(2)": "Evaluation moves a numeric square root out of the denominator.",
    "sqrt(1/2)": "Evaluation takes the fraction out from under the radical.",
    "sqrt(3/4)": "Evaluation takes the fraction out from under the radical.",
    "sqrt(50)/5": "Evaluation pulls 5 out of sqrt(50), which cancels the 5 below.",
    "2/(1 + sqrt(3))": "Evaluation leaves a binomial radical denominator alone.",
    "(1 + sqrt(2))/2": "Evaluation distributes 1/2 over the sum.",
    "sqrt(x**3)": "With no assumptions on x, the radical is not split.",
    "x*sqrt(x)": "Evaluation merges the product into one power of x.",
    "16**(1/3)": "Evaluation pulls the cube factor 8 out of a cube root.",
}

# Notes for sympify(s, evaluate=False), keyed by input.
UNEVALUATED = {
    "(x - 2)*(x - 3)": "Unevaluated subtraction stores x - 2 as Add(x, Mul(-1, 2)), which prints as x - 1*2.",
    "-(x - 2)*(x + 3)": "Even unevaluated, the leading minus is folded into the first sum: (2 - x)*(x + 3).",
    "(x + 1)*(x + 1)": "Unevaluated, the repeated factor stays written out twice.",
    "x**2 + x + x + 1": "Unevaluated, the repeated x terms stay separate.",
    "(2*x)**2 + 4*x + 1": "Unevaluated, (2*x)**2 stays a power of a product.",
    "-2*(x - 1)": ("Unevaluated, -2 stays one Integer(-2) factor: Mul(-2, x - 1). srepr() still "
                   "prints Mul(Integer(-1), Integer(2), ...) because it lists as_ordered_factors()."),
    "2*(x + 3)": "Unevaluated, the number is not distributed over the sum.",
    "6/8": "Unevaluated, an integer quotient stays Mul(6, Pow(8, -1)).",
    "x**2/x": "Unevaluated, both powers of x stay.",
    "(1/x)/(1 + 1/x)": "A nested fraction, read without evaluation.",
    "(x + 1)/(2*(x - 1))": "str() of this unevaluated quotient prints doubled parentheses.",
    "1/sqrt(2)": "Unevaluated, the radical stays in the denominator as Mul(1, Pow(sqrt(2), -1)).",
    "sqrt(1/2)": "Unevaluated, the fraction stays under the radical.",
    "sqrt(3) - 1": "Unevaluated subtraction of 1 is Mul(-1, 1), printed -1*1.",
    "x*sqrt(x)": "Unevaluated, x*sqrt(x) stays a product.",
    "16**(1/3)": "Unevaluated, the exponent 1/3 is Mul(1, Pow(3, -1)), not Rational(1, 3).",
    "Eq(y, 0.5*x + 2)": "A typed decimal is a Float even unevaluated.",
    "Eq(y - 1, 2*(x - 3))": "Unevaluated subtraction on both sides prints as - 1*1 and - 1*3.",
}

GCD_NOTES = {
    "(x - 1)/(1 - x)": "gcd of a polynomial and its negative.",
    "(2*x + 1)/(4*x + 2)": "The denominator is 2 times the numerator.",
}


def _parses(src, seen, evaluated=True):
    """sympify(s, evaluate=False), then (if `evaluated`) sympify(s); once per input."""
    if src in seen:
        return []
    seen.add(src)
    out = [case("sympify", [src], sp.sympify(src, evaluate=False),
                options={"evaluate": False}, note=UNEVALUATED.get(src))]
    if evaluated:
        out.append(case("sympify", [src], sp.sympify(src), note=EVALUATED.get(src)))
    return out


def _fraction_cases(src, note):
    numer, denom = sp.fraction(sp.sympify(src))
    out = [case("fraction", [src], (numer, denom), note=note)]
    if denom != 1:
        # The gcd args are str(numer), str(denom); they must read back as the
        # same expressions.
        assert sp.sympify(str(numer)) == numer and sp.sympify(str(denom)) == denom, src
        out.append(case("gcd", [numer, denom], sp.gcd(numer, denom), note=GCD_NOTES.get(src)))
    return out


def generate():
    seen = set()
    out = []
    for src, note in FACTORED:
        out += _parses(src, seen)
        out.append(case("factor", [src], sp.factor(sp.sympify(src)), note=note))
    for src, note in EXPANDED:
        out += _parses(src, seen)
        out.append(case("expand", [src], sp.expand(sp.sympify(src)), note=note))
    for src, note in FRACTIONS:
        out += _parses(src, seen, evaluated=False)
        out += _fraction_cases(src, note)
    for src in LINES + RADICALS:
        out += _parses(src, seen)
    return out
