//! `sympy.core.add`: `Add`, and `AssocOp.__new__` (shared with `Mul`).
//!
//! Ported from `sympy/core/add.py` and `sympy/core/operations.py`. The
//! `_eval_is_*` handlers live in [`handler`] at the bottom.

use crate::assume::{fuzzy_group, fuzzy_not, fuzzy_or};
use crate::compare::sort_args;
use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::Expr;
use crate::facts::Fact;
use crate::generated::{ClassId as C, Tag};
use crate::numbers::s;
use crate::ops;
use crate::py::PyDict;

/// What `cls.flatten(seq)` returns: `(c_part, nc_part, order_symbols)`.
/// `order_symbols` is always `None` without `Order`, which is not ported.
pub(crate) type Flattened = (Vec<Expr>, Vec<Expr>);

/// `AssocOp.__new__(cls, *args, evaluate=evaluate)` for Add and Mul.
pub(crate) fn assoc_new(
    cls: C,
    args: Vec<Expr>,
    evaluate: bool,
    flatten: fn(Vec<Expr>) -> Flattened,
) -> Expr {
    // Disallow non-Expr args in Add/Mul (non-Relational ones only warn)
    if args.iter().any(|a| a.is_a(Tag::Relational)) {
        let name = if cls == C::Add { "Add" } else { "Mul" };
        raise(
            ErrorKind::TypeError,
            format!("Relational cannot be used in {name}"),
        );
    }
    if !evaluate {
        return crate::construct::assoc_from_args(cls, args, None);
    }
    let identity = if cls == C::Add { C::Zero } else { C::One };
    let mut args: Vec<Expr> = args.into_iter().filter(|a| !a.is_cls(identity)).collect();
    match args.len() {
        0 => return crate::construct::assoc_from_args(cls, args, None),
        1 => return args.pop().expect("one arg"),
        _ => {}
    }
    let (mut c_part, nc_part) = flatten(args);
    let is_commutative = nc_part.is_empty();
    c_part.extend(nc_part);
    crate::construct::assoc_from_args(cls, c_part, Some(Some(is_commutative)))
}

/// `Add(*args)` under the current `global_parameters.evaluate`.
pub fn add(args: Vec<Expr>) -> Expr {
    add_with(args, crate::ctx::evaluate())
}

/// `Add(*args, evaluate=evaluate)`: `AssocOp.__new__` with `Add.flatten`.
pub fn add_with(args: Vec<Expr>, evaluate: bool) -> Expr {
    assoc_new(C::Add, args, evaluate, flatten)
}

