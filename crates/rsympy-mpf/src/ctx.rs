//! Port of the parts of mpmath's number context that SymPy's `evalf`
//! calls through: the `mpf`/`mpc` value types and their operators
//! (`ctx_mp_python.py`), the working precision and its managers, the
//! libmp functions wrapped as context methods (`_wrap_libmp_function`),
//! the elementary functions of `functions/functions.py`, and `hypsum`
//! (`ctx_mp.py`).
//!
//! mpmath keeps one global context, `mp`; here the context is one per
//! thread. Every function reads the working precision and rounding mode
//! from it, as the Python methods read `ctx._prec_rounding`.
//!
//! A Python `mpf` or `mpc` is a [`Value`]. Python ints, floats and
//! complexes that the Python code would convert with `ctx.convert` are
//! converted first ([`convert`]); the operators whose int forms take a
//! different path in mpmath (`mpf * int`, `int / mpf`, `mpf ** int`) have
//! their own functions.

use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;

use rsympy_num::Int;

use crate::constants::*;
use crate::elefun::*;
use crate::hyper::{
    HypArg, HypParam, HypValue, hyp_summator, mpc_agm, mpc_agm1, mpf_agm, mpf_agm1,
};
use crate::intmath::bitcount;
use crate::mpc::*;
use crate::mpf::*;

/// What the context functions raise.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CtxError {
    /// An error raised by a libmp function.
    Mpf(MpfError),
    /// Python `TypeError`.
    Type(String),
    /// Python `AttributeError` (a real-only function given an mpc reads
    /// `x._mpf_`).
    Attribute(String),
    /// Python `NotImplementedError` raised by the context code.
    NotImplemented(String),
    /// mpmath's `NoConvergence`.
    NoConvergence(String),
    /// Python `RecursionError` (`mpc % mpf` recurses without end).
    Recursion,
    /// A path of mpmath this port does not cover yet.
    NotPorted(&'static str),
}

impl CtxError {
    /// The Python exception class, or `NotPortedYet`.
    pub fn class(&self) -> &'static str {
        match self {
            CtxError::Mpf(e) => e.class(),
            CtxError::Type(_) => "TypeError",
            CtxError::Attribute(_) => "AttributeError",
            CtxError::NotImplemented(_) => "NotImplementedError",
            CtxError::NoConvergence(_) => "NoConvergence",
            CtxError::Recursion => "RecursionError",
            CtxError::NotPorted(_) => "NotPortedYet",
        }
    }
}

impl fmt::Display for CtxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CtxError::Mpf(e) => e.fmt(f),
            CtxError::Type(m) => write!(f, "TypeError: {m}"),
            CtxError::Attribute(m) => write!(f, "AttributeError: {m}"),
            CtxError::NotImplemented(m) => write!(f, "NotImplementedError: {m}"),
            CtxError::NoConvergence(m) => write!(f, "NoConvergence: {m}"),
            CtxError::Recursion => f.write_str("RecursionError"),
            CtxError::NotPorted(m) => write!(f, "NotPortedYet: {m}"),
        }
    }
}

impl std::error::Error for CtxError {}

impl From<MpfError> for CtxError {
    fn from(e: MpfError) -> Self {
        CtxError::Mpf(e)
    }
}

pub type CtxResult<T> = Result<T, CtxError>;

// ---------------------------------------------------------------------
// The context state
// ---------------------------------------------------------------------

#[derive(Clone, Copy)]
struct State {
    prec: i64,
    dps: i64,
    rounding: Rnd,
    trap_complex: bool,
}

thread_local! {
    static STATE: Cell<State> = const {
        Cell::new(State {
            prec: 53,
            dps: 15,
            rounding: Rnd::Nearest,
            trap_complex: false,
        })
    };
}

fn state() -> State {
    STATE.with(|s| s.get())
}

fn update(f: impl FnOnce(&mut State)) {
    STATE.with(|s| {
        let mut st = s.get();
        f(&mut st);
        s.set(st);
    });
}

/// `ctx._prec_rounding`.
fn prec_rounding() -> (i64, Rnd) {
    let s = state();
    (s.prec, s.rounding)
}

/// `ctx.prec`.
pub fn prec() -> i64 {
    state().prec
}

/// `ctx.dps`.
pub fn dps() -> i64 {
    state().dps
}

/// `ctx._prec_rounding[1]`.
pub fn rounding() -> Rnd {
    state().rounding
}

/// `ctx.trap_complex`.
pub fn trap_complex() -> bool {
    state().trap_complex
}

/// `ctx.prec = n` (`_set_prec`).
pub fn set_prec(n: i64) {
    update(|s| {
        s.prec = n.max(1);
        s.dps = prec_to_dps(n);
    });
}

/// `ctx.dps = n` (`_set_dps`).
pub fn set_dps(n: i64) {
    update(|s| {
        s.prec = dps_to_prec(n);
        s.dps = n.max(1);
    });
}

/// `ctx._prec_rounding[1] = rnd`.
pub fn set_rounding(rnd: Rnd) {
    update(|s| s.rounding = rnd);
}

/// `ctx.trap_complex = b`.
pub fn set_trap_complex(b: bool) {
    update(|s| s.trap_complex = b);
}

/// `ctx.default()`: 53 bits, 15 digits, `trap_complex` off. Like mpmath's
/// it leaves the rounding mode alone.
pub fn default() {
    update(|s| {
        s.prec = 53;
        s.dps = 15;
        s.trap_complex = false;
    });
}

/// The state of a fresh `mp`: [`default`] and round-to-nearest.
pub fn reset_context() {
    default();
    set_rounding(Rnd::Nearest);
}

/// `with ctx.workprec(n): f()`. The precision is restored however `f`
/// returns.
pub fn workprec<T>(n: i64, f: impl FnOnce() -> T) -> T {
    let orig = prec();
    set_prec(n);
    let r = f();
    set_prec(orig);
    r
}

/// `with ctx.extraprec(n): f()`.
pub fn extraprec<T>(n: i64, f: impl FnOnce() -> T) -> T {
    let orig = prec();
    set_prec(orig + n);
    let r = f();
    set_prec(orig);
    r
}

/// `with ctx.workdps(n): f()`.
pub fn workdps<T>(n: i64, f: impl FnOnce() -> T) -> T {
    let orig = prec();
    set_dps(n);
    let r = f();
    set_prec(orig);
    r
}

/// `with ctx.extradps(n): f()`.
pub fn extradps<T>(n: i64, f: impl FnOnce() -> T) -> T {
    let orig = prec();
    set_dps(dps() + n);
    let r = f();
    set_prec(orig);
    r
}

// ---------------------------------------------------------------------
// Values and conversion
// ---------------------------------------------------------------------

/// An mpmath number: `mpf` (`_mpf_`) or `mpc` (`_mpc_`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Mpf(Mpf),
    Mpc(Mpc),
}

impl Value {
    /// `x.real`: an mpf is its own real part (not rounded).
    pub fn real(&self) -> Mpf {
        match self {
            Value::Mpf(x) => x.clone(),
            Value::Mpc((re, _)) => re.clone(),
        }
    }

