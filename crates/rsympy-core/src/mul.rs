//! `sympy.core.mul`: `Mul`.
//!
//! Ported from `sympy/core/mul.py`; the constructor is `AssocOp.__new__`
//! (in `add.rs`) with [`flatten`]. The `_eval_is_*` handlers live in
//! [`handler`] at the bottom.

use std::collections::HashSet;

use rsympy_num::Int;

use crate::add::{Flattened, assoc_new};
use crate::assume::{fuzzy_group, fuzzy_not};
use crate::compare::sort_args;
use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::Expr;
use crate::facts::Fact;
use crate::generated::ClassId as C;
use crate::numbers::s;
use crate::ops;
use crate::power::{pow, pow_with};
use crate::py::PyDict;

/// `Mul(*args)` under the current `global_parameters.evaluate`.
pub fn mul(args: Vec<Expr>) -> Expr {
    mul_with(args, crate::ctx::evaluate())
}

/// `Mul(*args, evaluate=evaluate)`: `AssocOp.__new__` with `Mul.flatten`.
pub fn mul_with(args: Vec<Expr>, evaluate: bool) -> Expr {
    assoc_new(C::Mul, args, evaluate, flatten)
}

/// An entry of `Mul.flatten`'s work list: a factor, or `NC_Marker` ("there
/// are still non-commutative objects -- don't forget to process them").
#[derive(Clone)]
enum Item {
    E(Expr),
    NcMarker,
}

/// `Mul.make_args(e)`.
fn make_args(e: &Expr) -> Vec<Expr> {
    if e.is_Mul() {
        e.args().to_vec()
    } else {
        vec![e.clone()]
    }
}

/// `_gather(c_powers)` in `Mul.flatten`: gather exponents of common bases,
/// `x**(2*y) * x**(3*y) -> x**(5*y)`, matching terms by `as_coeff_Mul`.
fn gather(c_powers: Vec<(Expr, Expr)>) -> Vec<(Expr, Expr)> {
    let mut common_b: PyDict<Expr, PyDict<Expr, Vec<Expr>>> = PyDict::new();
    for (b, e) in c_powers {
        let (c, t) = e.as_coeff_Mul(false);
        common_b
            .setdefault_with(b, PyDict::new)
            .setdefault_with(t, Vec::new)
            .push(c);
    }
    let summed: Vec<(Expr, Vec<(Expr, Expr)>)> = common_b
        .into_iter()
        .map(|(b, d)| {
            let d = d
                .into_iter()
                .map(|(di, li)| (di, crate::add::add(li)))
                .collect();
            (b, d)
        })
        .collect();
    let mut new_c_powers = Vec::new();
    for (b, e) in summed {
        for (t, c) in e {
            new_c_powers.push((b.clone(), ops::mul(&c, &t)));
        }
    }
    new_c_powers
}

/// `divmod(p, q)` on Python ints, `q > 0`.
fn divmod(p: &Int, q: &Int) -> (Int, Int) {
    p.div_mod(q).expect("q != 0")
}

/// `Rational.gcd(other)` (`Number.gcd` for anything else goes to polys).
fn number_gcd(a: &Expr, b: &Expr) -> Expr {
    if !(a.is_Rational() && b.is_Rational()) {
        not_ported("Number.gcd (polys gcd) of a non-Rational");
    }
    if b.is_cls(C::Zero) {
        return b.clone();
    }
    Expr::rational(a.p().gcd(&b.p()), a.q().lcm(&b.q()))
}

