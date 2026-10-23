//! `sympy.core.basic`: the `Basic` methods that search and rebuild a tree.
//!
//! Ported from `sympy/core/basic.py`: `has` (with `_has` and
//! `AssocOp._has_matcher`), `has_free`, `has_xfree`, `atoms`, `is_same`
//! (`_aresame`), `subs` (with `_subs`, its `fallback` and the
//! `_eval_subs` dispatch), `xreplace` and `_xreplace`. `free_symbols`
//! lives in `methods.rs`; the tree walks are in [`crate::traversal`].
//!
//! SymPy returns sets from `atoms` and `free_symbols`; rsympy returns them
//! as vectors in first-seen order, because the iteration order of a Python
//! set of expressions depends on the hash seed.

use std::collections::HashSet;
use std::sync::Arc;

use crate::compare::{SortKey, default_sort_key, nodes, ordered_with};
use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::{Expr, UndefData};
use crate::facts::Fact;
use crate::generated::{ClassId as C, Method, Tag};
use crate::numbers::s;
use crate::ops;
use crate::py::PyDict;
use crate::traversal::{Keys, iterargs, iterfreeargs, postorder_traversal, preorder_traversal};

// ---------------------------------------------------------------------------
// patterns and types

/// An argument of `has` or `atoms`: an expression, or a class (the Python
/// `type` objects SymPy also accepts there).
#[derive(Clone)]
pub enum Pattern {
    /// A SymPy object.
    Expr(Expr),
    /// An abstract class, tested with `isinstance` (`Symbol`, `Number`,
    /// `Function`, ...).
    Tag(Tag),
    /// A class in the class table, tested with `isinstance`.
    Class(C),
    /// An undefined function `Function('f')`, used as a class.
    Undef(Arc<UndefData>),
}

impl std::fmt::Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Expr(e) => write!(f, "Expr({e:?})"),
            Pattern::Tag(t) => write!(f, "Tag({t:?})"),
            Pattern::Class(c) => write!(f, "Class({c:?})"),
            Pattern::Undef(u) => write!(f, "Undef({})", u.name),
        }
    }
}

impl Pattern {
    /// `type(e)`.
    pub fn type_of(e: &Expr) -> Pattern {
        match e.undef_data() {
            Some(u) if e.is_cls(C::AppliedUndef) => Pattern::Undef(u.clone()),
            _ => Pattern::Class(e.cls()),
        }
    }

    /// Whether this is a class (`isinstance(p, type)`).
    pub fn is_type(&self) -> bool {
        !matches!(self, Pattern::Expr(_))
    }

    /// `isinstance(e, cls)` for a class pattern (false for an expression
    /// pattern, which is not a class).
    pub fn isinstance(&self, e: &Expr) -> bool {
        match self {
            Pattern::Expr(_) => false,
            Pattern::Tag(t) => e.is_a(*t),
            Pattern::Undef(u) => e.is_cls(C::AppliedUndef) && e.undef_data() == Some(u),
            Pattern::Class(c) => isinstance_class(e, *c),
        }
    }
}

/// `isinstance(e, cls)` for a class of the class table. The only classes
/// of the table with subclasses in it are `Integer` (`Zero`, `One`,
/// `NegativeOne`), `Rational` (those, `Integer` and `Half`), `Symbol`
/// (`Dummy`, `Wild`), which have tags of their own, `Tuple`
/// (`ExprCondPair`), and the undefined functions, which all share
/// `AppliedUndef`.
fn isinstance_class(e: &Expr, c: C) -> bool {
    match c {
        C::Integer => e.is_a(Tag::Integer),
        C::Rational => e.is_a(Tag::Rational),
        C::Symbol => e.is_a(Tag::Symbol),
        C::Tuple => matches!(e.cls(), C::Tuple | C::ExprCondPair),
        C::AppliedUndef => e.is_a(Tag::AppliedUndef),
        _ => e.cls() == c,
    }
}

// ---------------------------------------------------------------------------
// has

/// `expr.has(*patterns)`: whether any subexpression matches any of the
/// patterns (equal to an expression, an instance of a class, or, for a
/// pattern that is an `AssocOp` like `x + 1`, contained in a node of the
/// same class, as in `(x + y + 1).has(x + 1)`).
pub fn has(expr: &Expr, patterns: &[Pattern]) -> bool {
    has_with(expr, patterns, false)
}

