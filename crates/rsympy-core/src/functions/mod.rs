//! `sympy.core.function` (the function framework: `Application`,
//! `Function`, `AppliedUndef`, `UndefinedFunction`, `WildFunction`,
//! `Lambda`), `sympy.core.mod.Mod`, and the `functions.elementary` kernel.
//!
//! # The `Head` trait
//!
//! Every SymPy function class is a *head*: the object that builds `f(*args)`,
//! evaluates it, and answers questions about the applied expressions of its
//! class. That covers every `Application` subclass: `Function` and its
//! defined subclasses (`Mod`, `sin`, ...), each undefined function
//! `Function('f')` (a class of its own in SymPy), `WildFunction`, and the
//! non-`Function` `Application`s (`Min`, `Max`, `And`, `Not`, ...).
//!
//! ## Who implements it
//!
//! - **Defined classes** (a [`ClassId`]): one zero-sized struct per ported
//!   class (`core_mod::ModHead`), returned by [`head`]. A class that has no
//!   port yet gets [`GenericHead`], whose every method is the inherited
//!   behaviour guarded by the table below, so using it raises
//!   `NotPortedYet` naming the first SymPy method that is missing.
//! - **Undefined functions**: [`UndefData`] (the payload of
//!   `Function('f', **kwargs)`, stored in each `f(x)` node) implements
//!   `Head` itself, so `f`'s name, assumptions and `nargs` travel with the
//!   node.
//! - **Foreign heads** (for `rsympy-py`): users' own `Function` subclasses,
//!   and SymPy classes not ported yet while Python serves them, are an
//!   `Arc<dyn Head>` whose methods call back into Python. They will sit in
//!   the node's payload exactly like `UndefData` (a new `Data` variant) and
//!   [`head_of`] will return them; nothing else changes, because all of core
//!   reaches a head through [`head_of`] (an applied expression) or
//!   [`FunctionClass::head`] (a class used as a value).
//!
//! ## Inheritance by table
//!
//! Each trait method's default is what SymPy's MRO would run when the class
//! does not override it (the `Function`, `Application`, `Expr` or `Basic`
//! method). Before running it, the default looks up which class defines the
//! method for this class: `FUNCTION_INFO[cls].hooks` in
//! `generated_function.rs` (written by `oracle/gen_tables_function.py`) and,
//! for assumptions, `ClassInfo::handler_owners`. When SymPy would find an
//! override that the Rust impl does not provide, the default raises
//! `NotPortedYet` naming it (`sin.eval`, `TrigonometricFunction._eval_is_rational`).
//! So porting a class means implementing the methods its table row names,
//! and a forgotten one fails loudly rather than answering with the base
//! class's behaviour. Intermediate bases (`ExpBase`, `TrigonometricFunction`,
//! `RoundFunction`, `MinMaxBase`) are free functions that each subclass's
//! impl calls for the hooks whose owner is that base. Every default is also
//! available as a free function (`default_eval_is`, ...), which is how an
//! override calls "super".
//!
//! ## The hooks
//!
//! | SymPy | `Head` method | default |
//! |---|---|---|
//! | `cls.__name__`, `cls.class_key()` | [`Head::name`], [`Head::class_key`] | table; `Function.class_key` |
//! | `cls._nargs`, `cls.nargs`, `obj.nargs` | [`Head::nargs`], [`Head::instance_nargs`] | table |
//! | `cls(*args, evaluate=...)` | [`Head::new`] | `Function._new_` + `Application.__new__` ([`application::function_new`]) |
//! | `cls.eval(*args)` | [`Head::eval`] | `Application.eval`: `None` |
//! | `cls._should_evalf(arg)` | [`Head::should_evalf`] | `Function._should_evalf` |
//! | `Basic.__new__(cls, *args)` | [`Head::raw`] | the node with the class's assumptions |
//! | `isinstance(e, cls)` | [`Head::is_instance`] | table (`supers`) |
//! | `_eval_is_<fact>` | [`Head::eval_is`] | `Function._eval_is_commutative`, `Expr._eval_is_extended_positive`/`negative` |
//! | `fdiff(argindex)` | [`Head::fdiff`] | `Function.fdiff` (needs `Derivative`/`Subs`) |
//! | `inverse(argindex)` | [`Head::inverse`] | `AttributeError` (no base class has it) |
//! | `_eval_subs(old, new)` | [`Head::eval_subs`] | `Application._eval_subs` |
//! | `_eval_evalf(prec)` | [`Head::eval_evalf`] | `Function._eval_evalf` |
//! | `_eval_conjugate`, `_eval_transpose`, `_eval_adjoint` | [`Head::eval_conjugate`], ... | `Expr`'s |
//! | `_eval_power(e)` | [`Head::eval_power`] | `Expr._eval_power`: `None` |
//! | `as_base_exp()`, `as_numer_denom()` | [`Head::as_base_exp`], [`Head::as_numer_denom`] | `Expr`'s: `(self, 1)` |
//! | `_eval_derivative`, `_eval_nseries`, `_eval_as_leading_term`, `_eval_simplify` | same names | not ported yet |
//! | `_eval_rewrite(rule, args)` | [`Head::eval_rewrite`] | `Basic._eval_rewrite`: `None` |
//! | `_eval_rewrite_as_<target>` | [`Head::eval_rewrite_as`] | the rewrite registry |
//! | `_eval_expand_<hint>` | [`Head::eval_expand`] | the expand registry |
//! | `_sympystr` (and later `_latex`, `_pretty`) | [`Head::sympystr`] | none: the printer's `_print_<Class>` |
//!
//! ## Open registries
//!
//! `_eval_rewrite_as_<target>` and `_eval_expand_<hint>` are entries of two
//! registries keyed by *(defining class, target or hint)*, which is how
//! Python finds them through the MRO: the table says which class defines
//! the method for a given class (`FUNCTION_INFO[cls].rewrites`), and the
//! registry maps that owner to the Rust function. Ported methods are listed
//! in [`BUILTIN_REWRITES`] and [`BUILTIN_EXPANDS`]; crates outside core (and
//! `rsympy-py`) add theirs at startup with [`register_rewrite`] and
//! [`register_expand`]. A method SymPy has but nobody registered raises
//! `NotPortedYet`.
//!
//! ## Who calls the heads
//!
//! `assume.rs` sends the `_eval_is_*` handlers of every class it does not
//! own to [`handler`]; `construct.rs` rebuilds applied expressions through
//! [`call`] and [`call_undefined`]; `methods.rs` (`as_base_exp`,
//! `as_numer_denom`) and `power.rs` (`_eval_power`, through [`eval_power`])
//! dispatch Application instances to their head.
//!
//! Submodules mirror SymPy's files:
//! - `application`: `core.function`'s `FunctionClass`, `Application`,
//!   `Function`, `UndefinedFunction`, `AppliedUndef`, `WildFunction`;
//! - `lambda`: `core.function.Lambda`;
//! - `core_mod`: `core.mod.Mod`;
//! - `misc`: `functions.elementary.miscellaneous` (`IdentityFunction`; later
//!   `sqrt`, `root`, `Min`, `Max`);
//! - later: `exponential` (`exp`, `log`), `trigonometric`, `complexes` (`re`,
//!   `im`, `Abs`, `sign`, `arg`, `conjugate`), `integers` (`floor`,
//!   `ceiling`), `combinatorial`.