/// `Mul.flatten(seq)`.
pub(crate) fn flatten(mut seq: Vec<Expr>) -> Flattened {
    if seq.len() == 2 {
        let (mut a, mut b) = (seq[0].clone(), seq[1].clone());
        if b.is_Rational() {
            std::mem::swap(&mut a, &mut b);
            seq = vec![a.clone(), b.clone()];
        }
        if a.is_cls(C::One) {
            raise(ErrorKind::AssertionError, "assert a is not S.One");
        }
        if a.is_Rational() && a.is_zero() != Some(true) {
            let (r, b) = b.as_coeff_Mul(false);
            if b.is_Add() {
                if !r.is_cls(C::One) {
                    // 2-arg hack
                    // leave the Mul as a Mul?
                    let ar = ops::mul(&a, &r);
                    let arb = if ar.is_cls(C::One) {
                        b
                    } else {
                        mul_with(vec![ar, b], false)
                    };
                    return (vec![arb], vec![]);
                } else if crate::ctx::distribute() && b.is_commutative() == Some(true) {
                    let newb = crate::add::add(
                        b.args()
                            .iter()
                            .map(|bi| keep_coeff(&a, bi, true, false))
                            .collect(),
                    );
                    return (vec![newb], vec![]);
                }
            }
        }
    }

    // apply associativity, separate commutative part of seq
    let mut c_part: Vec<Expr> = Vec::new(); // out: commutative factors
    let mut nc_part: Vec<Expr> = Vec::new(); // out: non-commutative factors
    let mut nc_seq: Vec<Expr> = Vec::new();
    let mut coeff = s::one(); // standalone term
    // e.g. 3 * ...
    let mut c_powers: Vec<(Expr, Expr)> = Vec::new(); // (base,exp)      n
    // e.g. (x,n) for x
    let mut num_exp: Vec<(Expr, Expr)> = Vec::new(); // (num-base, exp)           y
    // e.g.  (3, y)  for  ... * 3  * ...
    let mut neg1e = s::zero(); // exponent on -1 extracted from Number-based Pow and I
    let mut pnum_rat: PyDict<Expr, Vec<Expr>> = PyDict::new(); // (num-base, Rat-exp)          1/2
    // e.g.  (3, 1/2)  for  ... * 3     * ...
    let nan = || (vec![s::nan()], vec![]);

    let mut seq: Vec<Item> = seq.into_iter().map(Item::E).collect();
    let mut i = 0;
    // NOTE: this is optimized for all-objects-are-commutative case
    while i < seq.len() {
        let item = seq[i].clone();
        i += 1;
        let o = match item {
            Item::E(o) => Some(o),
            Item::NcMarker => None,
        };
        if let Some(o) = &o {
            if o.is_Order() {
                // O(x)
                not_ported("Mul.flatten with an Order factor");
            }
            if o.is_Mul() {
                // Mul([...])
                if o.is_commutative() == Some(true) {
                    seq.extend(o.args().iter().cloned().map(Item::E));
                } else {
                    // NCMul can have commutative parts as well
                    for q in o.args() {
                        if q.is_commutative() == Some(true) {
                            seq.push(Item::E(q.clone()));
                        } else {
                            nc_seq.push(q.clone());
                        }
                    }
                    // append non-commutative marker, so we don't forget to
                    // process scheduled non-commutative objects
                    seq.push(Item::NcMarker);
                }
                continue;
            } else if o.is_Number() {
                // 3
                if o.is_cls(C::NaN)
                    || (coeff.is_cls(C::ComplexInfinity) && o.is_zero() == Some(true))
                {
                    // we know for sure the result will be nan
                    return nan();
                } else if coeff.is_Number() {
                    // it could be zoo
                    coeff = ops::mul(&coeff, o);
                    if coeff.is_cls(C::NaN) {
                        // we know for sure the result will be nan
                        return nan();
                    }
                } else if coeff.is_cls(C::AccumulationBounds) {
                    not_ported("Mul.flatten with AccumBounds");
                }
                continue;
            } else if o.is_cls(C::AccumulationBounds) {
                not_ported("Mul.flatten with AccumBounds");
            } else if o.is_cls(C::ComplexInfinity) {
                if !coeff.truthy() {
                    // 0 * zoo = NaN
                    return nan();
                }
                coeff = s::complex_infinity();
                continue;
            } else if !coeff.truthy()
                && o.is_cls(C::Add)
                && o.args().iter().any(|a| {
                    make_args(a).iter().any(|x| {
                        x.is_cls(C::NegativeInfinity)
                            || x.is_cls(C::ComplexInfinity)
                            || x.is_cls(C::Infinity)
                    })
                })
            {
                // zero * Add with an infinity is NaN
                return nan();
            } else if o.is_cls(C::ImaginaryUnit) {
                neg1e = ops::add(&neg1e, &s::half());
                continue;
            } else if o.is_commutative() == Some(true) {
                //      e
                // o = b
                let (mut b, e) = o.as_base_exp();

                //  y
                // 3
                if o.is_Pow() && b.is_Number() {
                    // get all the factors with numeric base so they can be
                    // combined below, but don't combine negatives unless
                    // the exponent is an integer
                    if e.is_Rational() {
                        if e.is_Integer() {
                            // it is an unevaluated power
                            coeff = ops::mul(&coeff, &pow(&b, &e));
                            continue;
                        } else if e.is_negative() == Some(true) {
                            // also a sign of an unevaluated power
                            seq.push(Item::E(pow(&b, &e)));
                            continue;
                        } else if b.is_negative() == Some(true) {
                            neg1e = ops::add(&neg1e, &e);
                            b = ops::neg(&b);
                        }
                        if !b.is_cls(C::One) {
                            pnum_rat.setdefault_with(b, Vec::new).push(e);
                        }
                        continue;
                    } else if b.is_positive() == Some(true) || e.is_integer() == Some(true) {
                        num_exp.push((b, e));
                        continue;
                    }
                }
                c_powers.push((b, e));
                continue;
            }
        }

        // NON-COMMUTATIVE
        // TODO: Make non-commutative exponents not combine automatically
        if let Some(o) = o {
            nc_seq.push(o);
        }

        // process nc_seq (if any)
        while !nc_seq.is_empty() {
            let o = nc_seq.remove(0);
            let Some(o1) = nc_part.pop() else {
                nc_part.push(o);
                continue;
            };
            //                             b    c       b+c
            // try to combine last terms: a  * a   ->  a
            let (b1, e1) = o1.as_base_exp();
            let (b2, e2) = o.as_base_exp();
            let new_exp = ops::add(&e1, &e2);
            // Only allow powers to combine if the new exponent is
            // not an Add. This allow things like a**2*b**3 == a**5
            // if a.is_commutative == False, but prohibits
            // a**x*a**y and x**a*x**b from combining (x,y commute).
            if b1 == b2 && !new_exp.is_Add() {
                let o12 = pow(&b1, &new_exp);
                // now o12 could be a commutative object
                if o12.is_commutative() == Some(true) {
                    seq.push(Item::E(o12));
                    continue;
                } else {
                    nc_seq.insert(0, o12);
                }
            } else {
                nc_part.push(o1);
                nc_part.push(o);
            }
        }
    }

    // We do want a combined exponent if it would not be an Add, such as
    //  y    2y     3y
    // x  * x   -> x
    // We determine if two exponents have the same term by using
    // as_coeff_Mul.
    //
    // Unfortunately, this isn't smart enough to consider combining into
    // exponents that might already be adds, so things like:
    //  z - y    y
    // x      * x  will be left alone.  This is because checking every possible
    // combination can slow things down.

    // gather exponents of common bases...
    // in c_powers
    let mut c_powers = gather(c_powers);
    // and in num_exp
    let num_exp = gather(num_exp);

    // --- PART 2 ---
    //
    // o process collected powers  (x**0 -> 1; x**1 -> x; otherwise Pow)
    // o combine collected powers  (2**x * 3**x -> 6**x)
    //   with numeric base

    //  0             1
    // x  -> 1       x  -> x

    // this should only need to run twice; if it fails because
    // it needs to be run more times, perhaps this should be
    // changed to a "while True" loop -- the only reason it
    // isn't such now is to allow a less-than-perfect result to
    // be obtained rather than raising an error or entering an
    // infinite loop
    for _ in 0..2 {
        let mut new_c_powers = Vec::new();
        let mut changed = false;
        for (b, e) in &c_powers {
            let (mut b, mut e) = (b.clone(), e.clone());
            if e.is_zero() == Some(true) {
                // canceling out infinities yields NaN
                if (b.is_Add() || b.is_Mul())
                    && [s::complex_infinity(), s::infinity(), s::negative_infinity()]
                        .iter()
                        .any(|infty| b.args().contains(infty))
                {
                    return nan();
                }
                continue;
            }
            let p = if e.is_cls(C::One) {
                if b.is_Number() {
                    coeff = ops::mul(&coeff, &b);
                    continue;
                }
                b.clone()
            } else {
                let p = pow(&b, &e);
                // let's make sure that the base of a Pow is not a Pow
                if p.is_Pow() && !b.is_Pow() {
                    let bi = b.clone();
                    (b, e) = p.as_base_exp();
                    if b != bi {
                        changed = true;
                    }
                }
                p
            };
            c_part.push(p);
            new_c_powers.push((b, e));
        }
        // there might have been a change, but unless the base
        // matches some other base, there is nothing to do
        let distinct: HashSet<&Expr> = new_c_powers.iter().map(|(b, _)| b).collect();
        if changed && distinct.len() != new_c_powers.len() {
            // start over again
            c_part = Vec::new();
            c_powers = gather(new_c_powers);
        } else {
            break;
        }
    }

    //  x    x     x
    // 2  * 3  -> 6
    let mut inv_exp_dict: PyDict<Expr, Vec<Expr>> = PyDict::new(); // exp:Mul(num-bases)     x    x
    // e.g.  x:6  for  ... * 2  * 3  * ...
    for (b, e) in num_exp {
        inv_exp_dict.setdefault_with(e, Vec::new).push(b);
    }
    let inv_exp_dict: Vec<(Expr, Expr)> =
        inv_exp_dict.into_iter().map(|(e, b)| (e, mul(b))).collect();
    c_part.extend(
        inv_exp_dict
            .iter()
            .filter(|(e, _)| e.truthy())
            .map(|(e, b)| pow(b, e)),
    );

    //   b    b
    // 2  * 3  -> 6**b   (already done above)
    //   y    y
    // a  * a  -> a**(2*y)   (already done above)
    // combine the exponents of rational-base powers: 2**(1/2)*2**(1/3)
    let mut comb_e: PyDict<Expr, Vec<Expr>> = PyDict::new();
    for (b, e) in pnum_rat {
        comb_e.setdefault_with(crate::add::add(e), Vec::new).push(b);
    }

    // process them, reducing exponents to values less than 1
    // and updating coeff if necessary else adding them to
    // num_rat for further processing
    let mut num_rat: Vec<(Expr, Expr)> = Vec::new();
    for (e, b) in comb_e {
        let b = mul(b);
        let mut e = e;
        if e.q().is_one() {
            coeff = ops::mul(&coeff, &pow(&b, &e));
            continue;
        }
        if e.p() > e.q() {
            let (e_i, ep) = divmod(&e.p(), &e.q());
            coeff = ops::mul(&coeff, &pow(&b, &Expr::integer(e_i)));
            e = Expr::rational(ep, e.q());
        }
        num_rat.push((b, e));
    }

    // extract gcd of bases in num_rat
    // 2**(1/3)*6**(1/4) -> 2**(1/3+1/4)*3**(1/4)
    let mut pnew: PyDict<Expr, Vec<Expr>> = PyDict::new();
    let mut i = 0; // steps through num_rat which may grow
    while i < num_rat.len() {
        let (mut bi, ei) = num_rat[i].clone();
        if bi == s::one() {
            i += 1;
            continue;
        }
        let mut grow = Vec::new();
        for slot in num_rat.iter_mut().skip(i + 1) {
            let (bj, ej) = slot.clone();
            let g = number_gcd(&bi, &bj);
            if !g.is_cls(C::One) {
                // 4**r1*6**r2 -> 2**(r1+r2)  *  2**r1 *  3**r2
                // this might have a gcd with something else
                let mut e = ops::add(&ei, &ej);
                if e.q().is_one() {
                    coeff = ops::mul(&coeff, &pow(&g, &e));
                } else {
                    if e.p() > e.q() {
                        // change e in place
                        let (e_i, ep) = divmod(&e.p(), &e.q());
                        coeff = ops::mul(&coeff, &pow(&g, &Expr::integer(e_i)));
                        e = Expr::rational(ep, e.q());
                    }
                    grow.push((g.clone(), e));
                }
                // update the jth item
                *slot = (ops::div(&bj, &g), ej);
                // update bi that we are checking with
                bi = ops::div(&bi, &g);
                if bi.is_cls(C::One) {
                    break;
                }
            }
        }
        if !bi.is_cls(C::One) {
            let obj = pow(&bi, &ei);
            if obj.is_Number() {
                coeff = ops::mul(&coeff, &obj);
            } else {
                // changes like sqrt(12) -> 2*sqrt(3)
                for obj in make_args(&obj) {
                    if obj.is_Number() {
                        coeff = ops::mul(&coeff, &obj);
                    } else {
                        if !obj.is_Pow() {
                            raise(ErrorKind::AssertionError, "assert obj.is_Pow");
                        }
                        let (bi, ei) = (obj.arg(0).clone(), obj.arg(1).clone());
                        pnew.setdefault_with(ei, Vec::new).push(bi);
                    }
                }
            }
        }
        num_rat.extend(grow);
        i += 1;
    }

    // combine bases of the new powers
    let mut pnew: Vec<(Expr, Expr)> = pnew.into_iter().map(|(e, b)| (e, mul(b))).collect();

    // handle -1 and I
    if neg1e.truthy() {
        // treat I as (-1)**(1/2) and compute -1's total exponent
        let (p, q) = neg1e.as_numer_denom();
        let (p, q) = (p.p(), q.p());
        // if the integer part is odd, extract -1
        let (n, p) = divmod(&p, &q);
        if n.is_odd() {
            coeff = ops::neg(&coeff);
        }
        // if it's a multiple of 1/2 extract I
        if q == Int::small(2) {
            c_part.push(s::imaginary_unit());
        } else if !p.is_zero() {
            // see if there is any positive base this power of
            // -1 can join
            let neg1e = Expr::rational(p, q);
            let mut joined = false;
            for (e, b) in pnew.iter_mut() {
                if *e == neg1e && b.is_positive() == Some(true) {
                    *b = ops::neg(b);
                    joined = true;
                    break;
                }
            }
            if !joined {
                // keep it separate; we've already evaluated it to
                // the maximum extent possible
                c_part.push(pow_with(&s::negative_one(), &neg1e, false));
            }
        }
    }

    // add all the pnew powers
    c_part.extend(pnew.iter().map(|(e, b)| pow(b, e)));

    // oo, -oo
    if coeff
        .as_float()
        .is_some_and(|(m, _)| *m == rsympy_mpf::FINF || *m == rsympy_mpf::FNINF)
    {
        not_ported("Mul.flatten with a Float infinity coefficient");
    }
    if coeff.is_cls(C::Infinity) || coeff.is_cls(C::NegativeInfinity) {
        fn handle_for_oo(part: Vec<Expr>, mut coeff_sign: i64) -> (Vec<Expr>, i64) {
            let mut new_part = Vec::new();
            for t in part {
                if t.is_extended_positive() == Some(true) {
                    continue;
                }
                if t.is_extended_negative() == Some(true) {
                    coeff_sign *= -1;
                    continue;
                }
                new_part.push(t);
            }
            (new_part, coeff_sign)
        }
        let coeff_sign;
        (c_part, coeff_sign) = handle_for_oo(c_part, 1);
        let coeff_sign_nc;
        (nc_part, coeff_sign_nc) = handle_for_oo(nc_part, coeff_sign);
        coeff = ops::mul(&coeff, &Expr::integer(coeff_sign_nc));
    }

    // zoo
    if coeff.is_cls(C::ComplexInfinity) {
        // zoo might be
        //   infinite_real + bounded_im
        //   bounded_real + infinite_im
        //   infinite_real + infinite_im
        // and non-zero real or imaginary will not change that status.
        let keep =
            |c: &Expr| !(fuzzy_not(c.is_zero()) == Some(true) && c.is_extended_real().is_some());
        c_part.retain(keep);
        nc_part.retain(keep);
    } else if coeff.is_zero() == Some(true) {
        // 0
        // we know for sure the result will be 0 except the multiplicand
        // is infinity or a matrix
        if c_part.iter().any(|c| c.is_finite() == Some(false)) {
            return nan();
        }
        return (vec![coeff], vec![]);
    }

    // check for straggling Numbers that were produced
    let mut new = Vec::new();
    for i in c_part {
        if i.is_Number() {
            coeff = ops::mul(&coeff, &i);
        } else {
            new.push(i);
        }
    }
    let mut c_part = new;

    // order commutative part canonically
    sort_args(&mut c_part);

    // current code expects coeff to be always in slot-0
    if !coeff.is_cls(C::One) {
        c_part.insert(0, coeff);
    }

    // we are done
    if crate::ctx::distribute()
        && nc_part.is_empty()
        && c_part.len() == 2
        && c_part[0].is_Number()
        && c_part[0].is_finite() == Some(true)
        && c_part[1].is_Add()
    {
        // 2*(1+a) -> 2 + 2 * a
        let coeff = c_part[0].clone();
        c_part = vec![crate::add::add(
            c_part[1]
                .args()
                .iter()
                .map(|f| ops::mul(&coeff, f))
                .collect(),
        )];
    }

    (c_part, nc_part)
}

