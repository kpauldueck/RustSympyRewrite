//! Python built-ins whose exact behaviour SymPy's results depend on.
//!
//! - [`PyDict`]: a `dict`, which iterates in insertion order. SymPy builds
//!   many results by iterating dicts (`Add.flatten`'s `terms`,
//!   `Mul.flatten`'s `c_powers`, `factor_terms`), so a Rust `HashMap`
//!   would scramble argument order.
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
    fn repr_matches_python() {
        assert_eq!(repr_str("x"), "'x'");
        assert_eq!(repr_str("it's"), "\"it's\"");
        assert_eq!(repr_str("'\""), "'\\'\"'");
        assert_eq!(repr_str("a\\b\n"), "'a\\\\b\\n'");
        assert_eq!(repr_str("\u{1}\u{a0}α"), "'\\x01\\xa0α'");
    }
}
