//! Canonical orderings: `Basic.compare` (the order of `Add`/`Mul` args),
//! `sort_key`/`default_sort_key` (printing order, `ordered`), and a port of
//! CPython's `list.sort`, since SymPy's results depend on its exact
//! behaviour with the keys it is given.
//!
//! Ported from `sympy/core/basic.py` (`compare`, `_cmp_name`, `sort_key`),
//! `sympy/core/expr.py` (`sort_key`, `as_ordered_terms`, `as_terms`,
//! `_parse_order`, `__complex__`), `sympy/core/mul.py`
//! (`as_ordered_factors`), `sympy/core/sorting.py`,
//! `sympy/polys/orderings.py` (`monomial_key`), and CPython 3.13's
//! `Objects/listobject.c` (`list.sort`).
//!
//! Which class's method runs (`Symbol.sort_key` or `Expr.sort_key`, ...)
//! is read from `generated_compare.rs`, which records the method Python's
//! attribute lookup finds for each class.

use std::any::Any;
use std::cmp::Ordering;
use std::panic::{self, AssertUnwindSafe};

use rsympy_mpf as mlib;
use rsympy_num::Int;

use crate::error::{ErrorKind, catch, not_ported, raise, reraise};
use crate::expr::Expr;
use crate::generated::{ClassId as C, ORDERING_OF_CLASSES, Tag};
use crate::generated_compare::{COMPARE_INFO, CompareInfo};
use crate::numbers::{self, s};
use crate::py::PyDict;

fn info(e: &Expr) -> &'static CompareInfo {
    &COMPARE_INFO[e.cls() as usize]
}

// ===========================================================================
// CPython 3.13 list.sort (Objects/listobject.c)
// ===========================================================================
//
// The sort runs on a permutation of indices into the slice, comparing
// `lt(&v[i], &v[j])`, and applies the permutation at the end. A comparison
// that raises is caught, the C code's failure path runs (which leaves the
// list in some permutation of its items, as CPython does), and the error is
// re-raised once that permutation has been applied.

type Panic = Box<dyn Any + Send + 'static>;
type SortResult<T> = Result<T, Panic>;

/// `MAX_MINRUN`.
const MAX_MINRUN: usize = 64;
/// `MIN_GALLOP`.
const MIN_GALLOP: isize = 7;

/// `ISLT(X, Y)`: `X < Y`, with an exception as `Err`.
fn islt<F: FnMut(usize, usize) -> bool>(lt: &mut F, x: usize, y: usize) -> SortResult<bool> {
    panic::catch_unwind(AssertUnwindSafe(|| lt(x, y)))
}

/// `binarysort(ms, ss, n, ok)` on `a = keys[..n]`, with `a[:ok]` sorted.
fn binarysort<F: FnMut(usize, usize) -> bool>(
    lt: &mut F,
    a: &mut [usize],
    mut ok: usize,
) -> SortResult<()> {
    let n = a.len();
    if ok == 0 {
        ok += 1;
    }
    while ok < n {
        // set L to where a[ok] belongs
        let mut l = 0;
        let mut r = ok;
        let pivot = a[ok];
        loop {
            let m = (l + r) >> 1;
            if islt(lt, pivot, a[m])? {
                r = m;
            } else {
                l = m + 1;
            }
            if l >= r {
                break;
            }
        }
        a.copy_within(l..ok, l + 1);
        a[l] = pivot;
        ok += 1;
    }
    Ok(())
}

/// `count_run(ms, slo, nremaining)` on `lo = keys[..nremaining]`: the
/// length of the run at the start, made ascending in place.
fn count_run<F: FnMut(usize, usize) -> bool>(lt: &mut F, lo: &mut [usize]) -> SortResult<usize> {
    let nremaining = lo.len();
    // REVERSE_LAST_NEQ
    fn reverse_last_neq(lo: &mut [usize], n: usize, neq: &mut usize) {
        if *neq > 0 {
            *neq += 1;
            lo[n - *neq..n].reverse();
            *neq = 0;
        }
    }
    // try ascending run first
    let mut n = 1;
    while n < nremaining {
        // IF_NEXT_SMALLER
        if islt(lt, lo[n], lo[n - 1])? {
            break;
        }
        n += 1;
    }
    if n == nremaining {
        return Ok(n);
    }
    // lo[n] is strictly less
    if n > 1 {
        if islt(lt, lo[0], lo[n - 1])? {
            return Ok(n);
        }
        lo[..n].reverse();
    }
    n += 1;
    // Finish descending run.
    let mut neq = 0;
    while n < nremaining {
        if islt(lt, lo[n], lo[n - 1])? {
            reverse_last_neq(lo, n, &mut neq);
        } else if islt(lt, lo[n - 1], lo[n])? {
            // descending run is over
            break;
        } else {
            // not x < y and not y < x implies x == y
            neq += 1;
        }
        n += 1;
    }
    reverse_last_neq(lo, n, &mut neq);
    lo[..n].reverse();
    // extend with a naturally increasing suffix
    while n < nremaining {
        if islt(lt, lo[n], lo[n - 1])? {
            break;
        }
        n += 1;
    }
    Ok(n)
}

/// `gallop_left(ms, key, a, n, hint)` with `n = a.len()`.
fn gallop_left<F: FnMut(usize, usize) -> bool>(
    lt: &mut F,
    key: usize,
    a: &[usize],
    hint: usize,
) -> SortResult<usize> {
    let n = a.len() as isize;
    let hint = hint as isize;
    let at = |i: isize| a[i as usize];
    let mut lastofs: isize = 0;
    let mut ofs: isize = 1;
    if islt(lt, at(hint), key)? {
        // a[hint] < key -- gallop right
        let maxofs = n - hint;
        while ofs < maxofs {
            if islt(lt, at(hint + ofs), key)? {
                lastofs = ofs;
                ofs = (ofs << 1) + 1;
            } else {
                break;
            }
        }
        if ofs > maxofs {
            ofs = maxofs;
        }
        lastofs += hint;
        ofs += hint;
    } else {
        // key <= a[hint] -- gallop left
        let maxofs = hint + 1;
        while ofs < maxofs {
            if islt(lt, at(hint - ofs), key)? {
                break;
            }
            lastofs = ofs;
            ofs = (ofs << 1) + 1;
        }
        if ofs > maxofs {
            ofs = maxofs;
        }
        let k = lastofs;
        lastofs = hint - ofs;
        ofs = hint - k;
    }
    lastofs += 1;
    while lastofs < ofs {
        let m = lastofs + ((ofs - lastofs) >> 1);
        if islt(lt, at(m), key)? {
            lastofs = m + 1;
        } else {
            ofs = m;
        }
    }
    Ok(ofs as usize)
}