/// `Mul._from_args(args, is_commutative)`: no evaluation, args as given.
/// `is_commutative` is computed from the args when not given (`None` here
/// is SymPy's `is_commutative=None` argument).
pub fn from_args(args: Vec<Expr>, is_commutative: Option<Option<bool>>) -> Expr {
    crate::construct::assoc_from_args(crate::generated::ClassId::Mul, args, is_commutative)
}

/// `_unevaluated_Mul(*args)`.
pub fn unevaluated_mul(args: Vec<Expr>) -> Expr {
    let mut cargs = Vec::new();
    let mut ncargs = Vec::new();
    let mut args = args;
    let mut co = s::one();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].clone();
        i += 1;
        if a.is_Mul() {
            let (a_c, a_nc) = a.args_cnc(true);
            args.extend(a_c); // grow args
            ncargs.extend(a_nc);
        } else if a.is_Number() {
            co = ops::mul(&co, &a);
        } else if a.is_commutative() == Some(true) {
            cargs.push(a);
        } else {
            ncargs.push(a);
        }
    }
    sort_args(&mut cargs);
    if !co.is_cls(C::One) {
        cargs.insert(0, co);
    }
    cargs.extend(ncargs);
    from_args(cargs, None)
}

/// `_keep_coeff(coeff, factors, clear, sign)`: `coeff*factors` without
/// distributing a Number over an Add.
pub fn keep_coeff(coeff: &Expr, factors: &Expr, clear: bool, sign: bool) -> Expr {
    let (mut coeff, mut factors) = (coeff.clone(), factors.clone());
    if !coeff.is_Number() {
        if factors.is_Number() {
            std::mem::swap(&mut coeff, &mut factors);
        } else {
            return ops::mul(&coeff, &factors);
        }
    }
    if factors.is_cls(C::One) {
        return coeff;
    }
    if coeff.is_cls(C::One) {
        factors
    } else if coeff.is_cls(C::NegativeOne) && !sign {
        ops::neg(&factors)
    } else if factors.is_Add() {
        if !clear && coeff.is_Rational() && !coeff.q().is_one() {
            let args: Vec<(Expr, Expr)> = factors
                .args()
                .iter()
                .map(|i| {
                    let (c, m) = i.as_coeff_Mul(false);
                    (keep_coeff(&c, &coeff, true, false), m)
                })
                .collect();
            if args.iter().any(|(c, _)| c.is_Integer()) {
                return crate::add::from_args(
                    args.into_iter()
                        .map(|(c, m)| {
                            if c == s::one() {
                                m
                            } else {
                                from_args(vec![c, m], None)
                            }
                        })
                        .collect(),
                    None,
                );
            }
        }
        mul_with(vec![coeff, factors], false)
    } else if factors.is_Mul() {
        let mut margs = factors.args().to_vec();
        if margs[0].is_Number() {
            margs[0] = ops::mul(&margs[0], &coeff);
            if margs[0] == s::one() {
                margs.remove(0);
            }
        } else {
            margs.insert(0, coeff);
        }
        from_args(margs, None)
    } else {
        let mut m = ops::mul(&coeff, &factors);
        if m.is_Number() && !factors.is_Number() {
            m = from_args(vec![coeff, factors], None);
        }
        m
    }
}

