//! Differential tests against mpmath for milestone M2: replays
//! `oracle/diff/mpf2.json` (written by `oracle/gen_diff_mpf2.py`).
//!
//! Each vector in `vectors` runs from a fresh state (no memoized
//! constants, empty history caches, the default context), as the
//! generator ran it; each list in `sequences` runs in order from one
//! fresh state, because mpmath's memos carry over between those calls.
//! Results are encoded back into the generator's JSON form and compared
//! exactly.

use std::path::Path;

use rsympy_mpf::ctx::{
    self, Builtin, Converted, CtxConstant, CtxError, HypsumOpts, Mag, Num, Value,
};
use rsympy_mpf::hyper::{self, HypParam};
use rsympy_mpf::*;
use rsympy_num::Int;
use serde_json::{Value as J, json};

/// Vectors whose result is `NotPortedYet` (none: every vector runs).
const NOT_PORTED: usize = 0;

fn load() -> J {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../oracle/diff/mpf2.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).expect("valid JSON")
}

// ---------------------------------------------------------------------
// Decoding
// ---------------------------------------------------------------------

fn m(v: &J) -> Mpf {
    Mpf {
        sign: v[0].as_u64().unwrap() as u8,
        man: v[1].as_str().unwrap().parse().unwrap(),
        exp: v[2].as_i64().unwrap(),
        bc: v[3].as_i64().unwrap(),
    }
}

fn c(v: &J) -> Mpc {
    let p = &v["c"];
    (m(&p[0]), m(&p[1]))
}

fn int(v: &J) -> Int {
    v["i"].as_str().unwrap().parse().unwrap()
}

fn i(v: &J) -> i64 {
    v.as_i64().unwrap()
}

fn val(v: &J) -> Value {
    if v.is_array() {
        Value::Mpf(m(v))
    } else {
        Value::Mpc(c(v))
    }
}

fn float(v: &J) -> f64 {
    match v["fb"].as_str().unwrap() {
        "nan" => f64::NAN,
        bits => f64::from_bits(bits.parse::<i64>().unwrap() as u64),
    }
}

fn num(v: &J) -> Num {
    if v.is_array() {
        return Num::Val(Value::Mpf(m(v)));
    }
    if v.get("i").is_some() {
        Num::Int(int(v))
    } else if v.get("fb").is_some() {
        Num::Float(float(v))
    } else if let Some(t) = v.get("t") {
        Num::Complex(float(&t[0]), float(&t[1]))
    } else if let Some(q) = v.get("q") {
        Num::Mpq(
            q[0].as_str().unwrap().parse().unwrap(),
            q[1].as_str().unwrap().parse().unwrap(),
        )
    } else if let Some(s) = v.get("s") {
        Num::Str(s.as_str().unwrap().to_string())
    } else {
        Num::Val(Value::Mpc(c(v)))
    }
}

fn param(v: &J) -> HypParam {
    if let Some(z) = v.get("Z") {
        HypParam::Z(int(z))
    } else if let Some(q) = v.get("Q") {
        let q = &q["q"];
        HypParam::Q(
            q[0].as_str().unwrap().parse().unwrap(),
            q[1].as_str().unwrap().parse().unwrap(),
        )
    } else if let Some(r) = v.get("R") {
        HypParam::R(m(r))
    } else {
        HypParam::C(c(&v["C"]))
    }
}

fn rnd(v: &J) -> Rnd {
    match v.as_str().unwrap() {
        "n" => Rnd::Nearest,
        "f" => Rnd::Floor,
        "c" => Rnd::Ceiling,
        "u" => Rnd::Up,
        "d" => Rnd::Down,
        other => panic!("rounding mode {other}"),
    }
}

// ---------------------------------------------------------------------
// Encoding
// ---------------------------------------------------------------------

fn jm(x: &Mpf) -> J {
    json!([x.sign, x.man.to_string(), x.exp, x.bc])
}

fn jc(z: &Mpc) -> J {
    json!({"c": [jm(&z.0), jm(&z.1)]})
}

fn ji(n: impl ToString) -> J {
    json!({"i": n.to_string()})
}

fn jt(xs: Vec<J>) -> J {
    json!({ "t": xs })
}