    /// `x.imag`: `ctx.zero` for an mpf.
    pub fn imag(&self) -> Mpf {
        match self {
            Value::Mpf(_) => FZERO,
            Value::Mpc((_, im)) => im.clone(),
        }
    }

    fn mpf_or_attr(&self) -> CtxResult<&Mpf> {
        match self {
            Value::Mpf(x) => Ok(x),
            Value::Mpc(_) => Err(CtxError::Attribute(
                "'mpc' object has no attribute '_mpf_'".into(),
            )),
        }
    }
}

/// A Python object that `ctx.convert` and the constructors accept.
#[derive(Clone, Debug, PartialEq)]
pub enum Num {
    Int(Int),
    Float(f64),
    Complex(f64, f64),
    /// An `mpq` (`p/q`, reduced as `mpq(p, q)` reduces it).
    Mpq(Int, Int),
    Str(String),
    Val(Value),
}

/// `mpq(p, q)`: `create_reduced`, which divides by the gcd that
/// Euclid's algorithm with Python's `%` leaves (so it may flip signs).
pub fn mpq_reduce(p: &Int, q: &Int) -> CtxResult<(Int, Int)> {
    let (mut x, mut y) = (p.clone(), q.clone());
    while !y.is_zero() {
        let r = x.py_mod(&y).expect("y != 0");
        x = y;
        y = r;
    }
    if x != Int::ONE {
        let zd = || CtxError::Mpf(MpfError::ZeroDivision);
        let p2 = p.floor_div(&x).map_err(|_| zd())?;
        let q2 = q.floor_div(&x).map_err(|_| zd())?;
        return Ok((p2, q2));
    }
    Ok((p.clone(), q.clone()))
}

fn from_f64(x: f64) -> Mpf {
    from_float(x, 53, ROUND_FAST)
}

/// `ctx.make_mpf(from_int(n))`, what an int becomes under `ctx.convert`.
pub fn convert_int(n: &Int) -> Value {
    Value::Mpf(from_int(n, 0, ROUND_FAST))
}

/// `ctx.convert(x)`.
pub fn convert(x: &Num) -> CtxResult<Value> {
    let (prec, rounding) = prec_rounding();
    match x {
        Num::Val(v) => Ok(v.clone()),
        Num::Int(n) => Ok(convert_int(n)),
        Num::Float(f) => Ok(Value::Mpf(from_f64(*f))),
        Num::Complex(re, im) => Ok(Value::Mpc((from_f64(*re), from_f64(*im)))),
        Num::Mpq(p, q) => Ok(Value::Mpf(from_rational(p, q, prec, ROUND_FAST)?)),
        Num::Str(s) => match from_str(s, prec, rounding) {
            Ok(v) => Ok(Value::Mpf(v)),
            Err(MpfError::Value(_)) => {
                if s.to_lowercase().contains('j') {
                    return Err(CtxError::NotPorted("ctx.convert of a complex string"));
                }
                Err(CtxError::Type(format!("cannot create mpf from {s:?}")))
            }
            Err(e) => Err(e.into()),
        },
    }
}

/// `ctx.mpf(x, prec=prec, rounding=rnd)`.
pub fn mpf_new(x: &Num, prec: i64, rnd: Rnd) -> CtxResult<Mpf> {
    let v = match x {
        Num::Val(Value::Mpf(v)) => {
            if v.is_special() {
                return Ok(v.clone());
            }
            return Ok(normalize(v.sign, v.man.clone(), v.exp, v.bc, prec, rnd));
        }
        Num::Int(n) => from_int(n, 0, ROUND_FAST),
        Num::Float(f) => from_f64(*f),
        Num::Str(s) => from_str(s, prec, rnd)?,
        Num::Val(Value::Mpc(_)) | Num::Complex(..) | Num::Mpq(..) => {
            return Err(CtxError::Type("cannot create mpf".into()));
        }
    };
    Ok(mpf_pos(&v, prec, rnd))
}

/// `ctx.mpf(x)` at the working precision.
pub fn mpf(x: &Num) -> CtxResult<Mpf> {
    let (prec, rounding) = prec_rounding();
    mpf_new(x, prec, rounding)
}

/// `ctx.mpc(real, imag)`. An mpc or complex `real` supplies both parts
/// (and `imag` is ignored); each part is rounded by `ctx.mpf`.
pub fn mpc_new(real: &Num, imag: &Num) -> CtxResult<Mpc> {
    let (re, im) = match real {
        Num::Val(Value::Mpc((re, im))) => (
            Num::Val(Value::Mpf(re.clone())),
            Num::Val(Value::Mpf(im.clone())),
        ),
        Num::Complex(re, im) => (Num::Float(*re), Num::Float(*im)),
        _ => (real.clone(), imag.clone()),
    };
    Ok((mpf(&re)?, mpf(&im)?))
}

/// The constants of `init_builtins` that M2 covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CtxConstant {
    Pi,
    E,
    Ln2,
    Ln10,
    Phi,
    Degree,
    Eps,
}

impl CtxConstant {
    /// `constant.func(prec, rnd)`.
    pub fn eval(self, prec: i64, rnd: Rnd) -> Mpf {
        match self {
            CtxConstant::Pi => mpf_pi(prec, rnd),
            CtxConstant::E => mpf_e(prec, rnd),
            CtxConstant::Ln2 => mpf_ln2(prec, rnd),
            CtxConstant::Ln10 => mpf_ln10(prec, rnd),
            CtxConstant::Phi => mpf_phi(prec, rnd),
            CtxConstant::Degree => mpf_degree(prec, rnd),
            CtxConstant::Eps => Mpf {
                sign: 0,
                man: Int::ONE,
                exp: 1 - prec,
                bc: 1,
            },
        }
    }

    /// `constant._mpf_`: the value at the working precision, as an
    /// operand or under `+`.
    pub fn value(self) -> Value {
        let (prec, rounding) = prec_rounding();
        Value::Mpf(self.eval(prec, rounding))
    }
}

/// `ctx.one`.
pub fn one() -> Value {
    Value::Mpf(FONE)
}

/// `ctx.j`.
pub fn j() -> Value {
    Value::Mpc((FZERO, FONE))
}

// ---------------------------------------------------------------------
// Operators
// ---------------------------------------------------------------------

/// `bool(x)`.
pub fn truthy(x: &Value) -> bool {
    match x {
        Value::Mpf(v) => *v != FZERO,
        Value::Mpc(z) => mpc_is_nonzero(z),
    }
}

/// `+x`.
pub fn pos(x: &Value) -> Value {
    let (prec, rounding) = prec_rounding();
    match x {
        Value::Mpf(v) => Value::Mpf(mpf_pos(v, prec, rounding)),
        Value::Mpc(z) => Value::Mpc(mpc_pos(z, prec, rounding)),
    }
}