/// `Mul.__neg__`.
pub fn neg(a: &Expr) -> Expr {
    let (mut c, mut args) = as_coeff_mul(a, true);
    if !args[0].is_cls(C::ComplexInfinity) {
        c = ops::neg(&c);
    }
    if !c.is_cls(C::One) {
        if args[0].is_Number() {
            args[0] = if c.is_cls(C::NegativeOne) {
                ops::neg(&args[0])
            } else {
                ops::mul(&args[0], &c)
            };
        } else {
            args.insert(0, c);
        }
    }
    from_args(args, Some(a.is_commutative()))
}

/// `AssocOp._new_rawargs(*args)` for a Mul: `_from_args`, recomputing
/// commutativity only when `e` is non-commutative.
pub fn new_rawargs(e: &Expr, args: Vec<Expr>) -> Expr {
    crate::add::assoc_new_rawargs(e, args)
}

/// `Mul._eval_power(expt)`.
pub fn eval_power(base: &Expr, expt: &Expr) -> Option<Expr> {
    // don't break up NC terms: (A*B)**3 != A**3*B**3, it is A*B*A*B*A*B
    let (cargs, nc) = base.args_cnc(false);

    if expt.is_Integer() {
        let c = mul(cargs.iter().map(|b| pow_with(b, expt, false)).collect());
        return Some(ops::mul(&c, &pow_with(&from_args(nc, None), expt, false)));
    }
    if expt.is_Rational() && expt.q() == Int::small(2) && base.is_imaginary() == Some(true) {
        not_ported("Mul._eval_power: sqrt of an imaginary Mul (needs as_real_imag)");
    }

    let p = pow_with(base, expt, false);

    if expt.is_Rational() || expt.is_Float() {
        return Some(crate::power::expand_power_base(&p));
    }

    Some(p)
}

/// A non-commutative factor in `Mul._eval_subs`: a `[base, exponent]`
/// pair from `breakup`, or the expression that replaced it.
#[derive(Clone)]
enum NcFactor {
    Pair(Expr, Expr),
    Done(Expr),
}

impl NcFactor {
    /// `nc[i][0]`, `nc[i][1]`: only pairs are subscriptable.
    fn pair(&self) -> (&Expr, &Expr) {
        match self {
            NcFactor::Pair(b, e) => (b, e),
            NcFactor::Done(e) => raise(
                ErrorKind::TypeError,
                format!("'{}' object is not subscriptable", e.class_name()),
            ),
        }
    }
}

/// `base_exp(a)` in `Mul._eval_subs`: a true Pow or exp separated into
/// base and exponent, anything else with exponent 1.
fn subs_base_exp(a: &Expr) -> (Expr, Expr) {
    if a.is_Pow() || a.is_cls(C::Exp) {
        return a.as_base_exp();
    }
    (a.clone(), s::one())
}

/// `breakup(eq)` in `Mul._eval_subs`: the commutative factors as
/// `{base: rational exponent}` and the non-commutative ones as
/// `[base, exponent]` pairs, in order.
fn subs_breakup(eq: &Expr) -> (PyDict<Expr, Expr>, Vec<NcFactor>) {
    let mut c: PyDict<Expr, Expr> = PyDict::new();
    let mut nc = Vec::new();
    for a in make_args(eq) {
        let a = powdenest_factor(&a);
        let (mut b, mut e) = subs_base_exp(&a);
        if !e.is_cls(C::One) {
            let (co, _) = e.as_coeff_mul();
            b = pow(&b, &ops::div(&e, &co));
            e = co;
        }
        if a.is_commutative() == Some(true) {
            // c[b] += e (a defaultdict(int))
            let v = match c.get(&b) {
                Some(old) => ops::add(old, &e),
                None => e,
            };
            c.insert(b, v);
        } else {
            nc.push(NcFactor::Pair(b, e));
        }
    }
    (c, nc)
}

/// `rejoin(b, co)` in `Mul._eval_subs`: puts the rational back with the
/// exponent.
fn subs_rejoin(b: &Expr, co: &Expr) -> Expr {
    let (b, e) = subs_base_exp(b);
    pow(&b, &ops::mul(&e, co))
}

/// `ndiv(a, b)` in `Mul._eval_subs`: if `b` divides `a` in an extractive
/// way (like 1/4 divides 1/2 but not vice versa, and 2/5 does not divide
/// 1/3) the integer number of times it divides (`int(a/b)`), else 0.
fn subs_ndiv(a: &Expr, b: &Expr) -> Int {
    let (aq, bq) = (a.q(), b.q());
    if bq.py_mod(&aq).expect("q > 0").is_zero() || aq.py_mod(&bq).expect("q > 0").is_zero() {
        // int(Rational) truncates toward zero
        let r = ops::div(a, b);
        return r.p().div_rem_trunc(&r.q()).expect("q > 0").0;
    }
    Int::ZERO
}

/// `sign(r)` for a Rational exponent: -1, 0 or 1.
fn rational_sign(r: &Expr) -> i32 {
    if !r.is_Rational() {
        not_ported(format!(
            "sign() of a {} exponent in Mul._eval_subs",
            r.class_name()
        ));
    }
    r.p().signum()
}

/// `co_self.extract_multiplicatively(co_old)` for two different
/// Rationals, as `Expr.extract_multiplicatively` does it.
fn rational_extract_multiplicatively(e: &Expr, c: &Expr) -> Option<Expr> {
    if c.is_cls(C::One) {
        return Some(e.clone());
    } else if c == e {
        return Some(s::one());
    }
    let quotient = ops::div(e, c);
    let ok = if e.is_Integer() {
        quotient.is_Integer()
    } else {
        quotient.is_Rational()
    };
    if !ok || e.is_positive() == Some(true) && quotient.is_negative() == Some(true) {
        return None;
    }
    Some(quotient)
}

/// `multiplicity(p, n)` (`sympy.ntheory.factor_`) for two Rationals.
fn rational_multiplicity(p: &Expr, n: &Expr) -> Int {
    let m = |p: &Int, n: &Int| -> Int {
        match rsympy_ntheory::multiplicity(p, n) {
            Ok(k) => Int::from(k),
            Err(rsympy_ntheory::NtError::Value(msg)) => raise(ErrorKind::ValueError, msg),
            Err(rsympy_ntheory::NtError::ZeroDivision(msg)) => {
                raise(ErrorKind::ZeroDivisionError, msg)
            }
        }
    };
    let (pp, pq, np, nq) = (p.p(), p.q(), n.p(), n.q());
    if pq.is_one() && nq.is_one() {
        return m(&pp, &np);
    }
    if pq.is_one() {
        if np.is_one() {
            return -m(&pp, &nq);
        }
        return m(&pp, &np) - m(&pp, &nq);
    } else if pp.is_one() {
        return m(&pq, &nq);
    }
    let like = std::cmp::min(m(&pp, &np), m(&pq, &nq));
    let cross = std::cmp::min(m(&pq, &np), m(&pp, &nq));
    like - cross
}

/// `powdenest(a)` (`sympy.simplify.powsimp`) for a factor that
/// `Mul._eval_subs` breaks up: `powsimp(a)`, then `_denest_pow` on each
/// outermost power. Both are only ported where they leave the factor
/// alone: atoms, sums and products of symbols, undefined functions and
/// rational numbers, and powers of a symbol or an undefined function with
/// an integer, rational or symbol exponent. Everything else raises
/// `NotPortedYet`.
fn powdenest_factor(a: &Expr) -> Expr {
    let new = powsimp_unchanged(a);
    let transform = |m: &Expr| denest_pow_unchanged(m);
    let filter = |m: &Expr| m.is_Pow() || m.is_cls(C::Exp);
    crate::basic::xreplace_with(
        &new,
        &crate::basic::Transform {
            transform: &transform,
            filter: &filter,
        },
    )
}

/// A base whose powers `powsimp` and `_denest_pow` leave alone: a symbol
/// or an undefined function (`as_base_exp()` is `(b, 1)`, it is not a Mul,
/// a Rational, a Pow or exp, and its positive or polar cases all rebuild
/// `b**e`).
fn plain_base(b: &Expr) -> bool {
    b.is_Symbol() || b.is_cls(C::AppliedUndef)
}

