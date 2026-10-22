"""Physical quantities with sympy.physics.units, as SymPy-parity cases.

The physics quantity grader on the homework app's roadmap would read a
student's "12 m/s", convert it to the key's unit and compare dimensions. SymPy
has no unit-string parser and no quantity grader, so this topic records only
what the pinned SymPy does: its parsers on unit text as a student types it,
and the sympy.physics.units calls a grader would build on (convert_to and the
SI unit system's dimension queries). The inputs include the ones where SymPy's
answer is surprising, which is where a port is most likely to diverge: unit
names that shadow or miss (N, S, C, sec, min, in, mph), unicode that Python's
tokenizer folds into one name (kg·m, s², °C), Float coefficients, and
convert_to returning its input unchanged when the target does not fit.

UNITS below is the namespace `from sympy.physics.units import *` gives: every
name in sympy.physics.units.__all__, bound to that attribute (meter, m, s, N,
kg, kPa, newton, joule, degree, ohm, ..., and Dimensions such as force,
length, energy). Every case is one SymPy call:

  parse_expr     args [text]: parse_expr(text, local_dict=UNITS,
                 transformations=standard_transformations +
                 (implicit_multiplication, convert_xor)) from
                 sympy.parsing.sympy_parser, for unit text with no backslash.
                 options records these keyword arguments: local_dict as the
                 module name "sympy.physics.units" (meaning UNITS), and
                 transformations as the names of the functions in order. A
                 name missing from UNITS falls back to SymPy's own namespace
                 (sec is the secant function; parse_expr maps Python's min to
                 Min) or becomes a plain Symbol (mph, kN, furlongs).
  parse_latex    args [LaTeX]: parse_latex(text, backend="lark") from
                 sympy.parsing.latex, the backend the app uses, for unit text
                 with a backslash; options records {"backend": "lark"}.

For the remaining ops each arg is SymPy syntax read with
sympify(arg, locals=UNITS): unit names are sympy.physics.units Quantities
(spelled out in full: meter, kilometer, second, hour, newton, ...), dimension
names are its Dimensions, and a typed decimal is a Float. SI below is
UnitSystem.get_unit_system("SI").

  convert_to                   args [expr, target]:
                               sympy.physics.units.convert_to(expr, target).
                               target may be a list of units.
  get_dimensional_expr         args [expr]: SI.get_dimensional_expr(expr), an
                               expression in dimension names (each unit
                               contributes its own: newton -> force).
  get_dimensional_dependencies args [dimension]:
                               SI.get_dimension_system()
                                 .get_dimensional_dependencies(dimension),
                               a dict {base Dimension: power}, printed by
                               sympy.sstr(), which sorts the keys. The
                               dimension is the str() of a
                               get_dimensional_expr answer recorded in this
                               topic.
  check_dimensions             args [expr]:
                               sympy.physics.units.util.check_dimensions(expr),
                               which returns expr or raises ValueError when
                               addends have unlike dimensions.

The expected answer is SymPy's str() of the result, or the exception's class
name when the call raises. The parse_expr answers for text with unicode
(·, ², °) depend on Python's tokenizer, pinned to Python 3.13 in CI.
"""

import sympy as sp
from sympy.parsing.latex import parse_latex
from sympy.parsing.sympy_parser import (
    convert_xor, implicit_multiplication, parse_expr, standard_transformations,
)
from sympy.physics import units as u
from sympy.physics.units import convert_to
from sympy.physics.units.unitsystem import UnitSystem
from sympy.physics.units.util import check_dimensions

from caselib import outcome

TOPIC = "units"

UNITS = {name: getattr(u, name) for name in u.__all__}
SI = UnitSystem.get_unit_system("SI")

TRANSFORMATIONS = standard_transformations + (implicit_multiplication, convert_xor)
PARSE_EXPR_OPTIONS = {
    "local_dict": "sympy.physics.units",
    "transformations": [f.__name__ for f in TRANSFORMATIONS],
}
BACKEND = "lark"


def Q(s):
    """An arg as SymPy reads it (see the module docstring)."""
    return sp.sympify(s, locals=dict(UNITS))


# --- case builders ------------------------------------------------------------

