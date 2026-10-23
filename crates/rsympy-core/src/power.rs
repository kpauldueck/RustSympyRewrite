//! `sympy.core.power`: `Pow`.
//!
//! Ported from `sympy/core/power.py`. The `_eval_is_*` handlers live in
//! [`handler`] at the bottom.

use crate::assume::{fuzzy_and, fuzzy_not, fuzzy_or};
use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::Expr;
use crate::facts::Fact;
use crate::generated::{ClassId as C, Method, Tag};
use crate::numbers::s;
use crate::ops;
use crate::relational;

/// `Pow(b, e)` under the current `global_parameters.evaluate`.
pub fn pow(b: &Expr, e: &Expr) -> Expr {
    pow_with(b, e, crate::ctx::evaluate())
}

/// `Pow(b, e, evaluate=evaluate)`: `Pow.__new__`.
pub fn pow_with(b: &Expr, e: &Expr, evaluate: bool) -> Expr {
    if b.is_a(Tag::Relational) || e.is_a(Tag::Relational) {
        raise(ErrorKind::TypeError, "Relational cannot be used in Pow");
    }
    let mut base = b.clone();
    let exp = e.clone();
    if evaluate {
        if exp.is_cls(C::ComplexInfinity) {
            return s::nan();
        }
        if exp.is_cls(C::Infinity) {
            if relational::is_gt(&base, &s::one()) == Some(true) {
                return s::infinity();
            }
            if relational::is_gt(&base, &s::negative_one()) == Some(true)
                && relational::is_lt(&base, &s::one()) == Some(true)
            {
                return s::zero();
            }
            if relational::is_lt(&base, &s::negative_one()) == Some(true) {
                if base.is_finite() == Some(true) {
                    return s::complex_infinity();
                }
                if base.is_finite() == Some(false) {
                    return s::nan();
                }
            }
        }
        if exp.is_cls(C::Zero) {
            return s::one();
        } else if exp.is_cls(C::One) {
            return base;
        } else if exp.is_cls(C::NegativeOne) && !base.truthy() {
            return s::complex_infinity();
        } else if exp.is_cls(C::AccumulationBounds) {
            not_ported("Pow with an AccumulationBounds exponent");
        } else if ((exp.is_Symbol() && exp.is_integer() == Some(true)) || exp.is_Integer())
            && ((base.is_number() && base.is_Mul()) || base.is_Number())
            && base.could_extract_minus_sign()
        {
            // autosimplification if base is a number and exp odd/even
            if exp.is_even() == Some(true) {
                base = ops::neg(&base);
            } else if exp.is_odd() == Some(true) {
                return ops::neg(&pow(&ops::neg(&base), &exp));
            }
        }
        if base.is_cls(C::NaN) || exp.is_cls(C::NaN) {
            // S.NaN**x -> S.NaN under assumption that x != 0
            return s::nan();
        } else if base.is_cls(C::One) {
            if ops::abs(&exp).is_infinite() == Some(true) {
                return s::nan();
            }
            return s::one();
        } else {
            // recognize base as E
            // (exp_polar is not modelled, so `not isinstance(base,
            // exp_polar)` always holds)
            if !exp.is_Atom()
                && !base.is_cls(C::Exp1)
                && let Some(r) = recognize_exp(&base, &exp)
            {
                return r;
            }
            if let Some(obj) = eval_power_of(&base, &exp) {
                return obj;
            }
        }
    }
    from_args(&base, &exp)
}

/// The `factor_terms`/`fraction` test in `Pow.__new__` that turns
/// `b**(c*num/log(b))` into `E**(c*num)`.
fn recognize_exp(base: &Expr, exp: &Expr) -> Option<Expr> {
    let (c, ex) =
        crate::exprtools::factor_terms(exp, false, false, false, false).as_coeff_Mul(false);
    let (num, den) = crate::exprtools::fraction(&ex, false);
    if den.is_cls(C::Log) && den.arg(0) == base {
        return Some(pow(&s::exp1(), &ops::mul(&c, &num)));
    } else if den.is_Add() {
        let im = crate::functions::call(C::Im, vec![base.clone()]);
        let sg = crate::functions::call(C::Sign, vec![im]);
        if sg.is_Number() && sg.truthy() {
            let neg_base = ops::neg(&crate::exprtools::factor_terms(
                base, false, false, false, false,
            ));
            let log = crate::functions::call(C::Log, vec![neg_base]);
            let rhs = ops::add(
                &log,
                &ops::mul(&ops::mul(&sg, &s::imaginary_unit()), &s::pi()),
            );
            if den == rhs {
                return Some(pow(&s::exp1(), &ops::mul(&c, &num)));
            }
        }
    }
    None
}