/// `Add.flatten(seq)`.
pub(crate) fn flatten(mut seq: Vec<Expr>) -> Flattened {
    if seq.len() == 2 {
        let (mut a, mut b) = (&seq[0], &seq[1]);
        if b.is_Rational() {
            (a, b) = (b, a);
        }
        if a.is_Rational() && b.is_Mul() {
            let rv = vec![a.clone(), b.clone()];
            if rv.iter().all(|s| s.is_commutative() == Some(true)) {
                return (rv, vec![]);
            }
            return (vec![], rv);
        }
    }

    // term -> coeff
    // e.g. x**2 -> 5   for ... + 5*x**2 + ...
    let mut terms: PyDict<Expr, Expr> = PyDict::new();
    // coefficient (Number or zoo) to always be in slot 0
    // e.g. 3 + ...
    let mut coeff = s::zero();
    let nan = || (vec![s::nan()], vec![]);

    let mut i = 0;
    while i < seq.len() {
        let o = seq[i].clone();
        i += 1;
        let (c, s) = if o.is_Order() {
            not_ported("Add.flatten with an Order term")
        } else if o.is_Number() {
            // 3 or NaN
            if o.is_cls(C::NaN)
                || (coeff.is_cls(C::ComplexInfinity) && o.is_finite() == Some(false))
            {
                // we know for sure the result will be nan
                return nan();
            }
            if coeff.is_Number() {
                coeff = ops::add(&coeff, &o);
                if coeff.is_cls(C::NaN) {
                    // we know for sure the result will be nan
                    return nan();
                }
            } else if coeff.is_cls(C::AccumulationBounds) {
                not_ported("Add.flatten with AccumBounds");
            }
            continue;
        } else if o.is_cls(C::AccumulationBounds) {
            not_ported("Add.flatten with AccumBounds")
        } else if o.is_cls(C::ComplexInfinity) {
            if coeff.is_finite() == Some(false) {
                // we know for sure the result will be nan
                return nan();
            }
            coeff = s::complex_infinity();
            continue;
        } else if o.is_Add() {
            // NB: here we assume Add is always commutative
            seq.extend(o.args().iter().cloned());
            continue;
        } else if o.is_Mul() {
            o.as_coeff_Mul(false)
        } else if o.is_Pow() {
            // check for unevaluated Pow, e.g. 2**3 or 2**(-1/2)
            let (b, e) = o.as_base_exp();
            if b.is_Number()
                && (e.is_Integer() || (e.is_Rational() && e.is_negative() == Some(true)))
            {
                seq.push(ops::pow(&b, &e));
                continue;
            }
            (s::one(), o)
        } else {
            // everything else
            (s::one(), o)
        };

        // now we have:
        // o = c*s, where
        //
        // c is a Number
        // s is an expression with number factor extracted
        // let's collect terms with the same s, so e.g.
        // 2*x**2 + 3*x**2  ->  5*x**2
        if let Some(t) = terms.get_mut(&s) {
            *t = ops::add(t, &c);
            if t.is_cls(C::NaN) {
                // we know for sure the result will be nan
                return nan();
            }
        } else {
            terms.insert(s, c);
        }
    }

    // now let's construct new args:
    // [2*x**2, x**3, 7*x**4, pi, ...]
    let mut newseq = Vec::new();
    let mut noncommutative = false;
    for (s, c) in terms.iter() {
        if c.is_zero() == Some(true) {
            // 0*s
            continue;
        } else if c.is_cls(C::One) {
            // 1*s
            newseq.push(s.clone());
        } else if s.is_Mul() {
            // Mul, already keeps its arguments in perfect order.
            // so we can simply put c in slot0 and go the fast way.
            let mut args = vec![c.clone()];
            args.extend(s.args().iter().cloned());
            newseq.push(crate::mul::new_rawargs(s, args));
        } else if s.is_Add() {
            // we just re-create the unevaluated Mul
            newseq.push(crate::mul::mul_with(vec![c.clone(), s.clone()], false));
        } else {
            // alternatively we have to call all Mul's machinery (slow)
            newseq.push(crate::mul::mul(vec![c.clone(), s.clone()]));
        }
        noncommutative = noncommutative || s.is_commutative() != Some(true);
    }

    // oo, -oo
    if coeff.is_cls(C::Infinity) {
        newseq
            .retain(|f| !(f.is_extended_nonnegative() == Some(true) || f.is_real() == Some(true)));
    } else if coeff.is_cls(C::NegativeInfinity) {
        newseq
            .retain(|f| !(f.is_extended_nonpositive() == Some(true) || f.is_real() == Some(true)));
    }

    if coeff.is_cls(C::ComplexInfinity) {
        // zoo might be
        //   infinite_real + finite_im
        //   finite_real + infinite_im
        //   infinite_real + infinite_im
        // addition of a finite real or imaginary number won't be able to
        // change the zoo nature; adding an infinite qualtity would result
        // in a NaN condition if it had sign opposite of the infinite
        // portion of zoo, e.g., infinite_real - infinite_real.
        newseq.retain(|c| !(c.is_finite() == Some(true) && c.is_extended_real().is_some()));
    }

    // order args canonically
    sort_args(&mut newseq);

    // current code expects coeff to be first
    if !coeff.is_cls(C::Zero) {
        newseq.insert(0, coeff);
    }

    // we are done
    if noncommutative {
        (vec![], newseq)
    } else {
        (newseq, vec![])
    }
}

/// `Add._from_args(args, is_commutative)`: no evaluation, args as given.
/// `is_commutative` is computed from the args when not given (`None` here
/// is SymPy's `is_commutative=None` argument).
pub fn from_args(args: Vec<Expr>, is_commutative: Option<Option<bool>>) -> Expr {
    crate::construct::assoc_from_args(crate::generated::ClassId::Add, args, is_commutative)
}

/// `_unevaluated_Add(*args)`.
pub fn unevaluated_add(args: Vec<Expr>) -> Expr {
    let mut args = args;
    let mut newargs = Vec::new();
    let mut co = s::zero();
    while let Some(a) = args.pop() {
        if a.is_Add() {
            // this will keep nesting from building up
            // so that x + (x + 1) -> x + x + 1 (3 args)
            args.extend(a.args().iter().cloned());
        } else if a.is_Number() {
            co = ops::add(&co, &a);
        } else {
            newargs.push(a);
        }
    }
    sort_args(&mut newargs);
    if co.truthy() {
        newargs.insert(0, co);
    }
    from_args(newargs, None)
}

