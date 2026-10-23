//! Port of the factoring core of `sympy.ntheory.factor_`: `factorint` with
//! its trial division, Fermat, Pollard p-1, Pollard rho and ECM stages,
//! `perfect_power`, `multiplicity`, `primefactors`, `divisors` and the
//! `factor_cache`.
//!
//! `factorint(n, limit=...)` can return a composite last factor, and which
//! one depends on the exact sequence of stages (and on the seeded random
//! streams and `factor_cache`), so the stages are ported step for step.

use std::collections::VecDeque;

use rsympy_num::{Int, PyRandom};

use crate::ecm::ecm_one_factor;
use crate::external::{NtResult, bit_scan1, iroot, isqrt, md, remove, sqrtrem};
use crate::generate::{primerange, sieve_contains, sieve_last};
use crate::primetest::isprime;

/// A Python dict `{prime: exponent}`: keeps insertion order, and assigning
/// to an existing key keeps its position.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Factors(Vec<(Int, u64)>);

impl Factors {
    pub fn new() -> Factors {
        Factors(Vec::new())
    }

    pub fn get(&self, k: &Int) -> Option<u64> {
        self.0.iter().find(|(p, _)| p == k).map(|(_, e)| *e)
    }

    pub fn set(&mut self, k: Int, v: u64) {
        match self.0.iter_mut().find(|(p, _)| *p == k) {
            Some(slot) => slot.1 = v,
            None => self.0.push((k, v)),
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Entries in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&Int, u64)> {
        self.0.iter().map(|(p, e)| (p, *e))
    }

    /// Entries sorted by key, as `sorted(d.items())`.
    pub fn sorted(&self) -> Vec<(Int, u64)> {
        let mut v = self.0.clone();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }

    pub fn keys(&self) -> impl Iterator<Item = &Int> {
        self.0.iter().map(|(p, _)| p)
    }
}

/// SymPy's `factor_cache` (`FactorCache(maxsize=1000)`): maps `n` to its
/// largest known prime factor, least recently used first out.
#[derive(Clone, Debug)]
pub struct FactorCache {
    entries: VecDeque<(Int, Int)>,
    maxsize: Option<usize>,
}

impl Default for FactorCache {
    fn default() -> Self {
        FactorCache {
            entries: VecDeque::new(),
            maxsize: Some(1000),
        }
    }
}

impl FactorCache {
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `factor_cache[n] = factor`.
    pub fn set(&mut self, n: &Int, factor: &Int) {
        debug_assert!(
            *factor > 1 && factor <= n && md(n, factor).is_zero() && isprime(factor),
            "{factor} is not a prime factor of {n}"
        );
        match self.entries.iter_mut().find(|(k, _)| k == n) {
            Some(slot) => {
                if *factor > slot.1 {
                    slot.1 = factor.clone();
                }
            }
            None => self.entries.push_back((n.clone(), factor.clone())),
        }
        if let Some(max) = self.maxsize
            && self.entries.len() > max
        {
            self.entries.pop_front();
        }
    }

    /// `factor_cache.get(n)`.
    pub fn get(&mut self, n: &Int) -> Option<Int> {
        if let Some(v) = n.as_u64()
            && v <= sieve_last()
            && sieve_contains(v)
        {
            return Some(n.clone());
        }
        let pos = self.entries.iter().position(|(k, _)| k == n)?;
        let entry = self.entries.remove(pos).expect("position is valid");
        let f = entry.1.clone();
        self.entries.push_back(entry);
        Some(f)
    }

