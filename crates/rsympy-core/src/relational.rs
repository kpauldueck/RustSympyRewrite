//! `sympy.core.relational`: the Relational classes (`Eq`, `Ne`, `Lt`,
//! `Le`, `Gt`, `Ge`), their properties (`reversed`, `negated`,
//! `canonical`, ...), the fuzzy comparisons (`is_eq`, `is_ge`, ...) with
//! their multipledispatch tables, and Python's comparison operators on
//! SymPy objects (`Expr.__lt__` and the overrides in `core.numbers`,
//! `core.containers` and `logic.boolalg`).
//!
//! Which `_eval_is_eq` / `_eval_is_ge` implementation a pair of classes
//! dispatches to is read from `generated_relational.rs`, generated from the
//! live `Dispatcher` objects by `oracle/gen_tables_relational.py`.

use rsympy_mpf as mlib;

use crate::assume::{fuzzy_and, fuzzy_not, fuzzy_xor};
use crate::ctx;
use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::Expr;
use crate::generated::{ClassId as C, Method, Tag};
use crate::generated_relational::{
    EVAL_IS_EQ, EVAL_IS_EQ_IMPLS, EVAL_IS_GE, EVAL_IS_GE_IMPLS, PROPER_SUBCLASSES,
};
use crate::numbers::{self, RND, s};
use crate::ops;

// ===========================================================================
// Python's comparison operators
// ===========================================================================

/// One of Python's ordering comparisons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmpOp {
    /// `<` (`__lt__`)
    Lt,
    /// `<=` (`__le__`)
    Le,
    /// `>` (`__gt__`)
    Gt,
    /// `>=` (`__ge__`)
    Ge,
}

impl CmpOp {
    /// The reflected operation Python tries on the right operand (`__lt__`
    /// <-> `__gt__`, `__le__` <-> `__ge__`).
    pub fn reflected(self) -> CmpOp {
        match self {
            CmpOp::Lt => CmpOp::Gt,
            CmpOp::Le => CmpOp::Ge,
            CmpOp::Gt => CmpOp::Lt,
            CmpOp::Ge => CmpOp::Le,
        }
    }

    fn method(self) -> Method {
        match self {
            CmpOp::Lt => Method::Lt,
            CmpOp::Le => Method::Le,
            CmpOp::Gt => Method::Gt,
            CmpOp::Ge => Method::Ge,
        }
    }

    fn symbol(self) -> &'static str {
        match self {
            CmpOp::Lt => "<",
            CmpOp::Le => "<=",
            CmpOp::Gt => ">",
            CmpOp::Ge => ">=",
        }
    }

    /// The Relational class `Expr.__lt__` (etc.) builds.
    pub fn class(self) -> C {
        match self {
            CmpOp::Lt => C::StrictLessThan,
            CmpOp::Le => C::LessThan,
            CmpOp::Gt => C::StrictGreaterThan,
            CmpOp::Ge => C::GreaterThan,
        }
    }

    fn ord<T: PartialOrd>(self, a: &T, b: &T) -> bool {
        match self {
            CmpOp::Lt => a < b,
            CmpOp::Le => a <= b,
            CmpOp::Gt => a > b,
            CmpOp::Ge => a >= b,
        }
    }

    /// `mlib.mpf_lt` and friends.
    fn mpf(self, a: &mlib::Mpf, b: &mlib::Mpf) -> bool {
        match self {
            CmpOp::Lt => mlib::mpf_lt(a, b),
            CmpOp::Le => mlib::mpf_le(a, b),
            CmpOp::Gt => mlib::mpf_gt(a, b),
            CmpOp::Ge => mlib::mpf_ge(a, b),
        }
    }
}

/// Python's `lhs < rhs` for two SymPy objects: `S.true`, `S.false`, or the
/// relational it builds (`StrictLessThan`, or `StrictGreaterThan` with the
/// sides swapped where Python reflects the comparison).
pub fn lt(lhs: &Expr, rhs: &Expr) -> Expr {
    richcmp(lhs, rhs, CmpOp::Lt)
}

/// Python's `lhs <= rhs`.
pub fn le(lhs: &Expr, rhs: &Expr) -> Expr {
    richcmp(lhs, rhs, CmpOp::Le)
}

/// Python's `lhs > rhs`.
pub fn gt(lhs: &Expr, rhs: &Expr) -> Expr {
    richcmp(lhs, rhs, CmpOp::Gt)
}

/// Python's `lhs >= rhs`.
pub fn ge(lhs: &Expr, rhs: &Expr) -> Expr {
    richcmp(lhs, rhs, CmpOp::Ge)
}

/// `type(sub)` is a proper subclass of `type(base)`.
pub(crate) fn is_proper_subclass(sub: &Expr, base: &Expr) -> bool {
    let (a, b) = (sub.cls(), base.cls());
    a != b && PROPER_SUBCLASSES.iter().any(|&(x, y)| x == a && y == b)
}

/// `PyObject_RichCompare(v, w, op)` (CPython's `do_richcompare`): the right
/// operand's reflected method runs first when its type is a proper subclass
/// of the left operand's; a method returning `NotImplemented` hands the
/// comparison to the other operand, and `TypeError` is raised when neither
/// handles it.
pub fn richcmp(v: &Expr, w: &Expr, op: CmpOp) -> Expr {
    let mut checked_reverse = false;
    if is_proper_subclass(w, v) {
        checked_reverse = true;
        if let Some(r) = rich_method(w, v, op.reflected()) {
            return r;
        }
    }
    if let Some(r) = rich_method(v, w, op) {
        return r;
    }
    if !checked_reverse && let Some(r) = rich_method(w, v, op.reflected()) {
        return r;
    }
    raise(
        ErrorKind::TypeError,
        format!(
            "'{}' not supported between instances of '{}' and '{}'",
            op.symbol(),
            v.class_name(),
            w.class_name()
        ),
    )
}