/// `gallop_right(ms, key, a, n, hint)` with `n = a.len()`.
fn gallop_right<F: FnMut(usize, usize) -> bool>(
    lt: &mut F,
    key: usize,
    a: &[usize],
    hint: usize,
) -> SortResult<usize> {
    let n = a.len() as isize;
    let hint = hint as isize;
    let at = |i: isize| a[i as usize];
    let mut lastofs: isize = 0;
    let mut ofs: isize = 1;
    if islt(lt, key, at(hint))? {
        // key < a[hint] -- gallop left
        let maxofs = hint + 1;
        while ofs < maxofs {
            if islt(lt, key, at(hint - ofs))? {
                lastofs = ofs;
                ofs = (ofs << 1) + 1;
            } else {
                break;
            }
        }
        if ofs > maxofs {
            ofs = maxofs;
        }
        let k = lastofs;
        lastofs = hint - ofs;
        ofs = hint - k;
    } else {
        // a[hint] <= key -- gallop right
        let maxofs = n - hint;
        while ofs < maxofs {
            if islt(lt, key, at(hint + ofs))? {
                break;
            }
            lastofs = ofs;
            ofs = (ofs << 1) + 1;
        }
        if ofs > maxofs {
            ofs = maxofs;
        }
        lastofs += hint;
        ofs += hint;
    }
    lastofs += 1;
    while lastofs < ofs {
        let m = lastofs + ((ofs - lastofs) >> 1);
        if islt(lt, key, at(m))? {
            ofs = m;
        } else {
            lastofs = m + 1;
        }
    }
    Ok(ofs as usize)
}

/// A pending run: `struct s_slice`.
struct Run {
    base: usize,
    len: usize,
    power: i32,
}

/// `MergeState` (the arrays are passed alongside it).
struct MergeState {
    min_gallop: isize,
    pending: Vec<Run>,
    listlen: usize,
}

/// Where `merge_lo` / `merge_hi` leave their main loop.
enum Exit {
    Succeed,
    Fail(Panic),
    /// `CopyB` in `merge_lo`, `CopyA` in `merge_hi`.
    Copy,
}

fn ix(i: isize) -> usize {
    i as usize
}

impl MergeState {
    /// `merge_lo(ms, ssa, na, ssb, nb)`: `keys[ssa..ssa+na]` and
    /// `keys[ssb..ssb+nb]`, `ssb == ssa + na`, `na <= nb`.
    fn merge_lo<F: FnMut(usize, usize) -> bool>(
        &mut self,
        lt: &mut F,
        keys: &mut [usize],
        ssa: usize,
        mut na: usize,
        ssb: usize,
        mut nb: usize,
    ) -> SortResult<()> {
        // ms->a holds a copy of run a; dest walks the list.
        let tmp: Vec<usize> = keys[ssa..ssa + na].to_vec();
        let mut dest = ssa;
        let mut pa = 0usize;
        let mut pb = ssb;
        let exit = 'body: {
            keys[dest] = keys[pb];
            dest += 1;
            pb += 1;
            nb -= 1;
            if nb == 0 {
                break 'body Exit::Succeed;
            }
            if na == 1 {
                break 'body Exit::Copy;
            }
            let mut min_gallop = self.min_gallop;
            loop {
                let mut acount: isize = 0; // # of times A won in a row
                let mut bcount: isize = 0; // # of times B won in a row
                loop {
                    match islt(lt, keys[pb], tmp[pa]) {
                        Err(p) => break 'body Exit::Fail(p),
                        Ok(true) => {
                            keys[dest] = keys[pb];
                            dest += 1;
                            pb += 1;
                            bcount += 1;
                            acount = 0;
                            nb -= 1;
                            if nb == 0 {
                                break 'body Exit::Succeed;
                            }
                            if bcount >= min_gallop {
                                break;
                            }
                        }
                        Ok(false) => {
                            keys[dest] = tmp[pa];
                            dest += 1;
                            pa += 1;
                            acount += 1;
                            bcount = 0;
                            na -= 1;
                            if na == 1 {
                                break 'body Exit::Copy;
                            }
                            if acount >= min_gallop {
                                break;
                            }
                        }
                    }
                }
                min_gallop += 1;
                loop {
                    min_gallop -= (min_gallop > 1) as isize;
                    self.min_gallop = min_gallop;
                    let k = match gallop_right(lt, keys[pb], &tmp[pa..pa + na], 0) {
                        Ok(k) => k,
                        Err(p) => break 'body Exit::Fail(p),
                    };
                    acount = k as isize;
                    if k > 0 {
                        keys[dest..dest + k].copy_from_slice(&tmp[pa..pa + k]);
                        dest += k;
                        pa += k;
                        na -= k;
                        if na == 1 {
                            break 'body Exit::Copy;
                        }
                        // na==0 is impossible now if the comparison function
                        // is consistent, but we can't assume that it is.
                        if na == 0 {
                            break 'body Exit::Succeed;
                        }
                    }
                    keys[dest] = keys[pb];
                    dest += 1;
                    pb += 1;
                    nb -= 1;
                    if nb == 0 {
                        break 'body Exit::Succeed;
                    }
                    let k = match gallop_left(lt, tmp[pa], &keys[pb..pb + nb], 0) {
                        Ok(k) => k,
                        Err(p) => break 'body Exit::Fail(p),
                    };
                    bcount = k as isize;
                    if k > 0 {
                        keys.copy_within(pb..pb + k, dest);
                        dest += k;
                        pb += k;
                        nb -= k;
                        if nb == 0 {
                            break 'body Exit::Succeed;
                        }
                    }
                    keys[dest] = tmp[pa];
                    dest += 1;
                    pa += 1;
                    na -= 1;
                    if na == 1 {
                        break 'body Exit::Copy;
                    }
                    if !(acount >= MIN_GALLOP || bcount >= MIN_GALLOP) {
                        break;
                    }
                }
                min_gallop += 1; // penalize it for leaving galloping mode
                self.min_gallop = min_gallop;
            }
        };
        match exit {
            Exit::Copy => {
                // CopyB: the last element of ssa belongs at the end.
                keys.copy_within(pb..pb + nb, dest);
                keys[dest + nb] = tmp[pa];
                Ok(())
            }
            Exit::Succeed | Exit::Fail(_) => {
                // Succeed falls through to Fail's copy-back.
                if na > 0 {
                    keys[dest..dest + na].copy_from_slice(&tmp[pa..pa + na]);
                }
                match exit {
                    Exit::Fail(p) => Err(p),
                    _ => Ok(()),
                }
            }
        }
    }

    /// `merge_hi(ms, ssa, na, ssb, nb)`: like `merge_lo`, `na >= nb`.
    fn merge_hi<F: FnMut(usize, usize) -> bool>(
        &mut self,
        lt: &mut F,
        keys: &mut [usize],
        ssa: usize,
        mut na: usize,
        ssb: usize,
        mut nb: usize,
    ) -> SortResult<()> {
        // ms->a (baseb) holds a copy of run b; pb and pa point at the last
        // elements of the runs, dest at the last slot of the list.
        let tmp: Vec<usize> = keys[ssb..ssb + nb].to_vec();
        let basea = ssa;
        let mut dest = (ssb + nb - 1) as isize;
        let mut pb = nb as isize - 1;
        let mut pa = (ssa + na - 1) as isize;
        let exit = 'body: {
            keys[ix(dest)] = keys[ix(pa)];
            dest -= 1;
            pa -= 1;
            na -= 1;
            if na == 0 {
                break 'body Exit::Succeed;
            }
            if nb == 1 {
                break 'body Exit::Copy;
            }
            let mut min_gallop = self.min_gallop;
            loop {
                let mut acount: isize = 0;
                let mut bcount: isize = 0;
                loop {
                    match islt(lt, tmp[ix(pb)], keys[ix(pa)]) {
                        Err(p) => break 'body Exit::Fail(p),
                        Ok(true) => {
                            keys[ix(dest)] = keys[ix(pa)];
                            dest -= 1;
                            pa -= 1;
                            acount += 1;
                            bcount = 0;
                            na -= 1;
                            if na == 0 {
                                break 'body Exit::Succeed;
                            }
                            if acount >= min_gallop {
                                break;
                            }
                        }
                        Ok(false) => {
                            keys[ix(dest)] = tmp[ix(pb)];
                            dest -= 1;
                            pb -= 1;
                            bcount += 1;
                            acount = 0;
                            nb -= 1;
                            if nb == 1 {
                                break 'body Exit::Copy;
                            }
                            if bcount >= min_gallop {
                                break;
                            }
                        }
                    }
                }
                min_gallop += 1;
                loop {
                    min_gallop -= (min_gallop > 1) as isize;
                    self.min_gallop = min_gallop;
                    let k = match gallop_right(lt, tmp[ix(pb)], &keys[basea..basea + na], na - 1) {
                        Ok(k) => na - k,
                        Err(p) => break 'body Exit::Fail(p),
                    };
                    acount = k as isize;
                    if k > 0 {
                        dest -= k as isize;
                        pa -= k as isize;
                        keys.copy_within(ix(pa + 1)..ix(pa + 1) + k, ix(dest + 1));
                        na -= k;
                        if na == 0 {
                            break 'body Exit::Succeed;
                        }
                    }
                    keys[ix(dest)] = tmp[ix(pb)];
                    dest -= 1;
                    pb -= 1;
                    nb -= 1;
                    if nb == 1 {
                        break 'body Exit::Copy;
                    }
                    let k = match gallop_left(lt, keys[ix(pa)], &tmp[..nb], nb - 1) {
                        Ok(k) => nb - k,
                        Err(p) => break 'body Exit::Fail(p),
                    };
                    bcount = k as isize;
                    if k > 0 {
                        dest -= k as isize;
                        pb -= k as isize;
                        keys[ix(dest + 1)..ix(dest + 1) + k]
                            .copy_from_slice(&tmp[ix(pb + 1)..ix(pb + 1) + k]);
                        nb -= k;
                        if nb == 1 {
                            break 'body Exit::Copy;
                        }
                        // nb==0 is impossible now if the comparison function
                        // is consistent, but we can't assume that it is.
                        if nb == 0 {
                            break 'body Exit::Succeed;
                        }
                    }
                    keys[ix(dest)] = keys[ix(pa)];
                    dest -= 1;
                    pa -= 1;
                    na -= 1;
                    if na == 0 {
                        break 'body Exit::Succeed;
                    }
                    if !(acount >= MIN_GALLOP || bcount >= MIN_GALLOP) {
                        break;
                    }
                }
                min_gallop += 1; // penalize it for leaving galloping mode
                self.min_gallop = min_gallop;
            }
        };
        match exit {
            Exit::Copy => {
                // CopyA: the first element of ssb belongs at the front.
                let na_i = na as isize;
                keys.copy_within(ix(pa + 1 - na_i)..ix(pa + 1), ix(dest + 1 - na_i));
                dest -= na_i;
                keys[ix(dest)] = tmp[ix(pb)];
                Ok(())
            }
            Exit::Succeed | Exit::Fail(_) => {
                if nb > 0 {
                    let start = ix(dest - (nb as isize - 1));
                    keys[start..start + nb].copy_from_slice(&tmp[..nb]);
                }
                match exit {
                    Exit::Fail(p) => Err(p),
                    _ => Ok(()),
                }
            }
        }
    }

    /// `merge_at(ms, i)`: merges pending runs `i` and `i + 1`.
    fn merge_at<F: FnMut(usize, usize) -> bool>(
        &mut self,
        lt: &mut F,
        keys: &mut [usize],
        i: usize,
    ) -> SortResult<()> {
        let mut ssa = self.pending[i].base;
        let mut na = self.pending[i].len;
        let ssb = self.pending[i + 1].base;
        let nb = self.pending[i + 1].len;
        // Record the length of the combined runs; if i is the 3rd-last run
        // now, also slide over the last run. Run i+1 goes away.
        self.pending[i].len = na + nb;
        self.pending.remove(i + 1);
        // Where does b start in a?
        let k = gallop_right(lt, keys[ssb], &keys[ssa..ssa + na], 0)?;
        ssa += k;
        na -= k;
        if na == 0 {
            return Ok(());
        }
        // Where does a end in b?
        let nb = gallop_left(lt, keys[ssa + na - 1], &keys[ssb..ssb + nb], nb - 1)?;
        if nb == 0 {
            return Ok(());
        }
        if na <= nb {
            self.merge_lo(lt, keys, ssa, na, ssb, nb)
        } else {
            self.merge_hi(lt, keys, ssa, na, ssb, nb)
        }
    }

    /// `found_new_run(ms, n2)`: the powersort merge strategy.
    fn found_new_run<F: FnMut(usize, usize) -> bool>(
        &mut self,
        lt: &mut F,
        keys: &mut [usize],
        n2: usize,
    ) -> SortResult<()> {
        if let Some(top) = self.pending.last() {
            let power = powerloop(top.base, top.len, n2, self.listlen);
            while self.pending.len() > 1 && self.pending[self.pending.len() - 2].power > power {
                self.merge_at(lt, keys, self.pending.len() - 2)?;
            }
            let n = self.pending.len();
            self.pending[n - 1].power = power;
        }
        Ok(())
    }

    /// `merge_force_collapse(ms)`.
    fn merge_force_collapse<F: FnMut(usize, usize) -> bool>(
        &mut self,
        lt: &mut F,
        keys: &mut [usize],
    ) -> SortResult<()> {
        while self.pending.len() > 1 {
            let mut n = self.pending.len() - 2;
            if n > 0 && self.pending[n - 1].len < self.pending[n + 1].len {
                n -= 1;
            }
            self.merge_at(lt, keys, n)?;
        }
        Ok(())
    }
}

