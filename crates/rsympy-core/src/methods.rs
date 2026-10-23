//! `Expr` methods that SymPy overrides per class, dispatched on the node's
//! class the way Python's MRO lookup would (`Mul.as_coeff_Mul` before
//! `Expr.as_coeff_Mul`, and so on). The per-class bodies live with their
//! class; the `Expr`/`Basic` defaults live here.

use std::collections::HashSet;

use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::Expr;
use crate::generated::{ClassId as C, Method, Tag};
use crate::numbers::s;

impl Expr {
    /// `bool(self)`: Python's truthiness of a SymPy object. `Basic` has no
    /// `__bool__`, so most objects are true.
    pub fn truthy(&self) -> bool {
        match self.method_owner(Method::Bool) {
            "Zero" | "BooleanFalse" => false,
            "BooleanTrue" => true,
            "Float" => *self.as_float().expect("Float").0 != rsympy_mpf::FZERO,
            "Relational" => raise(
                ErrorKind::TypeError,
                "cannot determine truth value of Relational",
            ),
            "" => match self.method_owner(Method::Len) {
                "" => true,
                "Tuple" | "FiniteSet" => !self.args().is_empty(),
                "EmptySet" => false,
                o => not_ported(format!("{o}.__len__ for truthiness")),
            },
            o => not_ported(format!("{o}.__bool__")),
        }
    }

    /// `self.is_comparable`.
    pub fn is_comparable(&self) -> bool {
        match self.method_owner(Method::IsComparable) {
            "Zero" | "Infinity" | "NegativeInfinity" => true,
            // Symbol.is_comparable = False
            "NaN" | "Symbol" => false,
            "Basic" => {
                if self.is_extended_real() == Some(false) {
                    return false;
                }
                if !self.is_number() {
                    return false;
                }
                // n, i = self.as_real_imag(), evaluated to 2 digits
                if self.is_Rational() {
                    return true;
                }
                if let Some((_, prec)) = self.as_float() {
                    return prec != 1;
                }
                if self.is_NumberSymbol() {
                    // re(c) = c and im(c) = 0 for the real constants, and c
                    // evaluates to a Float with 2 digits.
                    return true;
                }
                not_ported(format!("{}.is_comparable (needs evalf)", self.class_name()))
            }
            o => not_ported(format!("{o}.is_comparable")),
        }
    }
}

/// `x in _illegal`: nan, oo, -oo or zoo.
fn is_illegal(e: &Expr) -> bool {
    matches!(
        e.cls(),
        C::NaN | C::Infinity | C::NegativeInfinity | C::ComplexInfinity
    )
}

/// `a.args_cnc(cset=True, warn=warn)`: the commutative factors as a set
/// (raising ValueError for repeated ones when `warn`) and the others.
fn args_cnc_set(a: &Expr, warn: bool) -> (HashSet<Expr>, Vec<Expr>) {
    let (c, nc) = a.args_cnc(true);
    let clen = c.len();
    let set: HashSet<Expr> = c.iter().cloned().collect();
    if clen > 0 && warn && set.len() != clen {
        let mut rep: Vec<String> = Vec::new();
        let mut shown = HashSet::new();
        for ci in &c {
            if a.args().iter().filter(|x| *x == ci).count() > 1 && shown.insert(ci.clone()) {
                rep.push(ci.to_string());
            }
        }
        raise(
            ErrorKind::ValueError,
            format!("repeated commutative arguments: [{}]", rep.join(", ")),
        );
    }
    (set, nc)
}

/// `find(l, sub, first)` in `Expr.coeff`: where `sub` appears in `l`, the
/// first occurrence from the left when `first`, else the last.
fn coeff_find(l: &[Expr], sub: &[Expr], first: bool) -> Option<usize> {
    if sub.is_empty() || l.is_empty() || sub.len() > l.len() {
        return None;
    }
    let n = sub.len();
    let (l2, sub2): (Vec<&Expr>, Vec<&Expr>) = if first {
        (l.iter().collect(), sub.iter().collect())
    } else {
        (l.iter().rev().collect(), sub.iter().rev().collect())
    };
    let i = (0..l2.len() - n + 1).find(|&i| (0..n).all(|j| l2[i + j] == sub2[j]))?;
    Some(if first { i } else { l.len() - (i + n) })
}

