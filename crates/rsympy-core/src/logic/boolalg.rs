//! `sympy.logic.boolalg`: `true`/`false`, the Boolean functions (`And`,
//! `Or`, `Not`, `Xor`, `Nand`, `Nor`, `Xnor`, `Implies`, `Equivalent`,
//! `ITE`, `Exclusive`), Python's `&`, `|`, `^`, `~`, `>>` and `<<` on SymPy
//! objects, and the normal forms (`to_nnf`, `to_cnf`, `to_dnf`, `to_anf`
//! and their `is_*` tests).
//!
//! SymPy keeps the arguments of `And`, `Or`, `Xor` and `Equivalent` in a
//! frozenset (`_argset`) next to the sorted tuple they print. Python sets
//! iterate in hash order; here they are vectors in insertion order, which
//! gives the same answers wherever SymPy's answer does not depend on that
//! order (ties in `ordered` and chains of three or more related
//! relationals in `Xor`/`Equivalent` are the cases that do).
//!
//! `Equivalent` stores `(FiniteSet(*argset),)` as its `_args` and answers
//! `.args` with `tuple(ordered(argset))`; rsympy stores the latter, which
//! `Basic.__eq__`, `Basic.compare` and `sort_key` see the same way.

use crate::compare::ordered;
use crate::ctx;
use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::Expr;
use crate::generated::{ClassId as C, Flag, Method, Tag};
use crate::numbers::s;
use crate::relational::{binary_symbols, canonical, negated};
use rsympy_num::Int;

// ===========================================================================
// Python equality with bools and small ints
// ===========================================================================

/// Python's `e == b` for a Python bool `b` (`e.__eq__(b)`, then
/// `bool.__eq__`, then identity): `Integer.__eq__` compares `p` with
/// `int(b)`, `BooleanTrue`/`BooleanFalse.__eq__` check `other is True` /
/// `other is False`, and every other class's `__eq__` answers False or
/// `NotImplemented`.
pub fn eq_py_bool(e: &Expr, b: bool) -> bool {
    match e.method_owner(Method::Eq) {
        "Integer" => e.p() == b as i64,
        "BooleanTrue" => b,
        "BooleanFalse" => !b,
        // Rational.__eq__: _sympify(b) is not a Number; Float.__eq__ ->
        // Basic.__eq__ -> NotImplemented; `other == float('inf')`; NaN:
        // `other is self`; Basic.__eq__ -> NotImplemented; NumberSymbol,
        // Tuple, Subs and FiniteSet answer False.
        "Rational" | "Float" | "Infinity" | "NegativeInfinity" | "NaN" | "Basic"
        | "NumberSymbol" | "Tuple" | "Subs" | "FiniteSet" => false,
        o => not_ported(format!("{o}.__eq__ with a Python bool")),
    }
}

/// `e in (True, False)`.
pub fn in_true_false(e: &Expr) -> bool {
    eq_py_bool(e, true) || eq_py_bool(e, false)
}

/// Python's `e == n` for a Python int `n`.
fn eq_py_int(e: &Expr, n: i64) -> bool {
    match e.method_owner(Method::Eq) {
        "Integer" => e.p() == n,
        // Rational.__eq__: a non-integer never equals Integer(n);
        // `BooleanTrue.__eq__(1)` -> Basic.__eq__ -> NotImplemented, and
        // `int.__eq__` -> NotImplemented, so identity decides.
        "Rational" | "Float" | "Infinity" | "NegativeInfinity" | "NaN" | "Basic"
        | "NumberSymbol" | "Tuple" | "Subs" | "FiniteSet" | "BooleanTrue" | "BooleanFalse" => false,
        o => not_ported(format!("{o}.__eq__ with a Python int")),
    }
}

/// `e in (0, 1)`.
fn in_zero_one(e: &Expr) -> bool {
    eq_py_int(e, 0) || eq_py_int(e, 1)
}

fn boolean(b: bool) -> Expr {
    if b { s::true_() } else { s::false_() }
}

// ===========================================================================
// Python sets of SymPy objects
// ===========================================================================

/// `set.add`: Basic objects of these classes compare structurally.
fn set_add(set: &mut Vec<Expr>, e: Expr) {
    if !set.contains(&e) {
        set.push(e);
    }
}

/// `set.discard`; returns whether `e` was there.
fn set_discard(set: &mut Vec<Expr>, e: &Expr) -> bool {
    match set.iter().position(|x| x == e) {
        Some(i) => {
            set.remove(i);
            true
        }
        None => false,
    }
}

/// `set.remove`: `KeyError` if `e` is not there.
fn set_remove(set: &mut Vec<Expr>, e: &Expr) {
    if !set_discard(set, e) {
        raise(ErrorKind::KeyError, format!("{e:?}"));
    }
}

/// `argset.remove(a) if a in argset else argset.add(a)`.
fn set_toggle(set: &mut Vec<Expr>, e: &Expr) {
    if !set_discard(set, e) {
        set.push(e.clone());
    }
}

/// `itertools.combinations(range(n), k)`, in its order.
fn combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    if k > n {
        return out;
    }
    let mut idx: Vec<usize> = (0..k).collect();
    loop {
        out.push(idx.clone());
        // find the rightmost index that can still move
        let Some(i) = (0..k).rev().find(|&i| idx[i] != i + n - k) else {
            return out;
        };
        idx[i] += 1;
        for j in i + 1..k {
            idx[j] = idx[j - 1] + 1;
        }
    }
}

// ===========================================================================
// as_Boolean and the shared argument check
// ===========================================================================

/// `as_Boolean(e)`: `true`/`false` for what equals `True`/`False`, a
/// Symbol's value when its `is_zero` is known, any other Boolean as is;
/// `TypeError` otherwise.
pub fn as_boolean(e: &Expr) -> Expr {
    if eq_py_bool(e, true) {
        return s::true_();
    }
    if eq_py_bool(e, false) {
        return s::false_();
    }
    if e.is_a(Tag::Symbol) {
        return match e.is_zero() {
            None => e.clone(),
            Some(true) => s::false_(),
            Some(false) => s::true_(),
        };
    }
    if e.is_a(Tag::Boolean) {
        return e.clone();
    }
    raise(
        ErrorKind::TypeError,
        format!("expecting bool or Boolean, not `{e:?}`."),
    )
}

/// `BooleanFunction.binary_check_and_simplify(*args)`.
pub fn binary_check_and_simplify(args: &[Expr]) -> Vec<Expr> {
    args.iter().map(as_boolean).collect()
}

// ===========================================================================
// And, Or (LatticeOp)
// ===========================================================================

/// `ShortCircuit`, raised by `LatticeOp._new_args_filter`.
struct ShortCircuit;

/// `(cls.zero, cls.identity)` of And or Or.
fn zero_identity(cls: C) -> (Expr, Expr) {
    match cls {
        C::And => (s::false_(), s::true_()),
        C::Or => (s::true_(), s::false_()),
        _ => unreachable!("{cls:?} is not a LatticeOp"),
    }
}

/// `LatticeOp._new_args_filter(arg_sequence, call_cls)`, consumed at once
/// (its callers consume the generator whole before using it).
fn lattice_filter(args: Vec<Expr>, cls: C) -> Result<Vec<Expr>, ShortCircuit> {
    let (zero, identity) = zero_identity(cls);
    let mut out = Vec::new();
    for arg in args {
        if arg == zero {
            return Err(ShortCircuit);
        } else if arg == identity {
            continue;
        } else if arg.cls() == cls {
            out.extend(arg.args().iter().cloned());
        } else {
            out.push(arg);
        }
    }
    Ok(out)
}

/// `And._new_args_filter(args)`.
fn and_new_args_filter(args: Vec<Expr>) -> Result<Vec<Expr>, ShortCircuit> {
    let args = binary_check_and_simplify(&args);
    let args = lattice_filter(args, C::And)?;
    let mut newargs = Vec::new();
    let mut rel: Vec<Expr> = Vec::new();
    for x in ordered(args) {
        if x.is_Relational() {
            let c = canonical(&x);
            if rel.contains(&c) {
                continue;
            } else if rel.contains(&canonical(&negated(&c))) {
                return Ok(vec![s::false_()]);
            } else {
                rel.push(c);
            }
        }
        newargs.push(x);
    }
    Ok(newargs)
}

/// `Or._new_args_filter(args)`.
fn or_new_args_filter(args: Vec<Expr>) -> Result<Vec<Expr>, ShortCircuit> {
    let mut newargs = Vec::new();
    let mut rel: Vec<Expr> = Vec::new();
    let args = binary_check_and_simplify(&args);
    for x in args {
        if x.is_Relational() {
            let c = canonical(&x);
            if rel.contains(&c) {
                continue;
            }
            let nc = canonical(&negated(&c));
            if rel.contains(&nc) {
                return Ok(vec![s::true_()]);
            }
            rel.push(c);
        }
        newargs.push(x);
    }
    lattice_filter(newargs, C::Or)
}

