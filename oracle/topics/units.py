"""Physical quantities for the physics quantity grader (ops "parse_quantity",
"convert_to", "dimension", "quantity_equivalent", "within_resolution").

Args, except parse_quantity's, are SymPy syntax over the unit names in
UNIT_NAMES and SHORT_ONLY below, spelled out in full (meter, kilometer,
second, hour, newton, kilopascal, ...), each meaning that
sympy.physics.units Quantity. Short aliases
(m, s, N, h, C, ...) are never used in args: in SymPy's own units namespace
many of them collide (N is newton but also N(), S is siemens, c is the speed
of light, plain sympify reads `degree` as the polynomial-degree function).
Typed decimals in args are exact rationals (9.8 means 49/5), as everywhere
in rsympy, so "9.8*meter/second**2" is 49*meter/(5*second**2). `celsius` and
`fahrenheit` are rsympy's own names (SymPy has no Celsius or Fahrenheit
unit): absolute temperatures on those scales, valid only as number*celsius.

convert_to      args [quantity, target unit]: sympy.physics.units.convert_to,
                source sympy where SymPy is right. Decided departures (spec):
                decimals stay exact (SymPy gives Floats); a target of another
                dimension is "invalid:dimension_mismatch" (SymPy silently
                returns the input unchanged); a bare number is
                "invalid:missing_unit"; celsius/fahrenheit/kelvin convert by
                the affine formulas K = C + 273.15 and F = 9/5*C + 32.
dimension       args [quantity]: SymPy's
                UnitSystem.get_unit_system("SI").get_dimensional_expr. Each
                named unit contributes its own dimension name (newton ->
                force, joule -> energy), so newton*meter is force*length,
                not energy; dimensionless (numbers, degree, meter/meter) is 1.
                options {"base": true}: the same in SI base dimensions
                (get_dimension_system().get_dimensional_dependencies), so
                newton*meter and joule both give length**2*mass/time**2;
                this is what the grader uses to decide a dimension mismatch.
                A sum of unlike dimensions is "invalid:dimension_mismatch"
                (spec: SymPy returns the first term's dimension).
parse_quantity  args [student text] -> number*unit in the names above
                (spec: SymPy has no unit-string parser; sympy_gives is the
                naive route, parse_expr over the whole sympy.physics.units
                namespace with implicit multiplication, or the Lark LaTeX
                parser for input containing a backslash). The magnitude is
                read by parse_number's rules (exact decimals, \\times10^{n},
                thousands commas). The unit uses a closed, case-sensitive
                vocabulary, matched as whole words (never split into letters):
                m, cm, mm, km, nm, um/µm/μm; s, sec, second(s), ms, µs/μs,
                min, h, hr, hour(s); kg, g, mg; N, J, W, Hz, Pa, C, V, A,
                Ω/\\Omega, K; mi, ft, in, mph; L, mL; ° (angle); °C, °F; the SI
                prefixes k and M on N, J, W, Pa, Hz; the words meter(s),
                metre(s), second(s), minute(s), hour(s). Compound units use
                /, ·, \\cdot, *, a space, ^ (also unicode superscripts),
                negative powers, parentheses, and \\mathrm{...}, \\text{...},
                \\frac{...}{...} wrappers; LaTeX spacing (\\, \\;) is a space.
                The result keeps the unit as typed when sympy.physics.units
                has a Quantity for it (km -> kilometer, nm -> nanometer,
                ms -> millisecond, mL -> milliliter, mph -> mile/hour,
                kPa -> kilopascal); a prefixed unit SymPy has no Quantity
                for is folded into the number (1.2 kN -> 1200*newton).
                Temperatures (decided): a standalone °C or °F is an absolute
                temperature (20 °C -> 20*celsius); inside a compound unit
                °C is a temperature step, equal to kelvin (J/(kg·°C) ->
                joule/(kelvin*kilogram)). A bare C is coulomb, never Celsius.
                Errors: no unit -> "invalid:missing_unit"; no number ->
                "invalid:missing_number"; a unit outside the vocabulary ->
                "refused:<unit as typed>".
quantity_equivalent  args [answer, key] (SymPy syntax, as parse_quantity
                returns them) -> True/False (spec: SymPy has no such
                operation; the expected answer is computed below with
                SymPy's convert_to on exact rationals). The answer is
                converted to the key's unit. No options: exactly equal.
                options {"rel_tol": "1/100"}: |answer - key| <= rel_tol*|key|.
                options {"abs_tol": quantity}: |answer - key| <= abs_tol, the
                tolerance itself carrying a unit. Bounds are inclusive and
                exact (no Floats). Unlike base dimensions ->
                compare "error", "invalid:dimension_mismatch"; but a bare
                number against a key with a unit is "invalid:missing_unit"
                (a free retry, as in parse_quantity, not graded wrong).
                Temperatures compare as absolute kelvin. sympy_gives is
                SymPy converting the answer to the key's unit with Floats,
                as a naive grader would (SymPy has no Celsius/Fahrenheit).
within_resolution  args [value, key, resolution] (SymPy syntax, units
                optional and convertible) -> |value - key| <= resolution/2,
                with exact rationals; a unitless value against a key with a
                unit is "invalid:missing_unit", other unlike dimensions are
                "invalid:dimension_mismatch" (spec: the instrument-precision check;
                sympy_gives is the same test with SymPy Floats, which fails
                on boundaries such as 1.3 vs 1.25 at resolution 0.1, and is
                left unevaluated when the units differ).
"""