/// `incommon(l1, l2)` in `Expr.coeff`: the common leading part.
fn coeff_incommon(l1: Vec<Expr>, l2: Vec<Expr>) -> Vec<Expr> {
    if l1.is_empty() || l2.is_empty() {
        return vec![];
    }
    let n = l1.len().min(l2.len());
    for i in 0..n {
        if l1[i] != l2[i] {
            return l1[..i].to_vec();
        }
    }
    l1
}

/// Python's `l[:stop]` for a possibly negative `stop`.
fn py_slice_to(l: &[Expr], stop: isize) -> &[Expr] {
    let n = l.len() as isize;
    let stop = if stop < 0 {
        (n + stop).max(0)
    } else {
        stop.min(n)
    };
    &l[..stop as usize]
}

impl Expr {
    /// `self.as_independent(*deps, as_Add=as_add)`: `(independent,
    /// dependent)` parts of a sum (`as_add`, the default for an Add) or a
    /// product, the dependent ones being those that have one of `deps`
    /// (a symbol in `deps` counts only when it is free). `as_add: None` is
    /// the default.
    pub fn as_independent(
        &self,
        deps: &[crate::basic::Pattern],
        as_add: Option<bool>,
    ) -> (Expr, Expr) {
        use crate::basic::Pattern;

        if !self.is_a(Tag::Expr) {
            raise(
                ErrorKind::AttributeError,
                format!(
                    "'{}' object has no attribute 'as_independent'",
                    self.class_name()
                ),
            );
        }
        if self.is_cls(C::Zero) {
            return (self.clone(), self.clone());
        }

        let want_add = as_add.unwrap_or(self.is_Add());

        // sift out deps into symbolic and other and ignore
        // all symbols but those that are in the free symbols
        let mut sym: Vec<Expr> = Vec::new();
        let mut other: Vec<Pattern> = Vec::new();
        for d in deps {
            match d {
                Pattern::Expr(e) if e.is_Symbol() => {
                    if !sym.contains(e) {
                        sym.push(e.clone());
                    }
                }
                d => other.push(d.clone()),
            }
        }

        // return the standard has() if there are no literal symbols, else
        // check to see that symbol-deps are in the free symbols.
        let has = |e: &Expr| -> bool {
            let has_other = crate::basic::has(e, &other);
            if sym.is_empty() {
                return has_other;
            }
            has_other || {
                let free: Vec<Pattern> = e
                    .free_symbols()
                    .into_iter()
                    .filter(|s| sym.contains(s))
                    .map(Pattern::Expr)
                    .collect();
                crate::basic::has(e, &free)
            }
        };

        let (func_add, func_mul) = (self.is_Add(), self.is_Mul());
        let want_is_func = if want_add { func_add } else { func_mul };
        if !want_is_func || !func_add && !func_mul {
            let identity = if want_add { s::zero() } else { s::one() };
            return if has(self) {
                (identity, self.clone())
            } else {
                (self.clone(), identity)
            };
        }
        let (args, nc) = if func_add {
            (self.args().to_vec(), vec![])
        } else {
            self.args_cnc(true)
        };

        let (mut depend, mut indep): (Vec<Expr>, Vec<Expr>) = args.into_iter().partition(&has);
        if func_add {
            // all terms were treated as commutative
            return (crate::add::add(indep), crate::add::unevaluated_add(depend));
        }
        // handle noncommutative by stopping at first dependent term
        for (i, n) in nc.iter().enumerate() {
            if has(n) {
                depend.extend(nc[i..].iter().cloned());
                break;
            }
            indep.push(n.clone());
        }
        (crate::mul::mul(indep), crate::mul::unevaluated_mul(depend))
    }

    /// `self.coeff(x, n, right)`: the coefficient of the terms of `self`
    /// that contain `x**n`, without other powers of `x` (for
    /// non-commutative factors, what is left or, with `right`, right of
    /// `x**n`).
    pub fn coeff(&self, x: &Expr, n: i64, right: bool) -> Expr {
        self.coeff_first(x, n, right, true)
    }