/// `type(a).__lt__(a, b)` (or `__le__`, ...): `None` for `NotImplemented`.
pub fn rich_method(a: &Expr, b: &Expr, op: CmpOp) -> Option<Expr> {
    match a.method_owner(op.method()) {
        "object" | "" => None,
        // Infinity, NegativeInfinity and NaN alias the (wrapped) Expr
        // methods
        "Expr" | "Infinity" | "NegativeInfinity" | "NaN" => expr_rich(a, b, op),
        "Integer" => integer_rich(a, b, op),
        "Rational" => rational_rich(a, b, op),
        "Float" => float_rich(a, b, op),
        // NumberSymbol.__le__ / __ge__ (it has no __lt__ / __gt__ of its own)
        "NumberSymbol" => {
            if a == b {
                return Some(s::true_());
            }
            expr_rich(a, b, op)
        }
        "BooleanAtom" | "BooleanFunction" => raise(
            ErrorKind::TypeError,
            "A Boolean argument can only be used in Eq and Ne; all other relationals \
             expect real expressions.",
        ),
        // Tuple.__lt__ / __le__: _sympify(self.args < other.args)
        "Tuple" => Some(tuple_rich(a, b, op)),
        "floor" => Some(round_rich(a, b, op, true)),
        "ceiling" => Some(round_rich(a, b, op, false)),
        // FiniteSet.__lt__ etc.
        "FiniteSet" => {
            if !b.is_a(Tag::Set) {
                raise(
                    ErrorKind::TypeError,
                    format!("Invalid comparison of set with {}", b.class_name()),
                );
            }
            not_ported("FiniteSet comparison (Set.is_subset)")
        }
        o => not_ported(format!("{o}.__{op:?}__").to_lowercase()),
    }
}

fn boolean(b: bool) -> Expr {
    if b { s::true_() } else { s::false_() }
}

/// `Expr.__lt__(self, other)` (and `__le__`, ...), as wrapped by
/// `sympify_return([('other', 'Expr')], NotImplemented)`: the evaluated
/// `StrictLessThan(self, other)`, or `NotImplemented` when `other` is not
/// an Expr.
fn expr_rich(a: &Expr, b: &Expr, op: CmpOp) -> Option<Expr> {
    if !b.is_a(Tag::Expr) {
        return None;
    }
    Some(new_rel(op.class(), a, b))
}

/// `Integer.__lt__` (etc.).
fn integer_rich(a: &Expr, other: &Expr, op: CmpOp) -> Option<Expr> {
    if other.is_Integer() {
        return Some(boolean(op.ord(&a.p(), &other.p())));
    }
    rational_rich(a, other, op)
}

/// What `Rational._Rrel` returns.
enum Rrel {
    /// `None`
    None,
    /// a value, from `getattr(o, attr)(s)`
    Value(Expr),
    /// `Integer(s.p), s.q*o`
    Pair(Expr, Expr),
}

/// `Rational._Rrel(other, attr)`: `attr` is the method of `other` that
/// answers `self <op> other` (`__gt__` for `__lt__`).
fn rrel(s_: &Expr, other: &Expr, attr: CmpOp) -> Rrel {
    if other.is_Number() {
        let mut s = s_.clone();
        let mut o = other.clone();
        let mut call = false;
        if other.is_NumberSymbol() || other.is_Float() {
            call = true;
        } else if other.is_Rational() {
            s = Expr::integer(&s_.p() * &other.q());
            o = Expr::integer(&s_.q() * &other.p());
            call = true;
        }
        if call {
            // op = getattr(o, attr); return op(s): Float and Integer
            // methods never return NotImplemented for a Number
            return Rrel::Value(rich_method(&o, &s, attr).expect("a Number answers"));
        }
        if o.is_number() && o.is_extended_real() == Some(true) {
            // `s.q*o`: `s.q` is a Python int, so this is `o.__rmul__(s.q)`;
            // `o` is oo or -oo here, whose `__rmul__` is `__mul__`, giving
            // `Mul(o, s.q)` when not evaluating.
            return Rrel::Pair(Expr::integer(s.p()), ops::mul(&o, &Expr::integer(s.q())));
        }
    }
    Rrel::None
}

/// `Rational.__lt__` (etc.): `_Rrel`, then `Expr.__lt__`.
fn rational_rich(a: &Expr, other: &Expr, op: CmpOp) -> Option<Expr> {
    match rrel(a, other, op.reflected()) {
        Rrel::None => expr_rich(a, other, op),
        Rrel::Value(v) => Some(v),
        Rrel::Pair(x, y) => expr_rich(&x, &y, op),
    }
}

/// `Float.__lt__` (etc.).
fn float_rich(a: &Expr, other: &Expr, op: CmpOp) -> Option<Expr> {
    if other.is_a(Tag::NumberSymbol) {
        return rich_method(other, a, op.reflected());
    }
    match frel(a, other, op) {
        Some(v) => Some(v),
        None => expr_rich(a, other, op),
    }
}

/// `Float._Frel(other, op)`.
fn frel(f: &Expr, other: &Expr, op: CmpOp) -> Option<Expr> {
    let (m, _) = f.as_float().expect("a Float");
    if other.is_Rational() {
        // test self*other.q <?> other.p without losing precision
        let smpf = mlib::mpf_mul(m, &mlib::from_int(&other.q(), 0, mlib::ROUND_FAST), 0, RND);
        let ompf = mlib::from_int(&other.p(), 0, mlib::ROUND_FAST);
        return Some(boolean(op.mpf(&smpf, &ompf)));
    }
    if let Some((o, _)) = other.as_float() {
        return Some(boolean(op.mpf(m, o)));
    }
    if other.is_comparable() && !other.is_cls(C::Infinity) && !other.is_cls(C::NegativeInfinity)
    {
        not_ported("Float._Frel against a comparable non-Number (needs evalf)");
    }
    None
}

/// `Tuple.__lt__` / `__le__`: `_sympify(self.args < other.args)`, Python's
/// lexicographic tuple comparison, which returns the comparison of the first
/// unequal items as is (so possibly an unevaluated Relational).
fn tuple_rich(a: &Expr, other: &Expr, op: CmpOp) -> Expr {
    let (x, y) = (a.args(), other.args());
    for (p, q) in x.iter().zip(y) {
        if p != q {
            return richcmp(p, q, op);
        }
    }
    boolean(op.ord(&x.len(), &y.len()))
}

