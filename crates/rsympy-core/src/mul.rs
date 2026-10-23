//! `sympy.core.mul`: `Mul`.
//!
//! Ported from `sympy/core/mul.py`; the constructor is `AssocOp.__new__`
//! (in `add.rs`) with [`flatten`]. The `_eval_is_*` handlers live in
//! [`handler`] at the bottom.

use std::collections::HashSet;

use rsympy_num::Int;

use crate::add::{Flattened, assoc_new};
use crate::assume::fuzzy_not;
use crate::compare::sort_args;
use crate::error::not_ported;
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
        debug_assert!(!a.is_cls(C::One));
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
                        assert!(obj.is_Pow());
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
    let _ = (e, fact);
    not_ported("Mul assumptions")
}