/// `powerloop(s1, n1, n2, n)`.
fn powerloop(s1: usize, n1: usize, n2: usize, n: usize) -> i32 {
    let mut result = 0;
    let mut a = 2 * s1 + n1; // 2*a
    let mut b = a + n1 + n2; // 2*b
    loop {
        result += 1;
        if a >= n {
            // both quotient bits are 1
            a -= n;
            b -= n;
        } else if b >= n {
            // a/n bit is 0, b/n bit is 1
            break;
        }
        a <<= 1;
        b <<= 1;
    }
    result
}

/// `merge_compute_minrun(n)`.
fn merge_compute_minrun(mut n: usize) -> usize {
    let mut r = 0; // becomes 1 if any 1 bits are shifted off
    while n >= MAX_MINRUN {
        r |= n & 1;
        n >>= 1;
    }
    n + r
}

/// The body of `list_sort_impl` from the reversal to `merge_force_collapse`.
fn sort_runs<F: FnMut(usize, usize) -> bool>(lt: &mut F, keys: &mut [usize]) -> SortResult<()> {
    let n = keys.len();
    let mut ms = MergeState {
        min_gallop: MIN_GALLOP,
        pending: Vec::new(),
        listlen: n,
    };
    let minrun = merge_compute_minrun(n);
    let mut lo = 0;
    let mut nremaining = n;
    loop {
        // Identify next run.
        let mut r = count_run(lt, &mut keys[lo..lo + nremaining])?;
        // If short, extend to min(minrun, nremaining).
        if r < minrun {
            let force = nremaining.min(minrun);
            binarysort(lt, &mut keys[lo..lo + force], r)?;
            r = force;
        }
        // Maybe merge pending runs.
        ms.found_new_run(lt, keys, r)?;
        // Push new run on stack.
        ms.pending.push(Run {
            base: lo,
            len: r,
            power: 0,
        });
        // Advance to find next run.
        lo += r;
        nremaining -= r;
        if nremaining == 0 {
            break;
        }
    }
    ms.merge_force_collapse(lt, keys)
}