pub mod application;
pub mod core_mod;
pub mod lambda;
pub mod misc;

use std::sync::{Arc, LazyLock, RwLock};

use rsympy_num::Int;

use crate::assume::fuzzy_and;
use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::{Data, Expr, UndefData};
use crate::facts::Fact;
use crate::generated::{CLASSES, ClassId, NUM_CLASSES, Tag};
use crate::generated_function::{FUNCTION_INFO, FunctionInfo, Hook};
use crate::numbers::s;

pub use application::{FunctionClass, Nargs, NargsArg};

// ---------------------------------------------------------------------------
// the trait

/// A SymPy function class: see the module docs.
///
/// Methods without an instance argument are class methods (`&self` is the
/// class); the others take the applied expression `e`, an instance of the
/// class.
pub trait Head: Send + Sync {
    /// The class in the class table. Undefined functions are
    /// `ClassId::AppliedUndef`.
    fn class_id(&self) -> ClassId;

    /// `cls.__name__`.
    fn name(&self) -> &str {
        CLASSES[self.class_id() as usize].name
    }

    /// `cls._nargs`, as `FunctionClass.__init__` canonicalized it: the
    /// allowed numbers of arguments, sorted, or `None` for any number.
    fn class_nargs(&self) -> Option<Vec<i64>> {
        info(self).nargs.map(<[i64]>::to_vec)
    }