/// `base._eval_power(exp)`, dispatched on the class that defines it.
pub(crate) fn eval_power_of(base: &Expr, exp: &Expr) -> Option<Expr> {
    match base.method_owner(Method::EvalPower) {
        "Expr" => None,
        "" => raise(
            ErrorKind::AttributeError,
            format!(
                "'{}' object has no attribute '_eval_power'",
                base.class_name()
            ),
        ),
        "Integer" | "Zero" | "One" | "NegativeOne" | "Rational" | "Float" | "Infinity"
        | "NegativeInfinity" | "ComplexInfinity" | "ImaginaryUnit" | "Exp1" => {
            crate::numbers::eval_power(base, exp)
        }
        "Pow" => eval_power(base, exp),
        "Mul" => crate::mul::eval_power(base, exp),
        "Add" => crate::add::eval_power(base, exp),
        "BooleanAtom" => raise(
            ErrorKind::TypeError,
            "BooleanAtom not allowed in this context.",
        ),
        owner => crate::functions::eval_power(owner, base, exp),
    }
}

/// `Expr.__new__(Pow, b, e)` plus `obj.is_commutative = ...`: the node
/// `Pow.__new__` builds when nothing evaluates.
pub fn from_args(b: &Expr, e: &Expr) -> Expr {
    let c = match b.is_commutative() {
        Some(true) => e.is_commutative(),
        other => other,
    };
    crate::construct::node_with_commutative(C::Pow, vec![b.clone(), e.clone()], c)
}

/// `_half(e)` in `Pow._eval_power`: true if the exponent has a literal 2
/// as its denominator, else None.
fn half(e: &Expr) -> Option<bool> {
    if e.is_Rational() && e.q() == 2 {
        return Some(true);
    }
    let (n, d) = e.as_numer_denom();
    if n.is_integer() == Some(true) && d.is_cls(C::Integer) && d.p() == 2 {
        return Some(true);
    }
    None
}

/// `(a < b) == True` as `Pow._eval_power` writes it.
fn lt_is_true(a: &Expr, b: &Expr) -> bool {
    relational::lt(a, b).is_cls(C::BooleanTrue)
}

/// `Pow._eval_power(expt)` (a power of a power).
pub fn eval_power(p: &Expr, expt: &Expr) -> Option<Expr> {
    let (mut b, e) = p.as_base_exp();
    if b.is_cls(C::NaN) {
        // let __new__ handle it
        return Some(pow(&pow(&b, &e), expt));
    }
    let mut s: Option<Expr> = None;
    if expt.is_integer() == Some(true) {
        s = Some(s::one());
    } else if b.is_polar() == Some(true) {
        // e.g. exp_polar, besselj, var('p', polar=True)...
        s = Some(s::one());
    } else if e.is_extended_real().is_some() {
        if e.is_extended_real() == Some(true) {
            // we need _half(expt) with constant floor or
            // floor(S.Half - e*arg(b)/2/pi) == 0

            // handle -1 as special case
            if e.is_cls(C::NegativeOne) {
                // floor arg. is 1/2 + arg(b)/2/pi
                if half(expt) == Some(true) {
                    if b.is_negative() == Some(true) {
                        return Some(ops::mul(
                            &pow(&s::negative_one(), expt),
                            &pow(&ops::neg(&b), &ops::mul(&e, expt)),
                        ));
                    } else if b.is_negative() == Some(false) {
                        // XXX ok if im(b) != 0?
                        return Some(pow(&b, &ops::neg(expt)));
                    }
                }
            } else if e.is_even() == Some(true) {
                if b.is_extended_real() == Some(true) {
                    b = ops::abs(&b);
                }
                if b.is_imaginary() == Some(true) {
                    let im = crate::functions::call(C::Im, vec![b.clone()]);
                    b = ops::mul(&ops::abs(&im), &s::imaginary_unit());
                }
            }

            // SymPy's if/elif chain, each branch setting s = 1 (floor = 0)
            if (lt_is_true(&ops::abs(&e), &s::one()) || e.is_cls(C::One))
                || b.is_extended_nonnegative() == Some(true)
                || (crate::functions::call(C::Re, vec![b.clone()]).is_extended_nonnegative()
                    == Some(true)
                    && lt_is_true(&ops::abs(&e), &Expr::integer(2)))
            {
                s = Some(s::one());
            } else if half(expt) == Some(true) {
                not_ported("Pow._eval_power: exp(2*pi*I*expt*floor(...)) sign test (needs evalf)");
            }
        } else {
            // e.is_extended_real is False requires:
            //     _half(expt) with constant floor or
            //     floor(S.Half - im(e*log(b))/2/pi) == 0
            not_ported("Pow._eval_power with a non-real exponent (needs evalf)");
        }
    }
    s.map(|s| ops::mul(&s, &pow(&b, &ops::mul(&e, expt))))
}

/// `Pow.as_base_exp()`.
pub fn as_base_exp(e: &Expr) -> (Expr, Expr) {
    let (b, x) = (e.arg(0), e.arg(1));
    if b.is_Rational() && b.p().is_one() && !b.q().is_one() {
        return (Expr::integer(b.q()), ops::neg(x));
    }
    (b.clone(), x.clone())
}