/// `powsimp(e)` (deep=False, combine='all') where it gives `e` back
/// (rebuilt the way it rebuilds it).
fn powsimp_unchanged(e: &Expr) -> Expr {
    if e.is_Atom() {
        return e.clone();
    }
    if e.is_Add() {
        // expr.func(*[recurse(w) for w in expr.args]), then returned
        return e.func(e.args().iter().map(powsimp_unchanged).collect());
    }
    if e.is_Mul() {
        // A product of rationals, distinct commutative symbols and
        // undefined functions, and non-commutative symbols no two
        // neighbours alike: every exponent is 1, no base is a Pow, no
        // base or its inverse or negation appears twice, so both the
        // 'exp' and the 'base' passes rebuild the same product.
        let mut seen = HashSet::new();
        let mut prev_nc: Option<&Expr> = None;
        let simple = e.args().iter().all(|t| {
            if t.is_Rational() {
                return true;
            }
            match t.is_commutative() {
                Some(true) => plain_base(t) && seen.insert(t.clone()),
                Some(false) if t.is_Symbol() => {
                    let ok = prev_nc != Some(t);
                    prev_nc = Some(t);
                    ok
                }
                _ => false,
            }
        });
        if simple {
            return mul(e.args().to_vec());
        }
        not_ported(
            "powsimp of a Mul other than a product of rationals, symbols and undefined functions",
        );
    }
    if e.is_Pow() {
        // recurse(expr*_y, deep=False)/_y: the product of b**e and a dummy
        // rebuilds Pow(b, e) when e is an integer, a rational (bkey splits
        // it as root(b, q)**p) or a symbol.
        let (b, x) = (e.arg(0), e.arg(1));
        let simple_exp = x.is_Rational() || (x.is_Symbol() && x.is_commutative() == Some(true));
        if plain_base(b) && simple_exp && !x.is_cls(C::One) {
            return pow(b, x);
        }
        not_ported(format!(
            "powsimp of a Pow with a {} base and a {} exponent",
            b.class_name(),
            x.class_name()
        ));
    }
    // not an Add, Mul or Pow: returned as it is
    e.clone()
}

/// `_denest_pow(eq)` where it gives `eq` back.
fn denest_pow_unchanged(eq: &Expr) -> Expr {
    if eq.is_Pow() && plain_base(eq.arg(0)) {
        return eq.clone();
    }
    not_ported(format!(
        "_denest_pow of a {} with a {} base",
        eq.class_name(),
        eq.arg(0).class_name()
    ));
}

/// `Mul._eval_subs(old, new)`: replaces a product `old` whose factors
/// (with their powers) divide this product's, as many times as they do
/// (`(x**2*y**4).subs(x*y**2, z)` is `z**2`), keeping the order of
/// non-commutative factors.
pub(crate) fn eval_subs(e: &Expr, old: &Expr, new: &Expr) -> Option<Expr> {
    use crate::basic::_subs;

    if !old.is_Mul() {
        return None;
    }

    // try keep replacement literal so -2*x doesn't replace 4*x
    let zero = s::zero();
    if old.arg(0).is_Number()
        && crate::numbers::number_lt(old.arg(0), &zero)
        && e.arg(0).is_Number()
    {
        if crate::numbers::number_lt(e.arg(0), &zero) {
            return Some(_subs(e, &ops::neg(old), &ops::neg(new)));
        }
        return None;
    }

    // give Muls in the denominator a chance to be changed (see issue 5651)
    // rv will be the default return value
    let mut rv: Option<Expr> = None;
    let (n, d) = crate::exprtools::fraction(e, false);
    let mut self2 = e.clone();
    if !d.is_cls(C::One) {
        self2 = ops::div(&_subs(&n, old, new), &_subs(&d, old, new));
        if !self2.is_Mul() {
            return Some(_subs(&self2, old, new));
        }
        if self2 != *e {
            rv = Some(self2.clone());
        }
    }

    // Now continue with regular substitution.

    // handle the leading coefficient and use it to decide if anything
    // should even be started; we always know where to find the Rational
    // so it's a quick test

    let co_self = self2.arg(0).clone();
    let co_old = old.arg(0).clone();
    let mut co_xmul: Option<Expr> = None;
    if co_old.is_Rational() && co_self.is_Rational() {
        // if coeffs are the same there will be no updating to do
        // below after breakup() step; so skip (and keep co_xmul=None)
        if co_old != co_self {
            co_xmul = rational_extract_multiplicatively(&co_self, &co_old);
        }
    } else if co_old.is_Rational() {
        return rv;
    }

    // break self and old into factors

    let (mut c, mut nc) = subs_breakup(&self2);
    let (old_c, old_nc) = subs_breakup(old);

    // update the coefficients if we had an extraction
    // e.g. if co_self were 2*(3/35*x)**2 and co_old = 3/5
    // then co_self in c is replaced by (3/5)**2 and co_residual
    // is 2*(1/7)**2

    let abs_co_old = || ops::abs(&co_old);
    let co_residual = if co_xmul
        .as_ref()
        .is_some_and(|x| x.truthy() && x.is_Rational())
        && abs_co_old() != s::one()
    {
        let mult = Expr::integer(rational_multiplicity(&abs_co_old(), &co_self));
        if c.remove(&co_self).is_none() {
            raise(ErrorKind::KeyError, format!("{co_self}"));
        }
        let v = match c.get(&co_old) {
            Some(v) => ops::add(v, &mult),
            None => mult.clone(),
        };
        c.insert(co_old.clone(), v);
        ops::div(&co_self, &ops::pow(&co_old, &mult))
    } else {
        s::one()
    };

    // do quick tests to see if we can't succeed

    let nc_bases: HashSet<&Expr> = nc.iter().map(|f| f.pair().0).collect();
    let ok = if old_nc.len() > nc.len() {
        // more non-commutative terms
        false
    } else if old_c.len() > c.len() {
        // more commutative terms
        false
    } else if old_nc.iter().any(|f| !nc_bases.contains(f.pair().0)) {
        // unmatched non-commutative bases
        false
    } else if old_c.keys().any(|b| !c.contains_key(b)) {
        // unmatched commutative terms
        false
    } else {
        // differences in sign
        !old_c
            .iter()
            .any(|(b, old_e)| rational_sign(c.get(b).expect("checked")) != rational_sign(old_e))
    };
    if !ok {
        return rv;
    }

    let cdid: Option<Int> = if old_c.is_empty() {
        None
    } else {
        let mut rat = Vec::new();
        for (b, old_e) in old_c.iter() {
            let c_e = c.get(b).expect("checked");
            rat.push(subs_ndiv(c_e, old_e));
            if rat.last().expect("pushed").is_zero() {
                return rv;
            }
        }
        rat.into_iter().min()
    };

    let ncdid: Option<Int>;
    if old_nc.is_empty() {
        ncdid = None;
        for f in nc.iter_mut() {
            let (b, x) = f.pair();
            *f = NcFactor::Done(subs_rejoin(b, x));
        }
    } else {
        let mut did = Int::ZERO; // number of nc replacements we did
        let take = old_nc.len(); // how much to look at each time
        // max number that we can take (None: S.Infinity)
        let mut limit: Option<Int> = cdid.clone();
        let mut failed: Vec<usize> = Vec::new(); // failed terms will need subs if other terms pass
        let mut i = 0;
        while limit.as_ref().is_none_or(|l| !l.is_zero()) && i + take <= nc.len() {
            let mut hit = false;

            // the bases must be equivalent in succession, and
            // the powers must be extractively compatible on the
            // first and last factor but equal in between.

            let mut rat: Vec<Int> = Vec::new();
            let mut broke = false;
            for j in 0..take {
                let (nb, ne) = nc[i + j].pair();
                let (ob, oe) = old_nc[j].pair();
                if nb != ob {
                    broke = true;
                    break;
                } else if j == 0 || j == take - 1 {
                    rat.push(subs_ndiv(ne, oe));
                } else if ne != oe {
                    broke = true;
                    break;
                } else {
                    rat.push(Int::from(1));
                }
            }
            if !broke {
                let mut ndo = rat.into_iter().min().expect("take > 0");
                if !ndo.is_zero() {
                    if take == 1 {
                        if let Some(cd) = &cdid {
                            ndo = std::cmp::min(cd.clone(), ndo);
                        }
                        let (b0, e0) = nc[i].pair();
                        let oe0 = old_nc[0].pair().1;
                        let rest = ops::sub(e0, &ops::mul(&Expr::integer(ndo.clone()), oe0));
                        nc[i] = NcFactor::Done(ops::mul(
                            &pow(new, &Expr::integer(ndo.clone())),
                            &subs_rejoin(b0, &rest),
                        ));
                    } else {
                        ndo = Int::from(1);
                        let ndo_e = Expr::integer(ndo.clone());

                        // the left residual

                        let (b0, e0) = nc[i].pair();
                        let l =
                            subs_rejoin(b0, &ops::sub(e0, &ops::mul(&ndo_e, old_nc[0].pair().1)));

                        // eliminate all middle terms

                        let mid = new.clone();

                        // the right residual (which may be the same as the middle if take == 2)

                        let ir = i + take - 1;
                        let (rb, re) = nc[ir].pair();
                        let r = (
                            rb.clone(),
                            ops::sub(re, &ops::mul(&ndo_e, old_nc[take - 1].pair().1)),
                        );
                        let lm = ops::mul(&l, &mid);
                        let replacement = if r.1.truthy() {
                            if i + take < nc.len() {
                                vec![NcFactor::Done(lm), NcFactor::Pair(r.0, r.1)]
                            } else {
                                let r = subs_rejoin(&r.0, &r.1);
                                vec![NcFactor::Done(ops::mul(&lm, &r))]
                            }
                        } else {
                            // there was nothing left on the right
                            vec![NcFactor::Done(lm)]
                        };
                        nc.splice(i..i + take, replacement);
                    }
                    if let Some(l) = &mut limit {
                        *l = &*l - &ndo;
                    }
                    did = did + &ndo;
                    hit = true;
                }
            }
            if !hit {
                // do the subs on this failing factor
                failed.push(i);
            }
            i += 1;
        }
        // (the loop's else clause: it has no break)
        if did.is_zero() {
            return rv;
        }

        // although we didn't fail, certain nc terms may have
        // failed so we rebuild them after attempting a partial
        // subs on them

        failed.extend(i..nc.len());
        for i in failed {
            let (b, x) = nc[i].pair();
            nc[i] = NcFactor::Done(subs_rejoin(b, x).subs(old, new));
        }
        ncdid = Some(did);
    }

    // rebuild the expression

    let do_ = match (&cdid, &ncdid) {
        (None, _) => ncdid.clone(),
        (_, None) => cdid.clone(),
        (Some(a), Some(b)) => Some(std::cmp::min(a.clone(), b.clone())),
    };

    let mut margs = Vec::new();
    for (b, cb) in c.iter() {
        if let Some(ob) = old_c.get(b) {
            // calculate the new exponent
            let do_e = Expr::integer(do_.clone().expect("old_c is not empty"));
            let x = ops::sub(cb, &ops::mul(ob, &do_e));
            margs.push(subs_rejoin(b, &x));
        } else {
            margs.push(subs_rejoin(&b.subs(old, new), cb));
        }
    }
    if let Some(cd) = &cdid
        && !cd.is_zero()
        && ncdid.as_ref().is_none_or(|n| n.is_zero())
    {
        // in case we are replacing commutative with non-commutative,
        // we want the new term to come at the front just like the
        // rest of this routine
        margs.insert(0, pow(new, &Expr::integer(cd.clone())));
    }
    let nc: Vec<Expr> = nc
        .into_iter()
        .map(|f| match f {
            NcFactor::Done(x) => x,
            NcFactor::Pair(..) => unreachable!("every nc factor is rebuilt"),
        })
        .collect();
    Some(ops::mul(
        &ops::mul(&co_residual, &self2.func(margs)),
        &self2.func(nc),
    ))
}

