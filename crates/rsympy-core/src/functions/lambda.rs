//! `sympy.core.function.Lambda`, and the `Lambda` methods of
//! `IdentityFunction` (whose `signature` and `expr` are properties; see
//! [`crate::functions::misc`]).
//!
//! A Python tuple passed as a signature or as a call argument is a `Tuple`
//! here, which is what `sympify` makes of it.

use std::collections::HashSet;

use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::{Data, Expr};
use crate::facts::Fact;
use crate::functions::misc;
use crate::functions::{Nargs, default_eval_is};
use crate::generated::ClassId as C;
use crate::generated_function::FUNCTION_INFO;
use crate::py::PyDict;

/// `Lambda(signature, expr)`.
pub fn lambda(signature: &Expr, expr: &Expr) -> Expr {
    if iterable(signature) && !signature.is_cls(C::Tuple) {
        // sympy_deprecation_warning(...); signature = tuple(signature)
        not_ported("Lambda: a non-tuple iterable signature (deprecated)");
    }
    // _sig = signature if iterable(signature) else (signature,)
    // sig: Tuple = sympify(_sig)
    let sig = if iterable(signature) {
        signature.clone()
    } else {
        tuple(vec![signature.clone()])
    };
    check_signature(&sig);

    if sig.args().len() == 1 && sig.arg(0) == expr {
        return misc::identity_function();
    }

    // Expr.__new__(cls, sig, sympify(expr))
    Expr::make(C::Lambda, Data::None, vec![sig, expr.clone()])
}

/// `iterable(obj)` for a SymPy object.
fn iterable(e: &Expr) -> bool {
    FUNCTION_INFO[e.cls() as usize].iterable
}

/// `Tuple(*args)`.
fn tuple(args: Vec<Expr>) -> Expr {
    crate::construct::raw(C::Tuple, args)
}

/// `Lambda._check_signature(sig)`.
fn check_signature(sig: &Expr) {
    // (a set: only membership is asked)
    let mut syms: HashSet<Expr> = HashSet::new();

    fn rcheck(args: &[Expr], syms: &mut HashSet<Expr>) {
        for a in args {
            if a.is_symbol() {
                if syms.contains(a) {
                    raise(
                        ErrorKind::BadSignatureError,
                        format!("Duplicate symbol {a:?}"),
                    );
                }
                syms.insert(a.clone());
            } else if a.is_cls(C::Tuple) {
                rcheck(a.args(), syms);
            } else {
                raise(
                    ErrorKind::BadSignatureError,
                    format!("Lambda signature should be only tuples and symbols, not {a:?}"),
                );
            }
        }
    }

    if !sig.is_cls(C::Tuple) {
        raise(
            ErrorKind::BadSignatureError,
            format!("Lambda signature should be a tuple not {sig:?}"),
        );
    }
    // Recurse through the signature:
    rcheck(sig.args(), &mut syms);
}

/// `self.signature`: the expected form of the arguments to be unpacked
/// into variables.
pub fn signature(l: &Expr) -> Expr {
    match l.cls() {
        C::IdentityFunction => misc::identity_signature(),
        _ => l.arg(0).clone(),
    }
}

/// `self.expr`: the return value of the function.
pub fn expr(l: &Expr) -> Expr {
    match l.cls() {
        C::IdentityFunction => misc::identity_expr(),
        _ => l.arg(1).clone(),
    }
}

/// `self.variables` (also `bound_symbols`): the signature's symbols, in
/// order.
pub fn variables(l: &Expr) -> Vec<Expr> {
    fn walk(args: &Expr, out: &mut Vec<Expr>) {
        if args.is_cls(C::Tuple) {
            for arg in args.args() {
                walk(arg, out);
            }
        } else {
            out.push(args.clone());
        }
    }
    let mut out = Vec::new();
    walk(&signature(l), &mut out);
    out
}

/// `self.nargs`: `FiniteSet(len(self.signature))`.
pub fn nargs(l: &Expr) -> Nargs {
    Nargs::Finite(vec![signature(l).args().len() as i64])
}

