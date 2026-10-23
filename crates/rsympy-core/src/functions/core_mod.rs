//! `sympy.core.mod`: `Mod`.
//!
//! `Mod.eval` is ported branch for branch. Its last stage extracts
//! `gcd(p, q)` with `sympy.polys`, which is not ported yet: every input
//! that gets that far raises `NotPortedYet` there. The code after the gcd
//! is ported too, ready for when it lands.

use rsympy_num::Int;

use crate::assume::{fuzzy_and, fuzzy_not};
use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::Expr;
use crate::facts::Fact;
use crate::functions::{Head, Hints, call, call_with, default_eval_is, head_of};
use crate::generated::{ClassId as C, Method, Tag};
use crate::numbers::{equal_valued, s};
use crate::ops;
use crate::relational::{is_ge, is_gt, is_le, is_lt};

/// The head of `Mod`.
pub struct ModHead;

impl Head for ModHead {
    fn class_id(&self) -> C {
        C::Mod
    }

    /// `Mod.eval(p, q)`.
    fn eval(&self, args: &[Expr]) -> Option<Expr> {
        let [p, q] = args else {
            // (Function._new_ has checked nargs)
            unreachable!("Mod takes 2 arguments")
        };
        eval(p, q)
    }

    fn eval_is(&self, e: &Expr, fact: Fact) -> Option<bool> {
        match e.handler_owner(fact) {
            Some("Mod._eval_is_integer") => eval_is_integer(e),
            Some("Mod._eval_is_nonnegative") => eval_is_nonnegative(e),
            Some("Mod._eval_is_nonpositive") => eval_is_nonpositive(e),
            _ => default_eval_is(e, fact),
        }
    }

    /// `Mod._eval_as_leading_term`: `self.rewrite(floor)._eval_as_leading_term(...)`.
    fn eval_as_leading_term(&self, e: &Expr, x: &Expr, logx: Option<&Expr>) -> Expr {
        let _ = (e, x, logx);
        not_ported("Mod._eval_as_leading_term (needs rewrite(floor) and series)")
    }

    /// `Mod._eval_nseries`: `self.rewrite(floor)._eval_nseries(...)`.
    fn eval_nseries(&self, e: &Expr, x: &Expr, n: i64, logx: Option<&Expr>) -> Expr {
        let _ = (e, x, n, logx);
        not_ported("Mod._eval_nseries (needs rewrite(floor) and series)")
    }
}

/// Python's `a == b` for two SymPy objects.
fn eq(a: &Expr, b: &Expr) -> bool {
    crate::numbers::py_eq(a, b)
}

/// `int(r)` for an expression: `None` where it raises `TypeError`
/// ("Cannot convert symbols to int", which `Mod.eval` catches).
pub(crate) fn py_int(r: &Expr) -> Option<Int> {
    // Integer.__int__, Rational.__int__, Float.__int__, NumberSymbol's
    if let Some(i) = crate::numbers::number_int(r) {
        return Some(i);
    }
    // Expr.__int__
    if !r.is_number() {
        // TypeError("Cannot convert symbols to int")
        return None;
    }
    not_ported(format!(
        "Expr.__int__ of a {} (needs Expr.round)",
        r.class_name()
    ))
}

/// `-2*q` and the like: Python's `int * Expr`.
fn int_times(n: i64, q: &Expr) -> Expr {
    ops::mul(&Expr::integer(n), q)
}

/// `bool((a < 0) == True)`: whether `a < 0` evaluates to `S.true`.
fn lt_zero_is_true(a: &Expr) -> bool {
    crate::relational::lt(a, &s::zero()).is_cls(C::BooleanTrue)
}

/// `getattr(p, '_eval_Mod')(q)` when `p` has the method, else `None`.
fn eval_mod_hook(p: &Expr, q: &Expr) -> Option<Expr> {
    match p.method_owner(Method::Eval_Mod) {
        "" => None,
        "Pow" => crate::power::eval_mod(p, q),
        _ if p.is_a(Tag::Application) => head_of(p).eval_mod(p, q),
        o => not_ported(format!("{o}._eval_Mod")),
    }
}

