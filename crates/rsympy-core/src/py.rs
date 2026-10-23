//! Python built-ins whose exact behaviour SymPy's results depend on.
//!
//! - [`PyDict`]: a `dict`, which iterates in insertion order. SymPy builds
//!   many results by iterating dicts (`Add.flatten`'s `terms`,
//!   `Mul.flatten`'s `c_powers`, `factor_terms`), so a Rust `HashMap`
//!   would scramble argument order.
//! - [`PyIntSet`]: a `set` of ints, which iterates in hash-table slot
//!   order (`Add.as_content_primitive` loops over one).
//! - [`repr_str`]: `str.__repr__`, used by `srepr` and the oracle's
//!   expression encoding.
//!
//! `list.sort` lives in [`crate::compare`], next to SymPy's sort keys.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hash};

use crate::error::not_ported;
use crate::expr::FxHasher;

type FxBuild = BuildHasherDefault<FxHasher>;

/// A Python `dict`: a hash map that iterates in insertion order.
///
/// As in Python, assigning to an existing key keeps its position, and a
/// key that is removed and inserted again moves to the end.
#[derive(Clone)]
pub struct PyDict<K, V> {
    entries: Vec<Option<(K, V)>>,
    index: HashMap<K, usize, FxBuild>,
}

impl<K: Hash + Eq + Clone, V> Default for PyDict<K, V> {
    fn default() -> Self {
        PyDict::new()
    }
}

impl<K: Hash + Eq + Clone, V> PyDict<K, V> {
    pub fn new() -> Self {
        PyDict {
            entries: Vec::new(),
            index: HashMap::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.index.contains_key(k)
    }

    /// `d.get(k)`.
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let &i = self.index.get(k)?;
        self.entries[i].as_ref().map(|(_, v)| v)
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let &i = self.index.get(k)?;
        self.entries[i].as_mut().map(|(_, v)| v)
    }

    /// `d[k] = v`. Returns the old value; an existing key keeps its place.
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        if let Some(&i) = self.index.get(&k) {
            let slot = self.entries[i].as_mut().expect("indexed entry");
            return Some(std::mem::replace(&mut slot.1, v));
        }
        self.index.insert(k.clone(), self.entries.len());
        self.entries.push(Some((k, v)));
        None
    }

    /// `d.setdefault(k, default())`, returning the stored value.
    pub fn setdefault_with(&mut self, k: K, default: impl FnOnce() -> V) -> &mut V {
        let i = match self.index.get(&k) {
            Some(&i) => i,
            None => {
                let i = self.entries.len();
                self.index.insert(k.clone(), i);
                self.entries.push(Some((k, default())));
                i
            }
        };
        &mut self.entries[i].as_mut().expect("indexed entry").1
    }

    /// `d.pop(k, None)` / `del d[k]`.
    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let i = self.index.remove(k)?;
        let (_, v) = self.entries[i].take().expect("indexed entry");
        if self.entries.len() > 32 && self.index.len() * 2 < self.entries.len() {
            self.compact();
        }
        Some(v)
    }

    fn compact(&mut self) {
        self.entries.retain(Option::is_some);
        for (i, e) in self.entries.iter().enumerate() {
            let (k, _) = e.as_ref().expect("retained");
            *self.index.get_mut(k).expect("indexed key") = i;
        }
    }

    /// `d.items()`, in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.entries.iter().flatten().map(|(k, v)| (k, v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.entries.iter_mut().flatten().map(|(k, v)| (&*k, v))
    }

    /// `d.keys()`.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(k, _)| k)
    }

    /// `d.values()`.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_, v)| v)
    }

    /// `d.popitem()`: removes and returns the last-inserted item.
    pub fn popitem(&mut self) -> Option<(K, V)> {
        while let Some(e) = self.entries.pop() {
            if let Some((k, v)) = e {
                self.index.remove(&k);
                return Some((k, v));
            }
        }
        None
    }
}

impl<K: Hash + Eq + Clone, V> IntoIterator for PyDict<K, V> {
    type Item = (K, V);
    type IntoIter = std::iter::Flatten<std::vec::IntoIter<Option<(K, V)>>>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter().flatten()
    }
}

impl<K: Hash + Eq + Clone, V> FromIterator<(K, V)> for PyDict<K, V> {
    /// `dict(pairs)`: a later duplicate key overwrites the value in place.
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut d = PyDict::new();
        for (k, v) in iter {
            d.insert(k, v);
        }
        d
    }
}

