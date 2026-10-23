"""Generate rsympy-core's class and assumption tables from the pinned SymPy.

    PYTHONHASHSEED=0 SYMPY_GROUND_TYPES=python .venv/bin/python oracle/gen_core_tables.py

Writes crates/rsympy-core/src/generated.rs. Everything in it is read off
live SymPy objects, so the Rust side never retypes SymPy's tables:

- the assumption facts and their compiled inference rules (the same
  `_assume_rules` object `FactKB.deduce_all_facts` runs on);
- for each SymPy class rsympy models: its name, `class_key()`, the abstract
  classes it is an instance of, its `default_assumptions` (already
  deduced) and which facts have an `_eval_is_*` handler, with the class that
  defines it;
- `ordering_of_classes`, which `Basic.compare` uses.

CI regenerates the file and fails if it differs from what is committed.
To model a new SymPy class, add it to CLASSES (and, if code needs to test
for it, an abstract base to TAGS), rerun, then port its handlers.
"""

import pathlib
import subprocess

import sympy as sp
from sympy.core import basic, numbers
from sympy.core.assumptions import _assume_defined, _assume_rules
from sympy.external.gmpy import GROUND_TYPES

assert sp.__version__ == "1.14.0", sp.__version__
assert GROUND_TYPES == "python", "set SYMPY_GROUND_TYPES=python"

OUT = pathlib.Path(__file__).parent.parent / "crates" / "rsympy-core" / "src" / "generated.rs"

# (Rust ClassId variant, SymPy class). Order is the enum's order.
CLASSES = [
    ("Integer", numbers.Integer),
    ("Zero", numbers.Zero),
    ("One", numbers.One),
    ("NegativeOne", numbers.NegativeOne),
    ("Rational", numbers.Rational),
    ("Half", numbers.Half),
    ("Float", numbers.Float),
    ("Infinity", numbers.Infinity),
    ("NegativeInfinity", numbers.NegativeInfinity),
    ("NaN", numbers.NaN),
    ("ComplexInfinity", numbers.ComplexInfinity),
    ("ImaginaryUnit", numbers.ImaginaryUnit),
    ("Exp1", numbers.Exp1),
    ("Pi", numbers.Pi),
    ("EulerGamma", numbers.EulerGamma),
    ("Catalan", numbers.Catalan),
    ("GoldenRatio", numbers.GoldenRatio),
    ("TribonacciConstant", numbers.TribonacciConstant),
    ("Symbol", sp.Symbol),
    ("Dummy", sp.Dummy),
    ("Wild", sp.Wild),
    ("Add", sp.Add),
    ("Mul", sp.Mul),
    ("Pow", sp.Pow),
    # f(x) for an undefined f: SymPy makes a class per name, so the name is
    # data here and `class_name()` reads it from the node.
    ("AppliedUndef", sp.core.function.AppliedUndef),
    ("Exp", sp.exp),
    ("Log", sp.log),
    ("Sin", sp.sin),
    ("Cos", sp.cos),
    ("Tan", sp.tan),
    ("Cot", sp.cot),
    ("Sec", sp.sec),
    ("Csc", sp.csc),
    ("Asin", sp.asin),
    ("Acos", sp.acos),
    ("Atan", sp.atan),
    ("Acot", sp.acot),
    ("Sinh", sp.sinh),
    ("Cosh", sp.cosh),
    ("Tanh", sp.tanh),
    ("Abs", sp.Abs),
    ("Sign", sp.sign),
    ("Re", sp.re),
    ("Im", sp.im),
    ("Arg", sp.arg),
    ("Conjugate", sp.conjugate),
    ("Floor", sp.floor),
    ("Ceiling", sp.ceiling),
    ("Factorial", sp.factorial),
    ("Binomial", sp.binomial),
    ("Mod", sp.Mod),
    ("Min", sp.Min),
    ("Max", sp.Max),
    ("Asinh", sp.asinh),
    ("Acosh", sp.acosh),
    ("Atanh", sp.atanh),
    ("Coth", sp.coth),
    ("Atan2", sp.atan2),
    ("Gamma", sp.gamma),
    ("RisingFactorial", sp.RisingFactorial),
    ("FallingFactorial", sp.FallingFactorial),
    ("KroneckerDelta", sp.KroneckerDelta),
    ("Heaviside", sp.Heaviside),
    ("DiracDelta", sp.DiracDelta),
    ("Piecewise", sp.Piecewise),
    ("ExprCondPair", sp.functions.elementary.piecewise.ExprCondPair),
    ("Lambda", sp.Lambda),
    ("Derivative", sp.Derivative),
    ("Subs", sp.Subs),
    ("Integral", sp.Integral),
    ("Sum", sp.Sum),
    ("Product", sp.Product),
    ("Limit", sp.Limit),
    ("Order", sp.Order),
    ("Equality", sp.Eq),
    ("Unequality", sp.Ne),
    ("StrictLessThan", sp.StrictLessThan),
    ("LessThan", sp.LessThan),
    ("StrictGreaterThan", sp.StrictGreaterThan),
    ("GreaterThan", sp.GreaterThan),
    ("BooleanTrue", sp.logic.boolalg.BooleanTrue),
    ("BooleanFalse", sp.logic.boolalg.BooleanFalse),
    ("And", sp.And),
    ("Or", sp.Or),
    ("Not", sp.Not),
    ("Tuple", sp.Tuple),
    ("Str", sp.core.symbol.Str),
    ("Interval", sp.Interval),
    ("FiniteSet", sp.FiniteSet),
    ("Union", sp.Union),
    ("Intersection", sp.Intersection),
    ("Complement", sp.Complement),
    ("EmptySet", sp.sets.sets.EmptySet),
    ("Range", sp.Range),
    ("Indexed", sp.Indexed),
    ("IndexedBase", sp.IndexedBase),
    ("Idx", sp.Idx),
    ("AccumulationBounds", sp.AccumBounds),
    ("IdentityFunction", sp.functions.elementary.miscellaneous.IdentityFunction),
    ("WildFunction", sp.core.function.WildFunction),
]

