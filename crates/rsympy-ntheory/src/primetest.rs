//! Port of `isprime` and `mr` from `sympy.ntheory.primetest`.

use rsympy_num::Int;

use crate::external::{bit_scan1, is_strong_bpsw_prp, md};
use crate::generate::{sieve_contains, sieve_last};

#[rustfmt::skip]
const MR_BASES_32: [u64; 256] = [
    15591, 2018, 166, 7429, 8064, 16045, 10503, 4399, 1949, 1295, 2776, 3620, 560, 3128, 5212,
    2657, 2300, 2021, 4652, 1471, 9336, 4018, 2398, 20462, 10277, 8028, 2213, 6219, 620, 3763,
    4852, 5012, 3185, 1333, 6227, 5298, 1074, 2391, 5113, 7061, 803, 1269, 3875, 422, 751, 580,
    4729, 10239, 746, 2951, 556, 2206, 3778, 481, 1522, 3476, 481, 2487, 3266, 5633, 488, 3373,
    6441, 3344, 17, 15105, 1490, 4154, 2036, 1882, 1813, 467, 3307, 14042, 6371, 658, 1005, 903,
    737, 1887, 7447, 1888, 2848, 1784, 7559, 3400, 951, 13969, 4304, 177, 41, 19875, 3110, 13221,
    8726, 571, 7043, 6943, 1199, 352, 6435, 165, 1169, 3315, 978, 233, 3003, 2562, 2994, 10587,
    10030, 2377, 1902, 5354, 4447, 1555, 263, 27027, 2283, 305, 669, 1912, 601, 6186, 429, 1930,
    14873, 1784, 1661, 524, 3577, 236, 2360, 6146, 2850, 55637, 1753, 4178, 8466, 222, 2579, 2743,
    2031, 2226, 2276, 374, 2132, 813, 23788, 1610, 4422, 5159, 1725, 3597, 3366, 14336, 579, 165,
    1375, 10018, 12616, 9816, 1371, 536, 1867, 10864, 857, 2206, 5788, 434, 8085, 17618, 727, 3639,
    1595, 4944, 2129, 2029, 8195, 8344, 6232, 9183, 8126, 1870, 3296, 7455, 8947, 25017, 541,
    19115, 368, 566, 5674, 411, 522, 1027, 8215, 2050, 6544, 10049, 614, 774, 2333, 3007, 35201,
    4706, 1152, 1785, 1028, 1540, 3743, 493, 4474, 2521, 26845, 8354, 864, 18915, 5465, 2447, 42,
    4511, 1660, 166, 1249, 6259, 2553, 304, 272, 7286, 73, 6554, 899, 2816, 5197, 13330, 7054,
    2818, 3199, 811, 922, 350, 7514, 4452, 3449, 2663, 4708, 418, 1621, 1171, 3471, 88, 11345,
    412, 1559, 194,
];

fn mr_test(n: &Int, base: &Int, s: u64, t: &Int) -> bool {
    let nm1 = n - 1;
    let mut b = base.pow_mod(t, n).expect("n > 0");
    if b.is_one() || b == nm1 {
        return true;
    }
    let two = Int::from(2);
    for _ in 0..s.saturating_sub(1) {
        b = b.pow_mod(&two, n).expect("n > 0");
        if b == nm1 {
            return true;
        }
        if b.is_one() {
            return false;
        }
    }
    false
}

/// `mr(n, bases)`: Miller-Rabin with the given bases (bases `>= n` wrap,
/// bases `< 2` are skipped).
pub fn mr(n: &Int, bases: &[Int]) -> bool {
    if *n < 2 || (*n > 2 && n.is_even()) {
        return false;
    }
    let nm1 = n - 1;
    let s = bit_scan1(&nm1).unwrap_or(0);
    let t = n >> s;
    for base in bases {
        let base = if base >= n { md(base, n) } else { base.clone() };
        if base >= 2 && !mr_test(n, &base, s, &t) {
            return false;
        }
    }
    true
}

fn mr_u64(n: &Int, bases: &[u64]) -> bool {
    let bases: Vec<Int> = bases.iter().map(|&b| Int::from(b)).collect();
    mr(n, &bases)
}

/// `isprime(n)`. Definitive below 2^64; above, the strong BPSW test, as in
/// SymPy (so any BPSW pseudoprime would be misjudged identically).
pub fn isprime(n: &Int) -> bool {
    if *n == 2 || *n == 3 || *n == 5 {
        return true;
    }
    if *n < 2 {
        return false;
    }
    if let Some(v) = n.as_u64() {
        if v % 2 == 0 || v % 3 == 0 || v % 5 == 0 {
            return false;
        }
        if v < 49 {
            return true;
        }
        for p in [7u64, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47] {
            if v % p == 0 {
                return false;
            }
        }
        if v < 2809 {
            return true;
        }
        if v < 65077 {
            let r = Int::from(2).pow_mod(&Int::from(v >> 1), n).expect("n > 0");
            return (r.is_one() || r == Int::from(v - 1))
                && ![8321, 31621, 42799, 49141, 49981].contains(&v);
        }
        if v <= sieve_last() {
            return sieve_contains(v);
        }
    } else {
        for p in [2i64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47] {
            if md(n, &Int::from(p)).is_zero() {
                return false;
            }
        }
    }
    // (SymPy consults factor_cache here; it can only confirm a prime, so
    // the answer is the same without it.)
    let lt = |b: u128| n.as_u64().is_some_and(|v| u128::from(v) < b);
    if lt(341_531) {
        return mr_u64(n, &[9_345_883_071_009_581_737]);
    }
    if lt(4_296_595_241) {
        let v = n.as_u64().expect("checked");
        let mut h = ((v >> 16) ^ v).wrapping_mul(0x45d_9f3b);
        h = ((h >> 16) ^ h).wrapping_mul(0x45d_9f3b);
        h = ((h >> 16) ^ h) & 255;
        return mr_u64(n, &[MR_BASES_32[h as usize]]);
    }
    if lt(350_269_456_337) {
        return mr_u64(
            n,
            &[
                4_230_279_247_111_683_200,
                14_694_767_155_120_705_706,
                16_641_139_526_367_750_375,
            ],
        );
    }
    if lt(55_245_642_489_451) {
        return mr_u64(
            n,
            &[
                2,
                141_889_084_524_735,
                1_199_124_725_622_454_117,
                11_096_072_698_276_303_650,
            ],
        );
    }
    if lt(7_999_252_175_582_851) {
        return mr_u64(
            n,
            &[
                2,
                4_130_806_001_517,
                149_795_463_772_692_060,
                186_635_894_390_467_037,
                3_967_304_179_347_715_805,
            ],
        );
    }
    if lt(585_226_005_592_931_977) {
        return mr_u64(
            n,
            &[
                2,
                123_635_709_730_000,
                9_233_062_284_813_009,
                43_835_965_440_333_360,
                761_179_012_939_631_437,
                1_263_739_024_124_850_375,
            ],
        );
    }
    if n.as_u64().is_some() {
        return mr_u64(n, &[2, 325, 9375, 28178, 450_775, 9_780_504, 1_795_265_022]);
    }
    let small: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    if *n < "318665857834031151167461".parse::<Int>().expect("literal") {
        return mr_u64(n, &small);
    }
    if *n < "3317044064679887385961981".parse::<Int>().expect("literal") {
        let mut b = small.to_vec();
        b.push(41);
        return mr_u64(n, &b);
    }
    is_strong_bpsw_prp(n).expect("n > 1")
}