/// `LatticeOp.__new__(cls, *args)` for And and Or (which ignore
/// `evaluate`).
fn lattice_new(cls: C, args: Vec<Expr>) -> Expr {
    let filtered = match cls {
        C::And => and_new_args_filter(args),
        C::Or => or_new_args_filter(args),
        _ => unreachable!("{cls:?} is not a LatticeOp"),
    };
    let (zero, identity) = zero_identity(cls);
    let Ok(filtered) = filtered else {
        return zero;
    };
    // frozenset(...)
    let mut set = Vec::new();
    for a in filtered {
        set_add(&mut set, a);
    }
    match set.len() {
        0 => identity,
        1 => set.pop().expect("one arg"),
        _ => crate::construct::raw(cls, ordered(set)),
    }
}

/// `And(*args)`.
pub fn and(args: Vec<Expr>) -> Expr {
    lattice_new(C::And, args)
}

/// `Or(*args)`.
pub fn or(args: Vec<Expr>) -> Expr {
    lattice_new(C::Or, args)
}

/// `LatticeOp.make_args(expr)` for And or Or: `expr._argset`, or
/// `frozenset([expr])`.
pub fn make_args(cls: C, expr: &Expr) -> Vec<Expr> {
    if expr.cls() == cls {
        expr.args().to_vec()
    } else {
        vec![expr.clone()]
    }
}

/// `conjuncts(expr)`: `And.make_args(expr)`.
pub fn conjuncts(expr: &Expr) -> Vec<Expr> {
    make_args(C::And, expr)
}

/// `disjuncts(expr)`: `Or.make_args(expr)`.
pub fn disjuncts(expr: &Expr) -> Vec<Expr> {
    make_args(C::Or, expr)
}

// ===========================================================================
// The Application-based functions: Not, Nand, Nor, Xnor, Implies, Exclusive
// ===========================================================================

/// `Not(arg)`, evaluating as `global_parameters.evaluate` says.
pub fn not(arg: &Expr) -> Expr {
    application_new(C::Not, vec![arg.clone()], ctx::evaluate())
}

/// `Not.eval(cls, arg)`.
fn not_eval(args: &[Expr]) -> Option<Expr> {
    let [arg] = args else {
        raise(
            ErrorKind::TypeError,
            format!(
                "Not.eval() takes 2 positional arguments but {} were given",
                args.len() + 1
            ),
        )
    };
    if arg.is_Number() || in_true_false(arg) {
        return Some(if arg.truthy() {
            s::false_()
        } else {
            s::true_()
        });
    }
    if arg.flag(Flag::Not) {
        return Some(first_arg(arg).clone());
    }
    // Simplify Relational objects.
    if arg.is_Relational() {
        return Some(negated(arg));
    }
    None
}

/// `expr.args[0]`, `IndexError` for no args.
fn first_arg(e: &Expr) -> &Expr {
    match e.args().first() {
        Some(a) => a,
        None => raise(ErrorKind::IndexError, "tuple index out of range"),
    }
}

/// The value of an `Implies.eval` argument: `bool(x)` for a Number or
/// what equals 0 or 1, else the object.
#[derive(Clone)]
enum PyArg {
    Bool(bool),
    Obj(Expr),
}

impl PyArg {
    /// `x in (True, False)`.
    fn in_true_false(&self) -> bool {
        match self {
            PyArg::Bool(_) => true,
            PyArg::Obj(e) => in_true_false(e),
        }
    }

    /// `_sympify(x)`.
    fn sympify(&self) -> Expr {
        match self {
            PyArg::Bool(b) => boolean(*b),
            PyArg::Obj(e) => e.clone(),
        }
    }
}

/// `Implies.eval(cls, *args)`.
fn implies_eval(args: &[Expr]) -> Option<Expr> {
    let newargs: Vec<PyArg> = args
        .iter()
        .map(|x| {
            if x.is_Number() || in_zero_one(x) {
                PyArg::Bool(x.truthy())
            } else {
                PyArg::Obj(x.clone())
            }
        })
        .collect();
    let [a, b] = newargs.as_slice() else {
        raise(
            ErrorKind::ValueError,
            format!(
                "{} operand(s) used for an Implies (pairs are required)",
                args.len()
            ),
        )
    };
    if a.in_true_false() || b.in_true_false() {
        return Some(or(vec![not(&a.sympify()), b.sympify()]));
    }
    let (PyArg::Obj(a), PyArg::Obj(b)) = (a, b) else {
        unreachable!("bools are in (True, False)")
    };
    if a == b {
        Some(s::true_())
    } else if a.is_Relational() && b.is_Relational() {
        if canonical(a) == canonical(b) {
            return Some(s::true_());
        }
        if canonical(&negated(a)) == canonical(b) {
            return Some(b.clone());
        }
        None
    } else {
        Some(crate::construct::raw(C::Implies, args.to_vec()))
    }
}

/// `Exclusive.eval(cls, *args)`.
fn exclusive_eval(args: &[Expr]) -> Expr {
    let and_args = combinations(args.len(), 2)
        .into_iter()
        .map(|ij| not(&and(vec![args[ij[0]].clone(), args[ij[1]].clone()])))
        .collect();
    and(and_args)
}

/// `Application.__new__(cls, *args, evaluate=evaluate)` for the
/// BooleanFunctions that keep it: `cls.eval(*args)` when evaluating, else
/// (or when it returns None) `Basic.__new__`.
fn application_new(cls: C, args: Vec<Expr>, evaluate: bool) -> Expr {
    if evaluate {
        let evaluated = match cls {
            C::Not => not_eval(&args),
            C::Nand => Some(not(&and(args.clone()))),
            C::Nor => Some(not(&or(args.clone()))),
            C::Xnor => Some(not(&xor(args.clone()))),
            C::Implies => implies_eval(&args),
            C::Exclusive => Some(exclusive_eval(&args)),
            _ => unreachable!("{cls:?} has no eval here"),
        };
        if let Some(r) = evaluated {
            return r;
        }
    }
    crate::construct::raw(cls, args)
}

/// `Nand(*args)`.
pub fn nand(args: Vec<Expr>) -> Expr {
    application_new(C::Nand, args, ctx::evaluate())
}

/// `Nor(*args)`.
pub fn nor(args: Vec<Expr>) -> Expr {
    application_new(C::Nor, args, ctx::evaluate())
}

/// `Xnor(*args)`.
pub fn xnor(args: Vec<Expr>) -> Expr {
    application_new(C::Xnor, args, ctx::evaluate())
}

/// `Implies(*args)`.
pub fn implies(args: Vec<Expr>) -> Expr {
    application_new(C::Implies, args, ctx::evaluate())
}

/// `Exclusive(*args)`.
pub fn exclusive(args: Vec<Expr>) -> Expr {
    application_new(C::Exclusive, args, ctx::evaluate())
}

// ===========================================================================
// Xor
// ===========================================================================

/// `Xor(*args)`.
pub fn xor(args: Vec<Expr>) -> Expr {
    xor_with(args, true)
}

/// `Xor(*args, remove_true=remove_true)` (`Xor.__new__`, which evaluates
/// whatever `evaluate` says).
pub fn xor_with(args: Vec<Expr>, remove_true: bool) -> Expr {
    let mut argset: Vec<Expr> = Vec::new();
    // obj = super().__new__(cls, *args): Application.eval returns None
    for arg in &args {
        let mut arg = arg.clone();
        if arg.is_Number() || in_true_false(&arg) {
            if arg.truthy() {
                arg = s::true_();
            } else {
                continue;
            }
        }
        if arg.is_cls(C::Xor) {
            for a in arg.args() {
                set_toggle(&mut argset, a);
            }
        } else {
            set_toggle(&mut argset, &arg);
        }
    }
    let rel: Vec<(Expr, Expr, Expr)> = argset
        .iter()
        .filter(|r| r.is_Relational())
        .map(|r| (r.clone(), canonical(r), canonical(&negated(r))))
        .collect();
    // is number of complimentary pairs odd? start 0 -> False
    let mut odd = false;
    let mut remove = Vec::new();
    for (i, (r, c, nc)) in rel.iter().enumerate() {
        let mut rj = None;
        for (rj_, cj, _) in &rel[i + 1..] {
            if cj == nc {
                odd = !odd;
                rj = Some(rj_);
                break;
            } else if cj == c {
                rj = Some(rj_);
                break;
            }
        }
        if let Some(rj) = rj {
            remove.push((r.clone(), rj.clone()));
        }
    }
    if odd {
        set_toggle(&mut argset, &s::true_());
    }
    for (a, b) in &remove {
        set_remove(&mut argset, a);
        set_remove(&mut argset, b);
    }
    // `True in argset` finds what hashes like True and equals it: `true`,
    // or a `1` toggled in from a nested Xor's args
    let true_at = argset.iter().position(|e| eq_py_bool(e, true));
    if argset.is_empty() {
        return s::false_();
    } else if argset.len() == 1 {
        return argset.pop().expect("one arg");
    } else if let Some(i) = true_at
        && remove_true
    {
        argset.remove(i);
        return not(&xor(argset));
    }
    crate::construct::raw(C::Xor, ordered(argset))
}