/// `number_eval(p, q)` in `Mod.eval`: try to return `p % q` if both are
/// numbers or `+/-p` is known to be less than or equal `q`.
fn number_eval(p: &Expr, q: &Expr) -> Option<Expr> {
    if q.is_zero() == Some(true) {
        raise(ErrorKind::ZeroDivisionError, "Modulo by zero");
    }
    if p.is_cls(C::NaN)
        || q.is_cls(C::NaN)
        || p.is_finite() == Some(false)
        || q.is_finite() == Some(false)
    {
        return Some(s::nan());
    }
    if p.is_cls(C::Zero)
        || eq(p, q)
        || eq(p, &ops::neg(q))
        || (p.is_integer() == Some(true) && eq(q, &s::one()))
    {
        return Some(s::zero());
    }

    if q.is_Number() {
        if p.is_Number() {
            return Some(ops::rem(p, q));
        }
        if eq(q, &Expr::integer(2)) {
            if p.is_even() == Some(true) {
                return Some(s::zero());
            } else if p.is_odd() == Some(true) {
                return Some(s::one());
            }
        }
    }

    if let Some(rv) = eval_mod_hook(p, q) {
        return Some(rv);
    }

    // by ratio
    let r = ops::div(p, q);
    if r.is_integer() == Some(true) {
        return Some(s::zero());
    }
    // try: d = int(r) except TypeError: pass
    if let Some(d) = py_int(&r) {
        // (isinstance(d, int) holds)
        let mut rv = ops::sub(p, &ops::mul(&Expr::integer(d), q));
        if lt_zero_is_true(&ops::mul(&rv, q)) {
            rv = ops::add(&rv, q);
        }
        return Some(rv);
    }

    // by difference
    // -2|q| < p < 2|q|
    type Cmp = fn(&Expr, &Expr) -> Option<bool>;
    let (comp1, comp2): (Cmp, Cmp) = if q.is_positive() == Some(true) {
        (is_le, is_lt)
    } else if q.is_negative() == Some(true) {
        (is_ge, is_gt)
    } else {
        return None;
    };
    let mut ls = int_times(-2, q);
    let r = ops::sub(p, q);
    for _ in 0..4 {
        if comp1(&ls, p) != Some(true) {
            return None;
        }
        if comp2(&r, &ls) == Some(true) {
            return Some(ops::sub(p, &ls));
        }
        ls = ops::add(&ls, q);
    }
    None
}

/// `isinstance(e, Mod)`.
fn is_mod(e: &Expr) -> bool {
    e.is_cls(C::Mod)
}

/// `Mod.eval(p, q)`.
pub fn eval(p: &Expr, q: &Expr) -> Option<Expr> {
    if let Some(rv) = number_eval(p, q) {
        return Some(rv);
    }
    let mut p = p.clone();
    let mut q = q.clone();

    // denest
    if is_mod(&p) {
        let qinner = p.arg(1).clone();
        if eq(&ops::rem(&qinner, &q), &s::zero()) {
            return Some(call(C::Mod, vec![p.arg(0).clone(), q]));
        } else if ops::mul(&qinner, &ops::sub(&q, &qinner)).is_nonnegative() == Some(true) {
            // |qinner| < |q| and have same sign
            return Some(p);
        }
    } else if is_mod(&ops::neg(&p)) {
        let qinner = ops::neg(&p).arg(1).clone();
        if eq(&ops::rem(&qinner, &q), &s::zero()) {
            return Some(call(C::Mod, vec![ops::neg(ops::neg(&p).arg(0)), q]));
        } else if ops::mul(&qinner, &ops::add(&q, &qinner)).is_nonpositive() == Some(true) {
            // |qinner| < |q| and have different sign
            return Some(p);
        }
    } else if p.is_Add() {
        // separating into modulus and non modulus
        let (mod_l, non_mod_l): (Vec<Expr>, Vec<Expr>) = p.args().iter().cloned().partition(is_mod);
        // if q same for all
        if !mod_l.is_empty() && mod_l.iter().all(|inner| eq(inner.arg(1), &q)) {
            let net = ops::add(
                &crate::add::add(non_mod_l),
                &crate::add::add(mod_l.iter().map(|i| i.arg(0).clone()).collect()),
            );
            return Some(call(C::Mod, vec![net, q]));
        }
    } else if p.is_Mul() {
        // separating into modulus and non modulus
        let (mod_l, mut non_mod_l): (Vec<Expr>, Vec<Expr>) =
            p.args().iter().cloned().partition(is_mod);

        if !mod_l.is_empty()
            && mod_l.iter().all(|inner| eq(inner.arg(1), &q))
            && p.args().iter().all(|t| t.is_integer() == Some(true))
            && q.is_integer() == Some(true)
        {
            // finding distributive term
            let non_mod_l: Vec<Expr> = non_mod_l
                .iter()
                .map(|x| call(C::Mod, vec![x.clone(), q.clone()]))
                .collect();
            let mut mod_ = Vec::new();
            let mut non_mod = Vec::new();
            for j in non_mod_l {
                if is_mod(&j) {
                    mod_.push(j.arg(0).clone());
                } else {
                    non_mod.push(j);
                }
            }
            let prod_mod = crate::mul::mul(mod_);
            let prod_non_mod = crate::mul::mul(non_mod);
            let prod_mod1 = crate::mul::mul(mod_l.iter().map(|i| i.arg(0).clone()).collect());
            let net = ops::mul(&prod_mod1, &prod_mod);
            return Some(ops::mul(&prod_non_mod, &call(C::Mod, vec![net, q])));
        }

        if q.is_Integer()
            && !q.is_cls(C::One)
            && p.args().iter().all(|t| t.is_integer() == Some(true))
        {
            // (this rebinds non_mod_l to all of p's factors, Mod ones
            // included, which the Mul below then repeats)
            non_mod_l = p
                .args()
                .iter()
                .map(|i| {
                    if i.is_Integer() {
                        ops::rem(i, &q)
                    } else {
                        i.clone()
                    }
                })
                .collect();
            if non_mod_l.iter().any(|iq| iq.is_cls(C::Zero)) {
                return Some(s::zero());
            }
        }

        p = crate::mul::mul(non_mod_l.into_iter().chain(mod_l).collect());
    }

    // XXX other possibilities?

    // extract gcd; any further simplification should be done by the user
    let mut g = match crate::error::catch_kind(ErrorKind::PolynomialError, || {
        let g = polys_gcd(&p, &q);
        let pq = if !equal_valued(&g, &s::one()) {
            Some(
                [&p, &q]
                    .map(|i| crate::exprtools::gcd_terms(&ops::div(i, &g), false, false, false)),
            )
        } else {
            None
        };
        (g, pq)
    }) {
        Some((g, pq)) => {
            if let Some([p1, q1]) = pq {
                p = p1;
                q = q1;
            }
            g
        }
        // issue 21373
        None => s::one(),
    };
    let (pwas, qwas) = (p.clone(), q.clone());

    // simplify terms
    // (x + y + 2) % x -> Mod(y + 2, x)
    if p.is_Add() {
        let mut args = Vec::with_capacity(p.args().len());
        for i in p.args() {
            let a = call(C::Mod, vec![i.clone(), q.clone()]);
            if count_mod(&a) > count_mod(i) {
                args.push(i.clone());
            } else {
                args.push(a);
            }
        }
        if args.len() != p.args().len() || args.iter().zip(p.args()).any(|(a, b)| !eq(a, b)) {
            p = crate::add::add(args);
        }
    } else {
        // handle coefficients if they are not Rational
        // since those are not handled by factor_terms
        // e.g. Mod(.6*x, .3*y) -> 0.3*Mod(2*x, y)
        let (cp, p1) = p.as_coeff_Mul(false);
        let (cq, q1) = q.as_coeff_Mul(false);
        p = p1;
        q = q1;
        let mut ok = false;
        if !cp.is_Rational() || !cq.is_Rational() {
            let r = ops::rem(&cp, &cq);
            if equal_valued(&r, &s::zero()) {
                g = ops::mul(&g, &cq);
                let Some(k) = py_int(&ops::div(&cp, &cq)) else {
                    raise(ErrorKind::TypeError, "Cannot convert to int")
                };
                p = ops::mul(&p, &Expr::integer(k));
                ok = true;
            }
        }
        if !ok {
            p = ops::mul(&cp, &p);
            q = ops::mul(&cq, &q);
        }
    }

    // simple -1 extraction
    if p.could_extract_minus_sign() && q.could_extract_minus_sign() {
        g = ops::neg(&g);
        p = ops::neg(&p);
        q = ops::neg(&q);
    }

    // check again to see if p and q can now be handled as numbers
    if let Some(rv) = number_eval(&p, &q) {
        return Some(ops::mul(&rv, &g));
    }

    // put 1.0 from G on inside
    if g.is_Float() && equal_valued(&g, &s::one()) {
        p = ops::mul(&p, &g);
        return Some(call_with(C::Mod, vec![p, q], Some(false)));
    } else if g.is_Mul() && g.arg(0).is_Float() && equal_valued(g.arg(0), &s::one()) {
        p = ops::mul(g.arg(0), &p);
        g = crate::construct::raw(C::Mul, g.args()[1..].to_vec());
    }
    let evaluate = !(eq(&p, &pwas) && eq(&q, &qwas));
    Some(ops::mul(&g, &call_with(C::Mod, vec![p, q], Some(evaluate))))
}