/// `Add.__neg__`.
pub fn neg(a: &Expr) -> Expr {
    if !crate::ctx::distribute() {
        // Expr.__neg__
        return crate::mul::from_args(vec![s::negative_one(), a.clone()], Some(a.is_commutative()));
    }
    crate::mul::mul(vec![s::negative_one(), a.clone()])
}

/// `AssocOp._new_rawargs(*args)`: `_from_args` with `self`'s
/// commutativity, recomputed when `self` is non-commutative.
pub(crate) fn assoc_new_rawargs(e: &Expr, args: Vec<Expr>) -> Expr {
    let c = e.is_commutative();
    let is_commutative = if c == Some(false) { None } else { Some(c) };
    crate::construct::assoc_from_args(e.cls(), args, is_commutative)
}

/// `Add._new_rawargs(*args)`.
pub fn new_rawargs(e: &Expr, args: Vec<Expr>) -> Expr {
    assoc_new_rawargs(e, args)
}

/// `Add._eval_power(expt)`.
pub fn eval_power(base: &Expr, expt: &Expr) -> Option<Expr> {
    if base.args().len() == 2 && base.args().iter().any(|a| a.is_infinite() == Some(true)) {
        if expt.is_zero() == Some(false) {
            not_ported("Add._eval_power: is_eq(expt, 1) and the a + I*b test (needs Expr.coeff)");
        }
        return None;
    }
    if expt.is_Rational() && base.is_number() {
        not_ported("Add._eval_power of a number (needs pure_complex)");
    }
    None
}

/// `Add.as_coeff_Add(rational)`.
#[allow(non_snake_case)]
pub fn as_coeff_Add(e: &Expr, rational: bool) -> (Expr, Expr) {
    let (coeff, args) = (e.arg(0), &e.args()[1..]);
    if coeff.is_Number() && !rational || coeff.is_Rational() {
        return (coeff.clone(), new_rawargs(e, args.to_vec()));
    }
    (s::zero(), e.clone())
}

/// `Add.as_coeff_add(*deps)` with no deps: `(coeff, terms)`.
pub fn as_coeff_add(e: &Expr) -> (Expr, Vec<Expr>) {
    let (coeff, notrat) = e.arg(0).as_coeff_add();
    if !coeff.is_cls(C::Zero) {
        let mut v = notrat;
        v.extend_from_slice(&e.args()[1..]);
        return (coeff, v);
    }
    (s::zero(), e.args().to_vec())
}

/// `Add.as_numer_denom()`.
pub fn as_numer_denom(e: &Expr) -> (Expr, Expr) {
    use crate::mul::keep_coeff;
    // clear rational denominator
    let (content, expr) = primitive(e);
    if !expr.is_cls(C::Add) {
        return crate::mul::mul_with(vec![content, expr], false).as_numer_denom();
    }
    let (ncon, dcon) = content.as_numer_denom();

    // collect numerators and denominators of the terms
    let mut nd: PyDict<Expr, Vec<Expr>> = PyDict::new();
    for f in expr.args() {
        let (ni, di) = f.as_numer_denom();
        nd.setdefault_with(di, Vec::new).push(ni);
    }

    // check for quick exit
    if nd.len() == 1 {
        let (d, n) = nd.popitem().expect("one item");
        return (
            e.func(
                n.iter()
                    .map(|ni| keep_coeff(&ncon, ni, true, false))
                    .collect(),
            ),
            keep_coeff(&dcon, &d, true, false),
        );
    }

    // sum up the terms having a common denominator
    let nd2: Vec<(Expr, Expr)> = nd
        .into_iter()
        .map(|(d, mut n)| {
            let v = if n.len() > 1 {
                e.func(n)
            } else {
                n.pop().expect("a term")
            };
            (d, v)
        })
        .collect();

    // assemble single numerator and denominator
    let (denoms, numers): (Vec<Expr>, Vec<Expr>) = nd2.into_iter().unzip();
    let n = e.func(
        (0..numers.len())
            .map(|i| {
                let mut f = denoms[..i].to_vec();
                f.push(numers[i].clone());
                f.extend_from_slice(&denoms[i + 1..]);
                crate::mul::mul(f)
            })
            .collect(),
    );
    let d = crate::mul::mul(denoms);
    (
        keep_coeff(&ncon, &n, true, false),
        keep_coeff(&dcon, &d, true, false),
    )
}

