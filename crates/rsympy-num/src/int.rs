//! Arbitrary-precision integers with Python's semantics.
//!
//! `Int` stores values that fit in an `i64` inline and falls back to
//! `dashu`'s `IBig` otherwise. The representation is canonical (a value that
//! fits in `i64` is never stored as `IBig`), so derived equality and hashing
//! are value equality and hashing.
//!
//! Division follows Python: `//` floors and `%` takes the sign of the
//! divisor. `>>` on negative numbers floors too, as in Python.

use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, BitAnd, Mul, Neg, Shl, Shr, Sub};
use std::str::FromStr;

use dashu_int::ops::{Abs, BitTest, DivRem, Gcd, SquareRootRem, UnsignedAbs};
use dashu_int::{IBig, Sign, UBig};

#[derive(Clone, PartialEq, Eq, Hash)]
enum Repr {
    Small(i64),
    Big(IBig),
}

/// An arbitrary-precision integer that behaves like Python's `int`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Int(Repr);

/// Raised where Python raises `ZeroDivisionError`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZeroDivision;

impl fmt::Display for ZeroDivision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("integer division or modulo by zero")
    }
}

impl std::error::Error for ZeroDivision {}

impl Int {
    pub const ZERO: Int = Int(Repr::Small(0));
    pub const ONE: Int = Int(Repr::Small(1));
    pub const NEG_ONE: Int = Int(Repr::Small(-1));

    pub const fn small(v: i64) -> Int {
        Int(Repr::Small(v))
    }

    fn from_ibig(v: IBig) -> Int {
        match i64::try_from(&v) {
            Ok(s) => Int(Repr::Small(s)),
            Err(_) => Int(Repr::Big(v)),
        }
    }