# Abstract classes code asks `isinstance` about. Each class row gets a bit
# for every one of these in its MRO.
TAGS = [
    ("Basic", sp.Basic),
    ("Atom", sp.Atom),
    ("Expr", sp.Expr),
    ("AtomicExpr", sp.AtomicExpr),
    ("Number", numbers.Number),
    ("Rational", numbers.Rational),
    ("Integer", numbers.Integer),
    ("NumberSymbol", numbers.NumberSymbol),
    ("Symbol", sp.Symbol),
    ("AssocOp", sp.core.operations.AssocOp),
    ("Application", sp.core.function.Application),
    ("Function", sp.Function),
    ("AppliedUndef", sp.core.function.AppliedUndef),
    ("ExpBase", sp.functions.elementary.exponential.ExpBase),
    ("TrigonometricFunction", sp.functions.elementary.trigonometric.TrigonometricFunction),
    ("InverseTrigonometricFunction", sp.functions.elementary.trigonometric.InverseTrigonometricFunction),
    ("HyperbolicFunction", sp.functions.elementary.hyperbolic.HyperbolicFunction),
    ("MinMaxBase", sp.functions.elementary.miscellaneous.MinMaxBase),
    ("LatticeOp", sp.core.operations.LatticeOp),
    ("Boolean", sp.logic.boolalg.Boolean),
    ("BooleanAtom", sp.logic.boolalg.BooleanAtom),
    ("Relational", sp.core.relational.Relational),
    ("Set", sp.Set),
    ("ExprWithLimits", sp.concrete.expr_with_limits.ExprWithLimits),
    ("RoundFunction", sp.functions.elementary.integers.RoundFunction),
    ("CombinatorialFunction", sp.functions.combinatorial.factorials.CombinatorialFunction),
    ("ReciprocalTrigonometricFunction", sp.functions.elementary.trigonometric.ReciprocalTrigonometricFunction),
]

# SymPy's facts plus `polar`, which has a property but no inference rules.
FACTS = sorted(_assume_defined)
FIDX = {f: i for i, f in enumerate(FACTS)}


