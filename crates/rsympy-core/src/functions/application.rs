//! `sympy.core.function`: `FunctionClass` (`nargs`), `Application.__new__`,
//! `Function._new_` (nargs checking, `eval`, the auto-evalf rule),
//! `Function.class_key`, `Function._eval_evalf`, `UndefinedFunction` and
//! `AppliedUndef`, and `WildFunction`.

use std::fmt;
use std::sync::Arc;

use crate::ctx;
use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::{Data, Expr, UndefData, WildFunctionData, WildFunctionName};
use crate::facts::{Fact, Kb};
use crate::functions::{Head, hook_owner};
use crate::generated::{ClassId as C, Tag};
use crate::generated_compare::{
    FUNCTION_CLASS_KEY_FUNCS, FUNCTION_CLASS_KEY_NATURALS0, FUNCTION_CLASS_KEY_OTHER,
};
use crate::generated_function::{FUNCTION_INFO, Hook};
use crate::py::PyDict;

// ---------------------------------------------------------------------------
// nargs

/// `nargs`: the numbers of arguments a function takes, `Naturals0` (any
/// number) or a `FiniteSet` of them.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Nargs {
    Naturals0,
    /// Sorted ascending (the order a `FiniteSet` of integers keeps). It is
    /// empty only for `WildFunction(name, nargs=())`, whose `nargs` is
    /// `EmptySet`.
    Finite(Vec<i64>),
}

impl Nargs {
    /// `FiniteSet(*nargs) if nargs else Naturals0`, for a canonical
    /// (`_nargs`-style) tuple.
    pub fn from_canonical(nargs: Option<Vec<i64>>) -> Nargs {
        match nargs {
            Some(v) if !v.is_empty() => Nargs::Finite(v),
            _ => Nargs::Naturals0,
        }
    }

    /// `n in nargs`.
    pub fn contains(&self, n: i64) -> bool {
        match self {
            Nargs::Naturals0 => n >= 0,
            Nargs::Finite(v) => v.contains(&n),
        }
    }
}

impl fmt::Display for Nargs {
    /// `str(nargs)`: `Naturals0`, `{1, 2}` or `EmptySet`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Nargs::Naturals0 => f.write_str("Naturals0"),
            Nargs::Finite(v) if v.is_empty() => f.write_str("EmptySet"),
            Nargs::Finite(v) => {
                let items: Vec<String> = v.iter().map(i64::to_string).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
        }
    }
}

/// The `nargs=` keyword as given (`Function('f', nargs=...)`,
/// `WildFunction('F', nargs=...)`): Python's `None`, an int, or a tuple of
/// ints, kept as given because `UndefinedFunction._kwargs` compares it that
/// way (`nargs=1` and `nargs=(1,)` are different functions).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NargsArg {
    None,
    Int(i64),
    Tuple(Vec<i64>),
}

impl NargsArg {
    /// The canonicalization `FunctionClass.__init__` applies:
    /// `tuple(ordered(set(nargs)))` for a sequence, `(as_int(nargs),)` for
    /// an int, `None` for `None`. A sequence must not be empty.
    pub fn function_class_nargs(&self) -> Option<Vec<i64>> {
        match self {
            NargsArg::None => None,
            NargsArg::Int(n) => Some(vec![*n]),
            NargsArg::Tuple(v) => {
                if v.is_empty() {
                    raise(
                        ErrorKind::ValueError,
                        "Incorrectly specified nargs as (): if there are no arguments, it \
                         should be `nargs = 0`; if there are any number of arguments, it \
                         should be `nargs = None`",
                    );
                }
                Some(sorted_set(v))
            }
        }
    }

    /// `repr(nargs)`, as `UndefinedFunction._kwargs` prints it.
    pub fn repr(&self) -> String {
        match self {
            NargsArg::None => "None".to_string(),
            NargsArg::Int(n) => n.to_string(),
            NargsArg::Tuple(v) if v.len() == 1 => format!("({},)", v[0]),
            NargsArg::Tuple(v) => {
                let items: Vec<String> = v.iter().map(i64::to_string).collect();
                format!("({})", items.join(", "))
            }
        }
    }
}

