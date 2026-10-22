"""Geometry: what sympy.geometry returns for figures, triangle centres and transformations.

The homework app's roadmap builds geometry figures and their exact keys with
sympy.geometry (app_sympy_audit.md, "Geometry"): Point, Segment, Line, Ray,
Circle, Triangle, Polygon; intersections, angles, triangle centres;
reflect/rotate/translate/scale; Polygon equality; "in terms of pi" and
special-right-triangle values; coordinate proofs with symbolic vertices. This
topic records what the pinned SymPy returns for those calls, including where
its answer is surprising: signed polygon areas, a collinear Triangle that
becomes a Segment2D, a reflected Circle with a negative radius, and an
intersection of identical circles that is not a list.

Every arg is a constructor string in SymPy syntax, read with sympify, e.g.
"Triangle(Point(0, 0), Point(4, 0), Point(0, 3))". Numbers inside are exact
("1/2" is a Rational). "Line(Eq(y, 2*x + 1))" builds a line from its
equation in x and y; "Line(Point(0, 2), slope=3)" from a point and a slope.
The letters a, b, c are read as Symbol(..., positive=True), and the case's
"assumptions" field says so. Every other name is read the way sympify reads
it: x, y and other ordinary letters become plain Symbols, while E, I, N, O,
Q, S (and names such as pi or beta) are SymPy's own objects, e.g.
"Point(E, 0)" is Point2D(E, 0) with E = exp(1) and "Point(I, 0)" raises.

Each answer is SymPy's return value as sympy.sstr() prints it (caselib):
Point as Point2D, Line as Line2D, Segment as Segment2D; a list in the order
SymPy returns it; a dict with its keys sorted by default_sort_key (sstr sorts
them; plain str() would keep insertion order).

Ops (P, Q are the sympified args):
  distance       [P, Q]: P.distance(Q). Q may be a Line or Segment.
  midpoint       [P, Q]: P.midpoint(Q); or [S]: the property S.midpoint of a
                 Segment.
  slope          [L]: the property L.slope of a Line or Segment.
  slope_product  [L1, L2]: L1.slope * L2.slope.
  intersection   [E1, E2]: E1.intersection(E2).
  is_parallel, is_perpendicular, angle_between, smallest_angle_between
                 [L1, L2]: L1.<op>(L2).
  area, perimeter, circumference
                 [F]: the property F.<op>.
  side_lengths   [T]: [s.length for s in T.sides].
  angles, circumcenter, incenter, orthocenter, centroid, circumradius,
  inradius, medians, altitudes
                 [T]: the property T.<op>.
  bisectors, is_right
                 [T]: T.<op>().
  is_similar     [T1, T2]: T1.is_similar(T2).
  tangent_lines  [C, P]: C.tangent_lines(P), a list of Line2D through P.
  reflect        [E, L]: E.reflect(L).
  rotate         [E, angle] or [E, angle, P]: E.rotate(angle) or
                 E.rotate(angle, P); radians.
  translate      [E, dx, dy]: E.translate(dx, dy).
  scale          [E, sx, sy] or [E, sx, sy, P]: E.scale(sx, sy) or
                 E.scale(sx, sy, P).
  equals         [E1, E2]: E1.equals(E2) (== ; for a Polygon, the same vertex
                 cycle from any start, in either direction).
"""

from sympy import Symbol, sympify

from caselib import outcome

TOPIC = "geometry"

LOCALS = {name: Symbol(name, positive=True) for name in "abc"}

# Recurring figures.
T345 = "Triangle(Point(0, 0), Point(4, 0), Point(0, 3))"
T_ACUTE = "Triangle(Point(0, 0), Point(6, 0), Point(2, 4))"
T_OBTUSE = "Triangle(Point(0, 0), Point(4, 0), Point(-2, 3))"
T_ISO = "Triangle(Point(0, 0), Point(6, 0), Point(3, 4))"
T_EQUI = "Triangle(Point(0, 0), Point(2, 0), Point(1, sqrt(3)))"
T_MOVE = "Triangle(Point(1, 1), Point(4, 1), Point(1, 3))"
SQUARE = "Polygon(Point(0, 0), Point(2, 0), Point(2, 2), Point(0, 2))"
Y_EQ_X = "Line(Point(0, 0), Point(1, 1))"
X_AXIS = "Line(Point(0, 0), Point(1, 0))"
O = "Point(0, 0)"


