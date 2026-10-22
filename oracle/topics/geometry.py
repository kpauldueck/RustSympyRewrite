"""Geometry: sympy.geometry keys for figures, triangle centres and transformations.

The homework app builds geometry figures and their exact keys with
sympy.geometry (app_sympy_audit.md, "Geometry"): Point, Segment, Line, Ray,
Circle, Triangle, Polygon; intersections, angles, triangle centres;
reflect/rotate/translate/scale; Polygon equality up to vertex order; "in terms
of pi" and special-right-triangle keys; coordinate proofs with symbolic
vertices.

Every arg is a constructor string in SymPy syntax, read with sympify, e.g.
"Triangle(Point(0, 0), Point(4, 0), Point(0, 3))". Numbers inside are exact
("1/2" is a Rational). SymPy prints Point(...) as Point2D(...), Line as
Line2D, Segment as Segment2D. "Line(Eq(y, 2*x + 1))" builds a line from its
equation in x and y; "Line(Point(0, 2), slope=3)" from a point and a slope.
Symbolic coordinates for coordinate proofs use a, b, c, declared positive in
options["assumptions"]; any other letter is a plain symbol.

Ops:
  distance       [p, q]: p.distance(q). q may be a Line or Segment: the
                 shortest distance from the point.
  midpoint       [p, q]: p.midpoint(q); or [segment]: segment.midpoint.
  slope          [line]: .slope of a Line or Segment; oo when vertical.
  slope_product  [l1, l2]: l1.slope*l2.slope (coordinate proofs: -1 means
                 perpendicular).
  intersection   [e1, e2]: e1.intersection(e2), a list of Point2D, or of the
                 common Line/Segment when the two overlap; [] when disjoint.
                 compare "set" (points are listed by x, then y).
  is_parallel, is_perpendicular
                 [l1, l2]: True/False.
  angle_between  [l1, l2]: Line.angle_between, the angle between the two
                 direction vectors (so it can be obtuse), exact.
  smallest_angle_between
                 [l1, l2]: the acute angle between the lines.
  area           [polygon or circle]. Polygon.area is signed in SymPy; see the
                 spec cases for clockwise and degenerate figures.
  perimeter      [polygon]; circumference [circle].
  arc_length, sector_area
                 [circle, central angle in radians] (no SymPy op: spec).
  side_lengths   [triangle]: [|AB|, |BC|, |CA|] for Triangle(A, B, C), the
                 lengths of SymPy's Triangle.sides.
  angles         [triangle]: Triangle.angles, {vertex: angle}, printed with
                 the vertices sorted by x, then y.
  circumcenter, incenter, orthocenter, centroid, circumradius, inradius
                 [triangle].
  medians, altitudes, bisectors
                 [triangle]: {vertex: Segment from that vertex to the opposite
                 side}, printed with the vertices sorted by x, then y.
  is_similar     [t1, t2]; is_right [t].
  tangent_lines  [circle, point]: the tangent lines through the point as
                 equations Eq(y, m*x + b), or Eq(x, c) when vertical, sorted.
                 SymPy returns Line2D objects through the point and a second
                 point of its choosing; the equations come from
                 Line2D.coefficients. [] when the point is inside the circle.
                 compare "set".
  reflect        [entity, line]: entity.reflect(line).
  rotate         [entity, angle] or [entity, angle, center]:
                 entity.rotate(angle, pt=center); counterclockwise, radians.
  translate      [entity, dx, dy].
  scale          [entity, sx, sy] or [entity, sx, sy, center]:
                 entity.scale(sx, sy, pt=center); a dilation by k is
                 sx = sy = k (SymPy scales about the origin unless pt= is
                 given).
                 A transformed polygon lists the images of its vertices in the
                 original vertex order.
  polygon_equal_up_to_order
                 [p1, p2]: SymPy's Polygon ==, which already ignores the
                 starting vertex and the direction of travel (so it is not
                 order sensitive), but a different cyclic order is a
                 different polygon.
"""

from sympy import Eq, N, Symbol, default_sort_key, symbols, sympify
from sympy.geometry import Point

from caselib import case, spec_case

TOPIC = "geometry"

x, y = symbols("x y")
a = Symbol("a", positive=True)
b = Symbol("b", positive=True)
c = Symbol("c", positive=True)
LOCALS = {"a": a, "b": b, "c": c, "x": x, "y": y}
POSITIVE = {a, b, c}

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
    """Read a constructor string the way the Rust side will: sympify."""
    return sympify(src, locals=LOCALS)