/// `Add.primitive()`: `(content, primitive part)` with rational content.
pub fn primitive(e: &Expr) -> (Expr, Expr) {
    use crate::mul::keep_coeff;
    use rsympy_num::Int;
    let mut terms = Vec::new();
    let mut inf = false;
    for a in e.args() {
        let (mut c, mut m) = a.as_coeff_Mul(false);
        if !c.is_Rational() {
            c = s::one();
            m = a.clone();
        }
        inf = inf || m.is_cls(C::ComplexInfinity);
        terms.push((c.p(), c.q(), m));
    }
    // (inf only drops terms with q == 0, and a Rational's q is never 0)
    let _ = inf;
    let ngcd = terms.iter().fold(Int::ZERO, |g, t| g.gcd(&t.0));
    let dlcm = terms.iter().fold(Int::ONE, |l, t| l.lcm(&t.1));
    if ngcd.is_one() && dlcm.is_one() {
        return (s::one(), e.clone());
    }
    let mut terms: Vec<Expr> = terms
        .into_iter()
        .map(|(p, q, term)| {
            let c = p.floor_div(&ngcd).expect("ngcd != 0") * dlcm.floor_div(&q).expect("q != 0");
            keep_coeff(&Expr::integer(c), &term, true, false)
        })
        .collect();

    // we don't need a complete re-flattening since no new terms will join
    // so we just use the same sort as is used in Add.flatten. When the
    // coefficient changes, the ordering of terms may change, e.g.
    //     (3*x, 6*y) -> (2*y, x)
    //
    // We do need to make sure that term[0] stays in position 0, however.
    let c = if terms[0].is_Number() || terms[0].is_cls(C::ComplexInfinity) {
        Some(terms.remove(0))
    } else {
        None
    };
    sort_args(&mut terms);
    if let Some(c) = c
        && c.truthy()
    {
        terms.insert(0, c);
    }
    (Expr::rational(ngcd, dlcm), new_rawargs(e, terms))
}

/// `Add.as_content_primitive(radical, clear)`: `(R, self/R)` with `R` the
/// positive Rational taken out of `self`; with `radical`, common radicals
/// of the terms are also pulled out of the primitive part.
pub fn as_content_primitive(e: &Expr, radical: bool, clear: bool) -> (Expr, Expr) {
    use crate::mul::keep_coeff;
    use crate::py::PyIntSet;
    use rsympy_num::Int;
    let terms = e
        .args()
        .iter()
        .map(|a| {
            let (c, p) = a.as_content_primitive(radical, clear);
            keep_coeff(&c, &p, true, false)
        })
        .collect();
    let (mut con, mut prim) = e.func(terms).primitive();
    if !clear && !con.is_Integer() && prim.is_Add() {
        let (n, d) = con.as_numer_denom();
        con = n;
        let p_ = ops::div(&prim, &d);
        if p_
            .args()
            .iter()
            .any(|a| a.as_coeff_Mul(false).0.is_Integer())
        {
            prim = p_;
        } else {
            con = ops::div(&con, &d);
        }
    }
    if radical && prim.is_Add() {
        // look for common radicals that can be removed
        let args = prim.args().to_vec();
        let mut rads: Vec<PyDict<Int, Vec<Int>>> = Vec::new();
        let mut common_q: Option<PyIntSet> = None;
        let mut broke = false;
        for m in &args {
            let mut term_rads: PyDict<Int, Vec<Int>> = PyDict::new();
            for ai in m.mul_args() {
                if ai.is_Pow() {
                    let (b, x) = ai.as_base_exp();
                    if x.is_Rational() && b.is_Integer() {
                        // abs(int(b))**e.p
                        let Some(xp) = x.p().as_u64() else {
                            not_ported(
                                "Add.as_content_primitive: a radical with a negative \
                                 exponent (abs(int(b))**e.p is a Python float)",
                            )
                        };
                        term_rads
                            .setdefault_with(x.q(), Vec::new)
                            .push(b.p().abs().pow(xp));
                    }
                }
            }
            if term_rads.is_empty() {
                broke = true;
                break;
            }
            let keys: PyIntSet = term_rads.keys().cloned().collect();
            match common_q.take() {
                None => common_q = Some(keys),
                Some(cq) => {
                    let cq = cq.intersection(&keys);
                    let empty = cq.is_empty();
                    common_q = Some(cq);
                    if empty {
                        broke = true;
                        break;
                    }
                }
            }
            rads.push(term_rads);
        }
        if !broke {
            // process rads
            // keep only those in common_q
            let common_q = common_q.expect("an Add has args");
            let rads: Vec<PyDict<Int, Int>> = rads
                .into_iter()
                .map(|r| {
                    r.into_iter()
                        .filter(|(q, _)| common_q.contains(q))
                        .map(|(q, v)| {
                            // Mul(*r[q]) of Python ints
                            let m = crate::mul::mul(v.into_iter().map(Expr::integer).collect());
                            if !m.is_Integer() {
                                not_ported("Add.as_content_primitive: igcd of an unevaluated Mul");
                            }
                            (q, m.p())
                        })
                        .collect()
                })
                .collect();
            // find the gcd of bases for each q
            let mut g_list = Vec::new();
            for q in common_q.iter() {
                let g = rads
                    .iter()
                    .fold(Int::ZERO, |g, r| g.gcd(r.get(q).expect("q in common_q")));
                if !g.is_one() {
                    g_list.push(crate::power::pow(
                        &Expr::integer(g),
                        &Expr::rational(Int::ONE, q.clone()),
                    ));
                }
            }
            if !g_list.is_empty() {
                let g = crate::mul::mul(g_list);
                let args = args.iter().map(|ai| ops::div(ai, &g)).collect();
                prim = ops::mul(&g, &prim.func(args));
            }
        }
    }
    (con, prim)
}