def typed(src, note=None):
    """Unit text as a student types it: parse_expr, or parse_latex when the
    text has a backslash."""
    if "\\" in src:
        return outcome("parse_latex", [src], lambda: parse_latex(src, backend=BACKEND),
                       options={"backend": BACKEND}, note=note)
    return outcome(
        "parse_expr", [src],
        lambda: parse_expr(src, local_dict=dict(UNITS), transformations=TRANSFORMATIONS),
        options=PARSE_EXPR_OPTIONS, note=note,
    )


def conv(expr, target, note=None):
    return outcome("convert_to", [expr, target], lambda: convert_to(Q(expr), Q(target)), note=note)


def dim(expr, note=None):
    return outcome("get_dimensional_expr", [expr], lambda: SI.get_dimensional_expr(Q(expr)), note=note)


def deps(expr, note=None):
    """get_dimensional_dependencies of the dimension SymPy gives for expr."""
    dimension = sp.sstr(SI.get_dimensional_expr(Q(expr)))
    return outcome(
        "get_dimensional_dependencies", [dimension],
        lambda: SI.get_dimension_system().get_dimensional_dependencies(Q(dimension)),
        note=note,
    )


def checked(expr, note=None):
    return outcome("check_dimensions", [expr], lambda: check_dimensions(Q(expr)), note=note)


# --- cases --------------------------------------------------------------------

def _typed_text():
    """Unit text read by parse_expr with the units namespace."""
    return [
        typed("12 m/s", "m and s are meter and second in the units namespace, not plain Symbols."),
        typed("9.8 m/s²", "A typed decimal is a Float; the superscript two is part of the name s², "
              "an unknown name, so it becomes a plain Symbol."),
        typed("9.8 m/s/s", "Repeated division."),
        typed("5 N", "N is newton in the units namespace, which shadows SymPy's N()."),
        typed("3 kg·m/s", "U+00B7 MIDDLE DOT is a Python identifier character, so kg·m is one plain Symbol."),
        typed("3 kg m/s", "A space between names is implicit multiplication."),
        typed("60 km/h", "h is hour in the units namespace."),
        typed("25 mph", "The units namespace has no mph, so it becomes a plain Symbol."),
        typed("3 hr", "The units namespace has no hr, so it becomes a plain Symbol."),
        typed("5 min", "min is not in the units namespace; parse_expr maps Python's min to SymPy's Min, "
              "and an Integer times a function class raises TypeError."),
        typed("4 s", "s is second in the units namespace."),
        typed("4 sec", "sec is not in the units namespace, so it is SymPy's sec function, and an Integer "
              "times a function class raises TypeError."),
        typed("3 ms", "ms is millisecond in the units namespace, not meter*second."),
        typed("3 m·s", "The middle dot makes m·s one plain Symbol."),
        typed("5m", "No space between the number and the unit."),
        typed("12 in", "in is a Python keyword, so the text is a Python SyntaxError."),
        typed("5 g", "g is gram in the units namespace."),
        typed("450 nm", "nm is nanometer in the units namespace."),
        typed("1.2 kN", "The units namespace has no kN, so it becomes a plain Symbol."),
        typed("101.3 kPa", "kPa is in the units namespace; the Quantity is named kilopascal."),
        typed("2 A", "A is ampere in the units namespace."),
        typed("300 K", "K is kelvin in the units namespace."),
        typed("1.5 L", "L is liter in the units namespace; the decimal is a Float."),
        typed("250 mL", "mL is milliliter in the units namespace, not meter*liter."),
        typed("6 ft", "ft is foot in the units namespace."),
        typed("4186 J/(kg·°C)", "Python's tokenizer reads kg·°C as one name, so it becomes one plain Symbol."),
        typed("20 °C", "°C is read as one name and becomes a plain Symbol; the units namespace has no "
              "Celsius unit."),
        typed("68 °F", "°F is read as one name and becomes a plain Symbol."),
        typed("20 C", "C is coulomb in the units namespace."),
        typed("30°", "A bare degree sign becomes the plain Symbol °, not the degree unit."),
        typed("12", "A bare number with no unit."),
        typed("m", "A lone m is meter."),
        typed("12 furlongs", "An unknown unit name becomes a plain Symbol."),
        typed("1.2 KN", "KN is one unknown name: implicit_multiplication does not split names into letters."),
        typed("4 S", "S is siemens in the units namespace, which shadows SymPy's S."),
        typed("3 kgm/s", "kgm is one unknown name, so it becomes a plain Symbol."),
    ]