def _assumptions(args):
    syms = set()
    for s in args:
        if isinstance(s, str):
            syms |= getattr(G(s), "free_symbols", set())
    pos = sorted((s for s in syms if s in POSITIVE), key=str)
    if not pos:
        return None
    return {"assumptions": {str(s): {"positive": True} for s in pos}}


def _numeric_key(p):
    return (float(N(p.x)), float(N(p.y)))


def _points_sorted(res):
    if isinstance(res, list) and res and all(isinstance(p, Point) for p in res):
        return sorted(res, key=_numeric_key)
    return res


def _vertex_dict(d):
    """A {vertex: value} dict. SymPy's printer sorts keys by default_sort_key;
    the docstring promises x-then-y order, so check that they agree here."""
    assert sorted(d, key=default_sort_key) == sorted(d, key=_numeric_key), d
    return d


def _line_equation(line):
    A, B, C = line.coefficients  # A*x + B*y + C = 0
    if B == 0:
        return Eq(x, -C / A)
    return Eq(y, -A / B * x - C / B)


def g(op, args, fn, compare="exact", note=None):
    """A SymPy-sourced geometry case: fn receives the sympified args."""
    result = fn(*[G(s) for s in args])
    return case(op, args, result, compare, options=_assumptions(args), note=note)


def _measurements():
    out = []
    dist = lambda p, q: p.distance(q)
    for p, q, note in [
        (O, "Point(3, 4)", None),
        (O, "Point(2, 2)", "sqrt(8) is printed as 2*sqrt(2)."),
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
        ("Line(Point(1, 2), Point(1, 8))", "Vertical line: SymPy's slope is oo; the app shows it as undefined."),
        ("Line(Eq(y, -2*x/3 + 4))", "A line given by its equation."),
    ]:
        out.append(g("slope", [line], lambda l: l.slope, note=note))
    return out


def _lines():
    out = []
    for l1, l2, want in [
        ("Line(Point(0, 0), Point(1, 2))", "Line(Point(3, 0), Point(4, 2))", True),
        ("Line(Point(0, 0), Point(1, 2))", "Line(Point(3, 0), Point(4, 3))", False),
        ("Line(Point(1, 0), Point(1, 1))", "Line(Point(3, 0), Point(3, 5))", True),
    ]:
        out.append(g("is_parallel", [l1, l2], lambda p, q: p.is_parallel(q)))
    for l1, l2, note in [
        ("Line(Point(0, 0), Point(1, 2))", "Line(Point(0, 0), Point(2, -1))", None),
        ("Line(Point(0, 1), Point(5, 1))", "Line(Point(3, 0), Point(3, 5))", None),
        ("Line(Point(0, 2), slope=2)", "Line(Point(1, 1), slope=1/2)",
         "Near miss: reciprocal slope without the sign change is not perpendicular."),
        ("Line(Point(0, 0), Point(1, 2))", "Line(Point(0, 0), Point(1, -2))",
         "Near miss: opposite slope without the reciprocal is not perpendicular."),
    ]:
        out.append(g("is_perpendicular", [l1, l2], lambda p, q: p.is_perpendicular(q), note=note))
    for l2, note in [
        (Y_EQ_X, None),
        ("Line(Point(0, 0), Point(-1, 1))",
         "Angle between direction vectors, so 3*pi/4 here; smallest_angle_between gives pi/4."),
        ("Line(Point(0, 0), Point(2, 1))", "Not a special angle: exact acos."),
    ]:
        out.append(g("angle_between", [X_AXIS, l2], lambda p, q: p.angle_between(q), note=note))
    out.append(g("angle_between", ["Line(Point(0, 0), Point(1, 2))", "Line(Point(0, 0), Point(3, 1))"],
                 lambda p, q: p.angle_between(q)))
    out.append(g("smallest_angle_between", [X_AXIS, "Line(Point(0, 0), Point(-1, 1))"],
                 lambda p, q: p.smallest_angle_between(q)))
    return out