# Basic's class-level `is_Foo` flags (`is_Add`, `is_Number`, ...).
FLAGS = sorted(k for k, v in vars(sp.Basic).items()
               if k.startswith("is_") and isinstance(v, bool) and k not in ("is_number", "is_symbol"))


def mask(facts):
    m = 0
    for f in facts:
        m |= 1 << FIDX[f]
    return m


def camel(fact):
    return "".join(p.capitalize() for p in fact.split("_"))


# Methods whose defining class the Rust port dispatches on, so that a
# class overriding one we have not ported raises NotPortedYet instead of
# silently getting the default.
METHODS = [
    "_eval_power", "as_base_exp", "as_numer_denom", "as_coeff_Mul",
    "as_coeff_Add", "as_coeff_mul", "as_coeff_add", "could_extract_minus_sign",
    "__neg__", "_eval_evalf", "as_real_imag", "_eval_expand_power_base",
    "_eval_expand_power_exp", "_eval_expand_mul", "_eval_expand_multinomial",
    "_eval_subs", "sort_key", "_hashable_content", "class_key",
    "as_content_primitive", "as_powers_dict", "primitive", "_eval_conjugate",
    "_eval_is_meromorphic", "_eval_derivative", "extract_multiplicatively",
    "_eval_Abs", "_eval_Mod", "_eval_nseries", "_eval_as_leading_term",
    "_eval_rewrite", "_eval_simplify", "_eval_is_polynomial",
    "_eval_is_rational_function", "_to_mpmath", "_as_mpf_val", "__abs__",
    "__mul__", "__add__", "__pow__", "__truediv__", "__mod__", "__floordiv__",
    "__eq__", "__hash__", "__lt__", "__gt__", "__le__", "__ge__", "__bool__",
    "_eval_Eq", "is_comparable", "is_number", "free_symbols", "args",
    "func", "__new__", "eval", "fdiff", "_eval_is_ge", "__len__",
    "_eval_is_extended_real",
]