    /// `cls.nargs` (`FunctionClass.nargs`): `FiniteSet(*cls._nargs)`, or
    /// `Naturals0` when `_nargs` is empty or `None`.
    fn nargs(&self) -> Nargs {
        Nargs::from_canonical(self.class_nargs())
    }

    /// The `nargs` that `Application.__new__` gives each instance.
    fn instance_nargs(&self) -> Nargs {
        Nargs::from_canonical(info(self).instance_nargs.map(<[i64]>::to_vec))
    }

    /// `cls._valid_nargs(n)`.
    fn valid_nargs(&self, n: usize) -> bool {
        match self.class_nargs() {
            Some(v) if !v.is_empty() => v.contains(&(n as i64)),
            _ => true,
        }
    }

    /// `cls.class_key()`.
    fn class_key(&self) -> (i32, i32, String) {
        default_class_key(self)
    }

    /// `cls(*args, evaluate=evaluate)`; `None` uses `global_parameters`.
    fn new(&self, args: Vec<Expr>, evaluate: Option<bool>) -> Expr {
        match hook_owner(self, Hook::New) {
            "DefinedFunction" | "Function" => application::function_new(self, args, evaluate),
            "Application" => application::application_new(self, args, evaluate),
            o => not_ported(format!("{o}.__new__")),
        }
    }

    /// `cls.eval(*args)`: the canonical form, or `None` to leave
    /// `cls(*args)` unevaluated.
    fn eval(&self, args: &[Expr]) -> Option<Expr> {
        let _ = args;
        match hook_owner(self, Hook::Eval) {
            "Application" => None,
            o => not_ported(format!("{o}.eval")),
        }
    }

    /// `cls._should_evalf(arg)`: the precision to evalf `cls(*args)` to, or
    /// a value `<= 0` for no evalf.
    fn should_evalf(&self, arg: &Expr) -> i64 {
        match hook_owner(self, Hook::ShouldEvalf) {
            "Function" => application::function_should_evalf(arg),
            o => not_ported(format!("{o}._should_evalf")),
        }
    }

    /// `Basic.__new__(cls, *args)`: the node, with the class's
    /// `default_assumptions` and no evaluation.
    fn raw(&self, args: Vec<Expr>) -> Expr {
        crate::construct::raw(self.class_id(), args)
    }

    /// `isinstance(e, cls)`.
    fn is_instance(&self, e: &Expr) -> bool {
        FUNCTION_INFO[e.cls() as usize]
            .supers
            .contains(&self.class_id())
    }

    /// `e._eval_is_<fact>()`, for a fact the class has a handler for.
    fn eval_is(&self, e: &Expr, fact: Fact) -> Option<bool> {
        default_eval_is(e, fact)
    }

    /// `e.fdiff(argindex)`.
    fn fdiff(&self, e: &Expr, argindex: usize) -> Expr {
        default_fdiff(self, e, argindex)
    }

    /// `e.inverse(argindex)`.
    fn inverse(&self, e: &Expr, argindex: usize) -> FunctionClass {
        let _ = argindex;
        match hook_owner(self, Hook::Inverse) {
            "" => raise(
                ErrorKind::AttributeError,
                format!("'{}' object has no attribute 'inverse'", e.class_name()),
            ),
            o => not_ported(format!("{o}.inverse")),
        }
    }

    /// `e._eval_subs(old, new)`.
    fn eval_subs(&self, e: &Expr, old: &Expr, new: &Expr) -> Option<Expr> {
        default_eval_subs(self, e, old, new)
    }

    /// `e._eval_evalf(prec)` (binary precision).
    fn eval_evalf(&self, e: &Expr, prec: i64) -> Option<Expr> {
        default_eval_evalf(self, e, prec)
    }

