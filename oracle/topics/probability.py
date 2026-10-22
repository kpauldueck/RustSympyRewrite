"""Probability: sympy.stats keys for finite probability and counting.

The homework app's probability items (app_sympy_audit.md, "probability:
sympy.stats FiniteRV, P, given, binomial, Rational") are two-way tables,
conditional probability, dice/coins/binomial experiments and counting. Every
answer is an exact Rational.

Args are SymPy syntax strings:
  args[0]  the random variables: one declaration, or a list of them, e.g.
           "Die('X', 6)", "[Die('X', 6), Die('Y', 6)]", "Coin('C')",
           "Coin('C', 2/3)", "Binomial('X', 10, 1/2)",
           "Hypergeometric('X', 10, 4, 3)" (population, successes, draws),
           "FiniteRV('X', {1: 1/2, 2: 1/4, 3: 1/4})". The quoted name is the
           variable's name in the events. Numbers are exact ("1/2" is a
           Rational).
  events   relations in those names: "X > 3", "Eq(X, 3)", "Ne(X, 6)",
           "Eq(X % 2, 0)", "And(X > 3, Y > 3)", "Or(...)", "Not(...)".
           Coin faces are the symbols H and T: "Eq(C, H)".

Two-way tables are one FiniteRV over (row, column) pairs with integer labels,
probabilities written as count/total, e.g. "FiniteRV('G', {(9, 1): 30/200,
...})"; a cell is "Eq(G, (9, 1))" and a row or column is an Or of its cells.
An Eq between a random variable and a pair is an event. SymPy evaluates
Eq(G, (9, 1)) to False on sight and silently answers 0, so every table case is
a spec case (sympy_gives records that 0): P is computed by SymPy with
Eq(..., evaluate=False), and P_given as P(A & B)/P(B) the same way, because
SymPy's P(A, B) still answers 0 on a table even then. (The table variable is
not called S, which sympify reads as SymPy's S singleton.)

Ops:
  P        [rvs, event]: sympy.stats.P(event).
  P_given  [rvs, event, condition]: P(event, condition).
  E        [rvs, expr] or [rvs, expr, condition]: sympy.stats.E.
  binomial [n, k]: sympy.binomial(n, k), the general binomial coefficient
           (symbolic n stays unevaluated unless SymPy simplifies it).
  nCr, nPr [n, r]: counting. nCr is sympy.binomial(n, r) and nPr is the
           falling factorial sympy.ff(n, r) = n!/(n - r)!. Counting takes an
           integer n >= 0; r < 0 or r > n gives 0 ways.

Spec decisions: conditioning on an event of probability 0 is refused
("invalid:zero_probability_condition"); a FiniteRV whose probabilities are
negative or do not sum to 1 is refused; typed decimals in a declaration are
exact (0.3 is 3/10), as in parse_latex.
"""

import re

import sympy as sp
import sympy.stats as st
from sympy import Eq, Symbol, binomial, ff, sympify

from caselib import case, spec_case

TOPIC = "probability"

STATS = {name: getattr(st, name) for name in [
    "Bernoulli", "Binomial", "Coin", "Die", "DiscreteUniform", "FiniteRV", "Hypergeometric",
]}
p_sym = Symbol("p", positive=True)
n_sym = Symbol("n", integer=True, nonnegative=True)

DIE = "Die('X', 6)"
DICE = "[Die('X', 6), Die('Y', 6)]"
# Two-way table: grade 9/10/11 by plays a sport (1) or not (0), 200 students.
#            sport  none  total
#   grade 9    30    20     50
#   grade 10   45    55    100
#   grade 11   35    15     50
#   total     110    90    200
TABLE = ("FiniteRV('G', {(9, 1): 30/200, (9, 0): 20/200, (10, 1): 45/200, "
         "(10, 0): 55/200, (11, 1): 35/200, (11, 0): 15/200})")
# A 2x2 table whose row and column are independent: 6, 14 / 9, 21 (50).
TABLE_INDEP = "FiniteRV('R', {(1, 1): 6/50, (1, 0): 14/50, (0, 1): 9/50, (0, 0): 21/50})"
SPORT = "Or(Eq(G, (9, 1)), Eq(G, (10, 1)), Eq(G, (11, 1)))"
NO_SPORT = "Or(Eq(G, (9, 0)), Eq(G, (10, 0)), Eq(G, (11, 0)))"
GRADE9 = "Or(Eq(G, (9, 1)), Eq(G, (9, 0)))"
GRADE10 = "Or(Eq(G, (10, 1)), Eq(G, (10, 0)))"