/// `-x`.
pub fn neg(x: &Value) -> Value {
    let (prec, rounding) = prec_rounding();
    match x {
        Value::Mpf(v) => Value::Mpf(mpf_neg(v, prec, rounding)),
        Value::Mpc(z) => Value::Mpc(mpc_neg(z, prec, rounding)),
    }
}

/// `abs(x)`.
pub fn abs(x: &Value) -> Value {
    let (prec, rounding) = prec_rounding();
    match x {
        Value::Mpf(v) => Value::Mpf(mpf_abs(v, prec, rounding)),
        Value::Mpc(z) => Value::Mpf(mpc_abs(z, prec, rounding)),
    }
}

/// `x + y`. For `int + x` mpmath calls `x.__radd__`, which is `x + int`.
pub fn add(x: &Value, y: &Value) -> Value {
    let (prec, rounding) = prec_rounding();
    match (x, y) {
        (Value::Mpf(s), Value::Mpf(t)) => Value::Mpf(mpf_add(s, t, prec, rounding)),
        (Value::Mpf(s), Value::Mpc(t)) => Value::Mpc(mpc_add_mpf(t, s, prec, rounding)),
        (Value::Mpc(s), Value::Mpf(t)) => Value::Mpc(mpc_add_mpf(s, t, prec, rounding)),
        (Value::Mpc(s), Value::Mpc(t)) => Value::Mpc(mpc_add(s, t, prec, rounding)),
    }
}

/// `x - y`.
pub fn sub(x: &Value, y: &Value) -> Value {
    let (prec, rounding) = prec_rounding();
    match (x, y) {
        (Value::Mpf(s), Value::Mpf(t)) => Value::Mpf(mpf_sub(s, t, prec, rounding)),
        (Value::Mpf(s), Value::Mpc(t)) => {
            Value::Mpc(mpc_sub(&(s.clone(), FZERO), t, prec, rounding))
        }
        (Value::Mpc(s), Value::Mpf(t)) => Value::Mpc(mpc_sub_mpf(s, t, prec, rounding)),
        (Value::Mpc(s), Value::Mpc(t)) => Value::Mpc(mpc_sub(s, t, prec, rounding)),
    }
}

/// `x * y`.
pub fn mul(x: &Value, y: &Value) -> Value {
    let (prec, rounding) = prec_rounding();
    match (x, y) {
        (Value::Mpf(s), Value::Mpf(t)) => Value::Mpf(mpf_mul(s, t, prec, rounding)),
        (Value::Mpf(s), Value::Mpc(t)) => Value::Mpc(mpc_mul_mpf(t, s, prec, rounding)),
        (Value::Mpc(s), Value::Mpf(t)) => Value::Mpc(mpc_mul_mpf(s, t, prec, rounding)),
        (Value::Mpc(s), Value::Mpc(t)) => Value::Mpc(mpc_mul(s, t, prec, rounding)),
    }
}

/// `x * n` and `n * x` for a Python int `n`.
pub fn mul_int(x: &Value, n: &Int) -> Value {
    let (prec, rounding) = prec_rounding();
    match x {
        Value::Mpf(s) => Value::Mpf(mpf_mul_int(s, n, prec, rounding)),
        Value::Mpc(s) => Value::Mpc(mpc_mul_int(s, n, prec, rounding)),
    }
}

/// `x / y`.
pub fn div(x: &Value, y: &Value) -> CtxResult<Value> {
    let (prec, rounding) = prec_rounding();
    Ok(match (x, y) {
        (Value::Mpf(s), Value::Mpf(t)) => Value::Mpf(mpf_div(s, t, prec, rounding)?),
        (Value::Mpf(s), Value::Mpc(t)) => Value::Mpc(mpc_mpf_div(s, t, prec, rounding)?),
        (Value::Mpc(s), Value::Mpf(t)) => Value::Mpc(mpc_div_mpf(s, t, prec, rounding)?),
        (Value::Mpc(s), Value::Mpc(t)) => Value::Mpc(mpc_div(s, t, prec, rounding)?),
    })
}

/// `n / x` for a Python int `n`.
pub fn rdiv_int(n: &Int, x: &Value) -> CtxResult<Value> {
    let (prec, rounding) = prec_rounding();
    match x {
        Value::Mpf(s) => Ok(Value::Mpf(mpf_rdiv_int(n, s, prec, rounding)?)),
        Value::Mpc(_) => div(&convert_int(n), x),
    }
}

/// `x ** y`. A real power that is complex goes to `mpc_pow` unless
/// `trap_complex` is set.
pub fn pow(x: &Value, y: &Value) -> CtxResult<Value> {
    let (prec, rounding) = prec_rounding();
    Ok(match (x, y) {
        (Value::Mpf(s), Value::Mpf(t)) => match mpf_pow(s, t, prec, rounding) {
            Ok(v) => Value::Mpf(v),
            Err(MpfError::ComplexResult(m)) => {
                if trap_complex() {
                    return Err(MpfError::ComplexResult(m).into());
                }
                Value::Mpc(mpc_pow(
                    &(s.clone(), FZERO),
                    &(t.clone(), FZERO),
                    prec,
                    rounding,
                )?)
            }
            Err(e) => return Err(e.into()),
        },
        (Value::Mpf(s), Value::Mpc(t)) => {
            Value::Mpc(mpc_pow(&(s.clone(), FZERO), t, prec, rounding)?)
        }
        (Value::Mpc(s), Value::Mpf(t)) => Value::Mpc(mpc_pow_mpf(s, t, prec, rounding)?),
        (Value::Mpc(s), Value::Mpc(t)) => Value::Mpc(mpc_pow(s, t, prec, rounding)?),
    })
}

/// `x ** n` for a Python int `n`. (`n ** x` converts `n`: use [`pow`].)
pub fn pow_int(x: &Value, n: &Int) -> CtxResult<Value> {
    let (prec, rounding) = prec_rounding();
    Ok(match x {
        Value::Mpf(s) => Value::Mpf(mpf_pow_int(s, n, prec, rounding)?),
        Value::Mpc(s) => Value::Mpc(mpc_pow_int(s, n, prec, rounding)?),
    })
}

/// `x % y`.
pub fn fmod(x: &Value, y: &Value) -> CtxResult<Value> {
    let (prec, rounding) = prec_rounding();
    match (x, y) {
        (Value::Mpf(s), Value::Mpf(t)) => Ok(Value::Mpf(mpf_mod(s, t, prec, rounding)?)),
        (Value::Mpf(_), Value::Mpc(_)) => Err(CtxError::NotImplemented("complex modulo".into())),
        // mpc has no __mod__; mpf.__rmod__ converts the mpc to an mpc and
        // tries `mpc % mpf` again.
        (Value::Mpc(_), Value::Mpf(_)) => Err(CtxError::Recursion),
        (Value::Mpc(_), Value::Mpc(_)) => Err(CtxError::Type(
            "unsupported operand type(s) for %: 'mpc' and 'mpc'".into(),
        )),
    }
}

