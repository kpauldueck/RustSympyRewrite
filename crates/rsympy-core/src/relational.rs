//! `sympy.core.relational`'s fuzzy comparisons: `is_ge`, `is_le`,
//! `is_gt`, `is_lt` (and later `is_eq`), which return `True`, `False` or
//! `None` without building a Relational.

use crate::assume::fuzzy_not;
use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::Expr;
use crate::generated::{ClassId as C, Method, Tag};
use crate::numbers::{RND, s};
use crate::ops;
use rsympy_mpf as mlib;

/// Python's `lhs < rhs` for two SymPy objects (`Expr.__lt__` and its
/// overrides in `core.numbers`): `S.true`, `S.false`, or the unevaluated
/// relational it builds (`StrictLessThan`, or `StrictGreaterThan` with the
/// sides swapped where SymPy reflects the comparison).
pub fn lt(lhs: &Expr, rhs: &Expr) -> Expr {
    for (e, side) in [(lhs, Method::Lt), (rhs, Method::Gt)] {
        if !e.is_a(Tag::Expr) {
            match e.method_owner(side) {
                "" | "BooleanAtom" => raise(
                    ErrorKind::TypeError,
                    format!("'<' not supported for {}", e.class_name()),
                ),
                o => not_ported(format!("{o}.__lt__/__gt__ with a non-Expr")),
            }
        }
    }
    match lhs.method_owner(Method::Lt) {
        // Infinity, NegativeInfinity and NaN alias Expr.__lt__
        "Expr" | "Infinity" | "NegativeInfinity" | "NaN" => strict_less_than(lhs, rhs),
        "Integer" | "Rational" => rational_lt(lhs, rhs),
        "Float" => float_lt(lhs, rhs),
        o => not_ported(format!("{o}.__lt__")),
    }
}

fn boolean(b: bool) -> Expr {
    if b { s::true_() } else { s::false_() }
}

/// `Rational.__lt__` (and `Integer.__lt__`) through `_Rrel`.
fn rational_lt(r: &Expr, other: &Expr) -> Expr {
    if other.is_Rational() {
        // Integer.__lt__ compares p directly; _Rrel cross-multiplies
        return boolean(r.p() * other.q() < other.p() * r.q());
    }
    if other.is_Number() {
        if other.is_Float() {
            // Float.__gt__(r)
            return boolean(float_rational_cmp(other, r) > 0);
        }
        if other.is_number() && other.is_extended_real() == Some(true) {
            let q = Expr::integer(r.q());
            return strict_less_than(&Expr::integer(r.p()), &ops::mul(&q, other));
        }
    }
    // (_Rrel's NumberSymbol test sits under `other.is_Number`, which is
    // false for NumberSymbols, so they take this path)
    strict_less_than(r, other)
}

/// `Float.__lt__` through `_Frel`.
fn float_lt(f: &Expr, other: &Expr) -> Expr {
    if other.is_a(Tag::NumberSymbol) {
        return strict_greater_than(other, f);
    }
    let (m, _) = f.as_float().unwrap();
    if other.is_Rational() {
        return boolean(float_rational_cmp(f, other) < 0);
    }
    if let Some((o, _)) = other.as_float() {
        return boolean(mlib::mpf_lt(m, o));
    }
    if other.is_comparable() && !other.is_cls(C::Infinity) && !other.is_cls(C::NegativeInfinity) {
        not_ported("Float.__lt__ against a comparable non-Number (needs evalf)");
    }
    strict_less_than(f, other)
}

/// The sign of `f - r` as `_Frel` computes it, `mpf_cmp(f*r.q, r.p)`, with
/// NaN comparing as neither smaller nor larger.
fn float_rational_cmp(f: &Expr, r: &Expr) -> i32 {
    let (m, _) = f.as_float().unwrap();
    let smpf = mlib::mpf_mul(m, &mlib::from_int(&r.q(), 0, RND), 0, RND);
    let ompf = mlib::from_int(&r.p(), 0, RND);
    if mlib::mpf_lt(&smpf, &ompf) {
        -1
    } else if mlib::mpf_gt(&smpf, &ompf) {
        1
    } else {
        0
    }
}