from lark import Tree
import sympy as sp
from sympy import Abs, Rational, S, sympify
from sympy.parsing.latex import parse_latex
from sympy.parsing.sympy_parser import (
    convert_xor, implicit_multiplication, parse_expr, standard_transformations,
)
from sympy.physics import units as u
from sympy.physics.units import Dimension, Quantity, convert_to
from sympy.physics.units.unitsystem import UnitSystem

from caselib import case, spec_case

TOPIC = "units"

SI = UnitSystem.get_unit_system("SI")

UNIT_NAMES = (
    "meter", "centimeter", "millimeter", "kilometer", "micrometer", "nanometer",
    "second", "millisecond", "microsecond", "minute", "hour", "day",
    "kilogram", "gram", "milligram", "newton", "joule", "watt", "hertz", "pascal",
    "coulomb", "volt", "ampere", "ohm", "kelvin", "degree", "radian",
    "mile", "foot", "inch", "liter", "milliliter",
)

CELSIUS = Quantity("celsius", latex_repr=r"^\circ\mathrm{C}")
FAHRENHEIT = Quantity("fahrenheit", latex_repr=r"^\circ\mathrm{F}")
ZERO_CELSIUS = Rational(27315, 100)  # 0 °C in kelvin, by definition

# SymPy defines some Quantities only under a short name: u.kPa is the Quantity
# named kilopascal, and there is no u.kilopascal.
SHORT_ONLY = {"kilopascal": u.kPa}
SYMPY_NAMES = dict({n: getattr(u, n) for n in UNIT_NAMES}, **SHORT_ONLY)
NAMES = dict(SYMPY_NAMES, celsius=CELSIUS, fahrenheit=FAHRENHEIT)


def Q(s):
    """rsympy's reading of an arg: unit names as Quantities, decimals exact."""
    return sympify(s, locals=NAMES, rational=True)


def Q_float(s):
    """SymPy's own reading of an arg: decimals become Floats."""
    return sympify(s, locals=SYMPY_NAMES)


# --- exact quantity arithmetic, grounded in SymPy's convert_to ---------------

def _absolute_kelvin(q):
    coeff, unit = q.as_coeff_Mul()
    if unit == CELSIUS:
        return (coeff + ZERO_CELSIUS) * u.kelvin
    if unit == FAHRENHEIT:
        return ((coeff - 32) * Rational(5, 9) + ZERO_CELSIUS) * u.kelvin
    assert not q.has(CELSIUS, FAHRENHEIT), q
    return q


def _base_dimension(q):
    """q's dimension in SI base dimensions, as a product of name symbols."""
    deps = SI.get_dimension_system().get_dimensional_dependencies(
        Dimension(SI.get_dimensional_expr(q))
    )
    return sp.Mul(*[d.name ** p for d, p in deps.items()])


def _unit_of(q):
    """The unit part of q (1 for a plain number)."""
    if not q.has(Quantity):
        return S.One
    return q.as_coeff_Mul()[1]


def _value_in(q, unit):
    """q as an exact number of `unit`s, or None if the dimensions differ."""
    if _base_dimension(q) != _base_dimension(unit):
        return None
    value = sp.simplify(convert_to(q, unit) / unit) if unit != 1 else q
    assert not value.has(Quantity), (q, unit, value)
    return value