// ===========================================================================
// Equivalent
// ===========================================================================

/// An element of `Equivalent.__new__`'s argset: a SymPy object, or the
/// Python bool that replaced a Number or a `True`/`False` lookalike.
#[derive(Clone)]
enum Item {
    Bool(bool),
    Obj(Expr),
}

impl Item {
    /// Python's `==` (with equal hashes) between two set elements.
    fn eq(&self, other: &Item) -> bool {
        match (self, other) {
            (Item::Bool(a), Item::Bool(b)) => a == b,
            (Item::Bool(b), Item::Obj(e)) | (Item::Obj(e), Item::Bool(b)) => eq_py_bool(e, *b),
            (Item::Obj(a), Item::Obj(b)) => a == b,
        }
    }

    fn sympify(&self) -> Expr {
        match self {
            Item::Bool(b) => boolean(*b),
            Item::Obj(e) => e.clone(),
        }
    }
}

fn items_discard(set: &mut Vec<Item>, x: &Item) -> bool {
    match set.iter().position(|y| y.eq(x)) {
        Some(i) => {
            set.remove(i);
            true
        }
        None => false,
    }
}

fn items_add(set: &mut Vec<Item>, x: Item) {
    if !set.iter().any(|y| y.eq(&x)) {
        set.push(x);
    }
}

/// `Equivalent(*args)` (`Equivalent.__new__`, which ignores `evaluate`).
pub fn equivalent(args: Vec<Expr>) -> Expr {
    let mut argset: Vec<Item> = Vec::new();
    for a in &args {
        items_add(&mut argset, Item::Obj(a.clone()));
    }
    for x in &args {
        if x.is_Number() || in_true_false(x) {
            // Includes 0, 1
            items_discard(&mut argset, &Item::Obj(x.clone()));
            items_add(&mut argset, Item::Bool(x.truthy()));
        }
    }
    let mut rel = Vec::new();
    for r in &argset {
        if let Item::Obj(r) = r
            && r.is_a(Tag::Relational)
        {
            rel.push((r.clone(), canonical(r), canonical(&negated(r))));
        }
    }
    let mut remove = Vec::new();
    for (i, (r, c, nc)) in rel.iter().enumerate() {
        for (rj, cj, _) in &rel[i + 1..] {
            if cj == nc {
                return s::false_();
            } else if cj == c {
                remove.push((r.clone(), rj.clone()));
                break;
            }
        }
    }
    for (a, b) in remove {
        for x in [a, b] {
            if !items_discard(&mut argset, &Item::Obj(x.clone())) {
                raise(ErrorKind::KeyError, format!("{x:?}"));
            }
        }
        items_add(&mut argset, Item::Bool(true));
    }
    if argset.len() <= 1 {
        return s::true_();
    }
    if items_discard(&mut argset, &Item::Bool(true)) {
        return and(argset.iter().map(Item::sympify).collect());
    }
    if items_discard(&mut argset, &Item::Bool(false)) {
        return and(argset.iter().map(|a| not(&a.sympify())).collect());
    }
    let argset: Vec<Expr> = argset.iter().map(Item::sympify).collect();
    crate::construct::raw(C::Equivalent, ordered(argset))
}

// ===========================================================================
// ITE
// ===========================================================================

/// `ITE(*args)` with `evaluate=evaluate` (default True, whatever
/// `global_parameters` says).
pub fn ite(args: Vec<Expr>, evaluate: bool) -> Expr {
    let [a, b, c] = <[Expr; 3]>::try_from(args)
        .unwrap_or_else(|_| raise(ErrorKind::ValueError, "expecting exactly 3 args"));
    let (mut a, mut b, mut c) = (a, b, c);
    // check use of binary symbols
    if matches!(a.cls(), C::Equality | C::Unequality) {
        // in this context, we can evaluate the Eq/Ne
        // if one arg is a binary symbol and the other
        // is true/false
        b = as_boolean(&b);
        c = as_boolean(&c);
        let mut bin_syms: Vec<Expr> = Vec::new();
        for i in [&b, &c] {
            for x in binary_symbols(i) {
                set_add(&mut bin_syms, x);
            }
        }
        let mut rest: Vec<Expr> = Vec::new();
        for x in a.args() {
            if !bin_syms.contains(x) {
                set_add(&mut rest, x.clone());
            }
        }
        if rest.len() == 1 {
            // one arg is a binary_symbols
            let _a = a.clone();
            let (l, r) = (a.arg(0).clone(), a.arg(1).clone());
            a = if l.is_cls(C::BooleanTrue) {
                r
            } else if r.is_cls(C::BooleanTrue) {
                l
            } else if l.is_cls(C::BooleanFalse) {
                not(&r)
            } else if r.is_cls(C::BooleanFalse) {
                not(&l)
            } else {
                // binary can only equal True or False
                s::false_()
            };
            if _a.is_cls(C::Unequality) {
                a = not(&a);
            }
        }
    } else {
        let v = binary_check_and_simplify(&[a, b, c]);
        [a, b, c] = <[Expr; 3]>::try_from(v).expect("three args");
    }
    let mut rv = None;
    if evaluate {
        rv = ite_eval(a.clone(), b.clone(), c.clone());
    }
    rv.unwrap_or_else(|| crate::construct::raw(C::ITE, vec![a, b, c]))
}

/// `ITE.eval(cls, *args)`.
fn ite_eval(a: Expr, b: Expr, c: Expr) -> Option<Expr> {
    let mut a = a;
    // do the args give a singular result?
    if matches!(a.cls(), C::Unequality | C::Equality) {
        let mut _a = Some(a.clone());
        let t = s::true_();
        let f = s::false_();
        if a.args().contains(&t) {
            a = if a.arg(1).is_cls(C::BooleanTrue) {
                a.arg(0).clone()
            } else {
                a.arg(1).clone()
            };
        } else if a.args().contains(&f) {
            a = if a.arg(1).is_cls(C::BooleanFalse) {
                not(a.arg(0))
            } else {
                not(a.arg(1))
            };
        } else {
            _a = None;
        }
        if let Some(_a) = _a
            && _a.is_cls(C::Unequality)
        {
            a = not(&a);
        }
    }
    if a.is_cls(C::BooleanTrue) {
        return Some(b);
    }
    if a.is_cls(C::BooleanFalse) {
        return Some(c);
    }
    if b == c {
        return Some(b);
    } else {
        // or maybe the results allow the answer to be expressed
        // in terms of the condition
        if b.is_cls(C::BooleanTrue) && c.is_cls(C::BooleanFalse) {
            return Some(a);
        }
        if b.is_cls(C::BooleanFalse) && c.is_cls(C::BooleanTrue) {
            return Some(not(&a));
        }
    }
    // `[a, b, c] != args` compares a list with a tuple: always true
    Some(ite(vec![a, b, c], false))
}

// ===========================================================================
// Construction by class
// ===========================================================================

/// `cls(*args, evaluate=evaluate)` for a BooleanFunction class
/// (`evaluate=None`: not passed).
pub fn new_boolfunc(cls: C, args: Vec<Expr>, evaluate: Option<bool>) -> Expr {
    match cls {
        C::And | C::Or => lattice_new(cls, args),
        C::Xor => xor_with(args, true),
        C::Equivalent => equivalent(args),
        C::ITE => ite(args, evaluate.unwrap_or(true)),
        C::Not | C::Nand | C::Nor | C::Xnor | C::Implies | C::Exclusive => {
            application_new(cls, args, evaluate.unwrap_or_else(ctx::evaluate))
        }
        _ => unreachable!("{cls:?} is not a BooleanFunction"),
    }
}

/// `cls(*args, remove_true=False)`: Xor keeps `true`, And, Or, ITE and
/// Equivalent ignore the option, and `Application.__new__` rejects it.
fn new_keep_true(cls: C, args: Vec<Expr>) -> Expr {
    match cls {
        C::Xor => xor_with(args, false),
        C::And | C::Or | C::Equivalent | C::ITE => new_boolfunc(cls, args, None),
        _ => raise(
            ErrorKind::ValueError,
            "Unknown options: {'remove_true': False}",
        ),
    }
}

// ===========================================================================
// Python's operators on Booleans and Integers
// ===========================================================================

/// One of Python's bitwise binary operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitOp {
    /// `&` (`__and__` / `__rand__`)
    And,
    /// `|` (`__or__` / `__ror__`)
    Or,
    /// `^` (`__xor__` / `__rxor__`)
    Xor,
    /// `>>` (`__rshift__` / `__rrshift__`)
    Rshift,
    /// `<<` (`__lshift__` / `__rlshift__`)
    Lshift,
}