/// `floor.__lt__` / `__le__` / `__gt__` / `__ge__` (`floor` true) and
/// `ceiling`'s (`functions.elementary.integers`), which all share one
/// shape and differ in the constants below.
fn round_rich(a: &Expr, other: &Expr, op: CmpOp, floor: bool) -> Expr {
    use CmpOp::{Ge, Gt, Le, Lt};
    // (the shift added to an integer `other`, the comparison then made
    // with `self.args[0]`, whether `self.args[0] == other` also needs
    // `other.is_noninteger`, what that case returns, and whether the
    // infinity that gives `S.true` for a finite self is `oo` or `-oo`)
    let (shift, cmp, nonint, eq_value, pos_inf) = match (floor, op) {
        (true, Le) => (1, Lt, false, true, true),
        (true, Ge) => (0, Ge, true, false, false),
        (true, Gt) => (1, Ge, false, false, false),
        (true, Lt) => (0, Lt, true, true, true),
        (false, Lt) => (-1, Le, false, false, true),
        (false, Gt) => (0, Gt, true, true, false),
        (false, Ge) => (-1, Gt, false, true, false),
        (false, Le) => (0, Le, true, false, true),
    };
    let arg = a.arg(0);
    if arg.is_real() == Some(true) {
        if other.is_integer() == Some(true) {
            let o = match shift {
                1 => ops::add(other, &s::one()),
                -1 => ops::sub(other, &s::one()),
                _ => other.clone(),
            };
            return richcmp(arg, &o, cmp);
        }
        if other.is_number() && other.is_real() == Some(true) {
            // floor compares with ceiling(other), ceiling with floor(other)
            let r = crate::functions::call(
                if floor { C::Ceiling } else { C::Floor },
                vec![other.clone()],
            );
            return richcmp(arg, &r, cmp);
        }
    }
    if arg == other
        && other.is_real() == Some(true)
        && (!nonint || other.is_noninteger() == Some(true))
    {
        return boolean(eq_value);
    }
    let inf = if pos_inf {
        C::Infinity
    } else {
        C::NegativeInfinity
    };
    if other.is_cls(inf) && a.is_finite() == Some(true) {
        return s::true_();
    }
    raw_rel(op.class(), a, other)
}

// ===========================================================================
// Relational construction
// ===========================================================================

/// `cls(lhs, rhs)` for a Relational class, evaluating as
/// `global_parameters.evaluate` says.
pub fn new_rel(cls: C, lhs: &Expr, rhs: &Expr) -> Expr {
    new_rel_with(cls, lhs, rhs, ctx::evaluate())
}

/// `cls(lhs, rhs, evaluate=evaluate)` for a Relational class
/// (`Equality.__new__`, `Unequality.__new__`, `_Inequality.__new__`).
pub fn new_rel_with(cls: C, lhs: &Expr, rhs: &Expr, evaluate: bool) -> Expr {
    match cls {
        C::Equality => {
            if evaluate {
                let val = is_eq_exact(lhs, rhs);
                if val.is_none() {
                    return raw_rel(cls, lhs, rhs);
                }
                return val.sympify();
            }
            raw_rel(cls, lhs, rhs)
        }
        C::Unequality => {
            if evaluate {
                return match is_neq(lhs, rhs) {
                    None => raw_rel(cls, lhs, rhs),
                    Some(v) => boolean(v),
                };
            }
            raw_rel(cls, lhs, rhs)
        }
        C::StrictLessThan | C::LessThan | C::StrictGreaterThan | C::GreaterThan => {
            if evaluate {
                for me in [lhs, rhs] {
                    if me.is_extended_real() == Some(false) {
                        raise(
                            ErrorKind::TypeError,
                            format!("Invalid comparison of non-real {me:?}"),
                        );
                    }
                    if me.is_cls(C::NaN) {
                        raise(ErrorKind::TypeError, "Invalid NaN comparison");
                    }
                }
                return eval_relation(cls, lhs, rhs);
            }
            raw_rel(cls, lhs, rhs)
        }
        _ => unreachable!("{cls:?} is not a Relational class"),
    }
}

/// `cls(*args, evaluate=evaluate)` for a Relational class (`evaluate`
/// `None`: not passed): `TypeError` unless there are exactly two args
/// (`Eq(x)`, the deprecated one-argument form, now fails this way).
pub fn new_rel_args(cls: C, args: &[Expr], evaluate: Option<bool>) -> Expr {
    let [lhs, rhs] = args else {
        raise(
            ErrorKind::TypeError,
            format!(
                "{}.__new__() takes 2 positional arguments but {} were given",
                crate::generated::CLASSES[cls as usize].name,
                args.len()
            ),
        )
    };
    new_rel_with(cls, lhs, rhs, evaluate.unwrap_or_else(ctx::evaluate))
}

/// `Relational.__new__(cls, lhs, rhs)` for a subclass: `Basic.__new__`.
pub fn raw_rel(cls: C, lhs: &Expr, rhs: &Expr) -> Expr {
    crate::construct::raw(cls, vec![lhs.clone(), rhs.clone()])
}

/// `_Inequality._eval_relation(lhs, rhs)`: `_eval_fuzzy_relation`, or the
/// unevaluated inequality.
pub fn eval_relation(cls: C, lhs: &Expr, rhs: &Expr) -> Expr {
    let val = eval_fuzzy_relation(cls, lhs, rhs);
    if val.is_none() {
        return raw_rel(cls, lhs, rhs);
    }
    val.sympify()
}

/// `cls._eval_fuzzy_relation(lhs, rhs)` for the inequalities.
pub fn eval_fuzzy_relation(cls: C, lhs: &Expr, rhs: &Expr) -> Fuzzy {
    match cls {
        C::GreaterThan => is_ge_exact(lhs, rhs),
        C::LessThan => is_le_exact(lhs, rhs),
        C::StrictGreaterThan => Fuzzy::Py(is_gt(lhs, rhs)),
        C::StrictLessThan => Fuzzy::Py(is_lt(lhs, rhs)),
        _ => unreachable!("{cls:?} is not an inequality"),
    }
}

/// `Eq(lhs, rhs)`.
pub fn equality(lhs: &Expr, rhs: &Expr) -> Expr {
    new_rel(C::Equality, lhs, rhs)
}

/// `Ne(lhs, rhs)`.
pub fn unequality(lhs: &Expr, rhs: &Expr) -> Expr {
    new_rel(C::Unequality, lhs, rhs)
}

