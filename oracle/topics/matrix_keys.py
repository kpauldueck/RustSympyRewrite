"""Matrix keys (Unit 4): what SymPy returns for the matrix calls on the app's roadmap.

The homework app does not use SymPy for matrices today: per
app_sympy_audit.md, SymPy "is NOT used by the numeric/matrix/... graders
(those use ast/Decimal/Fraction)". The audit lists the Unit 4 matrix keys
under "Calls the roadmap wants", to be built at compile time: Matrix(key),
det, inv, A*B, .T; is_invertible / det != 0 to refuse singular seeds;
(M**n)*v and eigenvects for Markov state prediction and steady state;
applyfunc(nsimplify) for exact display. This topic records what the pinned
SymPy returns for those planned calls, including where its answer is
surprising: det leaves sin(t)**2 + cos(t)**2 unsimplified, and inv leaves its
(1, 1) entry as -sin(t)**2/cos(t) + 1/cos(t) rather than cos(t); a decimal key
stays Float; and non-square or singular inputs raise (NonSquareMatrixError,
NonInvertibleMatrixError, ShapeError).

Args are matrices in SymPy syntax, Matrix([[row], [row], ...]); a column
vector is Matrix([[a], [b]]). Matrices with Float entries are passed as the
typed strings, read with sympify, since str() of a Float matrix pads every
entry to 15 digits. Answers are SymPy's str() of the result, on one line.

Single SymPy calls (source "sympy"; M, A, B are the matrix args):
  det                 [M]: M.det().
  inv                 [M]: M.inv().
  transpose           [M]: M.T.
  matmul              [A, B]: A*B.
  matpow              [M, n]: M**n, n an integer or a symbol.
  eigenvects          [M]: M.eigenvects(), a list of (eigenvalue, algebraic
                      multiplicity, [basis vectors]) in SymPy's order, printed
                      on one line as "[(val, mult, [Matrix(...), ...]), ...]"
                      with sstr() for the eigenvalue and str() for each
                      vector.
  nsimplify_entries   [M]: M.applyfunc(nsimplify), M read with sympify.
  rref                [M]: M.rref()[0], the reduced row echelon form.
  solve_linear_system [M, (x, y, ...)]: solve_linear_system(M, x, y, ...),
                      M the augmented matrix [A | b] of the system A*X = b.
                      SymPy returns a dict {unknown: value} with no key for a
                      free unknown, or None when the system is inconsistent;
                      the dict is printed with sstr(), which sorts the keys.

Roadmap recipes (source "app"), each a fixed sequence of SymPy calls. Source
"app" marks a recipe of several calls that the app is meant to run, rather
than one SymPy call; none of these runs in the app today. The audit names the
operations (is_invertible / det != 0, (M**n)*v, eigenvects for the steady
state) but not their exact steps. The steps below are this topic's way of
writing those planned calls, not app code. For steady_state in particular,
these are this topic's choices and not confirmed app behaviour: the structural
==, taking vecs[0], scaling by v/sum(v), and not checking for a transition
matrix. The steady_state cases on eye(2), a row-stochastic matrix and a matrix
with a negative entry depend on those choices.
  matvec_pow         [M, n, v]: (M**n)*v, the Markov state after n steps
                      from the column state vector v.
  is_invertible       [M]: M.det() != 0, with SymPy's structural "!=" on the
                      determinant (SymPy 1.14 has no Matrix.is_invertible).
                      M.det() raises for a non-square M, and so does the
                      recipe. The answer is printed "True" or "False".
  steady_state        [M]: the eigenvector for eigenvalue 1, scaled so its
                      entries sum to 1:
                        for val, mult, vecs in M.eigenvects():
                            if val == 1:   (structural ==)
                                v = vecs[0]
                                return v / sum(v)
                      where sum(v) is 0 + v[0] + v[1] + ... in SymPy
                      arithmetic. Every case's M has eigenvalue 1. The
                      recipe takes the first basis vector of that eigenspace
                      and does not check that M is a transition matrix.
"""

import sympy as sp
from sympy import (
    Matrix, Rational, Symbol, cos, eye, nsimplify, sin, solve_linear_system, sqrt, symbols, sympify,
)