/// `x == y`.
pub fn eq(x: &Value, y: &Value) -> bool {
    match (x, y) {
        (Value::Mpf(s), Value::Mpf(t)) => mpf_eq(s, t),
        (Value::Mpf(s), Value::Mpc(t)) => t.1 == FZERO && mpf_eq(&t.0, s),
        (Value::Mpc(s), Value::Mpf(t)) => mpf_eq(&s.0, t) && mpf_eq(&s.1, &FZERO),
        (Value::Mpc(s), Value::Mpc(t)) => mpf_eq(&s.0, &t.0) && mpf_eq(&s.1, &t.1),
    }
}

fn order(x: &Value, y: &Value, f: fn(&Mpf, &Mpf) -> bool) -> CtxResult<bool> {
    match (x, y) {
        (Value::Mpf(s), Value::Mpf(t)) => Ok(f(s, t)),
        _ => Err(CtxError::Type(
            "no ordering relation is defined for complex numbers".into(),
        )),
    }
}

/// `x < y`.
pub fn lt(x: &Value, y: &Value) -> CtxResult<bool> {
    order(x, y, mpf_lt)
}

/// `x <= y`.
pub fn le(x: &Value, y: &Value) -> CtxResult<bool> {
    order(x, y, mpf_le)
}

/// `x > y`.
pub fn gt(x: &Value, y: &Value) -> CtxResult<bool> {
    order(x, y, mpf_gt)
}

/// `x >= y`.
pub fn ge(x: &Value, y: &Value) -> CtxResult<bool> {
    order(x, y, mpf_ge)
}

// ---------------------------------------------------------------------
// Wrapped libmp functions
// ---------------------------------------------------------------------

/// The functions `init_builtins` makes with `_wrap_libmp_function`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Builtin {
    Sqrt,
    Cbrt,
    Ln,
    Atan,
    Exp,
    Expj,
    Expjpi,
    Sin,
    Cos,
    Tan,
    Sinh,
    Cosh,
    Tanh,
    Asin,
    Acos,
    Asinh,
    Acosh,
    Atanh,
    Sinpi,
    Cospi,
    Floor,
    Ceil,
    Nint,
    Frac,
    Fib,
    Agm1,
}

impl Builtin {
    /// The context method's name.
    pub fn from_name(name: &str) -> Option<Builtin> {
        use Builtin::*;
        Some(match name {
            "sqrt" => Sqrt,
            "cbrt" => Cbrt,
            "ln" => Ln,
            "atan" => Atan,
            "exp" => Exp,
            "expj" => Expj,
            "expjpi" => Expjpi,
            "sin" => Sin,
            "cos" => Cos,
            "tan" => Tan,
            "sinh" => Sinh,
            "cosh" => Cosh,
            "tanh" => Tanh,
            "asin" => Asin,
            "acos" => Acos,
            "asinh" => Asinh,
            "acosh" => Acosh,
            "atanh" => Atanh,
            "sinpi" => Sinpi,
            "cospi" => Cospi,
            "floor" => Floor,
            "ceil" => Ceil,
            "nint" => Nint,
            "frac" => Frac,
            "fib" | "fibonacci" => Fib,
            "agm1" => Agm1,
            _ => return None,
        })
    }

    fn mpf_f(self, x: &Mpf, prec: i64, rnd: Rnd) -> MpfResult<Mpf> {
        use Builtin::*;
        match self {
            Sqrt => mpf_sqrt(x, prec, rnd),
            Cbrt => mpf_cbrt(x, prec, rnd),
            Ln => mpf_log(x, prec, rnd),
            Atan => Ok(mpf_atan(x, prec, rnd)),
            Exp => Ok(mpf_exp(x, prec, rnd)),
            Expj => mpf_expj(x, prec, rnd),
            Expjpi => mpf_expjpi(x, prec, rnd),
            Sin => Ok(mpf_sin(x, prec, rnd)),
            Cos => Ok(mpf_cos(x, prec, rnd)),
            Tan => mpf_tan(x, prec, rnd),
            Sinh => Ok(mpf_sinh(x, prec, rnd)),
            Cosh => Ok(mpf_cosh(x, prec, rnd)),
            Tanh => Ok(mpf_tanh(x, prec, rnd)),
            Asin => mpf_asin(x, prec, rnd),
            Acos => mpf_acos(x, prec, rnd),
            Asinh => Ok(mpf_asinh(x, prec, rnd)),
            Acosh => mpf_acosh(x, prec, rnd),
            Atanh => mpf_atanh(x, prec, rnd),
            Sinpi => Ok(mpf_sin_pi(x, prec, rnd)),
            Cospi => Ok(mpf_cos_pi(x, prec, rnd)),
            Floor => Ok(mpf_floor(x, prec, rnd)),
            Ceil => Ok(mpf_ceil(x, prec, rnd)),
            Nint => Ok(mpf_nint(x, prec, rnd)),
            Frac => Ok(mpf_frac(x, prec, rnd)),
            Fib => Ok(mpf_fibonacci(x, prec, rnd)),
            Agm1 => mpf_agm1(x, prec, rnd),
        }
    }

    fn mpc_f(self, z: &Mpc, prec: i64, rnd: Rnd) -> MpfResult<Mpc> {
        use Builtin::*;
        match self {
            Sqrt => Ok(mpc_sqrt(z, prec, rnd)),
            Cbrt => mpc_cbrt(z, prec, rnd),
            Ln => Ok(mpc_log(z, prec, rnd)),
            Atan => Ok(mpc_atan(z, prec, rnd)),
            Exp => Ok(mpc_exp(z, prec, rnd)),
            Expj => Ok(mpc_expj(z, prec, rnd)),
            Expjpi => Ok(mpc_expjpi(z, prec, rnd)),
            Sin => Ok(mpc_sin(z, prec, rnd)),
            Cos => Ok(mpc_cos(z, prec, rnd)),
            Tan => mpc_tan(z, prec, rnd),
            Sinh => Ok(mpc_sinh(z, prec, rnd)),
            Cosh => Ok(mpc_cosh(z, prec, rnd)),
            Tanh => mpc_tanh(z, prec, rnd),
            Asin => mpc_asin(z, prec, rnd),
            Acos => mpc_acos(z, prec, rnd),
            Asinh => mpc_asinh(z, prec, rnd),
            Acosh => mpc_acosh(z, prec, rnd),
            Atanh => Ok(mpc_atanh(z, prec, rnd)),
            Sinpi => Ok(mpc_sin_pi(z, prec, rnd)),
            Cospi => Ok(mpc_cos_pi(z, prec, rnd)),
            Floor => Ok(mpc_floor(z, prec, rnd)),
            Ceil => Ok(mpc_ceil(z, prec, rnd)),
            Nint => Ok(mpc_nint(z, prec, rnd)),
            Frac => Ok(mpc_frac(z, prec, rnd)),
            Fib => mpc_fibonacci(z, prec, rnd),
            Agm1 => mpc_agm1(z, prec, rnd),
        }
    }
}