/// `tuple(ordered(set(v)))` for ints.
fn sorted_set(v: &[i64]) -> Vec<i64> {
    let mut v = v.to_vec();
    v.sort_unstable();
    v.dedup();
    v
}

/// The TypeError `Function._new_` raises for a wrong number of arguments.
fn nargs_type_error(name: &str, nargs: &Nargs, given: usize) -> ! {
    let Nargs::Finite(v) = nargs else {
        unreachable!("Naturals0 accepts any number of arguments")
    };
    let min = *v.iter().min().expect("a nonempty FiniteSet");
    let qual = if v.len() == 1 { "exactly" } else { "at least" };
    let plural = if min != 1 { "s" } else { "" };
    raise(
        ErrorKind::TypeError,
        format!("{name} takes {qual} {min} argument{plural} ({given} given)"),
    )
}

// ---------------------------------------------------------------------------
// function classes as values

/// A function class used as a value (`sin`, `Mod`, `Function('f')`): what
/// `expr.func` is for an applied function.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FunctionClass {
    /// A class of the class table.
    Defined(C),
    /// An `UndefinedFunction` (`Function('f', **kwargs)`); equality is
    /// `UndefinedFunction.__eq__` (same `class_key()` and `_kwargs`).
    Undefined(Arc<UndefData>),
}

impl FunctionClass {
    /// The class's head.
    pub fn head(&self) -> &dyn Head {
        match self {
            FunctionClass::Defined(c) => crate::functions::head(*c),
            FunctionClass::Undefined(u) => u,
        }
    }

    /// `cls(*args)`.
    pub fn call(&self, args: Vec<Expr>) -> Expr {
        self.head().new(args, None)
    }

    /// `cls(*args, evaluate=evaluate)`.
    pub fn call_with(&self, args: Vec<Expr>, evaluate: Option<bool>) -> Expr {
        self.head().new(args, evaluate)
    }