def _intersections():
    out = []
    inter = lambda p, q: _points_sorted(p.intersection(q))
    for e1, e2, note in [
        ("Line(Point(0, 1), Point(1, 3))", "Line(Point(0, 3), Point(3, 0))", "y = 2*x + 1 meets y = 3 - x at a rational point."),
        ("Line(Point(0, 0), Point(1, 1))", "Line(Point(0, 2), Point(1, 3))", "Parallel lines: empty."),
        ("Line(Point(0, 0), Point(1, 1))", "Line(Point(2, 2), Point(3, 3))",
         "Coincident lines: SymPy returns the argument line itself as the one element."),
        (Y_EQ_X, "Circle(Point(0, 0), 2)", "Secant through the centre: irrational points."),
        ("Line(Point(0, 1), Point(1, 2))", "Circle(Point(0, 0), 5)", "y = x + 1 meets x**2 + y**2 = 25 at lattice points."),
        ("Line(Point(0, 5), Point(1, 5))", "Circle(Point(0, 0), 5)", "Tangent line: one point."),
        ("Line(Point(0, 6), Point(1, 6))", "Circle(Point(0, 0), 5)", "The line misses the circle: empty."),
        ("Circle(Point(0, 0), 5)", "Circle(Point(6, 0), 5)", None),
        ("Circle(Point(0, 0), 2)", "Circle(Point(5, 0), 3)", "Externally tangent circles: one point."),
        ("Circle(Point(0, 0), 1)", "Circle(Point(0, 0), 2)", "Concentric circles: empty."),
        ("Segment(Point(0, 0), Point(4, 4))", "Segment(Point(0, 4), Point(4, 0))", None),
        ("Segment(Point(0, 0), Point(1, 1))", "Segment(Point(0, 4), Point(4, 0))",
         "The lines cross at (2, 2) but the segments do not reach it: empty."),
        ("Segment(Point(0, 0), Point(2, 2))", "Segment(Point(2, 2), Point(4, 0))",
         "Segments are closed: a shared endpoint is an intersection point."),
        ("Segment(Point(0, 0), Point(3, 0))", "Segment(Point(1, 0), Point(5, 0))", "Collinear overlap: the common segment."),
        ("Ray(Point(0, 0), Point(1, 1))", "Line(Point(-3, 0), Point(0, -3))",
         "The line meets the ray's line behind the ray's start: empty."),
    ]:
        out.append(g("intersection", [e1, e2], inter, compare="set", note=note))
    same = ["Circle(Point(0, 0), 2)", "Circle(Point(0, 0), 2)"]
    out.append(spec_case(
        "intersection", same, "[Circle(Point(0, 0), 2)]",
        "Identical circles: SymPy returns a bare Circle instead of a list; every other intersection is a list, so wrap it.",
        compare="set", sympy_gives=lambda: G(same[0]).intersection(G(same[1])),
    ))
    return out


def _areas():
    out = []
    for fig, note in [
        (T345, None),
        ("Polygon(Point(0, 0), Point(6, 0), Point(4, 3), Point(1, 3))", "Trapezoid."),
        ("RegularPolygon(Point(0, 0), 2, 6)", "Regular hexagon with circumradius 2."),
        ("Circle(Point(0, 0), 3)", "In terms of pi."),
        ("Circle(Eq(x**2 + y**2 - 4*x + 6*y - 3, 0))", "Circle from its general equation: centre (2, -3), radius 4."),
    ]:
        out.append(g("area", [fig], lambda f: f.area, note=note))
    cw = "Triangle(Point(0, 0), Point(0, 3), Point(4, 0))"
    out.append(spec_case(
        "area", [cw], "6",
        "SymPy's Polygon.area is signed (negative for clockwise vertices); the area of a figure is the absolute value.",
        sympy_gives=lambda: G(cw).area,
    ))
    flat = "Triangle(Point(0, 0), Point(1, 1), Point(2, 2))"
    out.append(spec_case(
        "area", [flat], "invalid:degenerate_triangle",
        "Collinear vertices: SymPy's Triangle silently becomes a Segment2D (which has no area); refuse by name.",
        compare="error", sympy_gives=lambda: G(flat).area,
    ))
    bowtie = "Polygon(Point(0, 0), Point(2, 2), Point(2, 0), Point(0, 2))"
    out.append(spec_case(
        "area", [bowtie], "invalid:self_intersecting_polygon",
        "Vertices out of order make a bow tie whose signed areas cancel; SymPy silently answers 0.",
        compare="error", sympy_gives=lambda: G(bowtie).area,
    ))
    neg = "Circle(Point(0, 0), -2)"
    out.append(spec_case(
        "area", [neg], "invalid:negative_radius",
        "SymPy accepts a negative radius and reports a negative area; a figure key with radius -2 is a typo.",
        compare="error", sympy_gives=lambda: G(neg).area,
    ))
    out.append(g("perimeter", [T_ACUTE], lambda f: f.perimeter))
    out.append(g("perimeter", ["Polygon(Point(0, 0), Point(4, 0), Point(5, 3), Point(1, 3))"],
                 lambda f: f.perimeter, note="Parallelogram."))
    out.append(g("circumference", ["Circle(Point(2, -1), 5/2)"], lambda f: f.circumference))
    for circ, ang, arc, sector in [
        ("Circle(Point(0, 0), 6)", "pi/3", "2*pi", "6*pi"),
        ("Circle(Point(0, 0), 9)", "120*pi/180", "6*pi", "27*pi"),
    ]:
        out.append(spec_case(
            "arc_length", [circ, ang], arc,
            "SymPy has no arc length op: r*theta, exact in terms of pi (a typed 120 degrees arrives as 120*pi/180).",
        ))
        out.append(spec_case(
            "sector_area", [circ, ang], sector,
            "SymPy has no sector area op: r**2*theta/2, exact in terms of pi.",
        ))
    return out