    /// `e._eval_conjugate()`.
    fn eval_conjugate(&self, e: &Expr) -> Option<Expr> {
        match hook_owner(self, Hook::EvalConjugate) {
            "Expr" => expr_eval_conjugate(e),
            o => not_ported(format!("{o}._eval_conjugate")),
        }
    }

    /// `e._eval_transpose()`.
    fn eval_transpose(&self, e: &Expr) -> Option<Expr> {
        match hook_owner(self, Hook::EvalTranspose) {
            "Expr" => expr_eval_transpose(e),
            o => not_ported(format!("{o}._eval_transpose")),
        }
    }

    /// `e._eval_adjoint()`.
    fn eval_adjoint(&self, e: &Expr) -> Option<Expr> {
        match hook_owner(self, Hook::EvalAdjoint) {
            "Expr" => expr_eval_adjoint(self, e),
            o => not_ported(format!("{o}._eval_adjoint")),
        }
    }

    /// `e._eval_power(exp)`: `None` leaves `Pow(e, exp)` unevaluated.
    fn eval_power(&self, e: &Expr, exp: &Expr) -> Option<Expr> {
        let _ = (e, exp);
        match hook_owner(self, Hook::EvalPower) {
            "Expr" => None,
            o => not_ported(format!("{o}._eval_power")),
        }
    }

    /// `e.as_base_exp()`.
    fn as_base_exp(&self, e: &Expr) -> (Expr, Expr) {
        match hook_owner(self, Hook::AsBaseExp) {
            "Expr" => (e.clone(), s::one()),
            o => not_ported(format!("{o}.as_base_exp")),
        }
    }

    /// `e.as_numer_denom()`.
    fn as_numer_denom(&self, e: &Expr) -> (Expr, Expr) {
        match hook_owner(self, Hook::AsNumerDenom) {
            "Expr" => (e.clone(), s::one()),
            o => not_ported(format!("{o}.as_numer_denom")),
        }
    }

    /// `e._eval_derivative(s)`.
    fn eval_derivative(&self, e: &Expr, sym: &Expr) -> Option<Expr> {
        let _ = (e, sym);
        not_ported(format!(
            "{}._eval_derivative",
            hook_owner(self, Hook::EvalDerivative)
        ))
    }

    /// `e._eval_nseries(x, n, logx, cdir)`.
    fn eval_nseries(&self, e: &Expr, x: &Expr, n: i64, logx: Option<&Expr>) -> Expr {
        let _ = (e, x, n, logx);
        not_ported(format!(
            "{}._eval_nseries",
            hook_owner(self, Hook::EvalNseries)
        ))
    }

    /// `e._eval_as_leading_term(x, logx, cdir)`.
    fn eval_as_leading_term(&self, e: &Expr, x: &Expr, logx: Option<&Expr>) -> Expr {
        let _ = (e, x, logx);
        not_ported(format!(
            "{}._eval_as_leading_term",
            hook_owner(self, Hook::EvalAsLeadingTerm)
        ))
    }

    /// `e._eval_simplify(**kwargs)`.
    fn eval_simplify(&self, e: &Expr) -> Expr {
        match hook_owner(self, Hook::EvalSimplify) {
            o => not_ported(format!("{o}._eval_simplify ({})", e.class_name())),
        }
    }

    /// `e._eval_rewrite(rule, args, **hints)`, the hook `Basic._rewrite`
    /// tries when the class has no `_eval_rewrite_as_<rule>` method.
    fn eval_rewrite(&self, e: &Expr, rule: &str, args: &[Expr], hints: &Hints) -> Option<Expr> {
        default_eval_rewrite(e, rule, args, hints)
    }

    /// `e._eval_rewrite_as_<target>(*args, **hints)`: `None` when the class
    /// has no such method, else what it returned.
    fn eval_rewrite_as(
        &self,
        e: &Expr,
        target: &str,
        args: &[Expr],
        hints: &Hints,
    ) -> Option<Option<Expr>> {
        default_eval_rewrite_as(e, target, args, hints)
    }

    /// `e._eval_expand_<hint>(**hints)`: `None` when the class has no such
    /// method, else what it returned.
    fn eval_expand(&self, e: &Expr, hint: &str, hints: &Hints) -> Option<Expr> {
        default_eval_expand(e, hint, hints)
    }