/// A Python `set` of non-negative `int`s, iterating in CPython 3.13's
/// order: by slot in the open-addressing table `setobject.c` builds
/// (`PySet_MINSIZE` 8, `LINEAR_PROBES` 9, `PERTURB_SHIFT` 5, resizing to
/// `used*4` once `fill*5 >= mask*3`). Entries are never removed, so there
/// are no dummy slots.
#[derive(Clone, Debug)]
pub struct PyIntSet {
    table: Vec<Option<(rsympy_num::Int, u64)>>,
    fill: usize,
    used: usize,
}

const SET_MINSIZE: usize = 8;
const SET_LINEAR_PROBES: usize = 9;
const SET_PERTURB_SHIFT: u32 = 5;

/// `hash(k)` for a non-negative Python int: `k mod (2**61 - 1)`.
fn py_int_hash(k: &rsympy_num::Int) -> u64 {
    assert!(!k.is_negative(), "PyIntSet holds non-negative ints");
    let modulus = rsympy_num::Int::from((1u64 << 61) - 1);
    k.py_mod(&modulus)
        .expect("nonzero modulus")
        .as_u64()
        .expect("reduced hash fits")
}

/// The next slot of the probe sequence: `i = (i*5 + 1 + perturb) & mask`.
fn set_next_slot(i: usize, perturb: &mut u64, mask: usize) -> usize {
    *perturb >>= SET_PERTURB_SHIFT;
    (i.wrapping_mul(5)
        .wrapping_add(1)
        .wrapping_add(*perturb as usize))
        & mask
}

impl Default for PyIntSet {
    fn default() -> Self {
        PyIntSet::new()
    }
}