    /// `e.func` for an applied function `e` (`None` for other expressions,
    /// whose `func` is not a function class).
    pub fn of(e: &Expr) -> Option<FunctionClass> {
        match e.data() {
            Data::Undef(u) => Some(FunctionClass::Undefined(u.clone())),
            _ if e.is_a(Tag::Application) => Some(FunctionClass::Defined(e.cls())),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// construction

/// `Function._new_(cls, *args, evaluate=evaluate)`: the nargs check, then
/// `Application.__new__`, then the auto-evalf rule.
pub fn function_new<H: Head + ?Sized>(h: &H, args: Vec<Expr>, evaluate: Option<bool>) -> Expr {
    let n = args.len();

    if !h.valid_nargs(n) {
        // XXX: exception message must be in exactly this format to
        // make it work with NumPy's functions like vectorize(). See,
        // for example, https://github.com/numpy/numpy/issues/1697.
        // The ideal solution would be just to attach metadata to
        // the exception and change NumPy to take advantage of this.
        nargs_type_error(h.name(), &h.nargs(), n);
    }

    let evaluate = evaluate.unwrap_or_else(ctx::evaluate);
    let mut result = application_new(h, args, Some(evaluate));
    if evaluate && h.is_instance(&result) && !result.args().is_empty() {
        let should_evalf: Vec<i64> = result.args().iter().map(|a| h.should_evalf(a)).collect();
        let pr2 = *should_evalf.iter().min().expect("args");
        if pr2 > 0 {
            let pr = *should_evalf.iter().max().expect("args");
            result = crate::evalf::evalf(&result, rsympy_mpf::prec_to_dps(pr));
        }
    }
    result
}

/// `Application.__new__(cls, *args, evaluate=evaluate)`: `cls.eval(*args)`
/// when evaluating, else (or when it returns `None`) the bare node.
///
/// The node's `nargs` attribute is not stored: [`Head::instance_nargs`]
/// derives it from the class (and a `WildFunction` keeps its own).
pub fn application_new<H: Head + ?Sized>(h: &H, args: Vec<Expr>, evaluate: Option<bool>) -> Expr {
    let evaluate = evaluate.unwrap_or_else(ctx::evaluate);
    if evaluate && let Some(evaluated) = h.eval(&args) {
        return evaluated;
    }
    h.raw(args)
}

/// `Function._should_evalf(arg)`: the precision of a Float argument (or
/// of a pure complex number with Float parts), else -1.
pub fn function_should_evalf(arg: &Expr) -> i64 {
    if let Some((_, prec)) = arg.as_float() {
        return prec;
    }
    if !arg.is_Add() {
        return -1;
    }
    let Some((re, im)) = crate::evalf::pure_complex(arg, false) else {
        return -1;
    };
    // the elements of m are of type Number, so have a _prec
    crate::numbers::num_prec(&re).max(crate::numbers::num_prec(&im))
}

/// `Function.class_key()` for a class named `name` with this `nargs`.
pub fn function_class_key(name: &str, nargs: &Nargs) -> (i32, i32, String) {
    let i = match FUNCTION_CLASS_KEY_FUNCS.iter().find(|(n, _)| *n == name) {
        Some(&(_, i)) => i,
        None if *nargs == Nargs::Naturals0 => FUNCTION_CLASS_KEY_NATURALS0,
        None => FUNCTION_CLASS_KEY_OTHER,
    };
    (4, i, name.to_string())
}

/// `Function._eval_evalf(prec)`.
pub fn function_eval_evalf<H: Head + ?Sized>(h: &H, e: &Expr, prec: i64) -> Option<Expr> {
    let _ = prec;
    // _eval_mpmath = getattr(self, '_eval_mpmath', None)
    let eval_mpmath = hook_owner(h, Hook::EvalMpmath);
    if !eval_mpmath.is_empty() {
        not_ported(format!("{eval_mpmath}._eval_mpmath"));
    }
    // _get_mpmath_func(self.func.__name__): None for an AppliedUndef
    // ("Shouldn't lookup in mpmath but might have ._imp_")
    let func = if e.is_a(Tag::AppliedUndef) {
        ""
    } else {
        FUNCTION_INFO[h.class_id() as usize].mpmath_func
    };
    if func.is_empty() {
        // Fall-back evaluation
        let imp = hook_owner(h, Hook::Imp);
        if !imp.is_empty() {
            not_ported(format!("{imp}._imp_"));
        }
        return None;
    }
    not_ported(format!("Function._eval_evalf: mpmath.{func}"))
}

// ---------------------------------------------------------------------------
// UndefinedFunction and AppliedUndef

/// `UndefinedFunction(name, **kwargs)` for a string name: `Function(name,
/// **assumptions, nargs=nargs)` (`nargs` not given: `None`).
pub fn undefined_function(
    name: &str,
    assumptions: &[(Fact, bool)],
    nargs: Option<NargsArg>,
) -> Arc<UndefData> {
    // commutative = assumptions.get('commutative', None)
    // assumptions = Symbol(name, **assumptions).assumptions0
    // if commutative is None: assumptions.pop('commutative')
    let sym = crate::symbol::symbol_with(name, assumptions);
    let mut kwargs = sym.symbol_data().expect("a Symbol").assumptions0;
    if !assumptions.iter().any(|(f, _)| *f == Fact::Commutative) {
        kwargs.remove(Fact::Commutative);
    }
    make_undefined(name, kwargs, nargs)
}

/// `UndefinedFunction(sym, **kwargs)` for a Symbol `sym`: the function
/// takes its name, and `sym._merge(assumptions)` (its `assumptions0`,
/// commutative included, updated with the given ones).
pub fn undefined_function_from_symbol(
    sym: &Expr,
    assumptions: &[(Fact, bool)],
    nargs: Option<NargsArg>,
) -> Arc<UndefData> {
    let Some(d) = sym.symbol_data() else {
        raise(ErrorKind::TypeError, "expecting string or Symbol for name")
    };
    // Symbol._merge: base = self.assumptions0, checked against the new ones
    let mut base = d.assumptions0;
    for &(k, v) in assumptions {
        if let Some(old) = base.get(k)
            && old != v
        {
            raise(
                ErrorKind::ValueError,
                format!(
                    "non-matching assumptions for {}: existing value is {} and new value is {}",
                    k.name(),
                    py_bool(old),
                    py_bool(v)
                ),
            );
        }
    }
    // base.update(assumptions): a dict update, no deduction
    for &(k, v) in assumptions {
        base.tell(k, Some(v));
    }
    make_undefined(&d.name, base, nargs)
}

fn py_bool(b: bool) -> &'static str {
    if b { "True" } else { "False" }
}

/// The rest of `UndefinedFunction.__new__` and `FunctionClass.__init__`:
/// the class's `is_<fact>` attributes become its `default_assumptions`
/// (`StdFactKB` of them; `AppliedUndef` adds none of its own), and `nargs`
/// is canonicalized (raising for an empty tuple).
fn make_undefined(name: &str, kwargs: Kb, nargs: Option<NargsArg>) -> Arc<UndefData> {
    let assumptions = Kb::deduced(&kwargs.items(false));
    if let Some(n) = &nargs {
        n.function_class_nargs();
    }
    Arc::new(UndefData {
        name: Arc::from(name),
        kwargs,
        assumptions,
        nargs,
    })
}

impl Head for Arc<UndefData> {
    fn class_id(&self) -> C {
        C::AppliedUndef
    }

    fn name(&self) -> &str {
        &self.name
    }

    /// `FunctionClass.__init__`: the `nargs` keyword, else `arity(cls)`
    /// (`None`: `Application.eval` takes `*args`), else `AppliedUndef._nargs`
    /// (`None`).
    fn class_nargs(&self) -> Option<Vec<i64>> {
        self.nargs.as_ref().and_then(NargsArg::function_class_nargs)
    }

    /// `Application.__new__` reads the `nargs` class attribute when it was
    /// given and canonicalizes it as `__init__` did, else uses `_nargs`:
    /// the same value either way.
    fn instance_nargs(&self) -> Nargs {
        self.nargs()
    }

    fn class_key(&self) -> (i32, i32, String) {
        function_class_key(&self.name, &self.nargs())
    }

    /// `AppliedUndef.__new__`: an `UndefinedFunction` (a class) cannot be an
    /// argument, which an `Expr` never is; then `Function.__new__`.
    fn new(&self, args: Vec<Expr>, evaluate: Option<bool>) -> Expr {
        function_new(self, args, evaluate)
    }

    fn raw(&self, args: Vec<Expr>) -> Expr {
        Expr::make_with_kb(
            C::AppliedUndef,
            Data::Undef(self.clone()),
            args,
            self.assumptions,
        )
    }

    /// `UndefinedFunction.__instancecheck__`: `cls in type(e).__mro__`,
    /// which compares classes with `UndefinedFunction.__eq__`.
    fn is_instance(&self, e: &Expr) -> bool {
        e.undef_data().is_some_and(|u| **u == **self)
    }
}

/// `f.nargs` for an undefined function.
pub fn undefined_nargs(f: &Arc<UndefData>) -> Nargs {
    f.nargs()
}

/// `repr` of an undefined function's `_kwargs` items, sorted by key, as
/// `sorted(f._kwargs.items())` lists them: `[("nargs", "1"), ("real",
/// "True"), ...]`.
pub fn undefined_kwargs_repr(f: &UndefData) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = f
        .kwargs
        .items(false)
        .into_iter()
        .map(|(k, v)| {
            let v = v.expect("items(false) has no None");
            (k.name().to_string(), py_bool(v).to_string())
        })
        .collect();
    if let Some(n) = &f.nargs {
        out.push(("nargs".to_string(), n.repr()));
    }
    out.sort();
    out
}

// ---------------------------------------------------------------------------
// WildFunction

/// The head of `WildFunction`.
pub struct WildFunctionHead;

impl Head for WildFunctionHead {
    fn class_id(&self) -> C {
        C::WildFunction
    }

