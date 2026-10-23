//! Exact rationals, as SymPy's `PythonMPQ` and `Rational` hold them: a
//! reduced fraction with a positive denominator.

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

use crate::int::{Int, ZeroDivision};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Rational {
    num: Int,
    den: Int,
}

impl Rational {
    pub const ZERO: Rational = Rational {
        num: Int::ZERO,
        den: Int::ONE,
    };
    pub const ONE: Rational = Rational {
        num: Int::ONE,
        den: Int::ONE,
    };

    /// `num/den` in lowest terms.
    pub fn new(num: Int, den: Int) -> Result<Rational, ZeroDivision> {
        if den.is_zero() {
            return Err(ZeroDivision);
        }
        let g = num.gcd(&den);
        let (mut n, mut d) = if g.is_one() {
            (num, den)
        } else {
            (num.div_exact(&g), den.div_exact(&g))
        };
        if d.is_negative() {
            n = -n;
            d = -d;
        }
        Ok(Rational { num: n, den: d })
    }

    /// Builds from parts already in lowest terms with `den > 0`.
    pub fn from_reduced(num: Int, den: Int) -> Rational {
        debug_assert!(den.is_positive() && num.gcd(&den).is_one());
        Rational { num, den }
    }

    pub fn from_int(n: Int) -> Rational {
        Rational {
            num: n,
            den: Int::ONE,
        }
    }

    pub fn numer(&self) -> &Int {
        &self.num
    }

    pub fn denom(&self) -> &Int {
        &self.den
    }

    pub fn into_parts(self) -> (Int, Int) {
        (self.num, self.den)
    }

    pub fn is_integer(&self) -> bool {
        self.den.is_one()
    }

    pub fn is_zero(&self) -> bool {
        self.num.is_zero()
    }

    pub fn is_negative(&self) -> bool {
        self.num.is_negative()
    }

    pub fn signum(&self) -> i32 {
        self.num.signum()
    }

    pub fn abs(&self) -> Rational {
        Rational {
            num: self.num.abs(),
            den: self.den.clone(),
        }
    }

    pub fn recip(&self) -> Result<Rational, ZeroDivision> {
        Rational::new(self.den.clone(), self.num.clone())
    }

    pub fn div(&self, rhs: &Rational) -> Result<Rational, ZeroDivision> {
        if rhs.is_zero() {
            return Err(ZeroDivision);
        }
        Rational::new(&self.num * &rhs.den, &self.den * &rhs.num)
    }

    /// `self ** e` for an integer exponent.
    pub fn pow(&self, e: i64) -> Result<Rational, ZeroDivision> {
        let u = e.unsigned_abs();
        let r = Rational {
            num: self.num.pow(u),
            den: self.den.pow(u),
        };
        if e < 0 { r.recip() } else { Ok(r) }
    }

    /// Python `math.floor`.
    pub fn floor(&self) -> Int {
        self.num.floor_div(&self.den).expect("den > 0")
    }

    /// Python `math.ceil`.
    pub fn ceil(&self) -> Int {
        -((-&self.num).floor_div(&self.den).expect("den > 0"))
    }

    /// `float(Fraction)`: correctly rounded num/den.
    pub fn to_f64(&self) -> f64 {
        if let (Some(n), Some(d)) = (self.num.as_i64(), self.den.as_i64()) {
            // Exact when both fit in 53 bits.
            if n.unsigned_abs() < 1 << 53 && d < 1 << 53 {
                return n as f64 / d as f64;
            }
        }
        // CPython's int.__truediv__ algorithm: scale to get 55 bits of
        // quotient, then round once.
        let a = self.num.abs();
        let b = &self.den;
        let shift = b.bit_length() as i64 - a.bit_length() as i64 + 55;
        let (a2, b2) = if shift >= 0 {
            (&a << shift as u64, b.clone())
        } else {
            (a.clone(), b << (-shift) as u64)
        };
        let (q, r) = a2.div_mod(&b2).expect("den > 0");
        let mut q = q;
        if !r.is_zero() {
            q = &(&q << 1) + 1; // sticky
            let v = q.to_f64().unwrap_or(f64::INFINITY);
            let v = scale(v, -(shift + 1));
            return if self.num.is_negative() { -v } else { v };
        }
        let v = scale(q.to_f64().unwrap_or(f64::INFINITY), -shift);
        if self.num.is_negative() { -v } else { v }
    }
}

fn scale(v: f64, e: i64) -> f64 {
    // ldexp; split to avoid overflowing powi's intermediate.
    let mut v = v;
    let mut e = e;
    while e > 1000 {
        v *= 2f64.powi(1000);
        e -= 1000;
    }
    while e < -1000 {
        v *= 2f64.powi(-1000);
        e += 1000;
    }
    v * 2f64.powi(e as i32)
}

impl From<Int> for Rational {
    fn from(n: Int) -> Self {
        Rational::from_int(n)
    }
}

impl From<i64> for Rational {
    fn from(n: i64) -> Self {
        Rational::from_int(Int::from(n))
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.den == other.den {
            return self.num.cmp(&other.num);
        }
        (&self.num * &other.den).cmp(&(&other.num * &self.den))
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add<&Rational> for &Rational {
    type Output = Rational;
    fn add(self, rhs: &Rational) -> Rational {
        if self.den == rhs.den {
            return Rational::new(&self.num + &rhs.num, self.den.clone()).expect("den > 0");
        }
        Rational::new(
            &self.num * &rhs.den + &rhs.num * &self.den,
            &self.den * &rhs.den,
        )
        .expect("den > 0")
    }
}

impl Sub<&Rational> for &Rational {
    type Output = Rational;
    fn sub(self, rhs: &Rational) -> Rational {
        self + &(-rhs)
    }
}

impl Mul<&Rational> for &Rational {
    type Output = Rational;
    fn mul(self, rhs: &Rational) -> Rational {
        Rational::new(&self.num * &rhs.num, &self.den * &rhs.den).expect("den > 0")
    }
}

impl Neg for &Rational {
    type Output = Rational;
    fn neg(self) -> Rational {
        Rational {
            num: -&self.num,
            den: self.den.clone(),
        }
    }
}

impl Neg for Rational {
    type Output = Rational;
    fn neg(self) -> Rational {
        -&self
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den.is_one() {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

impl fmt::Debug for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