    fn coeff_first(&self, x: &Expr, n: i64, right: bool, first: bool) -> Expr {
        use crate::add::add;
        use crate::basic::Pattern;
        use crate::mul::mul;
        use crate::ops;

        if !self.is_a(Tag::Expr) {
            raise(
                ErrorKind::AttributeError,
                format!("'{}' object has no attribute 'coeff'", self.class_name()),
            );
        }
        if !x.truthy() {
            return s::zero();
        }

        if x == self {
            return if n == 1 { s::one() } else { s::zero() };
        }

        if x.is_cls(C::One) {
            let co2: Vec<Expr> = self
                .add_args()
                .into_iter()
                .filter(|a| a.as_coeff_Mul(false).0.is_cls(C::One))
                .collect();
            if co2.is_empty() {
                return s::zero();
            }
            return add(co2);
        }

        if n == 0 {
            if x.is_Add() && self.is_Add() {
                let c = self.coeff_first(x, 1, right, true);
                if !c.truthy() {
                    return s::zero();
                }
                let terms = c
                    .add_args()
                    .iter()
                    .map(|a| {
                        if right {
                            ops::mul(x, a)
                        } else {
                            ops::mul(a, x)
                        }
                    })
                    .collect();
                return ops::sub(self, &add(terms));
            }
            return self
                .as_independent(&[Pattern::Expr(x.clone())], Some(true))
                .0;
        }

        // continue with the full method, looking for this power of x:
        let x = ops::pow(x, &Expr::integer(n));

        let args = self.add_args();
        let self_c = self.is_commutative() == Some(true);
        let x_c = x.is_commutative() == Some(true);
        if self_c && !x_c {
            return s::zero();
        }
        if first && self.is_Add() && !self_c && !x_c {
            // get the part that depends on x exactly
            let xargs = x.mul_args();
            let dep = self.as_independent(&[Pattern::Expr(x.clone())], None).1;
            let d = add(dep
                .add_args()
                .into_iter()
                .filter(|i| {
                    let margs = i.mul_args();
                    xargs.iter().all(|xi| margs.contains(xi))
                })
                .collect());
            let rv = d.coeff_first(&x, 1, right, false);
            if !rv.is_Add() || !right {
                return rv;
            }
            let (c_part, nc_part): (Vec<Vec<Expr>>, Vec<Vec<Expr>>) =
                rv.args().iter().map(|i| i.args_cnc(true)).unzip();
            // has_variety(c_part)
            if c_part.iter().any(|i| *i != c_part[0]) {
                return rv;
            }
            return add(nc_part
                .into_iter()
                .map(|i| crate::mul::from_args(i, None))
                .collect());
        }

        let one_c = self_c || x_c;
        let (xargs, nx) = args_cnc_set(&x, !x_c);
        // find the parts that pass the commutative terms
        let mut co2: Vec<Expr> = Vec::new();
        let mut co: Vec<(HashSet<Expr>, Vec<Expr>)> = Vec::new();
        for a in &args {
            let (margs, nc) = args_cnc_set(a, !self_c);
            if xargs.len() > margs.len() {
                continue;
            }
            let resid: HashSet<Expr> = margs.difference(&xargs).cloned().collect();
            if resid.len() + xargs.len() == margs.len() {
                if one_c {
                    let mut f: Vec<Expr> = resid.into_iter().collect();
                    f.extend(nc);
                    co2.push(mul(f));
                } else {
                    co.push((resid, nc));
                }
            }
        }
        if one_c {
            if co2.is_empty() {
                return s::zero();
            }
            return add(co2);
        }
        // both nc: now check the non-comm parts
        if co.is_empty() {
            return s::zero();
        }
        let set_mul = |r: &HashSet<Expr>, rest: &[Expr]| -> Expr {
            let mut f: Vec<Expr> = r.iter().cloned().collect();
            f.extend_from_slice(rest);
            mul(f)
        };
        if co.iter().all(|(_, n)| *n == co[0].1)
            && let Some(ii) = coeff_find(&co[0].1, &nx, right)
        {
            if !right {
                return mul(vec![
                    add(co.iter().map(|(r, _)| set_mul(r, &[])).collect()),
                    mul(co[0].1[..ii].to_vec()),
                ]);
            } else {
                return mul(co[0].1[ii + nx.len()..].to_vec());
            }
        }
        let beg = co
            .iter()
            .map(|(_, n)| n.clone())
            .reduce(coeff_incommon)
            .expect("co is not empty");
        if !beg.is_empty()
            && let Some(ii) = coeff_find(&beg, &nx, right)
        {
            if !right {
                let mut gcdc = co[0].0.clone();
                for c in &co[1..] {
                    gcdc = gcdc.intersection(&c.0).cloned().collect();
                    if gcdc.is_empty() {
                        break;
                    }
                }
                return set_mul(&gcdc, &beg[..ii]);
            } else {
                let m = ii + nx.len();
                return add(co
                    .iter()
                    .map(|(r, n)| set_mul(r, &n[m.min(n.len())..]))
                    .collect());
            }
        }
        let mut end = co
            .iter()
            .map(|(_, n)| n.iter().rev().cloned().collect::<Vec<_>>())
            .reduce(coeff_incommon)
            .expect("co is not empty");
        end.reverse();
        if !end.is_empty()
            && let Some(ii) = coeff_find(&end, &nx, right)
        {
            if !right {
                let stop = -(end.len() as isize) + ii as isize;
                return add(co
                    .iter()
                    .map(|(r, n)| set_mul(r, py_slice_to(n, stop)))
                    .collect());
            } else {
                return mul(end[ii + nx.len()..].to_vec());
            }
        }
        // look for single match
        let mut hit: Option<(usize, &HashSet<Expr>, &Vec<Expr>)> = None;
        let mut broke = false;
        for (r, n) in &co {
            if let Some(ii) = coeff_find(n, &nx, right) {
                if hit.is_none() {
                    hit = Some((ii, r, n));
                } else {
                    broke = true;
                    break;
                }
            }
        }
        if !broke && let Some((ii, r, n)) = hit {
            if !right {
                return set_mul(r, &n[..ii]);
            } else {
                return mul(n[ii + nx.len()..].to_vec());
            }
        }

        s::zero()
    }