/// `Mul._eval_is_polynomial(syms)`: every term is (`all()`, so an
/// unknown counts as not).
pub(crate) fn eval_is_polynomial(e: &Expr, syms: &[Expr]) -> Option<bool> {
    Some(
        e.args()
            .iter()
            .all(|term| term.eval_is_polynomial(syms) == Some(true)),
    )
}

/// `Mul._eval_is_rational_function(syms)`.
pub(crate) fn eval_is_rational_function(e: &Expr, syms: &[Expr]) -> Option<bool> {
    Some(
        e.args()
            .iter()
            .all(|term| term.eval_is_rational_function(syms) == Some(true)),
    )
}

/// `Mul.as_coeff_Mul(rational)`.
#[allow(non_snake_case)]
pub fn as_coeff_Mul(e: &Expr, rational: bool) -> (Expr, Expr) {
    let (coeff, args) = (e.arg(0), &e.args()[1..]);
    if coeff.is_Number() {
        if !rational || coeff.is_Rational() {
            if args.len() == 1 {
                return (coeff.clone(), args[0].clone());
            }
            return (coeff.clone(), new_rawargs(e, args.to_vec()));
        } else if coeff.is_extended_negative() == Some(true) {
            let mut v = vec![ops::neg(coeff)];
            v.extend_from_slice(args);
            return (s::negative_one(), new_rawargs(e, v));
        }
    }
    (s::one(), e.clone())
}

/// `Mul.as_coeff_mul(*deps, rational=True)` with no deps: `(coeff, factors)`.
pub fn as_coeff_mul(e: &Expr, rational: bool) -> (Expr, Vec<Expr>) {
    let args = e.args();
    if args[0].is_Number() {
        if !rational || args[0].is_Rational() {
            return (args[0].clone(), args[1..].to_vec());
        } else if args[0].is_extended_negative() == Some(true) {
            let mut v = vec![ops::neg(&args[0])];
            v.extend_from_slice(&args[1..]);
            return (s::negative_one(), v);
        }
    }
    (s::one(), args.to_vec())
}

/// `Mul.as_numer_denom()`.
pub fn as_numer_denom(e: &Expr) -> (Expr, Expr) {
    // don't use _from_args to rebuild the numerators and denominators
    // as the order is not guaranteed to be the same once they have
    // been separated from each other
    let (numers, denoms): (Vec<Expr>, Vec<Expr>) =
        e.args().iter().map(|f| f.as_numer_denom()).unzip();
    (e.func(numers), e.func(denoms))
}

/// `Mul.as_base_exp()`.
pub fn as_base_exp(e: &Expr) -> (Expr, Expr) {
    let mut e1: Option<Expr> = None;
    let mut bases = Vec::new();
    let mut nc = 0;
    for m in e.args() {
        let (b, x) = m.as_base_exp();
        if b.is_commutative() != Some(true) {
            nc += 1;
        }
        match &e1 {
            None => e1 = Some(x),
            Some(first) => {
                if x != *first || nc > 1 || !x.is_Integer() {
                    return (e.clone(), s::one());
                }
            }
        }
        bases.push(b);
    }
    (e.func(bases), e1.expect("a Mul has args"))
}

/// `Mul.as_content_primitive(radical, clear)`.
pub fn as_content_primitive(e: &Expr, radical: bool, clear: bool) -> (Expr, Expr) {
    let mut coef = s::one();
    let mut args = Vec::new();
    for a in e.args() {
        let (c, p) = a.as_content_primitive(radical, clear);
        coef = ops::mul(&coef, &c);
        if !p.is_cls(C::One) {
            args.push(p);
        }
    }
    // don't use self._from_args here to reconstruct args
    // since there may be identical args now that should be combined
    // e.g. (2+2*x)*(3+3*x) should be (6, (1 + x)**2) not (6, (1+x)*(1+x))
    (coef, e.func(args))
}

/// `Mul.as_powers_dict()`: the factors' `as_powers_dict`s summed into a
/// `defaultdict(int)`.
pub fn as_powers_dict(e: &Expr) -> PyDict<Expr, crate::exprtools::Exponent> {
    use crate::exprtools::Exponent;
    let mut d: PyDict<Expr, Exponent> = PyDict::new();
    for term in e.args() {
        for (b, x) in term.as_powers_dict() {
            let v = d.setdefault_with(b, || Exponent::Int(Int::ZERO));
            *v = v.add(&x);
        }
    }
    d
}