def _missing_unit(value, key):
    """A bare number against a key with a unit is a missing unit (a free
    retry), not a wrong dimension (graded incorrect)."""
    return not Q(value).has(Quantity) and Q(key).has(Quantity)


def _quantity_equivalent(answer, key, options):
    if _missing_unit(answer, key):
        return "invalid:missing_unit"
    a, k = _absolute_kelvin(Q(answer)), _absolute_kelvin(Q(key))
    unit = _unit_of(k)
    va, vk = _value_in(a, unit), _value_in(k, unit)
    if va is None:
        return "invalid:dimension_mismatch"
    if not options:
        return bool(va == vk)
    if "rel_tol" in options:
        return bool(Abs(va - vk) <= Rational(options["rel_tol"]) * Abs(vk))
    tol = _value_in(Q(options["abs_tol"]), unit)
    assert tol is not None, options
    return bool(Abs(va - vk) <= tol)


def _within_resolution(value, key, resolution):
    if _missing_unit(value, key):
        return "invalid:missing_unit"
    k = Q(key)
    unit = _unit_of(k)
    vv, vk, vr = _value_in(Q(value), unit), _value_in(k, unit), _value_in(Q(resolution), unit)
    if vv is None or vr is None:
        return "invalid:dimension_mismatch"
    return bool(Abs(vv - vk) <= vr / 2)


# --- what SymPy does, for sympy_gives ----------------------------------------

_FULL_NAMESPACE = {k: v for k, v in sorted(vars(u).items()) if not k.startswith("_")}
_NAIVE = standard_transformations + (implicit_multiplication, convert_xor)


def _naive_parse(src):
    """The naive SymPy route for a unit string (see the module docstring).
    Lark's error messages list expected tokens in hash order, so they are cut
    off to keep the output deterministic."""
    if "\\" not in src:
        return lambda: parse_expr(src, local_dict=dict(_FULL_NAMESPACE), transformations=_NAIVE)

    def lark():
        try:
            out = parse_latex(src, backend="lark")
        except Exception as e:  # noqa: BLE001 - recording SymPy's behaviour
            msg = " ".join(str(e).split()).split(" Expected one of")[0]
            return f"error: {type(e).__name__}: {msg[:120]}"
        assert not isinstance(out, Tree), src
        return out

    return lark


def _no_temperature_scale():
    return getattr(u, "celsius")


# --- case builders ------------------------------------------------------------

def _spec(op, args, expected, note, *, compare="exact", options=None, sympy_gives=None):
    """spec_case, with `expected` given as a SymPy object (or bool / error
    kind). Unit expressions cannot go through spec_case's plain sympify (it
    would read `degree` as a function and units as bare Symbols), so the
    object is passed as is and its units-aware LaTeX is added afterwards."""
    c = spec_case(op, args, expected, note, compare=compare, options=options,
                  sympy_gives=sympy_gives)
    if isinstance(expected, sp.Basic) and not isinstance(expected, sp.logic.boolalg.BooleanAtom):
        out = {}
        for k, v in c.items():
            out[k] = v
            if k == "expected":
                out["expected_latex"] = sp.latex(expected)
        c = out
    return c


def conv(q, target, note=None):
    result = convert_to(Q(q), Q(target))
    target_q = Q(target)
    if not isinstance(target_q, list):
        assert _value_in(Q(q), target_q) is not None, (q, target)
    return case("convert_to", [q, target], result, note=note)


def conv_spec(q, target, expected, note, *, compare="exact", sympy_gives=None):
    return _spec("convert_to", [q, target], expected, note, compare=compare,
                 sympy_gives=sympy_gives)


def _temperature_in(q, target):
    """Affine temperature conversion of q (any of the three scales)."""
    kelvin = _absolute_kelvin(Q(q)).as_coeff_Mul()[0]
    t = Q(target)
    if t == u.kelvin:
        return kelvin * u.kelvin
    celsius = kelvin - ZERO_CELSIUS
    if t == CELSIUS:
        return celsius * CELSIUS
    assert t == FAHRENHEIT, target
    return (celsius * Rational(9, 5) + 32) * FAHRENHEIT


def conv_temp(q, target, why):
    expected = _temperature_in(q, target)
    # Round trip back to the source scale as a self-check.
    back = _temperature_in(sp.sstr(expected), sp.sstr(_unit_of(Q(q))))
    assert back == Q(q), (q, target, back)
    return conv_spec(q, target, expected,
                     f"affine temperature conversion (K = C + 273.15, F = 9/5*C + 32); {why}",
                     sympy_gives=_no_temperature_scale)