/// `Lt(lhs, rhs)`.
pub fn strict_less_than(lhs: &Expr, rhs: &Expr) -> Expr {
    new_rel(C::StrictLessThan, lhs, rhs)
}

/// `Le(lhs, rhs)`.
pub fn less_than(lhs: &Expr, rhs: &Expr) -> Expr {
    new_rel(C::LessThan, lhs, rhs)
}

/// `Gt(lhs, rhs)`.
pub fn strict_greater_than(lhs: &Expr, rhs: &Expr) -> Expr {
    new_rel(C::StrictGreaterThan, lhs, rhs)
}

/// `Ge(lhs, rhs)`.
pub fn greater_than(lhs: &Expr, rhs: &Expr) -> Expr {
    new_rel(C::GreaterThan, lhs, rhs)
}

/// `Relational.ValidRelationOperator[rop]`.
pub fn class_for_rop(rop: Option<&str>) -> Option<C> {
    Some(match rop {
        None | Some("==" | "eq") => C::Equality,
        Some("!=" | "<>" | "ne") => C::Unequality,
        Some(">=" | "ge") => C::GreaterThan,
        Some("<=" | "le") => C::LessThan,
        Some(">" | "gt") => C::StrictGreaterThan,
        Some("<" | "lt") => C::StrictLessThan,
        _ => return None,
    })
}

/// `_nontrivBool(side)`.
fn nontriv_bool(side: &Expr) -> bool {
    side.is_a(Tag::Boolean) && !side.is_a(Tag::Atom)
}

/// `Relational(lhs, rhs, rop, evaluate=evaluate)` (`Rel`): the subclass
/// named by `rop` (`evaluate` `None`: not passed).
pub fn rel(lhs: &Expr, rhs: &Expr, rop: Option<&str>, evaluate: Option<bool>) -> Expr {
    let Some(cls) = class_for_rop(rop) else {
        raise(
            ErrorKind::ValueError,
            format!("Invalid relational operator symbol: {rop:?}"),
        )
    };
    if !matches!(cls, C::Equality | C::Unequality) && (nontriv_bool(lhs) || nontriv_bool(rhs)) {
        raise(
            ErrorKind::TypeError,
            "A Boolean argument can only be used in Eq and Ne; all other relationals \
             expect real expressions.",
        );
    }
    new_rel_with(cls, lhs, rhs, evaluate.unwrap_or_else(ctx::evaluate))
}

/// `_canonical_coeff(rel)`: `rel.canonical` with the coefficient of the
/// left side divided out (`-2*x + 1 < 0` becomes `x > 1/2`).
pub fn canonical_coeff(rel: &Expr) -> Expr {
    let rel = canonical(rel);
    if !rel.is_Relational() || rel.arg(1).is_Boolean() {
        return rel; // Eq(x, True)
    }
    if !rel.arg(0).is_a(Tag::Expr) {
        return reversed(&rel); // e.g.: Eq(True, x) -> Eq(x, True)
    }
    let (b, l) = rel.arg(0).as_coeff_Add(true);
    let (m, lhs) = l.as_coeff_Mul(true);
    let rhs = ops::div(&ops::sub(rel.arg(1), &b), &m);
    if lt(&m, &s::zero()).truthy() {
        return new_rel(reversed(&rel).cls(), &lhs, &rhs);
    }
    new_rel(rel.cls(), &lhs, &rhs)
}

// ===========================================================================
// Relational properties
// ===========================================================================

fn expect_rel(e: &Expr, what: &str) {
    if !e.is_a(Tag::Relational) {
        raise(
            ErrorKind::AttributeError,
            format!("'{}' object has no attribute '{what}'", e.class_name()),
        );
    }
}

/// `self.lhs`.
pub fn lhs(e: &Expr) -> Expr {
    expect_rel(e, "lhs");
    e.arg(0).clone()
}

/// `self.rhs`.
pub fn rhs(e: &Expr) -> Expr {
    expect_rel(e, "rhs");
    e.arg(1).clone()
}

/// `self.rel_op`.
pub fn rel_op(e: &Expr) -> &'static str {
    match e.cls() {
        C::Equality => "==",
        C::Unequality => "!=",
        C::GreaterThan => ">=",
        C::LessThan => "<=",
        C::StrictGreaterThan => ">",
        C::StrictLessThan => "<",
        _ => raise(
            ErrorKind::AttributeError,
            format!("'{}' object has no attribute 'rel_op'", e.class_name()),
        ),
    }
}

/// `self.gts` (`_Greater` / `_Less`): the greater side.
pub fn gts(e: &Expr) -> Expr {
    match e.cls() {
        C::GreaterThan | C::StrictGreaterThan => e.arg(0).clone(),
        C::LessThan | C::StrictLessThan => e.arg(1).clone(),
        _ => raise(
            ErrorKind::AttributeError,
            format!("'{}' object has no attribute 'gts'", e.class_name()),
        ),
    }
}

/// `self.lts`: the lesser side.
pub fn lts(e: &Expr) -> Expr {
    match e.cls() {
        C::GreaterThan | C::StrictGreaterThan => e.arg(1).clone(),
        C::LessThan | C::StrictLessThan => e.arg(0).clone(),
        _ => raise(
            ErrorKind::AttributeError,
            format!("'{}' object has no attribute 'lts'", e.class_name()),
        ),
    }
}

/// `{Eq: Eq, Gt: Lt, Ge: Le, Lt: Gt, Le: Ge, Ne: Ne}`.
fn reversed_class(cls: C) -> C {
    match cls {
        C::GreaterThan => C::LessThan,
        C::LessThan => C::GreaterThan,
        C::StrictGreaterThan => C::StrictLessThan,
        C::StrictLessThan => C::StrictGreaterThan,
        c => c,
    }
}

/// `{Eq: Ne, Ge: Lt, Gt: Le, Le: Gt, Lt: Ge, Ne: Eq}`.
fn negated_class(cls: C) -> C {
    match cls {
        C::Equality => C::Unequality,
        C::Unequality => C::Equality,
        C::GreaterThan => C::StrictLessThan,
        C::StrictGreaterThan => C::LessThan,
        C::LessThan => C::StrictGreaterThan,
        C::StrictLessThan => C::GreaterThan,
        _ => unreachable!("{cls:?} is not a Relational class"),
    }
}