from caselib import case, outcome

TOPIC = "matrix_keys"

R = Rational
a, b, c, d, k, n, t = symbols("a b c d k n t")
x, y, z = symbols("x y z")

INT2 = Matrix([[2, 1], [5, 3]])
SING2 = Matrix([[2, 4], [1, 2]])
FRAC2 = Matrix([[R(1, 2), R(1, 3)], [R(1, 4), 1]])
RAD2 = Matrix([[sqrt(2), 1], [1, sqrt(2)]])
SYM2 = Matrix([[a, b], [c, d]])
ROT_T = Matrix([[cos(t), -sin(t)], [sin(t), cos(t)]])
ROT60 = Matrix([[R(1, 2), -sqrt(3) / 2], [sqrt(3) / 2, R(1, 2)]])
INT3 = Matrix([[1, 2, 3], [0, 1, 4], [5, 6, 0]])
SING3 = Matrix([[1, 2, 3], [4, 5, 6], [7, 8, 9]])
WIDE = Matrix([[1, 2, 3], [4, 5, 6]])
# Column-stochastic transition matrices (each column sums to 1).
MARKOV = Matrix([[R(4, 5), R(3, 10)], [R(1, 5), R(7, 10)]])
ABSORB = Matrix([[1, R(1, 5)], [0, R(4, 5)]])
PERIODIC = Matrix([[0, 1], [1, 0]])
MARKOV3 = Matrix([[R(1, 2), R(1, 4), R(1, 4)], [R(1, 4), R(1, 2), R(1, 4)], [R(1, 4), R(1, 4), R(1, 2)]])
MARKOV3B = Matrix([[R(7, 10), R(1, 5), R(1, 10)], [R(1, 5), R(3, 5), R(1, 5)], [R(1, 10), R(1, 5), R(7, 10)]])
MARKOV3C = Matrix([[R(1, 2), R(1, 3), 0], [R(1, 2), R(1, 3), R(1, 2)], [0, R(1, 3), R(1, 2)]])
START = Matrix([1, 0])


def _det_inv():
    out = []
    for m, note in [
        (SING2, None),
        (FRAC2, None),
        (RAD2, None),
        (SYM2, None),
        (INT3, None),
        (SING3, None),
        (ROT_T, "det does not simplify sin(t)**2 + cos(t)**2 to 1."),
        (WIDE, "A 2x3 matrix: det of a non-square matrix."),
    ]:
        out.append(outcome("det", [m], lambda m=m: m.det(), note=note))
    for m, note in [
        (INT2, "det 1: an integer inverse."),
        (FRAC2, None),
        (ROT60, "A rotation's inverse is its transpose."),
        (Matrix([[k, 2], [3, k]]), "Symbolic entries: every entry over det = k**2 - 6."),
        (INT3, None),
        (ROT_T, "inv does not simplify the (1, 1) entry -sin(t)**2/cos(t) + 1/cos(t), which equals cos(t)."),
        (SING2, "det = 0: a singular matrix."),
        (WIDE, "A 2x3 matrix: inv of a non-square matrix."),
    ]:
        out.append(outcome("inv", [m], lambda m=m: m.inv(), note=note))
    return out