impl PyIntSet {
    /// `set()`.
    pub fn new() -> Self {
        PyIntSet {
            table: vec![None; SET_MINSIZE],
            fill: 0,
            used: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.used
    }

    pub fn is_empty(&self) -> bool {
        self.used == 0
    }

    /// The slot holding `k`, or the first free slot of its probe sequence
    /// (`set_add_entry`'s search; the table always has a free slot).
    fn find(&self, k: &rsympy_num::Int, hash: u64) -> (usize, bool) {
        let mask = self.table.len() - 1;
        let mut i = (hash as usize) & mask;
        let mut perturb = hash;
        loop {
            let probes = if i + SET_LINEAR_PROBES <= mask {
                SET_LINEAR_PROBES
            } else {
                0
            };
            for j in i..=i + probes {
                match &self.table[j] {
                    None => return (j, false),
                    Some((key, h)) if *h == hash && key == k => return (j, true),
                    Some(_) => {}
                }
            }
            i = set_next_slot(i, &mut perturb, mask);
        }
    }

    /// `k in s`.
    pub fn contains(&self, k: &rsympy_num::Int) -> bool {
        self.find(k, py_int_hash(k)).1
    }

    /// `s.add(k)` (`set_add_entry`).
    pub fn add(&mut self, k: rsympy_num::Int) {
        let hash = py_int_hash(&k);
        let (j, found) = self.find(&k, hash);
        if found {
            return;
        }
        self.table[j] = Some((k, hash));
        self.fill += 1;
        self.used += 1;
        let mask = self.table.len() - 1;
        if self.fill * 5 >= mask * 3 {
            let minused = if self.used > 50000 {
                self.used * 2
            } else {
                self.used * 4
            };
            self.resize(minused);
        }
    }

    /// `set_table_resize`: the smallest power-of-two table `> minused`,
    /// refilled in old slot order with `set_insert_clean`.
    fn resize(&mut self, minused: usize) {
        let mut newsize = SET_MINSIZE;
        while newsize <= minused {
            newsize <<= 1;
        }
        let old = std::mem::replace(&mut self.table, vec![None; newsize]);
        let mask = newsize - 1;
        for (k, hash) in old.into_iter().flatten() {
            let mut perturb = hash;
            let mut i = (hash as usize) & mask;
            let slot = loop {
                if self.table[i].is_none() {
                    break i;
                }
                if i + SET_LINEAR_PROBES <= mask
                    && let Some(j) =
                        (i + 1..=i + SET_LINEAR_PROBES).find(|&j| self.table[j].is_none())
                {
                    break j;
                }
                i = set_next_slot(i, &mut perturb, mask);
            };
            self.table[slot] = Some((k, hash));
        }
        self.fill = self.used;
    }

    /// Iteration order (`set_next`: by slot).
    pub fn iter(&self) -> impl Iterator<Item = &rsympy_num::Int> {
        self.table.iter().flatten().map(|(k, _)| k)
    }

    /// `self & other` (`set_intersection`): iterates the smaller set
    /// (`other`, unless it is the larger one) and adds its members that the
    /// other set has.
    pub fn intersection(&self, other: &PyIntSet) -> PyIntSet {
        let (so, other) = if other.len() > self.len() {
            (other, self)
        } else {
            (self, other)
        };
        let mut result = PyIntSet::new();
        for k in other.iter() {
            if so.contains(k) {
                result.add(k.clone());
            }
        }
        result
    }
}

impl FromIterator<rsympy_num::Int> for PyIntSet {
    /// `set(iterable)`: `set_add_key` for each item, in order.
    fn from_iter<I: IntoIterator<Item = rsympy_num::Int>>(iter: I) -> Self {
        let mut s = PyIntSet::new();
        for k in iter {
            s.add(k);
        }
        s
    }
}

/// `repr(s)` for a Python `str`.
///
/// Exact for ASCII, letters, digits, whitespace and control characters.
/// Python's notion of "printable" for other non-ASCII characters needs the
/// Unicode category tables, so those raise `NotPortedYet`.
pub fn repr_str(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(quote);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            c if c.is_ascii() || c.is_alphanumeric() => out.push(c),
            c if c.is_control() || c.is_whitespace() => {
                let n = c as u32;
                if n <= 0xff {
                    out.push_str(&format!("\\x{n:02x}"));
                } else if n <= 0xffff {
                    out.push_str(&format!("\\u{n:04x}"));
                } else {
                    out.push_str(&format!("\\U{n:08x}"));
                }
            }
            c => not_ported(format!("repr() of the character U+{:04X}", c as u32)),
        }
    }
    out.push(quote);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dict_keeps_insertion_order() {
        let mut d: PyDict<&str, i32> = PyDict::new();
        d.insert("b", 1);
        d.insert("a", 2);
        d.insert("b", 3);
        assert_eq!(d.iter().collect::<Vec<_>>(), [(&"b", &3), (&"a", &2)]);
        d.remove("b");
        d.insert("b", 4);
        assert_eq!(d.keys().copied().collect::<Vec<_>>(), ["a", "b"]);
        assert_eq!(d.popitem(), Some(("b", 4)));
    }

    #[test]
    fn int_set_iterates_like_cpython() {
        let mk = |v: &[i64]| -> PyIntSet { v.iter().map(|&k| rsympy_num::Int::small(k)).collect() };
        let order = |s: &PyIntSet| s.iter().map(|k| k.as_i64().unwrap()).collect::<Vec<_>>();
        // list(set([...])) and list(set(a) & set(b)) in CPython 3.13
        assert_eq!(order(&mk(&[3, 2, 1])), [1, 2, 3]);
        assert_eq!(order(&mk(&[11, 3])), [3, 11]);
        assert_eq!(order(&mk(&[3, 11])), [11, 3]);
        assert_eq!(order(&mk(&[9, 1, 17])), [9, 1, 17]);
        assert_eq!(order(&mk(&[5, 4, 3, 2, 1, 8])), [1, 2, 3, 4, 5, 8]);
        assert_eq!(order(&mk(&[16, 32, 8, 24])), [16, 8, 32, 24]);
        assert_eq!(order(&mk(&[48, 78, 72, 13, 3, 3])), [3, 72, 13, 78, 48]);
        assert_eq!(
            order(&mk(&[91, 5, 13, 15, 25, 44])),
            [5, 44, 13, 15, 25, 91]
        );
        assert_eq!(
            order(&mk(&[38, 11, 2, 14, 63, 77, 12])),
            [2, 38, 11, 12, 77, 14, 63]
        );
        assert_eq!(order(&mk(&[24, 13, 73, 0, 13, 52])), [0, 73, 13, 52, 24]);
        assert_eq!(
            order(&mk(&[17, 13, 1, 33, 1, 15, 66, 7, 20, 10])),
            [1, 33, 66, 7, 10, 13, 15, 17, 20]
        );
        let (a, b) = (mk(&[5, 13, 21, 2]), mk(&[21, 13, 40, 5, 7, 2, 29]));
        assert_eq!(order(&a.intersection(&b)), [5, 2, 21, 13]);
        assert_eq!(order(&b.intersection(&a)), [5, 2, 21, 13]);
        assert!(a.contains(&rsympy_num::Int::small(21)) && !a.contains(&rsympy_num::Int::small(3)));
    }

    #[test]
    fn repr_matches_python() {
        assert_eq!(repr_str("x"), "'x'");
        assert_eq!(repr_str("it's"), "\"it's\"");
        assert_eq!(repr_str("'\""), "'\\'\"'");
        assert_eq!(repr_str("a\\b\n"), "'a\\\\b\\n'");
        assert_eq!(repr_str("\u{1}\u{a0}α"), "'\\x01\\xa0α'");
    }
}