/// `self.reversed`: the sides swapped, unevaluated.
pub fn reversed(e: &Expr) -> Expr {
    expect_rel(e, "reversed");
    raw_rel(reversed_class(e.cls()), e.arg(1), e.arg(0))
}

/// Python's unary `-a` (`type(a).__neg__`).
fn py_neg(a: &Expr) -> Expr {
    match a.method_owner(Method::Neg) {
        "" => raise(
            ErrorKind::TypeError,
            format!("bad operand type for unary -: '{}'", a.class_name()),
        ),
        "Integer" | "Zero" | "One" | "NegativeOne" | "Rational" | "Float" | "Infinity"
        | "NegativeInfinity" | "NaN" | "ComplexInfinity" | "Expr" | "Add" | "Mul" => {
            ops::neg(a)
        }
        o => not_ported(format!("{o}.__neg__")),
    }
}

/// `self.reversedsign`: both sides negated, unevaluated (unless a side is
/// a BooleanAtom).
pub fn reversedsign(e: &Expr) -> Expr {
    expect_rel(e, "reversedsign");
    let (a, b) = (e.arg(0), e.arg(1));
    if !(a.is_a(Tag::BooleanAtom) || b.is_a(Tag::BooleanAtom)) {
        return raw_rel(reversed_class(e.cls()), &py_neg(a), &py_neg(b));
    }
    e.clone()
}

/// `self.negated` for a Relational or a BooleanAtom.
pub fn negated(e: &Expr) -> Expr {
    match e.method_owner(Method::Negated) {
        "Relational" => raw_rel(negated_class(e.cls()), e.arg(0), e.arg(1)),
        "BooleanTrue" => s::false_(),
        "BooleanFalse" => s::true_(),
        "" => raise(
            ErrorKind::AttributeError,
            format!("'{}' object has no attribute 'negated'", e.class_name()),
        ),
        o => not_ported(format!("{o}.negated")),
    }
}

/// `self.weak`: `Ge(*self.args)` for `Gt`, `Le(*self.args)` for `Lt`,
/// otherwise `self`.
pub fn weak(e: &Expr) -> Expr {
    expect_rel(e, "weak");
    match e.cls() {
        C::StrictGreaterThan => new_rel(C::GreaterThan, e.arg(0), e.arg(1)),
        C::StrictLessThan => new_rel(C::LessThan, e.arg(0), e.arg(1)),
        _ => e.clone(),
    }
}

/// `self.strict`: `Gt(*self.args)` for `Ge`, `Lt(*self.args)` for `Le`,
/// otherwise `self`.
pub fn strict(e: &Expr) -> Expr {
    expect_rel(e, "strict");
    match e.cls() {
        C::GreaterThan => new_rel(C::StrictGreaterThan, e.arg(0), e.arg(1)),
        C::LessThan => new_rel(C::StrictLessThan, e.arg(0), e.arg(1)),
        _ => e.clone(),
    }
}

/// `self.canonical` for a Relational or a BooleanAtom.
pub fn canonical(e: &Expr) -> Expr {
    match e.method_owner(Method::Canonical) {
        "Relational" => relational_canonical(e),
        "BooleanAtom" => e.clone(),
        "" => raise(
            ErrorKind::AttributeError,
            format!("'{}' object has no attribute 'canonical'", e.class_name()),
        ),
        o => not_ported(format!("{o}.canonical")),
    }
}

/// `Relational.canonical`.
fn relational_canonical(e: &Expr) -> Expr {
    let args: Vec<Expr> = e
        .args()
        .iter()
        .map(|i| {
            if i.is_a(Tag::Relational) {
                canonical(i)
            } else {
                i.clone()
            }
        })
        .collect();
    let mut r;
    if args.as_slice() != e.args() {
        r = e.func(args.clone());
        if !r.is_a(Tag::Relational) {
            return r;
        }
    } else {
        r = e.clone();
    }
    let (l, rh) = (r.arg(0).clone(), r.arg(1).clone());
    if rh.is_number() {
        if rh.is_Number() && l.is_Number() && gt(&l, &rh).truthy() {
            r = reversed(&r);
        }
    } else if l.is_number() {
        r = reversed(&r);
    } else if crate::compare::ordered(args.clone()) != args {
        r = reversed(&r);
    }
    let (l, rh) = (r.arg(0).clone(), r.arg(1).clone());
    if l.is_a(Tag::BooleanAtom) || rh.is_a(Tag::BooleanAtom) {
        return r;
    }
    // Check if first value has negative sign
    if could_extract_minus_sign(&l) {
        return reversedsign(&r);
    } else if !rh.is_number() && could_extract_minus_sign(&rh) {
        // Right hand side has a minus, but not lhs.
        // How does the expression with reversed signs behave?
        // This is so that expressions of the type
        // Eq(x, -y) and Eq(-x, y)
        // have the same canonical representation
        let expr1 = crate::compare::ordered(vec![l.clone(), py_neg(&rh)])
            .into_iter()
            .next()
            .expect("two items");
        if expr1 != l {
            return reversedsign(&reversed(&r));
        }
    }
    r
}

/// `getattr(x, 'could_extract_minus_sign')()`, which every Basic has.
fn could_extract_minus_sign(x: &Expr) -> bool {
    match x.method_owner(Method::CouldExtractMinusSign) {
        "Basic" | "Expr" => false,
        "Number" | "Add" | "Mul" => x.could_extract_minus_sign(),
        o => not_ported(format!("{o}.could_extract_minus_sign")),
    }
}

/// `self.binary_symbols`, as a list without repeats (SymPy returns a set).
pub fn binary_symbols(e: &Expr) -> Vec<Expr> {
    match e.method_owner(Method::BinarySymbols) {
        "Symbol" => vec![e.clone()],
        "Relational" => vec![],
        "Equality" | "Unequality" => {
            let args = e.args();
            if args.iter().any(|a| a.is_cls(C::BooleanTrue) || a.is_cls(C::BooleanFalse)) {
                if args[0].is_Symbol() {
                    return vec![args[0].clone()];
                } else if args[1].is_Symbol() {
                    return vec![args[1].clone()];
                }
            }
            vec![]
        }
        // Boolean.binary_symbols
        "Boolean" => {
            let mut out: Vec<Expr> = Vec::new();
            for i in e.args() {
                if i.is_Boolean()
                    || i.is_Symbol()
                    || matches!(i.cls(), C::Equality | C::Unequality)
                {
                    for x in binary_symbols(i) {
                        if !out.contains(&x) {
                            out.push(x);
                        }
                    }
                }
            }
            out
        }
        "" => raise(
            ErrorKind::AttributeError,
            format!(
                "'{}' object has no attribute 'binary_symbols'",
                e.class_name()
            ),
        ),
        o => not_ported(format!("{o}.binary_symbols")),
    }
}