def _triangles():
    out = []
    sides = lambda t: [s.length for s in t.sides]
    for tri, note in [
        ("Triangle(Point(0, 0), Point(7, 0), Point(0, 7))", "45-45-90 with legs 7: hypotenuse 7*sqrt(2)."),
        ("Triangle(Point(0, 0), Point(10, 0), Point(5, 5))", "45-45-90 with hypotenuse 10: legs 5*sqrt(2)."),
        ("Triangle(Point(0, 0), Point(12, 0), Point(3, 3*sqrt(3)))",
         "30-60-90 with hypotenuse 12: legs 6 and 6*sqrt(3) (sqrt(108) simplified)."),
    ]:
        out.append(g("side_lengths", [tri], sides, note=note))
    angles = lambda t: _vertex_dict(t.angles)
    for tri, note in [
        (T345, "Only the right angle is special; the others are exact acos values."),
        ("Triangle(Point(0, 0), Point(12, 0), Point(3, 3*sqrt(3)))", "30-60-90."),
        (T_EQUI, None),
        (T_OBTUSE, "Obtuse angle at the origin: acos of a negative number."),
    ]:
        out.append(g("angles", [tri], angles, note=note))
    for op, tri, note, cmp in [
        ("circumcenter", T345, "Right triangle: the midpoint of the hypotenuse.", "exact"),
        ("circumcenter", T_EQUI, "Equilateral: every centre is (1, sqrt(3)/3).", "exact"),
        ("incenter", T345, None, "exact"),
        ("incenter", T_ISO, None, "exact"),
        ("incenter", T_ACUTE,
         "SymPy leaves radicals in the denominators; compare each coordinate by value (about (2.4076, 1.4880)).",
         "equivalent"),
        ("orthocenter", T345, "Right triangle: the right-angle vertex.", "exact"),
        ("orthocenter", T_ACUTE, None, "exact"),
        ("orthocenter", T_OBTUSE, "Obtuse triangle: outside the triangle.", "exact"),
        ("centroid", T_ACUTE, None, "exact"),
        ("circumradius", T_ISO, None, "exact"),
        ("inradius", T345, None, "exact"),
    ]:
        out.append(g(op, [tri], lambda t, op=op: getattr(t, op), compare=cmp, note=note))
    out.append(g("medians", [T_ISO], lambda t: _vertex_dict(t.medians)))
    out.append(g("altitudes", [T345], lambda t: _vertex_dict(t.altitudes),
                 note="In a right triangle two altitudes are the legs."))
    out.append(g("bisectors", [T345], lambda t: _vertex_dict(t.bisectors()),
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
    tl = lambda circ, p: sorted((_line_equation(l) for l in circ.tangent_lines(p)), key=default_sort_key)
    for circ, p, note in [
        ("Circle(Point(0, 0), 5)", "Point(5, 5)", "The tangents are x = 5 and y = 5."),
        ("Circle(Point(0, 0), 5)", "Point(0, 10)", "Slopes +-sqrt(3)."),
        ("Circle(Point(0, 0), 5)", "Point(3, 4)", "A point on the circle: the one tangent there."),
        ("Circle(Point(0, 0), 5)", "Point(1, 1)", "A point inside: no tangent lines."),
    ]:
        out.append(g("tangent_lines", [circ, p], tl, compare="set", note=note))
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
    ]:
        out.append(g("reflect", [ent, line], refl, note=note))
    circ = "Circle(Point(2, 3), 1)"
    out.append(spec_case(
        "reflect", [circ, X_AXIS], "Circle(Point(2, -3), 1)",
        "SymPy 1.14 reflects a Circle to one with radius -1; a reflection keeps the radius.",
        sympy_gives=lambda: G(circ).reflect(G(X_AXIS)),
    ))
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
        (["Point(3, 1)", "2", "2", "Point(1, 1)"], "Dilation by 2 about (1, 1): SymPy's pt= argument."),
        ([T_MOVE, "1/2", "1/2", "Point(1, 1)"], None),
        ([T_MOVE, "-1", "-1"], "Dilation by -1: the half turn about the origin."),
        (["Circle(Point(1, 1), 1)", "3", "3"], "Dilating a circle by 3 about the origin: centre and radius both scale."),
        (["Circle(Point(0, 0), 1)", "2", "1"], "Unequal factors stretch a circle into an Ellipse."),
    ]:
        out.append(g("scale", args, sc, note=note))
    eq = lambda p, q: p == q
    for p2, note in [
        ("Polygon(Point(2, 2), Point(0, 2), Point(0, 0), Point(2, 0))", "Same square, different starting vertex."),
        ("Polygon(Point(0, 0), Point(0, 2), Point(2, 2), Point(2, 0))", "Same square traversed clockwise."),
        ("Polygon(Point(0, 0), Point(2, 2), Point(2, 0), Point(0, 2))",
         "Same vertex set in another cyclic order is a different (self-intersecting) polygon."),
        ("Polygon(Point(0, 0), Point(1, 0), Point(2, 0), Point(2, 2), Point(0, 2))",
         "SymPy drops the collinear vertex (1, 0), so this is the same square."),
    ]:
        out.append(g("polygon_equal_up_to_order", [SQUARE, p2], eq, note=note))
    out.append(g("polygon_equal_up_to_order", [T345, "Polygon(Point(0, 3), Point(4, 0), Point(0, 0))"], eq,
                 note="A three-vertex Polygon is a Triangle; reversed order still matches."))
    return out


