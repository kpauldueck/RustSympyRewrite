"""Matrix keys (Unit 4): what the app computes at compile time from a matrix key.

Args are matrices in SymPy syntax, Matrix([[row], [row], ...]); a column
vector is Matrix([[a], [b]]). Matrices with Float entries are passed as typed
strings, since str() of a Float matrix pads every entry to 15 digits.

Ops:
  det                 [M].
  inv                 [M]. A singular matrix is refused as
                      "invalid:singular_matrix" (spec; SymPy raises
                      NonInvertibleMatrixError). Non-square matrices (det,
                      inv, matpow) are "invalid:non_square_matrix".
  transpose           [M]: M.T.
  matmul              [A, B]: A*B. Incompatible shapes are refused as
                      "invalid:shape_mismatch" (spec; SymPy raises ShapeError).
  matpow              [M, n]: M**n, n an integer (negative means a power of
                      the inverse) or a symbol.
  matvec_pow          [M, n, v]: M**n*v, the Markov state after n steps with
                      a column-stochastic M (each column sums to 1) and a
                      column state vector v.
  is_invertible       [M]: True when M is square and det(M) is provably
                      nonzero (M.det().is_zero is False). SymPy 1.14 has no
                      Matrix.is_invertible; the app refuses singular seeds
                      with det != 0. A non-square matrix is False (spec).
  eigenvects          [M]: M.eigenvects(), printed on one line as
                      [(eigenvalue, algebraic multiplicity, [basis vectors]),
                      ...]. compare "set" ignores the order of the triples;
                      each basis vector is SymPy's: a nullspace vector of
                      M - lambda*I from its rref with the free variable set to
                      1, so the scaling is fixed.
  steady_state        [M]: the eigenvector of the column-stochastic M for
                      eigenvalue 1, scaled so its entries sum to 1 (computed
                      as (M - I).nullspace()[0] divided by its entry sum).
                      Spec errors: "invalid:steady_state_not_unique" when
                      that eigenspace is not one-dimensional, and
                      "invalid:not_stochastic" when a column does not sum
                      to 1 or an entry is negative.
  nsimplify_entries   [M]: M.applyfunc(nsimplify), Float entries to exact
                      ones for display.
  rref                [M]: the reduced row echelon form M.rref()[0], as in
                      the matrices topic.
  solve_linear_system [A, b, (x, y, ...)]: linsolve((A, b), x, y, ...), the
                      system A*X = b. Free unknowns stay as themselves.
"""

import sympy as sp
from sympy import Matrix, Rational, Symbol, cos, eye, linsolve, nsimplify, sin, sqrt, symbols, sympify

from caselib import case, spec_case

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

SINGULAR = "invalid:singular_matrix"
NON_SQUARE = "invalid:non_square_matrix"


def _det_inv():
    out = []
    for m, cmp, note in [
        (SING2, "exact", None),
        (FRAC2, "exact", None),
        (RAD2, "exact", None),
        (SYM2, "exact", None),
        (INT3, "exact", None),
        (SING3, "exact", None),
        (ROT_T, "equivalent", "SymPy leaves sin(t)**2 + cos(t)**2 unsimplified; it equals 1."),
    ]:
        out.append(case("det", [m], m.det(), compare=cmp, note=note))
    out.append(spec_case("det", [WIDE], NON_SQUARE, "Only a square matrix has a determinant; refuse by name.",
                         compare="error", sympy_gives=lambda: WIDE.det()))
    for m, cmp, note in [
        (INT2, "exact", "det 1: an integer inverse."),
        (FRAC2, "exact", None),
        (ROT60, "exact", "A rotation's inverse is its transpose."),
        (Matrix([[k, 2], [3, k]]), "exact", "Symbolic entries: every entry over det = k**2 - 6."),
        (INT3, "exact", None),
        (ROT_T, "equivalent", "SymPy does not simplify the (1, 1) entry 1/cos(t) - sin(t)**2/cos(t); it equals cos(t)."),
    ]:
        out.append(case("inv", [m], m.inv(), compare=cmp, note=note))
    out.append(spec_case("inv", [SING2], SINGULAR,
                         "det = 0, so there is no inverse; refuse by name (the app uses this to reject singular seeds).",
                         compare="error", sympy_gives=lambda: SING2.inv()))
    out.append(spec_case("inv", [WIDE], NON_SQUARE, "A non-square matrix has no inverse; refuse by name.",
                         compare="error", sympy_gives=lambda: WIDE.inv()))
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
        (INT2, INT2.inv(), "A matrix times its inverse is the identity."),
    ]:
        out.append(case("matmul", [p, q], p * q, note=note))
    out.append(spec_case("matmul", [WIDE, WIDE], "invalid:shape_mismatch",
                         "(2x3)*(2x3): the inner sizes differ, so the product is undefined; refuse by name.",
                         compare="error", sympy_gives=lambda: WIDE * WIDE))
    for m, p, cmp, note in [
        (MARKOV, 10, "exact", "Exact fractions with powers of 2 in the denominators."),
        (MARKOV, 0, "exact", "M**0 is the identity."),
        (Matrix([[1, 2], [3, 4]]), -2, "exact", "A negative power is a power of the inverse."),
        (Matrix([[1, 1], [1, 0]]), 10, "exact", "Fibonacci numbers."),
        (Matrix([[1, 1], [0, 1]]), n, "equivalent", "Symbolic power of a shear."),
        (MARKOV, n, "equivalent", "Closed form from the eigenvalues 1 and 1/2."),
    ]:
        out.append(case("matpow", [m, p], m**p, compare=cmp, note=note))
    out.append(spec_case("matpow", [SING2, -1], SINGULAR, "A negative power needs the inverse, and det = 0.",
                         compare="error", sympy_gives=lambda: SING2**-1))
    out.append(spec_case("matpow", [WIDE, 2], NON_SQUARE, "Only a square matrix has powers; refuse by name.",
                         compare="error", sympy_gives=lambda: WIDE**2))
    return out