impl BitOp {
    /// The forward and reflected method.
    fn methods(self) -> (Method, Method) {
        match self {
            BitOp::And => (Method::And, Method::Rand),
            BitOp::Or => (Method::Or, Method::Ror),
            BitOp::Xor => (Method::Xor, Method::Rxor),
            BitOp::Rshift => (Method::Rshift, Method::Rrshift),
            BitOp::Lshift => (Method::Lshift, Method::Rlshift),
        }
    }

    fn symbol(self) -> &'static str {
        match self {
            BitOp::And => "&",
            BitOp::Or => "|",
            BitOp::Xor => "^",
            BitOp::Rshift => ">>",
            BitOp::Lshift => "<<",
        }
    }
}

/// Whether `type(e)` fills the number slot of `op`: a Python class does
/// when its MRO defines the method or its reflection.
fn has_slot(e: &Expr, op: BitOp) -> bool {
    let (m, r) = op.methods();
    !e.method_owner(m).is_empty() || !e.method_owner(r).is_empty()
}

/// `method_is_overloaded(left, right, name)`: whether `type(right).name`
/// is not the same object as `type(left).name`.
fn method_is_overloaded(left: &Expr, right: &Expr, name: Method) -> bool {
    let b = right.method_owner(name);
    if b.is_empty() {
        return false;
    }
    let a = left.method_owner(name);
    if a.is_empty() {
        return true;
    }
    a != b
}

/// `type(slf).<method>(slf, other)`, `None` for `NotImplemented` (or no
/// such method).
fn call_bit_method(slf: &Expr, other: &Expr, method: Method) -> Option<Expr> {
    match slf.method_owner(method) {
        "" => None,
        // sympify_return([('other', 'Boolean')], NotImplemented)
        "Boolean" => {
            if !other.is_a(Tag::Boolean) {
                return None;
            }
            let (a, b) = (slf.clone(), other.clone());
            Some(match method {
                // __rand__ = __and__ and so on: `self` comes first
                Method::And | Method::Rand => and(vec![a, b]),
                Method::Or | Method::Ror => or(vec![a, b]),
                Method::Xor | Method::Rxor => xor(vec![a, b]),
                // __rlshift__ = __rshift__
                Method::Rshift | Method::Rlshift => implies(vec![a, b]),
                // __rrshift__ = __lshift__
                Method::Lshift | Method::Rrshift => implies(vec![b, a]),
                _ => unreachable!("{method:?} is not a bitwise operator"),
            })
        }
        // isinstance(other, (int, Integer, numbers.Integral)): an Integer
        "Integer" => {
            if !other.is_Integer() {
                return None;
            }
            let (p, o) = (slf.p(), other.p());
            Some(Expr::integer(match method {
                Method::And | Method::Rand => &p & &o,
                // two's-complement identities
                Method::Or | Method::Ror => &(&p + &o) - &(&p & &o),
                Method::Xor | Method::Rxor => {
                    &(&p + &o) - &(&(&p & &o) * &rsympy_num::Int::from(2))
                }
                Method::Lshift => shift(&p, &o, true),
                Method::Rshift => shift(&p, &o, false),
                Method::Rlshift => shift(&o, &p, true),
                Method::Rrshift => shift(&o, &p, false),
                _ => unreachable!("{method:?} is not a bitwise operator"),
            }))
        }
        o => not_ported(format!("{o}.{method:?}")),
    }
}

/// Python's `a << n` / `a >> n` on ints (CPython's `long_lshift` and
/// `long_rshift`).
fn shift(a: &rsympy_num::Int, n: &rsympy_num::Int, left: bool) -> rsympy_num::Int {
    use rsympy_num::Int;
    if n.is_negative() {
        raise(ErrorKind::ValueError, "negative shift count");
    }
    if a.is_zero() {
        return Int::from(0);
    }
    if left {
        // `divmod_shift` splits the count into 30-bit digits, clipping a
        // word count of `PY_SSIZE_T_MAX / sizeof(digit)` or more to that,
        // and `_PyLong_New` refuses more than `MAX_LONG_DIGITS` digits.
        const SHIFT: u64 = 30;
        let clip = Int::from(i64::MAX / 4);
        let max_long_digits = Int::from((i64::MAX - 24) / 4);
        let (wordshift, remshift) = if n.as_i64().is_some() {
            let k = n.as_u64().expect("a non-negative i64");
            (Int::from((k / SHIFT) as i64), k % SHIFT)
        } else {
            let shift = Int::from(SHIFT as i64);
            let (w, r) = n.div_mod(&shift).expect("nonzero divisor");
            if w < clip {
                (w, r.as_u64().expect("a digit"))
            } else {
                (clip, 0)
            }
        };
        let oldsize = Int::from(a.bit_length().div_ceil(SHIFT) as i64);
        let mut newsize = &oldsize + &wordshift;
        if remshift != 0 {
            newsize = &newsize + &Int::from(1);
        }
        if newsize > max_long_digits {
            raise(ErrorKind::OverflowError, "too many digits in integer");
        }
        match n.as_u64() {
            Some(k) if k <= 1 << 24 => a << k,
            // Python allocates the result or raises MemoryError, which
            // depends on the machine
            _ => not_ported("int << with a shift count beyond 2**24"),
        }
    } else {
        match n.as_u64() {
            Some(k) => a >> k,
            None => {
                if a.is_negative() {
                    rsympy_num::Int::from(-1)
                } else {
                    rsympy_num::Int::from(0)
                }
            }
        }
    }
}

/// The `SLOT1BINFULL` function CPython installs for a Python class's
/// `__and__`/`__rand__` (etc.), called as `slot(slf, other)`.
fn slot1binfull(slf: &Expr, other: &Expr, op: BitOp) -> Option<Expr> {
    let (m, rm) = op.methods();
    let mut do_other = slf.cls() != other.cls() && has_slot(other, op);
    if has_slot(slf, op) {
        if do_other
            && crate::relational::is_proper_subclass(other, slf)
            && method_is_overloaded(slf, other, rm)
        {
            if let Some(r) = call_bit_method(other, slf, rm) {
                return Some(r);
            }
            do_other = false;
        }
        let r = call_bit_method(slf, other, m);
        if r.is_some() || slf.cls() == other.cls() {
            return r;
        }
    }
    if do_other {
        return call_bit_method(other, slf, rm);
    }
    None
}

/// Python's `v <op> w` (`binary_op1`): every SymPy class is a Python
/// class, so both operands' slots are the same `SLOT1BINFULL` function and
/// the right operand's slot only runs on its own when the left has none.
pub fn bit_op(v: &Expr, w: &Expr, op: BitOp) -> Expr {
    if (has_slot(v, op) || (v.cls() != w.cls() && has_slot(w, op)))
        && let Some(r) = slot1binfull(v, w, op)
    {
        return r;
    }
    raise(
        ErrorKind::TypeError,
        format!(
            "unsupported operand type(s) for {}: '{}' and '{}'",
            op.symbol(),
            v.class_name(),
            w.class_name()
        ),
    )
}

/// Python's `~a` (`type(a).__invert__`).
pub fn invert(a: &Expr) -> Expr {
    match a.method_owner(Method::Invert) {
        // Boolean.__invert__
        "Boolean" => not(a),
        // Integer.__invert__
        "Integer" => Expr::integer(-a.p() - rsympy_num::Int::from(1)),
        "" => raise(
            ErrorKind::TypeError,
            format!("bad operand type for unary ~: '{}'", a.class_name()),
        ),
        o => not_ported(format!("{o}.__invert__")),
    }
}

// ===========================================================================
// Literals and normal-form tests
// ===========================================================================

/// `is_literal(expr)` (no `AppliedPredicate` exists here).
pub fn is_literal(expr: &Expr) -> bool {
    if expr.is_cls(C::Not) {
        is_literal(first_arg(expr))
    } else if in_true_false(expr) || expr.is_Atom() {
        true
    } else {
        !expr.is_a(Tag::BooleanFunction) && expr.args().iter().all(Expr::is_Atom)
    }
}

/// `is_nnf(expr, simplified)`.
pub fn is_nnf(expr: &Expr, simplified: bool) -> bool {
    if is_literal(expr) {
        return true;
    }
    let mut stack = vec![expr.clone()];
    while let Some(expr) = stack.pop() {
        if matches!(expr.cls(), C::And | C::Or) {
            if simplified {
                let args = expr.args();
                for arg in args {
                    if args.contains(&not(arg)) {
                        return false;
                    }
                }
            }
            stack.extend(expr.args().iter().cloned());
        } else if !is_literal(&expr) {
            return false;
        }
    }
    true
}