fn jv(x: &Value) -> J {
    match x {
        Value::Mpf(v) => jm(v),
        Value::Mpc(z) => jc(z),
    }
}

fn jf(x: f64) -> J {
    if x.is_nan() {
        return json!({"fb": "nan"});
    }
    json!({"fb": (x.to_bits() as i64).to_string()})
}

fn jmag(x: Mag) -> J {
    match x {
        Mag::Finite(n) => ji(n),
        Mag::NegInf => jm(&FNINF),
        Mag::PosInf => jm(&FINF),
        Mag::Nan => jm(&FNAN),
    }
}

fn jparam(x: &Converted) -> J {
    match x {
        Converted::Param(HypParam::Z(n)) => json!({"Z": ji(n)}),
        Converted::Param(HypParam::Q(p, q)) => json!({"Q": {"q": [p.to_string(), q.to_string()]}}),
        Converted::Param(HypParam::R(x)) => json!({"R": jm(x)}),
        Converted::Param(HypParam::C(z)) => json!({"C": jc(z)}),
        Converted::Unknown(v) => json!({"U": jv(v)}),
    }
}

trait Class {
    fn cls(&self) -> &'static str;
}

impl Class for MpfError {
    fn cls(&self) -> &'static str {
        self.class()
    }
}

impl Class for CtxError {
    fn cls(&self) -> &'static str {
        self.class()
    }
}

fn err(class: &str) -> J {
    json!({ "e": class })
}

fn r<T, E: Class>(x: Result<T, E>, enc: impl FnOnce(T) -> J) -> J {
    match x {
        Ok(v) => enc(v),
        Err(e) => err(e.cls()),
    }
}

fn pair_m((a, b): (Mpf, Mpf)) -> J {
    jt(vec![jm(&a), jm(&b)])
}

fn pair_i((a, b): (Int, Int)) -> J {
    jt(vec![ji(a), ji(b)])
}

fn pair_c((a, b): (Mpc, Mpc)) -> J {
    jt(vec![jc(&a), jc(&b)])
}

fn pair_v((a, b): (Value, Value)) -> J {
    jt(vec![jv(&a), jv(&b)])
}

// ---------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------

fn constant_fn(name: &str) -> Option<fn(i64, Rnd) -> Mpf> {
    Some(match name {
        "mpf_pi" => mpf_pi,
        "mpf_e" => mpf_e,
        "mpf_ln2" => mpf_ln2,
        "mpf_ln10" => mpf_ln10,
        "mpf_phi" => mpf_phi,
        "mpf_degree" => mpf_degree,
        "mpf_sqrtpi" => mpf_sqrtpi,
        "mpf_ln_sqrt2pi" => mpf_ln_sqrt2pi,
        _ => return None,
    })
}

fn fixed_fn(name: &str) -> Option<fn(i64) -> Int> {
    Some(match name {
        "pi_fixed" => pi_fixed,
        "e_fixed" => e_fixed,
        "ln2_fixed" => ln2_fixed,
        "ln10_fixed" => ln10_fixed,
        "phi_fixed" => phi_fixed,
        "degree_fixed" => degree_fixed,
        "sqrtpi_fixed" => sqrtpi_fixed,
        "ln_sqrt2pi_fixed" => ln_sqrt2pi_fixed,
        _ => return None,
    })
}

type MpfFn = fn(&Mpf, i64, Rnd) -> MpfResult<Mpf>;
type MpcFn = fn(&Mpc, i64, Rnd) -> MpfResult<Mpc>;
type CtxFn = fn(&Value) -> Result<Value, CtxError>;