/// `f(x, prec=prec, rounding=rnd)` for a wrapped function `f`: the mpf
/// function, or on `ComplexResult` (unless `trap_complex`) the mpc
/// function of `(x, 0)`.
pub fn builtin_with(f: Builtin, x: &Value, prec: i64, rnd: Rnd) -> CtxResult<Value> {
    match x {
        Value::Mpf(v) => match f.mpf_f(v, prec, rnd) {
            Ok(r) => Ok(Value::Mpf(r)),
            Err(MpfError::ComplexResult(m)) => {
                if trap_complex() {
                    return Err(MpfError::ComplexResult(m).into());
                }
                Ok(Value::Mpc(f.mpc_f(&(v.clone(), FZERO), prec, rnd)?))
            }
            Err(e) => Err(e.into()),
        },
        Value::Mpc(z) => Ok(Value::Mpc(f.mpc_f(z, prec, rnd)?)),
    }
}

/// `f(x)` for a wrapped function `f`, at the working precision.
pub fn builtin(f: Builtin, x: &Value) -> CtxResult<Value> {
    let (prec, rounding) = prec_rounding();
    builtin_with(f, x, prec, rounding)
}

macro_rules! wrapped_builtins {
    ($($name:ident => $b:ident),* $(,)?) => {
        $(
            #[doc = concat!("`ctx.", stringify!($name), "(x)`.")]
            pub fn $name(x: &Value) -> CtxResult<Value> {
                builtin(Builtin::$b, x)
            }
        )*
    };
}

wrapped_builtins! {
    sqrt => Sqrt, cbrt => Cbrt, ln => Ln, atan => Atan, exp => Exp,
    expj => Expj, expjpi => Expjpi, sin => Sin, cos => Cos, tan => Tan,
    sinh => Sinh, cosh => Cosh, tanh => Tanh, asin => Asin, acos => Acos,
    asinh => Asinh, acosh => Acosh, atanh => Atanh, sinpi => Sinpi,
    cospi => Cospi, floor => Floor, ceil => Ceil, nint => Nint, frac => Frac,
    fib => Fib, agm1 => Agm1,
}

// ---------------------------------------------------------------------
// functions/functions.py
// ---------------------------------------------------------------------

/// The `f_wrapped` that `defun_wrapped` makes: run `f` with 10 more bits,
/// then round the result (`+retval`) at the caller's precision.
fn defun_wrapped(f: impl FnOnce() -> CtxResult<Value>) -> CtxResult<Value> {
    let p = prec();
    set_prec(p + 10);
    let r = f();
    set_prec(p);
    Ok(pos(&r?))
}

/// `ctx.cot(z)`.
pub fn cot(z: &Value) -> CtxResult<Value> {
    defun_wrapped(|| div(&one(), &tan(z)?))
}

/// `ctx.sec(z)`.
pub fn sec(z: &Value) -> CtxResult<Value> {
    defun_wrapped(|| div(&one(), &cos(z)?))
}

/// `ctx.csc(z)`.
pub fn csc(z: &Value) -> CtxResult<Value> {
    defun_wrapped(|| div(&one(), &sin(z)?))
}

/// `ctx.coth(z)`.
pub fn coth(z: &Value) -> CtxResult<Value> {
    defun_wrapped(|| div(&one(), &tanh(z)?))
}

/// `ctx.sech(z)`.
pub fn sech(z: &Value) -> CtxResult<Value> {
    defun_wrapped(|| div(&one(), &cosh(z)?))
}

/// `ctx.csch(z)`.
pub fn csch(z: &Value) -> CtxResult<Value> {
    defun_wrapped(|| div(&one(), &sinh(z)?))
}

/// `ctx.pi * 0.5` (an mpf times a float).
fn half_pi() -> Value {
    mul(&CtxConstant::Pi.value(), &Value::Mpf(from_f64(0.5)))
}

/// `ctx.acot(z)`.
pub fn acot(z: &Value) -> CtxResult<Value> {
    defun_wrapped(|| {
        if !truthy(z) {
            return Ok(half_pi());
        }
        atan(&div(&one(), z)?)
    })
}

/// `ctx.asec(z)`.
pub fn asec(z: &Value) -> CtxResult<Value> {
    defun_wrapped(|| acos(&div(&one(), z)?))
}

/// `ctx.acsc(z)`.
pub fn acsc(z: &Value) -> CtxResult<Value> {
    defun_wrapped(|| asin(&div(&one(), z)?))
}

/// `ctx.acoth(z)`.
pub fn acoth(z: &Value) -> CtxResult<Value> {
    defun_wrapped(|| {
        if !truthy(z) {
            // ctx.pi * 0.5j: mpf times a Python complex
            let (prec, rounding) = prec_rounding();
            let pi = CtxConstant::Pi.eval(prec, rounding);
            let t = (from_f64(0.0), from_f64(0.5));
            return Ok(Value::Mpc(mpc_mul_mpf(&t, &pi, prec, rounding)));
        }
        atanh(&div(&one(), z)?)
    })
}

/// `ctx.asech(z)`.
pub fn asech(z: &Value) -> CtxResult<Value> {
    defun_wrapped(|| acosh(&div(&one(), z)?))
}

/// `ctx.acsch(z)`.
pub fn acsch(z: &Value) -> CtxResult<Value> {
    defun_wrapped(|| asinh(&div(&one(), z)?))
}

/// `ctx.sign(x)`.
pub fn sign(x: &Value) -> CtxResult<Value> {
    if !truthy(x) || isnan(x) {
        return Ok(x.clone());
    }
    match x {
        Value::Mpf(_) => {
            if gt(x, &Value::Mpf(FZERO))? {
                Ok(one())
            } else {
                Ok(neg(&one()))
            }
        }
        Value::Mpc(_) => div(x, &abs(x)),
    }
}

/// `ctx.agm(a, b)`; `b = None` is the default `b=1`.
pub fn agm(a: &Value, b: Option<&Value>) -> CtxResult<Value> {
    let b = match b {
        None => return agm1(a),
        Some(b) => {
            if eq(b, &Value::Mpf(FONE)) {
                return agm1(a);
            }
            b
        }
    };
    let (prec, rounding) = prec_rounding();
    if let (Value::Mpf(x), Value::Mpf(y)) = (a, b) {
        match mpf_agm(x, y, prec, rounding) {
            Ok(v) => return Ok(Value::Mpf(v)),
            Err(MpfError::ComplexResult(_)) => {}
            Err(e) => return Err(e.into()),
        }
    }
    let ac = match a {
        Value::Mpf(x) => (x.clone(), FZERO),
        Value::Mpc(z) => z.clone(),
    };
    let bc = match b {
        Value::Mpf(x) => (x.clone(), FZERO),
        Value::Mpc(z) => z.clone(),
    };
    Ok(Value::Mpc(mpc_agm(&ac, &bc, prec, rounding)?))
}

/// `ctx.sinc(x)`.
pub fn sinc(x: &Value) -> CtxResult<Value> {
    defun_wrapped(|| {
        if isinf(x) {
            return rdiv_int(&Int::ONE, x);
        }
        if !truthy(x) {
            return Ok(add(x, &convert_int(&Int::ONE)));
        }
        div(&sin(x)?, x)
    })
}