def _rvs(decls):
    """The declared random variables by name."""
    parsed = sympify(decls, locals={**STATS, "p": p_sym})
    rvs = parsed if isinstance(parsed, (list, tuple)) else [parsed]
    return {str(rv): rv for rv in rvs}


def _eq_event(lhs, rhs):
    return Eq(lhs, rhs, evaluate=False)


def _event(src, rvs, pairs=False):
    names = dict(rvs)
    names["H"], names["T"] = Symbol("H"), Symbol("T")
    if pairs:
        names["Eq"] = _eq_event
    return sympify(src, locals=names)


def _options(decls):
    """A symbolic success probability p is declared positive (args are strings)."""
    if re.search(r"\bp\b", decls):
        return {"assumptions": {"p": {"positive": True}}}
    return None


def prob(decls, event, compare="exact", note=None, pairs=False):
    rvs = _rvs(decls)
    res = st.P(_event(event, rvs, pairs))
    return case("P", [decls, event], res, compare, options=_options(decls), note=note)


def prob_given(decls, event, cond, note=None, pairs=False):
    rvs = _rvs(decls)
    res = st.P(_event(event, rvs, pairs), _event(cond, rvs, pairs))
    return case("P_given", [decls, event, cond], res, options=_options(decls), note=note)


def expect(decls, expr, cond=None, compare="exact", note=None):
    rvs = _rvs(decls)
    if cond is None:
        res = st.E(_event(expr, rvs))
        args = [decls, expr]
    else:
        res = st.E(_event(expr, rvs), _event(cond, rvs))
        args = [decls, expr, cond]
    return case("E", args, res, compare, options=_options(decls), note=note)


def _dice_and_coins():
    out = []
    for event, note in [
        ("X > 3", None),
        ("X >= 3", None),
        ("Eq(X, 3)", None),
        ("Eq(X % 2, 0)", "Even roll."),
        ("Ne(X, 6)", None),
        ("Not(X > 4)", "Complement."),
        ("X > 6", "Impossible event: 0."),
        ("X <= 6", "Certain event: 1."),
    ]:
        out.append(prob(DIE, event, note=note))
    out.append(prob("Die('D', 20)", "D >= 15", note="A 20-sided die."))
    for event, note in [
        ("Eq(X + Y, 7)", "Sum of two dice."),
        ("X + Y > 9", None),
        ("Eq(X, Y)", "Doubles."),
        ("Eq(X*Y, 12)", None),
        ("And(X > 3, Y > 3)", "Independent dice: 1/2 * 1/2."),
        ("Or(X > 3, Y > 3)", "At least one: 1 - 1/4."),
    ]:
        out.append(prob(DICE, event, note=note))
    out.append(prob("Coin('C')", "Eq(C, H)"))
    out.append(prob("Coin('C', 2/3)", "Eq(C, T)", note="Biased coin: P(H) = 2/3."))
    out.append(prob("[Coin('C1'), Coin('C2')]", "And(Eq(C1, H), Eq(C2, H))"))
    out.append(prob("[Coin('C1'), Coin('C2')]", "Or(Eq(C1, H), Eq(C2, H))"))
    out.append(prob_given(DIE, "X > 3", "X > 1"))
    out.append(prob_given(DIE, "Eq(X % 2, 0)", "X > 3", note="Even given more than 3: {4, 6} out of {4, 5, 6}."))
    even = "Eq(X % 2, 0)"
    out.append(spec_case(
        "P_given", [DIE, "X > 3", even], "2/3",
        "Given an even roll: {4, 6} out of {2, 4, 6}. SymPy 1.14 crashes conditioning on Eq(X % 2, 0) "
        "(Or(Eq(X, 2), Eq(X, 4), Eq(X, 6)) or Ne(X % 2, 1) works).",
        sympy_gives=lambda: st.P(_event("X > 3", _rvs(DIE)), _event(even, _rvs(DIE))),
    ))
    for event, cond, note in [
        ("Eq(X + Y, 7)", "Eq(X, 3)", "Equals P(X + Y = 7): the events are independent."),
        ("Eq(X, Y)", "Eq(X + Y, 8)", "Doubles given a sum of 8: (4, 4) out of 5 outcomes."),
        ("X + Y >= 10", "Eq(X, 6)", None),
    ]:
        out.append(prob_given(DICE, event, cond, note=note))
    out.append(prob_given("[Coin('C1'), Coin('C2')]", "Eq(C1, H)", "Eq(C2, H)", note="Independent flips."))
    out.append(spec_case(
        "P_given", [DIE, "X > 3", "X > 6"], "invalid:zero_probability_condition",
        "P(A | B) is undefined when P(B) = 0; SymPy silently answers 0.",
        compare="error", sympy_gives=lambda: st.P(_event("X > 3", _rvs(DIE)), _event("X > 6", _rvs(DIE))),
    ))
    return out


