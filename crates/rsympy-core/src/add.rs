//! `sympy.core.add`: `Add`, and `AssocOp.__new__` (shared with `Mul`).
//!
//! Ported from `sympy/core/add.py` and `sympy/core/operations.py`. The
//! `_eval_is_*` handlers live in [`handler`] at the bottom.

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
    let _ = (e, fact);
    not_ported("Add assumptions")
}