/// `_is_form(expr, function1, function2)`.
fn is_form(expr: &Expr, function1: C, function2: C) -> bool {
    let vals = if expr.cls() == function1 {
        make_args(function1, expr)
    } else {
        vec![expr.clone()]
    };
    for lit in &vals {
        if lit.cls() == function2 {
            for l in make_args(function2, lit) {
                if !is_literal(&l) {
                    return false;
                }
            }
        } else if !is_literal(lit) {
            return false;
        }
    }
    true
}

/// `is_cnf(expr)`.
pub fn is_cnf(expr: &Expr) -> bool {
    is_form(expr, C::And, C::Or)
}

/// `is_dnf(expr)`.
pub fn is_dnf(expr: &Expr) -> bool {
    is_form(expr, C::Or, C::And)
}

/// `is_anf(expr)`.
pub fn is_anf(expr: &Expr) -> bool {
    if is_literal(expr) && !expr.is_cls(C::Not) {
        return true;
    }
    if expr.is_cls(C::And) {
        expr.args().iter().all(Expr::is_Symbol)
    } else if expr.is_cls(C::Xor) {
        for arg in expr.args() {
            if arg.is_cls(C::And) {
                if !arg.args().iter().all(Expr::is_Symbol) {
                    return false;
                }
            } else if is_literal(arg) {
                if arg.is_cls(C::Not) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    } else {
        false
    }
}

// ===========================================================================
// Negation normal form
// ===========================================================================

/// `to_nnf(expr, simplify)`.
pub fn to_nnf(expr: &Expr, simplify: bool) -> Expr {
    if is_nnf(expr, simplify) {
        return expr.clone();
    }
    to_nnf_method(expr, simplify)
}

/// `expr.to_nnf(simplify)`, dispatched on its class.
fn to_nnf_method(e: &Expr, simplify: bool) -> Expr {
    match e.method_owner(Method::ToNnf) {
        // Boolean.to_nnf: override where necessary
        "Boolean" => e.clone(),
        // BooleanFunction.to_nnf
        "BooleanFunction" => bf_to_nnf(e.cls(), e.args().to_vec(), simplify),
        "Not" => not_to_nnf(e, simplify),
        "Xor" => xor_to_nnf(e, simplify),
        "Implies" => {
            let [a, b] = unpack::<2>(e.args());
            bf_to_nnf(C::Or, vec![not(&a), b], simplify)
        }
        "Equivalent" => {
            let args = e.args();
            let mut out = Vec::new();
            for (a, b) in args.iter().zip(args.iter().skip(1)) {
                out.push(or(vec![not(a), b.clone()]));
            }
            let (Some(last), Some(first)) = (args.last(), args.first()) else {
                raise(ErrorKind::IndexError, "tuple index out of range")
            };
            out.push(or(vec![not(last), first.clone()]));
            bf_to_nnf(C::And, out, simplify)
        }
        "ITE" => {
            let [a, b, c] = unpack::<3>(e.args());
            bf_to_nnf(C::And, vec![or(vec![not(&a), b]), or(vec![a, c])], simplify)
        }
        "" => raise(
            ErrorKind::AttributeError,
            format!("'{}' object has no attribute 'to_nnf'", e.class_name()),
        ),
        o => not_ported(format!("{o}.to_nnf")),
    }
}

/// `a, b = args`: `ValueError` for the wrong count.
fn unpack<const N: usize>(args: &[Expr]) -> [Expr; N] {
    <[Expr; N]>::try_from(args.to_vec()).unwrap_or_else(|_| {
        raise(
            ErrorKind::ValueError,
            format!("expected {N} values to unpack, got {}", args.len()),
        )
    })
}

/// `cls.zero` for `BooleanFunction._to_nnf`.
fn class_zero(cls: C) -> Expr {
    match cls {
        C::And | C::Or => zero_identity(cls).0,
        _ => raise(
            ErrorKind::AttributeError,
            format!("type object '{cls:?}' has no attribute 'zero'"),
        ),
    }
}

/// `BooleanFunction._to_nnf(cls, *args, simplify=simplify)`.
fn bf_to_nnf(cls: C, args: Vec<Expr>, simplify: bool) -> Expr {
    let mut argset: Vec<Expr> = Vec::new();
    for arg in args {
        let mut arg = arg;
        if !is_literal(&arg) {
            arg = to_nnf_method(&arg, simplify);
        }
        if simplify {
            let parts = if arg.cls() == cls {
                arg.args().to_vec()
            } else {
                vec![arg]
            };
            for a in parts {
                if argset.contains(&not(&a)) {
                    return class_zero(cls);
                }
                set_add(&mut argset, a);
            }
        } else {
            set_add(&mut argset, arg);
        }
    }
    new_boolfunc(cls, argset, None)
}

/// The clauses of `Xor.to_nnf` and `Not.to_nnf` of a Xor: for each subset
/// `neg` of `args` with a size in `sizes`, `Or` of the args with those in
/// `neg` negated.
fn xor_clauses(args: &[Expr], sizes: impl Iterator<Item = usize>) -> Vec<Expr> {
    let mut out = Vec::new();
    for i in sizes {
        for idx in combinations(args.len(), i) {
            let neg: Vec<&Expr> = idx.iter().map(|&k| &args[k]).collect();
            let clause = args
                .iter()
                .map(|s_| {
                    if neg.contains(&s_) {
                        not(s_)
                    } else {
                        s_.clone()
                    }
                })
                .collect();
            out.push(or(clause));
        }
    }
    out
}

/// `Xor.to_nnf(simplify)`.
fn xor_to_nnf(e: &Expr, simplify: bool) -> Expr {
    let args = e.args();
    let clauses = xor_clauses(args, (0..=args.len()).step_by(2));
    bf_to_nnf(C::And, clauses, simplify)
}

/// `Not.to_nnf(simplify)`.
fn not_to_nnf(e: &Expr, simplify: bool) -> Expr {
    if is_literal(e) {
        return e.clone();
    }
    let expr = first_arg(e);
    let args = expr.args();
    let nots = || args.iter().map(not).collect::<Vec<_>>();
    match expr.cls() {
        C::And => bf_to_nnf(C::Or, nots(), simplify),
        C::Or => bf_to_nnf(C::And, nots(), simplify),
        C::Implies => {
            let [a, b] = unpack::<2>(args);
            bf_to_nnf(C::And, vec![a, not(&b)], simplify)
        }
        C::Equivalent => bf_to_nnf(C::And, vec![or(args.to_vec()), or(nots())], simplify),
        C::Xor => {
            let clauses = xor_clauses(args, (1..=args.len()).step_by(2));
            bf_to_nnf(C::And, clauses, simplify)
        }
        C::ITE => {
            let [a, b, c] = unpack::<3>(args);
            bf_to_nnf(
                C::And,
                vec![or(vec![a.clone(), not(&c)]), or(vec![not(&a), not(&b)])],
                simplify,
            )
        }
        _ => raise(
            ErrorKind::ValueError,
            format!("Illegal operator {} in expression", expr.class_name()),
        ),
    }
}

/// `eliminate_implications(expr)`: `to_nnf(expr, simplify=False)`.
pub fn eliminate_implications(expr: &Expr) -> Expr {
    to_nnf(expr, false)
}

// ===========================================================================
// Conjunctive and disjunctive normal form
// ===========================================================================

/// `_distribute((expr, f1, f2))`: distributes `f1` over `f2`.
fn distribute(expr: &Expr, f1: C, f2: C) -> Expr {
    if expr.cls() == f2 {
        let args = expr.args();
        let Some(k) = args.iter().position(|a| a.cls() == f1) else {
            return expr.clone();
        };
        let conj = &args[k];
        // [a for a in info[0].args if a is not conj]
        let others: Vec<Expr> = args
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != k)
            .map(|(_, a)| a.clone())
            .collect();
        let rest = new_boolfunc(f2, others, None);
        let parts = conj
            .args()
            .iter()
            .map(|c| {
                distribute(
                    &new_boolfunc(f2, vec![c.clone(), rest.clone()], None),
                    f1,
                    f2,
                )
            })
            .collect();
        new_keep_true(f1, parts)
    } else if expr.cls() == f1 {
        let parts = expr.args().iter().map(|x| distribute(x, f1, f2)).collect();
        new_keep_true(f1, parts)
    } else {
        expr.clone()
    }
}

/// `distribute_and_over_or(expr)`.
pub fn distribute_and_over_or(expr: &Expr) -> Expr {
    distribute(expr, C::And, C::Or)
}

/// `distribute_or_over_and(expr)`.
pub fn distribute_or_over_and(expr: &Expr) -> Expr {
    distribute(expr, C::Or, C::And)
}

/// `distribute_xor_over_and(expr)`.
pub fn distribute_xor_over_and(expr: &Expr) -> Expr {
    distribute(expr, C::Xor, C::And)
}

/// `_find_predicates(expr)`: what is not a BooleanFunction, without
/// repeats (SymPy returns a set).
pub fn find_predicates(expr: &Expr) -> Vec<Expr> {
    let mut out = Vec::new();
    collect_predicates(expr, &mut out);
    out
}

fn collect_predicates(expr: &Expr, out: &mut Vec<Expr>) {
    if !expr.is_a(Tag::BooleanFunction) {
        set_add(out, expr.clone());
        return;
    }
    for a in expr.args() {
        collect_predicates(a, out);
    }
}

/// `to_cnf(expr, simplify, force)` / `to_dnf(...)`: the shared body.
fn to_normal_form(expr: &Expr, simplify: bool, force: bool, cnf: bool) -> Expr {
    if !expr.is_a(Tag::BooleanFunction) {
        return expr.clone();
    }
    if simplify {
        if !force && find_predicates(expr).len() > 8 {
            raise(
                ErrorKind::ValueError,
                "To simplify a logical expression with more than 8 variables may take a \
                 long time and requires the use of `force=True`.",
            );
        }
        return simplify_logic(expr, Some(if cnf { "cnf" } else { "dnf" }), true, force);
    }
    // Don't convert unless we have to
    if if cnf { is_cnf(expr) } else { is_dnf(expr) } {
        return expr.clone();
    }
    let expr = eliminate_implications(expr);
    if cnf {
        distribute_and_over_or(&expr)
    } else {
        distribute_or_over_and(&expr)
    }
}

/// `simplify_logic(expr, form, deep, force)` (no `dontcare`), as far as
/// it goes without `Basic.xreplace`: the quick exit for an expression
/// already in the requested form, and expressions that are not
/// BooleanFunctions.
pub fn simplify_logic(expr: &Expr, form: Option<&str>, deep: bool, force: bool) -> Expr {
    if !matches!(form, None | Some("cnf" | "dnf")) {
        raise(ErrorKind::ValueError, "form can be cnf or dnf only");
    }
    // check for quick exit if form is given: right form and all args are
    // literal and do not involve Not
    if let Some(form) = form {
        let form_ok = if form == "cnf" {
            is_cnf(expr)
        } else {
            is_dnf(expr)
        };
        if form_ok && expr.args().iter().all(is_literal) {
            return expr.clone();
        }
    }
    // `expr.atoms(Relational)`; with none, `expr.xreplace({})` is `expr`
    let has_relationals = expr.has_a(Tag::Relational);
    if deep && has_relationals {
        not_ported("simplify_logic: simplify of the Relationals (simplify, xreplace)");
    }
    if !expr.is_a(Tag::BooleanFunction) {
        return expr.clone();
    }
    if has_relationals {
        not_ported("simplify_logic: replacing the Relationals with Dummies (xreplace)");
    }
    // Get new variables after replacing
    let variables = find_predicates(expr);
    if !force && variables.len() > 8 {
        // expr.xreplace(undo), with nothing to undo
        return expr.clone();
    }
    not_ported("simplify_logic: _get_truthtable (needs Basic.xreplace)")
}

/// `to_cnf(expr, simplify, force)`.
pub fn to_cnf(expr: &Expr, simplify: bool, force: bool) -> Expr {
    to_normal_form(expr, simplify, force, true)
}

/// `to_dnf(expr, simplify, force)`.
pub fn to_dnf(expr: &Expr, simplify: bool, force: bool) -> Expr {
    to_normal_form(expr, simplify, force, false)
}

// ===========================================================================
// Algebraic normal form
// ===========================================================================

/// `to_anf(expr, deep)`.
pub fn to_anf(expr: &Expr, deep: bool) -> Expr {
    if is_anf(expr) {
        return expr.clone();
    }
    to_anf_method(expr, deep)
}

/// `expr.to_anf(deep=deep)`, dispatched on its class.
fn to_anf_method(e: &Expr, deep: bool) -> Expr {
    let args = e.args();
    match e.method_owner(Method::ToAnf) {
        "And" => {
            if deep {
                let result = bf_to_anf(C::And, args.to_vec(), deep);
                return distribute_xor_over_and(&result);
            }
            e.clone()
        }
        "Or" => {
            // the powerset of the args, as Ands
            let mut out = Vec::new();
            for j in 1..=args.len() {
                for idx in combinations(args.len(), j) {
                    let x = and(idx.iter().map(|&k| args[k].clone()).collect());
                    out.push(if deep { to_anf(&x, deep) } else { x });
                }
            }
            xor_with(out, false)
        }
        "Not" => bf_to_anf(C::Xor, vec![s::true_(), first_arg(e).clone()], deep),
        "BooleanFunction" => bf_to_anf(e.cls(), args.to_vec(), deep),
        "Implies" => {
            let [a, b] = unpack::<2>(args);
            let ab = and(vec![a.clone(), b]);
            bf_to_anf(C::Xor, vec![s::true_(), a, ab], deep)
        }
        "Equivalent" => {
            let a = and(args.to_vec());
            let b = and(args.iter().map(|arg| to_anf(&not(arg), false)).collect());
            let b = distribute_xor_over_and(&b);
            bf_to_anf(C::Xor, vec![a, b], deep)
        }
        "" => raise(
            ErrorKind::AttributeError,
            format!("'{}' object has no attribute 'to_anf'", e.class_name()),
        ),
        o => not_ported(format!("{o}.to_anf")),
    }
}

/// `BooleanFunction._to_anf(cls, *args, deep=deep)`.
fn bf_to_anf(cls: C, args: Vec<Expr>, deep: bool) -> Expr {
    let mut new_args = Vec::new();
    for arg in args {
        let mut arg = arg;
        if deep && (!is_literal(&arg) || arg.is_cls(C::Not)) {
            arg = to_anf_method(&arg, deep);
        }
        new_args.push(arg);
    }
    new_keep_true(cls, new_args)
}

// ===========================================================================
// Integer representations of terms
// ===========================================================================

/// `ibin(n, bits)` (`sympy.utilities.iterables`, with an int `bits`): the
/// bits of `n`, most significant first, left-padded with zeros to `bits`.
pub fn ibin(n: &Int, bits: usize) -> Vec<i64> {
    if n.is_negative() {
        raise(ErrorKind::ValueError, "negative numbers are not allowed");
    }
    let nbits = n.bit_length();
    if nbits > bits as u64 {
        raise(ErrorKind::ValueError, format!("`bits` must be >= {nbits}"));
    }
    // bin(n)[2:].rjust(bits, "0"): bin(0) is '0b0'
    let width = nbits.max(1);
    let one = Int::from(1);
    let mut out = vec![0; (bits as u64).saturating_sub(width) as usize];
    for i in (0..width).rev() {
        out.push(if (&(n >> i) & &one).is_one() { 1 } else { 0 });
    }
    out
}

/// `term_to_integer(term)` for a list: `int(''.join(map(str, term)), 2)`.
pub fn term_to_integer(term: &[i64]) -> Int {
    let text: String = term.iter().map(i64::to_string).collect();
    // int(text, 2): an optional sign, then binary digits
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(&text)),
    };
    if digits.is_empty() || !digits.bytes().all(|c| c == b'0' || c == b'1') {
        raise(
            ErrorKind::ValueError,
            format!("invalid literal for int() with base 2: '{text}'"),
        );
    }
    let two = Int::from(2);
    let mut v = Int::from(0);
    for c in digits.bytes() {
        v = &(&v * &two) + &Int::from((c - b'0') as i64);
    }
    if negative { -v } else { v }
}