def _binomial_and_finite():
    out = []
    for decls, event, note in [
        ("Binomial('X', 10, 1/2)", "Eq(X, 5)", None),
        ("Binomial('X', 10, 1/2)", "X >= 8", None),
        ("Binomial('X', 5, 1/3)", "X >= 1", "At least one success: 1 - (2/3)**5."),
        ("Binomial('X', 5, 1/3)", "X <= 1", None),
        ("Binomial('X', 3, 0)", "Eq(X, 0)", "p = 0: no successes for certain."),
        ("Hypergeometric('X', 8, 5, 3)", "Eq(X, 2)",
         "Without replacement: 2 of 3 drawn from 5 red and 3 blue, C(5, 2)*C(3, 1)/C(8, 3)."),
    ]:
        out.append(prob(decls, event, note=note))
    out.append(prob("Binomial('X', 3, p)", "Eq(X, 2)", compare="equivalent", note="Symbolic p: 3*p**2*(1 - p)."))
    out.append(prob_given("Binomial('X', 5, 1/3)", "Eq(X, 2)", "X >= 1"))
    fin = "FiniteRV('X', {1: 1/2, 2: 1/4, 3: 1/4})"
    out.append(prob(fin, "X > 1"))
    out.append(prob("Bernoulli('B', 3/10)", "Eq(B, 1)"))

    dec = "Binomial('X', 4, 0.3)"
    out.append(spec_case(
        "P", [dec, "Eq(X, 2)"], "1323/5000",
        "A typed decimal probability is exact (0.3 = 3/10), as in parse_latex; SymPy computes with a Float.",
        sympy_gives=lambda: st.P(_event("Eq(X, 2)", _rvs(dec))),
    ))
    short = "FiniteRV('X', {1: 1/2, 2: 1/4})"
    out.append(spec_case(
        "P", [short, "X > 1"], "invalid:probabilities_do_not_sum_to_1",
        "The probabilities sum to 3/4; SymPy accepts the distribution and answers anyway.",
        compare="error", sympy_gives=lambda: st.P(_event("X > 1", _rvs(short))),
    ))
    neg = "FiniteRV('X', {1: 3/2, 2: -1/2})"
    out.append(spec_case(
        "P", [neg, "X > 1"], "invalid:negative_probability",
        "A negative probability (though the sum is 1); SymPy answers -1/2.",
        compare="error", sympy_gives=lambda: st.P(_event("X > 1", _rvs(neg))),
    ))

    for decls, expr, note in [
        (DIE, "X", None),
        (DIE, "X**2", None),
        (DIE, "2*X + 1", "Linearity: 2*E(X) + 1."),
        (DICE, "X + Y", None),
        ("Binomial('X', 10, 1/2)", "X", "n*p."),
        (fin, "X", None),
        ("FiniteRV('W', {-1: 5/6, 4: 1/6})", "W", "Expected winnings of a game: lose 1 on 5/6, win 4 on 1/6."),
    ]:
        out.append(expect(decls, expr, note=note))
    out.append(expect("Binomial('X', 3, p)", "X", compare="equivalent",
                      note="Symbolic p: SymPy leaves an unsimplified sum that equals 3*p."))
    out.append(expect(DIE, "X", "X > 3", note="Conditional expectation: mean of {4, 5, 6}."))
    out.append(spec_case(
        "E", [DIE, "X", "X > 6"], "invalid:zero_probability_condition",
        "E(X | B) is undefined when P(B) = 0; SymPy silently answers 0.",
        compare="error", sympy_gives=lambda: st.E(_event("X", _rvs(DIE)), _event("X > 6", _rvs(DIE))),
    ))
    return out