def dim(q, note=None):
    return case("dimension", [q], SI.get_dimensional_expr(Q(q)), note=note)


def dim_base(q, note=None):
    return case("dimension", [q], _base_dimension(Q(q)), options={"base": True}, note=note)


PARSE = "closed unit vocabulary"


def pq(src, expected, why):
    exp = Q(expected) if isinstance(expected, str) else expected
    return _spec("parse_quantity", [src], exp, f"{PARSE}; {why}", sympy_gives=_naive_parse(src))


def pq_err(src, kind, why):
    return _spec("parse_quantity", [src], kind, f"{PARSE}; {why}", compare="error",
                 sympy_gives=_naive_parse(src))


def _prefixed(coeff, prefix, unit):
    """coeff * (prefix unit), folded into the base unit by SymPy. The prefix
    must multiply the unit first: SymPy keeps `coeff*prefix*unit` as an
    unevaluated product containing the Prefix."""
    out = Q(coeff) * (prefix * unit)
    assert not out.has(sp.physics.units.prefixes.Prefix), out
    return out


def qeq(answer, key, why, options=None):
    got = _quantity_equivalent(answer, key, options)
    compare = "error" if isinstance(got, str) else "exact"

    def sympy_gives():
        return convert_to(Q_float(answer), _unit_of(Q_float(key)))

    if any(t in s for s in (answer, key) for t in ("celsius", "fahrenheit")):
        # Q_float would make a bare Symbol named celsius; say what SymPy lacks.
        sympy_gives = _no_temperature_scale
    note = f"quantity grader, exact after converting the answer to the key's unit; {why}"
    return _spec("quantity_equivalent", [answer, key], got, note, compare=compare,
                 options=options, sympy_gives=sympy_gives)


def within(value, key, resolution, why):
    got = _within_resolution(value, key, resolution)
    compare = "error" if isinstance(got, str) else "exact"

    def sympy_gives():
        return Abs(Q_float(value) - Q_float(key)) <= Q_float(resolution) / 2

    return _spec("within_resolution", [value, key, resolution], got,
                 f"instrument precision |value - key| <= resolution/2, exact; {why}",
                 compare=compare, sympy_gives=sympy_gives)


# --- cases --------------------------------------------------------------------

def _convert_to():
    out = [
        conv("12*meter/second", "kilometer/hour"),
        conv("60*kilometer/hour", "meter/second"),
        conv("25*mile/hour", "meter/second"),
        conv("3*hour", "second"),
        conv("90*minute", "hour"),
        conv("5*newton", "[meter, kilogram, second]", note="a list target: express in these base units"),
        conv("3*kilogram*meter/second", "newton*second"),
        conv("1*newton*meter", "joule", note="coefficient 1 is not printed"),
        conv("450*nanometer", "meter"),
        conv("60*hertz", "1/minute"),
        conv("30*degree", "radian"),
    ]
    out += [
        conv_spec(
            "9.8*meter/second**2", "foot/second**2",
            convert_to(Q("9.8*meter/second**2"), Q("foot/second**2")),
            "typed decimals are exact, so the conversion stays exact (SymPy gives a Float)",
            sympy_gives=lambda: convert_to(Q_float("9.8*meter/second**2"), Q_float("foot/second**2")),
        ),
        conv_spec(
            "5*meter", "second", "invalid:dimension_mismatch",
            "a target of another dimension is an error; SymPy silently returns the input",
            compare="error",
            sympy_gives=lambda: convert_to(Q_float("5*meter"), Q_float("second")),
        ),
        conv_spec(
            "12", "meter", "invalid:missing_unit",
            "a bare number has no unit to convert; SymPy silently returns it",
            compare="error",
            sympy_gives=lambda: convert_to(Q_float("12"), Q_float("meter")),
        ),
        conv_temp("20*celsius", "kelvin", "room temperature"),
        conv_temp("68*fahrenheit", "celsius", "the offset matters, not just the 9/5 factor"),
        conv_temp("300*kelvin", "celsius", "kelvin to Celsius"),
        conv_temp("-40*celsius", "fahrenheit", "the scales cross at -40"),
    ]
    return out