def G(src):
    """Read a constructor string: sympify, with a, b, c positive."""
    return sympify(src, locals=LOCALS)


def _assumptions(objs):
    """{symbol: assumptions} for the symbols in objs that carry any."""
    syms = set().union(*(o.free_symbols for o in objs))
    return {
        str(s): {k: bool(v) for k, v in sorted(s._assumptions_orig.items())}
        for s in sorted(syms, key=str)
        if s._assumptions_orig
    }


def g(op, args, fn, note=None):
    """A geometry case: fn receives the sympified args. The args stay the
    strings above (as typed), so their symbols' assumptions are added here."""
    objs = [G(s) for s in args]
    out = outcome(op, args, lambda: fn(*objs), note=note)
    assumptions = _assumptions(objs)
    if not assumptions:
        return out
    items = list(out.items())
    i = [k for k, _ in items].index("expected")
    return dict(items[:i] + [("assumptions", assumptions)] + items[i:])


def _measurements():
    out = []
    dist = lambda p, q: p.distance(q)
    for p, q, note in [
        (O, "Point(3, 4)", None),
        (O, "Point(2, 2)", "sqrt(8) evaluates to 2*sqrt(2) (Pow construction takes out square factors)."),
        ("Point(1/2, 1)", "Point(2, 3)", "sqrt(25/4) = 5/2: rational coordinates stay exact."),
        (O, "Line(Point(2, 1), Point(-2, 4))", "Distance from a point to the line 3*x + 4*y = 10."),
        ("Point(6, 3)", "Segment(Point(0, 0), Point(4, 0))",
         "The nearest point of the segment is its endpoint (4, 0), not the foot of the perpendicular."),
    ]:
        out.append(g("distance", [p, q], dist, note=note))
    out.append(g("midpoint", [O, "Point(3, 4)"], lambda p, q: p.midpoint(q)))
    out.append(g("midpoint", ["Point(1/2, 1/3)", "Point(3/2, -1/3)"], lambda p, q: p.midpoint(q)))
    out.append(g("midpoint", ["Segment(Point(1, 2), Point(5, -4))"], lambda s: s.midpoint))
    for line, note in [
        ("Line(Point(1, 2), Point(3, 8))", None),
        ("Segment(Point(1, 2), Point(4, 1))", None),
        ("Line(Point(1, 2), Point(5, 2))", None),
        ("Line(Point(1, 2), Point(1, 8))", "Vertical line: SymPy's slope is oo."),
        ("Line(Eq(y, -2*x/3 + 4))", "A line given by its equation."),
    ]:
        out.append(g("slope", [line], lambda l: l.slope, note=note))
    return out