/// `Pow.as_content_primitive(radical, clear)`.
pub fn as_content_primitive(p: &Expr, radical: bool, clear: bool) -> (Expr, Expr) {
    use crate::mul::keep_coeff;
    let (b, e) = p.as_base_exp();
    let (bc, bp) = b.as_content_primitive(radical, clear);
    let b = keep_coeff(&bc, &bp, true, false);
    let (ce, pe) = e.as_content_primitive(radical, clear);
    if b.is_Rational() {
        //e
        //= ce*pe
        //= ce*(h + t)
        //= ce*h + ce*t
        //=> self
        //= b**(ce*h)*b**(ce*t)
        //= b**(cehp/cehq)*b**(ce*t)
        //= b**(iceh + r/cehq)*b**(ce*t)
        //= b**(iceh)*b**(r/cehq)*b**(ce*t)
        //= b**(iceh)*b**(ce*t + r/cehq)
        let (h, t) = pe.as_coeff_Add(false);
        if h.is_Rational() && !b.is_cls(C::Zero) {
            let ceh = ops::mul(&ce, &h);
            let mut c = pow(&b, &ceh);
            let mut r = s::zero();
            if !c.is_Rational() {
                let (iceh, rr) = ceh.p().div_mod(&ceh.q()).expect("q != 0");
                c = pow(&b, &Expr::integer(iceh));
                r = Expr::integer(rr);
            }
            let rest = ops::div(&ops::div(&r, &ce), &Expr::integer(ceh.q()));
            return (
                c,
                pow(&b, &keep_coeff(&ce, &ops::add(&t, &rest), true, false)),
            );
        }
    }
    let e = keep_coeff(&ce, &pe, true, false);
    // b**e = (h*t)**e = h**e*t**e = c*m*t**e
    if e.is_Rational() && b.is_Mul() {
        let (h, t) = b.as_content_primitive(radical, clear); // h is positive
        let (c, m) = pow(&h, &e).as_coeff_Mul(false); // so c is positive
        let (m, me) = m.as_base_exp();
        if m.is_cls(C::One) || me == e {
            // probably always true
            // return the following, not return c, m*Pow(t, e)
            // which would change Pow into Mul; we let SymPy
            // decide what to do by using the unevaluated Mul, e.g
            // should it stay as sqrt(2 + 2*sqrt(5)) or become
            // sqrt(2)*sqrt(1 + sqrt(5))
            return (c, pow(&keep_coeff(&m, &t, true, false), &e));
        }
    }
    (s::one(), pow(&b, &e))
}

/// `Pow.as_numer_denom()`.
pub fn as_numer_denom(p: &Expr) -> (Expr, Expr) {
    if p.is_commutative() != Some(true) {
        return (p.clone(), s::one());
    }
    let (base, mut exp) = p.as_base_exp();
    let (mut n, mut d) = base.as_numer_denom();
    // this should be the same as ExpBase.as_numer_denom wrt
    // exponent handling
    let mut neg_exp = exp.is_negative();
    if exp.is_Mul() && neg_exp != Some(true) && exp.is_positive() != Some(true) {
        neg_exp = Some(exp.could_extract_minus_sign());
    }
    let int_exp = exp.is_integer();
    // the denominator cannot be separated from the numerator if
    // its sign is unknown unless the exponent is an integer, e.g.
    // sqrt(a/b) != sqrt(a)/sqrt(b) when a=1 and b=-1. But if the
    // denominator is negative the numerator and denominator can
    // be negated and the denominator (now positive) separated.
    if !(d.is_extended_real() == Some(true) || int_exp == Some(true)) {
        n = base.clone();
        d = s::one();
    }
    let dnonpos = d.is_nonpositive();
    if dnonpos == Some(true) {
        (n, d) = (ops::neg(&n), ops::neg(&d));
    } else if dnonpos.is_none() && int_exp != Some(true) {
        n = base.clone();
        d = s::one();
    }
    if neg_exp == Some(true) {
        (n, d) = (d, n);
        exp = ops::neg(&exp);
    }
    if exp.is_infinite() == Some(true) {
        if n.is_cls(C::One) && !d.is_cls(C::One) {
            return (n, pow(&d, &exp));
        }
        if !n.is_cls(C::One) && d.is_cls(C::One) {
            return (pow(&n, &exp), d);
        }
    }
    (pow(&n, &exp), pow(&d, &exp))
}

/// `Pow._eval_expand_power_base()` with default hints.
pub fn expand_power_base(e: &Expr) -> Expr {
    expand_power_base_with(e, false)
}