def _dimension():
    out = [
        dim("12*meter/second"),
        dim("49/5*meter/second**2"),
        dim("5*newton"),
        dim("3*kilogram*meter/second", note="momentum has no dimension name in SymPy"),
        dim("2*liter", note="volume is length**3"),
        dim("30*degree", note="angles are dimensionless in SI"),
        dim("12", note="a bare number is dimensionless"),
        dim("newton*meter", note="SymPy keeps each unit's own dimension name: force*length, not energy"),
        dim("kilometer/hour - meter/second", note="a difference of like dimensions"),
        dim_base("newton*meter", note="base dimensions: torque and energy agree"),
        dim_base("joule"),
        dim_base("3*newton*second"),
        dim_base("10*ohm"),
    ]
    out += [
        _spec("dimension", ["meter + second"], "invalid:dimension_mismatch",
              "adding unlike dimensions is an error; SymPy's get_dimensional_expr returns "
              "the first term's dimension", compare="error",
              sympy_gives=lambda: SI.get_dimensional_expr(Q_float("meter + second"))),
        _spec("dimension", ["20*celsius"], sp.Symbol("temperature"),
              "rsympy's celsius is a temperature; SymPy has no Celsius unit",
              sympy_gives=_no_temperature_scale),
    ]
    return out


def _parse_quantity():
    ms = "12*meter/second"
    g = "49/5*meter/second**2"
    return [
        pq("12 m/s", ms, "the basic case (fails in the app's numeric grader today)"),
        pq("12\\,\\mathrm{m/s}", ms, "LaTeX: thin space and \\mathrm wrapper (refused by the app today)"),
        pq("12\\,\\frac{\\mathrm{m}}{\\mathrm{s}}", ms, "unit written as a LaTeX fraction"),
        pq("9.8\\,\\mathrm{m/s^2}", g, "exact decimal magnitude and a squared unit"),
        pq("9.8 m/s²", g, "unicode superscript two"),
        pq("9.8 m/s/s", g, "repeated division is left-associative"),
        pq("-9.8\\,\\mathrm{m\\,s^{-2}}", "-" + g, "negative magnitude; negative power instead of /"),
        pq("5 N", "5*newton", "N is newton"),
        pq("3 kg·m/s", "3*kilogram*meter/second", "middle dot U+00B7 multiplies units"),
        pq("3\\,\\mathrm{kg\\cdot m/s}", "3*kilogram*meter/second", "\\cdot inside \\mathrm"),
        pq("3 kg m/s", "3*kilogram*meter/second", "a space multiplies units"),
        pq("2.5\\times10^{3}\\,\\mathrm{J}", "2500*joule", "scientific magnitude"),
        pq("60 km/h", "60*kilometer/hour", "h is hour"),
        pq("25 mph", "25*mile/hour", "mph is mile/hour (SymPy has no mph)"),
        pq("3 hr", "3*hour", "hr is hour (SymPy has no hr)"),
        pq("5 min", "5*minute", "min is minute, never min() or minimum"),
        pq("4 s", "4*second", "s is second, not a variable"),
        pq("4 sec", "4*second", "sec is second, not secant"),
        pq("3 ms", "3*millisecond", "ms is millisecond, never meter*second"),
        pq("3 m·s", "3*meter*second", "meter times second needs an explicit separator"),
        pq("5m", "5*meter", "no space between number and unit"),
        pq("12 in", "12*inch", "in is inch, not the word"),
        pq("5 g", "5*gram", "g is gram, not gravitational acceleration"),
        pq("450 nm", "450*nanometer", "a prefixed unit SymPy names keeps its name"),
        pq("1.2 kN", _prefixed("1.2", u.kilo, u.newton),
           "SymPy has no kilonewton Quantity, so the prefix folds into the number"),
        pq("101.3 kPa", u.kPa * Rational(1013, 10),
           "SymPy has a kilopascal Quantity (u.kPa), so this prefixed unit keeps its name, unlike kN"),
        pq("10\\,\\Omega", "10*ohm", "\\Omega is ohm"),
        pq("2 A", "2*ampere", "A is ampere, not a variable"),
        pq("300 K", "300*kelvin", "K is kelvin, an absolute temperature with no degree sign"),
        pq("1.5 L", "3/2*liter", "capital L is liter"),
        pq("250 mL", "250*milliliter", "mL is milliliter, never meter*liter"),
        pq("6 ft", "6*foot", "ft is foot"),
        pq("4186 J/(kg·°C)", "4186*joule/(kilogram*kelvin)",
           "decided: °C inside a compound unit is a temperature step, equal to kelvin"),
        pq("20 °C", CELSIUS * 20, "decided: a standalone °C is an absolute Celsius temperature"),
        pq("20^{\\circ}\\mathrm{C}", CELSIUS * 20, "LaTeX degree-Celsius"),
        pq("68 °F", FAHRENHEIT * 68, "decided: a standalone °F is an absolute Fahrenheit temperature"),
        pq("20 C", "20*coulomb", "a bare C is coulomb, never Celsius"),
        pq("30°", "30*degree", "a bare degree sign is an angle"),
        pq("3 \\text{ m}", "3*meter", "\\text wrapper with a leading space"),
        pq_err("12", "invalid:missing_unit", "a bare number on a quantity item is a free retry"),
        pq_err("m", "invalid:missing_number", "a unit with no number; m is still metre, never a variable"),
        pq_err("12 furlongs", "refused:furlongs", "unknown units are refused by name, never a Symbol"),
        pq_err("1.2 KN", "refused:KN", "units are case-sensitive whole words: never kelvin*newton"),
        pq_err("4 S", "refused:S", "capital S (siemens) is not in the vocabulary, never seconds"),
        pq_err("3 kgm/s", "refused:kgm", "whole words only: kgm is not split into kg*m"),
    ]