def _products():
    out = []
    for m in [WIDE, Matrix([1, 2, 3]), SYM2]:
        out.append(case("transpose", [m], m.T))
    A = Matrix([[1, 2], [3, 4]])
    B = Matrix([[0, 1], [1, 0]])
    for p, q, note in [
        (A, B, None),
        (B, A, "Matrix products do not commute: compare with A*B."),
        (WIDE, Matrix([[1, 0], [0, 1], [1, 1]]), "(2x3)*(3x2) is 2x2."),
        (Matrix([[1, 2, 3]]), Matrix([4, 5, 6]), "Row times column: a 1x1 matrix, not a scalar."),
        (SYM2, Matrix([x, y]), None),
        (ROT60, ROT60, "Two 60 degree rotations make a 120 degree rotation."),
        (INT2, INT2.inv(), "A matrix times its inverse."),
        (WIDE, WIDE, "(2x3)*(2x3): the inner sizes differ."),
    ]:
        out.append(outcome("matmul", [p, q], lambda p=p, q=q: p * q, note=note))
    for m, p, note in [
        (MARKOV, 10, "Exact fractions with powers of 2 in the denominators."),
        (MARKOV, 0, "M**0 is the identity."),
        (Matrix([[1, 2], [3, 4]]), -2, "A negative power is a power of the inverse."),
        (Matrix([[1, 1], [1, 0]]), 10, "Fibonacci numbers."),
        (Matrix([[1, 1], [0, 1]]), n, "Symbolic power of a shear."),
        (MARKOV, n, "Symbolic power: a closed form from the eigenvalues 1 and 1/2."),
        (SING2, -1, "A negative power needs the inverse, and det = 0."),
        (WIDE, 2, "A 2x3 matrix: powers of a non-square matrix."),
    ]:
        out.append(outcome("matpow", [m, p], lambda m=m, p=p: m**p, note=note))
    return out


def _markov():
    out = []
    for m, p, v, note in [
        (MARKOV, 1, START, "One step from state 1."),
        (MARKOV, 3, START, None),
        (MARKOV, 10, START, None),
        (MARKOV, 4, Matrix([300, 200]), "The population starts at the steady state."),
        (MARKOV3B, 3, Matrix([100, 0, 0]), None),
        (MARKOV, n, START, "Symbolic number of steps: a closed form in 2**n."),
    ]:
        out.append(case("matvec_pow", [m, p, v], (m**p) * v, note=note, source="app"))
    src = "Matrix([[0.8, 0.3], [0.2, 0.7]])"
    out.append(case(
        "matvec_pow", [src, 3, "Matrix([[1], [0]])"], (sympify(src) ** 3) * Matrix([1, 0]),
        note="Decimals typed in SymPy syntax are Floats, so the state is printed with 15-digit Floats.",
        source="app",
    ))
    pos_a = Symbol("a", positive=True)
    for m, note in [
        (INT2, None),
        (SING2, None),
        (SING3, None),
        (Matrix([[sqrt(2), 2], [1, sqrt(2)]]), "det = sqrt(2)*sqrt(2) - 2, which SymPy evaluates to 0."),
        (Matrix([[pos_a, 1], [0, pos_a]]), "det = a**2 with a positive; != compares structurally."),
        (WIDE, "A 2x3 matrix: M.det() raises inside the recipe."),
    ]:
        out.append(outcome("is_invertible", [m], lambda m=m: m.det() != 0, note=note, source="app"))
    return out


def _eig_text(m):
    """SymPy's eigenvects() on one line (sstr would print each Matrix over several lines)."""
    parts = []
    for val, mult, vecs in m.eigenvects():
        parts.append(f"({sp.sstr(val)}, {mult}, [{', '.join(str(v) for v in vecs)}])")
    return "[" + ", ".join(parts) + "]"


def app_steady_state(m):
    """The roadmap steady_state recipe as this topic writes it (see the module docstring)."""
    for val, _mult, vecs in m.eigenvects():
        if val == 1:
            v = vecs[0]
            return v / sum(v)
    raise AssertionError("steady_state cases need eigenvalue 1")


def _eigen():
    out = []
    for m, note in [
        (MARKOV, "Transition matrix: eigenvalues 1 and 1/2."),
        (ABSORB, "Absorbing state 1."),
        (Matrix([[2, 1], [1, 2]]), "Symmetric: orthogonal eigenvectors."),
        (Matrix([[1, 1], [1, 0]]), "Symmetric with irrational eigenvalues (golden ratio)."),
        (Matrix([[2, 1], [0, 2]]), "Defective: eigenvalue 2 has multiplicity 2 but one eigenvector."),
        (MARKOV3, "Eigenvalue 1/4 has a two-dimensional eigenspace."),
    ]:
        out.append(case("eigenvects", [m], _eig_text(m), note=note))
    for m, note in [
        (MARKOV, None),
        (Matrix([[R(9, 10), R(1, 2)], [R(1, 10), R(1, 2)]]), None),
        (ABSORB, "Absorbing state 1."),
        (PERIODIC, "M**n*v oscillates and never converges, but eigenvalue 1 has a one-dimensional eigenspace."),
        (MARKOV3C, None),
        (eye(2), "Every vector is fixed: the eigenspace for 1 is two-dimensional, and the recipe takes its "
                 "first basis vector."),
        (Matrix([[R(1, 2), R(1, 2)], [R(1, 4), R(3, 4)]]),
         "Row-stochastic (the rows sum to 1, not the columns): the recipe uses the column convention regardless."),
        (Matrix([[R(6, 5), R(1, 2)], [R(-1, 5), R(1, 2)]]),
         "Each column sums to 1 but the entry -1/5 is not a probability; the recipe does not check."),
    ]:
        out.append(case("steady_state", [m], app_steady_state(m), note=note, source="app"))
    return out