/// `Mul.as_two_terms()`: the head and the tail of `self` as a product.
pub fn as_two_terms(e: &Expr) -> (Expr, Expr) {
    let args = e.args();
    match args.len() {
        1 => (s::one(), e.clone()),
        2 => (args[0].clone(), args[1].clone()),
        _ => (args[0].clone(), new_rawargs(e, args[1..].to_vec())),
    }
}

/// `Mul.could_extract_minus_sign()`.
pub fn could_extract_minus_sign(e: &Expr) -> bool {
    if *e == ops::neg(e) {
        return false; // e.g. zoo*x == -zoo*x
    }
    let c = e.arg(0);
    c.is_Number() && c.is_extended_negative() == Some(true)
}

/// `type(e)._prop_handler[fact](e)` for a Mul.
pub(crate) fn handler(e: &Expr, fact: Fact) -> Option<bool> {
    match fact {
        Fact::Commutative => fuzzy_group(e.args().iter().map(Expr::is_commutative), false),
        Fact::Complex => eval_is_complex(e),
        Fact::Zero => eval_is_zero(e),
        Fact::Infinite => eval_is_infinite(e),
        Fact::Rational => eval_is_rational(e),
        Fact::Algebraic => eval_is_algebraic(e),
        Fact::Integer => eval_is_integer(e),
        Fact::Polar => eval_is_polar(e),
        Fact::ExtendedReal => eval_real_imag(e, true),
        Fact::Imaginary => eval_is_imaginary(e),
        Fact::Hermitian => eval_herm_antiherm(e, true),
        Fact::Antihermitian => eval_herm_antiherm(e, false),
        Fact::Irrational => eval_is_irrational(e),
        Fact::ExtendedPositive => eval_pos_neg(e, 1),
        Fact::ExtendedNegative => eval_pos_neg(e, -1),
        Fact::Odd => eval_is_odd(e),
        Fact::Even => eval_is_even(e),
        Fact::Composite => eval_is_composite(e),
        _ => unreachable!("Mul has no handler for {fact:?}"),
    }
}

/// `Mul._eval_is_complex`.
fn eval_is_complex(e: &Expr) -> Option<bool> {
    let comp = fuzzy_group(e.args().iter().map(Expr::is_complex), false);
    if comp == Some(false) && e.args().iter().any(|a| a.is_infinite() == Some(true)) {
        if e.args().iter().any(|a| a.is_zero() != Some(false)) {
            return None;
        }
        return Some(false);
    }
    comp
}

/// `Mul._eval_is_zero_infinite_helper`: `(seen_zero, seen_infinite)`.
fn zero_infinite_helper(e: &Expr) -> (Option<bool>, Option<bool>) {
    // Three-valued logic: the flags only move from False to True or None,
    // and as soon as neither is False the answer is (None, None).
    let mut seen_zero = Some(false);
    let mut seen_infinite = Some(false);
    for a in e.args() {
        if a.is_zero() == Some(true) {
            if seen_infinite != Some(false) {
                return (None, None);
            }
            seen_zero = Some(true);
        } else if a.is_infinite() == Some(true) {
            if seen_zero != Some(false) {
                return (None, None);
            }
            seen_infinite = Some(true);
        } else {
            if seen_zero == Some(false) && a.is_zero().is_none() {
                if seen_infinite != Some(false) {
                    return (None, None);
                }
                seen_zero = None;
            }
            if seen_infinite == Some(false) && a.is_infinite().is_none() {
                if seen_zero != Some(false) {
                    return (None, None);
                }
                seen_infinite = None;
            }
        }
    }
    (seen_zero, seen_infinite)
}

/// `Mul._eval_is_zero`.
fn eval_is_zero(e: &Expr) -> Option<bool> {
    // True iff any arg is zero and no arg is infinite
    match zero_infinite_helper(e) {
        (Some(false), _) => Some(false),
        (Some(true), Some(false)) => Some(true),
        _ => None,
    }
}

/// `Mul._eval_is_infinite`.
fn eval_is_infinite(e: &Expr) -> Option<bool> {
    // True iff any arg is infinite and no arg is zero
    match zero_infinite_helper(e) {
        (Some(false), Some(true)) => Some(true),
        (_, Some(false)) => Some(false),
        _ => None,
    }
}

/// `Mul._eval_is_rational` and `_eval_is_algebraic`.
fn rational_or_algebraic(e: &Expr, f: fn(&Expr) -> Option<bool>) -> Option<bool> {
    let r = fuzzy_group(e.args().iter().map(f), true);
    if r == Some(true) {
        return r;
    } else if r == Some(false) {
        // All args except one are rational
        if e.args().iter().all(|a| a.is_zero() == Some(false)) {
            return Some(false);
        }
    }
    None
}

fn eval_is_rational(e: &Expr) -> Option<bool> {
    rational_or_algebraic(e, Expr::is_rational)
}

fn eval_is_algebraic(e: &Expr) -> Option<bool> {
    rational_or_algebraic(e, Expr::is_algebraic)
}

/// `Add(*[i.as_base_exp()[1] for i in terms if i.is_even]) - trailing(n)`.
fn twos_minus_trailing(terms: &[Expr], n: &Int) -> Expr {
    let exps: Vec<Expr> = terms
        .iter()
        .filter(|i| i.is_even() == Some(true))
        .map(|i| i.as_base_exp().1)
        .collect();
    let trailing = n.trailing_zeros().unwrap_or(0);
    ops::sub(&crate::add::add(exps), &Expr::integer(Int::from(trailing)))
}

/// `Mul._eval_is_integer`.
fn eval_is_integer(e: &Expr) -> Option<bool> {
    let is_rational = eval_is_rational(e);
    if is_rational == Some(false) {
        return Some(false);
    }

    let mut numerators = Vec::new();
    let mut denominators = Vec::new();
    let mut unknown = false;
    for a in e.args() {
        let mut hit = false;
        if a.is_integer() == Some(true) {
            if !ops::abs(a).is_cls(C::One) {
                numerators.push(a.clone());
            }
        } else if a.is_Rational() {
            let (n, d) = a.as_numer_denom();
            if !ops::abs(&n).is_cls(C::One) {
                numerators.push(n);
            }
            if !d.is_cls(C::One) {
                denominators.push(d);
            }
        } else if a.is_Pow() {
            let (b, ex) = a.as_base_exp();
            if b.is_integer() != Some(true) || ex.is_integer() != Some(true) {
                hit = true;
                unknown = true;
            }
            if ex.is_negative() == Some(true) {
                denominators.push(if a.is_cls(C::Half) {
                    Expr::integer(2)
                } else {
                    pow(a, &s::negative_one())
                });
            } else if !hit {
                // int b and pos int e: a = b**e is integer
                if ex.is_positive() == Some(true) {
                    raise(ErrorKind::AssertionError, "assert not e.is_positive");
                }
                // for rational self and e equal to zero: a = b**e is 1
                if ex.is_zero() == Some(true) {
                    raise(ErrorKind::AssertionError, "assert not e.is_zero");
                }
                return None; // sign of e unknown -> self.is_integer unknown
            } else {
                // x**2, 2**x, or x**y with x and y int-unknown -> unknown
                return None;
            }
        } else {
            return None;
        }
    }

    if denominators.is_empty() && !unknown {
        return Some(true);
    }

    let allodd = |x: &[Expr]| x.iter().all(|i| i.is_odd() == Some(true));
    let alleven = |x: &[Expr]| x.iter().all(|i| i.is_even() == Some(true));
    let anyeven = |x: &[Expr]| x.iter().any(|i| i.is_even() == Some(true));

    if numerators.is_empty()
        && !denominators.is_empty()
        && denominators
            .iter()
            .all(|d| crate::relational::is_gt(d, &s::one()) == Some(true))
    {
        return Some(false);
    } else if unknown {
        return None;
    } else if allodd(&numerators) && anyeven(&denominators) {
        return Some(false);
    } else if anyeven(&numerators) && denominators == [Expr::integer(2)] {
        return Some(true);
    } else if alleven(&numerators)
        && allodd(&denominators)
        && ops::sub(&mul_with(denominators.clone(), false), &s::one()).is_positive() == Some(true)
    {
        return Some(false);
    }
    if denominators.len() == 1 {
        let d = &denominators[0];
        if d.is_Integer()
            && d.is_even() == Some(true)
            // if minimal power of 2 in num vs den is not
            // negative then we have an integer
            && twos_minus_trailing(&numerators, &d.p()).is_nonnegative() == Some(true)
        {
            return Some(true);
        }
    }
    if numerators.len() == 1 {
        let n = &numerators[0];
        if n.is_Integer()
            && n.is_even() == Some(true)
            // if minimal power of 2 in den vs num is positive
            // then we have have a non-integer
            && twos_minus_trailing(&denominators, &n.p()).is_positive() == Some(true)
        {
            return Some(false);
        }
    }
    None
}