def _two_way_tables():
    out = []
    def table_p(decls, event, why):
        rvs = _rvs(decls)
        want = st.P(_event(event, rvs, pairs=True))
        return spec_case(
            "P", [decls, event], sp.sstr(want),
            f"{why} Eq between a random variable and a pair is an event; SymPy evaluates such an Eq to "
            "False on sight and silently answers 0. The answer is computed by SymPy with Eq(..., evaluate=False).",
            sympy_gives=lambda: st.P(_event(event, rvs)),
        )

    for event, why in [
        ("Eq(G, (11, 1))", "Two-way table, one cell: 35/200."),
        (SPORT, "Two-way table, column total: 110/200."),
        (f"Or({GRADE9}, {SPORT})", "Two-way table, union: (50 + 110 - 30)/200."),
    ]:
        out.append(table_p(TABLE, event, why))

    def table_given(decls, event, cond, why):
        rvs = _rvs(decls)
        A, B = _event(event, rvs, pairs=True), _event(cond, rvs, pairs=True)
        want = st.P(sp.And(A, B)) / st.P(B)
        return spec_case(
            "P_given", [decls, event, cond], sp.sstr(want),
            f"{why} SymPy 1.14's P(A, B) answers 0 for events on a pair-valued FiniteRV (even with "
            "Eq(..., evaluate=False)); the answer is P(A & B)/P(B), computed by SymPy.",
            sympy_gives=lambda: st.P(A, B),
        )

    for event, cond, why in [
        ("Eq(G, (9, 1))", GRADE9, "P(sport | grade 9) = 30/50."),
        ("Eq(G, (10, 0))", NO_SPORT, "P(grade 10 | no sport) = 55/90."),
        (GRADE10, SPORT, "P(grade 10 | sport) = 45/110."),
    ]:
        out.append(table_given(TABLE, event, cond, why))
    row1 = "Or(Eq(R, (1, 1)), Eq(R, (1, 0)))"
    col1 = "Or(Eq(R, (1, 1)), Eq(R, (0, 1)))"
    out.append(table_p(TABLE_INDEP, col1, "2x2 table with independent row and column: P(column 1) = 15/50."))
    out.append(table_given(TABLE_INDEP, "Eq(R, (1, 1))", row1,
                           "P(column 1 | row 1) = 6/20, equal to P(column 1): independent."))
    return out


def _counting():
    out = []
    for n, k, note in [
        (52, 5, "Five-card hands."),
        (5, 7, "k > n: 0."),
        (n_sym, 2, "Symbolic n stays unevaluated."),
        (n_sym, 1, None),
        ("1/2", 2, "General binomial coefficient (binomial series): (1/2)*(-1/2)/2."),
    ]:
        args = [n, k]
        val = binomial(sympify(n), k)
        out.append(case("binomial", args, val, note=note))
    for n, r, note in [
        (10, 3, None),
        (5, 0, "One way to choose nothing."),
        (0, 0, None),
        (5, 7, "r > n: no ways."),
        (5, -1, "r < 0: no ways (SymPy's convention for binomial)."),
    ]:
        out.append(case("nCr", [n, r], binomial(n, r), note=note))
    for n, r, note in [
        (10, 3, None),
        (5, 5, "All arrangements: 5!."),
        (5, 0, None),
        (3, 5, "r > n: no ways (the falling factorial passes through 0)."),
        (n_sym, 2, "Symbolic n: n*(n - 1)."),
    ]:
        out.append(case("nPr", [n, r], ff(n, r), note=note))
    out.append(spec_case(
        "nPr", [5, -1], "0",
        "Counting: there are no arrangements of -1 objects, matching nCr(5, -1) = 0; "
        "SymPy's ff(5, -1) extends the falling factorial to 1/6.",
        sympy_gives=lambda: ff(5, -1),
    ))
    out.append(spec_case(
        "nCr", [-5, 2], "invalid:negative_n",
        "Counting needs n >= 0; SymPy's general binomial(-5, 2) = 15 is not a count.",
        compare="error", sympy_gives=lambda: binomial(-5, 2),
    ))
    out.append(spec_case(
        "nPr", [-3, 2], "invalid:negative_n",
        "Counting needs n >= 0; SymPy's falling factorial ff(-3, 2) = (-3)*(-4) = 12 is not a count.",
        compare="error", sympy_gives=lambda: ff(-3, 2),
    ))
    out.append(spec_case(
        "nCr", ["5/2", 2], "invalid:non_integer",
        "Counting needs whole numbers; SymPy's binomial(5/2, 2) = 15/8.",
        compare="error", sympy_gives=lambda: binomial(sp.Rational(5, 2), 2),
    ))
    return out


def generate():
    return _dice_and_coins() + _binomial_and_finite() + _two_way_tables() + _counting()