/// `Pow._eval_expand_power_base(force=force)`.
pub fn expand_power_base_with(p: &Expr, force: bool) -> Expr {
    let (b, e) = (p.arg(0), p.arg(1));
    if !b.is_Mul() {
        return p.clone();
    }
    let (cargs, mut nc) = b.args_cnc(false);

    // expand each term - this is top-level-only
    // expansion but we have to watch out for things
    // that don't have an _eval_expand method
    if !nc.is_empty() {
        nc = nc
            .iter()
            .map(|i| match i.method_owner(Method::EvalExpandPowerBase) {
                "Pow" => expand_power_base_with(i, force),
                "" => i.clone(),
                o => not_ported(format!("{o}._eval_expand_power_base")),
            })
            .collect();
        if e.is_Integer() {
            let n = e.p().as_i64().unwrap_or_else(|| {
                raise(
                    ErrorKind::OverflowError,
                    "cannot repeat a list that many times",
                )
            });
            let mut rv = if e.is_positive() == Some(true) {
                let mut v = Vec::new();
                for _ in 0..n {
                    v.extend(nc.iter().cloned());
                }
                crate::mul::mul(v)
            } else {
                let inv: Vec<Expr> = nc.iter().rev().map(|i| i.powi(-1)).collect();
                let mut v = Vec::new();
                for _ in 0..(-n) {
                    v.extend(inv.iter().cloned());
                }
                crate::mul::mul(v)
            };
            if !cargs.is_empty() {
                rv = ops::mul(&rv, &pow(&crate::mul::mul(cargs), e));
            }
            return rv;
        }
        if cargs.is_empty() {
            return pow_with(&crate::mul::mul(nc), e, false);
        }
        nc = vec![crate::mul::mul(nc)];
    }

    // sift the commutative bases
    let (mut other, maybe_real): (Vec<Expr>, Vec<Expr>) = cargs
        .into_iter()
        .partition(|x| x.is_extended_real() == Some(false));
    // pred(x): I, True, None or False
    let mut nonneg = Vec::new();
    let mut none = Vec::new();
    let mut neg = Vec::new();
    let mut imag = Vec::new();
    for x in maybe_real {
        if x.is_cls(C::ImaginaryUnit) {
            imag.push(x);
            continue;
        }
        let polar = x.is_polar();
        let key = if polar == Some(true) {
            Some(true)
        } else if polar.is_none() {
            x.is_extended_nonnegative()
        } else {
            None
        };
        match key {
            Some(true) => nonneg.push(x),
            None => none.push(x),
            Some(false) => neg.push(x),
        }
    }
    other.extend(none);
    if !imag.is_empty() {
        let i = imag.len() % 4;
        if i == 1 {
            other.push(s::imaginary_unit());
        } else if i == 2 || i == 3 {
            if let Some(last) = neg.pop() {
                let nonn = ops::neg(&last);
                if !nonn.is_cls(C::One) {
                    nonneg.push(nonn);
                }
            } else {
                neg.push(s::negative_one());
            }
            if i == 3 {
                other.push(s::imaginary_unit());
            }
        }
    }

    // bring out the bases that can be separated from the base
    let cargs;
    if force || e.is_integer() == Some(true) {
        // treat all commutatives the same and put nc in other
        let mut v = nonneg;
        v.extend(neg);
        v.extend(other);
        cargs = v;
        other = nc;
    } else {
        // this is just like what is happening automatically, except
        // that now we are doing it for an arbitrary exponent for which
        // no automatic expansion is done
        if e.is_Integer() {
            raise(ErrorKind::AssertionError, "assert not e.is_Integer");
        }

        // handle negatives by making them all positive and putting
        // the residual -1 in other
        if neg.len() > 1 {
            let mut o = s::one();
            if other.is_empty() && neg[0].is_Number() {
                o = ops::mul(&o, &neg.remove(0));
            }
            if neg.len() % 2 == 1 {
                o = ops::neg(&o);
            }
            for n in &neg {
                nonneg.push(ops::neg(n));
            }
            if !o.is_cls(C::One) {
                other.push(o);
            }
        } else if !neg.is_empty() && !other.is_empty() {
            if neg[0].is_Number() && !neg[0].is_cls(C::NegativeOne) {
                other.push(s::negative_one());
                nonneg.push(ops::neg(&neg[0]));
            } else {
                other.extend(neg);
            }
        } else {
            other.extend(neg);
        }
        cargs = nonneg;
        other.extend(nc);
    }

    let mut rv = s::one();
    if !cargs.is_empty() {
        let mut rest = cargs;
        if e.is_Rational() {
            let (npow, keep): (Vec<Expr>, Vec<Expr>) = rest
                .into_iter()
                .partition(|x| x.is_Pow() && x.arg(1).is_Rational() && x.arg(0).is_number());
            rest = keep;
            rv = crate::mul::mul(
                npow.iter()
                    .map(|b| pow(&b.func(b.args().to_vec()), e))
                    .collect(),
            );
        }
        rv = ops::mul(
            &rv,
            &crate::mul::mul(rest.iter().map(|b| pow_with(b, e, false)).collect()),
        );
    }
    if !other.is_empty() {
        rv = ops::mul(&rv, &pow_with(&crate::mul::mul(other), e, false));
    }
    rv
}