def _markov():
    out = []
    for m, p, v, cmp, note in [
        (MARKOV, 1, START, "exact", "One step from state 1."),
        (MARKOV, 3, START, "exact", None),
        (MARKOV, 10, START, "exact", None),
        (MARKOV, 4, Matrix([300, 200]), "exact", "The population is already at the steady state, so it stays."),
        (MARKOV3B, 3, Matrix([100, 0, 0]), "exact", None),
        (MARKOV, n, START, "equivalent", "Closed form: approaches the steady state (3/5, 2/5)."),
    ]:
        out.append(case("matvec_pow", [m, p, v], m**p * v, compare=cmp, note=note))
    out.append(case(
        "matvec_pow", ["Matrix([[0.8, 0.3], [0.2, 0.7]])", 3, "Matrix([[1], [0]])"],
        sympify("Matrix([[0.8, 0.3], [0.2, 0.7]])") ** 3 * Matrix([1, 0]), compare="equivalent",
        note="A key typed with decimals stays Float in SymPy syntax (0.65, 0.35); nsimplify_entries gives the exact display.",
    ))
    pos_a = Symbol("a", positive=True)
    for m, note in [
        (INT2, None),
        (SING2, None),
        (SING3, None),
        (Matrix([[sqrt(2), 2], [1, sqrt(2)]]), "det = sqrt(2)*sqrt(2) - 2 = 0 exactly: singular."),
        (Matrix([[pos_a, 1], [0, pos_a]]), "det = a**2 is nonzero because a is positive."),
    ]:
        dz = m.det().is_zero
        assert dz is not None
        out.append(case("is_invertible", [m], dz is False, note=note))
    out.append(spec_case("is_invertible", [WIDE], False, "A non-square matrix is never invertible.",
                         sympy_gives=lambda: WIDE.det()))
    return out


def _eig_text(m):
    """SymPy's eigenvects() on one line (sstr would print each Matrix over several lines)."""
    parts = []
    for val, mult, vecs in m.eigenvects():
        parts.append(f"({sp.sstr(val)}, {mult}, [{', '.join(str(v) for v in vecs)}])")
    return "[" + ", ".join(parts) + "]"


def _steady(m):
    v = (m - eye(m.rows)).nullspace()[0]
    return v / sum(v)


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
        out.append(case("eigenvects", [m], _eig_text(m), compare="set", note=note))
    how = "Computed as (M - I).nullspace()[0] divided by its entry sum."
    for m, note in [
        (MARKOV, None),
        (Matrix([[R(9, 10), R(1, 2)], [R(1, 10), R(1, 2)]]), None),
        (ABSORB, "Everything ends in the absorbing state."),
        (PERIODIC, "Unique steady state although M**n*v oscillates and never converges."),
        (MARKOV3C, None),
    ]:
        out.append(case("steady_state", [m], _steady(m), note=f"{how} {note}" if note else how))
    out.append(spec_case(
        "steady_state", [eye(2)], "invalid:steady_state_not_unique",
        "Every vector is fixed by the identity (the eigenspace for 1 is two-dimensional), so there is no single steady state.",
        compare="error",
        sympy_gives=lambda: "[" + ", ".join(str(v) for v in (eye(2) - eye(2)).nullspace()) + "]",
    ))
    row = Matrix([[R(1, 2), R(1, 2)], [R(1, 4), R(3, 4)]])
    out.append(spec_case(
        "steady_state", [row], "invalid:not_stochastic",
        "The rows sum to 1, not the columns (a row-stochastic key used with M**n*v); the nullspace recipe silently "
        "gives (1/2, 1/2) where the row convention's answer is (1/3, 2/3), so refuse by name.",
        compare="error", sympy_gives=lambda: _steady(row),
    ))
    neg = Matrix([[R(6, 5), R(1, 2)], [R(-1, 5), R(1, 2)]])
    out.append(spec_case(
        "steady_state", [neg], "invalid:not_stochastic",
        "Each column sums to 1 but the entry -1/5 is not a probability; the nullspace recipe silently gives "
        "(5/3, -2/3), which is not a distribution, so refuse by name.",
        compare="error", sympy_gives=lambda: _steady(neg),
    ))
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
        (Matrix([[2, 3], [4, 6]]), Matrix([5, 10]), (x, y), "Dependent: infinitely many solutions in terms of y."),
        (Matrix([[2, 3], [4, 6]]), Matrix([5, 11]), (x, y), "Inconsistent: no solution."),
        (Matrix([[2, 1, -1], [-3, -1, 2], [-2, 1, 2]]), Matrix([8, -11, -3]), (x, y, z), None),
        (Matrix([[1, 2, 3], [2, 4, 6], [1, 0, 1]]), Matrix([6, 12, 2]), (x, y, z), "One free unknown, z."),
        (Matrix([[3, -2], [5, 4]]), Matrix([R(1, 2), 7]), (x, y), None),
        (Matrix([[1, 1], [1, -1], [2, 1]]), Matrix([3, 1, 5]), (x, y), "Three equations, two unknowns, consistent."),
    ]:
        out.append(case("solve_linear_system", [A, rhs, sp.Tuple(*syms)], linsolve((A, rhs), *syms),
                        compare="set", note=note))
    return out


def generate():
    return _det_inv() + _products() + _markov() + _eigen() + _exact_display()
