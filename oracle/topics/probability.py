"""Probability: what sympy.stats returns for finite random variables, and SymPy's counting functions.

The homework app's probability items (app_sympy_audit.md, "probability:
sympy.stats FiniteRV, P, given, binomial, Rational") are two-way tables,
conditional probability, dice/coins/binomial experiments and counting. This
topic records what the pinned SymPy returns for those calls, including where
its answer is surprising: an Eq between a random variable and a pair is
False on sight, conditioning a two-way table event gives 0, conditioning on
Eq(X % 2, 0) raises TypeError, a condition of probability 0 gives 0, a typed
decimal makes the answer a Float, FiniteRV takes probabilities that are
negative or do not sum to 1 unless check=True, and binomial and ff extend to
negative and fractional arguments.

Args are SymPy syntax strings, read with sympify:
  args[0]  the random variables: one declaration, or a list of them, e.g.
           "Die('X', 6)", "[Die('X', 6), Die('Y', 6)]", "Coin('C')",
           "Coin('C', 2/3)", "Binomial('X', 10, 1/2)",
           "Hypergeometric('X', 8, 5, 3)" (population, successes, draws),
           "FiniteRV('X', {1: 1/2, 2: 1/4, 3: 1/4})", read with the
           sympy.stats constructors Bernoulli, Binomial, Coin, Die, FiniteRV
           and Hypergeometric as locals. The quoted name is the variable's
           name in the events. Numbers are as sympify reads them ("1/2" is a
           Rational, "0.3" a Float) and "p" is a plain Symbol.
  events   expressions in those names, read with sympify with each declared
           variable as a local: "X > 3", "Eq(X, 3)", "Ne(X, 6)",
           "Eq(X % 2, 0)", "And(X > 3, Y > 3)", "Or(...)", "Not(...)". Coin
           faces are the Symbols H and T: "Eq(C, H)".

Two-way tables are one FiniteRV over (row, column) pairs, e.g.
"FiniteRV('G', {(9, 1): 30/200, ...})". Typed as "Eq(G, (9, 1))", a cell is
False, since SymPy evaluates an Eq between a RandomSymbol and a Tuple on
sight, and an Or of such cells folds to false while sympify reads it;
"Eq(G, (9, 1), evaluate=False)" keeps it as an event, and a row or column is
an Or of its cells. Each table event is recorded both ways.

Ops (single SymPy calls, source "sympy"):
  P         [rvs, event]: sympy.stats.P(event); or [rvs, event, condition]:
            P(event, condition).
  E         [rvs, expr]: sympy.stats.E(expr); or [rvs, expr, condition]:
            E(expr, condition).
  binomial  [n, k]: sympy.binomial(n, k), n read with sympify.
  ff        [n, k]: sympy.ff(n, k), the falling factorial.

A P or E error case records an exception raised anywhere in the case: while
the args are read (FiniteRV(..., check=True) validates its density when
args[0] is read, before P runs) or inside the P or E call itself.
"""

import sympy.stats as st
from sympy import Symbol, binomial, ff, sympify

from caselib import case, outcome

TOPIC = "probability"

STATS = {name: getattr(st, name) for name in [
    "Bernoulli", "Binomial", "Coin", "Die", "FiniteRV", "Hypergeometric",
]}
n_sym = Symbol("n", integer=True, nonnegative=True)

DIE = "Die('X', 6)"
DICE = "[Die('X', 6), Die('Y', 6)]"
COINS = "[Coin('C1'), Coin('C2')]"
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


def _cell(rv, pair):
    """A table cell kept as an event."""
    return f"Eq({rv}, {pair}, evaluate=False)"


def _cells(rv, *pairs):
    return f"Or({', '.join(_cell(rv, p) for p in pairs)})"


SPORT = _cells("G", (9, 1), (10, 1), (11, 1))
NO_SPORT = _cells("G", (9, 0), (10, 0), (11, 0))
GRADE9 = _cells("G", (9, 1), (9, 0))
GRADE10 = _cells("G", (10, 1), (10, 0))


def _rvs(decls):
    """The declared random variables by name."""
    parsed = sympify(decls, locals=STATS)
    rvs = parsed if isinstance(parsed, (list, tuple)) else [parsed]
    return {str(rv): rv for rv in rvs}