/// `list.sort(reverse=reverse)` on `n` items compared by index.
fn list_sort_perm<F: FnMut(usize, usize) -> bool>(
    n: usize,
    mut lt: F,
    reverse: bool,
) -> (Vec<usize>, Option<Panic>) {
    let mut keys: Vec<usize> = (0..n).collect();
    if n < 2 {
        return (keys, None);
    }
    // Reverse sort stability achieved by initially reversing the list,
    // applying a stable forward sort, then reversing the final result.
    if reverse {
        keys.reverse();
    }
    let result = sort_runs(&mut lt, &mut keys);
    if reverse {
        keys.reverse();
    }
    (keys, result.err())
}

/// Puts `v[perm[k]]` at position `k`.
fn apply_permutation<T>(v: &mut [T], mut perm: Vec<usize>) {
    for k in 0..perm.len() {
        if perm[k] == k {
            continue;
        }
        let mut j = k;
        loop {
            let next = perm[j];
            perm[j] = j;
            if next == k {
                break;
            }
            v.swap(j, next);
            j = next;
        }
    }
}

fn list_sort<T>(v: &mut [T], mut lt: impl FnMut(&T, &T) -> bool, reverse: bool) {
    let (perm, err) = {
        let items: &[T] = v;
        list_sort_perm(items.len(), |i, j| lt(&items[i], &items[j]), reverse)
    };
    apply_permutation(v, perm);
    if let Some(p) = err {
        panic::resume_unwind(p);
    }
}

/// CPython's `list.sort()` with a `<` predicate (stable, and with the same
/// result as CPython even when the predicate is not a total order). When
/// `lt` raises, the slice is left in the permutation CPython leaves the
/// list in and the error propagates.
pub fn py_sort_by<T>(v: &mut [T], lt: impl FnMut(&T, &T) -> bool) {
    list_sort(v, lt, false);
}

/// `list.sort(reverse=True)` with a `<` predicate.
pub fn py_sort_by_reverse<T>(v: &mut [T], lt: impl FnMut(&T, &T) -> bool) {
    list_sort(v, lt, true);
}

/// `list.sort(key=key, reverse=reverse)` where `keys[i]` is `key(v[i])`
/// (CPython computes every key before sorting).
fn sort_with_keys<T, K>(
    v: &mut Vec<T>,
    keys: Vec<K>,
    mut lt: impl FnMut(&K, &K) -> bool,
    reverse: bool,
) {
    let mut pairs: Vec<(K, T)> = keys.into_iter().zip(v.drain(..)).collect();
    let r = catch(|| list_sort(&mut pairs, |a, b| lt(&a.0, &b.0), reverse));
    v.extend(pairs.into_iter().map(|(_, t)| t));
    if let Err(e) = r {
        reraise(e);
    }
}

// ===========================================================================
// Basic.compare
// ===========================================================================

/// One element of `_hashable_content()`.
#[derive(Clone, Debug)]
enum Hc {
    Basic(Expr),
    Int(Int),
    Str(String),
    Bool(bool),
    Tuple(Vec<Hc>),
}

/// `Symbol._hashable_content`: `(self.name,) + self._assumptions0`, where
/// `_assumptions0 = tuple(sorted(assumptions0.items()))`.
fn symbol_hashable_content(e: &Expr) -> Vec<Hc> {
    let d = e.symbol_data().expect("a Symbol");
    let mut items: Vec<(&str, bool)> = d
        .assumptions0
        .items(false)
        .into_iter()
        .map(|(f, v)| (f.name(), v.expect("stored facts are True or False")))
        .collect();
    items.sort();
    let mut out = vec![Hc::Str(d.name.to_string())];
    out.extend(
        items
            .into_iter()
            .map(|(k, v)| Hc::Tuple(vec![Hc::Str(k.to_string()), Hc::Bool(v)])),
    );
    out
}

/// `self._hashable_content()`.
fn hashable_content(e: &Expr) -> Vec<Hc> {
    match info(e).hashable_content {
        "Basic._hashable_content" | "Expr._hashable_content" => {
            e.args().iter().cloned().map(Hc::Basic).collect()
        }
        "Rational._hashable_content" => vec![Hc::Int(e.p()), Hc::Int(e.q())],
        "Float._hashable_content" => {
            let (m, prec) = e.as_float().expect("a Float");
            let mpf = Hc::Tuple(vec![
                Hc::Int(Int::from(m.sign as i64)),
                Hc::Int(m.man.clone()),
                Hc::Int(Int::from(m.exp)),
                Hc::Int(Int::from(m.bc)),
            ]);
            vec![mpf, Hc::Int(Int::from(prec))]
        }
        "Symbol._hashable_content" => symbol_hashable_content(e),
        "Dummy._hashable_content" => {
            let mut out = symbol_hashable_content(e);
            let i = e.symbol_data().and_then(|d| d.dummy_index).expect("index");
            out.push(Hc::Int(Int::from(i)));
            out
        }
        "Wild._hashable_content" => {
            // + (self.exclude, self.properties), both () here.
            let mut out = symbol_hashable_content(e);
            out.push(Hc::Tuple(vec![]));
            out.push(Hc::Tuple(vec![]));
            out
        }
        "Str._hashable_content" => vec![Hc::Str(e.name().expect("Str.name").to_string())],
        other => not_ported(other),
    }
}

/// Python's `(l > r) - (l < r)` on non-Basic hashable-content items.
fn py_cmp(l: &Hc, r: &Hc) -> Ordering {
    fn int_of(h: &Hc) -> Option<Int> {
        match h {
            Hc::Int(i) => Some(i.clone()),
            Hc::Bool(b) => Some(Int::from(*b as i64)),
            _ => None,
        }
    }
    match (l, r) {
        (Hc::Str(a), Hc::Str(b)) => a.cmp(b),
        (Hc::Tuple(a), Hc::Tuple(b)) => {
            // tuple_richcompare: the first items that differ decide.
            for (x, y) in a.iter().zip(b) {
                let c = match x {
                    Hc::Basic(_) => not_ported("Basic.compare: a Basic inside a tuple"),
                    _ => py_cmp(x, y),
                };
                if c != Ordering::Equal {
                    return c;
                }
            }
            a.len().cmp(&b.len())
        }
        (_, Hc::Basic(_)) => not_ported("Basic.compare: a Python value against a Basic"),
        _ => match (int_of(l), int_of(r)) {
            (Some(a), Some(b)) => a.cmp(&b),
            _ => raise(
                ErrorKind::TypeError,
                "'>' not supported between these hashable-content items",
            ),
        },
    }
}

/// `l.compare(r)` for a Basic `l` and a non-Basic `r`: `_cmp_name` sees a
/// non-Basic class and returns -1.
fn compare_with_non_basic(l: &Expr) -> Ordering {
    if info(l).compare != "Basic.compare" {
        not_ported(info(l).compare);
    }
    Ordering::Less
}

