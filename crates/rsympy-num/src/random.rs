//! CPython's `random.Random`: MT19937 seeded and consumed exactly as
//! CPython 3.13 does, so seeded SymPy code replays the same sequences.

use crate::int::Int;

const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908_b0df;
const UPPER_MASK: u32 = 0x8000_0000;
const LOWER_MASK: u32 = 0x7fff_ffff;

/// Raised where Python raises `ValueError` from `randrange`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmptyRange;

#[derive(Clone)]
pub struct PyRandom {
    mt: [u32; N],
    mti: usize,
}

impl PyRandom {
    /// `random.Random(seed)` for an integer seed.
    pub fn new(seed: &Int) -> PyRandom {
        let mut r = PyRandom {
            mt: [0; N],
            mti: N + 1,
        };
        r.seed(seed);
        r
    }

    /// `seed(a)` for an int `a`: the key is `abs(a)` split into 32-bit
    /// words, least significant first (`[0]` for 0).
    pub fn seed(&mut self, a: &Int) {
        let mut n = a.abs();
        let mut key = Vec::new();
        let mask = Int::from(0xffff_ffffu32);
        while !n.is_zero() {
            key.push((&n & &mask).as_u32().expect("masked to 32 bits"));
            n = &n >> 32;
        }
        if key.is_empty() {
            key.push(0);
        }
        self.init_by_array(&key);
    }

    fn init_genrand(&mut self, s: u32) {
        self.mt[0] = s;
        for i in 1..N {
            let prev = self.mt[i - 1];
            self.mt[i] = 1_812_433_253u32
                .wrapping_mul(prev ^ (prev >> 30))
                .wrapping_add(i as u32);
        }
        self.mti = N;
    }

    fn init_by_array(&mut self, key: &[u32]) {
        self.init_genrand(19_650_218);
        let (mut i, mut j) = (1usize, 0usize);
        let mut k = N.max(key.len());
        while k > 0 {
            let prev = self.mt[i - 1];
            self.mt[i] = (self.mt[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1_664_525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                self.mt[0] = self.mt[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
            k -= 1;
        }
        k = N - 1;
        while k > 0 {
            let prev = self.mt[i - 1];
            self.mt[i] = (self.mt[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1_566_083_941))
                .wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                self.mt[0] = self.mt[N - 1];
                i = 1;
            }
            k -= 1;
        }
        self.mt[0] = 0x8000_0000;
        self.mti = N;
    }

    /// `genrand_uint32`.
    pub fn next_u32(&mut self) -> u32 {
        let mag01 = [0u32, MATRIX_A];
        if self.mti >= N {
            for kk in 0..N - M {
                let y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
                self.mt[kk] = self.mt[kk + M] ^ (y >> 1) ^ mag01[(y & 1) as usize];
            }
            for kk in N - M..N - 1 {
                let y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
                self.mt[kk] = self.mt[kk + M - N] ^ (y >> 1) ^ mag01[(y & 1) as usize];
            }
            let y = (self.mt[N - 1] & UPPER_MASK) | (self.mt[0] & LOWER_MASK);
            self.mt[N - 1] = self.mt[M - 1] ^ (y >> 1) ^ mag01[(y & 1) as usize];
            self.mti = 0;
        }
        let mut y = self.mt[self.mti];
        self.mti += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }

    /// `random()`: a float in [0, 1) with 53 random bits.
    pub fn random(&mut self) -> f64 {
        let a = self.next_u32() >> 5;
        let b = self.next_u32() >> 6;
        (f64::from(a) * 67_108_864.0 + f64::from(b)) * (1.0 / 9_007_199_254_740_992.0)
    }

    /// `getrandbits(k)`.
    pub fn getrandbits(&mut self, k: u64) -> Int {
        if k == 0 {
            return Int::ZERO;
        }
        if k <= 32 {
            return Int::from(self.next_u32() >> (32 - k));
        }
        let words = k.div_ceil(32);
        let mut out = Int::ZERO;
        let mut left = k;
        for i in 0..words {
            let mut r = self.next_u32();
            if left < 32 {
                r >>= 32 - left;
            }
            out = &out + &(&Int::from(r) << (32 * i));
            left = left.saturating_sub(32);
        }
        out
    }

    /// `_randbelow(n)` for `n > 0`.
    pub fn randbelow(&mut self, n: &Int) -> Int {
        let k = n.bit_length();
        let mut r = self.getrandbits(k);
        while &r >= n {
            r = self.getrandbits(k);
        }
        r
    }

    /// `randrange(start, stop)`.
    pub fn randrange(&mut self, start: &Int, stop: &Int) -> Result<Int, EmptyRange> {
        let width = stop - start;
        if width.is_positive() {
            Ok(start + &self.randbelow(&width))
        } else {
            Err(EmptyRange)
        }
    }

    /// `randint(a, b)`: uniform on [a, b].
    pub fn randint(&mut self, a: &Int, b: &Int) -> Result<Int, EmptyRange> {
        self.randrange(a, &(b + 1))
    }
}