    /// Borrows the value as an `IBig`, converting only the inline case.
    fn big(&self) -> Cow<'_, IBig> {
        match &self.0 {
            Repr::Small(s) => Cow::Owned(IBig::from(*s)),
            Repr::Big(b) => Cow::Borrowed(b),
        }
    }

    fn to_ibig(&self) -> IBig {
        match &self.0 {
            Repr::Small(s) => IBig::from(*s),
            Repr::Big(b) => b.clone(),
        }
    }

    pub fn from_ubig(v: UBig) -> Int {
        Int::from_ibig(IBig::from(v))
    }

    /// The magnitude as a `UBig`.
    pub fn to_ubig_abs(&self) -> UBig {
        match &self.0 {
            Repr::Small(s) => UBig::from(s.unsigned_abs()),
            Repr::Big(b) => b.unsigned_abs(),
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self.0 {
            Repr::Small(s) => Some(s),
            Repr::Big(_) => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match &self.0 {
            Repr::Small(s) => u64::try_from(*s).ok(),
            Repr::Big(b) => u64::try_from(b).ok(),
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        self.as_u64().and_then(|v| usize::try_from(v).ok())
    }

    pub fn as_u32(&self) -> Option<u32> {
        self.as_u64().and_then(|v| u32::try_from(v).ok())
    }

    pub fn is_zero(&self) -> bool {
        matches!(self.0, Repr::Small(0))
    }

    pub fn is_one(&self) -> bool {
        matches!(self.0, Repr::Small(1))
    }

    pub fn is_negative(&self) -> bool {
        match &self.0 {
            Repr::Small(s) => *s < 0,
            Repr::Big(b) => b.sign() == Sign::Negative,
        }
    }

    pub fn is_positive(&self) -> bool {
        !self.is_negative() && !self.is_zero()
    }

    pub fn signum(&self) -> i32 {
        if self.is_zero() {
            0
        } else if self.is_negative() {
            -1
        } else {
            1
        }
    }

    pub fn is_even(&self) -> bool {
        match &self.0 {
            Repr::Small(s) => s & 1 == 0,
            Repr::Big(b) => !b.bit(0),
        }
    }

    pub fn is_odd(&self) -> bool {
        !self.is_even()
    }

    pub fn abs(&self) -> Int {
        match &self.0 {
            Repr::Small(s) => match s.checked_abs() {
                Some(a) => Int::small(a),
                None => Int::from_ibig(IBig::from(*s).abs()),
            },
            Repr::Big(b) => Int::from_ibig(b.abs()),
        }
    }

    /// Python's `int.bit_length()`: bits in the magnitude, 0 for 0.
    pub fn bit_length(&self) -> u64 {
        match &self.0 {
            Repr::Small(s) => 64 - u64::from(s.unsigned_abs().leading_zeros()),
            Repr::Big(b) => b.unsigned_abs().bit_len() as u64,
        }
    }

    /// Whether bit `n` of the magnitude is set.
    pub fn bit(&self, n: u64) -> bool {
        match &self.0 {
            Repr::Small(s) => n < 64 && (s.unsigned_abs() >> n) & 1 == 1,
            Repr::Big(b) => b.unsigned_abs().bit(n as usize),
        }
    }

    /// Index of the lowest set bit of the magnitude (`gmpy2.bit_scan1`), or
    /// `None` for 0.
    pub fn trailing_zeros(&self) -> Option<u64> {
        match &self.0 {
            Repr::Small(0) => None,
            Repr::Small(s) => Some(u64::from(s.trailing_zeros())),
            Repr::Big(b) => b.unsigned_abs().trailing_zeros().map(|z| z as u64),
        }
    }

    /// Python `a // b` (floor division).
    pub fn floor_div(&self, rhs: &Int) -> Result<Int, ZeroDivision> {
        Ok(self.div_mod(rhs)?.0)
    }

    /// Python `a % b` (result has the sign of `b`).
    pub fn py_mod(&self, rhs: &Int) -> Result<Int, ZeroDivision> {
        Ok(self.div_mod(rhs)?.1)
    }

    /// Python `divmod(a, b)`.
    pub fn div_mod(&self, rhs: &Int) -> Result<(Int, Int), ZeroDivision> {
        if rhs.is_zero() {
            return Err(ZeroDivision);
        }
        if let (Repr::Small(a), Repr::Small(b)) = (&self.0, &rhs.0)
            && let (Some(q), Some(r)) = (a.checked_div(*b), a.checked_rem(*b))
        {
            return Ok(if r != 0 && ((r < 0) != (*b < 0)) {
                (Int::small(q - 1), Int::small(r + b))
            } else {
                (Int::small(q), Int::small(r))
            });
        }
        let (a, b) = (self.big(), rhs.big());
        let (q, r) = a.as_ref().div_rem(b.as_ref());
        let r_neg = r.sign() == Sign::Negative;
        if r != IBig::ZERO && r_neg != (b.sign() == Sign::Negative) {
            Ok((Int::from_ibig(q - 1), Int::from_ibig(r + b.as_ref())))
        } else {
            Ok((Int::from_ibig(q), Int::from_ibig(r)))
        }
    }

    /// Division truncated toward zero, with the matching remainder (C
    /// semantics). Used where SymPy's code calls `int(a / b)`-free helpers
    /// that need it, and for exact division.
    pub fn div_rem_trunc(&self, rhs: &Int) -> Result<(Int, Int), ZeroDivision> {
        if rhs.is_zero() {
            return Err(ZeroDivision);
        }
        if let (Repr::Small(a), Repr::Small(b)) = (&self.0, &rhs.0)
            && let (Some(q), Some(r)) = (a.checked_div(*b), a.checked_rem(*b))
        {
            return Ok((Int::small(q), Int::small(r)));
        }
        let (q, r) = self.big().as_ref().div_rem(rhs.big().as_ref());
        Ok((Int::from_ibig(q), Int::from_ibig(r)))
    }

    /// `a / b` for `b` known to divide `a`.
    pub fn div_exact(&self, rhs: &Int) -> Int {
        let (q, r) = self.div_rem_trunc(rhs).expect("div_exact by zero");
        debug_assert!(r.is_zero(), "div_exact: {self} not divisible by {rhs}");
        q
    }

    /// `self ** exp`.
    pub fn pow(&self, exp: u64) -> Int {
        if let Repr::Small(b) = self.0 {
            match b {
                0 => return if exp == 0 { Int::ONE } else { Int::ZERO },
                1 => return Int::ONE,
                -1 => {
                    return if exp.is_multiple_of(2) {
                        Int::ONE
                    } else {
                        Int::NEG_ONE
                    };
                }
                _ => {
                    if let Ok(e) = u32::try_from(exp)
                        && let Some(v) = b.checked_pow(e)
                    {
                        return Int::small(v);
                    }
                }
            }
        }
        let e = usize::try_from(exp).expect("exponent too large");
        Int::from_ibig(self.to_ibig().pow(e))
    }

    /// Python's three-argument `pow(self, exp, m)` for `exp >= 0`, `m != 0`.
    /// The result has the sign of `m`, as in Python.
    pub fn pow_mod(&self, exp: &Int, m: &Int) -> Result<Int, ZeroDivision> {
        assert!(!exp.is_negative(), "pow_mod: negative exponent");
        if m.is_zero() {
            return Err(ZeroDivision);
        }
        let mabs = m.abs();
        if mabs.is_one() {
            return Ok(Int::ZERO);
        }
        let base = self.py_mod(&mabs)?;
        let r = if let (Some(b), Some(md)) = (base.as_u64(), mabs.as_u64()) {
            if md <= u64::from(u32::MAX) {
                Int::from(pow_mod_u64(b, exp, md))
            } else {
                pow_mod_big(&base, exp, &mabs)
            }
        } else {
            pow_mod_big(&base, exp, &mabs)
        };
        if m.is_negative() && !r.is_zero() {
            Ok(&r + m)
        } else {
            Ok(r)
        }
    }

    /// Non-negative gcd, as `math.gcd`.
    pub fn gcd(&self, rhs: &Int) -> Int {
        if let (Repr::Small(a), Repr::Small(b)) = (&self.0, &rhs.0) {
            let (mut a, mut b) = (a.unsigned_abs(), b.unsigned_abs());
            while b != 0 {
                (a, b) = (b, a % b);
            }
            return Int::from(a);
        }
        let (a, b) = (self.to_ubig_abs(), rhs.to_ubig_abs());
        if a == UBig::ZERO {
            return Int::from_ubig(b);
        }
        if b == UBig::ZERO {
            return Int::from_ubig(a);
        }
        Int::from_ubig(a.gcd(b))
    }

    /// Non-negative lcm, as `math.lcm`.
    pub fn lcm(&self, rhs: &Int) -> Int {
        if self.is_zero() || rhs.is_zero() {
            return Int::ZERO;
        }
        let g = self.gcd(rhs);
        &self.abs().div_exact(&g) * &rhs.abs()
    }

    /// `math.isqrt`; `None` for negative input.
    pub fn isqrt(&self) -> Option<Int> {
        self.sqrt_rem().map(|(s, _)| s)
    }

    /// `(isqrt(n), n - isqrt(n)**2)` for `n >= 0`.
    pub fn sqrt_rem(&self) -> Option<(Int, Int)> {
        if self.is_negative() {
            return None;
        }
        if let Some(v) = self.as_u64() {
            let mut s = (v as f64).sqrt() as u64;
            // Correct the float estimate.
            while s.checked_mul(s).is_none_or(|sq| sq > v) {
                s -= 1;
            }
            while (s + 1).checked_mul(s + 1).is_some_and(|sq| sq <= v) {
                s += 1;
            }
            return Some((Int::from(s), Int::from(v - s * s)));
        }
        let (s, r) = self.to_ubig_abs().sqrt_rem();
        Some((Int::from_ubig(s), Int::from_ubig(r)))
    }

    /// `floor(self ** (1/n))` for `self >= 0`, `n >= 1`.
    pub fn nth_root_floor(&self, n: u64) -> Int {
        assert!(!self.is_negative() && n >= 1);
        if n == 1 || self.is_zero() || self.is_one() {
            return self.clone();
        }
        if n >= self.bit_length() {
            return Int::ONE;
        }
        Int::from_ubig(self.to_ubig_abs().nth_root(n as usize))
    }

    /// Python's `float(int)`: round half to even, `None` on overflow
    /// (Python raises `OverflowError`).
    pub fn to_f64(&self) -> Option<f64> {
        match &self.0 {
            Repr::Small(s) => Some(i64_to_f64(*s)),
            Repr::Big(b) => {
                let bits = b.unsigned_abs().bit_len() as u64;
                if bits > 1024 {
                    return None;
                }
                // Keep 55 significant bits (53 + guard + sticky) and let the
                // hardware round the rest, exactly as CPython does.
                let shift = bits.saturating_sub(55);
                let mag = b.unsigned_abs();
                let top = &mag >> (shift as usize);
                let mut top = u64::try_from(&top).expect("55 bits fit in u64");
                if shift > 0 && mag.trailing_zeros().is_some_and(|z| (z as u64) < shift) {
                    top |= 1; // sticky bit
                }
                let v = (top as f64) * 2f64.powi(shift as i32);
                if v.is_infinite() {
                    return None;
                }
                Some(if b.sign() == Sign::Negative { -v } else { v })
            }
        }
    }

    /// Decimal digits of the magnitude.
    pub fn num_decimal_digits(&self) -> usize {
        let s = self.abs().to_string();
        s.len()
    }

    /// Python `int(s)` for decimal input: optional sign, surrounding
    /// whitespace and single underscores between digits.
    pub fn parse_python(s: &str) -> Option<Int> {
        let t = s.trim();
        let (neg, digits) = match t.as_bytes().first()? {
            b'-' => (true, &t[1..]),
            b'+' => (false, &t[1..]),
            _ => (false, t),
        };
        if digits.is_empty()
            || digits.starts_with('_')
            || digits.ends_with('_')
            || digits.contains("__")
            || !digits.bytes().all(|c| c.is_ascii_digit() || c == b'_')
        {
            return None;
        }
        let clean: String = digits.chars().filter(|&c| c != '_').collect();
        let v = if clean.len() <= 18 {
            Int::small(clean.parse::<i64>().ok()?)
        } else {
            Int::from_ibig(IBig::from_str(&clean).ok()?)
        };
        Some(if neg { -v } else { v })
    }
}

fn i64_to_f64(v: i64) -> f64 {
    // `as` rounds to nearest, ties to even, like CPython.
    v as f64
}

fn pow_mod_u64(base: u64, exp: &Int, m: u64) -> u64 {
    // m <= u32::MAX so products fit in u64.
    let mut result = 1 % m;
    let mut b = base % m;
    let nbits = exp.bit_length();
    for i in 0..nbits {
        if exp.bit(i) {
            result = result * b % m;
        }
        b = b * b % m;
    }
    result
}

fn pow_mod_big(base: &Int, exp: &Int, m: &Int) -> Int {
    if exp.bit_length() <= 8 {
        // Short exponents (squares and cubes in ECM and Lucas sequences):
        // plain square-and-multiply beats setting up a Montgomery ring.
        let mut result = Int::ONE;
        for i in (0..exp.bit_length()).rev() {
            result = (&result * &result).py_mod(m).expect("m != 0");
            if exp.bit(i) {
                result = (&result * base).py_mod(m).expect("m != 0");
            }
        }
        return result;
    }
    use dashu_int::fast_div::ConstDivisor;
    let ring = ConstDivisor::new(m.to_ubig_abs());
    let b = ring.reduce(base.to_ubig_abs());
    let e = exp.to_ubig_abs();
    Int::from_ubig(b.pow(&e).residue())
}

impl Default for Int {
    fn default() -> Self {
        Int::ZERO
    }
}

impl From<i64> for Int {
    fn from(v: i64) -> Self {
        Int::small(v)
    }
}

impl From<i32> for Int {
    fn from(v: i32) -> Self {
        Int::small(i64::from(v))
    }
}

impl From<u32> for Int {
    fn from(v: u32) -> Self {
        Int::small(i64::from(v))
    }
}

impl From<u64> for Int {
    fn from(v: u64) -> Self {
        match i64::try_from(v) {
            Ok(s) => Int::small(s),
            Err(_) => Int(Repr::Big(IBig::from(v))),
        }
    }
}

impl From<usize> for Int {
    fn from(v: usize) -> Self {
        Int::from(v as u64)
    }
}

impl From<i128> for Int {
    fn from(v: i128) -> Self {
        match i64::try_from(v) {
            Ok(s) => Int::small(s),
            Err(_) => Int(Repr::Big(IBig::from(v))),
        }
    }
}

impl From<IBig> for Int {
    fn from(v: IBig) -> Self {
        Int::from_ibig(v)
    }
}

impl From<&Int> for IBig {
    fn from(v: &Int) -> Self {
        v.to_ibig()
    }
}

impl FromStr for Int {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        Int::parse_python(s).ok_or(())
    }
}

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Repr::Small(s) => write!(f, "{s}"),
            Repr::Big(b) => write!(f, "{b}"),
        }
    }
}