/// `self.equals(other)`.
pub fn equals(e: &Expr, other: &Expr) -> Option<bool> {
    let _ = (e, other);
    not_ported("Relational.equals (needs Expr.equals)")
}

/// `self._eval_simplify()` / `simplify(self)`.
pub fn simplify(e: &Expr) -> Expr {
    let _ = e;
    not_ported("Relational._eval_simplify (needs simplify)")
}

// ===========================================================================
// The fuzzy comparisons
// ===========================================================================

/// What SymPy's fuzzy comparisons return: Python's `True`, `False` or
/// `None`, or a SymPy object where SymPy hands one back (`n2 >= 0` in
/// `is_ge` and `_sympify(n2 == 0)` in `is_eq` give `S.true` or `S.false`,
/// `Interval._eval_Eq` gives `S.false`, the `(Interval, Interval)`
/// handler an `And`). Code that tests `is True` sees the difference.
#[derive(Clone, Debug, PartialEq)]
pub enum Fuzzy {
    /// `True`, `False` or `None`.
    Py(Option<bool>),
    /// A SymPy object.
    Sym(Expr),
}

impl Fuzzy {
    /// `x is None`.
    pub fn is_none(&self) -> bool {
        matches!(self, Fuzzy::Py(None))
    }

    /// `fuzzy_bool(x)`: `None` for None and for anything not equal to
    /// `True` or `False`, else `bool(x)`.
    pub fn fuzzy_bool(&self) -> Option<bool> {
        match self {
            Fuzzy::Py(v) => *v,
            Fuzzy::Sym(e) => {
                if crate::logic::boolalg::in_true_false(e) {
                    Some(e.truthy())
                } else {
                    None
                }
            }
        }
    }

    /// `fuzzy_not(x)`: `None` for None, else `not x` (Python's truth value
    /// of a SymPy object: `TypeError` for a Relational).
    pub fn fuzzy_not(&self) -> Option<bool> {
        match self {
            Fuzzy::Py(v) => v.map(|b| !b),
            Fuzzy::Sym(e) => Some(!e.truthy()),
        }
    }

    /// `_sympify(x)` for a value that is not None.
    pub fn sympify(&self) -> Expr {
        match self {
            Fuzzy::Py(Some(b)) => boolean(*b),
            Fuzzy::Py(None) => unreachable!("_sympify(None) is not called"),
            Fuzzy::Sym(e) => e.clone(),
        }
    }

    /// The value for callers that want a fuzzy bool: `True`/`S.true`,
    /// `False`/`S.false` or `None`. A SymPy object that is neither (an
    /// unevaluated Boolean from a dispatch handler) has no such reading.
    pub fn known(&self) -> Option<bool> {
        match self {
            Fuzzy::Py(v) => *v,
            Fuzzy::Sym(e) if e.is_cls(C::BooleanTrue) => Some(true),
            Fuzzy::Sym(e) if e.is_cls(C::BooleanFalse) => Some(false),
            Fuzzy::Sym(e) => not_ported(format!(
                "a fuzzy comparison returning {} where a bool is expected",
                e.class_name()
            )),
        }
    }
}

const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

/// The label of the implementation a generated dispatch table picks.
fn dispatch(table: &[&str], impls: &'static [&'static str], a: &Expr, b: &Expr) -> &'static str {
    let c = table[a.cls() as usize].as_bytes()[b.cls() as usize];
    match DIGITS.iter().position(|&d| d == c) {
        Some(i) => impls[i],
        None => raise(
            ErrorKind::NotImplementedError,
            format!(
                "Could not find signature for ({}, {})",
                a.class_name(),
                b.class_name()
            ),
        ),
    }
}

/// `_eval_is_ge(lhs, rhs)` (multipledispatch).
fn eval_is_ge(lhs: &Expr, rhs: &Expr) -> Fuzzy {
    match dispatch(&EVAL_IS_GE, EVAL_IS_GE_IMPLS, lhs, rhs) {
        "relational(Expr, Expr)" => Fuzzy::Py(None),
        other => not_ported(format!("_eval_is_ge {other}")),
    }
}

/// `_eval_is_eq(lhs, rhs)` (multipledispatch).
fn eval_is_eq(lhs: &Expr, rhs: &Expr) -> Fuzzy {
    match dispatch(&EVAL_IS_EQ, EVAL_IS_EQ_IMPLS, lhs, rhs) {
        "relational(Basic, Basic)"
        | "relational(Tuple, AppliedUndef)"
        | "relational(Tuple, Symbol)" => Fuzzy::Py(None),
        "relational(Tuple, Expr)"
        | "numbers(NaN, Expr)"
        | "numbers(Tuple, Number)"
        | "sets(Interval, FiniteSet)"
        | "sets(FiniteSet, Interval)"
        | "sets(Set, Basic)" => Fuzzy::Py(Some(false)),
        // And(Eq(lhs.left, rhs.left), Eq(lhs.right, rhs.right),
        //     lhs.left_open == rhs.left_open,
        //     lhs.right_open == rhs.right_open)
        "sets(Interval, Interval)" => Fuzzy::Sym(crate::logic::boolalg::and(vec![
            equality(lhs.arg(0), rhs.arg(0)),
            equality(lhs.arg(1), rhs.arg(1)),
            boolean(lhs.arg(2) == rhs.arg(2)),
            boolean(lhs.arg(3) == rhs.arg(3)),
        ])),
        "relational(Tuple, Tuple)" => {
            if lhs.args().len() != rhs.args().len() {
                return Fuzzy::Py(Some(false));
            }
            // fuzzy_and stops at the first False, so later pairs are not
            // compared
            Fuzzy::Py(fuzzy_and(
                lhs.args()
                    .iter()
                    .zip(rhs.args())
                    .map(|(s, o)| is_eq_exact(s, o).fuzzy_bool()),
            ))
        }
        other => not_ported(format!("_eval_is_eq {other}")),
    }
}