def _lines():
    out = []
    for l1, l2 in [
        ("Line(Point(0, 0), Point(1, 2))", "Line(Point(3, 0), Point(4, 2))"),
        ("Line(Point(0, 0), Point(1, 2))", "Line(Point(3, 0), Point(4, 3))"),
        ("Line(Point(1, 0), Point(1, 1))", "Line(Point(3, 0), Point(3, 5))"),
    ]:
        out.append(g("is_parallel", [l1, l2], lambda p, q: p.is_parallel(q)))
    for l1, l2, note in [
        ("Line(Point(0, 0), Point(1, 2))", "Line(Point(0, 0), Point(2, -1))", None),
        ("Line(Point(0, 1), Point(5, 1))", "Line(Point(3, 0), Point(3, 5))", None),
        ("Line(Point(0, 2), slope=2)", "Line(Point(1, 1), slope=1/2)",
         "Near miss: reciprocal slope without the sign change."),
        ("Line(Point(0, 0), Point(1, 2))", "Line(Point(0, 0), Point(1, -2))",
         "Near miss: opposite slope without the reciprocal."),
    ]:
        out.append(g("is_perpendicular", [l1, l2], lambda p, q: p.is_perpendicular(q), note=note))
    for l2, note in [
        (Y_EQ_X, None),
        ("Line(Point(0, 0), Point(-1, 1))",
         "Angle between the direction vectors, so it can be obtuse; smallest_angle_between gives the acute one."),
        ("Line(Point(0, 0), Point(2, 1))", "Not a special angle: an exact acos."),
    ]:
        out.append(g("angle_between", [X_AXIS, l2], lambda p, q: p.angle_between(q), note=note))
    out.append(g("angle_between", ["Line(Point(0, 0), Point(1, 2))", "Line(Point(0, 0), Point(3, 1))"],
                 lambda p, q: p.angle_between(q)))
    out.append(g("smallest_angle_between", [X_AXIS, "Line(Point(0, 0), Point(-1, 1))"],
                 lambda p, q: p.smallest_angle_between(q)))
    return out


def _intersections():
    out = []
    inter = lambda p, q: p.intersection(q)
    for e1, e2, note in [
        ("Line(Point(0, 1), Point(1, 3))", "Line(Point(0, 3), Point(3, 0))", "y = 2*x + 1 meets y = 3 - x at a rational point."),
        ("Line(Point(0, 0), Point(1, 1))", "Line(Point(0, 2), Point(1, 3))", "Parallel lines."),
        ("Line(Point(0, 0), Point(1, 1))", "Line(Point(2, 2), Point(3, 3))",
         "Coincident lines: SymPy returns the argument line itself as the one element."),
        (Y_EQ_X, "Circle(Point(0, 0), 2)", "Secant through the centre: irrational points, in the order SymPy finds them."),
        ("Line(Point(0, 1), Point(1, 2))", "Circle(Point(0, 0), 5)", "y = x + 1 meets x**2 + y**2 = 25 at lattice points."),
        ("Line(Point(0, 5), Point(1, 5))", "Circle(Point(0, 0), 5)", "Tangent line: one point."),
        ("Line(Point(0, 6), Point(1, 6))", "Circle(Point(0, 0), 5)", "The line misses the circle."),
        ("Circle(Point(0, 0), 5)", "Circle(Point(6, 0), 5)", None),
        ("Circle(Point(0, 0), 2)", "Circle(Point(5, 0), 3)", "Externally tangent circles: one point."),
        ("Circle(Point(0, 0), 1)", "Circle(Point(0, 0), 2)", "Concentric circles."),
        ("Circle(Point(0, 0), 2)", "Circle(Point(0, 0), 2)",
         "Identical circles: SymPy returns the Circle itself, not a list."),
        ("Segment(Point(0, 0), Point(4, 4))", "Segment(Point(0, 4), Point(4, 0))", None),
        ("Segment(Point(0, 0), Point(1, 1))", "Segment(Point(0, 4), Point(4, 0))",
         "The lines cross at (2, 2) but the segments do not reach it."),
        ("Segment(Point(0, 0), Point(2, 2))", "Segment(Point(2, 2), Point(4, 0))",
         "Segments are closed: a shared endpoint."),
        ("Segment(Point(0, 0), Point(3, 0))", "Segment(Point(1, 0), Point(5, 0))", "Collinear overlap: the common segment."),
        ("Ray(Point(0, 0), Point(1, 1))", "Line(Point(-3, 0), Point(0, -3))",
         "The line meets the ray's line behind the ray's start."),
    ]:
        out.append(g("intersection", [e1, e2], inter, note=note))
    return out