/// `ctx.sincpi(x)`.
pub fn sincpi(x: &Value) -> CtxResult<Value> {
    defun_wrapped(|| {
        if isinf(x) {
            return rdiv_int(&Int::ONE, x);
        }
        if !truthy(x) {
            return Ok(add(x, &convert_int(&Int::ONE)));
        }
        div(&sinpi(x)?, &mul(&CtxConstant::Pi.value(), x))
    })
}

/// `ctx._rootof1(k, n)`.
fn rootof1(k: i64, n: i64) -> CtxResult<Value> {
    if n == 0 {
        return Err(MpfError::ZeroDivision.into());
    }
    let k = Int::from(k)
        .py_mod(&Int::from(n))
        .expect("n != 0")
        .as_i64()
        .expect("|k| < |n|");
    if k == 0 {
        return Ok(one());
    } else if 2 * k == n {
        return Ok(neg(&one()));
    } else if 4 * k == n {
        return Ok(j());
    } else if 4 * k == 3 * n {
        return Ok(neg(&j()));
    }
    let mk = Value::Mpf(mpf(&Num::Int(Int::from(k)))?);
    let t = div(&mul_int(&mk, &Int::from(2)), &convert_int(&Int::from(n)))?;
    expjpi(&t)
}

/// `ctx._nthroot(x, n)`.
fn nthroot_impl(x: &Value, n: i64) -> CtxResult<Value> {
    let (prec, rounding) = prec_rounding();
    let z = match x {
        Value::Mpf(v) => match mpf_nthroot(v, n, prec, rounding) {
            Ok(r) => return Ok(Value::Mpf(r)),
            Err(MpfError::ComplexResult(m)) => {
                if trap_complex() {
                    return Err(MpfError::ComplexResult(m).into());
                }
                (v.clone(), FZERO)
            }
            Err(e) => return Err(e.into()),
        },
        Value::Mpc(z) => z.clone(),
    };
    Ok(Value::Mpc(mpc_nthroot(&z, n, prec, rounding)?))
}

/// `ctx.root(x, n, k)` (also `ctx.nthroot`).
pub fn root(x: &Value, n: i64, k: i64) -> CtxResult<Value> {
    if k != 0 {
        // Special case: there is an exact real root
        if (n & 1 != 0 && 2 * k == n - 1) && !truthy(&im(x)) && lt(&re(x), &Value::Mpf(FZERO))? {
            return Ok(neg(&root(&neg(x), n, 0)?));
        }
        // Multiply by root of unity
        let p = prec();
        set_prec(p + 10);
        let v = root(x, n, 0).and_then(|r| Ok(mul(&r, &rootof1(k, n)?)));
        set_prec(p);
        return Ok(pos(&v?));
    }
    nthroot_impl(x, n)
}

/// `ctx.arg(x)`.
pub fn arg(x: &Value) -> CtxResult<Value> {
    atan2(&Value::Mpf(x.imag()), &Value::Mpf(x.real()))
}

/// `ctx.fabs(x)`.
pub fn fabs(x: &Value) -> Value {
    abs(x)
}

/// `ctx.re(x)`.
pub fn re(x: &Value) -> Value {
    Value::Mpf(x.real())
}

/// `ctx.im(x)`.
pub fn im(x: &Value) -> Value {
    Value::Mpf(x.imag())
}

/// `ctx.conj(x)`: an mpf is returned as it is.
pub fn conj(x: &Value) -> Value {
    match x {
        Value::Mpf(_) => x.clone(),
        Value::Mpc(z) => {
            let (prec, rounding) = prec_rounding();
            Value::Mpc(mpc_conjugate(z, prec, rounding))
        }
    }
}

/// `ctx.log(x, b)`; `b = None` is the natural logarithm.
pub fn log(x: &Value, b: Option<&Value>) -> CtxResult<Value> {
    let Some(b) = b else {
        return ln(x);
    };
    let wp = prec() + 20;
    let r = rounding();
    div(
        &builtin_with(Builtin::Ln, x, wp, r)?,
        &builtin_with(Builtin::Ln, b, wp, r)?,
    )
}

/// `ctx.log10(x)`.
pub fn log10(x: &Value) -> CtxResult<Value> {
    log(x, Some(&convert_int(&Int::from(10))))
}

/// `ctx.degrees(x)`.
pub fn degrees(x: &Value) -> CtxResult<Value> {
    div(x, &CtxConstant::Degree.value())
}

/// `ctx.radians(x)`.
pub fn radians(x: &Value) -> Value {
    mul(x, &CtxConstant::Degree.value())
}

/// `ctx.power(x, y)`.
pub fn power(x: &Value, y: &Value) -> CtxResult<Value> {
    pow(x, y)
}

/// `ctx.nthroot(x, n)`.
pub fn nthroot(x: &Value, n: i64) -> CtxResult<Value> {
    root(x, n, 0)
}

// ---------------------------------------------------------------------
// ctx_mp.py
// ---------------------------------------------------------------------

/// `ctx.hypot(x, y)`.
pub fn hypot(x: &Value, y: &Value) -> CtxResult<Value> {
    let (prec, rounding) = prec_rounding();
    Ok(Value::Mpf(mpf_hypot(
        x.mpf_or_attr()?,
        y.mpf_or_attr()?,
        prec,
        rounding,
    )?))
}

/// `ctx.atan2(y, x)`.
pub fn atan2(y: &Value, x: &Value) -> CtxResult<Value> {
    let (prec, rounding) = prec_rounding();
    let (yv, xv) = (y.mpf_or_attr()?, x.mpf_or_attr()?);
    Ok(Value::Mpf(mpf_atan2(yv, xv, prec, rounding)))
}

/// `ctx.cos_sin(x, prec=prec, rounding=rnd)`.
pub fn cos_sin_with(x: &Value, prec: i64, rnd: Rnd) -> (Value, Value) {
    match x {
        Value::Mpf(v) => {
            let (c, s) = mpf_cos_sin(v, prec, rnd);
            (Value::Mpf(c), Value::Mpf(s))
        }
        Value::Mpc(z) => {
            let (c, s) = mpc_cos_sin(z, prec, rnd);
            (Value::Mpc(c), Value::Mpc(s))
        }
    }
}

/// `ctx.cos_sin(x)`.
pub fn cos_sin(x: &Value) -> (Value, Value) {
    let (prec, rounding) = prec_rounding();
    cos_sin_with(x, prec, rounding)
}

/// `ctx.cospi_sinpi(x, prec=prec, rounding=rnd)`.
pub fn cospi_sinpi_with(x: &Value, prec: i64, rnd: Rnd) -> (Value, Value) {
    match x {
        Value::Mpf(v) => {
            let (c, s) = mpf_cos_sin_pi(v, prec, rnd);
            (Value::Mpf(c), Value::Mpf(s))
        }
        Value::Mpc(z) => {
            let (c, s) = mpc_cos_sin_pi(z, prec, rnd);
            (Value::Mpc(c), Value::Mpc(s))
        }
    }
}