    /// `e._sympystr(printer)`: `None` when the class does not print itself
    /// (the printer then uses its `_print_<Class>` method).
    fn sympystr(&self, e: &Expr, printer: &crate::printing::str::StrPrinter) -> Option<String> {
        let _ = (e, printer);
        match hook_owner(self, Hook::Sympystr) {
            "" => None,
            o => not_ported(format!("{o}._sympystr")),
        }
    }
}

fn info<H: Head + ?Sized>(h: &H) -> &'static FunctionInfo {
    &FUNCTION_INFO[h.class_id() as usize]
}

/// The class that defines `hook` for `h`'s class ("" if none).
pub fn hook_owner<H: Head + ?Sized>(h: &H, hook: Hook) -> &'static str {
    info(h).hooks[hook as usize]
}

// ---------------------------------------------------------------------------
// the registry of heads

/// The head of a class with no Rust port yet: everything it inherits from
/// `Function`/`Application`/`Expr`, and `NotPortedYet` for anything the
/// class overrides.
pub struct GenericHead(pub ClassId);

impl Head for GenericHead {
    fn class_id(&self) -> ClassId {
        self.0
    }
}

static GENERIC: LazyLock<Vec<GenericHead>> = LazyLock::new(|| {
    crate::generated::ALL_CLASSES
        .iter()
        .map(|&c| GenericHead(c))
        .collect()
});

/// The head of a defined class.
pub fn head(cls: ClassId) -> &'static dyn Head {
    match cls {
        ClassId::Mod => &core_mod::ModHead,
        ClassId::WildFunction => &application::WildFunctionHead,
        _ => &GENERIC[cls as usize],
    }
}

/// The head of an applied expression's class: its undefined function, or
/// its defined class's head.
pub fn head_of(e: &Expr) -> &dyn Head {
    match e.data() {
        Data::Undef(u) => &**u,
        _ => head(e.cls()),
    }
}

const _: () = assert!(NUM_CLASSES == crate::generated::ALL_CLASSES.len());

// ---------------------------------------------------------------------------
// the existing entry points

/// `cls(*args)`: `Function.__new__` for a defined function class, including
/// its `eval`.
pub fn call(cls: ClassId, args: Vec<Expr>) -> Expr {
    head(cls).new(args, None)
}

/// `cls(*args, evaluate=evaluate)` (`None`: `global_parameters.evaluate`).
pub fn call_with(cls: ClassId, args: Vec<Expr>, evaluate: Option<bool>) -> Expr {
    head(cls).new(args, evaluate)
}

/// `Function(name, **assumptions)` (`UndefinedFunction.__new__` for a
/// string name).
pub fn undefined_function(name: &str, assumptions: &[(Fact, bool)]) -> Arc<UndefData> {
    application::undefined_function(name, assumptions, None)
}

/// `f(*args)` for an undefined function `f = Function(name)`.
pub fn call_undefined(f: &Arc<UndefData>, args: Vec<Expr>) -> Expr {
    f.new(args, None)
}

/// `f(*args, evaluate=evaluate)` for an undefined function.
pub fn call_undefined_with(f: &Arc<UndefData>, args: Vec<Expr>, evaluate: Option<bool>) -> Expr {
    f.new(args, evaluate)
}

/// `exp(x)`.
pub fn exp(x: &Expr) -> Expr {
    call(ClassId::Exp, vec![x.clone()])
}

/// `exp.as_base_exp()`.
pub fn exp_as_base_exp(e: &Expr) -> (Expr, Expr) {
    let _ = e;
    not_ported("exp.as_base_exp")
}

/// `exp.as_numer_denom()`.
pub fn exp_as_numer_denom(e: &Expr) -> (Expr, Expr) {
    let _ = e;
    not_ported("exp.as_numer_denom")
}

/// `base._eval_power(exp)` for a class outside `core` that defines it
/// (`owner` is that class: `ExpBase`, `Abs`, `sign`, `Piecewise`, ...).
pub fn eval_power(owner: &str, base: &Expr, exp: &Expr) -> Option<Expr> {
    if base.is_a(Tag::Application) {
        return head_of(base).eval_power(base, exp);
    }
    not_ported(format!("{owner}._eval_power"))
}