/// `side._eval_Eq(other)` for the classes that have one.
fn eval_eq_method(side: &Expr, other: &Expr) -> Fuzzy {
    match side.method_owner(Method::Eval_Eq) {
        "" => Fuzzy::Py(None),
        // Interval._eval_Eq
        "Interval" => {
            if !other.is_cls(C::Interval) {
                if other.is_cls(C::FiniteSet) {
                    return Fuzzy::Sym(s::false_());
                } else if other.is_a(Tag::Set) {
                    return Fuzzy::Py(None);
                }
                return Fuzzy::Sym(s::false_());
            }
            Fuzzy::Py(None)
        }
        o => not_ported(format!("{o}._eval_Eq")),
    }
}

/// `is_lt(lhs, rhs)`.
pub fn is_lt(lhs: &Expr, rhs: &Expr) -> Option<bool> {
    is_ge_exact(lhs, rhs).fuzzy_not()
}

/// `is_gt(lhs, rhs)`.
pub fn is_gt(lhs: &Expr, rhs: &Expr) -> Option<bool> {
    is_le_exact(lhs, rhs).fuzzy_not()
}

/// `is_le(lhs, rhs)`, read as a fuzzy bool (see [`Fuzzy::known`]).
pub fn is_le(lhs: &Expr, rhs: &Expr) -> Option<bool> {
    is_le_exact(lhs, rhs).known()
}

/// `is_le(lhs, rhs)` exactly as SymPy returns it.
pub fn is_le_exact(lhs: &Expr, rhs: &Expr) -> Fuzzy {
    is_ge_exact(rhs, lhs)
}

/// `is_ge(lhs, rhs)`, read as a fuzzy bool (see [`Fuzzy::known`]).
pub fn is_ge(lhs: &Expr, rhs: &Expr) -> Option<bool> {
    is_ge_exact(lhs, rhs).known()
}

/// `is_ge(lhs, rhs)` (no assumptions) exactly as SymPy returns it.
pub fn is_ge_exact(lhs: &Expr, rhs: &Expr) -> Fuzzy {
    if !(lhs.is_a(Tag::Expr) && rhs.is_a(Tag::Expr)) {
        raise(
            ErrorKind::TypeError,
            "Can only compare inequalities with Expr",
        );
    }
    let retval = eval_is_ge(lhs, rhs);
    if !retval.is_none() {
        return retval;
    }
    if let Some(n2) = n2(lhs, rhs) {
        // use float comparison for infinity.
        // otherwise get stuck in infinite recursion
        return match n2.cls() {
            C::Infinity => Fuzzy::Py(Some(true)),
            C::NegativeInfinity => Fuzzy::Py(Some(false)),
            // `n2 >= 0` on S.Zero or a Float: S.true or S.false
            _ => Fuzzy::Sym(ge(&n2, &s::zero())),
        };
    }
    if lhs.is_extended_real() == Some(true) && rhs.is_extended_real() == Some(true) {
        if (lhs.is_infinite() == Some(true) && lhs.is_extended_positive() == Some(true))
            || (rhs.is_infinite() == Some(true) && rhs.is_extended_negative() == Some(true))
        {
            return Fuzzy::Py(Some(true));
        }
        let diff = ops::sub(lhs, rhs);
        if !diff.is_cls(C::NaN) {
            let rv = diff.is_extended_nonnegative();
            if rv.is_some() {
                return Fuzzy::Py(rv);
            }
        }
    }
    Fuzzy::Py(None)
}

/// `is_neq(lhs, rhs)`.
pub fn is_neq(lhs: &Expr, rhs: &Expr) -> Option<bool> {
    is_eq_exact(lhs, rhs).fuzzy_not()
}

/// `is_eq(lhs, rhs)`, read as a fuzzy bool (see [`Fuzzy::known`]).
pub fn is_eq(lhs: &Expr, rhs: &Expr) -> Option<bool> {
    is_eq_exact(lhs, rhs).known()
}

/// `sift(Add.make_args(expr), real_imag)` in `is_eq`: the `'real'`,
/// `'imag'` and `None` groups.
fn split_real_imag(expr: &Expr) -> (Vec<Expr>, Vec<Expr>, Vec<Expr>) {
    let (mut re, mut im, mut none) = (Vec::new(), Vec::new(), Vec::new());
    for t in expr.add_args() {
        if t.is_extended_real() == Some(true) {
            re.push(t);
        } else if ops::mul(&s::imaginary_unit(), &t).is_extended_real() == Some(true) {
            im.push(t);
        } else {
            none.push(t);
        }
    }
    (re, im, none)
}