def _areas():
    out = []
    for fig, note in [
        (T345, None),
        ("Triangle(Point(0, 0), Point(0, 3), Point(4, 0))", "Clockwise vertices: SymPy's Polygon.area is signed."),
        ("Triangle(Point(0, 0), Point(1, 1), Point(2, 2))",
         "Collinear vertices: SymPy's Triangle evaluates to a Segment2D, which has no area attribute."),
        ("Polygon(Point(0, 0), Point(6, 0), Point(4, 3), Point(1, 3))", "Trapezoid."),
        ("Polygon(Point(0, 0), Point(2, 2), Point(2, 0), Point(0, 2))",
         "Vertices out of order make a self-intersecting bow tie whose signed areas cancel."),
        ("RegularPolygon(Point(0, 0), 2, 6)", "Regular hexagon with circumradius 2."),
        ("Circle(Point(0, 0), 3)", "In terms of pi."),
        ("Circle(Eq(x**2 + y**2 - 4*x + 6*y - 3, 0))", "Circle from its general equation: centre (2, -3), radius 4."),
        ("Circle(Point(0, 0), -2)",
         "SymPy keeps the radius -2; Ellipse.area is pi*hradius*vradius and a Circle's vradius is Abs(radius)."),
    ]:
        out.append(g("area", [fig], lambda f: f.area, note=note))
    out.append(g("perimeter", [T_ACUTE], lambda f: f.perimeter))
    out.append(g("perimeter", ["Polygon(Point(0, 0), Point(4, 0), Point(5, 3), Point(1, 3))"],
                 lambda f: f.perimeter, note="Parallelogram."))
    out.append(g("circumference", ["Circle(Point(2, -1), 5/2)"], lambda f: f.circumference))
    return out


def _triangles():
    out = []
    sides = lambda t: [s.length for s in t.sides]
    for tri, note in [
        ("Triangle(Point(0, 0), Point(7, 0), Point(0, 7))", "45-45-90 with legs 7."),
        ("Triangle(Point(0, 0), Point(10, 0), Point(5, 5))", "45-45-90 with hypotenuse 10."),
        ("Triangle(Point(0, 0), Point(12, 0), Point(3, 3*sqrt(3)))",
         "30-60-90 with hypotenuse 12: sqrt(108) evaluates to 6*sqrt(3)."),
    ]:
        out.append(g("side_lengths", [tri], sides, note=note))
    for tri, note in [
        (T345, "Only the right angle is special; the others are exact acos values."),
        ("Triangle(Point(0, 0), Point(12, 0), Point(3, 3*sqrt(3)))", "30-60-90."),
        (T_EQUI, None),
        (T_OBTUSE, "Obtuse angle at the origin: acos of a negative number."),
    ]:
        out.append(g("angles", [tri], lambda t: t.angles, note=note))
    for op, tri, note in [
        ("circumcenter", T345, "Right triangle: the midpoint of the hypotenuse."),
        ("circumcenter", T_EQUI, "Equilateral: every centre is (1, sqrt(3)/3)."),
        ("incenter", T345, None),
        ("incenter", T_ISO, None),
        ("incenter", T_ACUTE, "SymPy leaves radicals in the denominators (about (2.4076, 1.4880))."),
        ("orthocenter", T345, "Right triangle: the right-angle vertex."),
        ("orthocenter", T_ACUTE, None),
        ("orthocenter", T_OBTUSE, "Obtuse triangle: outside the triangle."),
        ("centroid", T_ACUTE, None),
        ("circumradius", T_ISO, None),
        ("inradius", T345, None),
    ]:
        out.append(g(op, [tri], lambda t, op=op: getattr(t, op), note=note))
    out.append(g("medians", [T_ISO], lambda t: t.medians))
    out.append(g("altitudes", [T345], lambda t: t.altitudes,
                 note="In a right triangle two altitudes are the legs."))
    out.append(g("bisectors", [T345], lambda t: t.bisectors(),
                 note="Angle bisectors end on the opposite side."))
    for t2, note in [
        ("Triangle(Point(0, 0), Point(8, 0), Point(0, 6))", "Scaled by 2."),
        ("Triangle(Point(1, 1), Point(1, 9), Point(-5, 1))", "Rotated, scaled and listed in another vertex order."),
        ("Triangle(Point(0, 0), Point(4, 0), Point(0, 4))", None),
    ]:
        out.append(g("is_similar", [T345, t2], lambda p, q: p.is_similar(q), note=note))
    out.append(g("is_right", ["Triangle(Point(1, 1), Point(4, 5), Point(5, -2))"], lambda t: t.is_right(),
                 note="Right angle at (1, 1), not axis-aligned."))
    out.append(g("is_right", [T_ISO], lambda t: t.is_right()))
    return out