/// Real functions `f(x, prec, rnd) -> Mpf`.
fn mpf_unary(name: &str) -> Option<MpfFn> {
    Some(match name {
        "mpf_exp" => |x, p, r| Ok(mpf_exp(x, p, r)),
        "mpf_cosh" => |x, p, r| Ok(mpf_cosh(x, p, r)),
        "mpf_sinh" => |x, p, r| Ok(mpf_sinh(x, p, r)),
        "mpf_tanh" => |x, p, r| Ok(mpf_tanh(x, p, r)),
        "mpf_log" => mpf_log,
        "mpf_cbrt" => mpf_cbrt,
        "mpf_atan" => |x, p, r| Ok(mpf_atan(x, p, r)),
        "mpf_asin" => mpf_asin,
        "mpf_acos" => mpf_acos,
        "mpf_asinh" => |x, p, r| Ok(mpf_asinh(x, p, r)),
        "mpf_acosh" => mpf_acosh,
        "mpf_atanh" => mpf_atanh,
        "mpf_cos" => |x, p, r| Ok(mpf_cos(x, p, r)),
        "mpf_sin" => |x, p, r| Ok(mpf_sin(x, p, r)),
        "mpf_tan" => mpf_tan,
        "mpf_cos_pi" => |x, p, r| Ok(mpf_cos_pi(x, p, r)),
        "mpf_sin_pi" => |x, p, r| Ok(mpf_sin_pi(x, p, r)),
        "mpf_tan_pi" => mpf_tan_pi,
        "mpf_fibonacci" => |x, p, r| Ok(mpf_fibonacci(x, p, r)),
        "mpf_expj" => mpf_expj,
        "mpf_expjpi" => mpf_expjpi,
        "mpf_agm1" => hyper::mpf_agm1,
        _ => return None,
    })
}

/// Complex functions `f(z, prec, rnd) -> Mpc`.
fn mpc_unary(name: &str) -> Option<MpcFn> {
    Some(match name {
        "mpc_pos" => |z, p, r| Ok(mpc_pos(z, p, r)),
        "mpc_neg" => |z, p, r| Ok(mpc_neg(z, p, r)),
        "mpc_conjugate" => |z, p, r| Ok(mpc_conjugate(z, p, r)),
        "mpc_floor" => |z, p, r| Ok(mpc_floor(z, p, r)),
        "mpc_ceil" => |z, p, r| Ok(mpc_ceil(z, p, r)),
        "mpc_nint" => |z, p, r| Ok(mpc_nint(z, p, r)),
        "mpc_frac" => |z, p, r| Ok(mpc_frac(z, p, r)),
        "mpc_square" => |z, p, r| Ok(mpc_square(z, p, r)),
        "mpc_reciprocal" => mpc_reciprocal,
        "mpc_sqrt" => |z, p, r| Ok(mpc_sqrt(z, p, r)),
        "mpc_cbrt" => mpc_cbrt,
        "mpc_exp" => |z, p, r| Ok(mpc_exp(z, p, r)),
        "mpc_log" => |z, p, r| Ok(mpc_log(z, p, r)),
        "mpc_cos" => |z, p, r| Ok(mpc_cos(z, p, r)),
        "mpc_sin" => |z, p, r| Ok(mpc_sin(z, p, r)),
        "mpc_tan" => mpc_tan,
        "mpc_cos_pi" => |z, p, r| Ok(mpc_cos_pi(z, p, r)),
        "mpc_sin_pi" => |z, p, r| Ok(mpc_sin_pi(z, p, r)),
        "mpc_cosh" => |z, p, r| Ok(mpc_cosh(z, p, r)),
        "mpc_sinh" => |z, p, r| Ok(mpc_sinh(z, p, r)),
        "mpc_tanh" => mpc_tanh,
        "mpc_expj" => |z, p, r| Ok(mpc_expj(z, p, r)),
        "mpc_expjpi" => |z, p, r| Ok(mpc_expjpi(z, p, r)),
        "mpc_atan" => |z, p, r| Ok(mpc_atan(z, p, r)),
        "mpc_acos" => mpc_acos,
        "mpc_asin" => mpc_asin,
        "mpc_asinh" => mpc_asinh,
        "mpc_acosh" => mpc_acosh,
        "mpc_atanh" => |z, p, r| Ok(mpc_atanh(z, p, r)),
        "mpc_fibonacci" => mpc_fibonacci,
        "mpc_agm1" => hyper::mpc_agm1,
        _ => return None,
    })
}