def _quantity_equivalent_cases():
    rel = {"rel_tol": "1/100"}
    return [
        qeq("12*meter/second", "216/5*kilometer/hour", "the same speed in km/h"),
        qeq("12*kilometer/hour", "12*meter/second",
            "the right number in the wrong unit of the same dimension is False, not an error"),
        qeq("60*kilometer/hour", "16.7*meter/second", "a rounded answer is not exactly equal"),
        qeq("60*kilometer/hour", "16.7*meter/second", "within 1% after conversion", options=rel),
        qeq("60*kilometer/hour", "17*meter/second", "2% off, outside 1%", options=rel),
        qeq("101*meter", "100*meter", "the relative bound is inclusive", options=rel),
        qeq("101.01*meter", "100*meter", "just outside the relative bound", options=rel),
        qeq("-9.8*meter/second**2", "9.8*meter/second**2", "the sign matters", options=rel),
        qeq("9.7*meter/second**2", "9.8*meter/second**2",
            "exact boundary of an absolute tolerance (Floats say 0.100000000000001 > 0.1)",
            options={"abs_tol": "0.1*meter/second**2"}),
        qeq("12.47*meter", "1250*centimeter", "an absolute tolerance in a third unit (3 cm <= 5 cm)",
            options={"abs_tol": "5*centimeter"}),
        qeq("5*newton*meter", "5*joule", "decided: same base dimensions, so torque units equal energy units"),
        qeq("3*kilogram*meter/second", "3*newton*second", "momentum in two spellings"),
        qeq("0.5*liter", "500*milliliter", "volume units; exact decimal"),
        qeq("20*celsius", "293.15*kelvin", "temperatures compare as absolute kelvin"),
        qeq("68*fahrenheit", "20*celsius", "Fahrenheit vs Celsius"),
        qeq("20*celsius", "20*kelvin", "Celsius is not kelvin for absolute temperatures"),
        qeq("12*meter", "12*second", "wrong dimension is an error, not just False"),
        qeq("5*newton", "5*joule", "force vs energy"),
        qeq("12", "12*meter", "a bare number is a missing unit (free retry), not a wrong dimension"),
    ]


def _within_resolution_cases():
    return [
        within("12.3", "12.34", "0.1", "0.04 <= 0.05"),
        within("12.3", "12.36", "0.1", "0.06 > 0.05"),
        within("1.3", "1.25", "0.1", "exact boundary 0.05 <= 0.05; Floats say False"),
        within("3.14", "pi", "0.01", "an exact irrational key"),
        within("12.4*centimeter", "124.4*millimeter", "1*millimeter", "units convert before the check"),
        within("2.5*second", "2.53*second", "0.01*second", "0.03 s > 0.005 s"),
        within("12*meter", "12*second", "1*meter", "unlike dimensions"),
        within("12.3", "12.34*centimeter", "0.1*centimeter",
               "the key has a unit and the value does not: a missing unit, not a wrong dimension"),
    ]


def generate():
    return (
        _parse_quantity()
        + _convert_to()
        + _dimension()
        + _quantity_equivalent_cases()
        + _within_resolution_cases()
    )