/// `expr.has_free(*patterns)`: like [`has`], but only looking at free
/// subexpressions (`iterfreeargs`).
pub fn has_free(expr: &Expr, patterns: &[Pattern]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    // (a single non-Basic iterable raises TypeError: patterns are always
    // Basic objects or classes here)
    // try quick test first
    let s: Vec<&Pattern> = patterns.iter().collect();
    if has_xfree_patterns(expr, &s) {
        return true;
    }
    // now try matching through slower _has
    has_with(expr, patterns, true)
}

/// `expr.has_xfree(s)`: whether a free subexpression is in the set `s`.
pub fn has_xfree(expr: &Expr, s: &[Expr]) -> bool {
    let set: HashSet<&Expr> = s.iter().collect();
    iterfreeargs(expr).any(|a| set.contains(&a))
}

/// `has_xfree(set(patterns))`: `a in s` is `a == p` for an expression `p`
/// (a class is never equal to an expression).
fn has_xfree_patterns(expr: &Expr, s: &[&Pattern]) -> bool {
    let set: HashSet<&Expr> = s
        .iter()
        .filter_map(|p| match p {
            Pattern::Expr(e) => Some(e),
            _ => None,
        })
        .collect();
    iterfreeargs(expr).any(|a| set.contains(&a))
}

/// `self._has(iterargs, *patterns)` (`iterfreeargs` when `free`).
fn has_with(expr: &Expr, patterns: &[Pattern], free: bool) -> bool {
    let walk = |e: &Expr| -> Box<dyn Iterator<Item = Expr>> {
        if free {
            Box::new(iterfreeargs(e))
        } else {
            Box::new(iterargs(e))
        }
    };
    // separate out types and unhashable objects
    let types: Vec<&Pattern> = patterns.iter().filter(|p| p.is_type()).collect();
    let mut p_set: Vec<&Expr> = Vec::new();
    for p in patterns {
        if let Pattern::Expr(e) = p
            && !p_set.contains(&e)
        {
            p_set.push(e);
        }
    }
    let set: HashSet<&Expr> = p_set.iter().copied().collect();
    for i in walk(expr) {
        if set.contains(&i) {
            return true;
        }
        if types.iter().any(|t| t.isinstance(&i)) {
            return true;
        }
    }

    // use matcher if defined, e.g. operations defines
    // matcher that checks for exact subset containment,
    // (x + y + 1).has(x + 1) -> True
    for i in p_set {
        // types don't have matchers; only AssocOp defines _has_matcher
        if !i.is_a(Tag::AssocOp) {
            continue;
        }
        if walk(expr).any(|arg| has_matcher(i, &arg)) {
            return true;
        }
    }

    // no success
    false
}

/// `_ncsplit(expr)` in `AssocOp._has_matcher`: the commutative args (a
/// set) and the others, in order.
fn ncsplit(expr: &Expr) -> (HashSet<Expr>, Vec<Expr>) {
    let mut c = HashSet::new();
    let mut nc = Vec::new();
    for arg in expr.args() {
        if arg.is_commutative() == Some(true) {
            c.insert(arg.clone());
        } else {
            nc.push(arg.clone());
        }
    }
    (c, nc)
}