fn run_raw(f: &str, a: &[J], p: i64, rd: Rnd) -> J {
    if let Some(g) = constant_fn(f) {
        return jm(&g(p, rd));
    }
    if let Some(g) = fixed_fn(f) {
        return ji(g(i(&a[0])));
    }
    if let Some(g) = mpf_unary(f) {
        return r(g(&m(&a[0]), p, rd), |x| jm(&x));
    }
    if let Some(g) = mpc_unary(f) {
        return r(g(&c(&a[0]), p, rd), |x| jc(&x));
    }
    match f {
        "mpf_pow" => r(mpf_pow(&m(&a[0]), &m(&a[1]), p, rd), |x| jm(&x)),
        "mpf_nthroot" => r(mpf_nthroot(&m(&a[0]), i(&a[1]), p, rd), |x| jm(&x)),
        "mpf_cosh_sinh" => pair_m(mpf_cosh_sinh(&m(&a[0]), p, rd)),
        "mpf_cos_sin" => pair_m(mpf_cos_sin(&m(&a[0]), p, rd)),
        "mpf_cos_sin_pi" => pair_m(mpf_cos_sin_pi(&m(&a[0]), p, rd)),
        "mpf_atan2" => jm(&mpf_atan2(&m(&a[0]), &m(&a[1]), p, rd)),
        "mpf_log_hypot" => jm(&mpf_log_hypot(&m(&a[0]), &m(&a[1]), p, rd)),
        "mpf_agm" => r(hyper::mpf_agm(&m(&a[0]), &m(&a[1]), p, rd), |x| jm(&x)),
        "int_pow_fixed" => {
            let (pm, pe) = int_pow_fixed(&int(&a[0]), i(&a[1]), i(&a[2]));
            jt(vec![ji(pm), ji(pe)])
        }
        "nthroot_fixed" => ji(nthroot_fixed(&int(&a[0]), i(&a[1]), i(&a[2]), i(&a[3]))),
        "log_int_fixed" => ji(log_int_fixed(&int(&a[0]), i(&a[1]), None)),
        "agm_fixed" => ji(agm_fixed(&int(&a[0]), &int(&a[1]), i(&a[2]))),
        "log_agm" => ji(log_agm(&int(&a[0]), i(&a[1]))),
        "log_taylor" => ji(log_taylor(&int(&a[0]), i(&a[1]), i(&a[2]))),
        "log_taylor_cached" => ji(log_taylor_cached(&int(&a[0]), i(&a[1]))),
        "atan_newton" => ji(atan_newton(&int(&a[0]), i(&a[1]))),
        "atan_taylor" => ji(atan_taylor(&int(&a[0]), i(&a[1]))),
        "exponential_series" => match exponential_series(&int(&a[0]), i(&a[1]), i(&a[2]) as u8) {
            ExpSeries::Exp(v) => ji(v),
            ExpSeries::Pair(x, y) => pair_i((x, y)),
        },
        "exp_basecase" => ji(exp_basecase(&int(&a[0]), i(&a[1]))),
        "exp_expneg_basecase" => pair_i(exp_expneg_basecase(&int(&a[0]), i(&a[1]))),
        "cos_sin_basecase" => pair_i(cos_sin_basecase(&int(&a[0]), i(&a[1]))),
        "cos_sin_fixed" => pair_i(cos_sin_fixed(&int(&a[0]), i(&a[1]), None)),
        "exp_fixed" => ji(exp_fixed(&int(&a[0]), i(&a[1]), None)),
        "mod_pi2" => {
            let (t, n, wp) = mod_pi2(&int(&a[0]), i(&a[1]), i(&a[2]), i(&a[3]));
            jt(vec![ji(t), ji(n), ji(wp)])
        }
        "ifib" => ji(ifib(&int(&a[0]))),
        // libmpc
        "mpc_is_inf" => json!(mpc_is_inf(&c(&a[0]))),
        "mpc_is_infnan" => json!(mpc_is_infnan(&c(&a[0]))),
        "mpc_is_nonzero" => json!(mpc_is_nonzero(&c(&a[0]))),
        "mpc_hash" => ji(mpc_hash(&c(&a[0]))),
        "mpc_to_str" => json!({"s": mpc_to_str(&c(&a[0]), i(&a[1]), true, None, None, false)}),
        "mpc_to_complex" => match mpc_to_complex(&c(&a[0]), false, ROUND_FAST) {
            Some((x, y)) => jt(vec![jf(x), jf(y)]),
            None => err("OverflowError"),
        },
        "mpc_add" => jc(&mpc_add(&c(&a[0]), &c(&a[1]), p, rd)),
        "mpc_sub" => jc(&mpc_sub(&c(&a[0]), &c(&a[1]), p, rd)),
        "mpc_mul" => jc(&mpc_mul(&c(&a[0]), &c(&a[1]), p, rd)),
        "mpc_div" => r(mpc_div(&c(&a[0]), &c(&a[1]), p, rd), |x| jc(&x)),
        "mpc_add_mpf" => jc(&mpc_add_mpf(&c(&a[0]), &m(&a[1]), p, rd)),
        "mpc_sub_mpf" => jc(&mpc_sub_mpf(&c(&a[0]), &m(&a[1]), p, rd)),
        "mpc_mul_mpf" => jc(&mpc_mul_mpf(&c(&a[0]), &m(&a[1]), p, rd)),
        "mpc_mul_imag_mpf" => jc(&mpc_mul_imag_mpf(&c(&a[0]), &m(&a[1]), p, rd)),
        "mpc_div_mpf" => r(mpc_div_mpf(&c(&a[0]), &m(&a[1]), p, rd), |x| jc(&x)),
        "mpc_mpf_div" => r(mpc_mpf_div(&m(&a[0]), &c(&a[1]), p, rd), |x| jc(&x)),
        "mpc_mul_int" => jc(&mpc_mul_int(&c(&a[0]), &int(&a[1]), p, rd)),
        "mpc_shift" => jc(&mpc_shift(&c(&a[0]), i(&a[1]))),
        "mpc_abs" => jm(&mpc_abs(&c(&a[0]), p, rd)),
        "mpc_arg" => jm(&mpc_arg(&c(&a[0]), p, rd)),
        "complex_int_pow" => pair_i(complex_int_pow(&int(&a[0]), &int(&a[1]), &int(&a[2]))),
        "mpc_pow_int" => r(mpc_pow_int(&c(&a[0]), &int(&a[1]), p, rd), |x| jc(&x)),
        "mpc_pow_mpf" => r(mpc_pow_mpf(&c(&a[0]), &m(&a[1]), p, rd), |x| jc(&x)),
        "mpc_pow" => r(mpc_pow(&c(&a[0]), &c(&a[1]), p, rd), |x| jc(&x)),
        "mpc_nthroot" => r(mpc_nthroot(&c(&a[0]), i(&a[1]), p, rd), |x| jc(&x)),
        "mpc_nthroot_fixed" => r(
            mpc_nthroot_fixed(&int(&a[0]), &int(&a[1]), i(&a[2]), i(&a[3])),
            pair_i,
        ),
        "mpc_cos_sin" => pair_c(mpc_cos_sin(&c(&a[0]), p, rd)),
        "mpc_cos_sin_pi" => pair_c(mpc_cos_sin_pi(&c(&a[0]), p, rd)),
        "mpc_agm" => r(hyper::mpc_agm(&c(&a[0]), &c(&a[1]), p, rd), |x| jc(&x)),
        other => panic!("no dispatch for {other}"),
    }
}

