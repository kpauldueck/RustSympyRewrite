//! Expression nodes.
//!
//! An [`Expr`] is a shared, immutable SymPy object: its class (a
//! [`ClassId`]), any atomic payload (an integer's value, a symbol's name)
//! and its `args`. Structural equality and hashing follow `Basic.__eq__`
//! and `_hashable_content`. Each node carries its own `_assumptions`
//! knowledge base, filled in as facts are asked, like SymPy's.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, LazyLock};

use rsympy_mpf::Mpf;
use rsympy_num::{Int, Rational};

use crate::facts::{Fact, FactCell, Kb};
use crate::generated::{CLASSES, ClassId, ClassInfo, Flag, Method, NUM_CLASSES, Tag};

/// A SymPy expression (any `Basic`).
#[derive(Clone)]
pub struct Expr(pub(crate) Arc<Node>);

pub(crate) struct Node {
    pub(crate) cls: ClassId,
    pub(crate) data: Data,
    pub(crate) args: Box<[Expr]>,
    hash: u64,
    pub(crate) facts: FactCell,
    /// The `is_commutative` slot attribute that `AssocOp._from_args` and
    /// `Pow.__new__` set (`None`: the class has no such slot). It answers
    /// `obj.is_commutative` directly but is not part of `_assumptions`, so
    /// `_ask` still runs the class's commutative handler when another fact
    /// needs it.
    pub(crate) commutative_slot: Option<Option<bool>>,
}

/// The atomic part of a node's `_hashable_content`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) enum Data {
    None,
    /// `Integer.p` (also for `Zero`, `One`, `NegativeOne`).
    Int(Int),
    /// `Rational.p/q`, with `q > 1` (also for `Half`).
    Rat(Rational),
    /// `Float._mpf_` and `Float._prec`.
    Float(Mpf, i64),
    Sym(Arc<SymbolData>),
    /// An undefined function's name, for `AppliedUndef` nodes.
    Undef(Arc<UndefData>),
    /// A `WildFunction`'s `name` and `nargs` attributes, which its
    /// `__eq__` and `__hash__` ignore (they compare `args` only).
    WildFunction(Arc<WildFunctionData>),
    /// `Str.name`.
    Str(Arc<str>),
}

/// A `Symbol`, `Dummy` or `Wild`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SymbolData {
    pub name: Arc<str>,
    /// `_assumptions0`: the deduced assumptions, as stored for hashing.
    pub assumptions0: Kb,
    /// `_assumptions_orig`: the assumptions as passed, for `srepr`.
    pub assumptions_orig: Vec<(Fact, bool)>,
    /// `Dummy.dummy_index`.
    pub dummy_index: Option<u64>,
}

/// An `UndefinedFunction` (the class of `f(x)` for `f = Function('f')`).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct UndefData {
    pub name: Arc<str>,
    /// `_kwargs`: the sanitized assumptions given to
    /// `Function('f', **assumptions)`, which `__eq__` and `__hash__` use.
    pub kwargs: Kb,
    /// The class's `default_assumptions`, which each `f(x)` starts from.
    pub assumptions: Kb,
    /// The `nargs` keyword as given to `Function(name, nargs=...)` (`None`
    /// when not given). It is part of `_kwargs`, so `__eq__` compares it as
    /// given: `nargs=1` and `nargs=(1,)` make different functions.
    pub nargs: Option<crate::functions::NargsArg>,
}

/// The attributes `WildFunction.__init__` sets on an instance: `name` (the
/// string given) and `nargs`. Neither is part of `_hashable_content`, so
/// they never make two `WildFunction`s unequal.
#[derive(Clone, Debug)]
pub struct WildFunctionData {
    pub name: Arc<str>,
    pub nargs: crate::functions::Nargs,
}

impl PartialEq for WildFunctionData {
    fn eq(&self, _: &WildFunctionData) -> bool {
        true
    }
}

impl Eq for WildFunctionData {}

impl Hash for WildFunctionData {
    fn hash<H: Hasher>(&self, _: &mut H) {}
}

impl Node {
    fn compute_hash(cls: ClassId, data: &Data, args: &[Expr]) -> u64 {
        let mut h = FxHasher::default();
        (cls as u16).hash(&mut h);
        data.hash(&mut h);
        for a in args {
            h.write_u64(a.0.hash);
        }
        h.finish()
    }
}

/// The class-level `default_assumptions` of each class, as a KB.
static CLASS_DEFAULTS: LazyLock<[Kb; NUM_CLASSES]> = LazyLock::new(|| {
    std::array::from_fn(|i| {
        let c = &CLASSES[i];
        Kb::from_masks(c.default_known, c.default_true, c.default_none)
    })
});

impl Expr {
    /// Builds a node with its class's default assumptions.
    pub(crate) fn make(cls: ClassId, data: Data, args: Vec<Expr>) -> Expr {
        let kb = CLASS_DEFAULTS[cls as usize];
        Expr::make_with_kb(cls, data, args, kb)
    }

