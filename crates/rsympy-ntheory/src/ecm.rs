//! Port of `sympy.ntheory.ecm._ecm_one_factor`: Lenstra's elliptic curve
//! method on Montgomery curves, with SymPy's seeded curve choice.

use rsympy_num::Int;

use crate::external::{invert, isqrt, md};
use crate::factor::{NtCtx, log_base};
use crate::generate::primerange;

#[derive(Clone)]
struct Point<'a> {
    x: Int,
    z: Int,
    a24: &'a Int,
    m: &'a Int,
}

impl<'a> Point<'a> {
    fn add(&self, q: &Point<'a>, diff: &Point<'a>) -> Point<'a> {
        let u = &(&self.x - &self.z) * &(&q.x + &q.z);
        let v = &(&self.x + &self.z) * &(&q.x - &q.z);
        let (add, subt) = (&u + &v, &u - &v);
        let x = md(&(&(&diff.z * &add) * &add), self.m);
        let z = md(&(&(&diff.x * &subt) * &subt), self.m);
        Point {
            x,
            z,
            a24: self.a24,
            m: self.m,
        }
    }

    fn double(&self) -> Point<'a> {
        let two = Int::from(2);
        let u = (&self.x + &self.z).pow_mod(&two, self.m).expect("m > 0");
        let v = (&self.x - &self.z).pow_mod(&two, self.m).expect("m > 0");
        let diff = &u - &v;
        let x = md(&(&u * &v), self.m);
        let z = md(&(&diff * &(&v + &(self.a24 * &diff))), self.m);
        Point {
            x,
            z,
            a24: self.a24,
            m: self.m,
        }
    }

    fn mont_ladder(&self, k: &Int) -> Point<'a> {
        let mut q = self.clone();
        let mut r = self.double();
        let len = k.bit_length();
        for i in (0..len.saturating_sub(1)).rev() {
            if k.bit(i) {
                q = r.add(&q, self);
                r = r.double();
            } else {
                r = q.add(&r, self);
                q = q.double();
            }
        }
        q
    }
}

/// `_ecm_one_factor(n, B1, B2, max_curve, seed)`: a nontrivial factor of
/// `n`, or `None` if no curve finds one.
pub fn ecm_one_factor(
    nt: &mut NtCtx,
    n: &Int,
    b1: &Int,
    b2: &Int,
    max_curve: &Int,
    seed: &Int,
) -> Option<Int> {
    nt.rng.seed(seed);
    let d_big = isqrt(b2).min(&b1.floor_div(&Int::from(2)).expect("2 != 0") - 1);
    let d = d_big.as_usize().expect("D is small");
    let mut beta = vec![Int::ZERO; d];
    let mut k = Int::ONE;
    for p in primerange(&Int::from(2), &(b1 + 1)) {
        k = &k * &p.pow(log_base(b1, &p) as u64);
    }
    let two_d = &d_big * 2;
    let four_d = &d_big * 4;
    let mut deltas_list: Vec<Vec<usize>> = Vec::new();
    let mut r = b1 + &two_d;
    let r_end = b2 + &two_d;
    while r < r_end {
        let mut deltas: Vec<usize> = primerange(&(&r - &two_d), &(&r + &two_d))
            .iter()
            .map(|q| ((q - &r).abs() >> 1).as_usize().expect("small"))
            .collect();
        // A Python set: the product below doesn't depend on its order.
        deltas.sort_unstable();
        deltas.dedup();
        deltas_list.push(deltas);
        r = &r + &four_d;
    }

    let curves = max_curve.as_u64().expect("small");
    for _ in 0..curves {
        let sigma = nt.rng.randint(&Int::from(6), &(n - 1)).expect("n > 6");
        let u = md(&(&(&sigma * &sigma) - 5), n);
        let v = md(&(&sigma * 4), n);
        let three = Int::from(3);
        let u_3 = u.pow_mod(&three, n).expect("n > 0");
        let inv = invert(&(&(&u_3 * &v) * 16), n);
        let a24 = match inv {
            Ok(inv) => md(
                &(&(&(&v - &u).pow_mod(&three, n).expect("n > 0") * &(&(&u * 3) + &v)) * &inv),
                n,
            ),
            Err(_) => {
                let g = (&(&u_3 * &v) * 2).gcd(n);
                if g == *n {
                    continue;
                }
                return Some(g);
            }
        };
        let q0 = Point {
            x: u_3.clone(),
            z: v.pow_mod(&three, n).expect("n > 0"),
            a24: &a24,
            m: n,
        };
        let q = q0.mont_ladder(&k);
        let g = q.z.gcd(n);
        if !g.is_one() && g != *n {
            return Some(g);
        } else if g == *n {
            continue;
        }
        let mut s: Vec<Point> = Vec::with_capacity(d);
        s.push(q.clone());
        let q2 = q.double();
        s.push(q2.add(&q, &q));
        beta[0] = md(&(&s[0].x * &s[0].z), n);
        beta[1] = md(&(&s[1].x * &s[1].z), n);
        for dd in 2..d {
            let next = s[dd - 1].add(&q2, &s[dd - 2]);
            beta[dd] = md(&(&next.x * &next.z), n);
            s.push(next);
        }
        let mut g = Int::ONE;
        let w = q.mont_ladder(&four_d);
        let mut t = q.mont_ladder(&(b1 - &two_d));
        let mut rr = q.mont_ladder(&(b1 + &two_d));
        for deltas in &deltas_list {
            let alpha = md(&(&rr.x * &rr.z), n);
            for &delta in deltas {
                let f =
                    &(&(&(&rr.x - &s[delta].x) * &(&rr.z + &s[delta].z)) - &alpha) + &beta[delta];
                g = md(&(&g * &f), n);
            }
            let next = rr.add(&w, &t);
            t = std::mem::replace(&mut rr, next);
        }
        let g = n.gcd(&g);
        if !g.is_one() && g != *n {
            return Some(g);
        }
    }
    None
}