/// `type(e)._prop_handler[fact](e)` for a Pow.
pub(crate) fn handler(e: &Expr, fact: Fact) -> Option<bool> {
    let (b, x) = (e.arg(0), e.arg(1));
    match fact {
        Fact::Even => {
            if x.is_integer() == Some(true) && x.is_positive() == Some(true) {
                return b.is_even();
            }
            None
        }
        Fact::Negative => {
            let ext_neg = eval_is_extended_negative(b, x);
            if ext_neg == Some(true) {
                return e.is_finite();
            }
            ext_neg
        }
        Fact::ExtendedPositive => eval_is_extended_positive(b, x),
        Fact::ExtendedNegative => eval_is_extended_negative(b, x),
        Fact::Zero => eval_is_zero(b, x),
        Fact::Integer => eval_is_integer(e, b, x),
        Fact::ExtendedReal => eval_is_extended_real(b, x),
        Fact::Complex => {
            // XXX: is_complex should be is_finite_complex
            if b == &s::exp1() {
                return fuzzy_or([x.is_complex(), x.is_extended_negative()]);
            }
            if e.args().iter().all(|a| a.is_complex() == Some(true))
                && eval_is_finite(b, x) == Some(true)
            {
                return Some(true);
            }
            None
        }
        Fact::Imaginary => eval_is_imaginary(b, x),
        Fact::Odd => {
            if x.is_integer() == Some(true) {
                if x.is_positive() == Some(true) {
                    return b.is_odd();
                } else if x.is_nonnegative() == Some(true) && b.is_odd() == Some(true) {
                    return Some(true);
                } else if b.is_cls(C::NegativeOne) {
                    return Some(true);
                }
            }
            None
        }
        Fact::Finite => eval_is_finite(b, x),
        Fact::Prime => {
            // An integer raised to the n(>=2)-th power cannot be a prime.
            if b.is_integer() == Some(true)
                && x.is_integer() == Some(true)
                && ops::sub(x, &s::one()).is_positive() == Some(true)
            {
                return Some(false);
            }
            None
        }
        Fact::Composite => {
            // A power is composite if both base and exponent are greater
            // than 1
            if b.is_integer() == Some(true)
                && x.is_integer() == Some(true)
                && ((ops::sub(b, &s::one()).is_positive() == Some(true)
                    && ops::sub(x, &s::one()).is_positive() == Some(true))
                    || (ops::add(b, &s::one()).is_negative() == Some(true)
                        && x.is_positive() == Some(true)
                        && x.is_even() == Some(true)))
            {
                return Some(true);
            }
            None
        }
        Fact::Polar => b.is_polar(),
        Fact::Rational => eval_is_rational(e),
        Fact::Algebraic => eval_is_algebraic(e, b, x),
        _ => unreachable!("Pow has no handler for {fact:?}"),
    }
}

fn log(b: &Expr) -> Expr {
    crate::functions::call(C::Log, vec![b.clone()])
}

/// `Pow._eval_is_extended_positive`.
fn eval_is_extended_positive(b: &Expr, x: &Expr) -> Option<bool> {
    if b == x {
        if b.is_extended_nonnegative() == Some(true) {
            return Some(true);
        }
    } else if b.is_positive() == Some(true) {
        if x.is_real() == Some(true) {
            return Some(true);
        }
    } else if b.is_extended_negative() == Some(true) {
        if x.is_even() == Some(true) {
            return Some(true);
        }
        if x.is_odd() == Some(true) {
            return Some(false);
        }
    } else if b.is_zero() == Some(true) {
        if x.is_extended_real() == Some(true) {
            return x.is_zero();
        }
    } else if b.is_extended_nonpositive() == Some(true) {
        if x.is_odd() == Some(true) {
            return Some(false);
        }
    } else if b.is_imaginary() == Some(true) {
        if x.is_integer() == Some(true) {
            let m = ops::rem(x, &Expr::integer(4));
            if m.is_zero() == Some(true) {
                return Some(true);
            }
            if m.is_integer() == Some(true) && m.is_zero() == Some(false) {
                return Some(false);
            }
        }
        if x.is_imaginary() == Some(true) {
            return log(b).is_imaginary();
        }
    }
    None
}

