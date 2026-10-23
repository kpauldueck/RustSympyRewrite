//! Port of `sympy.external.ntheory`: the pure-Python fallbacks SymPy uses
//! for gmpy2's number-theory primitives (SymPy's Python ground types).

use rsympy_num::Int;

/// Errors these primitives raise in SymPy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NtError {
    /// Python `ValueError`.
    Value(String),
    /// Python `ZeroDivisionError`.
    ZeroDivision(String),
}

impl NtError {
    /// The Python exception class SymPy raises.
    pub fn class(&self) -> &'static str {
        match self {
            NtError::Value(_) => "ValueError",
            NtError::ZeroDivision(_) => "ZeroDivisionError",
        }
    }
}

impl std::fmt::Display for NtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NtError::Value(m) | NtError::ZeroDivision(m) => write!(f, "{}: {m}", self.class()),
        }
    }
}

impl std::error::Error for NtError {}

pub type NtResult<T> = Result<T, NtError>;

pub(crate) fn value_err<T>(msg: impl Into<String>) -> NtResult<T> {
    Err(NtError::Value(msg.into()))
}

/// `x % m` for `m > 0`, known not to be zero.
pub(crate) fn md(x: &Int, m: &Int) -> Int {
    x.py_mod(m).expect("modulus is nonzero")
}

/// `bit_scan1(x)`: index of the lowest set bit of `|x|`; `None` for 0.
pub fn bit_scan1(x: &Int) -> Option<u64> {
    x.trailing_zeros()
}

/// `remove(x, f)`: divides out every factor `f` of `x`; returns the cofactor
/// and the multiplicity.
pub fn remove(x: &Int, f: &Int) -> NtResult<(Int, u64)> {
    if *f < 2 {
        return value_err("factor must be > 1");
    }
    if x.is_zero() {
        return Ok((Int::ZERO, 0));
    }
    if *f == 2 {
        let b = bit_scan1(x).expect("x != 0");
        return Ok((x >> b, b));
    }
    let mut x = x.clone();
    let mut m: u64 = 0;
    let (mut y, mut rem) = x.div_mod(f).expect("f > 1");
    while rem.is_zero() {
        x = y;
        m += 1;
        if m > 5 {
            let mut pow_list = vec![f * f];
            while let Some(last) = pow_list.last() {
                let (yy, r) = x.div_mod(last).expect("f > 1");
                if r.is_zero() {
                    m += 1 << pow_list.len();
                    x = yy;
                    let sq = last * last;
                    pow_list.push(sq);
                } else {
                    pow_list.pop();
                }
            }
        }
        (y, rem) = x.div_mod(f).expect("f > 1");
    }
    Ok((x, m))
}

/// Integer square root (`mlib.isqrt`), for `x >= 0`.
pub fn isqrt(x: &Int) -> Int {
    x.isqrt().expect("isqrt of a negative number")
}

/// `sqrtrem(x)` for `x >= 0`.
pub fn sqrtrem(x: &Int) -> (Int, Int) {
    x.sqrt_rem().expect("sqrtrem of a negative number")
}

/// `is_square(x)`.
pub fn is_square(x: &Int) -> bool {
    if x.is_negative() {
        return false;
    }
    let low = (x & &Int::from(127u32)).as_u64().expect("7 bits") as u32;
    if (0xfdfd_fded_fdfd_fdec_fdfd_fded_fdfc_fdecu128 >> low) & 1 == 1 {
        return false;
    }
    let m = md(x, &Int::from(765_765u32)).as_u64().expect("small");
    if (0x0005_f6f9_ffb6_fb7d_dfcb_75be_fdec_u128 >> (m % 99)) & 1 == 1 {
        return false;
    }
    if (0x0000_06fd_1bfc_fed5_f367_9d3e_bdec_u128 >> (m % 91)) & 1 == 1 {
        return false;
    }
    if (0x000d_ef9a_e771_ffe3_b9d6_7dec_u128 >> (m % 85)) & 1 == 1 {
        return false;
    }
    sqrtrem(x).1.is_zero()
}

/// `gcdext(a, b)`: `(g, s, t)` with `g = s*a + t*b`, following SymPy's
/// extended Euclid exactly (the cofactors are not unique).
pub fn gcdext(a: &Int, b: &Int) -> (Int, Int, Int) {
    if a.is_zero() || b.is_zero() {
        let g = if a.is_zero() { b.abs() } else { a.abs() };
        if g.is_zero() {
            return (Int::ZERO, Int::ZERO, Int::ZERO);
        }
        return (
            g.clone(),
            a.floor_div(&g).expect("g != 0"),
            b.floor_div(&g).expect("g != 0"),
        );
    }
    let (x_sign, mut a) = if a.is_negative() {
        (-1, -a)
    } else {
        (1, a.clone())
    };
    let (y_sign, mut b) = if b.is_negative() {
        (-1, -b)
    } else {
        (1, b.clone())
    };
    let (mut x, mut r) = (Int::ONE, Int::ZERO);
    let (mut y, mut s) = (Int::ZERO, Int::ONE);
    while !b.is_zero() {
        let (q, c) = a.div_mod(&b).expect("b != 0");
        a = std::mem::replace(&mut b, c);
        let nr = &x - &(&q * &r);
        x = std::mem::replace(&mut r, nr);
        let ns = &y - &(&q * &s);
        y = std::mem::replace(&mut s, ns);
    }
    (a, x * x_sign, y * y_sign)
}