/// `Add.could_extract_minus_sign()`: `_could_extract_minus_sign(self)`.
pub fn could_extract_minus_sign(e: &Expr) -> bool {
    // We choose the one with less arguments with minus signs
    let negative_args = e
        .args()
        .iter()
        .filter(|i| i.could_extract_minus_sign())
        .count();
    let positive_args = e.args().len() - negative_args;
    if positive_args > negative_args {
        false
    } else if positive_args < negative_args {
        true
    } else {
        // choose based on .sort_key() to prefer
        // x - 1 instead of 1 - x and
        // 3 - sqrt(2) instead of -3 + sqrt(2)
        use crate::compare::{key_lt, sort_key};
        key_lt(&sort_key(e, None), &sort_key(&ops::neg(e), None))
    }
}

/// `type(e)._prop_handler[fact](e)` for an Add.
pub(crate) fn handler(e: &Expr, fact: Fact) -> Option<bool> {
    let group = |f: fn(&Expr) -> Option<bool>| fuzzy_group(e.args().iter().map(f), true);
    match fact {
        Fact::Real => group(Expr::is_real),
        Fact::ExtendedReal => group(Expr::is_extended_real),
        Fact::Complex => group(Expr::is_complex),
        Fact::Antihermitian => group(Expr::is_antihermitian),
        Fact::Finite => group(Expr::is_finite),
        Fact::Hermitian => group(Expr::is_hermitian),
        Fact::Integer => group(Expr::is_integer),
        Fact::Rational => group(Expr::is_rational),
        Fact::Algebraic => group(Expr::is_algebraic),
        Fact::Commutative => fuzzy_group(e.args().iter().map(Expr::is_commutative), false),
        Fact::Infinite => eval_is_infinite(e),
        Fact::Imaginary => eval_is_imaginary(e),
        Fact::Zero => eval_is_zero(e),
        Fact::Odd => eval_is_odd(e),
        Fact::Irrational => eval_is_irrational(e),
        Fact::ExtendedPositive => eval_is_extended_positive(e),
        Fact::ExtendedNonnegative => eval_is_extended_nonneg_nonpos(e, true),
        Fact::ExtendedNonpositive => eval_is_extended_nonneg_nonpos(e, false),
        Fact::ExtendedNegative => eval_is_extended_negative(e),
        _ => unreachable!("Add has no handler for {fact:?}"),
    }
}

/// `Add._eval_is_infinite`.
fn eval_is_infinite(e: &Expr) -> Option<bool> {
    let mut sawinf = false;
    for a in e.args() {
        match a.is_infinite() {
            None => return None,
            Some(true) => {
                // infinite+infinite might not be infinite
                if sawinf {
                    return None;
                }
                sawinf = true;
            }
            Some(false) => {}
        }
    }
    Some(sawinf)
}

