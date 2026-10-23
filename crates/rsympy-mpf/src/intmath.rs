//! Port of the parts of `mpmath.libmp.libintmath` that libmpf and the
//! constants use, with mpmath's pure-Python backend (the one SymPy's pinned
//! environment runs).

use rsympy_num::Int;

/// `bitcount(n)`: bits in `|n|`.
pub fn bitcount(n: &Int) -> i64 {
    n.bit_length() as i64
}

/// `trailing(n)`: trailing zero bits of `n` (0 for 0).
pub fn trailing(n: &Int) -> i64 {
    n.trailing_zeros().unwrap_or(0) as i64
}

/// `rshift(x, n)`: `x >> n`, or `x << -n` for negative `n`.
pub fn rshift(x: &Int, n: i64) -> Int {
    if n >= 0 {
        x >> n as u64
    } else {
        x << n.unsigned_abs()
    }
}

/// `lshift(x, n)`: `x << n`, or `x >> -n` for negative `n`.
pub fn lshift(x: &Int, n: i64) -> Int {
    if n >= 0 {
        x << n as u64
    } else {
        x >> n.unsigned_abs()
    }
}

/// `giant_steps(start, target, n)`.
pub fn giant_steps(start: i64, target: i64, n: i64) -> Vec<i64> {
    let mut l = vec![target];
    while *l.last().expect("non-empty") > start * n {
        let last = *l.last().expect("non-empty");
        l.push(last.div_euclid(n) + 2);
    }
    l.reverse();
    l
}

/// `bin_to_radix(x, xbits, base, bdigits)`: `floor(x * base**bdigits / 2**xbits)`.
pub fn bin_to_radix(x: &Int, xbits: i64, base: u32, bdigits: i64) -> Int {
    let p = Int::from(base).pow(bdigits as u64);
    &(x * &p) >> xbits as u64
}

/// `numeral(n, base=10, size)`: decimal digits of `n`, split recursively the
/// way mpmath's Python backend does (which can matter for the zero padding
/// when `size` overestimates the length).
pub fn numeral(n: &Int, size: i64) -> String {
    if !n.is_positive() {
        if n.is_zero() {
            return "0".into();
        }
        return format!("-{}", numeral(&-n, size));
    }
    if size < 250 {
        return n.to_string();
    }
    let half = size / 2 + (size & 1);
    let (a, b) = n
        .div_mod(&Int::from(10).pow(half as u64))
        .expect("10**half > 0");
    let ad = numeral(&a, half);
    let bd = numeral(&b, half);
    format!("{ad}{bd:0>width$}", width = half as usize)
}

/// Python's `int ** 0.5` (and other float powers) for a non-negative int.
fn int_powf(x: &Int, e: f64) -> f64 {
    x.to_f64().expect("small enough for a float").powf(e)
}

/// `isqrt_small_python(x)`: exact floor square root.
pub fn isqrt_small(x: &Int) -> Int {
    if x.is_zero() {
        return Int::ZERO;
    }
    let one = Int::ONE;
    let mut r = if *x < (&one << 800) {
        if *x < (&one << 50) {
            return Int::from_f64_trunc(int_powf(x, 0.5)).expect("finite");
        }
        &Int::from_f64_trunc(int_powf(x, 0.5) * 1.00000000000001).expect("finite") + 1
    } else {
        let bc = bitcount(x);
        let n = bc / 2;
        let t = x >> (2 * n - 100) as u64;
        &Int::from_f64_trunc(int_powf(&t, 0.5) + 2.0).expect("finite") << (n - 50) as u64
    };
    loop {
        let y = &(&r + &x.floor_div(&r).expect("r > 0")) >> 1;
        if y >= r {
            return r;
        }
        r = y;
    }
}

/// `isqrt_fast_python(x)`: Newton's method without division for large `x`.
/// Usually `floor(sqrt(x))`, sometimes 1 too small; mpmath's constants
/// depend on exactly which, so this is ported step for step.
pub fn isqrt_fast(x: &Int) -> Int {
    let one = Int::ONE;
    if *x < (&one << 800) {
        let mut y = Int::from_f64_trunc(int_powf(x, 0.5)).expect("finite");
        if *x >= (&one << 100) {
            y = &(&y + &x.floor_div(&y).expect("y > 0")) >> 1;
            if *x >= (&one << 200) {
                y = &(&y + &x.floor_div(&y).expect("y > 0")) >> 1;
                if *x >= (&one << 400) {
                    y = &(&y + &x.floor_div(&y).expect("y > 0")) >> 1;
                }
            }
        }
        return y;
    }
    let mut bc = bitcount(x);
    let guard_bits = 10;
    let x = x << (2 * guard_bits) as u64;
    bc += 2 * guard_bits;
    bc += bc & 1;
    let hbc = bc / 2;
    let startprec = 50.min(hbc);
    let top = &x >> (bc - 2 * startprec) as u64;
    let mut r = Int::from_f64_trunc(2f64.powf((2 * startprec) as f64) * int_powf(&top, -0.5))
        .expect("finite");
    let mut pp = startprec;
    let mut p = startprec;
    for step in giant_steps(startprec, hbc, 2) {
        p = step;
        let r2 = rshift(&(&r * &r), 2 * pp - p);
        let xr2 = &(&rshift(&x, bc - p) * &r2) >> p as u64;
        r = rshift(&(&r * &(&(&Int::from(3) << p as u64) - &xr2)), pp + 1);
        pp = p;
    }
    rshift(&(&r * &(&x >> hbc as u64)), p + guard_bits)
}

/// `sqrtrem_python(x)`: exact `(floor(sqrt(x)), remainder)`.
pub fn sqrtrem(x: &Int) -> (Int, Int) {
    x.sqrt_rem().expect("sqrtrem of a negative number")
}

/// `isqrt(x)`: exact floor square root.
pub fn isqrt(x: &Int) -> Int {
    sqrtrem(x).0
}

/// `sqrt_fixed(x, prec)`.
pub fn sqrt_fixed(x: &Int, prec: i64) -> Int {
    isqrt_fast(&(x << prec as u64))
}

/// `ifac(n)`: n!.
pub fn ifac(n: u64) -> Int {
    (2..=n).fold(Int::ONE, |acc, k| &acc * &Int::from(k))
}