/// `self.free_symbols`: `self.expr.free_symbols - set(self.variables)`,
/// in the order `Expr::free_symbols` lists them.
pub fn free_symbols(l: &Expr) -> Vec<Expr> {
    let vars = variables(l);
    expr(l)
        .free_symbols()
        .into_iter()
        .filter(|s| !vars.contains(s))
        .collect()
}

/// `self.is_identity`: `self.signature == self.expr`.
pub fn is_identity(l: &Expr) -> bool {
    signature(l) == expr(l)
}

/// `self(*args)`: `Lambda.__call__`.
pub fn call(l: &Expr, args: &[Expr]) -> Expr {
    let n = args.len();
    let Nargs::Finite(nargs) = nargs(l) else {
        unreachable!("a Lambda's nargs is a FiniteSet")
    };
    if !nargs.contains(&(n as i64)) {
        // Lambda only ever has 1 value in nargs
        let k = nargs[0];
        raise(
            ErrorKind::BadArgumentsError,
            format!(
                "{l:?} takes exactly {k} argument{} ({n} given)",
                if k != 1 { "s" } else { "" }
            ),
        );
    }

    let d = match_signature(&signature(l), args);

    xreplace(&expr(l), &d)
}

/// `self._match_signature(sig, args)`.
fn match_signature(sig: &Expr, args: &[Expr]) -> PyDict<Expr, Expr> {
    fn rmatch(pars: &[Expr], args: &[Expr], symargmap: &mut PyDict<Expr, Expr>) {
        for (par, arg) in pars.iter().zip(args) {
            if par.is_symbol() {
                symargmap.insert(par.clone(), arg.clone());
            } else if par.is_cls(C::Tuple) {
                // (SymPy compares the lengths of the outer sequences here,
                // not of `par` and `arg`)
                if !arg.is_cls(C::Tuple) || args.len() != pars.len() {
                    raise(
                        ErrorKind::BadArgumentsError,
                        format!("Can't match {args:?} and {pars:?}"),
                    );
                }
                rmatch(par.args(), arg.args(), symargmap);
            }
        }
    }

    let mut symargmap = PyDict::new();
    rmatch(sig.args(), args, &mut symargmap);
    symargmap
}

/// `expr.xreplace(rule)` (`Atom.xreplace` for an atom, else
/// `Basic._xreplace`).
///
/// `Basic.xreplace` belongs to the subs unit; this private copy serves
/// `Lambda.__call__` until that port is merged.
fn xreplace(e: &Expr, rule: &PyDict<Expr, Expr>) -> Expr {
    if e.is_Atom() {
        // Atom.xreplace: rule.get(self, self)
        return rule.get(e).cloned().unwrap_or_else(|| e.clone());
    }
    xreplace_tracked(e, rule).0
}

/// `Basic._xreplace(rule)`: the result and whether anything was replaced.
fn xreplace_tracked(e: &Expr, rule: &PyDict<Expr, Expr>) -> (Expr, bool) {
    if let Some(v) = rule.get(e) {
        return (v.clone(), true);
    } else if !rule.is_empty() {
        let mut args = Vec::with_capacity(e.args().len());
        let mut changed = false;
        for a in e.args() {
            let (a_xr, hit) = xreplace_tracked(a, rule);
            args.push(a_xr);
            changed |= hit;
        }
        if changed {
            return (e.func(args), true);
        }
    }
    (e.clone(), false)
}

/// `Lambda._eval_evalf(prec)`: `self.func(self.args[0],
/// self.args[1].evalf(n=prec_to_dps(prec)))`.
pub(crate) fn eval_evalf(l: &Expr, prec: i64) -> Option<Expr> {
    if l.args().len() < 2 {
        // IdentityFunction has no args: self.args[0] fails
        raise(ErrorKind::IndexError, "tuple index out of range");
    }
    let body = crate::evalf::evalf(l.arg(1), rsympy_mpf::prec_to_dps(prec));
    Some(lambda(l.arg(0), &body))
}

/// The `_eval_is_<fact>` handlers of `Lambda` and `IdentityFunction`: only
/// the `Expr` ones.
pub(crate) fn eval_is(l: &Expr, fact: Fact) -> Option<bool> {
    default_eval_is(l, fact)
}