/// `a.as_coeff_mul(S.ImaginaryUnit)` for a Mul `a`: the factors without
/// `I` (as a Mul) and those with it.
fn coeff_mul_i(a: &Expr) -> (Expr, Vec<Expr>) {
    let i = s::imaginary_unit();
    let (l1, l2): (Vec<Expr>, Vec<Expr>) = a.args().iter().cloned().partition(|x| x.has(&i));
    (crate::mul::new_rawargs(a, l2), l1)
}

/// `Add._eval_is_imaginary`.
fn eval_is_imaginary(e: &Expr) -> Option<bool> {
    let mut nz = Vec::new();
    let mut im_i = Vec::new();
    let i = s::imaginary_unit();
    for a in e.args() {
        if a.is_extended_real() == Some(true) {
            match a.is_zero() {
                Some(true) => {}
                Some(false) => nz.push(a.clone()),
                None => return None,
            }
        } else if a.is_imaginary() == Some(true) {
            im_i.push(ops::mul(a, &i));
        } else if a.is_Mul() && a.args().contains(&i) {
            let (coeff, ai) = coeff_mul_i(a);
            if ai == [i.clone()] && coeff.is_extended_real() == Some(true) {
                im_i.push(ops::neg(&coeff));
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    let b = e.func(nz);
    if b != *e {
        if b.is_zero() == Some(true) {
            return fuzzy_not(e.func(im_i).is_zero());
        } else if b.is_zero() == Some(false) {
            return Some(false);
        }
    }
    None
}

/// `Add._eval_is_zero`.
fn eval_is_zero(e: &Expr) -> Option<bool> {
    if e.is_commutative() == Some(false) {
        // issue 10528: there is no way to know if a nc symbol
        // is zero or not
        return None;
    }
    let mut nz = Vec::new();
    let mut z = 0;
    let mut im_or_z = false;
    let mut im = 0;
    let i = s::imaginary_unit();
    for a in e.args() {
        if a.is_extended_real() == Some(true) {
            match a.is_zero() {
                Some(true) => z += 1,
                Some(false) => nz.push(a.clone()),
                None => return None,
            }
        } else if a.is_imaginary() == Some(true) {
            im += 1;
        } else if a.is_Mul() && a.args().contains(&i) {
            let (coeff, ai) = coeff_mul_i(a);
            if ai == [i.clone()] && coeff.is_extended_real() == Some(true) {
                im_or_z = true;
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    let n = e.args().len();
    if z == n {
        return Some(true);
    }
    if nz.is_empty() || nz.len() == n {
        return None;
    }
    let b = e.func(nz);
    if b.is_zero() == Some(true) && !im_or_z {
        if im == 0 {
            return Some(true);
        } else if im == 1 {
            return Some(false);
        }
    }
    if b.is_zero() == Some(false) {
        return Some(false);
    }
    None
}

/// `Add._eval_is_odd`.
fn eval_is_odd(e: &Expr) -> Option<bool> {
    let l: Vec<Expr> = e
        .args()
        .iter()
        .filter(|f| f.is_even() != Some(true))
        .cloned()
        .collect();
    if l.is_empty() {
        return Some(false);
    }
    if l[0].is_odd() == Some(true) {
        return new_rawargs(e, l[1..].to_vec()).is_even();
    }
    None
}

/// `Add._eval_is_irrational`.
fn eval_is_irrational(e: &Expr) -> Option<bool> {
    for (k, t) in e.args().iter().enumerate() {
        let a = t.is_irrational();
        if a == Some(true) {
            // others = list(self.args); others.remove(t)
            let first = e.args().iter().position(|x| x == t).unwrap_or(k);
            let others = e.args().iter().enumerate().filter(|(j, _)| *j != first);
            if others
                .into_iter()
                .all(|(_, x)| x.is_rational() == Some(true))
            {
                return Some(true);
            }
            return None;
        }
        a?;
    }
    Some(false)
}

/// The `_monotonic_sign` shortcut at the top of `Add`'s sign handlers:
/// `Some(true)` when it proves `test(self)`, `None` to carry on. `test`
/// is the fact asked about; `a_test` is the extra condition on the
/// non-constant part (`None` when the handler has none).
fn monotonic_shortcut(
    e: &Expr,
    test: fn(&Expr) -> Option<bool>,
    a_test: Option<fn(&Expr) -> Option<bool>>,
    a_first: bool,
) -> bool {
    let (c, a) = e.as_coeff_Add(false);
    if c.is_zero() == Some(true) {
        return false;
    }
    // (the nonnegative/nonpositive handlers test `a` before computing v)
    if a_first && a_test.is_some_and(|t| t(&a) != Some(true)) {
        return false;
    }
    if let Some(v) = crate::exprtools::monotonic_sign(&a) {
        let s = ops::add(&v, &c);
        if s != *e
            && test(&s) == Some(true)
            && (a_first || a_test.is_none_or(|t| t(&a) == Some(true)))
        {
            return true;
        }
        if e.free_symbols().len() == 1
            && let Some(v) = crate::exprtools::monotonic_sign(e)
            && v != *e
            && test(&v) == Some(true)
        {
            return true;
        }
    }
    false
}

/// `Add._eval_is_extended_positive`.
fn eval_is_extended_positive(e: &Expr) -> Option<bool> {
    if e.is_number() {
        return crate::functions::expr_eval_is_extended_positive_negative(e);
    }
    if monotonic_shortcut(
        e,
        Expr::is_extended_positive,
        Some(Expr::is_extended_nonnegative),
        false,
    ) {
        return Some(true);
    }
    sign_loop(
        e,
        Expr::is_extended_positive,
        Expr::is_extended_nonnegative,
        Expr::is_extended_nonpositive,
    )
}

/// `Add._eval_is_extended_negative`.
fn eval_is_extended_negative(e: &Expr) -> Option<bool> {
    if e.is_number() {
        return crate::functions::expr_eval_is_extended_positive_negative(e);
    }
    if monotonic_shortcut(
        e,
        Expr::is_extended_negative,
        Some(Expr::is_extended_nonpositive),
        false,
    ) {
        return Some(true);
    }
    sign_loop(
        e,
        Expr::is_extended_negative,
        Expr::is_extended_nonpositive,
        Expr::is_extended_nonnegative,
    )
}

/// The term-by-term loop shared by `Add._eval_is_extended_positive`
/// (`strict` = positive, `weak` = nonnegative, `opposite` = nonpositive)
/// and `_eval_is_extended_negative` (the mirror image).
fn sign_loop(
    e: &Expr,
    strict: fn(&Expr) -> Option<bool>,
    weak: fn(&Expr) -> Option<bool>,
    opposite: fn(&Expr) -> Option<bool>,
) -> Option<bool> {
    let (mut pos, mut nonneg, mut nonpos, mut unknown_sign) = (false, false, false, false);
    // saw_INF: a set of True/False/None
    let mut saw_inf: Vec<Option<bool>> = Vec::new();
    let args: Vec<&Expr> = e
        .args()
        .iter()
        .filter(|a| a.is_zero() != Some(true))
        .collect();
    if args.is_empty() {
        return Some(false);
    }
    for a in args {
        let ispos = strict(a);
        let infinite = a.is_infinite();
        if infinite == Some(true) {
            let v = fuzzy_or([ispos, weak(a)]);
            if !saw_inf.contains(&v) {
                saw_inf.push(v);
            }
            if saw_inf.contains(&Some(true)) && saw_inf.contains(&Some(false)) {
                return None;
            }
        }
        if ispos == Some(true) {
            pos = true;
            continue;
        } else if weak(a) == Some(true) {
            nonneg = true;
            continue;
        } else if opposite(a) == Some(true) {
            nonpos = true;
            continue;
        }

        infinite?;
        unknown_sign = true;
    }

    if !saw_inf.is_empty() {
        if saw_inf.len() > 1 {
            return None;
        }
        return saw_inf[0];
    } else if unknown_sign {
        return None;
    } else if !nonpos && !nonneg && pos {
        return Some(true);
    } else if !nonpos && pos {
        return Some(true);
    } else if !pos && !nonneg {
        return Some(false);
    }
    None
}

/// `Add._eval_is_extended_nonnegative` (`nonneg`) and
/// `_eval_is_extended_nonpositive`.
fn eval_is_extended_nonneg_nonpos(e: &Expr, nonneg: bool) -> Option<bool> {
    if !e.is_number() {
        let test = if nonneg {
            Expr::is_extended_nonnegative
        } else {
            Expr::is_extended_nonpositive
        };
        if monotonic_shortcut(e, test, Some(test), true) {
            return Some(true);
        }
    }
    None
}