def _read(src, rvs):
    return sympify(src, locals=dict(rvs))


def prob(decls, event, cond=None, note=None):
    def run():
        rvs = _rvs(decls)
        if cond is None:
            return st.P(_read(event, rvs))
        return st.P(_read(event, rvs), _read(cond, rvs))

    args = [decls, event] if cond is None else [decls, event, cond]
    return outcome("P", args, run, note=note)


def expect(decls, expr, cond=None, note=None):
    def run():
        rvs = _rvs(decls)
        if cond is None:
            return st.E(_read(expr, rvs))
        return st.E(_read(expr, rvs), _read(cond, rvs))

    args = [decls, expr] if cond is None else [decls, expr, cond]
    return outcome("E", args, run, note=note)


def _dice_and_coins():
    out = []
    for event, note in [
        ("X > 3", None),
        ("X >= 3", None),
        ("Eq(X, 3)", None),
        ("Eq(X % 2, 0)", "Even roll."),
        ("Ne(X, 6)", None),
        ("Not(X > 4)", "Complement."),
        ("X > 6", "Impossible event."),
        ("X <= 6", "Certain event."),
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
    out.append(prob(COINS, "And(Eq(C1, H), Eq(C2, H))"))
    out.append(prob(COINS, "Or(Eq(C1, H), Eq(C2, H))"))
    for event, cond, note in [
        ("X > 3", "X > 1", None),
        ("Eq(X % 2, 0)", "X > 3", "Even given more than 3: {4, 6} out of {4, 5, 6}."),
        ("X > 3", "Eq(X % 2, 0)",
         "Conditioning on Eq(X % 2, 0): given() solves the condition for X with solveset, "
         "gets a ConditionSet and iterates over it."),
        ("X > 3", "X > 6", "The condition has probability 0."),
    ]:
        out.append(prob(DIE, event, cond, note=note))
    for event, cond, note in [
        ("Eq(X + Y, 7)", "Eq(X, 3)", "The events are independent: equals P(X + Y = 7)."),
        ("Eq(X, Y)", "Eq(X + Y, 8)", "Doubles given a sum of 8: (4, 4) out of 5 outcomes."),
        ("X + Y >= 10", "Eq(X, 6)", None),
    ]:
        out.append(prob(DICE, event, cond, note=note))
    out.append(prob(COINS, "Eq(C1, H)", "Eq(C2, H)", note="Independent flips."))
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
        ("Binomial('X', 3, p)", "Eq(X, 2)", "Symbolic success probability p."),
        ("Binomial('X', 4, 0.3)", "Eq(X, 2)", "A typed decimal probability: 0.3 is a Float, so SymPy computes with Floats."),
    ]:
        out.append(prob(decls, event, note=note))
    out.append(prob("Binomial('X', 5, 1/3)", "Eq(X, 2)", "X >= 1"))
    fin = "FiniteRV('X', {1: 1/2, 2: 1/4, 3: 1/4})"
    for decls, note in [
        (fin, None),
        ("FiniteRV('X', {1: 1/2, 2: 1/4})", "The probabilities sum to 3/4; FiniteRV checks the density only with check=True."),
        ("FiniteRV('X', {1: 3/2, 2: -1/2})",
         "A negative probability (the sum is 1); FiniteRV checks the density only with check=True."),
        ("FiniteRV('X', {1: 1/2, 2: 1/4}, check=True)",
         "check=True with probabilities that sum to 3/4. FiniteRV(check=True) validates the density "
         "while args[0] is read, so the ValueError comes from there, before P is called."),
        ("FiniteRV('X', {1: 3/2, 2: -1/2}, check=True)",
         "check=True with a negative probability. FiniteRV(check=True) validates the density "
         "while args[0] is read, so the ValueError comes from there, before P is called."),
    ]:
        out.append(prob(decls, "X > 1", note=note))
    out.append(prob("Bernoulli('B', 3/10)", "Eq(B, 1)"))

    for decls, expr, note in [
        (DIE, "X", None),
        (DIE, "X**2", None),
        (DIE, "2*X + 1", "Linearity: 2*E(X) + 1."),
        (DICE, "X + Y", None),
        ("Binomial('X', 10, 1/2)", "X", "n*p."),
        (fin, "X", None),
        ("FiniteRV('W', {-1: 5/6, 4: 1/6})", "W", "Expected winnings of a game: lose 1 on 5/6, win 4 on 1/6."),
        ("Binomial('X', 3, p)", "X", "Symbolic p: SymPy leaves an unsimplified sum that equals 3*p."),
    ]:
        out.append(expect(decls, expr, note=note))
    out.append(expect(DIE, "X", "X > 3", note="Conditional expectation: mean of {4, 5, 6}."))
    out.append(expect(DIE, "X", "X > 6",
                      note="The condition has probability 0; SymPy 1.14 returns the Python int 0 here."))
    return out