    /// `self.is_polynomial(*syms)`: whether `self` is a polynomial in
    /// `syms` (all its free symbols when none are given). `None` when an
    /// `Expr` subclass without its own test cannot tell.
    pub fn is_polynomial(&self, syms: &[Expr]) -> Option<bool> {
        self.expr_default(Method::EvalIsPolynomial, "is_polynomial");
        let syms = if !syms.is_empty() {
            let mut v: Vec<Expr> = Vec::new();
            for s in syms {
                if !v.contains(s) {
                    v.push(s.clone());
                }
            }
            v
        } else {
            let syms = self.free_symbols();
            if syms.is_empty() {
                return Some(true);
            }
            syms
        };
        self.eval_is_polynomial(&syms)
    }

    /// `self._eval_is_polynomial(syms)`.
    pub fn eval_is_polynomial(&self, syms: &[Expr]) -> Option<bool> {
        use crate::basic::Pattern;

        let patterns = || -> Vec<Pattern> { syms.iter().cloned().map(Pattern::Expr).collect() };
        match self.method_owner(Method::EvalIsPolynomial) {
            "AtomicExpr" => Some(true),
            "Add" => crate::add::eval_is_polynomial(self, syms),
            "Mul" => crate::mul::eval_is_polynomial(self, syms),
            "Pow" => crate::power::eval_is_polynomial(self, syms),
            "Expr" => {
                if syms.contains(self) {
                    return Some(true);
                }
                if !crate::basic::has_free(self, &patterns()) {
                    // constant polynomial
                    return Some(true);
                }
                // subclasses should return True or False
                None
            }
            "" => raise(
                ErrorKind::AttributeError,
                format!(
                    "'{}' object has no attribute '_eval_is_polynomial'",
                    self.class_name()
                ),
            ),
            o => not_ported(format!("{o}._eval_is_polynomial")),
        }
    }

    /// `self.is_rational_function(*syms)`: whether `self` is a ratio of
    /// polynomials in `syms` (all its free symbols when none are given).
    pub fn is_rational_function(&self, syms: &[Expr]) -> Option<bool> {
        self.expr_default(Method::EvalIsRationalFunction, "is_rational_function");
        let syms = if !syms.is_empty() {
            let mut v: Vec<Expr> = Vec::new();
            for s in syms {
                if !v.contains(s) {
                    v.push(s.clone());
                }
            }
            v
        } else {
            let syms = self.free_symbols();
            if syms.is_empty() {
                return Some(!is_illegal(self));
            }
            syms
        };
        self.eval_is_rational_function(&syms)
    }