/// `to_int_repr(clauses, symbols)`: each clause as the set of its
/// literals, `i` for the `i`-th symbol (from 1) and `-i` for its negation.
/// The sets are returned sorted.
pub fn to_int_repr(clauses: &[Expr], symbols: &[Expr]) -> Vec<Vec<i64>> {
    // symbols = dict(zip(symbols, range(1, len(symbols) + 1))): a repeated
    // symbol keeps its last number
    let number = |e: &Expr| -> i64 {
        match symbols.iter().rposition(|s_| s_ == e) {
            Some(i) => i as i64 + 1,
            None => raise(ErrorKind::KeyError, format!("{e:?}")),
        }
    };
    clauses
        .iter()
        .map(|c| {
            let mut set: Vec<i64> = disjuncts(c)
                .iter()
                .map(|arg| {
                    if arg.is_cls(C::Not) {
                        -number(first_arg(arg))
                    } else {
                        number(arg)
                    }
                })
                .collect();
            set.sort_unstable();
            set.dedup();
            set
        })
        .collect()
}

// ===========================================================================
// SOPform, POSform and ANFform (Quine-McCluskey)
// ===========================================================================

/// `variables[n]`: `IndexError` past the end.
fn variable(variables: &[Expr], n: usize) -> Expr {
    match variables.get(n) {
        Some(v) => v.clone(),
        None => raise(ErrorKind::IndexError, "tuple index out of range"),
    }
}

