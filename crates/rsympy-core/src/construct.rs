//! `expr.func(*args)`: rebuilding a node of the same class.

use crate::assume::fuzzy_and;
use crate::error::not_ported;
use crate::expr::{Data, Expr};
use crate::facts::Fact;
use crate::generated::{ClassId as C, Tag};

/// A node with its class's default assumptions and `is_commutative` set
/// as an attribute, the way `AssocOp` and `Pow` store it.
pub(crate) fn node_with_commutative(cls: C, args: Vec<Expr>, commutative: Option<bool>) -> Expr {
    let e = Expr::make(cls, Data::None, args);
    let mut kb = e.assumptions_kb();
    kb.tell(Fact::Commutative, commutative);
    e.merge_kb(kb);
    e
}

/// `AssocOp._from_args(args, is_commutative)` for Add and Mul.
pub(crate) fn assoc_from_args(
    cls: C,
    mut args: Vec<Expr>,
    is_commutative: Option<Option<bool>>,
) -> Expr {
    match args.len() {
        0 => {
            return if cls == C::Add {
                crate::numbers::s::zero()
            } else {
                crate::numbers::s::one()
            };
        }
        1 => return args.pop().expect("one arg"),
        _ => {}
    }
    let c = match is_commutative {
        Some(c) => c,
        None => fuzzy_and(args.iter().map(Expr::is_commutative)),
    };
    node_with_commutative(cls, args, c)
}

/// A node of `cls` with these args and no evaluation (`Basic.__new__`).
/// For Add and Mul this is `_from_args`.
pub fn raw(cls: C, args: Vec<Expr>) -> Expr {
    match cls {
        C::Add | C::Mul => assoc_from_args(cls, args, None),
        C::Pow => {
            let [b, e] = <[Expr; 2]>::try_from(args).expect("Pow has two args");
            crate::power::from_args(&b, &e)
        }
        _ => Expr::make(cls, Data::None, args),
    }
}

/// `e.func(*args)`.
pub fn rebuild(e: &Expr, args: Vec<Expr>) -> Expr {
    match e.cls() {
        C::Add => crate::add::add(args),
        C::Mul => crate::mul::mul(args),
        C::Pow => {
            let [b, x] = <[Expr; 2]>::try_from(args).unwrap_or_else(|_| {
                crate::error::raise(crate::error::ErrorKind::TypeError, "Pow takes 2 arguments")
            });
            crate::power::pow(&b, &x)
        }
        _ if e.args().is_empty() && args.is_empty() => e.clone(),
        C::AppliedUndef => {
            let u = e
                .undef_data()
                .expect("AppliedUndef has its function")
                .clone();
            crate::functions::call_undefined(&u, args)
        }
        _ if e.is_a(Tag::Application) => crate::functions::call(e.cls(), args),
        _ => not_ported(format!("{}.func(*args)", e.class_name())),
    }
}