/// The `_eval_is_<fact>` handlers of function classes, of `Lambda`, and of
/// the numeric singletons (which only have `Expr`'s).
pub(crate) fn handler(e: &Expr, fact: Fact) -> Option<bool> {
    if e.is_a(Tag::Application) {
        return head_of(e).eval_is(e, fact);
    }
    match e.cls() {
        ClassId::Lambda | ClassId::IdentityFunction => lambda::handler(e, fact),
        _ => crate::numbers::singleton_handler(e, fact),
    }
}

// ---------------------------------------------------------------------------
// the inherited methods (the trait's defaults, callable as "super")

/// `Function.class_key()` for a `Function` subclass, else the class
/// table's key.
pub fn default_class_key<H: Head + ?Sized>(h: &H) -> (i32, i32, String) {
    match hook_owner(h, Hook::ClassKey) {
        "Function" => application::function_class_key(h.name(), &h.nargs()),
        _ => {
            let (a, b) = CLASSES[h.class_id() as usize].class_key;
            let name = crate::generated_compare::COMPARE_INFO[h.class_id() as usize]
                .class_key
                .2;
            (a, b, name.to_string())
        }
    }
}

/// The `_eval_is_<fact>` handler SymPy's `_prop_handler` holds for `e`'s
/// class, when it is one every function inherits.
pub fn default_eval_is(e: &Expr, fact: Fact) -> Option<bool> {
    match e.handler_owner(fact) {
        Some("Function._eval_is_commutative") => function_eval_is_commutative(e),
        Some("Expr._eval_is_extended_positive" | "Expr._eval_is_extended_negative") => {
            expr_eval_is_extended_positive_negative(e)
        }
        Some(owner) => not_ported(owner),
        None => unreachable!("{} has no handler for {fact:?}", e.class_name()),
    }
}

/// `Function._eval_is_commutative`.
pub fn function_eval_is_commutative(e: &Expr) -> Option<bool> {
    fuzzy_and(e.args().iter().map(Expr::is_commutative))
}

/// `Expr._eval_is_extended_positive_negative(positive)`: only numbers
/// (decided by evaluating them) get an answer.
pub(crate) fn expr_eval_is_extended_positive_negative(e: &Expr) -> Option<bool> {
    if !e.is_number() {
        return None;
    }
    // check to see that we can get a value
    let n2 = match crate::error::catch_kind(ErrorKind::ValueError, || eval_evalf_of(e, 2)) {
        // XXX: This shouldn't be caught here
        None => return None,
        Some(n2) => n2,
    };
    if n2.is_none() {
        return None;
    }
    not_ported(format!(
        "Expr._eval_is_extended_positive_negative for {} (needs evalf)",
        e.class_name()
    ))
}

/// `e._eval_evalf(prec)` for any expression: the head's for applied
/// functions and `Lambda`, `core.evalf`'s otherwise.
pub(crate) fn eval_evalf_of(e: &Expr, prec: i64) -> Option<Expr> {
    if e.is_a(Tag::Application) {
        return head_of(e).eval_evalf(e, prec);
    }
    match e.cls() {
        ClassId::Lambda | ClassId::IdentityFunction => lambda::eval_evalf(e, prec),
        _ => crate::evalf::eval_evalf(e, prec),
    }
}

/// `Function.fdiff(argindex)` (and the `ArgumentIndexError` every
/// override starts with).
pub fn default_fdiff<H: Head + ?Sized>(h: &H, e: &Expr, argindex: usize) -> Expr {
    match hook_owner(h, Hook::Fdiff) {
        "Function" => {
            if !(1 <= argindex && argindex <= e.args().len()) {
                raise(
                    ErrorKind::ValueError,
                    format!(
                        "Invalid operation with argument number {argindex} for Function {e}"
                    ),
                );
            }
            not_ported("Function.fdiff (needs Derivative and Subs)")
        }
        o => not_ported(format!("{o}.fdiff")),
    }
}