def _tangents():
    out = []
    for circ, p, note in [
        ("Circle(Point(0, 0), 5)", "Point(5, 5)", "The tangents x = 5 and y = 5; each Line2D's second point is SymPy's choice."),
        ("Circle(Point(0, 0), 5)", "Point(0, 10)", "Slopes +-sqrt(3)."),
        ("Circle(Point(0, 0), 5)", "Point(3, 4)", "A point on the circle: the one tangent there."),
        ("Circle(Point(0, 0), 5)", "Point(1, 1)", "A point inside the circle."),
    ]:
        out.append(g("tangent_lines", [circ, p], lambda c, q: c.tangent_lines(q), note=note))
    return out


def _transformations():
    out = []
    refl = lambda e, l: e.reflect(l)
    for ent, line, note in [
        ("Point(2, 5)", Y_EQ_X, "Across y = x: swap the coordinates."),
        ("Point(2, 5)", "Line(Point(0, 0), Point(1, -1))", "Across y = -x."),
        ("Point(2, 5)", "Line(Point(0, 2), Point(1, 2))", "Across y = 2."),
        ("Point(2, 5)", "Line(Point(3, 0), Point(3, 1))", "Across x = 3."),
        (T_MOVE, Y_EQ_X, None),
        ("Triangle(Point(0, 0), Point(2, 0), Point(0, 2))", "Line(Eq(y, x + 3))", "Across y = x + 3, given by its equation."),
        ("Circle(Point(2, 3), 1)", X_AXIS, "Circle.reflect negates the radius."),
    ]:
        out.append(g("reflect", [ent, line], refl, note=note))
    rot = lambda e, ang, *pt: e.rotate(ang, *pt)
    for args, note in [
        (["Point(3, 1)", "pi/2"], "90 degrees counterclockwise: (x, y) -> (-y, x)."),
        (["Point(3, 1)", "-pi/2"], "Clockwise."),
        (["Point(3, 1)", "pi/2", "Point(1, 1)"], "About the point (1, 1)."),
        (["Point(2, 0)", "pi/3"], "Exact radical image."),
        (["Point(-2, 5)", "pi"], "Half turn about the origin: (x, y) -> (-x, -y)."),
        ([T_MOVE, "pi", "Point(1, 1)"], "Half turn about a vertex."),
        ([SQUARE, "pi/2", "Point(1, 1)"], "A square turned about its centre lands on itself with shifted vertex labels."),
    ]:
        out.append(g("rotate", args, rot, note=note))
    tr = lambda e, dx, dy: e.translate(dx, dy)
    out.append(g("translate", ["Point(3, 1)", "2", "-5"], tr))
    out.append(g("translate", [T_MOVE, "-1", "2"], tr))
    sc = lambda e, sx, sy, *pt: e.scale(sx, sy, *pt)
    for args, note in [
        (["Point(3, 1)", "2", "2"], "Dilation by 2 about the origin."),
        (["Point(3, 1)", "2", "2", "Point(1, 1)"], "Dilation by 2 about (1, 1)."),
        ([T_MOVE, "1/2", "1/2", "Point(1, 1)"], None),
        ([T_MOVE, "-1", "-1"], "Dilation by -1: the half turn about the origin."),
        (["Circle(Point(1, 1), 1)", "3", "3"], "Dilating a circle by 3 about the origin: centre and radius both scale."),
        (["Circle(Point(0, 0), 1)", "2", "1"], "Unequal factors stretch a circle into an Ellipse."),
    ]:
        out.append(g("scale", args, sc, note=note))
    eq = lambda p, q: p.equals(q)
    for p2, note in [
        ("Polygon(Point(2, 2), Point(0, 2), Point(0, 0), Point(2, 0))", "Same square, different starting vertex."),
        ("Polygon(Point(0, 0), Point(0, 2), Point(2, 2), Point(2, 0))", "Same square traversed clockwise."),
        ("Polygon(Point(0, 0), Point(2, 2), Point(2, 0), Point(0, 2))",
         "Same vertex set in another cyclic order: a self-intersecting polygon."),
        ("Polygon(Point(0, 0), Point(1, 0), Point(2, 0), Point(2, 2), Point(0, 2))",
         "SymPy drops the collinear vertex (1, 0)."),
    ]:
        out.append(g("equals", [SQUARE, p2], eq, note=note))
    out.append(g("equals", [T345, "Polygon(Point(0, 3), Point(4, 0), Point(0, 0))"], eq,
                 note="A three-vertex Polygon is a Triangle, listed in reverse order."))
    return out


