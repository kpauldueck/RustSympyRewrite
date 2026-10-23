//! `sympy.core.power`: `Pow`.
//!
//! Ported from `sympy/core/power.py`. The `_eval_is_*` handlers live in
//! [`handler`] at the bottom.

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
        assert!(!e.is_Integer());

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
    let _ = (e, fact);
    not_ported("Pow assumptions")
}