    pub(crate) fn make_with_kb(cls: ClassId, data: Data, args: Vec<Expr>, kb: Kb) -> Expr {
        Expr::make_full(cls, data, args, kb, None)
    }

    /// A node with its class's default assumptions and `is_commutative`
    /// set as a slot attribute (Add, Mul and Pow).
    pub(crate) fn make_with_commutative_slot(
        cls: ClassId,
        args: Vec<Expr>,
        commutative: Option<bool>,
    ) -> Expr {
        let kb = CLASS_DEFAULTS[cls as usize];
        Expr::make_full(cls, Data::None, args, kb, Some(commutative))
    }

    fn make_full(
        cls: ClassId,
        data: Data,
        args: Vec<Expr>,
        kb: Kb,
        commutative_slot: Option<Option<bool>>,
    ) -> Expr {
        let hash = Node::compute_hash(cls, &data, &args);
        Expr(Arc::new(Node {
            cls,
            data,
            args: args.into_boxed_slice(),
            hash,
            facts: FactCell::new(kb),
            commutative_slot,
        }))
    }

    pub fn cls(&self) -> ClassId {
        self.0.cls
    }

    pub(crate) fn info(&self) -> &'static ClassInfo {
        &CLASSES[self.0.cls as usize]
    }

    pub(crate) fn data(&self) -> &Data {
        &self.0.data
    }

    /// `self.args`.
    pub fn args(&self) -> &[Expr] {
        &self.0.args
    }

    pub fn arg(&self, i: usize) -> &Expr {
        &self.0.args[i]
    }

    /// `self.func(*args)`: rebuilds a node of the same class (and payload)
    /// with new args, evaluating as the class constructor does.
    pub fn func(&self, args: Vec<Expr>) -> Expr {
        crate::construct::rebuild(self, args)
    }

    /// The `Class._eval_is_<fact>` that `type(self)._prop_handler[fact]`
    /// calls, if the class has one.
    pub fn handler_owner(&self, fact: Fact) -> Option<&'static str> {
        self.info()
            .handler_owners
            .iter()
            .find(|(f, _)| *f == fact)
            .map(|(_, owner)| *owner)
    }

    /// `type(self).__name__`.
    pub fn class_name(&self) -> &str {
        match &self.0.data {
            Data::Undef(u) if self.0.cls == ClassId::AppliedUndef => &u.name,
            _ => self.info().name,
        }
    }

    /// The class that defines `method` for this node's class (`""` when
    /// no class in the MRO has it, so calling it raises `AttributeError`).
    pub fn method_owner(&self, method: Method) -> &'static str {
        self.info().method_owners[method as usize]
    }

    /// `isinstance(self, tag)`.
    pub fn is_a(&self, tag: Tag) -> bool {
        self.info().tags >> tag as u8 & 1 == 1
    }

    /// The class-level `is_<Flag>` attribute.
    pub fn flag(&self, flag: Flag) -> bool {
        self.info().flags >> flag as u8 & 1 == 1
    }

    /// `a is b`: the same node.
    pub fn ptr_eq(&self, other: &Expr) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub(crate) fn hash_value(&self) -> u64 {
        self.0.hash
    }

    /// The node's `_assumptions` as known so far.
    pub fn assumptions_kb(&self) -> Kb {
        self.0.facts.load()
    }

    pub(crate) fn merge_kb(&self, kb: Kb) {
        self.0.facts.merge(kb);
    }

    // ----- payload accessors -----

    /// `Integer.p` for integers.
    pub fn as_int(&self) -> Option<&Int> {
        match &self.0.data {
            Data::Int(i) => Some(i),
            _ => None,
        }
    }

    /// `(p, q)` for any `Rational` (including integers).
    pub fn as_rational(&self) -> Option<Rational> {
        match &self.0.data {
            Data::Int(i) => Some(Rational::from_int(i.clone())),
            Data::Rat(r) => Some(r.clone()),
            _ => None,
        }
    }

    /// `Rational.p`.
    pub fn p(&self) -> Int {
        match &self.0.data {
            Data::Int(i) => i.clone(),
            Data::Rat(r) => r.numer().clone(),
            _ => panic!("{} has no .p", self.class_name()),
        }
    }

    /// `Rational.q`.
    pub fn q(&self) -> Int {
        match &self.0.data {
            Data::Int(_) => Int::ONE,
            Data::Rat(r) => r.denom().clone(),
            _ => panic!("{} has no .q", self.class_name()),
        }
    }

    /// `Float._mpf_` and `Float._prec`.
    pub fn as_float(&self) -> Option<(&Mpf, i64)> {
        match &self.0.data {
            Data::Float(m, p) => Some((m, *p)),
            _ => None,
        }
    }

    pub fn symbol_data(&self) -> Option<&SymbolData> {
        match &self.0.data {
            Data::Sym(s) => Some(s),
            _ => None,
        }
    }

    /// `Symbol.name`.
    pub fn name(&self) -> Option<&str> {
        match &self.0.data {
            Data::Sym(s) => Some(&s.name),
            Data::Undef(u) => Some(&u.name),
            Data::Str(s) => Some(s),
            Data::WildFunction(w) => Some(&w.name),
            _ => None,
        }
    }

    pub(crate) fn undef_data(&self) -> Option<&Arc<UndefData>> {
        match &self.0.data {
            Data::Undef(u) => Some(u),
            _ => None,
        }
    }

    pub(crate) fn wild_function_data(&self) -> Option<&Arc<WildFunctionData>> {
        match &self.0.data {
            Data::WildFunction(w) => Some(w),
            _ => None,
        }
    }
}