/// `a.compare(b)`: the `compare` method `a`'s class has.
pub fn compare_method(a: &Expr, b: &Expr) -> Ordering {
    match info(a).compare {
        "Basic.compare" => compare(a, b),
        // FiniteSet.compare returns hash(self) - hash(other), which depends
        // on Python's string hashing.
        other => not_ported(format!("{other} (hash-based)")),
    }
}

/// `Basic.compare(a, b)`.
pub fn compare(a: &Expr, b: &Expr) -> Ordering {
    // all redefinitions of __cmp__ method should start with the
    // following lines:
    if a.ptr_eq(b) {
        return Ordering::Equal;
    }
    let c = cmp_name(a.class_name(), b.class_name());
    if c != Ordering::Equal {
        return c;
    }
    let st = hashable_content(a);
    let ot = hashable_content(b);
    let c = st.len().cmp(&ot.len());
    if c != Ordering::Equal {
        return c;
    }
    for (l, r) in st.iter().zip(&ot) {
        let c = match (l, r) {
            (Hc::Basic(l), Hc::Basic(r)) => compare_method(l, r),
            (Hc::Basic(l), _) => compare_with_non_basic(l),
            _ => py_cmp(l, r),
        };
        if c != Ordering::Equal {
            return c;
        }
    }
    Ordering::Equal
}

/// `_cmp_name(a, b)` on the class names of two Basic classes.
pub fn cmp_name(a: &str, b: &str) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }
    let unknown = ORDERING_OF_CLASSES.len() + 1;
    let index = |n: &str| {
        ORDERING_OF_CLASSES
            .iter()
            .position(|c| *c == n)
            .unwrap_or(unknown)
    };
    let (i1, i2) = (index(a), index(b));
    if i1 == unknown && i2 == unknown {
        return a.cmp(b);
    }
    i1.cmp(&i2)
}

// ===========================================================================
// sort keys
// ===========================================================================

/// A `sort_key()` value: SymPy's nested tuples of ints, strings and
/// numbers, compared like Python tuples.
#[derive(Clone, Debug)]
pub enum SortKey {
    Int(i64),
    Str(String),
    /// A SymPy Number (the coefficient slot), compared with SymPy's `<`.
    Num(Expr),
    Tuple(Vec<SortKey>),
    /// A Python bool (`as_ordered_terms`' term keys).
    Bool(bool),
    /// A Python float (`as_ordered_terms`' term keys).
    Float(f64),
}

fn tup(v: Vec<SortKey>) -> SortKey {
    SortKey::Tuple(v)
}

fn py_int(i: &Int) -> i64 {
    i.as_i64()
        .unwrap_or_else(|| not_ported("sort key: an int beyond i64"))
}

/// `cls.class_key()` as its three entries.
pub fn class_key(e: &Expr) -> (i32, i32, String) {
    if e.is_cls(C::AppliedUndef) {
        // Function.class_key on the UndefinedFunction
        return crate::functions::head_of(e).class_key();
    }
    let (a, b, n) = info(e).class_key;
    (a, b, n.to_string())
}

fn class_key_value(e: &Expr) -> SortKey {
    let (a, b, n) = class_key(e);
    tup(vec![
        SortKey::Int(a as i64),
        SortKey::Int(b as i64),
        SortKey::Str(n),
    ])
}

/// `S.One.sort_key()`.
fn one_sort_key() -> SortKey {
    sort_key(&s::one(), None)
}

/// `expr.sort_key(order)`.
pub fn sort_key(e: &Expr, order: Option<&str>) -> SortKey {
    match info(e).sort_key {
        // Number.sort_key: self.class_key(), (0, ()), (), self
        "Number.sort_key" => tup(vec![
            class_key_value(e),
            tup(vec![SortKey::Int(0), tup(vec![])]),
            tup(vec![]),
            SortKey::Num(e.clone()),
        ]),
        // Atom.sort_key: self.class_key(), (1, (str(self),)),
        // S.One.sort_key(), S.One
        "Atom.sort_key" => tup(vec![
            class_key_value(e),
            tup(vec![
                SortKey::Int(1),
                tup(vec![SortKey::Str(crate::printing::str::sstr(e))]),
            ]),
            one_sort_key(),
            SortKey::Num(s::one()),
        ]),
        // Symbol.sort_key: self.class_key(), (1, (self.name,)),
        // S.One.sort_key(), S.One
        "Symbol.sort_key" => tup(vec![
            class_key_value(e),
            tup(vec![
                SortKey::Int(1),
                tup(vec![SortKey::Str(e.name().expect("a name").to_string())]),
            ]),
            one_sort_key(),
            SortKey::Num(s::one()),
        ]),
        // Dummy.sort_key: self.class_key(), (2, (self.name,
        // self.dummy_index)), S.One.sort_key(), S.One
        "Dummy.sort_key" => {
            let d = e.symbol_data().expect("a Dummy");
            let index = d.dummy_index.expect("a dummy_index");
            let index = i64::try_from(index)
                .unwrap_or_else(|_| not_ported("Dummy.sort_key: dummy_index beyond i64"));
            tup(vec![
                class_key_value(e),
                tup(vec![
                    SortKey::Int(2),
                    tup(vec![SortKey::Str(d.name.to_string()), SortKey::Int(index)]),
                ]),
                one_sort_key(),
                SortKey::Num(s::one()),
            ])
        }
        "Expr.sort_key" => expr_sort_key(e, order),
        "Basic.sort_key" => basic_sort_key(e, order),
        other => not_ported(other),
    }
}

/// `Expr.sort_key(order)`.
fn expr_sort_key(e: &Expr, order: Option<&str>) -> SortKey {
    let (coeff, mut expr) = e.as_coeff_Mul(false);
    let exp;
    if expr.is_Pow() {
        let (mut base, mut ex) = expr.as_base_exp();
        if base.is_cls(C::Exp1) {
            // If we remove this, many doctests will go crazy:
            // (keeps E**x sorted like the exp(x) function,
            //  part of exp(x) to E**x transition)
            let f = crate::functions::undefined_function("exp", &[]);
            base = crate::functions::call_undefined(&f, vec![ex]);
            ex = s::one();
        }
        expr = base;
        exp = ex;
    } else {
        exp = s::one();
    }
    let args: Vec<SortKey> = if expr.is_Dummy() {
        vec![sort_key(&expr, None)]
    } else if expr.is_Atom() {
        vec![SortKey::Str(crate::printing::str::sstr(&expr))]
    } else {
        let args = if expr.is_Add() {
            as_ordered_terms(&expr, order)
        } else if expr.is_Mul() {
            as_ordered_factors(&expr, order)
        } else {
            expr.args().to_vec()
        };
        args.iter().map(|a| default_sort_key(a, order)).collect()
    };
    let args = tup(vec![SortKey::Int(args.len() as i64), tup(args)]);
    let exp = sort_key(&exp, order);
    tup(vec![class_key_value(&expr), args, exp, SortKey::Num(coeff)])
}

/// `self._sorted_args`.
fn sorted_args(e: &Expr) -> Vec<Expr> {
    match info(e).sorted_args {
        "Basic._sorted_args" | "FiniteSet._sorted_args" => e.args().to_vec(),
        // tuple(sorted(self.args, key=default_sort_key))
        "Add._sorted_args" => {
            let mut v = e.args().to_vec();
            sorted_default(&mut v);
            v
        }
        // tuple(self.as_ordered_factors())
        "Mul._sorted_args" => as_ordered_factors(e, None),
        "Atom._sorted_args" => raise(
            ErrorKind::AttributeError,
            "Atoms have no args. It might be necessary to make a check for Atoms in the \
             calling code.",
        ),
        other => not_ported(other),
    }
}

