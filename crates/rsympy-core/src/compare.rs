//! Canonical orderings: `Basic.compare` (the order of `Add`/`Mul` args),
//! `sort_key`/`default_sort_key` (printing order, `ordered`), and a port of
//! CPython's `list.sort`, since SymPy's results depend on its exact
//! behaviour with the keys it is given.
//!
//! SKELETON: signatures are the contract; bodies to be ported from
//! `sympy/core/basic.py` (`compare`, `_cmp_name`, `sort_key`),
//! `sympy/core/expr.py` (`sort_key`, `as_ordered_terms`, `as_terms`,
//! `_parse_order`), `sympy/core/mul.py` (`as_ordered_factors`),
//! `sympy/core/sorting.py` and `sympy/polys/orderings.py` (`monomial_key`),
//! and CPython's `Objects/listobject.c` (`list.sort`).

use std::cmp::Ordering;

use crate::error::not_ported;
use crate::expr::Expr;

/// `Basic.compare(a, b)`.
pub fn compare(a: &Expr, b: &Expr) -> Ordering {
    let _ = (a, b);
    not_ported("Basic.compare")
}

/// `_cmp_name(a, b)` on class names.
pub fn cmp_name(a: &str, b: &str) -> Ordering {
    let _ = (a, b);
    not_ported("_cmp_name")
}

/// A `sort_key()` value: SymPy's nested tuples of ints, strings and
/// numbers, compared like Python tuples.
#[derive(Clone, Debug)]
pub enum SortKey {
    Int(i64),
    Str(String),
    /// A SymPy Number (the coefficient slot), compared with SymPy's `<`.
    Num(Expr),
    Tuple(Vec<SortKey>),
}

/// `expr.sort_key(order)`.
pub fn sort_key(e: &Expr, order: Option<&str>) -> SortKey {
    let _ = (e, order);
    not_ported("sort_key")
}

/// `default_sort_key(item, order)` for an expression.
pub fn default_sort_key(e: &Expr, order: Option<&str>) -> SortKey {
    sort_key(e, order)
}

/// Python's `a < b` on sort keys (`TypeError` on mismatched types).
pub fn key_lt(a: &SortKey, b: &SortKey) -> bool {
    let _ = (a, b);
    not_ported("sort key comparison")
}

/// CPython's `list.sort()` with a `<` predicate (stable, and with the same
/// result as CPython even when the predicate is not a total order).
pub fn py_sort_by<T>(v: &mut [T], lt: impl FnMut(&T, &T) -> bool) {
    let _ = (v, lt);
    not_ported("list.sort")
}

/// `args.sort(key=_args_sortkey)`: sort by `Basic.compare`.
pub fn sort_args(v: &mut [Expr]) {
    py_sort_by(v, |a, b| compare(a, b) == Ordering::Less);
}

/// `sorted(seq, key=default_sort_key)`.
pub fn sorted_default(v: &mut [Expr]) {
    let keys: Vec<(SortKey, Expr)> = v
        .iter()
        .map(|e| (default_sort_key(e, None), e.clone()))
        .collect();
    let mut keys = keys;
    py_sort_by(&mut keys, |a, b| key_lt(&a.0, &b.0));
    for (slot, (_, e)) in v.iter_mut().zip(keys) {
        *slot = e;
    }
}

/// `expr.as_ordered_terms(order)` (an Add's terms in printing order).
pub fn as_ordered_terms(e: &Expr, order: Option<&str>) -> Vec<Expr> {
    let _ = (e, order);
    not_ported("Expr.as_ordered_terms")
}

/// `expr.as_ordered_factors(order)`.
pub fn as_ordered_factors(e: &Expr, order: Option<&str>) -> Vec<Expr> {
    let _ = (e, order);
    not_ported("Expr.as_ordered_factors")
}

/// `ordered(seq)` with the default keys.
pub fn ordered(seq: Vec<Expr>) -> Vec<Expr> {
    let _ = seq;
    not_ported("ordered")
}