def lines():
    out = ["// @generated by oracle/gen_core_tables.py from SymPy 1.14.0. Do not edit.", ""]
    out.append("use crate::facts::Fact;")
    out.append("")

    # --- facts ---
    out.append(f"pub const NUM_FACTS: usize = {len(FACTS)};")
    out.append("")
    out.append("pub const FACT_NAMES: [&str; NUM_FACTS] = [")
    out.extend(f'    "{f}",' for f in FACTS)
    out.append("];")
    out.append("")
    out.append("/// Every fact, in `FACT_NAMES` order (sorted, as SymPy's `defined_facts`).")
    out.append("pub const ALL_FACTS: [Fact; NUM_FACTS] = [")
    out.extend(f"    Fact::{camel(f)}," for f in FACTS)
    out.append("];")
    out.append("")
    out.append("macro_rules! define_facts {")
    out.append("    () => {")
    out.append("        /// An assumption fact (`is_<name>`).")
    out.append("        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]")
    out.append("        #[repr(u8)]")
    out.append("        pub enum Fact {")
    out.extend(f"            {camel(f)} = {i}," for i, f in enumerate(FACTS))
    out.append("        }")
    out.append("    };")
    out.append("}")
    out.append("pub(crate) use define_facts;")
    out.append("")

    # full_implications[(fact, value)] -> (true mask, false mask)
    out.append("/// `full_implications[fact, value]` as (facts implied true, facts implied false),")
    out.append("/// indexed by `2 * fact + value`.")
    out.append("pub(crate) const FULL_IMPLICATIONS: [(u64, u64); 2 * NUM_FACTS] = [")
    for f in FACTS:
        for v in (False, True):
            imp = _assume_rules.full_implications.get((f, v), ())
            t = mask(k for k, vv in imp if vv)
            fl = mask(k for k, vv in imp if not vv)
            out.append(f"    ({t:#x}, {fl:#x}), // {f} = {v}")
    out.append("];")
    out.append("")

    out.append("/// `beta_rules` in SymPy's order: (facts required true, facts required false,")
    out.append("/// implied fact, implied value).")
    out.append(f"pub(crate) const BETA_RULES: [(u64, u64, Fact, bool); {len(_assume_rules.beta_rules)}] = [")
    for cond, (f, v) in _assume_rules.beta_rules:
        t = mask(k for k, vv in cond if vv)
        fl = mask(k for k, vv in cond if not vv)
        out.append(f"    ({t:#x}, {fl:#x}, Fact::{camel(f)}, {'true' if v else 'false'}),")
    out.append("];")
    out.append("")

    out.append("/// `beta_triggers[fact, value]`, indexed by `2 * fact + value`, sorted.")
    out.append("pub(crate) const BETA_TRIGGERS: [&[u16]; 2 * NUM_FACTS] = [")
    for f in FACTS:
        for v in (False, True):
            trig = sorted(_assume_rules.beta_triggers.get((f, v), []))
            out.append(f"    &{trig!r}, // {f} = {v}")
    out.append("];")
    out.append("")

    out.append("/// `prereq[fact]` as a mask. SymPy iterates this set in hash order and then")
    out.append("/// shuffles it; rsympy visits it in fact order (see `assume::ask`).")
    out.append("pub(crate) const PREREQ: [u64; NUM_FACTS] = [")
    for f in FACTS:
        out.append(f"    {mask(_assume_rules.prereq.get(f, ())):#x}, // {f}")
    out.append("];")
    out.append("")

    # --- tags ---
    out.append("/// Abstract SymPy classes that code tests with `isinstance`.")
    out.append("#[derive(Clone, Copy, Debug, PartialEq, Eq)]")
    out.append("#[repr(u8)]")
    out.append("pub enum Tag {")
    out.extend(f"    {name} = {i}," for i, (name, _) in enumerate(TAGS))
    out.append("}")
    out.append("")

    # --- classes ---
    out.append("/// The SymPy classes rsympy models.")
    out.append("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]")
    out.append("#[repr(u16)]")
    out.append("pub enum ClassId {")
    out.extend(f"    {name} = {i}," for i, (name, _) in enumerate(CLASSES))
    out.append("}")
    out.append("")
    out.append(f"pub(crate) const NUM_CLASSES: usize = {len(CLASSES)};")
    out.append("")
    out.append("/// Every `ClassId`, indexed by its value.")
    out.append("pub(crate) const ALL_CLASSES: [ClassId; NUM_CLASSES] = [")
    out.extend(f"    ClassId::{name}," for name, _ in CLASSES)
    out.append("];")
    out.append("")
    out.append("pub(crate) const ORDERING_OF_CLASSES: &[&str] = &[")
    out.extend(f'    "{n}",' for n in basic.ordering_of_classes)
    out.append("];")
    out.append("")

    out.append("/// Methods whose defining class rsympy dispatches on (see")
    out.append("/// `ClassInfo::method_owners`).")
    out.append("#[allow(non_camel_case_types)]")
    out.append("#[derive(Clone, Copy, Debug, PartialEq, Eq)]")
    out.append("#[repr(u8)]")
    out.append("pub enum Method {")
    out.extend(f"    {method_variant(m)} = {i}," for i, m in enumerate(METHODS))
    out.append("}")
    out.append("")
    out.append(f"pub(crate) const NUM_METHODS: usize = {len(METHODS)};")
    out.append("")
    out.append("/// Basic's class-level `is_Foo` flags.")
    out.append("#[allow(non_camel_case_types)]")
    out.append("#[derive(Clone, Copy, Debug, PartialEq, Eq)]")
    out.append("#[repr(u8)]")
    out.append("pub enum Flag {")
    out.extend(f"    {name[3:]} = {i}," for i, name in enumerate(FLAGS))
    out.append("}")
    out.append("")
    out.append("pub(crate) struct ClassInfo {")
    out.append("    /// `cls.__name__`.")
    out.append("    pub name: &'static str,")
    out.append("    /// `cls.class_key()[:2]` (the third entry is the name).")
    out.append("    pub class_key: (i32, i32),")
    out.append("    /// Bit `Tag as u8` is set when the class is a subclass of that tag.")
    out.append("    pub tags: u64,")
    out.append("    /// Bit `Flag as u8` is set when `cls.is_<Flag>` is true.")
    out.append("    pub flags: u64,")
    out.append("    /// `cls.is_number` when it is a plain class attribute (`None`: the")
    out.append("    /// `Expr.is_number` property, true when all args are numbers).")
    out.append("    pub is_number: Option<bool>,")
    out.append("    /// `cls.is_symbol`.")
    out.append("    pub is_symbol: bool,")
    out.append("    /// `default_assumptions`: facts it knows, which of those are true, and")
    out.append("    /// facts stored as `None` (a class attribute `is_foo = None`).")
    out.append("    pub default_known: u64,")
    out.append("    pub default_true: u64,")
    out.append("    pub default_none: u64,")
    out.append("    /// Facts with an `_eval_is_*` handler in `_prop_handler`.")
    out.append("    pub handlers: u64,")
    out.append("    /// For each handled fact, the SymPy method that handles it.")
    out.append("    pub handler_owners: &'static [(Fact, &'static str)],")
    out.append("    /// For each [`Method`], the class that defines it for this class")
    out.append("    /// (`cls.__mro__`'s first class with it in `__dict__`), or \"\" if none.")
    out.append("    pub method_owners: [&'static str; NUM_METHODS],")
    out.append("}")
    out.append("")
    out.append("pub(crate) static CLASSES: [ClassInfo; NUM_CLASSES] = [")
    for name, cls in CLASSES:
        da = cls.default_assumptions
        known = mask(k for k, v in da.items() if v is not None)
        true = mask(k for k, v in da.items() if v)
        none = mask(k for k, v in da.items() if v is None)
        handled = sorted(cls._prop_handler.items())
        owners = ", ".join(f'(Fact::{camel(k)}, "{fn.__qualname__}")' for k, fn in handled)
        tags = 0
        for i, (_, t) in enumerate(TAGS):
            if issubclass(cls, t):
                tags |= 1 << i
        ck = cls.class_key()
        flags = 0
        for i, f in enumerate(FLAGS):
            if getattr(cls, f) is True:
                flags |= 1 << i
        num = None
        for klass in cls.__mro__:
            if "is_number" in vars(klass):
                v = vars(klass)["is_number"]
                num = v if isinstance(v, bool) else None
                break
        out.append(f"    // {cls.__module__}.{cls.__qualname__}")
        out.append("    ClassInfo {")
        out.append(f'        name: "{cls.__name__}",')
        out.append(f"        class_key: ({ck[0]}, {ck[1]}),")
        out.append(f"        tags: {tags:#x},")
        out.append(f"        flags: {flags:#x},")
        out.append(f"        is_number: {'None' if num is None else 'Some(' + str(num).lower() + ')'},")
        out.append(f"        is_symbol: {str(bool(getattr(cls, 'is_symbol', False))).lower()},")
        out.append(f"        default_known: {known:#x},")
        out.append(f"        default_true: {true:#x},")
        out.append(f"        default_none: {none:#x},")
        out.append(f"        handlers: {mask(k for k, _ in handled):#x},")
        out.append(f"        handler_owners: &[{owners}],")
        mowners = ", ".join(f'"{method_owner(cls, m)}"' for m in METHODS)
        out.append(f"        method_owners: [{mowners}],")
        out.append("    },")
    out.append("];")
    out.append("")
    return out


def method_variant(m):
    """`as_coeff_add` -> `AsCoeffAdd`; a word SymPy capitalizes keeps its
    underscore, so `as_coeff_Add` -> `AsCoeff_Add` stays distinct."""
    return "".join("_" + w if w[:1].isupper() and i else w[:1].upper() + w[1:]
                   for i, w in enumerate(m.strip("_").split("_")))


def method_owner(cls, m):
    for klass in cls.__mro__:
        if m in vars(klass):
            return klass.__qualname__
    return ""


if __name__ == "__main__":
    OUT.write_text("\n".join(lines()))
    # Format like the rest of the crate, so `cargo fmt` leaves it alone.
    subprocess.run(["rustfmt", "--edition", "2024", str(OUT)], check=True)
    print(f"wrote {OUT}")