def _as_typed(src):
    """The same table event as typed, without evaluate=False."""
    return src.replace(", evaluate=False", "")


def _two_way_tables():
    out = []
    for decls, event, typed_note, note in [
        (TABLE, _cell("G", (11, 1)),
         "As typed, Eq(G, (11, 1)) is False: SymPy evaluates an Eq between a RandomSymbol "
         "and a Tuple on sight.",
         "Two-way table, one cell kept as an event: 35/200."),
        (TABLE, SPORT,
         "As typed, each Eq is False on sight and sympify folds the Or of them to false.",
         "Two-way table, column total: 110/200."),
        (TABLE, f"Or({GRADE9}, {SPORT})",
         "As typed, every Eq is False on sight, so both inner Ors and the outer Or fold to false.",
         "Two-way table, union: (50 + 110 - 30)/200."),
        (TABLE_INDEP, _cells("R", (1, 1), (0, 1)),
         "As typed, each Eq is False on sight and sympify folds the Or of them to false.",
         "2x2 table with independent row and column: P(column 1) = 15/50."),
    ]:
        out.append(prob(decls, _as_typed(event), note=typed_note))
        out.append(prob(decls, event, note=note))
    given_note = ("; P(A, B) goes through given(), which moves the event onto the conditional space with "
                  "xreplace; that rebuilds the Eq with evaluation on, so given() returns False.")
    for decls, event, cond, why in [
        (TABLE, _cell("G", (9, 1)), GRADE9,
         "P(sport | grade 9). SymPy returns 0, although the table ratio P(A & B)/P(B) is 30/50"),
        (TABLE, _cell("G", (10, 0)), NO_SPORT,
         "P(grade 10 | no sport). SymPy returns 0, although the table ratio P(A & B)/P(B) is 55/90"),
        (TABLE, GRADE10, SPORT,
         "P(grade 10 | sport). SymPy returns 0, although the table ratio P(A & B)/P(B) is 45/110"),
        (TABLE_INDEP, _cell("R", (1, 1)), _cells("R", (1, 1), (1, 0)),
         "P(column 1 | row 1). SymPy returns 0, although the table ratio P(A & B)/P(B) is 6/20"),
    ]:
        out.append(prob(decls, _as_typed(event), _as_typed(cond),
                        note="As typed, the event and the condition are both False."))
        out.append(prob(decls, event, cond, note=why + given_note))
    return out


def _counting():
    out = []
    for n, k, note in [
        (52, 5, "Five-card hands."),
        (10, 3, None),
        (5, 0, "Choosing nothing."),
        (0, 0, None),
        (5, 7, "k > n."),
        (5, -1, "Negative k."),
        (-5, 2, "Negative n: the general binomial coefficient (-5)*(-6)/2."),
        ("1/2", 2, "Rational n (binomial series): (1/2)*(-1/2)/2."),
        ("5/2", 2, "Rational n: (5/2)*(3/2)/2."),
        (n_sym, 2, "Symbolic n."),
        (n_sym, 1, None),
    ]:
        out.append(case("binomial", [n, k], binomial(sympify(n), k), note=note))
    for n, k, note in [
        (10, 3, None),
        (5, 5, "All arrangements: 5!."),
        (5, 0, None),
        (3, 5, "k > n: the product passes through 0."),
        (5, -1, "Negative k: SymPy extends the falling factorial, ff(x, -1) = 1/(x + 1)."),
        (-3, 2, "Negative n: (-3)*(-4)."),
        (n_sym, 2, "Symbolic n."),
    ]:
        out.append(case("ff", [n, k], ff(n, k), note=note))
    return out


def generate():
    return _dice_and_coins() + _binomial_and_finite() + _two_way_tables() + _counting()