def _typed_latex():
    """Unit text in LaTeX, read by the Lark parser."""
    return [
        typed(r"12\,\mathrm{m/s}", "The \\, spacing parses; the Lark grammar accepts \\mathrm only as the differential \\mathrm{d}, so \\mathrm{m/s} matches no terminal at the backslash."),
        typed(r"12\,\frac{\mathrm{m}}{\mathrm{s}}", "The unit written as a fraction of \\mathrm names."),
        typed(r"9.8\,\mathrm{m/s^2}", "A decimal magnitude and a squared unit inside \\mathrm."),
        typed(r"-9.8\,\mathrm{m\,s^{-2}}", "A negative magnitude and a negative power inside \\mathrm."),
        typed(r"3\,\mathrm{kg\cdot m/s}", "\\cdot between units inside \\mathrm."),
        typed(r"2.5\times10^{3}\,\mathrm{J}", "The \\times10^{3} magnitude parses; the parse fails at \\mathrm."),
        typed(r"10\,\Omega", "The Lark grammar has no \\Omega."),
        typed(r"20^{\circ}\mathrm{C}", "The Lark grammar has no \\circ."),
        typed(r"3 \text{ m}", "The Lark grammar accepts \\text only as the differential \\text{d}, so \\text{ m} matches no terminal."),
    ]


def _convert_to():
    return [
        conv("12*meter/second", "kilometer/hour"),
        conv("60*kilometer/hour", "meter/second"),
        conv("25*mile/hour", "meter/second"),
        conv("3*hour", "second"),
        conv("90*minute", "hour"),
        conv("5*newton", "[meter, kilogram, second]", "A list target: express in these units together."),
        conv("3*kilogram*meter/second", "newton*second"),
        conv("1*newton*meter", "joule", "The coefficient is 1, so the result prints as the bare unit."),
        conv("450*nanometer", "meter"),
        conv("60*hertz", "1/minute"),
        conv("30*degree", "radian"),
        conv("9.8*meter/second**2", "foot/second**2",
             "A typed decimal is a Float, so the converted coefficient is a Float."),
        conv("12.47*meter", "centimeter", "A Float coefficient times 100."),
        conv("0.5*liter", "milliliter", "A Float coefficient times 1000."),
        conv("5*meter", "second", "A target of another dimension: convert_to returns the input unchanged."),
        conv("5*newton", "joule", "Force to energy: the base dimensions overlap but no power of joule "
             "matches, so convert_to returns the input unchanged."),
        conv("12", "meter", "A bare number: convert_to returns it unchanged."),
    ]


def _dimensions():
    return [
        dim("12*meter/second"),
        dim("49/5*meter/second**2"),
        dim("5*newton"),
        dim("3*kilogram*meter/second",
            "Each unit contributes its own dimension, so this is not SymPy's momentum dimension."),
        dim("2*liter", "liter's dimension is length**3."),
        dim("30*degree", "Angles are dimensionless in SI."),
        dim("12", "A bare number is dimensionless."),
        dim("newton*meter", "Each unit keeps its own dimension name: force*length, not energy."),
        dim("kilometer/hour - meter/second", "A difference of like dimensions."),
        dim("meter + second", "A sum of unlike dimensions: get_dimensional_expr looks only at the "
            "first term."),
        dim("joule"),
        dim("3*newton*second"),
        dim("10*ohm"),
        deps("newton*meter", "In SI base dimensions, force*length and energy agree."),
        deps("joule"),
        deps("3*newton*second"),
        deps("10*ohm", "impedance in base dimensions, with current."),
        checked("meter + second", "Addends of unlike dimensions."),
        checked("kilometer/hour - meter/second", "Addends of like dimensions: the expression comes back."),
    ]


def generate():
    return _typed_text() + _typed_latex() + _convert_to() + _dimensions()