fn opt_val(v: &J) -> Option<Value> {
    if v.is_null() { None } else { Some(val(v)) }
}

fn run_ctx(f: &str, a: &[J]) -> J {
    let v0 = || val(&a[0]);
    let v1 = || val(&a[1]);
    let rv = |x: Result<Value, CtxError>| r(x, |x| jv(&x));
    if let Some(b) = Builtin::from_name(f) {
        return rv(ctx::builtin(b, &v0()));
    }
    if let Some(name) = f.strip_suffix("_kw")
        && let Some(b) = Builtin::from_name(name)
    {
        return rv(ctx::builtin_with(b, &v0(), i(&a[1]), rnd(&a[2])));
    }
    let wrapped: Option<CtxFn> = match f {
        "cot" => Some(ctx::cot),
        "sec" => Some(ctx::sec),
        "csc" => Some(ctx::csc),
        "coth" => Some(ctx::coth),
        "sech" => Some(ctx::sech),
        "csch" => Some(ctx::csch),
        "acot" => Some(ctx::acot),
        "asec" => Some(ctx::asec),
        "acsc" => Some(ctx::acsc),
        "acoth" => Some(ctx::acoth),
        "asech" => Some(ctx::asech),
        "acsch" => Some(ctx::acsch),
        "sinc" => Some(ctx::sinc),
        "sincpi" => Some(ctx::sincpi),
        "sign" => Some(ctx::sign),
        "arg" => Some(ctx::arg),
        "log10" => Some(ctx::log10),
        "degrees" => Some(ctx::degrees),
        _ => None,
    };
    if let Some(g) = wrapped {
        return rv(g(&v0()));
    }
    match f {
        "add" => jv(&ctx::add(&v0(), &v1())),
        "sub" => jv(&ctx::sub(&v0(), &v1())),
        "mul" => jv(&ctx::mul(&v0(), &v1())),
        "div" => rv(ctx::div(&v0(), &v1())),
        "pow" => rv(ctx::pow(&v0(), &v1())),
        "power" => rv(ctx::power(&v0(), &v1())),
        "fmod" => rv(ctx::fmod(&v0(), &v1())),
        "eq" => json!(ctx::eq(&v0(), &v1())),
        "lt" => r(ctx::lt(&v0(), &v1()), |b| json!(b)),
        "le" => r(ctx::le(&v0(), &v1()), |b| json!(b)),
        "gt" => r(ctx::gt(&v0(), &v1()), |b| json!(b)),
        "ge" => r(ctx::ge(&v0(), &v1()), |b| json!(b)),
        "mul_int" => jv(&ctx::mul_int(&v0(), &int(&a[1]))),
        "rdiv_int" => rv(ctx::rdiv_int(&int(&a[0]), &v1())),
        "pow_int" => rv(ctx::pow_int(&v0(), &int(&a[1]))),
        "neg" => jv(&ctx::neg(&v0())),
        "pos" => jv(&ctx::pos(&v0())),
        "abs" | "fabs" => jv(&ctx::abs(&v0())),
        "truthy" => json!(ctx::truthy(&v0())),
        "re" => jv(&ctx::re(&v0())),
        "im" => jv(&ctx::im(&v0())),
        "conj" => jv(&ctx::conj(&v0())),
        "radians" => jv(&ctx::radians(&v0())),
        "isnan" => json!(ctx::isnan(&v0())),
        "isinf" => json!(ctx::isinf(&v0())),
        "isfinite" => json!(ctx::isfinite(&v0())),
        "isnormal" => json!(ctx::isnormal(&v0())),
        "isnpint" => json!(ctx::isnpint(&v0())),
        "isint" => json!(ctx::isint(&v0(), a[1].as_bool().unwrap())),
        "mag" => jmag(ctx::mag(&v0())),
        "nint_distance" => r(ctx::nint_distance(&v0()), |(n, d)| {
            jt(vec![ji(n), d.map_or_else(|| jm(&FNINF), ji)])
        }),
        "frexp" => r(ctx::frexp(&v0()), |(y, n)| jt(vec![jv(&y), ji(n)])),
        "cos_sin" => pair_v(ctx::cos_sin(&v0())),
        "cospi_sinpi" => pair_v(ctx::cospi_sinpi(&v0())),
        "log" => rv(ctx::log(&v0(), opt_val(&a[1]).as_ref())),
        "hypot" => rv(ctx::hypot(&v0(), &v1())),
        "atan2" => rv(ctx::atan2(&v0(), &v1())),
        "agm" => rv(ctx::agm(&v0(), opt_val(&a[1]).as_ref())),
        "ldexp" => rv(ctx::ldexp(&v0(), i(&a[1]))),
        "root" => rv(ctx::root(&v0(), i(&a[1]), i(&a[2]))),
        "const" => {
            let k = match a[0]["s"].as_str().unwrap() {
                "pi" => CtxConstant::Pi,
                "e" => CtxConstant::E,
                "ln2" => CtxConstant::Ln2,
                "ln10" => CtxConstant::Ln10,
                "phi" => CtxConstant::Phi,
                "degree" => CtxConstant::Degree,
                "eps" => CtxConstant::Eps,
                other => panic!("constant {other}"),
            };
            jv(&ctx::pos(&k.value()))
        }
        "convert" => rv(ctx::convert(&num(&a[0]))),
        "mpf" => r(ctx::mpf(&num(&a[0])), |x| jm(&x)),
        "mpc" => r(ctx::mpc_new(&num(&a[0]), &num(&a[1])), |z| jc(&z)),
        "convert_param" => r(ctx::convert_param(&num(&a[0])), |x| jparam(&x)),
        "workprec" | "extraprec" | "workdps" | "extradps" => {
            let n = i(&a[0]);
            let b = Builtin::from_name(a[1]["s"].as_str().unwrap()).unwrap();
            let x = val(&a[2]);
            let g = || ctx::builtin(b, &x);
            let y = match f {
                "workprec" => ctx::workprec(n, g),
                "extraprec" => ctx::extraprec(n, g),
                "workdps" => ctx::workdps(n, g),
                _ => ctx::extradps(n, g),
            };
            let y = y.expect("these calls succeed in the generator");
            jt(vec![jv(&y), ji(ctx::prec()), ji(ctx::dps())])
        }
        "set_prec" => {
            ctx::set_prec(i(&a[0]));
            jt(vec![ji(ctx::prec()), ji(ctx::dps())])
        }
        "set_dps" => {
            ctx::set_dps(i(&a[0]));
            jt(vec![ji(ctx::prec()), ji(ctx::dps())])
        }
        "hypsum" => {
            let p = a[0].as_u64().unwrap() as usize;
            let coeffs: Vec<HypParam> = a[2].as_array().unwrap().iter().map(param).collect();
            let z = val(&a[3]);
            let o = &a[4];
            let opts = HypsumOpts {
                accurate_small: o.get("accurate_small").is_none_or(|b| b.as_bool().unwrap()),
                maxprec: o.get("maxprec").map(i),
                maxterms: o.get("maxterms").map(i),
                zeroprec: o.get("zeroprec").map(i),
            };
            rv(ctx::hypsum(p, &coeffs, &z, &opts))
        }
        other => panic!("no dispatch for ctx.{other}"),
    }
}