/// `ctx.cospi_sinpi(x)`.
pub fn cospi_sinpi(x: &Value) -> (Value, Value) {
    let (prec, rounding) = prec_rounding();
    cospi_sinpi_with(x, prec, rounding)
}

/// `ctx.ldexp(x, n)`: no rounding.
pub fn ldexp(x: &Value, n: i64) -> CtxResult<Value> {
    Ok(Value::Mpf(mpf_shift(x.mpf_or_attr()?, n)))
}

/// `ctx.frexp(x)`.
pub fn frexp(x: &Value) -> CtxResult<(Value, i64)> {
    let (y, n) = mpf_frexp(x.mpf_or_attr()?)?;
    Ok((Value::Mpf(y), n))
}

/// `ctx.isnan(x)`.
pub fn isnan(x: &Value) -> bool {
    match x {
        Value::Mpf(v) => *v == FNAN,
        Value::Mpc((re, im)) => *re == FNAN || *im == FNAN,
    }
}

/// `ctx.isinf(x)`.
pub fn isinf(x: &Value) -> bool {
    let inf = |v: &Mpf| *v == FINF || *v == FNINF;
    match x {
        Value::Mpf(v) => inf(v),
        Value::Mpc((re, im)) => inf(re) || inf(im),
    }
}

/// `ctx.isfinite(x)`.
pub fn isfinite(x: &Value) -> bool {
    !(isinf(x) || isnan(x))
}

/// `ctx.isnormal(x)`.
pub fn isnormal(x: &Value) -> bool {
    match x {
        Value::Mpf(v) => !v.man.is_zero(),
        Value::Mpc((re, im)) => {
            let re_normal = !re.man.is_zero();
            let im_normal = !im.man.is_zero();
            if *re == FZERO {
                return im_normal;
            }
            if *im == FZERO {
                return re_normal;
            }
            re_normal && im_normal
        }
    }
}

/// `ctx.isint(x, gaussian)`.
pub fn isint(x: &Value, gaussian: bool) -> bool {
    let is_int = |v: &Mpf| (!v.man.is_zero() && v.exp >= 0) || *v == FZERO;
    match x {
        Value::Mpf(v) => is_int(v),
        Value::Mpc((re, im)) => {
            if gaussian {
                return is_int(re) && is_int(im);
            }
            is_int(re) && *im == FZERO
        }
    }
}

/// `ctx.isnpint(x)`: whether `x` is a nonpositive integer.
pub fn isnpint(x: &Value) -> bool {
    if !truthy(x) {
        return true;
    }
    match x {
        Value::Mpf(v) => v.sign == 1 && v.exp >= 0,
        Value::Mpc((re, im)) => *im == FZERO && isnpint(&Value::Mpf(re.clone())),
    }
}

/// A result of `ctx.mag`: an int or `-inf`, `+inf`, `nan`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mag {
    Finite(i64),
    NegInf,
    PosInf,
    Nan,
}

impl Mag {
    /// Python's `self > other` between ints and mpf infinities or nan.
    fn gt(self, other: Mag) -> bool {
        use Mag::*;
        match (self, other) {
            (Nan, _) | (_, Nan) => false,
            (NegInf, _) => false,
            (PosInf, o) => o != PosInf,
            (Finite(_), NegInf) => true,
            (Finite(a), Finite(b)) => a > b,
            (Finite(_), PosInf) => false,
        }
    }
}

/// `ctx._mpf_mag(x)`.
fn mpf_mag(x: &Mpf) -> Mag {
    if !x.man.is_zero() {
        return Mag::Finite(x.exp + x.bc);
    }
    if *x == FZERO {
        return Mag::NegInf;
    }
    if *x == FINF || *x == FNINF {
        return Mag::PosInf;
    }
    Mag::Nan
}

/// `ctx.mag(x)`.
pub fn mag(x: &Value) -> Mag {
    match x {
        Value::Mpf(v) => mpf_mag(v),
        Value::Mpc((r, i)) => {
            if *r == FZERO {
                return mpf_mag(i);
            }
            if *i == FZERO {
                return mpf_mag(r);
            }
            let (a, b) = (mpf_mag(r), mpf_mag(i));
            // 1 + max(a, b)
            match if b.gt(a) { b } else { a } {
                Mag::Finite(m) => Mag::Finite(1 + m),
                m => m,
            }
        }
    }
}

/// `ctx.nint_distance(x)` for an mpf or mpc: `(n, d)` with `d = None`
/// for `-inf`.
pub fn nint_distance(x: &Value) -> CtxResult<(Int, Option<i64>)> {
    let (re, im_dist) = match x {
        Value::Mpf(v) => (v, None),
        Value::Mpc((re, im)) => {
            let im_dist = if !im.man.is_zero() {
                Some(im.exp + im.bc)
            } else if *im == FZERO {
                None
            } else {
                return Err(MpfError::Value("requires a finite number".into()).into());
            };
            (re, im_dist)
        }
    };
    let mag = re.exp + re.bc;
    let (n, re_dist);
    // |x| < 0.5
    if mag < 0 {
        n = Int::ZERO;
        re_dist = Some(mag);
    } else if !re.man.is_zero() {
        let mut man = re.man.clone();
        let exp = re.exp;
        let mut nn;
        // exact integer
        if exp >= 0 {
            nn = &man << exp as u64;
            re_dist = None;
        }
        // exact half-integer
        else if exp == -1 {
            nn = &(&man >> 1) + 1;
            re_dist = Some(0);
        } else {
            let d = (-exp - 1) as u64;
            let mut t = &man >> d;
            if t.is_odd() {
                t = &t + 1;
                man = &(&t << d) - &man;
            } else {
                man = &man - &(&t << d);
            }
            nn = &t >> 1;
            re_dist = Some(exp + bitcount(&man));
        }
        if re.sign == 1 {
            nn = -nn;
        }
        n = nn;
    } else if *re == FZERO {
        re_dist = None;
        n = Int::ZERO;
    } else {
        return Err(MpfError::Value("requires a finite number".into()).into());
    }
    // max(re_dist, im_dist), with -inf below every int
    Ok((n, re_dist.max(im_dist)))
}

/// `ctx.nint_distance(x)` for an `mpq`.
fn nint_distance_mpq(p: &Int, q: &Int) -> CtxResult<(Int, Option<i64>)> {
    let (mut n, r) = p
        .div_mod(q)
        .map_err(|_| CtxError::Mpf(MpfError::ZeroDivision))?;
    if &(&r * &Int::from(2)) >= q {
        n = &n + 1;
    } else if r.is_zero() {
        return Ok((n, None));
    }
    let d = bitcount(&(p - &(&n * q)).abs()) - bitcount(q);
    Ok((n, Some(d)))
}

/// A result of `ctx._convert_param(x)`: a typed parameter, or `(x, 'U')`.
#[derive(Clone, Debug, PartialEq)]
pub enum Converted {
    Param(HypParam),
    Unknown(Value),
}