/// `pattern._has_matcher()(expr)` for an `AssocOp` pattern: whether
/// `expr` is a node of the same class whose commutative args include the
/// pattern's and whose non-commutative args contain the pattern's as a
/// contiguous run.
fn has_matcher(pattern: &Expr, expr: &Expr) -> bool {
    let (c, nc) = ncsplit(pattern);
    if Pattern::type_of(pattern).isinstance(expr) {
        if expr == pattern {
            return true;
        }
        let (c2, nc2) = ncsplit(expr);
        if c.is_subset(&c2) {
            if nc.is_empty() {
                return true;
            } else if nc.len() <= nc2.len() {
                for i in 0..nc2.len() - nc.len() + 1 {
                    if nc2[i..i + nc.len()] == nc[..] {
                        return true;
                    }
                }
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// atoms

/// `expr.atoms(*types)`: the subexpressions that are instances of any of
/// `types` (an expression stands for its class, `type(e)`), or, with no
/// types, those without args. A set in SymPy; here in preorder, each once.
pub fn atoms(expr: &Expr, types: &[Pattern]) -> Vec<Expr> {
    let nodes = preorder_traversal(expr, Keys::None);
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    if !types.is_empty() {
        let types2: Vec<Pattern> = types
            .iter()
            .map(|t| match t {
                Pattern::Expr(e) => Pattern::type_of(e),
                t => t.clone(),
            })
            .collect();
        for node in nodes {
            if types2.iter().any(|t| t.isinstance(&node)) && seen.insert(node.clone()) {
                out.push(node);
            }
        }
    } else {
        for node in nodes {
            if node.args().is_empty() && seen.insert(node.clone()) {
                out.push(node);
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// is_same

/// `Basic.is_same(a, b)` (`_aresame`) without `approx`: whether `a` and
/// `b` are the same tree, node by node, including each node's class (so
/// `S.Half` and `Float(0.5)` differ).
pub fn is_same(a: &Expr, b: &Expr) -> bool {
    let pa = postorder_traversal(a, Keys::None);
    let pb = postorder_traversal(b, Keys::None);
    // zip_longest: a missing node (`None in t`) means the trees differ
    pa.len() == pb.len() && pa.iter().zip(&pb).all(|(x, y)| node_same(x, y))
}

/// One pair of `is_same`'s loop: `a == b and a.__class__ == b.__class__`
/// (a Number is never the same as a non-Number).
fn node_same(a: &Expr, b: &Expr) -> bool {
    if a.is_a(Tag::Number) && !b.is_a(Tag::Number) {
        return false;
    }
    a == b && a.cls() == b.cls()
}

// ---------------------------------------------------------------------------
// subs

/// The arguments of `Basic.subs`, once Python has unpacked them.
#[derive(Clone, Debug)]
pub enum SubsArgs {
    /// `subs(old, new)`.
    Pair(Expr, Expr),
    /// `subs(iterable)`: `(old, new)` pairs, substituted in the order
    /// given.
    Sequence(Vec<(Expr, Expr)>),
    /// `subs(dict)` (in the dict's order) or `subs(set)`: SymPy sorts the
    /// pairs, most complex `old` first.
    Unordered(Vec<(Expr, Expr)>),
}

/// `sympify_old(old)` in `Basic.subs` for a Python string: `Symbol(old)`.
pub fn sympify_old_str(old: &str) -> Expr {
    crate::symbol::symbol(old)
}

/// `sympify_new(new)` in `Basic.subs` for a Python string:
/// `sympify(new, strict=False)`, which parses it.
pub fn sympify_new_str(new: &str) -> Expr {
    crate::sympify::sympify_str(new, true)
}

/// `expr.subs(arg)` for a single SymPy object `arg`: an iterable one
/// (`Tuple`, `FiniteSet`, ...) holds the `(old, new)` pairs; anything else
/// raises `ValueError`.
pub fn subs_single(expr: &Expr, arg: &Expr, simultaneous: bool) -> Expr {
    let items = match py_iter(arg) {
        Some(items) => items,
        None => raise(
            ErrorKind::ValueError,
            "When a single argument is passed to subs it should be a dictionary of old: new \
             pairs or an iterable of (old, new) tuples.",
        ),
    };
    let mut pairs = Vec::new();
    for item in items {
        // for s1, s2 in items: unpack each item
        let Some(parts) = py_iter(&item) else {
            raise(
                ErrorKind::TypeError,
                format!("cannot unpack non-iterable {} object", item.class_name()),
            )
        };
        match <[Expr; 2]>::try_from(parts) {
            Ok([s1, s2]) => pairs.push((s1, s2)),
            Err(parts) if parts.len() > 2 => raise(
                ErrorKind::ValueError,
                "too many values to unpack (expected 2)",
            ),
            Err(parts) => raise(
                ErrorKind::ValueError,
                format!(
                    "not enough values to unpack (expected 2, got {})",
                    parts.len()
                ),
            ),
        }
    }
    subs(expr, SubsArgs::Sequence(pairs), simultaneous)
}

/// `list(iter(e))` for an iterable SymPy object, `None` when
/// `iterable(e)` is false.
fn py_iter(e: &Expr) -> Option<Vec<Expr>> {
    match e.cls() {
        // Tuple.__iter__, FiniteSet.__iter__ and ExprCondPair.__iter__
        // iterate over args; EmptySet.__iter__ over nothing.
        C::Tuple | C::FiniteSet | C::ExprCondPair | C::EmptySet => Some(e.args().to_vec()),
        C::Union | C::Intersection | C::Complement | C::Range => {
            not_ported(format!("iterating over a {}", e.class_name()))
        }
        _ => None,
    }
}

/// `x in _illegal`: nan, oo, -oo or zoo.
fn is_illegal(e: &Expr) -> bool {
    matches!(
        e.cls(),
        C::NaN | C::Infinity | C::NegativeInfinity | C::ComplexInfinity
    )
}

/// `expr.subs(...)`: substitutes `new` for `old` (see [`SubsArgs`]),
/// letting each class decide how (`_eval_subs`), then re-evaluating. With
/// `simultaneous`, nothing is evaluated until every substitution is made.
pub fn subs(expr: &Expr, args: SubsArgs, simultaneous: bool) -> Expr {
    subs_with_hints(expr, args, simultaneous, false)
}

/// `expr.subs(..., simultaneous=simultaneous, hack2=hack2)`: `hack2` is
/// passed on to `_subs` (simultaneous substitution always sets it).
pub fn subs_with_hints(expr: &Expr, args: SubsArgs, simultaneous: bool, hack2: bool) -> Expr {
    let (items, unordered) = match args {
        SubsArgs::Pair(old, new) => (vec![(old, new)], false),
        SubsArgs::Sequence(v) => (v, false),
        SubsArgs::Unordered(v) => (v, true),
    };
    // (sympify_old and sympify_new leave SymPy objects alone)

    // skip if there is no change
    let mut sequence: Vec<(Expr, Expr)> = items
        .into_iter()
        .filter(|(s1, s2)| !is_same(s1, s2))
        .collect();

    if unordered {
        let sequence_dict: PyDict<Expr, Expr> = sequence.into_iter().collect();
        // order so more complex items are first and items
        // of identical complexity are ordered so
        // f(x) < f(y) < x < y
        // \___ 2 __/    \_1_/  <- number of nodes
        //
        // For more complex ordering use an unordered sequence.
        let neg_nodes = |x: &Expr| SortKey::Float(-nodes(x));
        let dsk = |x: &Expr| default_sort_key(x, None);
        let k = ordered_with(
            sequence_dict.keys().cloned().collect(),
            &[&neg_nodes, &dsk],
            false,
        );
        sequence = k
            .into_iter()
            .map(|k| {
                let v = sequence_dict.get(&k).expect("a key of the dict").clone();
                (k, v)
            })
            .collect();
        // do infinities first
        if !simultaneous {
            let redo: Vec<usize> = sequence
                .iter()
                .enumerate()
                .filter(|(_, seq)| is_illegal(&seq.1))
                .map(|(i, _)| i)
                .collect();
            // (indices shift as items move: SymPy's loop, as written)
            for i in redo.into_iter().rev() {
                let item = sequence.remove(i);
                sequence.insert(0, item);
            }
        }
    }

    if simultaneous {
        // XXX should this be the default for dict subs?
        let mut reps: PyDict<Expr, Expr> = PyDict::new();
        let mut rv = expr.clone();
        let m = crate::symbol::dummy(Some("subs_m"), &[]);
        for (old, new) in &sequence {
            let com = new.is_commutative().unwrap_or(true);
            let d = crate::symbol::dummy(Some("subs_d"), &[(Fact::Commutative, com)]);
            // using d*m so Subs will be used on dummy variables
            // in things like Derivative(f(x, y), x) in which x
            // is both free and bound
            rv = subs_hint(&rv, old, &ops::mul(&d, &m), true);
            reps.insert(d, new.clone());
        }
        reps.insert(m, s::one()); // get rid of m
        xreplace(&rv, &reps)
    } else {
        let mut rv = expr.clone();
        for (old, new) in &sequence {
            rv = subs_hint(&rv, old, new, hack2);
        }
        rv
    }
}

impl Expr {
    /// `self.subs(old, new)`.
    pub fn subs(&self, old: &Expr, new: &Expr) -> Expr {
        subs(self, SubsArgs::Pair(old.clone(), new.clone()), false)
    }

    /// `self.xreplace(rule)`.
    pub fn xreplace(&self, rule: &PyDict<Expr, Expr>) -> Expr {
        xreplace(self, rule)
    }
}

/// `self._subs(old, new)`: `new` if `self` is `old`, else what the class's
/// `_eval_subs` makes of it, else the substitution in each arg.
pub fn _subs(e: &Expr, old: &Expr, new: &Expr) -> Expr {
    subs_hint(e, old, new, false)
}

/// `self._subs(old, new, hack2=hack2)`. `hack2` (set by simultaneous
/// `subs`) reaches the fallback of every node below, but not the `_subs`
/// calls an `_eval_subs` makes itself.
pub fn subs_hint(e: &Expr, old: &Expr, new: &Expr, hack2: bool) -> Expr {
    if is_same(e, old) {
        return new.clone();
    }
    match eval_subs(e, old, new) {
        Some(rv) => rv,
        None => fallback(e, old, new, hack2),
    }
}

/// `fallback(self, old, new)` in `Basic._subs`: try to replace `old` with
/// `new` in any of `self`'s arguments.
fn fallback(e: &Expr, old: &Expr, new: &Expr, hack2: bool) -> Expr {
    let mut hit = false;
    let mut args = e.args().to_vec();
    for a in args.iter_mut() {
        // (every arg is a Basic, which has _eval_subs)
        let arg = subs_hint(a, old, new, hack2);
        if !is_same(&arg, a) {
            hit = true;
            *a = arg;
        }
    }
    if hit {
        let rv = e.func(args.clone());
        if hack2 && e.is_Mul() && !rv.is_Mul() {
            // 2-arg hack
            let mut coeff = s::one();
            let mut nonnumber = Vec::new();
            for i in args {
                if i.is_Number() {
                    coeff = ops::mul(&coeff, &i);
                } else {
                    nonnumber.push(i);
                }
            }
            let nonnumber = e.func(nonnumber);
            if coeff.is_cls(C::One) {
                return nonnumber;
            } else {
                return crate::mul::mul_with(vec![coeff, nonnumber], false);
            }
        }
        return rv;
    }
    e.clone()
}

/// `self._eval_subs(old, new)`, dispatched on the class that defines it
/// (`None`: no special replacement, so `_subs` falls back to the args).
pub fn eval_subs(e: &Expr, old: &Expr, new: &Expr) -> Option<Expr> {
    match e.method_owner(Method::EvalSubs) {
        "Basic" => None,
        "Number" => Some(crate::numbers::number_eval_subs(e, old, new)),
        "Infinity" | "NegativeInfinity" => crate::numbers::infinity_eval_subs(e, old, new),
        "Symbol" => crate::symbol::eval_subs(e, old, new),
        "Add" => crate::add::eval_subs(e, old, new),
        "Mul" => crate::mul::eval_subs(e, old, new),
        "Pow" => crate::power::eval_subs(e, old, new),
        // Application._eval_subs only acts when `old` is the function class
        // itself and `new` a callable (`f(x).subs(f, g)`): `old == self.func`
        // is false for every SymPy object `old`, so here it is always None.
        "Application" => None,
        o => not_ported(format!("{o}._eval_subs")),
    }
}

// ---------------------------------------------------------------------------
// xreplace

/// A rule for [`xreplace_with`]: `obj in rule`, `rule[obj]` and
/// `bool(rule)`.
pub trait XreplaceRule {
    /// `rule[e]` if `e in rule`.
    fn lookup(&self, e: &Expr) -> Option<Expr>;
    /// `bool(rule)`.
    fn truthy(&self) -> bool;
}

impl XreplaceRule for PyDict<Expr, Expr> {
    fn lookup(&self, e: &Expr) -> Option<Expr> {
        self.get(e).cloned()
    }

    fn truthy(&self) -> bool {
        !self.is_empty()
    }
}

/// `sympy.core.rules.Transform(transform, filter)`: a rule that maps each
/// object passing `filter` to `transform(obj)`. It is always true.
pub struct Transform<'a> {
    pub transform: &'a dyn Fn(&Expr) -> Expr,
    pub filter: &'a dyn Fn(&Expr) -> bool,
}

impl XreplaceRule for Transform<'_> {
    fn lookup(&self, e: &Expr) -> Option<Expr> {
        if (self.filter)(e) {
            Some((self.transform)(e))
        } else {
            None
        }
    }

    fn truthy(&self) -> bool {
        true
    }
}

/// `expr.xreplace(rule)`: replaces each whole node found in `rule`, top
/// down, rebuilding (and so re-evaluating) the nodes above a replacement.
pub fn xreplace(expr: &Expr, rule: &PyDict<Expr, Expr>) -> Expr {
    xreplace_with(expr, rule)
}

/// [`xreplace`] for any [`XreplaceRule`].
pub fn xreplace_with(expr: &Expr, rule: &dyn XreplaceRule) -> Expr {
    if expr.is_Atom() {
        // Atom.xreplace: rule.get(self, self)
        return rule.lookup(expr).unwrap_or_else(|| expr.clone());
    }
    _xreplace(expr, rule).0
}

/// `self._xreplace(rule)`: the result and whether a replacement occurred.
pub fn _xreplace(e: &Expr, rule: &dyn XreplaceRule) -> (Expr, bool) {
    if let Some(v) = rule.lookup(e) {
        return (v, true);
    } else if rule.truthy() {
        let mut args = Vec::with_capacity(e.args().len());
        let mut changed = false;
        for a in e.args() {
            let (a_xr, c) = _xreplace(a, rule);
            args.push(a_xr);
            changed |= c;
        }
        if changed {
            return (e.func(args), true);
        }
    }
    (e.clone(), false)
}