/// `Pow._eval_is_extended_negative`.
fn eval_is_extended_negative(b: &Expr, x: &Expr) -> Option<bool> {
    if x.is_cls(C::Half) && (b.is_complex() == Some(true) || b.is_extended_real() == Some(true)) {
        return Some(false);
    }
    if b.is_extended_negative() == Some(true) {
        if x.is_odd() == Some(true) && b.is_finite() == Some(true) {
            return Some(true);
        }
        if x.is_even() == Some(true) {
            return Some(false);
        }
    } else if b.is_extended_positive() == Some(true) {
        if x.is_extended_real() == Some(true) {
            return Some(false);
        }
    } else if b.is_zero() == Some(true) {
        if x.is_extended_real() == Some(true) {
            return Some(false);
        }
    } else if b.is_extended_nonnegative() == Some(true) {
        if x.is_extended_nonnegative() == Some(true) {
            return Some(false);
        }
    } else if b.is_extended_nonpositive() == Some(true) || b.is_extended_real() == Some(true) {
        // (two branches in SymPy, both `if exp.is_even: return False`)
        if x.is_even() == Some(true) {
            return Some(false);
        }
    }
    None
}

/// `Pow._eval_is_zero`.
fn eval_is_zero(b: &Expr, x: &Expr) -> Option<bool> {
    if b.is_zero() == Some(true) {
        if x.is_extended_positive() == Some(true) {
            return Some(true);
        } else if x.is_extended_nonpositive() == Some(true) {
            return Some(false);
        }
    } else if b == &s::exp1() {
        return Some(x.is_cls(C::NegativeInfinity));
    } else if b.is_zero() == Some(false) {
        if b.is_finite() == Some(true) && x.is_finite() == Some(true) {
            return Some(false);
        } else if x.is_negative() == Some(true) {
            return b.is_infinite();
        } else if x.is_nonnegative() == Some(true) {
            return Some(false);
        } else if x.is_infinite() == Some(true) && x.is_extended_real() == Some(true) {
            let one_minus_abs = ops::sub(&s::one(), &ops::abs(b));
            if one_minus_abs.is_extended_positive() == Some(true) {
                return x.is_extended_positive();
            } else if one_minus_abs.is_extended_negative() == Some(true) {
                return x.is_extended_negative();
            }
        }
    } else if b.is_finite() == Some(true) && x.is_negative() == Some(true) {
        // when self.base.is_zero is None
        return Some(false);
    }
    None
}

/// `Pow._eval_is_integer`.
fn eval_is_integer(p: &Expr, b: &Expr, e: &Expr) -> Option<bool> {
    if b.is_rational() == Some(true)
        && b.is_integer() == Some(false)
        && e.is_positive() == Some(true)
    {
        return Some(false); // rat**nonneg
    }
    if b.is_integer() == Some(true) && e.is_integer() == Some(true) {
        if b.is_cls(C::NegativeOne) {
            return Some(true);
        }
        if e.is_nonnegative() == Some(true) || e.is_positive() == Some(true) {
            return Some(true);
        }
    }
    if b.is_integer() == Some(true)
        && e.is_negative() == Some(true)
        && (e.is_finite() == Some(true) || e.is_integer() == Some(true))
        && fuzzy_not(ops::sub(b, &s::one()).is_zero()) == Some(true)
        && fuzzy_not(ops::add(b, &s::one()).is_zero()) == Some(true)
    {
        return Some(false);
    }
    if b.is_Number() && e.is_Number() {
        let check = p.func(p.args().to_vec());
        return Some(check.is_Integer());
    }
    if e.is_negative() == Some(true)
        && b.is_positive() == Some(true)
        && ops::sub(b, &s::one()).is_positive() == Some(true)
    {
        return Some(false);
    }
    if e.is_negative() == Some(true)
        && b.is_negative() == Some(true)
        && ops::add(b, &s::one()).is_negative() == Some(true)
    {
        return Some(false);
    }
    None
}