/// `_check_pair(minterm1, minterm2)`: the index of the only position
/// where the terms differ, or -1.
fn check_pair(minterm1: &[i64], minterm2: &[i64]) -> Option<usize> {
    let mut index = None;
    for (x, i) in minterm1.iter().enumerate() {
        let Some(other) = minterm2.get(x) else {
            raise(ErrorKind::IndexError, "list index out of range")
        };
        if i != other {
            if index.is_none() {
                index = Some(x);
            } else {
                return None;
            }
        }
    }
    index
}

/// `_convert_to_varsSOP(minterm, variables)`.
fn convert_to_vars_sop(minterm: &[i64], variables: &[Expr]) -> Expr {
    let temp = minterm
        .iter()
        .enumerate()
        .filter(|&(_, &val)| val != 3)
        .map(|(n, &val)| {
            let v = variable(variables, n);
            if val == 1 { v } else { not(&v) }
        })
        .collect();
    and(temp)
}

/// `_convert_to_varsPOS(maxterm, variables)`.
fn convert_to_vars_pos(maxterm: &[i64], variables: &[Expr]) -> Expr {
    let temp = maxterm
        .iter()
        .enumerate()
        .filter(|&(_, &val)| val != 3)
        .map(|(n, &val)| {
            let v = variable(variables, n);
            if val == 0 { v } else { not(&v) }
        })
        .collect();
    or(temp)
}

/// `_convert_to_varsANF(term, variables)`.
fn convert_to_vars_anf(term: &[i64], variables: &[Expr]) -> Expr {
    let temp: Vec<Expr> = term
        .iter()
        .enumerate()
        .filter(|&(_, &t)| t == 1)
        .map(|(n, _)| variable(variables, n))
        .collect();
    if temp.is_empty() {
        return s::true_();
    }
    and(temp)
}

/// `_simplified_pairs(terms)`: merges terms that differ in one position
/// (marked 3, "don't care"), repeatedly.
fn simplified_pairs(terms: Vec<Vec<i64>>) -> Vec<Vec<i64>> {
    if terms.is_empty() {
        return vec![];
    }
    let mut simplified_terms: Vec<Vec<i64>> = Vec::new();
    let mut todo: Vec<Option<usize>> = (0..terms.len()).map(Some).collect();
    // Count number of ones as _check_pair can only potentially match if
    // there is at most a difference of a single one
    let mut termdict: std::collections::BTreeMap<usize, Vec<usize>> = Default::default();
    for (n, term) in terms.iter().enumerate() {
        let ones = term.iter().filter(|&&t| t == 1).count();
        termdict.entry(ones).or_default().push(n);
    }
    let empty = Vec::new();
    let variables = terms[0].len();
    for k in 0..variables {
        for &i in termdict.get(&k).unwrap_or(&empty) {
            for &j in termdict.get(&(k + 1)).unwrap_or(&empty) {
                if let Some(index) = check_pair(&terms[i], &terms[j]) {
                    // Mark terms handled
                    todo[i] = None;
                    todo[j] = None;
                    // Copy old term
                    let mut newterm = terms[i].clone();
                    // Set differing position to don't care
                    newterm[index] = 3;
                    // Add if not already there
                    if !simplified_terms.contains(&newterm) {
                        simplified_terms.push(newterm);
                    }
                }
            }
        }
    }
    if !simplified_terms.is_empty() {
        // Further simplifications only among the new terms
        simplified_terms = simplified_pairs(simplified_terms);
    }
    // Add remaining, non-simplified, terms
    simplified_terms.extend(todo.iter().flatten().map(|&i| terms[i].clone()));
    simplified_terms
}

