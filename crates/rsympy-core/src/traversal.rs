//! `sympy.core.traversal`: walking an expression tree.
//!
//! Ported from `sympy/core/traversal.py`: [`iterargs`] and
//! [`iterfreeargs`] (breadth first), [`preorder_traversal`] (with
//! `skip()`), [`postorder_traversal`], [`walk`] and [`bottom_up`].
//!
//! Without `keys`, SymPy's `preorder_traversal` walks the `_argset`
//! frozenset of a `LatticeOp` (`Min`, `Max`, `And`, `Or`, ...) instead of
//! its `args`, so its order there depends on Python's string hashing; rsympy
//! walks `args`. Every other order is SymPy's.

use crate::compare::{SortKey, ordered, ordered_with};
use crate::error::not_ported;
use crate::expr::Expr;
use crate::generated::{ClassId as C, Tag};

/// The `keys` argument of [`preorder_traversal`] and
/// [`postorder_traversal`].
#[derive(Clone, Copy)]
pub enum Keys<'a> {
    /// `keys=None`: args in storage order.
    None,
    /// `keys=True`: `ordered(args)` with its default keys.
    Default,
    /// `keys=[k1, k2, ...]`: `ordered(args, keys, default=False)`.
    With(&'a [&'a dyn Fn(&Expr) -> SortKey]),
}

impl Keys<'_> {
    /// `args` sorted as the traversals sort them.
    fn order(&self, args: &[Expr]) -> Vec<Expr> {
        match self {
            Keys::None => args.to_vec(),
            Keys::Default => ordered(args.to_vec()),
            Keys::With(keys) => ordered_with(args.to_vec(), keys, false),
        }
    }
}

/// `iterargs(expr)`: `expr` and all its subexpressions, breadth first.
pub fn iterargs(expr: &Expr) -> IterArgs {
    IterArgs {
        args: vec![expr.clone()],
        i: 0,
    }
}

/// The generator [`iterargs`] returns.
pub struct IterArgs {
    args: Vec<Expr>,
    i: usize,
}

impl Iterator for IterArgs {
    type Item = Expr;

    fn next(&mut self) -> Option<Expr> {
        // args = [expr]; for i in args: yield i; args.extend(i.args)
        let i = self.args.get(self.i)?.clone();
        self.i += 1;
        self.args.extend(i.args().iter().cloned());
        Some(i)
    }
}

/// `hasattr(e, 'bound_symbols')`: the classes with bound variables
/// (`Lambda`, `Subs` and the `ExprWithLimits`: `Integral`, `Sum`,
/// `Product`).
pub(crate) fn has_bound_symbols(e: &Expr) -> bool {
    matches!(e.cls(), C::Lambda | C::Subs) || e.is_a(Tag::ExprWithLimits)
}

/// `iterfreeargs(expr)`: like [`iterargs`], but for an object with bound
/// symbols it also yields the subexpressions of its canonical form
/// (`as_dummy()`) that do not contain the canonical variables.
///
/// `canonical_variables` and `as_dummy` are not ported, so reaching an
/// object with bound symbols raises `NotPortedYet`.
pub fn iterfreeargs(expr: &Expr) -> IterFreeArgs {
    IterFreeArgs {
        args: vec![expr.clone()],
        i: 0,
    }
}

/// The generator [`iterfreeargs`] returns.
pub struct IterFreeArgs {
    args: Vec<Expr>,
    i: usize,
}

impl Iterator for IterFreeArgs {
    type Item = Expr;

    fn next(&mut self) -> Option<Expr> {
        // `for i in args: yield i; if _first and hasattr(i,
        // 'bound_symbols'): ...; args.extend(i.args)`: the work after the
        // yield of the previous item happens when the next one is asked
        // for, so a caller that stops early never reaches it.
        if self.i > 0 {
            let last = self.args[self.i - 1].clone();
            if has_bound_symbols(&last) {
                not_ported(format!(
                    "iterfreeargs of {}: canonical_variables and as_dummy",
                    last.class_name()
                ));
            }
        }
        let i = self.args.get(self.i)?.clone();
        self.i += 1;
        self.args.extend(i.args().iter().cloned());
        Some(i)
    }
}

/// `preorder_traversal(node, keys)`: each node, then the preorder
/// traversal of each of its args. [`PreorderTraversal::skip`] drops the
/// subtrees of the node yielded last.
pub fn preorder_traversal<'a>(node: &Expr, keys: Keys<'a>) -> PreorderTraversal<'a> {
    PreorderTraversal {
        stack: vec![node.clone()],
        pending: None,
        skip_flag: false,
        keys,
    }
}

/// The iterator [`preorder_traversal`] returns.
pub struct PreorderTraversal<'a> {
    /// Nodes still to yield, the next one last.
    stack: Vec<Expr>,
    /// The node yielded last, whose args are pushed when the next node is
    /// asked for (unless `skip()` was called in between).
    pending: Option<Expr>,
    skip_flag: bool,
    keys: Keys<'a>,
}

impl PreorderTraversal<'_> {
    /// `pt.skip()`: do not descend into the node yielded last.
    pub fn skip(&mut self) {
        self.skip_flag = true;
    }
}

impl Iterator for PreorderTraversal<'_> {
    type Item = Expr;

    fn next(&mut self) -> Option<Expr> {
        if let Some(node) = self.pending.take() {
            if self.skip_flag {
                self.skip_flag = false;
            } else {
                // (LatticeOp's `_argset` is walked as `args`: see the
                // module docs.)
                let args = self.keys.order(node.args());
                self.stack.extend(args.into_iter().rev());
            }
        }
        let node = self.stack.pop()?;
        self.pending = Some(node.clone());
        Some(node)
    }
}

/// `postorder_traversal(node, keys)`: the postorder traversal of each arg,
/// then the node itself.
pub fn postorder_traversal(node: &Expr, keys: Keys<'_>) -> Vec<Expr> {
    let mut out = Vec::new();
    postorder(node, &keys, &mut out);
    out
}

fn postorder(node: &Expr, keys: &Keys<'_>, out: &mut Vec<Expr>) {
    for arg in keys.order(node.args()) {
        postorder(&arg, keys, out);
    }
    out.push(node.clone());
}

/// `walk(e, *target)`: `e` and, depth first, the args reached through
/// nodes for which `target` holds (`isinstance(e, target)`).
pub fn walk(e: &Expr, target: &dyn Fn(&Expr) -> bool) -> Vec<Expr> {
    let mut out = Vec::new();
    walk_into(e, target, &mut out);
    out
}

fn walk_into(e: &Expr, target: &dyn Fn(&Expr) -> bool, out: &mut Vec<Expr>) {
    if target(e) {
        out.push(e.clone());
        for i in e.args() {
            walk_into(i, target, out);
        }
    }
}

/// `bottom_up(rv, F, atoms)`: applies `f` to every subexpression with
/// args, from the leaves up (and to the leaves too when `atoms`).
pub fn bottom_up(rv: &Expr, f: &dyn Fn(&Expr) -> Expr, atoms: bool) -> Expr {
    let mut rv = rv.clone();
    if !rv.args().is_empty() {
        let args: Vec<Expr> = rv.args().iter().map(|a| bottom_up(a, f, atoms)).collect();
        if args.as_slice() != rv.args() {
            rv = rv.func(args);
        }
        rv = f(&rv);
    } else if atoms {
        rv = f(&rv);
    }
    rv
}