/// `Pow._eval_is_extended_real`.
fn eval_is_extended_real(b: &Expr, x: &Expr) -> Option<bool> {
    if b.is_cls(C::Exp1) {
        if x.is_extended_real() == Some(true) {
            return Some(true);
        } else if x.is_imaginary() == Some(true) {
            let f = ops::div(
                &ops::mul(&ops::mul(&Expr::integer(2), &s::imaginary_unit()), x),
                &s::pi(),
            );
            return f.is_even();
        }
    }

    let real_b = b.is_extended_real();
    let Some(real_b) = real_b else {
        if b.is_cls(C::Exp) && b.arg(0).is_imaginary() == Some(true) {
            return x.is_imaginary();
        }
        if b.is_cls(C::Pow) && b.arg(0).is_cls(C::Exp1) && b.arg(1).is_imaginary() == Some(true) {
            return x.is_imaginary();
        }
        return None;
    };
    let real_e = x.is_extended_real()?;
    if real_b && real_e {
        if b.is_extended_positive() == Some(true) {
            return Some(true);
        } else if b.is_extended_nonnegative() == Some(true)
            && x.is_extended_nonnegative() == Some(true)
        {
            return Some(true);
        } else if x.is_integer() == Some(true) && b.is_extended_nonzero() == Some(true) {
            return Some(true);
        } else if x.is_integer() == Some(true) && x.is_nonnegative() == Some(true) {
            return Some(true);
        } else if b.is_extended_negative() == Some(true) && x.is_Rational() {
            return Some(false);
        }
    }
    if real_e && x.is_extended_negative() == Some(true) && b.is_zero() == Some(false) {
        return pow(b, &ops::neg(x)).is_extended_real();
    }
    let im_b = b.is_imaginary();
    let im_e = x.is_imaginary();
    if im_b == Some(true) {
        if x.is_integer() == Some(true) {
            if x.is_even() == Some(true) {
                return Some(true);
            } else if x.is_odd() == Some(true) {
                return Some(false);
            }
        } else if im_e == Some(true) && log(b).is_imaginary() == Some(true) {
            return Some(true);
        } else if x.is_Add() {
            let (c, a) = x.as_coeff_Add(false);
            if c.truthy() && c.is_Integer() {
                return crate::mul::mul_with(vec![pow(b, &c), pow(b, &a)], false)
                    .is_extended_real();
            }
        } else if (b == &ops::neg(&s::imaginary_unit()) || b == &s::imaginary_unit())
            && ops::div(x, &Expr::integer(2)).is_integer() == Some(false)
        {
            return Some(false);
        }
    }
    if real_b && im_e == Some(true) {
        if b.is_cls(C::NegativeOne) {
            return Some(true);
        }
        not_ported("Pow._eval_is_extended_real: exp.coeff(I) (needs Expr.coeff)");
    }

    if !real_b && real_e {
        // we already know it's not imag
        if x.is_Rational() && x.p().is_one() {
            return Some(false);
        }
        let i = ops::div(
            &ops::mul(&crate::functions::call(C::Arg, vec![b.clone()]), x),
            &s::pi(),
        );
        if i.is_complex() == Some(true) {
            // finite
            return i.is_integer();
        }
    }
    None
}

/// `Pow._eval_is_imaginary`.
fn eval_is_imaginary(b: &Expr, x: &Expr) -> Option<bool> {
    if b.is_commutative() == Some(false) {
        return Some(false);
    }

    if b.is_imaginary() == Some(true) && x.is_integer() == Some(true) {
        return x.is_odd();
    }

    if b == &s::exp1() {
        let f = ops::div(
            &ops::mul(&Expr::integer(2), x),
            &ops::mul(&s::pi(), &s::imaginary_unit()),
        );
        // exp(pi*integer) = 1 or -1, so not imaginary
        if f.is_even() == Some(true) {
            return Some(false);
        }
        // exp(pi*integer + pi/2) = I or -I, so it is imaginary
        if f.is_odd() == Some(true) {
            return Some(true);
        }
        return None;
    }

    if x.is_imaginary() == Some(true) && log(b).is_imaginary().is_some() {
        return Some(false); // I**i -> real; (2*I)**i -> complex ==> not imaginary
    }

    if b.is_extended_real() == Some(true) && x.is_extended_real() == Some(true) {
        if b.is_positive() == Some(true) {
            return Some(false);
        } else {
            let rat = x.is_rational();
            if rat != Some(true) {
                return rat;
            }
            if x.is_integer() == Some(true) {
                return Some(false);
            } else {
                let half = ops::mul(&Expr::integer(2), x).is_integer();
                if half == Some(true) {
                    return b.is_negative();
                }
                return half;
            }
        }
    }

    if b.is_extended_real() == Some(false) {
        // we already know it's not imag
        let i = ops::div(
            &ops::mul(&crate::functions::call(C::Arg, vec![b.clone()]), x),
            &s::pi(),
        );
        let isodd = ops::mul(&Expr::integer(2), &i).is_odd();
        if isodd.is_some() {
            return isodd;
        }
    }
    None
}

/// `Pow._eval_is_finite`.
fn eval_is_finite(b: &Expr, x: &Expr) -> Option<bool> {
    if x.is_negative() == Some(true) {
        if b.is_zero() == Some(true) {
            return Some(false);
        }
        if b.is_infinite() == Some(true) || b.is_nonzero() == Some(true) {
            return Some(true);
        }
    }
    let c1 = b.is_finite()?;
    let c2 = x.is_finite()?;
    if c1 && c2 && (x.is_nonnegative() == Some(true) || fuzzy_not(b.is_zero()) == Some(true)) {
        return Some(true);
    }
    None
}