/// `Mul._eval_is_polar`.
fn eval_is_polar(e: &Expr) -> Option<bool> {
    let has_polar = e.args().iter().any(|a| a.is_polar() == Some(true));
    Some(
        has_polar
            && e.args()
                .iter()
                .all(|a| a.is_polar() == Some(true) || a.is_positive() == Some(true)),
    )
}

/// `Mul._eval_real_imag(real)`.
fn eval_real_imag(e: &Expr, mut real: bool) -> Option<bool> {
    let mut zero = Some(false);
    let mut t_not_re_im: Option<Expr> = None;
    let truthy = |t: &Option<Expr>| t.as_ref().is_some_and(Expr::truthy);

    for t in e.args() {
        // (t.is_complex or t.is_infinite) is False and t.is_extended_real is False
        if t.is_complex() != Some(true)
            && t.is_infinite() == Some(false)
            && t.is_extended_real() == Some(false)
        {
            return Some(false);
        } else if t.is_imaginary() == Some(true) {
            // I
            real = !real;
        } else if t.is_extended_real() == Some(true) {
            // 2
            if zero != Some(true) {
                let z = t.is_zero();
                if z != Some(true) && zero == Some(false) {
                    zero = z;
                } else if z == Some(true) {
                    if e.args().iter().all(|a| a.is_finite() == Some(true)) {
                        return Some(true);
                    }
                    return None;
                }
            }
        } else if t.is_extended_real() == Some(false) {
            // symbolic or literal like `2 + I` or symbolic imaginary
            if truthy(&t_not_re_im) {
                return None; // complex terms might cancel
            }
            t_not_re_im = Some(t.clone());
        } else if t.is_imaginary() == Some(false) {
            // symbolic like `2` or `2 + I`
            if truthy(&t_not_re_im) {
                return None; // complex terms might cancel
            }
            t_not_re_im = Some(t.clone());
        } else {
            return None;
        }
    }

    if truthy(&t_not_re_im) {
        let t = t_not_re_im.expect("set");
        if t.is_extended_real() == Some(false) && real {
            // like 3
            return zero; // 3*(smthng like 2 + I or i) is not real
        }
        if t.is_imaginary() == Some(false) && !real {
            // like I
            return zero; // I*(smthng like 2 or 2 + I) is not real
        }
    } else if zero == Some(false) {
        return Some(real); // can't be trumped by 0
    } else if real {
        return Some(real); // doesn't matter what zero is
    }
    None
}

/// `Mul._eval_is_imaginary`.
fn eval_is_imaginary(e: &Expr) -> Option<bool> {
    if e.args()
        .iter()
        .all(|a| a.is_zero() == Some(false) && a.is_finite() == Some(true))
    {
        return eval_real_imag(e, false);
    }
    None
}

/// `Mul._eval_herm_antiherm(herm)`.
fn eval_herm_antiherm(e: &Expr, mut herm: bool) -> Option<bool> {
    for t in e.args() {
        let (h, ah) = (t.is_hermitian(), t.is_antihermitian());
        if h.is_none() || ah.is_none() {
            return None;
        }
        if h == Some(true) {
            continue;
        } else if ah == Some(true) {
            herm = !herm;
        } else {
            return None;
        }
    }

    if herm {
        return Some(herm);
    }

    match eval_is_zero(e) {
        Some(true) => Some(true),
        Some(false) => Some(herm),
        None => None,
    }
}

/// `Mul._eval_is_irrational`.
fn eval_is_irrational(e: &Expr) -> Option<bool> {
    for (k, t) in e.args().iter().enumerate() {
        let a = t.is_irrational();
        if a == Some(true) {
            // others = list(self.args); others.remove(t)
            let first = e.args().iter().position(|x| x == t).unwrap_or(k);
            if e.args()
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != first)
                .all(|(_, x)| x.is_rational() == Some(true) && fuzzy_not(x.is_zero()) == Some(true))
            {
                return Some(true);
            }
            return None;
        }
        a?;
    }
    if e.args().iter().all(|x| x.is_real() == Some(true)) {
        return Some(false);
    }
    None
}

/// `Mul._eval_pos_neg(sign)`.
fn eval_pos_neg(e: &Expr, mut sign: i32) -> Option<bool> {
    let (mut saw_non, mut saw_not) = (false, false);
    for t in e.args() {
        if t.is_extended_positive() == Some(true) {
            continue;
        } else if t.is_extended_negative() == Some(true) {
            sign = -sign;
        } else if t.is_zero() == Some(true) {
            if e.args().iter().all(|a| a.is_finite() == Some(true)) {
                return Some(false);
            }
            return None;
        } else if t.is_extended_nonpositive() == Some(true) {
            sign = -sign;
            saw_non = true;
        } else if t.is_extended_nonnegative() == Some(true) {
            saw_non = true;
        // FIXME: is_positive/is_negative is False doesn't take account of
        // Symbol('x', infinite=True, extended_real=True) which has
        // e.g. is_positive is False but has uncertain sign.
        } else if t.is_positive() == Some(false) {
            sign = -sign;
            if saw_not {
                return None;
            }
            saw_not = true;
        } else if t.is_negative() == Some(false) {
            if saw_not {
                return None;
            }
            saw_not = true;
        } else {
            return None;
        }
    }
    if sign == 1 && !saw_non && !saw_not {
        return Some(true);
    }
    if sign < 0 {
        return Some(false);
    }
    None
}

/// `Mul._eval_is_odd`.
fn eval_is_odd(e: &Expr) -> Option<bool> {
    let is_integer = eval_is_integer(e);
    if is_integer != Some(true) {
        return is_integer;
    }

    let (n, d) = crate::exprtools::fraction(e, false);
    if d.is_Integer() && d.is_even() == Some(true) {
        // if minimal power of 2 in num vs den is
        // positive then we have an even number
        if twos_minus_trailing(&make_args(&n), &d.p()).is_positive() == Some(true) {
            return Some(false);
        }
        return None;
    }
    let mut r = Some(true);
    let mut acc = s::one();
    for t in e.args() {
        if ops::abs(t).is_cls(C::One) {
            continue;
        }
        if t.is_even() == Some(true) {
            return Some(false);
        }
        if r == Some(false) {
        } else if acc != s::one() && ops::add(&acc, t).is_odd() == Some(true) {
            r = Some(false);
        } else if t.is_even().is_none() {
            r = None;
        }
        acc = t.clone();
    }
    r
}

/// `Mul._eval_is_even`.
fn eval_is_even(e: &Expr) -> Option<bool> {
    let (n, d) = crate::exprtools::fraction(e, false);
    if n.is_Integer()
        && n.is_even() == Some(true)
        // if minimal power of 2 in den vs num is not
        // negative then this is not an integer and
        // can't be even
        && twos_minus_trailing(&make_args(&d), &n.p()).is_nonnegative() == Some(true)
    {
        return Some(false);
    }
    None
}

/// `Mul._eval_is_composite`.
fn eval_is_composite(e: &Expr) -> Option<bool> {
    // count of symbols with minimum value greater than one
    let mut number_of_args = 0;
    for arg in e.args() {
        if !(arg.is_integer() == Some(true) && arg.is_positive() == Some(true)) {
            return None;
        }
        if ops::sub(arg, &s::one()).is_positive() == Some(true) {
            number_of_args += 1;
        }
    }
    if number_of_args > 1 {
        return Some(true);
    }
    None
}