/// `is_eq(lhs, rhs)` (no assumptions) exactly as SymPy returns it.
pub fn is_eq_exact(lhs: &Expr, rhs: &Expr) -> Fuzzy {
    // here, _eval_Eq is only called for backwards compatibility
    for (side1, side2) in [(lhs, rhs), (rhs, lhs)] {
        let retval = eval_eq_method(side1, side2);
        if !retval.is_none() {
            return retval;
        }
    }
    let retval = eval_is_eq(lhs, rhs);
    if !retval.is_none() {
        return retval;
    }
    // `dispatch(type(lhs), type(rhs)) != dispatch(type(rhs), type(lhs))`
    // compares two new decorator closures, so it always holds.
    let retval = eval_is_eq(rhs, lhs);
    if !retval.is_none() {
        return retval;
    }
    let t = Fuzzy::Py(Some(true));
    let f = Fuzzy::Py(Some(false));
    // retval is still None, so go through the equality logic
    // If expressions have the same structure, they must be equal.
    if lhs == rhs {
        return t; // e.g. True == True
    } else if lhs.is_a(Tag::BooleanAtom) && rhs.is_a(Tag::BooleanAtom) {
        return f; // True != False
    } else if !(lhs.is_Symbol() || rhs.is_Symbol())
        && lhs.is_a(Tag::Boolean) != rhs.is_a(Tag::Boolean)
    {
        return f; // only Booleans can equal Booleans
    }
    if lhs.is_infinite() == Some(true) || rhs.is_infinite() == Some(true) {
        if fuzzy_xor([lhs.is_infinite(), rhs.is_infinite()]) == Some(true) {
            return f;
        }
        if fuzzy_xor([lhs.is_extended_real(), rhs.is_extended_real()]) == Some(true) {
            return f;
        }
        if fuzzy_and([lhs.is_extended_real(), rhs.is_extended_real()]) == Some(true) {
            return Fuzzy::Py(fuzzy_xor([
                lhs.is_extended_positive(),
                fuzzy_not(rhs.is_extended_positive()),
            ]));
        }
        // Try to split real/imaginary parts and equate them
        let (lre, lim, lnone) = split_real_imag(lhs);
        if lnone.is_empty() {
            let (rre, rim, rnone) = split_real_imag(rhs);
            if rnone.is_empty() {
                let i = s::imaginary_unit();
                let eq_real = is_eq_exact(&crate::add::add(lre), &crate::add::add(rre));
                let eq_imag = is_eq_exact(
                    &ops::mul(&i, &crate::add::add(lim)),
                    &ops::mul(&i, &crate::add::add(rim)),
                );
                return Fuzzy::Py(fuzzy_and([eq_real.fuzzy_bool(), eq_imag.fuzzy_bool()]));
            }
        }
        // Compare e.g. zoo with 1+I*oo by comparing args
        let arglhs = crate::functions::call(C::Arg, vec![lhs.clone()]);
        let argrhs = crate::functions::call(C::Arg, vec![rhs.clone()]);
        // Guard against Eq(nan, nan) -> False
        if !(arglhs.is_cls(C::NaN) && argrhs.is_cls(C::NaN)) {
            return Fuzzy::Py(is_eq_exact(&arglhs, &argrhs).fuzzy_bool());
        }
    }
    if lhs.is_a(Tag::Expr) && rhs.is_a(Tag::Expr) {
        // see if the difference evaluates
        let dif = ops::sub(lhs, rhs);
        let z = dif.is_zero();
        if let Some(z) = z {
            if !z && dif.is_commutative() == Some(true) {
                // issue 10728
                return f;
            }
            if z {
                return t;
            }
        }
        // is_zero cannot help decide integer/rational with Float
        let (c, rest) = dif.as_coeff_Add(false);
        if c.is_Float() {
            if numbers::int_valued(&c) {
                if rest.is_integer() == Some(false) {
                    return f;
                }
            } else if rest.is_rational() == Some(false) {
                return f;
            }
        }
        if let Some(n2) = n2(lhs, rhs) {
            // _sympify(n2 == 0)
            return Fuzzy::Sym(boolean(numbers::number_eq(&n2, &s::zero())));
        }
        // see if the ratio evaluates
        let (n, d) = dif.as_numer_denom();
        let mut rv = None;
        if n.is_zero() == Some(true) {
            rv = d.is_nonzero();
        } else if n.is_finite() == Some(true) {
            if d.is_infinite() == Some(true) {
                rv = Some(true);
            } else if n.is_zero() == Some(false) {
                rv = d.is_infinite();
                if rv.is_none() {
                    // if the condition that makes the denominator
                    // infinite does not make the original expression
                    // True then False can be returned
                    not_ported("is_eq: clear_coefficients(d, oo) and subs");
                }
            }
        } else if n.add_args().iter().any(|a| a.is_infinite() == Some(true)) {
            // (inf or nan)/x != 0
            rv = Some(false);
        }
        if rv.is_some() {
            return Fuzzy::Py(rv);
        }
    }
    Fuzzy::Py(None)
}

/// `_n2(a, b)`: `(a - b).evalf(2)` if `a` and `b` are comparable and the
/// result is too, else `None`.
fn n2(a: &Expr, b: &Expr) -> Option<Expr> {
    // /!\ it is very important (see issue 8245) not to
    // use a re-evaluated number in the calculation of dif
    if a.is_comparable() && b.is_comparable() {
        let dif = number_evalf_2(&ops::sub(a, b));
        if dif.is_comparable() {
            return Some(dif);
        }
    }
    None
}

/// `n.evalf(2)` for a Number `n`: `EvalfMixin.evalf` runs the Number's
/// `evalf_table` entry at `prec + 4` bits (`Infinity` has none and goes
/// through `_eval_evalf`), then `Float._new(re, max(min(prec, re_acc), 1))`.
fn number_evalf_2(n: &Expr) -> Expr {
    if !n.is_Number() {
        not_ported("_n2: evalf of a non-Number difference");
    }
    let prec = mlib::dps_to_prec(2);
    let wp = prec + 4;
    let re = match n.cls() {
        C::Zero => return s::zero(),
        C::NaN => return s::nan(),
        C::Infinity => return s::infinity(),
        C::NegativeInfinity => return s::negative_infinity(),
        C::Float => n.as_float().expect("a Float").0.clone(),
        C::Integer | C::One | C::NegativeOne => mlib::from_int(&n.p(), wp, mlib::ROUND_FAST),
        C::Rational | C::Half => {
            mlib::from_rational(&n.p(), &n.q(), wp, mlib::ROUND_FAST).expect("q > 0")
        }
        _ => unreachable!("{} is not a Number", n.class_name()),
    };
    // re_acc is the working precision, so p = min(prec, wp) = prec
    numbers::float_new(re, prec, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::symbol;

    #[test]
    fn numbers_compare() {
        let one = s::one();
        let two = Expr::integer(2);
        assert!(lt(&one, &two).is_cls(C::BooleanTrue));
        assert!(ge(&one, &two).is_cls(C::BooleanFalse));
        assert!(lt(&one, &s::infinity()).is_cls(C::BooleanTrue));
        assert!(lt(&s::negative_infinity(), &s::infinity()).is_cls(C::BooleanTrue));
    }

    #[test]
    fn symbols_stay_unevaluated() {
        let x = symbol("x");
        let r = lt(&x, &s::one());
        assert!(r.is_cls(C::StrictLessThan));
        assert!(negated(&r).is_cls(C::GreaterThan));
        assert!(reversed(&r).is_cls(C::StrictGreaterThan));
    }
}