/// `Basic.sort_key(order)`.
fn basic_sort_key(e: &Expr, order: Option<&str>) -> SortKey {
    let args = sorted_args(e);
    let keys: Vec<SortKey> = args.iter().map(|a| sort_key(a, order)).collect();
    tup(vec![
        class_key_value(e),
        tup(vec![SortKey::Int(keys.len() as i64), tup(keys)]),
        one_sort_key(),
        SortKey::Num(s::one()),
    ])
}

/// `default_sort_key(item, order)` for an expression.
pub fn default_sort_key(e: &Expr, order: Option<&str>) -> SortKey {
    sort_key(e, order)
}

/// A Python int, bool or float in a key, for numeric comparison.
enum PyNum {
    Int(i64),
    Float(f64),
}

fn py_num(k: &SortKey) -> Option<PyNum> {
    match k {
        SortKey::Int(i) => Some(PyNum::Int(*i)),
        SortKey::Bool(b) => Some(PyNum::Int(*b as i64)),
        SortKey::Float(f) => Some(PyNum::Float(*f)),
        _ => None,
    }
}

/// Python's exact comparison of an int with a float (`None` for NaN).
fn cmp_int_float(i: i64, f: f64) -> Option<Ordering> {
    if f.is_nan() {
        return None;
    }
    if f >= 9_223_372_036_854_775_808.0 {
        return Some(Ordering::Less);
    }
    if f < -9_223_372_036_854_775_808.0 {
        return Some(Ordering::Greater);
    }
    let fl = f.floor();
    let fi = fl as i64;
    Some(match i.cmp(&fi) {
        Ordering::Equal if f > fl => Ordering::Less,
        c => c,
    })
}

/// Python's comparison of two numbers (`None` when unordered: NaN).
fn py_num_cmp(a: &PyNum, b: &PyNum) -> Option<Ordering> {
    match (a, b) {
        (PyNum::Int(x), PyNum::Int(y)) => Some(x.cmp(y)),
        (PyNum::Float(x), PyNum::Float(y)) => x.partial_cmp(y),
        (PyNum::Int(x), PyNum::Float(y)) => cmp_int_float(*x, *y),
        (PyNum::Float(x), PyNum::Int(y)) => cmp_int_float(*y, *x).map(Ordering::reverse),
    }
}

/// Python's `a == b` on sort keys.
pub fn key_eq(a: &SortKey, b: &SortKey) -> bool {
    use SortKey as K;
    match (a, b) {
        // tuple_richcompare: different lengths are unequal at once, then
        // items are compared with == until one differs.
        (K::Tuple(x), K::Tuple(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| key_eq(p, q))
        }
        (K::Str(x), K::Str(y)) => x == y,
        (K::Num(x), K::Num(y)) => x.ptr_eq(y) || numbers::number_eq(x, y),
        (K::Num(_), _) | (_, K::Num(_)) => {
            not_ported("sort key: a SymPy Number compared with a Python value")
        }
        (K::Str(_) | K::Tuple(_), _) | (_, K::Str(_) | K::Tuple(_)) => false,
        _ => {
            let (x, y) = (py_num(a).expect("number"), py_num(b).expect("number"));
            py_num_cmp(&x, &y) == Some(Ordering::Equal)
        }
    }
}

/// Python's `a < b` on sort keys (`TypeError` on mismatched types).
pub fn key_lt(a: &SortKey, b: &SortKey) -> bool {
    use SortKey as K;
    match (a, b) {
        (K::Tuple(x), K::Tuple(y)) => {
            // The first index where the items differ decides; if there is
            // none, the shorter tuple is smaller.
            for (p, q) in x.iter().zip(y) {
                if !key_eq(p, q) {
                    return key_lt(p, q);
                }
            }
            x.len() < y.len()
        }
        (K::Str(x), K::Str(y)) => x < y,
        (K::Num(x), K::Num(y)) => numbers::number_lt(x, y),
        (K::Num(_), _) | (_, K::Num(_)) => {
            not_ported("sort key: a SymPy Number compared with a Python value")
        }
        _ => match (py_num(a), py_num(b)) {
            (Some(x), Some(y)) => py_num_cmp(&x, &y) == Some(Ordering::Less),
            _ => raise(
                ErrorKind::TypeError,
                "'<' not supported between instances of different types",
            ),
        },
    }
}

/// `args.sort(key=_args_sortkey)`: sort by `Basic.compare`
/// (`cmp_to_key(Basic.compare)`).
pub fn sort_args(v: &mut [Expr]) {
    py_sort_by(v, |a, b| compare(a, b) == Ordering::Less);
}

/// `sorted(seq, key=default_sort_key)`.
pub fn sorted_default(v: &mut [Expr]) {
    let keys: Vec<(SortKey, Expr)> = v
        .iter()
        .map(|e| (default_sort_key(e, None), e.clone()))
        .collect();
    let mut keys = keys;
    py_sort_by(&mut keys, |a, b| key_lt(&a.0, &b.0));
    for (slot, (_, e)) in v.iter_mut().zip(keys) {
        *slot = e;
    }
}

// ===========================================================================
// as_ordered_factors, as_ordered_terms, as_terms
// ===========================================================================

/// `expr.as_ordered_factors(order)`.
pub fn as_ordered_factors(e: &Expr, order: Option<&str>) -> Vec<Expr> {
    match info(e).as_ordered_factors {
        // Return list of ordered factors (if Mul) else [self].
        "Expr.as_ordered_factors" => vec![e.clone()],
        "Mul.as_ordered_factors" => {
            let (mut cpart, ncpart) = e.args_cnc(true);
            let keys = cpart.iter().map(|x| sort_key(x, order)).collect();
            sort_with_keys(&mut cpart, keys, key_lt, false);
            cpart.extend(ncpart);
            cpart
        }
        "" => raise(
            ErrorKind::AttributeError,
            format!(
                "'{}' object has no attribute 'as_ordered_factors'",
                e.class_name()
            ),
        ),
        other => not_ported(other),
    }
}

/// A monomial order from `sympy.polys.orderings`.
#[derive(Clone)]
enum MonomialOrder {
    Lex,
    GradedLex,
    ReversedGradedLex,
    Inverse(Box<MonomialOrder>),
}

/// `monomial_key(order)` for a string (or no) order.
fn monomial_key(order: Option<&str>) -> MonomialOrder {
    use MonomialOrder as M;
    match order {
        None | Some("lex") => M::Lex,
        Some("grlex") => M::GradedLex,
        Some("grevlex") => M::ReversedGradedLex,
        Some("ilex") => M::Inverse(Box::new(M::Lex)),
        Some("igrlex") => M::Inverse(Box::new(M::GradedLex)),
        Some("igrevlex") => M::Inverse(Box::new(M::ReversedGradedLex)),
        Some(other) => raise(
            ErrorKind::ValueError,
            format!("supported monomial orderings are 'lex', 'grlex' and 'grevlex', got {other:?}"),
        ),
    }
}

fn checked(v: Option<i64>) -> i64 {
    v.unwrap_or_else(|| not_ported("monomial key: an exponent beyond i64"))
}