    /// `self._eval_is_rational_function(syms)`.
    pub fn eval_is_rational_function(&self, syms: &[Expr]) -> Option<bool> {
        match self.method_owner(Method::EvalIsRationalFunction) {
            "AtomicExpr" => Some(!is_illegal(self)),
            "Add" => crate::add::eval_is_rational_function(self, syms),
            "Mul" => crate::mul::eval_is_rational_function(self, syms),
            "Pow" => crate::power::eval_is_rational_function(self, syms),
            "Expr" => {
                if syms.contains(self) {
                    return Some(true);
                }
                if !crate::basic::has_xfree(self, syms) {
                    return Some(true);
                }
                // subclasses should return True or False
                None
            }
            "" => raise(
                ErrorKind::AttributeError,
                format!(
                    "'{}' object has no attribute '_eval_is_rational_function'",
                    self.class_name()
                ),
            ),
            o => not_ported(format!("{o}._eval_is_rational_function")),
        }
    }
}

impl Expr {
    /// The fallback of a method dispatched below: `Expr`'s default, unless
    /// the class has no such method (a non-`Expr` Basic such as a Boolean
    /// or a Tuple), where Python raises AttributeError.
    fn expr_default(&self, method: Method, name: &str) {
        if self.method_owner(method).is_empty() {
            raise(
                ErrorKind::AttributeError,
                format!("'{}' object has no attribute '{name}'", self.class_name()),
            );
        }
    }
}

#[allow(non_snake_case)]
impl Expr {
    /// `self.as_coeff_Mul(rational)`: `(coeff, rest)` with a Number coeff.
    pub fn as_coeff_Mul(&self, rational: bool) -> (Expr, Expr) {
        if self.is_Rational() {
            return (self.clone(), s::one());
        }
        if self.is_Number() {
            return if rational {
                (s::one(), self.clone())
            } else {
                (self.clone(), s::one())
            };
        }
        if self.is_Mul() {
            return crate::mul::as_coeff_Mul(self, rational);
        }
        self.expr_default(Method::AsCoeff_Mul, "as_coeff_Mul");
        (s::one(), self.clone())
    }

    /// `self.as_coeff_Add(rational)`: `(coeff, rest)` with a Number coeff.
    pub fn as_coeff_Add(&self, rational: bool) -> (Expr, Expr) {
        if self.is_Rational() {
            return (self.clone(), s::zero());
        }
        if self.is_Number() {
            return if rational {
                (s::zero(), self.clone())
            } else {
                (self.clone(), s::zero())
            };
        }
        if self.is_Add() {
            return crate::add::as_coeff_Add(self, rational);
        }
        self.expr_default(Method::AsCoeff_Add, "as_coeff_Add");
        (s::zero(), self.clone())
    }

    /// `self.as_coeff_mul()` (no deps, `rational=True`).
    pub fn as_coeff_mul(&self) -> (Expr, Vec<Expr>) {
        if self.is_Number() {
            if self.is_Rational() {
                return (self.clone(), vec![]);
            }
            if self.is_negative() == Some(true) {
                return (s::negative_one(), vec![crate::ops::neg(self)]);
            }
            return (s::one(), vec![self.clone()]);
        }
        if self.is_Mul() {
            return crate::mul::as_coeff_mul(self, true);
        }
        self.expr_default(Method::AsCoeffMul, "as_coeff_mul");
        (s::one(), vec![self.clone()])
    }

    /// `self.as_coeff_add()` (no deps).
    pub fn as_coeff_add(&self) -> (Expr, Vec<Expr>) {
        if self.is_Number() {
            if self.is_Rational() {
                return (self.clone(), vec![]);
            }
            return (s::zero(), vec![self.clone()]);
        }
        if self.is_Add() {
            return crate::add::as_coeff_add(self);
        }
        self.expr_default(Method::AsCoeffAdd, "as_coeff_add");
        (s::zero(), vec![self.clone()])
    }

    /// `self.as_base_exp()`: `b, e` with `self == b**e`.
    pub fn as_base_exp(&self) -> (Expr, Expr) {
        match self.cls() {
            C::Pow => crate::power::as_base_exp(self),
            C::Mul => crate::mul::as_base_exp(self),
            C::ImaginaryUnit => (s::negative_one(), s::half()),
            C::Exp => crate::functions::exp_as_base_exp(self),
            _ => {
                self.expr_default(Method::AsBaseExp, "as_base_exp");
                (self.clone(), s::one())
            }
        }
    }