// ----- SymPy's class-level `is_Foo` flags -----

#[allow(non_snake_case)]
impl Expr {
    pub fn is_Atom(&self) -> bool {
        self.flag(Flag::Atom)
    }
    pub fn is_Number(&self) -> bool {
        self.flag(Flag::Number)
    }
    pub fn is_Integer(&self) -> bool {
        self.flag(Flag::Integer)
    }
    pub fn is_Rational(&self) -> bool {
        self.flag(Flag::Rational)
    }
    pub fn is_Float(&self) -> bool {
        self.flag(Flag::Float)
    }
    pub fn is_NumberSymbol(&self) -> bool {
        self.flag(Flag::NumberSymbol)
    }
    pub fn is_Symbol(&self) -> bool {
        self.flag(Flag::Symbol)
    }
    pub fn is_Dummy(&self) -> bool {
        self.flag(Flag::Dummy)
    }
    pub fn is_Wild(&self) -> bool {
        self.flag(Flag::Wild)
    }
    pub fn is_Add(&self) -> bool {
        self.flag(Flag::Add)
    }
    pub fn is_Mul(&self) -> bool {
        self.flag(Flag::Mul)
    }
    pub fn is_Pow(&self) -> bool {
        self.flag(Flag::Pow)
    }
    pub fn is_Function(&self) -> bool {
        self.flag(Flag::Function)
    }
    pub fn is_Order(&self) -> bool {
        self.flag(Flag::Order)
    }
    pub fn is_Relational(&self) -> bool {
        self.flag(Flag::Relational)
    }
    pub fn is_Boolean(&self) -> bool {
        self.flag(Flag::Boolean)
    }
    pub fn is_Piecewise(&self) -> bool {
        self.flag(Flag::Piecewise)
    }
    pub fn is_Derivative(&self) -> bool {
        self.flag(Flag::Derivative)
    }
    pub fn is_Matrix(&self) -> bool {
        self.flag(Flag::Matrix)
    }
    /// `is_symbol` (lower case): true for symbols and other "symbol-like"
    /// atoms.
    pub fn is_symbol(&self) -> bool {
        self.info().is_symbol
    }
}

impl Expr {
    /// `self.is_number`.
    pub fn is_number(&self) -> bool {
        match self.info().is_number {
            Some(v) => v,
            None => self.args().iter().all(Expr::is_number),
        }
    }

    /// `self is S.<cls>` for a singleton class.
    pub fn is_cls(&self, cls: ClassId) -> bool {
        self.0.cls == cls
    }
}

impl PartialEq for Expr {
    /// `Basic.__eq__`: structural equality.
    fn eq(&self, other: &Expr) -> bool {
        if Arc::ptr_eq(&self.0, &other.0) {
            return true;
        }
        let (a, b) = (&*self.0, &*other.0);
        a.hash == b.hash && a.cls == b.cls && a.data == b.data && a.args == b.args
    }
}

impl Eq for Expr {}

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0.hash);
    }
}

impl fmt::Display for Expr {
    /// `str(self)`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::printing::str::sstr(self))
    }
}

impl fmt::Debug for Expr {
    /// The oracle's lossless encoding ([`crate::testing::encode`]): `srepr`
    /// with args in storage order.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::testing::encode(self))
    }
}

/// FxHash: fast and good enough for structural hashes.
#[derive(Default)]
pub(crate) struct FxHasher(u64);

impl Hasher for FxHasher {
    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut buf = [0u8; 8];
            buf[..chunk.len()].copy_from_slice(chunk);
            self.write_u64(u64::from_le_bytes(buf));
        }
    }

    fn write_u64(&mut self, n: u64) {
        self.0 = (self.0.rotate_left(5) ^ n).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }

    fn write_u8(&mut self, n: u8) {
        self.write_u64(n as u64);
    }

    fn write_u16(&mut self, n: u16) {
        self.write_u64(n as u64);
    }

    fn write_u32(&mut self, n: u32) {
        self.write_u64(n as u64);
    }

    fn write_i64(&mut self, n: i64) {
        self.write_u64(n as u64);
    }

    fn write_usize(&mut self, n: usize) {
        self.write_u64(n as u64);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}