fn ints(m: impl IntoIterator<Item = i64>) -> SortKey {
    tup(m.into_iter().map(SortKey::Int).collect())
}

impl MonomialOrder {
    /// `order(monomial)`.
    fn call(&self, m: &[i64]) -> SortKey {
        use MonomialOrder as M;
        let sum = || checked(m.iter().try_fold(0i64, |a, &b| a.checked_add(b)));
        match self {
            M::Lex => ints(m.iter().copied()),
            M::GradedLex => tup(vec![SortKey::Int(sum()), ints(m.iter().copied())]),
            M::ReversedGradedLex => tup(vec![
                SortKey::Int(sum()),
                ints(m.iter().rev().map(|x| checked(x.checked_neg()))),
            ]),
            M::Inverse(o) => negate(&o.call(m)),
        }
    }
}

/// `InverseOrder`'s `inv` and `_parse_order`'s `neg`: negates every int,
/// recursing into tuples.
fn negate(k: &SortKey) -> SortKey {
    match k {
        SortKey::Tuple(v) => tup(v.iter().map(negate).collect()),
        SortKey::Int(i) => SortKey::Int(checked(i.checked_neg())),
        _ => unreachable!("monomial keys hold ints"),
    }
}

/// One entry of `as_terms`' result: `(term, ((re, im), monom, ncpart))`.
struct TermData {
    term: Expr,
    coeff: (f64, f64),
    monom: Vec<i64>,
    ncpart: Vec<Expr>,
}

/// `Expr._parse_order(order)`'s key applied to a term.
fn term_key(monom_key: &MonomialOrder, order: Option<&str>, t: &TermData) -> SortKey {
    let (re, im) = t.coeff;
    let monom = negate(&monom_key.call(&t.monom));
    let ncpart = tup(t.ncpart.iter().map(|e| sort_key(e, order)).collect());
    let coeff = tup(vec![
        tup(vec![SortKey::Bool(im != 0.0), SortKey::Float(im)]),
        tup(vec![SortKey::Float(re), SortKey::Float(im)]),
    ]);
    tup(vec![monom, ncpart, coeff])
}

fn is_number_or_number_symbol(x: &Expr) -> bool {
    x.is_a(Tag::Number) || x.is_a(Tag::NumberSymbol)
}

/// `sorted(seq, key=lambda x: not isinstance(x, (Number, NumberSymbol)))`.
fn numbers_first(mut v: Vec<Expr>) -> Vec<Expr> {
    let keys: Vec<bool> = v.iter().map(|x| !is_number_or_number_symbol(x)).collect();
    sort_with_keys(&mut v, keys, |a, b| !*a && *b, false);
    v
}

/// `expr.as_ordered_terms(order)` (an Add's terms in printing order).
pub fn as_ordered_terms(e: &Expr, order: Option<&str>) -> Vec<Expr> {
    match info(e).as_ordered_terms {
        "Expr.as_ordered_terms" => expr_as_ordered_terms(e, order),
        "" => raise(
            ErrorKind::AttributeError,
            format!(
                "'{}' object has no attribute 'as_ordered_terms'",
                e.class_name()
            ),
        ),
        other => not_ported(other),
    }
}

/// `Expr.as_ordered_terms(order, data=False)`.
fn expr_as_ordered_terms(e: &Expr, order: Option<&str>) -> Vec<Expr> {
    if order.is_none() && e.is_Add() {
        // Spot the special case of Add(Number, Mul(Number, expr)) with the
        // first number positive and the second number negative
        let add_args = numbers_first(e.add_args());
        if add_args.len() == 2 && is_number_or_number_symbol(&add_args[0]) && add_args[1].is_Mul() {
            let mul_args = numbers_first(add_args[1].mul_args());
            if mul_args.len() == 2
                && mul_args[0].is_a(Tag::Number)
                && add_args[0].is_positive() == Some(true)
                && mul_args[0].is_negative() == Some(true)
            {
                return add_args;
            }
        }
    }
    // key, reverse = self._parse_order(order)
    let (order, reverse) = match order {
        Some(o) => match o.strip_prefix("rev-") {
            Some(rest) => (Some(rest), true),
            None => (Some(o), false),
        },
        None => (None, false),
    };
    let monom_key = monomial_key(order);
    let (terms, _gens) = as_terms(e);
    let sort = |mut ts: Vec<TermData>, reverse: bool| {
        let keys = ts.iter().map(|t| term_key(&monom_key, order, t)).collect();
        sort_with_keys(&mut ts, keys, key_lt, reverse);
        ts
    };
    let ordered = if !terms.iter().any(|t| t.term.is_Order()) {
        sort(terms, reverse)
    } else {
        let (order_terms, others): (Vec<TermData>, Vec<TermData>) =
            terms.into_iter().partition(|t| t.term.is_Order());
        let mut out = sort(others, true);
        out.extend(sort(order_terms, true));
        out
    };
    ordered.into_iter().map(|t| t.term).collect()
}

/// `expr.as_terms()`: each term's complex coefficient, monomial over the
/// sorted generators, and non-commutative factors; and the generators.
fn as_terms(e: &Expr) -> (Vec<TermData>, Vec<Expr>) {
    // gens is a set in SymPy; it is sorted by default_sort_key below, so
    // only generators with equal sort keys depend on the set's order.
    let mut gens: Vec<Expr> = Vec::new();
    type Term = (Expr, (f64, f64), PyDict<Expr, i64>, Vec<Expr>);
    let mut terms: Vec<Term> = Vec::new();
    for term in e.add_args() {
        let (coeff, term_) = term.as_coeff_Mul(false);
        let mut coeff = py_complex(&coeff);
        let mut cpart: PyDict<Expr, i64> = PyDict::new();
        let mut ncpart = Vec::new();
        if !term_.is_cls(C::One) {
            for factor in term_.mul_args() {
                if factor.is_number() {
                    match catch(|| py_complex(&factor)) {
                        Ok(c) => {
                            coeff = complex_mul(coeff, c);
                            continue;
                        }
                        Err(err)
                            if err.kind.is_subclass_of(ErrorKind::TypeError)
                                || err.kind.is_subclass_of(ErrorKind::ValueError) => {}
                        Err(err) => reraise(err),
                    }
                }
                if factor.is_commutative() == Some(true) {
                    let (base, exp) = crate::exprtools::decompose_power(&factor);
                    cpart.insert(base.clone(), exp);
                    if !gens.contains(&base) {
                        gens.push(base);
                    }
                } else {
                    ncpart.push(factor);
                }
            }
        }
        terms.push((term, coeff, cpart, ncpart));
    }
    sorted_default(&mut gens);
    let mut indices: PyDict<Expr, usize> = PyDict::new();
    for (i, g) in gens.iter().enumerate() {
        indices.insert(g.clone(), i);
    }
    let result = terms
        .into_iter()
        .map(|(term, coeff, cpart, ncpart)| {
            let mut monom = vec![0i64; gens.len()];
            for (base, exp) in cpart.iter() {
                monom[*indices.get(base).expect("a generator")] = *exp;
            }
            TermData {
                term,
                coeff,
                monom,
                ncpart,
            }
        })
        .collect();
    (result, gens)
}

/// Python's `complex * complex` (CPython 3.13 `_Py_c_prod`).
fn complex_mul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

// ===========================================================================
// complex() and float()
// ===========================================================================