    /// `factor_cache.add(n, factors)`.
    pub fn add(&mut self, n: &Int, factors: &[Int]) {
        let mut fs = factors.to_vec();
        fs.sort();
        let mut n = n.clone();
        for p in fs.iter().rev() {
            self.set(&n, p);
            n = remove(&n, p).expect("p > 1").0;
        }
    }
}

/// The mutable global state SymPy's number theory reads and writes:
/// `sympy.core.random.rng` (re-seeded by Pollard rho, p-1 and ECM) and
/// `factor_cache`.
#[derive(Clone)]
pub struct NtCtx {
    pub rng: PyRandom,
    pub factor_cache: FactorCache,
}

impl Default for NtCtx {
    /// The state the oracle records SymPy in: `sympy.core.random.seed(0)`
    /// and an empty cache.
    fn default() -> Self {
        NtCtx {
            rng: PyRandom::new(&Int::ZERO),
            factor_cache: FactorCache::default(),
        }
    }
}

fn gcd(a: &Int, b: &Int) -> Int {
    a.gcd(b)
}

/// `multiplicity(p, n)` for integers: how many times `p` divides `n`.
pub fn multiplicity(p: &Int, n: &Int) -> NtResult<u64> {
    if n.is_zero() {
        return crate::external::value_err(format!(
            "no such integer exists: multiplicity of {n} is not-defined"
        ));
    }
    Ok(remove(n, p)?.1)
}

fn prod_root(factors: &Factors, g: u64) -> Int {
    factors.iter().map(|(p, e)| p.pow(e / g)).product()
}

fn gcd_u64(a: u64, b: u64) -> u64 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// `_PyLong_Frexp`: `(m, e)` with `n ~= m * 2**e`, `m` in [0.5, 1) correctly
/// rounded to 53 bits.
fn frexp_int(n: &Int) -> (f64, i64) {
    let bits = n.bit_length();
    let shift = bits.saturating_sub(55);
    let mut top = (n >> shift).as_u64().expect("55 bits");
    if shift > 0 && n.trailing_zeros().is_some_and(|z| z < shift) {
        top |= 1; // sticky bit
    }
    let lead = 64 - u64::from(top.leading_zeros());
    let mut m = (top as f64) / 2f64.powi(lead as i32);
    let mut e = (shift + lead) as i64;
    if m == 1.0 {
        m = 0.5;
        e += 1;
    }
    (m, e)
}

/// CPython's `loghelper`: `func(float(n))`, or `func(m) + func(2.0) * e`
/// when `float(n)` overflows.
fn log_helper(n: &Int, func: fn(f64) -> f64) -> f64 {
    match n.to_f64() {
        Some(f) => func(f),
        None => {
            let (m, e) = frexp_int(n);
            func(m) + func(2.0) * e as f64
        }
    }
}

/// `math.log2(n)` for a positive int.
pub(crate) fn log2_int(n: &Int) -> f64 {
    log_helper(n, f64::log2)
}

/// `math.log(n)` for a positive int.
pub(crate) fn ln_int(n: &Int) -> f64 {
    log_helper(n, f64::ln)
}

/// `math.log(x, base)` = `log(x) / log(base)`, as CPython computes it.
pub(crate) fn log_base(x: &Int, base: &Int) -> f64 {
    ln_int(x) / ln_int(base)
}

/// `_perfect_power(n, next_p)`: `(b, e)` with `n == b**e`, `e` maximal and
/// `> 1`, assuming `n` has no prime factor below `next_p`.
pub fn perfect_power_raw(nt: &mut NtCtx, n: &Int, next_p: u64) -> Option<(Int, u64)> {
    if *n <= 3 {
        return None;
    }
    let mut factors = Factors::new();
    let mut g: u64 = 0;
    let mut multi: u64 = 1;
    let mut n = n.clone();
    let mut next_p = next_p;

    let done = |n: Int, mut factors: Factors, g: u64, multi: u64| -> Option<(Int, u64)> {
        let g = gcd_u64(g, multi);
        if g == 1 {
            return None;
        }
        factors.set(n, multi);
        Some((prod_root(&factors, g), g))
    };

    if n <= 1_000_000 {
        let (rest, _) = factorint_small(&mut factors, &n, 1_000, 1_000, next_p);
        if rest > 1 {
            return None;
        }
        let g = factors.iter().fold(0, |acc, (_, e)| gcd_u64(acc, e));
        if g == 1 {
            return None;
        }
        return Some((prod_root(&factors, g), g));
    }

    if next_p < 3 {
        let tz = bit_scan1(&n).expect("n > 3");
        g = tz;
        if g != 0 {
            if g == 1 {
                return None;
            }
            n = &n >> g;
            factors.set(Int::from(2), g);
            if n.is_one() {
                return Some((Int::from(2), g));
            }
            let (m, exact) = iroot(&n, g).expect("valid");
            if exact {
                return Some((&m * 2, g));
            } else if isprime(&Int::from(g)) {
                return None;
            }
        }
        next_p = 3;
    }

    let seven = Int::from(7);
    while (&n & &seven).is_one() {
        let (m, exact) = iroot(&n, 2).expect("valid");
        if exact {
            n = m;
            multi <<= 1;
        } else {
            break;
        }
    }
    if n < Int::from(next_p).pow(3) {
        return done(n, factors, g, multi);
    }

    let tf_max = n.bit_length() / 27 + 24;
    if next_p < tf_max {
        for p in primerange(&Int::from(next_p), &Int::from(tf_max)) {
            let (m, t) = remove(&n, &p).expect("p > 1");
            if t != 0 {
                n = m;
                let t = t * multi;
                let g2 = gcd_u64(g, t);
                if g2 == 1 {
                    return None;
                }
                factors.set(p.clone(), t);
                if n.is_one() {
                    return Some((prod_root(&factors, g2), g2));
                } else if g == 0 || g2 < g {
                    g = g2;
                    let (m, exact) = iroot(&n.pow(multi), g).expect("valid");
                    if exact {
                        return Some((&m * &prod_root(&factors, g), g));
                    } else if isprime(&Int::from(g)) {
                        return None;
                    }
                }
            }
        }
        next_p = tf_max;
    }
    if n < Int::from(next_p).pow(3) {
        return done(n, factors, g, multi);
    }

    let prime_iter: Vec<u64> = if g != 0 {
        let odd = Int::from(g >> g.trailing_zeros());
        let f = factorint(nt, &odd, &FactorOpts::default());
        let mut ks: Vec<u64> = f.keys().map(|k| k.as_u64().expect("small")).collect();
        ks.sort_unstable();
        ks
    } else {
        let top = log_base(&n, &Int::from(next_p)) as i64 + 2;
        primerange(&Int::from(3), &Int::from(top))
            .iter()
            .map(|p| p.as_u64().expect("small"))
            .collect()
    };
    let mut logn = log2_int(&n);
    let threshold = logn / 40.0;
    for p in prime_iter {
        if threshold < p as f64 {
            loop {
                let b = 2f64.powf(logn / p as f64);
                let rb_f = (b + 0.5).trunc();
                let rb = Int::from(rb_f as i64);
                if (rb_f - b).abs() < 0.01 && rb.pow(p) == n {
                    n = rb;
                    multi *= p;
                    logn = log2_int(&n);
                } else {
                    break;
                }
            }
        } else {
            loop {
                let (m, exact) = iroot(&n, p).expect("valid");
                if exact {
                    n = m;
                    multi *= p;
                    logn = log2_int(&n);
                } else {
                    break;
                }
            }
        }
        if n < Int::from(next_p).pow(p + 2) {
            break;
        }
    }
    done(n, factors, g, multi)
}

/// `perfect_power(n)` for an integer `n` with the default `big=True,
/// factor=True` and no candidates.
pub fn perfect_power(nt: &mut NtCtx, n: &Int) -> Option<(Int, u64)> {
    if n.is_negative() {
        let (b, e) = perfect_power(nt, &-n)?;
        let e2 = 1u64 << e.trailing_zeros();
        let (b, e) = (b.pow(e2), e / e2);
        if e <= 1 {
            return None;
        }
        return Some((-b, e));
    }
    perfect_power_raw(nt, n, 2)
}

/// Options for `factorint`.
#[derive(Clone, Debug)]
pub struct FactorOpts {
    pub limit: Option<Int>,
    pub use_trial: bool,
    pub use_rho: bool,
    pub use_pm1: bool,
    pub use_ecm: bool,
}

impl Default for FactorOpts {
    fn default() -> Self {
        FactorOpts {
            limit: None,
            use_trial: true,
            use_rho: true,
            use_pm1: true,
            use_ecm: true,
        }
    }
}

impl FactorOpts {
    pub fn with_limit(limit: Int) -> FactorOpts {
        FactorOpts {
            limit: Some(limit),
            ..FactorOpts::default()
        }
    }
}

/// `_trial(factors, n, candidates)`.
fn trial(nt: &mut NtCtx, factors: &mut Factors, n: &Int, candidates: &[Int]) -> (Int, bool) {
    let before = factors.len();
    let mut n = n.clone();
    for d in candidates {
        if md(&n, d).is_zero() {
            if n != *d {
                nt.factor_cache.set(&n, d);
            }
            let (rest, m) = remove(&n.div_exact(d), d).expect("d > 1");
            n = rest;
            factors.set(d.clone(), m + 1);
        }
    }
    let changed = factors.len() != before;
    (n, changed)
}

/// `_check_termination(...)`.
fn check_termination(
    nt: &mut NtCtx,
    factors: &mut Factors,
    n: &Int,
    opts: &FactorOpts,
    next_p: &Int,
) -> bool {
    if n.is_one() {
        return true;
    }
    if *n < next_p * next_p || isprime(n) {
        nt.factor_cache.set(n, n);
        factors.set(n.clone(), 1);
        return true;
    }
    let np = next_p.as_u64().unwrap_or(u64::MAX);
    let Some((base, exp)) = perfect_power_raw(nt, n, np) else {
        return false;
    };
    if base < next_p * next_p || isprime(&base) {
        nt.factor_cache.set(n, &base);
        factors.set(base, exp);
    } else {
        let sub = FactorOpts {
            use_ecm: true,
            ..opts.clone()
        };
        let facs = factorint(nt, &base, &sub);
        for (b, e) in facs.iter() {
            factors.set(b.clone(), exp * e);
        }
    }
    true
}

/// `_factorint_small(factors, n, limit, fail_max, next_p)`: trial division
/// by 2, 3 and 6k +- 1, stopping after `fail_max` misses in a row. Returns
/// the cofactor and the next candidate (0 when the cofactor is 1 or prime).
pub(crate) fn factorint_small(
    factors: &mut Factors,
    n: &Int,
    limit: u64,
    fail_max: u64,
    next_p: u64,
) -> (Int, u64) {
    let done = |n: Int, d: u64| -> (Int, u64) {
        if Int::from(d) * Int::from(d) <= n {
            (n, d)
        } else {
            (n, 0)
        }
    };
    let mut n = n.clone();
    let mut next_p = next_p;
    let limit2 = Int::from(limit) * Int::from(limit);
    let mut threshold2 = n.clone().min(limit2.clone());

    // Divides out every factor `p`, recording its multiplicity.
    let divide_out = |n: &mut Int, factors: &mut Factors, p: u64| {
        let pi = Int::from(p);
        *n = n.div_exact(&pi);
        let mut m = 1;
        while md(n, &pi).is_zero() {
            *n = n.div_exact(&pi);
            m += 1;
            if m == 20 {
                let (rest, mm) = remove(n, &pi).expect("p > 1");
                *n = rest;
                m += mm;
                break;
            }
        }
        factors.set(pi, m);
    };

    if next_p < 3 {
        if n.is_even() {
            let m = bit_scan1(&n).expect("n > 0");
            factors.set(Int::from(2), m);
            n = &n >> m;
            threshold2 = n.clone().min(limit2.clone());
        }
        next_p = 3;
        if threshold2 < 9 {
            return done(n, next_p);
        }
    }
    if next_p < 5 {
        if md(&n, &Int::from(3)).is_zero() {
            divide_out(&mut n, factors, 3);
            threshold2 = n.clone().min(limit2.clone());
        }
        next_p = 5;
        if threshold2 < 25 {
            return done(n, next_p);
        }
    }
    let p6 = next_p % 6;
    next_p = next_p + if p6 < 2 { 0 } else { 6 } - 1 - p6;
    let mut fails = 0;
    while fails < fail_max {
        for step in [2u64, 4] {
            if !md(&n, &Int::from(next_p)).is_zero() {
                fails += 1;
            } else {
                divide_out(&mut n, factors, next_p);
                fails = 0;
                threshold2 = n.clone().min(limit2.clone());
            }
            next_p += step;
            if threshold2 < Int::from(next_p) * Int::from(next_p) {
                return done(n, next_p);
            }
        }
    }
    done(n, next_p)
}

/// `pollard_rho(n, s=2, a=1, retries, seed, max_steps)`.
pub fn pollard_rho(
    nt: &mut NtCtx,
    n: &Int,
    retries: u64,
    seed: &Int,
    max_steps: Option<&Int>,
) -> NtResult<Option<Int>> {
    if *n < 5 {
        return crate::external::value_err("pollard_rho should receive n > 4");
    }
    nt.rng.seed(&(seed + &Int::from(retries)));
    let mut v = Int::from(2);
    let mut a = Int::ONE;
    let two = Int::from(2);
    let max_steps = max_steps.filter(|m| !m.is_zero());
    for _ in 0..=retries {
        let mut u = v.clone();
        let f = |x: &Int, a: &Int| md(&(&x.pow_mod(&two, n).expect("n > 0") + a), n);
        let mut j = Int::ZERO;
        loop {
            if let Some(ms) = max_steps
                && j > *ms
            {
                break;
            }
            j = &j + 1;
            u = f(&u, &a);
            v = f(&f(&v, &a), &a);
            let g = gcd(&(&u - &v), n);
            if g.is_one() {
                continue;
            }
            if g == *n {
                break;
            }
            return Ok(Some(g));
        }
        v = nt.rng.randint(&Int::ZERO, &(n - 1)).expect("n > 4");
        a = nt.rng.randint(&Int::ONE, &(n - 3)).expect("n > 4");
    }
    Ok(None)
}

/// `pollard_pm1(n, B, a=2, retries=0, seed)`.
pub fn pollard_pm1(nt: &mut NtCtx, n: &Int, b: &Int, seed: &Int) -> NtResult<Option<Int>> {
    if *n < 4 || *b < 3 {
        return crate::external::value_err("pollard_pm1 should receive n > 3 and B > 2");
    }
    nt.rng.seed(&(seed + b));
    let a = Int::from(2);
    // retries = 0: one pass.
    let mut am = a.clone();
    for p in primerange(&Int::from(2), &(b + 1)) {
        let e = log_base(b, &p) as u64;
        am = am.pow_mod(&p.pow(e), n).expect("n > 0");
    }
    let g = gcd(&(&am - 1), n);
    if g > 1 && g < *n {
        return Ok(Some(g));
    }
    // SymPy draws a new `a` here even though no retry follows.
    let _ = nt.rng.randint(&Int::from(2), &(n - 2));
    Ok(None)
}

/// `factorint(n, limit, use_trial, use_rho, use_pm1, use_ecm)` for an
/// integer `n`.
pub fn factorint(nt: &mut NtCtx, n: &Int, opts: &FactorOpts) -> Factors {
    let mut opts = opts.clone();
    if opts.limit.is_some() {
        opts.use_ecm = false;
    }
    // `if limit:` treats 0 as no limit.
    if opts.limit.as_ref().is_some_and(Int::is_zero) {
        opts.limit = None;
    }
    let limit = opts.limit.clone();

    if n.is_negative() {
        let mut f = factorint(nt, &-n, &opts);
        f.set(Int::NEG_ONE, 1);
        return f;
    }
    if limit.as_ref().is_some_and(|l| *l < 2) {
        let mut f = Factors::new();
        if !n.is_one() {
            f.set(n.clone(), 1);
        }
        return f;
    } else if *n < 10 {
        let small: [&[(i64, u64)]; 10] = [
            &[(0, 1)],
            &[],
            &[(2, 1)],
            &[(3, 1)],
            &[(2, 2)],
            &[(5, 1)],
            &[(2, 1), (3, 1)],
            &[(7, 1)],
            &[(2, 3)],
            &[(3, 2)],
        ];
        let mut f = Factors::new();
        for &(p, e) in small[n.as_usize().expect("0 <= n < 10")] {
            f.set(Int::from(p), e);
        }
        return f;
    }

    let mut factors = Factors::new();
    let small: u64 = 1 << 15;
    let fail_max = 600;
    let small = match &limit {
        Some(l) if *l < Int::from(small) => l.as_u64().expect("small limit"),
        _ => small,
    };
    let (mut n, next_p) = factorint_small(&mut factors, n, small, fail_max, 2);
    if next_p == 0 {
        if n > 1 {
            factors.set(n, 1);
        }
        return factors;
    }
    let mut next_p = Int::from(next_p);
    while let Some(p) = nt.factor_cache.get(&n) {
        let (rest, e) = remove(&n, &p).expect("p > 1");
        n = rest;
        factors.set(p, e);
    }
    if let Some(l) = &limit
        && next_p > *l
    {
        if check_termination(nt, &mut factors, &n, &opts, &next_p) {
            return factors;
        }
        if n > 1 {
            factors.set(n, 1);
        }
        return factors;
    }
    if check_termination(nt, &mut factors, &n, &opts, &next_p) {
        return factors;
    }

    // Fermat's method, for factors close to sqrt(n).
    let sqrt_n = isqrt(&n);
    let mut a = &sqrt_n + 1;
    let n_mod4_is1 = md(&n, &Int::from(4)).is_one();
    if n_mod4_is1 ^ a.is_odd() {
        a = &a + 1;
    }
    let a2 = &a * &a;
    let mut b2 = &a2 - &n;
    for _ in 0..3 {
        let (b, rem) = sqrtrem(&b2);
        if rem.is_zero() {
            // SymPy doesn't pass use_ecm here, so it is the default (on).
            let sub = FactorOpts {
                use_ecm: true,
                ..opts.clone()
            };
            for r in [&a - &b, &a + &b] {
                let facs = factorint(nt, &r, &sub);
                for (k, v) in facs.iter() {
                    let prev = factors.get(k).unwrap_or(0);
                    factors.set(k.clone(), prev + v);
                }
                let keys: Vec<Int> = facs.keys().cloned().collect();
                nt.factor_cache.add(&n, &keys);
            }
            return factors;
        }
        b2 = &b2 + &((&a + 1) << 2);
        a = &a + 2;
    }

    let mut low = next_p.clone();
    let mut high = &next_p * 2;
    let lim = &limit.clone().unwrap_or_else(|| sqrt_n.clone()) + 1;
    let mut iteration = 0u64;
    loop {
        let high_ = high.clone().min(lim.clone());
        let found_trial = if opts.use_trial {
            let ps = primerange(&low, &high_);
            let (rest, found) = trial(nt, &mut factors, &n, &ps);
            n = rest;
            next_p = high_.clone();
            if found && check_termination(nt, &mut factors, &n, &opts, &next_p) {
                return factors;
            }
            found
        } else {
            false
        };

        if high > lim {
            if n > 1 {
                factors.set(n, 1);
            }
            return factors;
        }

        if !found_trial {
            if opts.use_pm1
                && let Ok(Some(c)) = pollard_pm1(nt, &n, &low, &high_)
            {
                let ps = if c < &next_p * &next_p || isprime(&c) {
                    vec![c]
                } else {
                    factorint(nt, &c, &opts).keys().cloned().collect()
                };
                n = trial(nt, &mut factors, &n, &ps).0;
                if check_termination(nt, &mut factors, &n, &opts, &next_p) {
                    return factors;
                }
            }
            if opts.use_rho
                && let Ok(Some(c)) = pollard_rho(nt, &n, 1, &high_, Some(&low))
            {
                let ps = if c < &next_p * &next_p || isprime(&c) {
                    vec![c]
                } else {
                    factorint(nt, &c, &opts).keys().cloned().collect()
                };
                n = trial(nt, &mut factors, &n, &ps).0;
                if check_termination(nt, &mut factors, &n, &opts, &next_p) {
                    return factors;
                }
            }
        }
        iteration += 1;
        if opts.use_ecm && iteration >= 3 && n.num_decimal_digits() >= 24 {
            break;
        }
        low = high.clone();
        high = &high * 2;
    }

    let mut b1 = Int::from(10_000);
    let mut b2 = &b1 * 100;
    let mut num_curves = Int::from(50);
    loop {
        let found = ecm_one_factor(nt, &n, &b1, &b2, &num_curves, &b1);
        if let Some(factor) = found {
            let ps = if factor < &next_p * &next_p || isprime(&factor) {
                vec![factor]
            } else {
                factorint(nt, &factor, &opts).keys().cloned().collect()
            };
            n = trial(nt, &mut factors, &n, &ps).0;
            if check_termination(nt, &mut factors, &n, &opts, &next_p) {
                return factors;
            }
        }
        b1 = &b1 * 5;
        b2 = &b1 * 100;
        num_curves = &num_curves * 4;
    }
}

/// `primefactors(n, limit)`: the sorted prime factors (a composite last
/// factor left by `limit` is dropped).
pub fn primefactors(nt: &mut NtCtx, n: &Int, limit: Option<Int>) -> Vec<Int> {
    let opts = FactorOpts {
        limit,
        ..FactorOpts::default()
    };
    let mut fs: Vec<Int> = factorint(nt, n, &opts).keys().cloned().collect();
    fs.sort();
    let mut s: Vec<Int> = Vec::new();
    if let Some((last, rest)) = fs.split_last() {
        s.extend(rest.iter().filter(|f| **f > 1).cloned());
        if isprime(last) {
            s.push(last.clone());
        }
    }
    s
}

/// `divisors(n)`: all positive divisors of `|n|`, sorted.
pub fn divisors(nt: &mut NtCtx, n: &Int) -> Vec<Int> {
    let n = n.abs();
    if n <= 1 {
        return if n.is_zero() { vec![] } else { vec![Int::ONE] };
    }
    let f = factorint(nt, &n, &FactorOpts::default());
    let mut out = vec![Int::ONE];
    for (p, e) in f.sorted() {
        let mut next = Vec::with_capacity(out.len() * (e as usize + 1));
        for d in &out {
            let mut q = d.clone();
            next.push(q.clone());
            for _ in 0..e {
                q = &q * &p;
                next.push(q.clone());
            }
        }
        out = next;
    }
    out.sort();
    out
}