    /// `self.as_numer_denom()`.
    pub fn as_numer_denom(&self) -> (Expr, Expr) {
        match self.cls() {
            C::Integer | C::Zero | C::One | C::NegativeOne => (self.clone(), s::one()),
            C::Rational | C::Half => (Expr::integer(self.p()), Expr::integer(self.q())),
            C::Add => crate::add::as_numer_denom(self),
            C::Mul => crate::mul::as_numer_denom(self),
            C::Pow => crate::power::as_numer_denom(self),
            C::Exp => crate::functions::exp_as_numer_denom(self),
            _ => {
                self.expr_default(Method::AsNumerDenom, "as_numer_denom");
                (self.clone(), s::one())
            }
        }
    }

    /// `self.could_extract_minus_sign()`.
    pub fn could_extract_minus_sign(&self) -> bool {
        if self.is_Number() {
            return self.is_extended_negative() == Some(true);
        }
        match self.cls() {
            C::Add => crate::add::could_extract_minus_sign(self),
            C::Mul => crate::mul::could_extract_minus_sign(self),
            _ => false,
        }
    }

    /// `Mul.make_args(self)`.
    pub fn mul_args(&self) -> Vec<Expr> {
        if self.is_Mul() {
            self.args().to_vec()
        } else {
            vec![self.clone()]
        }
    }

    /// `Add.make_args(self)`.
    pub fn add_args(&self) -> Vec<Expr> {
        if self.is_Add() {
            self.args().to_vec()
        } else {
            vec![self.clone()]
        }
    }

    /// `self.args_cnc(split_1=split_1)`: commutative and non-commutative
    /// factors, keeping order.
    pub fn args_cnc(&self, split_1: bool) -> (Vec<Expr>, Vec<Expr>) {
        let args = self.mul_args();
        let split = args
            .iter()
            .position(|a| a.is_commutative() != Some(true))
            .unwrap_or(args.len());
        let mut c = args[..split].to_vec();
        let nc = args[split..].to_vec();
        if split_1
            && !c.is_empty()
            && c[0].is_Number()
            && c[0].is_extended_negative() == Some(true)
            && !c[0].is_cls(C::NegativeOne)
        {
            let n = crate::ops::neg(&c[0]);
            c.splice(0..1, [s::negative_one(), n]);
        }
        (c, nc)
    }

    /// `self.free_symbols`, as a list in first-seen order (SymPy returns a
    /// set; callers that iterate it in SymPy depend on hash order).
    pub fn free_symbols(&self) -> Vec<Expr> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        collect_free_symbols(self, &mut out, &mut seen);
        out
    }

    /// `self.has(pattern)` for an expression pattern: whether `pattern`
    /// occurs as a subtree (for an Add or Mul pattern, also as part of a
    /// node of its class: see [`crate::basic::has`]).
    pub fn has(&self, pattern: &Expr) -> bool {
        crate::basic::has(self, &[crate::basic::Pattern::Expr(pattern.clone())])
    }

    /// `self.has(cls)`: whether any subtree is an instance of `tag`.
    pub fn has_a(&self, tag: Tag) -> bool {
        self.is_a(tag) || self.args().iter().any(|a| a.has_a(tag))
    }

    /// `self.has(cls)` for a concrete class.
    pub fn has_cls(&self, cls: C) -> bool {
        self.cls() == cls || self.args().iter().any(|a| a.has_cls(cls))
    }

    /// `preorder_traversal(self)`.
    pub fn preorder(&self) -> Vec<Expr> {
        let mut out = Vec::new();
        let mut stack = vec![self.clone()];
        while let Some(e) = stack.pop() {
            for a in e.args().iter().rev() {
                stack.push(a.clone());
            }
            out.push(e);
        }
        out
    }
}

fn collect_free_symbols(e: &Expr, out: &mut Vec<Expr>, seen: &mut HashSet<Expr>) {
    match e.method_owner(Method::FreeSymbols) {
        "Basic" | "Symbol" | "Lambda" | "ExprWithLimits" | "Integral" | "Derivative" | "Subs" => {}
        o => crate::error::not_ported(format!("{o}.free_symbols")),
    }
    if e.is_Symbol() {
        if seen.insert(e.clone()) {
            out.push(e.clone());
        }
        return;
    }
    if e.is_a(Tag::ExprWithLimits) || matches!(e.cls(), C::Lambda | C::Derivative | C::Subs) {
        crate::error::not_ported(format!("{}.free_symbols", e.class_name()));
    }
    for a in e.args() {
        collect_free_symbols(a, out, seen);
    }
}