/// `Pow._eval_is_rational`.
fn eval_is_rational(e: &Expr) -> Option<bool> {
    let (b, x) = (e.arg(0), e.arg(1));
    // The evaluation of self.func below can be very expensive in the case
    // of integer**integer if the exponent is large.  We should try to exit
    // before that if possible:
    if x.is_integer() == Some(true)
        && b.is_rational() == Some(true)
        && fuzzy_not(fuzzy_and([x.is_negative(), b.is_zero()])) == Some(true)
    {
        return Some(true);
    }
    let (bb, ee) = e.as_base_exp();
    let p = pow(&bb, &ee); // in case it's unevaluated
    if !p.is_Pow() {
        return p.is_rational();
    }
    let (b, e) = p.as_base_exp();
    if e.is_Rational() && b.is_Rational() {
        // we didn't check that e is not an Integer
        // because Rational**Integer autosimplifies
        return Some(false);
    }
    if e.is_integer() == Some(true) {
        if b.is_rational() == Some(true) {
            if fuzzy_not(b.is_zero()) == Some(true) || e.is_nonnegative() == Some(true) {
                return Some(true);
            }
            if b == e {
                // always rational, even for 0**0
                return Some(true);
            }
        } else if b.is_irrational() == Some(true) {
            return e.is_zero();
        }
    }
    if b.is_cls(C::Exp1) && e.is_rational() == Some(true) && e.is_nonzero() == Some(true) {
        return Some(false);
    }
    None
}

/// `Pow._eval_is_algebraic`.
fn eval_is_algebraic(p: &Expr, b: &Expr, x: &Expr) -> Option<bool> {
    // _is_one(expr): (expr - 1).is_zero (False if the subtraction raises
    // ValueError)
    let is_one = |e: &Expr| -> Option<bool> {
        crate::error::catch_kind(ErrorKind::ValueError, || ops::sub(e, &s::one()).is_zero())
            .unwrap_or(Some(false))
    };

    if b.is_zero() == Some(true) || is_one(b) == Some(true) {
        return Some(true);
    } else if b.is_cls(C::Exp1) {
        let s = p.func(p.args().to_vec());
        if s.is_cls(C::Pow) {
            if x.is_nonzero() == Some(true) {
                if x.is_algebraic() == Some(true) {
                    return Some(false);
                } else if ops::div(x, &s::pi()).is_rational() == Some(true) {
                    return Some(false);
                } else if ops::div(x, &ops::mul(&s::imaginary_unit(), &s::pi())).is_rational()
                    == Some(true)
                {
                    return Some(true);
                }
            }
        } else {
            return s.is_algebraic();
        }
    } else if x.is_rational() == Some(true) {
        if b.is_algebraic() == Some(false) {
            return x.is_zero();
        }
        if b.is_zero() == Some(false) {
            if x.is_nonzero() == Some(true) {
                return b.is_algebraic();
            } else if b.is_algebraic() == Some(true) {
                return Some(true);
            }
        }
        if x.is_positive() == Some(true) {
            return b.is_algebraic();
        }
    } else if b.is_algebraic() == Some(true)
        && x.is_algebraic() == Some(true)
        && ((fuzzy_not(b.is_zero()) == Some(true) && fuzzy_not(is_one(b)) == Some(true))
            || b.is_integer() == Some(false)
            || b.is_irrational() == Some(true))
    {
        return x.is_rational();
    }
    None
}

/// `Pow._eval_Mod(q)`: `b**e mod q`, dispatched by `Mod`.
///
/// The totient reductions (for large exponents, and for a power in the
/// exponent) need `totient`, which is not ported yet.
pub(crate) fn eval_mod(p: &Expr, q: &Expr) -> Option<Expr> {
    use rsympy_num::Int;
    let (base, exp) = (p.arg(0), p.arg(1));

    if exp.is_integer() == Some(true) && exp.is_positive() == Some(true) {
        if q.is_integer() == Some(true) {
            // base % q == 0
            let r = ops::rem(base, q);
            if r.is_Number() && crate::numbers::number_eq(&r, &s::zero()) {
                return Some(s::zero());
            }
        }

        if base.is_Integer() && exp.is_Integer() && q.is_Integer() {
            let (b, e, m) = (base.p(), exp.p(), q.p());
            let mb = m.bit_length();
            if mb <= 80 && e >= Int::from(mb) && Int::from(e.bit_length()).pow(4) >= m {
                not_ported("Pow._eval_Mod: totient(m)");
            }
            let r = b
                .pow_mod(&e, &m)
                .unwrap_or_else(|_| raise(ErrorKind::ValueError, "pow() 3rd argument cannot be 0"));
            return Some(Expr::integer(r));
        }

        if base.is_Pow() && base.is_integer() == Some(true) && base.is_number() {
            let base = crate::functions::mod_(base, q);
            return Some(crate::functions::mod_(&pow_with(&base, exp, false), q));
        }

        if exp.is_Pow() && exp.is_integer() == Some(true) && exp.is_number() {
            // bit_length = int(q).bit_length()
            let Some(qi) = crate::functions::core_mod::py_int(q) else {
                raise(ErrorKind::TypeError, "Cannot convert symbols to int")
            };
            // XXX Mod-Pow actually attempts to do a hanging evaluation
            // if this dispatched function returns None.
            // May need some fixes in the dispatcher itself.
            if qi.bit_length() <= 80 {
                not_ported("Pow._eval_Mod: totient(q)");
            }
        }
    }
    None
}