/// Runs one vector in the current state and encodes the result.
fn run(v: &J) -> J {
    let f = v["f"].as_str().unwrap();
    let a = v["a"].as_array().unwrap();
    let p = v.get("p").and_then(J::as_i64).unwrap_or(0);
    let rd = v.get("r").map_or(Rnd::Nearest, rnd);
    if let Some(name) = f.strip_prefix("ctx.") {
        ctx::reset_context();
        ctx::set_prec(p);
        ctx::set_rounding(rd);
        ctx::set_trap_complex(v.get("tc").is_some_and(|t| t.as_bool().unwrap()));
        let out = run_ctx(name, a);
        ctx::reset_context();
        return out;
    }
    run_raw(f, a, p, rd)
}

fn fresh() {
    clear_constant_cache();
    ctx::reset_context();
}

fn describe(v: &J) -> String {
    let s = v.to_string();
    if s.len() > 600 {
        format!("{}...", &s[..600])
    } else {
        s
    }
}

#[test]
fn mpmath_m2() {
    let data = load();
    let mut failures = Vec::new();
    let mut not_ported = 0;
    let mut total = 0;
    let mut check = |v: &J, got: J| {
        total += 1;
        if got.get("e").and_then(J::as_str) == Some("NotPortedYet") {
            not_ported += 1;
            return;
        }
        if got != v["o"] {
            failures.push(format!("{}\n   got  {}", describe(v), describe(&got)));
        }
    };
    for v in data["vectors"].as_array().unwrap() {
        fresh();
        let got = run(v);
        check(v, got);
    }
    for seq in data["sequences"].as_array().unwrap() {
        fresh();
        for v in seq.as_array().unwrap() {
            let got = run(v);
            check(v, got);
        }
    }
    fresh();
    let n = failures.len();
    for f in failures.iter().take(40) {
        eprintln!("MISMATCH {f}");
    }
    assert_eq!(n, 0, "{n} of {total} vectors differ from mpmath");
    assert_eq!(
        not_ported, NOT_PORTED,
        "NotPortedYet count changed ({total} vectors)"
    );
}