def _exact_display():
    out = []
    for src, note in [
        ("Matrix([[0.8, 0.3], [0.2, 0.7]])", None),
        ("Matrix([[0.3333333333333333, 0.6666666666666666], [1.5, -2.0]])", "Full-precision repeating decimals become 1/3 and 2/3."),
        ("Matrix([[0.333, 0.1], [0.05, 0.001]])", "A short decimal keeps its typed digits: 333/1000, not 1/3."),
        ("Matrix([[0.8660254037844386, -0.5], [0.5, 0.8660254037844386]])", "nsimplify recognises sqrt(3)/2."),
        ("Matrix([[3.14159265358979, 2.5e-5]])", "pi is not guessed: without constants nsimplify keeps the digits."),
        ("Matrix([[0.9, 0.5], [0.1, 0.5]])**3", "The Float power is computed first; nsimplify absorbs its rounding error."),
    ]:
        out.append(case("nsimplify_entries", [src], sympify(src).applyfunc(nsimplify), note=note))
    for m, note in [
        (Matrix([[2, 1, -1, 8], [-3, -1, 2, -11], [-2, 1, 2, -3]]), "Augmented 3x3 system with one solution."),
        (Matrix([[1, 2, 1, 4], [2, 4, 0, 6], [1, 2, 2, 5]]), "Dependent system: a free variable and a zero row."),
        (Matrix([[1, 1, 2], [1, 1, 3]]), "Inconsistent system: the row [0, 0, 1]."),
        (Matrix([[R(1, 2), R(1, 3), 1], [R(1, 4), 1, 2]]), None),
        (Matrix([[1, sqrt(2), 3], [sqrt(2), 1, 0]]), None),
        (Matrix([[0, 0, 1], [0, 2, 4]]), "A zero first column: pivots in columns 2 and 3."),
    ]:
        out.append(case("rref", [m], m.rref()[0], note=note))
    for A, rhs, syms, note in [
        (Matrix([[1, 1], [2, -1]]), Matrix([5, 1]), (x, y), None),
        (Matrix([[2, 3], [4, 6]]), Matrix([5, 10]), (x, y),
         "Dependent: infinitely many solutions; the free unknown y gets no key in the dict."),
        (Matrix([[2, 3], [4, 6]]), Matrix([5, 11]), (x, y), "Inconsistent: SymPy returns None."),
        (Matrix([[2, 1, -1], [-3, -1, 2], [-2, 1, 2]]), Matrix([8, -11, -3]), (x, y, z), None),
        (Matrix([[1, 2, 3], [2, 4, 6], [1, 0, 1]]), Matrix([6, 12, 2]), (x, y, z),
         "One free unknown, z: no key for z in the dict."),
        (Matrix([[3, -2], [5, 4]]), Matrix([R(1, 2), 7]), (x, y), None),
        (Matrix([[1, 1], [1, -1], [2, 1]]), Matrix([3, 1, 5]), (x, y), "Three equations, two unknowns, consistent."),
    ]:
        M = A.row_join(rhs)
        out.append(outcome(
            "solve_linear_system", [M, sp.Tuple(*syms)], lambda M=M, syms=syms: solve_linear_system(M, *syms),
            note=note,
        ))
    return out


def generate():
    return _det_inv() + _products() + _markov() + _eigen() + _exact_display()