/// `Application._eval_subs(old, new)`, or `Basic._eval_subs` (`None`).
///
/// `Application._eval_subs` only fires when `old` is the function class
/// itself (`f(x).subs(f, g)`); an `Expr` never equals a class, so with
/// expressions for `old` and `new` it is always `None`. Substituting a
/// function class is [`application::subs_function_class`].
pub fn default_eval_subs<H: Head + ?Sized>(
    h: &H,
    e: &Expr,
    old: &Expr,
    new: &Expr,
) -> Option<Expr> {
    let _ = (e, old, new);
    match hook_owner(h, Hook::EvalSubs) {
        "Application" | "Basic" => None,
        o => not_ported(format!("{o}._eval_subs")),
    }
}

/// `Function._eval_evalf(prec)`.
pub fn default_eval_evalf<H: Head + ?Sized>(h: &H, e: &Expr, prec: i64) -> Option<Expr> {
    match hook_owner(h, Hook::EvalEvalf) {
        "Function" => application::function_eval_evalf(h, e, prec),
        o => not_ported(format!("{o}._eval_evalf")),
    }
}

/// `Expr._eval_conjugate()`.
pub fn expr_eval_conjugate(e: &Expr) -> Option<Expr> {
    if e.is_extended_real() == Some(true) {
        Some(e.clone())
    } else if e.is_imaginary() == Some(true) {
        Some(-e)
    } else {
        None
    }
}

/// `Expr._eval_transpose()`.
pub fn expr_eval_transpose(e: &Expr) -> Option<Expr> {
    if e.is_complex() == Some(true) || e.is_infinite() == Some(true) {
        Some(e.clone())
    } else if e.is_hermitian() == Some(true) {
        Some(call(ClassId::Conjugate, vec![e.clone()]))
    } else if e.is_antihermitian() == Some(true) {
        Some(-call(ClassId::Conjugate, vec![e.clone()]))
    } else {
        None
    }
}

/// `Expr._eval_adjoint()`.
pub fn expr_eval_adjoint<H: Head + ?Sized>(h: &H, e: &Expr) -> Option<Expr> {
    if e.is_hermitian() == Some(true) {
        return Some(e.clone());
    } else if e.is_antihermitian() == Some(true) {
        return Some(-e);
    }
    if h.eval_conjugate(e).is_some() {
        not_ported("Expr._eval_adjoint: transpose");
    }
    if let Some(obj) = h.eval_transpose(e) {
        return Some(call(ClassId::Conjugate, vec![obj]));
    }
    None
}

// ---------------------------------------------------------------------------
// rewrite and expand registries

/// Keyword arguments passed through `rewrite(**hints)` and
/// `expand(**hints)`, in the order given.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hints(pub Vec<(String, HintValue)>);

/// A hint's value.
#[derive(Clone, Debug, PartialEq)]
pub enum HintValue {
    Bool(bool),
    Int(Int),
    Expr(Expr),
    None,
}

impl Hints {
    /// `hints.get(key)`.
    pub fn get(&self, key: &str) -> Option<&HintValue> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// `hints[key] = value`, keeping an existing key's position.
    pub fn set(&mut self, key: &str, value: HintValue) {
        match self.0.iter_mut().find(|(k, _)| k == key) {
            Some((_, v)) => *v = value,
            None => self.0.push((key.to_string(), value)),
        }
    }

    /// `hints.pop(key, None)`.
    pub fn pop(&mut self, key: &str) -> Option<HintValue> {
        let i = self.0.iter().position(|(k, _)| k == key)?;
        Some(self.0.remove(i).1)
    }
}

/// `Class._eval_rewrite_as_<target>(self, *args, **hints)`.
pub type RewriteFn = fn(e: &Expr, args: &[Expr], hints: &Hints) -> Option<Expr>;

/// `Class._eval_expand_<hint>(self, **hints)`.
pub type ExpandFn = fn(e: &Expr, hints: &Hints) -> Expr;

/// The ported `_eval_rewrite_as_<target>` methods: (defining class,
/// target, function).
pub static BUILTIN_REWRITES: &[(&str, &str, RewriteFn)] =
    &[("Mod", "floor", core_mod::eval_rewrite_as_floor)];

/// The ported `_eval_expand_<hint>` methods: (defining class, hint,
/// function).
pub static BUILTIN_EXPANDS: &[(&str, &str, ExpandFn)] = &[];