/// `x.evalf()` with the default 15 digits, for the atoms whose entry in
/// `evalf_table` (or `_eval_evalf`) needs nothing else; other expressions
/// go to `evalf::evalf`.
fn evalf15(x: &Expr) -> Expr {
    // Infinity.evalf and NegativeInfinity.evalf return
    // self._eval_evalf(prec), Float('inf') / Float('-inf'): themselves.
    if x.is_cls(C::Infinity) || x.is_cls(C::NegativeInfinity) {
        return x.clone();
    }
    let prec = mlib::dps_to_prec(15);
    // result = evalf(self, prec + 4, options)
    let wp = prec + 4;
    let fast = mlib::ROUND_FAST;
    type Parts = (
        Option<mlib::Mpf>,
        Option<mlib::Mpf>,
        Option<i64>,
        Option<i64>,
    );
    let real = |m: mlib::Mpf| -> Parts { (Some(m), None, Some(wp), None) };
    let result: Parts = match x.cls() {
        C::Float => real(x.as_float().expect("Float").0.clone()),
        C::Rational => real(mlib::from_rational(&x.p(), &x.q(), wp, fast).expect("q > 0")),
        C::Integer => real(mlib::from_int(&x.p(), wp, fast)),
        C::Zero => (None, None, Some(wp), None),
        C::One => real(mlib::FONE),
        C::Half => real(mlib::FHALF),
        C::Pi => real(mlib::mpf_pi(wp, fast)),
        C::Exp1 => real(mlib::mpf_e(wp, fast)),
        C::ImaginaryUnit => (None, Some(mlib::FONE), None, Some(wp)),
        C::NegativeOne => real(mlib::FNONE),
        C::ComplexInfinity => return s::complex_infinity(),
        C::NaN => real(mlib::FNAN),
        // Not in evalf_table: xe = x._eval_evalf(prec), here
        // NumberSymbol._eval_evalf = Float._new(self._as_mpf_val(prec),
        // prec); then re, im = xe.as_real_imag() is (xe, 0) and
        // re._to_mpmath(prec) is Float._as_mpf_val(prec).
        C::GoldenRatio | C::EulerGamma | C::Catalan => {
            let xe = numbers::float_new(numbers::as_mpf_val(x, wp), wp, true);
            if !xe.is_Float() {
                not_ported("evalf: a NumberSymbol that is not a Float at this precision");
            }
            real(numbers::as_mpf_val(&xe, wp))
        }
        _ => return crate::evalf::evalf(x, 15),
    };
    let (re, im, re_acc, im_acc) = result;
    let re = match re {
        Some(m) => {
            let p = prec.min(re_acc.expect("re_acc")).max(1);
            numbers::float_new(m, p, true)
        }
        None => s::zero(),
    };
    match im {
        Some(m) => {
            let p = prec.min(im_acc.expect("im_acc")).max(1);
            let im = numbers::float_new(m, p, true);
            crate::ops::add(&re, &crate::ops::mul(&im, &s::imaginary_unit()))
        }
        None => re,
    }
}

/// `x.as_real_imag()` (`(re(x), im(x))`) for an evalf result.
fn as_real_imag(x: &Expr) -> (Expr, Expr) {
    if !crate::ctx::evaluate() {
        not_ported("Expr.as_real_imag under evaluate(False)");
    }
    // re.eval / im.eval
    if x.is_cls(C::NaN) || x.is_cls(C::ComplexInfinity) {
        return (s::nan(), s::nan());
    }
    if x.is_Number() && x.is_extended_real() == Some(true) {
        return (x.clone(), s::zero());
    }
    not_ported(format!("as_real_imag of {}", x.class_name()))
}

/// `float(x)` for an expression: `Number.__float__`, else `Expr.__float__`.
pub fn py_float(x: &Expr) -> f64 {
    if x.is_Number() {
        return numbers::number_float(x);
    }
    not_ported(format!("Expr.__float__ of {}", x.class_name()))
}

/// `complex(x)`: `Expr.__complex__`.
pub fn py_complex(x: &Expr) -> (f64, f64) {
    let result = evalf15(x);
    let (re, im) = as_real_imag(&result);
    (py_float(&re), py_float(&im))
}

// ===========================================================================
// ordered
// ===========================================================================

/// `_node_count(e)`.
fn node_count(e: &Expr) -> f64 {
    if e.is_Float() {
        return 0.5;
    }
    1.0 + e.args().iter().map(node_count).sum::<f64>()
}

/// `_nodes(e)` for an expression.
pub fn nodes(e: &Expr) -> f64 {
    if e.is_Derivative() {
        not_ported("_nodes of a Derivative");
    }
    node_count(e)
}

/// `d = defaultdict(list); for a in seq: d[f(a)].append(a)` followed by
/// `sorted(d.items())`, for keys that are all distinct after grouping (so
/// that the item comparison reduces to the keys').
fn group_and_sort<K>(
    seq: Vec<Expr>,
    f: impl Fn(&Expr) -> K,
    eq: impl Fn(&K, &K) -> bool,
    lt: impl Fn(&K, &K) -> bool,
) -> Vec<Vec<Expr>> {
    let mut groups: Vec<(K, Vec<Expr>)> = Vec::new();
    for a in seq {
        let k = f(&a);
        match groups.iter_mut().find(|(g, _)| eq(g, &k)) {
            Some((_, v)) => v.push(a),
            None => groups.push((k, vec![a])),
        }
    }
    py_sort_by(&mut groups, |a, b| lt(&a.0, &b.0));
    groups.into_iter().map(|(_, v)| v).collect()
}

/// `list(ordered(seq, keys, default=default))` for a list of key
/// functions: group by the first key, sort the groups, and break ties in
/// a group with the remaining keys (then, if `default`, with the default
/// keys of [`ordered`]; otherwise ties keep their order).
pub fn ordered_with(
    seq: Vec<Expr>,
    keys: &[&dyn Fn(&Expr) -> SortKey],
    default: bool,
) -> Vec<Expr> {
    let Some((f, rest)) = keys.split_first() else {
        if !default {
            raise(
                ErrorKind::ValueError,
                "if default=False then keys must be provided",
            );
        }
        return ordered(seq);
    };
    let mut out = Vec::new();
    for value in group_and_sort(seq, |a| f(a), key_eq, key_lt) {
        if value.len() > 1 {
            if !rest.is_empty() {
                out.extend(ordered_with(value, rest, default));
            } else if default {
                out.extend(ordered(value));
            } else {
                out.extend(value);
            }
        } else {
            out.extend(value);
        }
    }
    out
}

/// `list(ordered(seq))` with the default keys: by `_nodes`, then
/// `default_sort_key`, then input order.
pub fn ordered(seq: Vec<Expr>) -> Vec<Expr> {
    // d[None].extend(seq): a single group.
    if seq.len() <= 1 {
        return seq;
    }
    // ordered(value, (_nodes, default_sort_key), default=False)
    let mut out = Vec::new();
    for group in group_and_sort(seq, nodes, |a, b| a == b, |a, b| a < b) {
        if group.len() > 1 {
            // ordered(value, [default_sort_key], default=False): ties keep
            // their order.
            let groups = group_and_sort(group, |e| default_sort_key(e, None), key_eq, key_lt);
            out.extend(groups.into_iter().flatten());
        } else {
            out.extend(group);
        }
    }
    out
}