/// `StrictLessThan(lhs, rhs)` (`_Inequality.__new__`).
fn strict_less_than(lhs: &Expr, rhs: &Expr) -> Expr {
    inequality(C::StrictLessThan, lhs, rhs, is_lt)
}

/// `StrictGreaterThan(lhs, rhs)`.
fn strict_greater_than(lhs: &Expr, rhs: &Expr) -> Expr {
    inequality(C::StrictGreaterThan, lhs, rhs, is_gt)
}

fn inequality(cls: C, lhs: &Expr, rhs: &Expr, fuzzy: fn(&Expr, &Expr) -> Option<bool>) -> Expr {
    if crate::ctx::evaluate() {
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
        if let Some(v) = fuzzy(lhs, rhs) {
            return boolean(v);
        }
    }
    crate::construct::raw(cls, vec![lhs.clone(), rhs.clone()])
}

/// `is_lt(lhs, rhs)`.
pub fn is_lt(lhs: &Expr, rhs: &Expr) -> Option<bool> {
    fuzzy_not(is_ge(lhs, rhs))
}

/// `is_gt(lhs, rhs)`.
pub fn is_gt(lhs: &Expr, rhs: &Expr) -> Option<bool> {
    fuzzy_not(is_le(lhs, rhs))
}

/// `is_le(lhs, rhs)`.
pub fn is_le(lhs: &Expr, rhs: &Expr) -> Option<bool> {
    is_ge(rhs, lhs)
}

/// `is_ge(lhs, rhs)` with no assumptions.
pub fn is_ge(lhs: &Expr, rhs: &Expr) -> Option<bool> {
    if !(lhs.is_a(Tag::Expr) && rhs.is_a(Tag::Expr)) {
        raise(
            ErrorKind::TypeError,
            "Can only compare inequalities with Expr",
        );
    }
    // _eval_is_ge(lhs, rhs): only Idx and AccumulationBounds register
    // anything besides the (Expr, Expr) default of None.
    for e in [lhs, rhs] {
        if e.is_cls(C::Idx) || e.is_cls(C::AccumulationBounds) {
            not_ported(format!("_eval_is_ge for {}", e.class_name()));
        }
    }
    if let Some(n2) = n2(lhs, rhs) {
        // n2 >= 0, with oo and -oo compared as floats
        return Some(match n2.cls() {
            C::Infinity => true,
            C::NegativeInfinity => false,
            _ => number_sign(&n2) >= 0,
        });
    }
    if lhs.is_extended_real() == Some(true) && rhs.is_extended_real() == Some(true) {
        if (lhs.is_infinite() == Some(true) && lhs.is_extended_positive() == Some(true))
            || (rhs.is_infinite() == Some(true) && rhs.is_extended_negative() == Some(true))
        {
            return Some(true);
        }
        let diff = ops::sub(lhs, rhs);
        if !diff.is_cls(C::NaN) {
            let rv = diff.is_extended_nonnegative();
            if rv.is_some() {
                return rv;
            }
        }
    }
    None
}

/// `_n2(a, b)`: `(a - b).evalf(2)` when both sides and the result are
/// comparable.
fn n2(a: &Expr, b: &Expr) -> Option<Expr> {
    if !(a.is_comparable() && b.is_comparable()) {
        return None;
    }
    let dif = ops::sub(a, b);
    if !dif.is_Number() {
        not_ported("_n2: evalf of a non-Number difference");
    }
    // A Number's evalf(2) is a Float of the same sign (or the same
    // singleton), so only its comparability and sign matter here.
    if dif.is_cls(C::NaN) {
        return None;
    }
    Some(dif)
}

/// The sign of a finite real Number.
fn number_sign(n: &Expr) -> i32 {
    if let Some(r) = n.as_rational() {
        return r.signum();
    }
    if let Some((m, _)) = n.as_float() {
        return rsympy_mpf::mpf_sign(m);
    }
    if n.is_cls(C::Zero) {
        return 0;
    }
    not_ported(format!("sign of {}", n.class_name()))
}