/// `sympy.polys.polytools.gcd(p, q)`.
fn polys_gcd(p: &Expr, q: &Expr) -> Expr {
    let _ = (p, q);
    not_ported("Mod.eval: gcd(p, q) (sympy.polys)")
}

/// `e.count(Mod)`: the number of `Mod` nodes in a preorder traversal.
fn count_mod(e: &Expr) -> usize {
    e.preorder().iter().filter(|s| is_mod(s)).count()
}

/// `Mod._eval_is_integer`.
fn eval_is_integer(e: &Expr) -> Option<bool> {
    let (p, q) = (e.arg(0), e.arg(1));
    if fuzzy_and([p.is_integer(), q.is_integer(), fuzzy_not(q.is_zero())]) == Some(true) {
        return Some(true);
    }
    None
}

/// `Mod._eval_is_nonnegative`.
fn eval_is_nonnegative(e: &Expr) -> Option<bool> {
    if e.arg(1).is_positive() == Some(true) {
        return Some(true);
    }
    None
}

/// `Mod._eval_is_nonpositive`.
fn eval_is_nonpositive(e: &Expr) -> Option<bool> {
    if e.arg(1).is_negative() == Some(true) {
        return Some(true);
    }
    None
}

/// `Mod._eval_rewrite_as_floor(a, b)`: `a - b*floor(a/b)`.
pub(crate) fn eval_rewrite_as_floor(e: &Expr, args: &[Expr], hints: &Hints) -> Option<Expr> {
    let _ = (e, hints);
    let [a, b] = args else {
        unreachable!("Mod has two args")
    };
    let floor = call(C::Floor, vec![ops::div(a, b)]);
    Some(ops::sub(a, &ops::mul(b, &floor)))
}