/// `_rem_redundancy(l1, terms)`: the prime implicants of `l1` that the
/// prime implicant table method keeps to cover `terms`.
fn rem_redundancy(l1: &[Vec<i64>], terms: &[Vec<i64>]) -> Vec<Vec<i64>> {
    if terms.is_empty() {
        return vec![];
    }
    let nterms = terms.len();
    let nl1 = l1.len();
    // Create dominating matrix
    let mut dommatrix = vec![vec![0u8; nl1]; nterms];
    let mut colcount = vec![0usize; nl1];
    let mut rowcount = vec![0usize; nterms];
    for (primei, prime) in l1.iter().enumerate() {
        for (termi, term) in terms.iter().enumerate() {
            // Check prime implicant covering term
            if prime.iter().zip(term).all(|(&t, &mt)| t == 3 || t == mt) {
                dommatrix[termi][primei] = 1;
                colcount[primei] += 1;
                rowcount[termi] += 1;
            }
        }
    }
    let column = |dommatrix: &[Vec<u8>], coli: usize| -> Vec<u8> {
        (0..nterms).map(|i| dommatrix[i][coli]).collect()
    };
    // Keep track if anything changed
    let mut anythingchanged = true;
    // Then, go again
    while anythingchanged {
        anythingchanged = false;
        for rowi in 0..nterms {
            // Still non-dominated?
            if rowcount[rowi] != 0 {
                let row = dommatrix[rowi].clone();
                for row2i in 0..nterms {
                    // Still non-dominated?
                    if rowi != row2i && rowcount[rowi] != 0 && rowcount[rowi] <= rowcount[row2i] {
                        let row2 = dommatrix[row2i].clone();
                        if (0..nl1).all(|n| row2[n] >= row[n]) {
                            // row2 dominating row, remove row2
                            rowcount[row2i] = 0;
                            anythingchanged = true;
                            for (primei, &prime) in row2.iter().enumerate() {
                                if prime != 0 {
                                    // Make corresponding entry 0
                                    dommatrix[row2i][primei] = 0;
                                    colcount[primei] -= 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut colcache: std::collections::HashMap<usize, Vec<u8>> = Default::default();
        for coli in 0..nl1 {
            // Still non-dominated?
            if colcount[coli] != 0 {
                let col = colcache
                    .entry(coli)
                    .or_insert_with(|| column(&dommatrix, coli))
                    .clone();
                for col2i in 0..nl1 {
                    // Still non-dominated?
                    if coli != col2i && colcount[col2i] != 0 && colcount[coli] >= colcount[col2i] {
                        let col2 = colcache
                            .entry(col2i)
                            .or_insert_with(|| column(&dommatrix, col2i))
                            .clone();
                        if (0..nterms).all(|n| col[n] >= col2[n]) {
                            // col dominating col2, remove col2
                            colcount[col2i] = 0;
                            anythingchanged = true;
                            for (termi, &term) in col2.iter().enumerate() {
                                if term != 0 && dommatrix[termi][col2i] != 0 {
                                    // Make corresponding entry 0
                                    dommatrix[termi][col2i] = 0;
                                    rowcount[termi] -= 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        if !anythingchanged {
            // Heuristically select the prime implicant covering most terms
            let mut maxterms = 0;
            let mut bestcolidx = None;
            for (coli, &s_) in colcount.iter().enumerate() {
                if s_ > maxterms {
                    bestcolidx = Some(coli);
                    maxterms = s_;
                }
            }
            // In case we found a prime implicant covering at least two terms
            if let Some(best) = bestcolidx
                && maxterms > 1
            {
                let Some(bestcol) = colcache.get(&best).cloned() else {
                    raise(ErrorKind::KeyError, format!("{best}"))
                };
                for primei in 0..nl1 {
                    if primei != best {
                        for (termi, &term) in bestcol.iter().enumerate() {
                            if term != 0 && dommatrix[termi][primei] != 0 {
                                // Make corresponding entry 0
                                dommatrix[termi][primei] = 0;
                                anythingchanged = true;
                                rowcount[termi] -= 1;
                                colcount[primei] -= 1;
                            }
                        }
                    }
                }
            }
        }
    }
    (0..nl1)
        .filter(|&i| colcount[i] != 0)
        .map(|i| l1[i].clone())
        .collect()
}

/// A term in the `minterms` or `dontcares` of `SOPform` and `POSform`.
#[derive(Clone, Debug)]
pub enum TermSpec {
    /// A Python int, read as the bits of a term.
    Int(Int),
    /// A list or tuple of bits.
    Bits(Vec<i64>),
    /// A dict giving the bits of some of the variables; the term stands
    /// for every value of the others.
    Dict(Vec<(Expr, i64)>),
    /// Anything else (`TypeError`).
    Other,
}

/// `_input_to_binlist(inputlist, variables)`.
fn input_to_binlist(inputlist: &[TermSpec], variables: &[Expr]) -> Vec<Vec<i64>> {
    let mut binlist = Vec::new();
    let bits = variables.len();
    for val in inputlist {
        match val {
            TermSpec::Int(n) => binlist.push(ibin(n, bits)),
            TermSpec::Dict(val) => {
                let mut nonspecvars = variables.to_vec();
                for (key, _) in val {
                    // list.remove: the first equal item, or ValueError
                    match nonspecvars.iter().position(|v| v == key) {
                        Some(i) => {
                            nonspecvars.remove(i);
                        }
                        None => raise(ErrorKind::ValueError, "list.remove(x): x not in list"),
                    }
                }
                for t in product01(nonspecvars.len()) {
                    // d = dict(zip(nonspecvars, t)); d.update(val)
                    let mut d: Vec<(Expr, i64)> = Vec::new();
                    let mut set = |k: &Expr, v: i64| match d.iter_mut().find(|(x, _)| x == k) {
                        Some(slot) => slot.1 = v,
                        None => d.push((k.clone(), v)),
                    };
                    for (k, v) in nonspecvars.iter().zip(&t) {
                        set(k, *v);
                    }
                    for (k, v) in val {
                        set(k, *v);
                    }
                    binlist.push(
                        variables
                            .iter()
                            .map(|v| d.iter().find(|(k, _)| k == v).expect("every variable").1)
                            .collect(),
                    );
                }
            }
            TermSpec::Bits(val) => {
                if val.len() != bits {
                    raise(
                        ErrorKind::ValueError,
                        format!(
                            "Each term must contain {bits} bits as there are\n{bits} variables \
                             (or be an integer)."
                        ),
                    );
                }
                binlist.push(val.clone());
            }
            TermSpec::Other => raise(
                ErrorKind::TypeError,
                "A term list can only contain lists, ints or dicts.",
            ),
        }
    }
    binlist
}

/// `itertools.product((0, 1), repeat=n)`, in its order.
fn product01(n: usize) -> Vec<Vec<i64>> {
    let mut out = vec![vec![]];
    for _ in 0..n {
        out = out
            .into_iter()
            .flat_map(|t| {
                [0, 1].map(|b| {
                    let mut t = t.clone();
                    t.push(b);
                    t
                })
            })
            .collect();
    }
    out
}

/// The shared start of `SOPform` and `POSform`: the minterms and
/// don't-cares as bit lists, `ValueError` for a term in both.
fn minterms_and_dontcares(
    variables: &[Expr],
    minterms: &[TermSpec],
    dontcares: &[TermSpec],
) -> (Vec<Vec<i64>>, Vec<Vec<i64>>) {
    let minterms = input_to_binlist(minterms, variables);
    let dontcares = input_to_binlist(dontcares, variables);
    for d in &dontcares {
        if minterms.contains(d) {
            raise(
                ErrorKind::ValueError,
                format!("{d:?} in minterms is also in dontcares"),
            );
        }
    }
    (minterms, dontcares)
}

/// `SOPform(variables, minterms, dontcares)`: the smallest sum of
/// products that is true for the minterms.
pub fn sop_form(variables: &[Expr], minterms: &[TermSpec], dontcares: &[TermSpec]) -> Expr {
    if minterms.is_empty() {
        return s::false_();
    }
    let (minterms, dontcares) = minterms_and_dontcares(variables, minterms, dontcares);
    sop_form_bits(variables, minterms, dontcares)
}

/// `_sop_form(variables, minterms, dontcares)`.
fn sop_form_bits(variables: &[Expr], minterms: Vec<Vec<i64>>, dontcares: Vec<Vec<i64>>) -> Expr {
    let mut all = minterms.clone();
    all.extend(dontcares);
    let new = simplified_pairs(all);
    let essential = rem_redundancy(&new, &minterms);
    or(essential
        .iter()
        .map(|x| convert_to_vars_sop(x, variables))
        .collect())
}

/// `POSform(variables, minterms, dontcares)`: the smallest product of
/// sums that is true for the minterms.
pub fn pos_form(variables: &[Expr], minterms: &[TermSpec], dontcares: &[TermSpec]) -> Expr {
    if minterms.is_empty() {
        return s::false_();
    }
    let (minterms, dontcares) = minterms_and_dontcares(variables, minterms, dontcares);
    let mut maxterms = Vec::new();
    for t in product01(variables.len()) {
        if !minterms.contains(&t) && !dontcares.contains(&t) {
            maxterms.push(t);
        }
    }
    let mut all = maxterms.clone();
    all.extend(dontcares);
    let new = simplified_pairs(all);
    let essential = rem_redundancy(&new, &maxterms);
    and(essential
        .iter()
        .map(|x| convert_to_vars_pos(x, variables))
        .collect())
}

/// `anf_coeffs(truthvalues)`: the coefficients of the Zhegalkin
/// polynomial of a truth table.
pub fn anf_coeffs(truthvalues: &[i64]) -> Vec<i64> {
    // s = '{:b}'.format(len(truthvalues)); n = len(s) - 1
    let len = truthvalues.len();
    let n = (usize::BITS - len.leading_zeros()).max(1) as usize - 1;
    if len != 1 << n {
        raise(
            ErrorKind::ValueError,
            format!("The number of truth values must be a power of two, got {len}"),
        );
    }
    let mut coeffs: Vec<Vec<i64>> = truthvalues.iter().map(|&v| vec![v]).collect();
    for i in 0..n {
        let mut tmp = Vec::new();
        for j in 0..1usize << (n - i - 1) {
            let (a, b) = (&coeffs[2 * j], &coeffs[2 * j + 1]);
            let mut row = a.clone();
            row.extend(a.iter().zip(b).map(|(x, y)| x ^ y));
            tmp.push(row);
        }
        coeffs = tmp;
    }
    coeffs.swap_remove(0)
}

/// `ANFform(variables, truthvalues)`: the algebraic normal form of a
/// truth table.
pub fn anf_form(variables: &[Expr], truthvalues: &[i64]) -> Expr {
    let n_vars = variables.len();
    let n_values = truthvalues.len();
    if u32::try_from(n_vars).map_or(true, |k| 1usize.checked_shl(k) != Some(n_values)) {
        raise(
            ErrorKind::ValueError,
            format!("The number of truth values must be equal to 2^{n_vars}, got {n_values}"),
        );
    }
    let coeffs = anf_coeffs(truthvalues);
    let mut terms = Vec::new();
    for (i, t) in product01(n_vars).into_iter().enumerate() {
        if coeffs[i] == 1 {
            terms.push(t);
        }
    }
    xor_with(
        terms
            .iter()
            .map(|x| convert_to_vars_anf(x, variables))
            .collect(),
        false,
    )
}

/// The `k` of `bool_minterm`, `bool_maxterm` and `bool_monomial`.
#[derive(Clone, Debug)]
pub enum TermIndex {
    /// A Python int: `ibin(k, len(variables))`.
    Int(Int),
    /// A list of bits.
    Bits(Vec<i64>),
}

impl TermIndex {
    fn bits(&self, variables: &[Expr]) -> Vec<i64> {
        match self {
            TermIndex::Int(k) => ibin(k, variables.len()),
            TermIndex::Bits(b) => b.clone(),
        }
    }
}

/// `bool_minterm(k, variables)`: the `k`-th minterm.
pub fn bool_minterm(k: &TermIndex, variables: &[Expr]) -> Expr {
    convert_to_vars_sop(&k.bits(variables), variables)
}

/// `bool_maxterm(k, variables)`: the `k`-th maxterm.
pub fn bool_maxterm(k: &TermIndex, variables: &[Expr]) -> Expr {
    convert_to_vars_pos(&k.bits(variables), variables)
}

/// `bool_monomial(k, variables)`: the `k`-th Zhegalkin monomial.
pub fn bool_monomial(k: &TermIndex, variables: &[Expr]) -> Expr {
    convert_to_vars_anf(&k.bits(variables), variables)
}

/// `gateinputcount(expr)`: the number of inputs of the logic gates
/// (And, Or, Xor, Nand, Nor, Not, Xnor, ITE) that realize `expr`.
pub fn gateinputcount(expr: &Expr) -> usize {
    if !expr.is_a(Tag::Boolean) {
        raise(ErrorKind::TypeError, "Expression must be Boolean");
    }
    // BooleanGates
    if matches!(
        expr.cls(),
        C::And | C::Or | C::Xor | C::Nand | C::Nor | C::Not | C::Xnor | C::ITE
    ) {
        return expr.args().len() + expr.args().iter().map(gateinputcount).sum::<usize>();
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::symbol;

    #[test]
    fn combinations_follow_itertools() {
        assert_eq!(
            combinations(4, 2),
            vec![
                vec![0, 1],
                vec![0, 2],
                vec![0, 3],
                vec![1, 2],
                vec![1, 3],
                vec![2, 3]
            ]
        );
        assert_eq!(combinations(3, 0), vec![Vec::<usize>::new()]);
        assert!(combinations(2, 3).is_empty());
    }

    #[test]
    fn lattice_basics() {
        let x = symbol("x");
        let y = symbol("y");
        assert_eq!(and(vec![x.clone(), s::true_()]), x);
        assert!(and(vec![x.clone(), s::false_()]).is_cls(C::BooleanFalse));
        assert!(or(vec![x.clone(), y.clone()]).is_cls(C::Or));
        assert!(and(vec![]).is_cls(C::BooleanTrue));
        assert_eq!(not(&not(&x)), x);
    }
}