/// `invert(x, m)`: the inverse of `x` modulo `m` (Python's `pow(x, -1, m)`).
pub fn invert(x: &Int, m: &Int) -> NtResult<Int> {
    if m.is_zero() {
        // Python's pow raises ValueError here, which invert() turns into
        // ZeroDivisionError.
        return Err(NtError::ZeroDivision("invert() no inverse exists".into()));
    }
    let mabs = m.abs();
    if mabs.is_one() {
        return Ok(Int::ZERO);
    }
    let (g, s, _) = gcdext(&md(x, &mabs), &mabs);
    if !g.is_one() {
        return Err(NtError::ZeroDivision("invert() no inverse exists".into()));
    }
    // Python's pow(x, -1, m) has the sign of m.
    Ok(s.py_mod(m).expect("m != 0"))
}

/// `jacobi(x, y)` for odd `y > 0`.
pub fn jacobi(x: &Int, y: &Int) -> NtResult<i32> {
    if !y.is_positive() || y.is_even() {
        return value_err("y should be an odd positive integer");
    }
    let mut x = md(x, y);
    let mut y = y.clone();
    if x.is_zero() {
        return Ok(i32::from(y.is_one()));
    }
    if y.is_one() || x.is_one() {
        return Ok(1);
    }
    if !x.gcd(&y).is_one() {
        return Ok(0);
    }
    let mut j = 1;
    let eight = Int::from(8);
    let four = Int::from(4);
    while !x.is_zero() {
        while x.is_even() && x.is_positive() {
            x = &x >> 1;
            let r = md(&y, &eight);
            if r == 3 || r == 5 {
                j = -j;
            }
        }
        std::mem::swap(&mut x, &mut y);
        if md(&x, &four) == 3 && md(&y, &four) == 3 {
            j = -j;
        }
        x = md(&x, &y);
    }
    Ok(j)
}

/// `iroot(y, n)`: `(floor(y ** (1/n)), exact)`.
pub fn iroot(y: &Int, n: u64) -> NtResult<(Int, bool)> {
    if y.is_negative() {
        return value_err("y must be nonnegative");
    }
    if n < 1 {
        return value_err("n must be positive");
    }
    if y.is_zero() || y.is_one() || n == 1 {
        return Ok((y.clone(), true));
    }
    if n == 2 {
        let (x, rem) = sqrtrem(y);
        return Ok((x, rem.is_zero()));
    }
    if n >= y.bit_length() {
        return Ok((Int::ONE, false));
    }
    // floor root and exactness are unique, so any exact method reproduces
    // SymPy's answer.
    let x = y.nth_root_floor(n);
    let exact = &x.pow(n) == y;
    Ok((x, exact))
}

pub(crate) fn is_strong_prp_raw(n: &Int, a: &Int) -> bool {
    let nm1 = n - 1;
    let s = bit_scan1(&nm1).expect("n > 1");
    let mut a = a.pow_mod(&(n >> s), n).expect("n > 0");
    if a.is_one() || a == nm1 {
        return true;
    }
    let two = Int::from(2);
    for _ in 0..s.saturating_sub(1) {
        a = a.pow_mod(&two, n).expect("n > 0");
        if a == nm1 {
            return true;
        }
        if a.is_one() {
            return false;
        }
    }
    false
}

/// `is_strong_prp(n, a)`.
pub fn is_strong_prp(n: &Int, a: &Int) -> NtResult<bool> {
    if *a < 2 {
        return value_err("is_strong_prp() requires 'a' greater than or equal to 2");
    }
    if *n < 1 {
        return value_err("is_strong_prp() requires 'n' be greater than 0");
    }
    if n.is_one() {
        return Ok(false);
    }
    if n.is_even() {
        return Ok(*n == 2);
    }
    let a = md(a, n);
    if !n.gcd(&a).is_one() {
        return value_err("is_strong_prp() requires gcd(n,a) == 1");
    }
    Ok(is_strong_prp_raw(n, &a))
}