def _coordinate_proofs():
    """Symbolic vertices (a, 0), (0, b), ... with a, b, c positive."""
    out = []
    dist = lambda p, q: p.distance(q)
    out.append(g("distance", ["Point(a, 0)", "Point(0, b)"], dist))
    out.append(g("distance", ["Point(x, 0)", O], dist,
                 note="x has no assumptions (it may be complex), so sqrt(x**2) stays; compare the positive a below."))
    out.append(g("distance", ["Point(a, 0)", O], dist, note="a is positive, so the distance is a."))
    hyp_mid = "Point(a/2, b/2)"
    for v in [O, "Point(a, 0)"]:
        out.append(g("distance", [hyp_mid, v], dist, compare="equivalent",
                     note="The midpoint of the hypotenuse is equidistant from all three vertices: "
                          "sqrt(a**2 + b**2)/2 (SymPy prints sqrt(a**2/4 + b**2/4))."))
    for p, q, note in [
        ("Point(0, 0)", "Point(a + b, c)", "Parallelogram diagonal from (0, 0)."),
        ("Point(a, 0)", "Point(b, c)", "The other diagonal: same midpoint, so the diagonals bisect each other."),
    ]:
        out.append(g("midpoint", [p, q], lambda u, v: u.midpoint(v), note=note))
    out.append(g("slope", ["Line(Point(a, 0), Point(0, b))"], lambda l: l.slope))
    sp_ = lambda l1, l2: l1.slope * l2.slope
    for l1, l2, note in [
        ("Line(Point(0, 0), Point(a, b))", "Line(Point(a, b), Point(a - b, a + b))",
         "Adjacent sides of the square on (0, 0), (a, b): slopes b/a and -a/b."),
        ("Line(Point(0, 0), Point(a, a))", "Line(Point(a, 0), Point(0, a))", "Diagonals of a square."),
        ("Line(Point(0, 0), Point(a, 0))", "Line(Point(0, 0), Point(0, b))",
         "Horizontal times vertical is 0*oo = nan: slope products cannot show these are perpendicular; use is_perpendicular."),
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
