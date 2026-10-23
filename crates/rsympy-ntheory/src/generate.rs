//! Port of the parts of `sympy.ntheory.generate` that the factoring code
//! uses: the prime sieve, `primerange` and `nextprime`.
//!
//! SymPy's global `sieve` grows as it is used. How far it has grown never
//! changes an answer (it only decides how primes are found), so here it is a
//! per-thread cache.

use std::cell::RefCell;

use rsympy_num::Int;

use crate::external::{NtResult, value_err};
use crate::primetest::isprime;

/// Primes below this are served from the sieve; above it, by `nextprime`.
const SIEVE_CAP: u64 = 1 << 28;

thread_local! {
    static SIEVE: RefCell<Vec<u64>> = RefCell::new(vec![2, 3, 5, 7, 11, 13]);
}

/// Grows the sieve to cover every prime `<= n` (capped).
fn extend(n: u64) {
    let n = n.min(SIEVE_CAP);
    SIEVE.with(|s| {
        let mut list = s.borrow_mut();
        let last = *list.last().expect("sieve is never empty");
        if n <= last {
            return;
        }
        // Double at least, so repeated small extensions stay cheap.
        let hi = n.max(last.saturating_mul(2)).min(SIEVE_CAP);
        let lo = last + 1;
        let base: Vec<u64> = {
            let r = (hi as f64).sqrt() as u64 + 2;
            simple_sieve(r)
        };
        let len = (hi - lo + 1) as usize;
        let mut composite = vec![false; len];
        for &p in &base {
            if p * p > hi {
                break;
            }
            let mut start = (lo.div_ceil(p) * p).max(p * p);
            while start <= hi {
                composite[(start - lo) as usize] = true;
                start += p;
            }
        }
        for (i, &c) in composite.iter().enumerate() {
            if !c {
                list.push(lo + i as u64);
            }
        }
    });
}

fn simple_sieve(n: u64) -> Vec<u64> {
    let n = n as usize;
    let mut is = vec![true; n + 1];
    let mut out = Vec::new();
    for i in 2..=n {
        if is[i] {
            out.push(i as u64);
            let mut j = i * i;
            while j <= n {
                is[j] = false;
                j += i;
            }
        }
    }
    out
}

/// The largest prime currently in the sieve (`sieve._list[-1]`).
pub(crate) fn sieve_last() -> u64 {
    SIEVE.with(|s| *s.borrow().last().expect("sieve is never empty"))
}

/// Whether `n` is in the sieve, for `n <= sieve_last()`.
pub(crate) fn sieve_contains(n: u64) -> bool {
    SIEVE.with(|s| s.borrow().binary_search(&n).is_ok())
}

/// Primes `p` with `a <= p < b`, in increasing order (`primerange(a, b)`).
pub fn primerange(a: &Int, b: &Int) -> Vec<Int> {
    let mut out = Vec::new();
    let a = if *a < 2 { Int::from(2) } else { a.clone() };
    if a >= *b {
        return out;
    }
    let cap = Int::from(SIEVE_CAP);
    let sieve_hi = if *b > cap {
        SIEVE_CAP
    } else {
        b.as_u64().expect("b <= cap")
    };
    if a < cap {
        let lo = a.as_u64().expect("a < cap");
        extend(sieve_hi);
        SIEVE.with(|s| {
            let list = s.borrow();
            let start = list.partition_point(|&p| p < lo);
            let end = list.partition_point(|&p| p < sieve_hi);
            out.extend(list[start..end].iter().map(|&p| Int::from(p)));
        });
    }
    if *b > cap {
        let mut p = if a > cap { &a - 1 } else { &cap - 1 };
        loop {
            p = nextprime(&p);
            if p >= *b {
                break;
            }
            out.push(p.clone());
        }
    }
    out
}

/// The smallest prime greater than `n` (`nextprime(n)`).
pub fn nextprime(n: &Int) -> Int {
    nextprime_ith(n, 1).expect("ith = 1 is valid")
}

/// `nextprime(n, ith)`: the `ith` prime greater than `n`.
pub fn nextprime_ith(n: &Int, ith: u64) -> NtResult<Int> {
    if ith == 0 {
        return value_err("ith should be positive");
    }
    let mut i = ith;
    let mut n = n.clone();
    if n < 2 {
        n = Int::from(2);
        i -= 1;
        if i == 0 {
            return Ok(n);
        }
    }
    // Primes are found the same way whatever the sieve holds; walk the
    // 6k +- 1 candidates.
    if let Some(v) = n.as_u64()
        && v < sieve_last()
    {
        let found = SIEVE.with(|s| {
            let list = s.borrow();
            let idx = list.partition_point(|&p| p <= v);
            let want = idx + i as usize - 1;
            list.get(want).copied()
        });
        if let Some(p) = found {
            return Ok(Int::from(p));
        }
    }
    let six = Int::from(6);
    let nn = &n.floor_div(&six).expect("6 != 0") * &six;
    if nn == n {
        n = &n + 1;
        if isprime(&n) {
            i -= 1;
            if i == 0 {
                return Ok(n);
            }
        }
        n = &n + 4;
    } else if &n - &nn == 5 {
        n = &n + 2;
        if isprime(&n) {
            i -= 1;
            if i == 0 {
                return Ok(n);
            }
        }
        n = &n + 4;
    } else {
        n = &nn + 5;
    }
    loop {
        if isprime(&n) {
            i -= 1;
            if i == 0 {
                return Ok(n);
            }
        }
        n = &n + 2;
        if isprime(&n) {
            i -= 1;
            if i == 0 {
                return Ok(n);
            }
        }
        n = &n + 4;
    }
}