/// `ctx._convert_param(x)`.
pub fn convert_param(x: &Num) -> CtxResult<Converted> {
    let mpf_param = |v: &Mpf, orig: &Value| -> CtxResult<Converted> {
        if !v.man.is_zero() {
            if v.exp >= -4 {
                let man = if v.sign == 1 { -&v.man } else { v.man.clone() };
                if v.exp >= 0 {
                    return Ok(Converted::Param(HypParam::Z(&man << v.exp as u64)));
                }
                let (p, q) = mpq_reduce(&man, &(&Int::ONE << (-v.exp) as u64))?;
                return Ok(Converted::Param(HypParam::Q(p, q)));
            }
            return Ok(Converted::Param(HypParam::R(v.clone())));
        } else if v.exp == 0 {
            return Ok(Converted::Param(HypParam::Z(Int::ZERO)));
        }
        Ok(Converted::Unknown(orig.clone()))
    };
    let classify = |val: &Value| -> CtxResult<Converted> {
        match val {
            Value::Mpc((re, im)) => {
                if *im != FZERO {
                    return Ok(Converted::Param(HypParam::C((re.clone(), im.clone()))));
                }
                mpf_param(re, val)
            }
            Value::Mpf(v) => mpf_param(v, val),
        }
    };
    let ratio = |p: &Int, q: &Int| -> CtxResult<Converted> {
        let r = p
            .py_mod(q)
            .map_err(|_| CtxError::Mpf(MpfError::ZeroDivision))?;
        if r.is_zero() {
            return Ok(Converted::Param(HypParam::Z(
                p.floor_div(q).expect("q != 0"),
            )));
        }
        let (p, q) = mpq_reduce(p, q)?;
        Ok(Converted::Param(HypParam::Q(p, q)))
    };
    match x {
        Num::Val(v) => classify(v),
        Num::Int(n) => Ok(Converted::Param(HypParam::Z(n.clone()))),
        Num::Mpq(p, q) => ratio(p, q),
        Num::Str(s) if s.contains('/') => {
            // p, q = x.split('/'); p = int(p); q = int(q)
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() != 2 {
                return Err(MpfError::Value("too many values to unpack".into()).into());
            }
            let parse = |t: &str| {
                Int::parse_python(t).ok_or_else(|| {
                    CtxError::Mpf(MpfError::Value(format!(
                        "invalid literal for int() with base 10: {t:?}"
                    )))
                })
            };
            ratio(&parse(parts[0])?, &parse(parts[1])?)
        }
        _ => classify(&convert(x)?),
    }
}

/// `ctx._default_hyper_maxprec(p)`.
pub fn default_hyper_maxprec(p: i64) -> i64 {
    (1000.0 * (p as f64).powf(0.25) + (4 * p) as f64) as i64
}

/// The keyword arguments of `ctx.hypsum` that it reads.
#[derive(Clone, Debug)]
pub struct HypsumOpts {
    pub accurate_small: bool,
    pub maxprec: Option<i64>,
    pub maxterms: Option<i64>,
    pub zeroprec: Option<i64>,
}

impl Default for HypsumOpts {
    fn default() -> Self {
        HypsumOpts {
            accurate_small: true,
            maxprec: None,
            maxterms: None,
            zeroprec: None,
        }
    }
}

/// `ctx.hypsum(p, q, flags, coeffs, z, accurate_small, **kwargs)`, with
/// each coefficient carrying its flag.
pub fn hypsum(p: usize, coeffs: &[HypParam], z: &Value, opts: &HypsumOpts) -> CtxResult<Value> {
    let v = match z {
        Value::Mpf(x) => HypArg::R(x.clone()),
        Value::Mpc(c) => HypArg::C(c.clone()),
    };
    let prec = prec();
    let maxprec = opts.maxprec.unwrap_or_else(|| default_hyper_maxprec(prec));
    let mut extraprec: i64 = 50;
    let mut epsshift: i64 = 25;
    // Jumps in magnitude occur when parameters are close to negative
    // integers. We must ensure that these terms are included in
    // the sum and added accurately
    let mut magnitude_check: Vec<Int> = Vec::new();
    let mut max_total_jump: i64 = 0;
    for (i, c) in coeffs.iter().enumerate() {
        let (n, d) = match c {
            HypParam::Z(c) => {
                if i >= p && !c.is_positive() {
                    let mut ok = false;
                    for cc in &coeffs[..p] {
                        // Note: c <= cc or c < cc, depending on convention
                        if let HypParam::Z(cc) = cc
                            && !cc.is_positive()
                            && c <= cc
                        {
                            ok = true;
                        }
                    }
                    if !ok {
                        // "pole in hypergeometric series"
                        return Err(MpfError::ZeroDivision.into());
                    }
                }
                continue;
            }
            HypParam::Q(pp, qq) => nint_distance_mpq(pp, qq)?,
            HypParam::R(x) => nint_distance(&Value::Mpf(x.clone()))?,
            HypParam::C(x) => nint_distance(&Value::Mpc(x.clone()))?,
        };
        let Some(d) = d else {
            return Err(CtxError::NotPorted(
                "hypsum with an integer-valued parameter not tagged Z",
            ));
        };
        let n = -n;
        let d = -d;
        if i >= p && !n.is_negative() && d > 4 {
            if !magnitude_check.contains(&n) {
                magnitude_check.push(n);
            }
            extraprec = extraprec.max(d - prec + 60);
        }
        max_total_jump += d.abs();
    }
    loop {
        if extraprec > maxprec {
            return Err(MpfError::Value(format!(
                "hypsum() failed to converge to the requested {} bits of accuracy\n\
                 using a working precision of {} bits. Try with a higher maxprec,\n\
                 maxterms, or set zeroprec.",
                prec,
                prec + extraprec
            ))
            .into());
        }
        let wp = prec + extraprec;
        let mut mag_dict: HashMap<Int, Option<i64>> =
            magnitude_check.iter().map(|n| (n.clone(), None)).collect();
        let (zv, have_complex, magnitude) = hyp_summator(
            p,
            coeffs,
            &v,
            prec,
            wp,
            epsshift,
            &mut mag_dict,
            opts.maxterms,
        )?;
        let cancel = -magnitude;
        let mut jumps_resolved = true;
        if extraprec < max_total_jump {
            for n in mag_dict.values() {
                if n.is_none_or(|n| n < prec) {
                    jumps_resolved = false;
                    break;
                }
            }
        }
        let accurate = cancel < extraprec - 25 - 5 || !opts.accurate_small;
        if jumps_resolved {
            if accurate {
                return Ok(match zv {
                    HypValue::Real(x) => Value::Mpf(x),
                    HypValue::Complex(z) => Value::Mpc(z),
                });
            }
            // zero?
            if let Some(zeroprec) = opts.zeroprec
                && cancel > zeroprec
            {
                if have_complex {
                    return Ok(Value::Mpc((FZERO, FZERO)));
                }
                return Ok(Value::Mpf(FZERO));
            }
        }
        // Some near-singularities were not included, so increase
        // precision and repeat until they are
        extraprec *= 2;
        // Possible workaround for bad roundoff in fixed-point arithmetic
        epsshift += 5;
        extraprec += 5;
    }
}