    /// `WildFunction(*args)`, as `func(*args)` rebuilds one: `Function.__new__`
    /// with the args, then `__init__(obj, *args)`, whose `name` is the first
    /// (sympified) argument and whose `nargs` is `S.Naturals0`.
    fn new(&self, args: Vec<Expr>, evaluate: Option<bool>) -> Expr {
        let Some(name) = args.first().cloned() else {
            raise(
                ErrorKind::TypeError,
                "WildFunction.__init__() missing 1 required positional argument: 'name'",
            )
        };
        if args.len() > 1 {
            raise(
                ErrorKind::TypeError,
                "WildFunction.__init__() takes 2 positional arguments",
            );
        }
        let obj = function_new(self, args, evaluate);
        wild_function_init(&obj, WildFunctionName::Expr(name), Nargs::Naturals0)
    }

    /// `Basic.__new__(WildFunction, *args)`: no `__init__` runs, so the
    /// instance has no `name` or `nargs` attribute.
    fn raw(&self, args: Vec<Expr>) -> Expr {
        wild_function_raw(args)
    }
}

/// `Basic.__new__(WildFunction, *args)`.
pub(crate) fn wild_function_raw(args: Vec<Expr>) -> Expr {
    Expr::make(
        C::WildFunction,
        Data::WildFunction(Arc::new(WildFunctionData {
            name: WildFunctionName::Unset,
            nargs: None,
        })),
        args,
    )
}

/// `WildFunction.__init__(obj, name, nargs=...)`: sets the instance's
/// `name` and `nargs` (the node is rebuilt with them; they are not part of
/// its identity). `__init__` runs on `obj` whatever `__new__` returned, as
/// long as it is a WildFunction.
fn wild_function_init(obj: &Expr, name: WildFunctionName, nargs: Nargs) -> Expr {
    if !obj.is_cls(C::WildFunction) {
        return obj.clone();
    }
    Expr::make_with_kb(
        C::WildFunction,
        Data::WildFunction(Arc::new(WildFunctionData {
            name,
            nargs: Some(nargs),
        })),
        obj.args().to_vec(),
        obj.assumptions_kb(),
    )
}

/// `WildFunction(name, nargs=nargs)` (`nargs` not given: `Naturals0`).
pub fn wild_function(name: &str, nargs: Option<NargsArg>) -> Expr {
    // Function.__new__ -> Function._new_ -> Application.__new__, which
    // sympifies the name and drops `nargs`
    let arg = crate::sympify::sympify_str(name, ctx::evaluate());
    let obj = function_new(&WildFunctionHead, vec![arg], None);
    // WildFunction.__init__
    let nargs = match nargs {
        None => Nargs::Naturals0,
        // FiniteSet(*None)
        Some(NargsArg::None) => raise(
            ErrorKind::TypeError,
            "sympy.sets.sets.FiniteSet() argument after * must be an iterable, not NoneType",
        ),
        Some(NargsArg::Int(n)) => Nargs::Finite(vec![n]),
        // (an empty tuple gives FiniteSet() = EmptySet)
        Some(NargsArg::Tuple(v)) => Nargs::Finite(sorted_set(&v)),
    };
    wild_function_init(&obj, WildFunctionName::Str(Arc::from(name)), nargs)
}

/// A `WildFunction`'s `nargs` attribute.
pub fn wild_function_nargs(w: &Expr) -> Nargs {
    match &w.wild_function_data().expect("a WildFunction").nargs {
        Some(n) => n.clone(),
        None => raise(
            ErrorKind::AttributeError,
            "'WildFunction' object has no attribute 'nargs'",
        ),
    }
}

/// `WildFunction.matches(expr, repl_dict)`.
pub fn wild_function_matches(
    w: &Expr,
    expr: &Expr,
    repl_dict: Option<&PyDict<Expr, Expr>>,
) -> Option<PyDict<Expr, Expr>> {
    if !(expr.is_a(Tag::AppliedUndef) || expr.is_a(Tag::Function)) {
        return None;
    }
    if !wild_function_nargs(w).contains(expr.args().len() as i64) {
        return None;
    }
    let mut repl_dict = repl_dict.cloned().unwrap_or_default();
    repl_dict.insert(w.clone(), expr.clone());
    Some(repl_dict)
}

/// `obj.nargs` for an applied function.
pub fn nargs_of(e: &Expr) -> Nargs {
    if e.wild_function_data().is_some() {
        return wild_function_nargs(e);
    }
    crate::functions::head_of(e).instance_nargs()
}