impl fmt::Debug for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Ord for Int {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.0, &other.0) {
            (Repr::Small(a), Repr::Small(b)) => a.cmp(b),
            (Repr::Big(a), Repr::Big(b)) => a.cmp(b),
            // A Big is always outside i64's range.
            (Repr::Small(_), Repr::Big(b)) => {
                if b.sign() == Sign::Negative {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (Repr::Big(a), Repr::Small(_)) => {
                if a.sign() == Sign::Negative {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
        }
    }
}

impl PartialOrd for Int {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq<i64> for Int {
    fn eq(&self, other: &i64) -> bool {
        matches!(self.0, Repr::Small(s) if s == *other)
    }
}

impl PartialOrd<i64> for Int {
    fn partial_cmp(&self, other: &i64) -> Option<Ordering> {
        Some(self.cmp(&Int::small(*other)))
    }
}

macro_rules! binop {
    ($trait:ident, $method:ident, $checked:ident, $big:expr) => {
        impl $trait<&Int> for &Int {
            type Output = Int;
            fn $method(self, rhs: &Int) -> Int {
                if let (Repr::Small(a), Repr::Small(b)) = (&self.0, &rhs.0) {
                    if let Some(v) = a.$checked(*b) {
                        return Int::small(v);
                    }
                }
                let f: fn(&IBig, &IBig) -> IBig = $big;
                Int::from_ibig(f(&self.big(), &rhs.big()))
            }
        }
        impl $trait<Int> for Int {
            type Output = Int;
            fn $method(self, rhs: Int) -> Int {
                (&self).$method(&rhs)
            }
        }
        impl $trait<&Int> for Int {
            type Output = Int;
            fn $method(self, rhs: &Int) -> Int {
                (&self).$method(rhs)
            }
        }
        impl $trait<Int> for &Int {
            type Output = Int;
            fn $method(self, rhs: Int) -> Int {
                self.$method(&rhs)
            }
        }
        impl $trait<i64> for &Int {
            type Output = Int;
            fn $method(self, rhs: i64) -> Int {
                self.$method(&Int::small(rhs))
            }
        }
        impl $trait<i64> for Int {
            type Output = Int;
            fn $method(self, rhs: i64) -> Int {
                (&self).$method(&Int::small(rhs))
            }
        }
    };
}

binop!(Add, add, checked_add, |a, b| a + b);
binop!(Sub, sub, checked_sub, |a, b| a - b);
binop!(Mul, mul, checked_mul, |a, b| a * b);

impl Neg for &Int {
    type Output = Int;
    fn neg(self) -> Int {
        match &self.0 {
            Repr::Small(s) => match s.checked_neg() {
                Some(v) => Int::small(v),
                None => Int::from_ibig(-IBig::from(*s)),
            },
            Repr::Big(b) => Int::from_ibig(-b.clone()),
        }
    }
}

impl Neg for Int {
    type Output = Int;
    fn neg(self) -> Int {
        -&self
    }
}

impl Shl<u64> for &Int {
    type Output = Int;
    fn shl(self, n: u64) -> Int {
        if let Repr::Small(s) = self.0
            && n < 63
            && (s.unsigned_abs() >> (63 - n)) == 0
        {
            return Int::small(s << n);
        }
        Int::from_ibig(self.to_ibig() << (n as usize))
    }
}

impl Shl<u64> for Int {
    type Output = Int;
    fn shl(self, n: u64) -> Int {
        &self << n
    }
}

impl Shr<u64> for &Int {
    type Output = Int;
    /// Arithmetic (flooring) shift, as Python's `>>`.
    fn shr(self, n: u64) -> Int {
        match &self.0 {
            Repr::Small(s) => Int::small(if n >= 64 {
                if *s < 0 { -1 } else { 0 }
            } else {
                s >> n
            }),
            Repr::Big(b) => Int::from_ibig(b >> (n as usize)),
        }
    }
}

impl Shr<u64> for Int {
    type Output = Int;
    fn shr(self, n: u64) -> Int {
        &self >> n
    }
}

impl BitAnd<&Int> for &Int {
    type Output = Int;
    /// Two's-complement `&`, as Python's.
    fn bitand(self, rhs: &Int) -> Int {
        if let (Repr::Small(a), Repr::Small(b)) = (&self.0, &rhs.0) {
            return Int::small(a & b);
        }
        Int::from_ibig(self.big().as_ref() & rhs.big().as_ref())
    }
}

impl std::iter::Sum for Int {
    fn sum<I: Iterator<Item = Int>>(iter: I) -> Int {
        iter.fold(Int::ZERO, |a, b| a + b)
    }
}

impl std::iter::Product for Int {
    fn product<I: Iterator<Item = Int>>(iter: I) -> Int {
        iter.fold(Int::ONE, |a, b| a * b)
    }
}