/// Binary digits of `k` after the leading one (`bin(k)[3:]`).
fn bits_after_top(k: &Int) -> impl Iterator<Item = bool> + '_ {
    let len = k.bit_length();
    (0..len.saturating_sub(1)).rev().map(move |i| k.bit(i))
}

/// `_lucas_sequence(n, P, Q, k)`: `(U_k mod n, V_k mod n, Q^k mod n)`.
pub fn lucas_sequence(n: &Int, p: &Int, q: &Int, k: &Int) -> (Int, Int, Int) {
    if k.is_zero() {
        return (Int::ZERO, Int::from(2), Int::ONE);
    }
    let d = &(p * p) - &(q * 4);
    let mut u = Int::ONE;
    let mut v = p.clone();
    let mut qk = md(q, n);
    let two = Int::from(2);
    if q.is_one() {
        for b in bits_after_top(k) {
            u = md(&(&u * &v), n);
            v = md(&(&(&v * &v) - &two), n);
            if b {
                let nu = &(&u * p) + &v;
                let nv = &(&v * p) + &(&u * &d);
                u = nu;
                v = nv;
                if u.is_odd() {
                    u = &u + n;
                }
                if v.is_odd() {
                    v = &v + n;
                }
                u = &u >> 1;
                v = &v >> 1;
            }
        }
    } else if p.is_one() && *q == -1 {
        for b in bits_after_top(k) {
            u = md(&(&u * &v), n);
            if qk.is_one() {
                v = md(&(&(&v * &v) - &two), n);
            } else {
                v = md(&(&(&v * &v) + &two), n);
                qk = Int::ONE;
            }
            if b {
                let nu = &u + &v;
                let nv = &u << 1;
                u = nu;
                v = nv;
                if u.is_odd() {
                    u = &u + n;
                }
                u = &u >> 1;
                v = &v + &u;
                qk = Int::NEG_ONE;
            }
        }
        qk = md(&qk, n);
    } else if p.is_one() {
        for b in bits_after_top(k) {
            u = md(&(&u * &v), n);
            v = md(&(&(&v * &v) - &(&qk * 2)), n);
            qk = &qk * &qk;
            if b {
                let nu = &u + &v;
                let nv = &(q * &u) << 1;
                u = nu;
                v = nv;
                if u.is_odd() {
                    u = &u + n;
                }
                u = &u >> 1;
                v = &u - &v;
                qk = &qk * q;
            }
            qk = md(&qk, n);
        }
    } else {
        for b in bits_after_top(k) {
            u = md(&(&u * &v), n);
            v = md(&(&(&v * &v) - &(&qk * 2)), n);
            qk = &qk * &qk;
            if b {
                let nu = &(&u * p) + &v;
                let nv = &(&v * p) + &(&u * &d);
                u = nu;
                v = nv;
                if u.is_odd() {
                    u = &u + n;
                }
                if v.is_odd() {
                    v = &v + n;
                }
                u = &u >> 1;
                v = &v >> 1;
                qk = &qk * q;
            }
            qk = md(&qk, n);
        }
    }
    (md(&u, n), md(&v, n), qk)
}

fn is_strong_selfridge_prp_raw(n: &Int) -> NtResult<bool> {
    let mut dd: i64 = 5;
    while dd < 1_000_000 {
        let d = if dd & 2 != 0 { -dd } else { dd };
        let di = Int::from(d);
        let j = jacobi(&di, n)?;
        if j == -1 {
            let np1 = n + 1;
            let s = bit_scan1(&np1).expect("n + 1 > 0");
            let q = Int::from((1 - d).div_euclid(4));
            let (u, mut v, mut qk) = lucas_sequence(n, &Int::ONE, &q, &(&np1 >> s));
            if u.is_zero() || v.is_zero() {
                return Ok(true);
            }
            let two = Int::from(2);
            for _ in 0..s.saturating_sub(1) {
                v = md(&(&(&v * &v) - &(&qk * 2)), n);
                if v.is_zero() {
                    return Ok(true);
                }
                qk = qk.pow_mod(&two, n).expect("n > 0");
            }
            return Ok(false);
        }
        if j == 0 && !md(&di, n).is_zero() {
            return Ok(false);
        }
        if d == 13 && is_square(n) {
            return Ok(false);
        }
        dd += 2;
    }
    value_err("appropriate value for D cannot be found in is_strong_selfridge_prp()")
}

/// `is_strong_bpsw_prp(n)`.
pub fn is_strong_bpsw_prp(n: &Int) -> NtResult<bool> {
    if *n < 1 {
        return value_err("is_strong_bpsw_prp() requires 'n' be greater than 0");
    }
    if n.is_one() {
        return Ok(false);
    }
    if n.is_even() {
        return Ok(*n == 2);
    }
    Ok(is_strong_prp_raw(n, &Int::from(2)) && is_strong_selfridge_prp_raw(n)?)
}