def _coordinate_proofs():
    """Symbolic vertices (a, 0), (0, b), ... with a, b, c positive."""
    out = []
    dist = lambda p, q: p.distance(q)
    out.append(g("distance", ["Point(a, 0)", "Point(0, b)"], dist))
    out.append(g("distance", ["Point(x, 0)", O], dist,
                 note="x has no assumptions, so sqrt(x**2) stays; the next case has a positive."))
    out.append(g("distance", ["Point(a, 0)", O], dist))
    hyp_mid = "Point(a/2, b/2)"
    for v in [O, "Point(a, 0)"]:
        out.append(g("distance", [hyp_mid, v], dist,
                     note="The midpoint of the hypotenuse is equidistant from all three vertices; "
                          "SymPy does not take the 1/4 out of the root."))
    for p, q, note in [
        ("Point(0, 0)", "Point(a + b, c)", "Parallelogram diagonal from (0, 0)."),
        ("Point(a, 0)", "Point(b, c)", "The other diagonal: the same midpoint."),
    ]:
        out.append(g("midpoint", [p, q], lambda u, v: u.midpoint(v), note=note))
    out.append(g("slope", ["Line(Point(a, 0), Point(0, b))"], lambda l: l.slope))
    sp_ = lambda l1, l2: l1.slope * l2.slope
    for l1, l2, note in [
        ("Line(Point(0, 0), Point(a, b))", "Line(Point(a, b), Point(a - b, a + b))",
         "Adjacent sides of the square on (0, 0), (a, b): slopes b/a and -a/b."),
        ("Line(Point(0, 0), Point(a, a))", "Line(Point(a, 0), Point(0, a))", "Diagonals of a square."),
        ("Line(Point(0, 0), Point(a, 0))", "Line(Point(0, 0), Point(0, b))",
         "Horizontal times vertical: 0*oo is nan."),
    ]:
        out.append(g("slope_product", [l1, l2], sp_, note=note))
    out.append(g("is_parallel", ["Line(Point(0, 0), Point(b, c))", "Line(Point(a, 0), Point(a + b, c))"],
                 lambda p, q: p.is_parallel(q), note="Opposite sides of a parallelogram."))
    out.append(g("is_perpendicular", ["Line(Point(0, 0), Point(a, 0))", "Line(Point(0, 0), Point(0, b))"],
                 lambda p, q: p.is_perpendicular(q)))
    rt = "Triangle(Point(0, 0), Point(a, 0), Point(0, b))"
    out.append(g("circumcenter", [rt], lambda t: t.circumcenter))
    out.append(g("area", [rt], lambda t: t.area))
    return out


def generate():
    return (
        _measurements()
        + _lines()
        + _intersections()
        + _areas()
        + _triangles()
        + _tangents()
        + _transformations()
        + _coordinate_proofs()
    )