type Registry<F> = RwLock<Vec<(&'static str, &'static str, F)>>;

static EXTRA_REWRITES: Registry<RewriteFn> = RwLock::new(Vec::new());
static EXTRA_EXPANDS: Registry<ExpandFn> = RwLock::new(Vec::new());

/// Registers `owner._eval_rewrite_as_<target>` (for every class whose MRO
/// finds the method in `owner`).
pub fn register_rewrite(owner: &'static str, target: &'static str, f: RewriteFn) {
    EXTRA_REWRITES
        .write()
        .expect("rewrite registry")
        .push((owner, target, f));
}

/// Registers `owner._eval_expand_<hint>`.
pub fn register_expand(owner: &'static str, hint: &'static str, f: ExpandFn) {
    EXTRA_EXPANDS
        .write()
        .expect("expand registry")
        .push((owner, hint, f));
}

fn lookup<F: Copy>(builtin: &[(&str, &str, F)], extra: &Registry<F>, owner: &str, name: &str) -> Option<F> {
    if let Some(&(_, _, f)) = builtin.iter().find(|(o, n, _)| *o == owner && *n == name) {
        return Some(f);
    }
    let extra = extra.read().expect("registry");
    extra
        .iter()
        .find(|(o, n, _)| *o == owner && *n == name)
        .map(|&(_, _, f)| f)
}

/// `getattr(e, "_eval_rewrite_as_" + target, None)(*args, **hints)`, for
/// any expression: `None` when its class has no such method.
pub fn default_eval_rewrite_as(
    e: &Expr,
    target: &str,
    args: &[Expr],
    hints: &Hints,
) -> Option<Option<Expr>> {
    let rewrites = FUNCTION_INFO[e.cls() as usize].rewrites;
    let &(_, owner) = rewrites.iter().find(|(t, _)| *t == target)?;
    match lookup(BUILTIN_REWRITES, &EXTRA_REWRITES, owner, target) {
        Some(f) => Some(f(e, args, hints)),
        None => not_ported(format!("{owner}._eval_rewrite_as_{target}")),
    }
}

/// `Basic._eval_rewrite(rule, args, **hints)` (`None`), for any
/// expression whose class does not override it.
pub fn default_eval_rewrite(e: &Expr, rule: &str, args: &[Expr], hints: &Hints) -> Option<Expr> {
    let _ = (rule, args, hints);
    match FUNCTION_INFO[e.cls() as usize].hooks[Hook::EvalRewrite as usize] {
        "Basic" => None,
        o => not_ported(format!("{o}._eval_rewrite")),
    }
}

/// `getattr(e, "_eval_expand_" + hint, None)(**hints)`, for any
/// expression: `None` when its class has no such method.
pub fn default_eval_expand(e: &Expr, hint: &str, hints: &Hints) -> Option<Expr> {
    let expands = FUNCTION_INFO[e.cls() as usize].expand_hints;
    let &(_, owner) = expands.iter().find(|(h, _)| *h == hint)?;
    match lookup(BUILTIN_EXPANDS, &EXTRA_EXPANDS, owner, hint) {
        Some(f) => Some(f(e, hints)),
        None => not_ported(format!("{owner}._eval_expand_{hint}")),
    }
}

/// `e._eval_rewrite_as_<target>(...)` through `e`'s head for applied
/// functions, else the registry.
pub(crate) fn eval_rewrite_as_of(
    e: &Expr,
    target: &str,
    args: &[Expr],
    hints: &Hints,
) -> Option<Option<Expr>> {
    if e.is_a(Tag::Application) {
        return head_of(e).eval_rewrite_as(e, target, args, hints);
    }
    default_eval_rewrite_as(e, target, args, hints)
}

/// `e._eval_rewrite(rule, args, **hints)` through `e`'s head for applied
/// functions.
pub(crate) fn eval_rewrite_of(e: &Expr, rule: &str, args: &[Expr], hints: &Hints) -> Option<Expr> {
    if e.is_a(Tag::Application) {
        return head_of(e).eval_rewrite(e, rule, args, hints);
    }
    default_eval_rewrite(e, rule, args, hints)
}
