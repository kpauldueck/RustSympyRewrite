//! SymPy's assumption facts and `FactKB` (`sympy.core.facts`), on bitmasks.
//!
//! A knowledge base maps each fact to one of four states: not known yet,
//! asked and found undecidable (`None` stored), `False` or `True`. That is
//! two bits per fact, so a whole `FactKB` is one `u64`.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{ErrorKind, raise};
use crate::generated::{
    ALL_FACTS, BETA_RULES, BETA_TRIGGERS, FACT_NAMES, FULL_IMPLICATIONS, NUM_FACTS,
};

crate::generated::define_facts!();

impl Fact {
    pub fn name(self) -> &'static str {
        FACT_NAMES[self as usize]
    }

    pub fn from_name(name: &str) -> Option<Fact> {
        FACT_NAMES
            .iter()
            .position(|n| *n == name)
            .map(|i| ALL_FACTS[i])
    }

    pub(crate) fn bit(self) -> u64 {
        1 << self as u32
    }

    pub fn all() -> impl Iterator<Item = Fact> {
        ALL_FACTS.into_iter()
    }
}

/// Iterates the facts in a mask in fact order.
pub(crate) fn facts_in(mask: u64) -> impl Iterator<Item = Fact> {
    (0..NUM_FACTS)
        .filter(move |i| mask >> i & 1 == 1)
        .map(|i| ALL_FACTS[i])
}

const STATE_NONE: u64 = 1;
const STATE_FALSE: u64 = 2;
const STATE_TRUE: u64 = 3;

/// A `FactKB`.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Kb(u64);

impl Kb {
    pub const EMPTY: Kb = Kb(0);

    /// The KB holding exactly `known` (values from `truth`) plus `none`
    /// stored as `None`.
    pub(crate) fn from_masks(known: u64, truth: u64, none: u64) -> Kb {
        let mut kb = Kb::EMPTY;
        for f in facts_in(known) {
            kb.set(f, Some(truth & f.bit() != 0));
        }
        for f in facts_in(none) {
            kb.set(f, None);
        }
        kb
    }

    fn state(self, f: Fact) -> u64 {
        self.0 >> (2 * f as u32) & 3
    }

    fn set(&mut self, f: Fact, v: Option<bool>) {
        let s = match v {
            None => STATE_NONE,
            Some(false) => STATE_FALSE,
            Some(true) => STATE_TRUE,
        };
        let shift = 2 * f as u32;
        self.0 = (self.0 & !(3 << shift)) | s << shift;
    }

    /// `fact in kb`.
    pub fn contains(self, f: Fact) -> bool {
        self.state(f) != 0
    }

    /// `kb.get(fact)`: `None` when absent or stored as `None`.
    pub fn get(self, f: Fact) -> Option<bool> {
        match self.state(f) {
            STATE_TRUE => Some(true),
            STATE_FALSE => Some(false),
            _ => None,
        }
    }

    /// `kb[fact]` when `fact in kb`.
    pub fn lookup(self, f: Fact) -> Option<Option<bool>> {
        match self.state(f) {
            0 => None,
            STATE_NONE => Some(None),
            STATE_FALSE => Some(Some(false)),
            _ => Some(Some(true)),
        }
    }

    /// The `(fact, value)` items, in fact order, skipping stored `None`s
    /// when `with_none` is false.
    pub fn items(self, with_none: bool) -> Vec<(Fact, Option<bool>)> {
        Fact::all()
            .filter_map(|f| self.lookup(f).map(|v| (f, v)))
            .filter(|(_, v)| with_none || v.is_some())
            .collect()
    }

    /// `del kb[fact]`.
    pub fn remove(&mut self, f: Fact) {
        let shift = 2 * f as u32;
        self.0 &= !(3 << shift);
    }

    /// `FactKB._tell(k, v)`: whether the KB changed.
    pub fn tell(&mut self, f: Fact, v: Option<bool>) -> bool {
        match self.lookup(f) {
            Some(Some(old)) => {
                if Some(old) == v {
                    false
                } else {
                    raise(
                        ErrorKind::InconsistentAssumptions,
                        format!("{}={:?}", f.name(), v),
                    )
                }
            }
            _ => {
                self.set(f, v);
                true
            }
        }
    }

    /// `FactKB.deduce_all_facts(facts)`.
    pub fn deduce_all_facts(&mut self, facts: &[(Fact, Option<bool>)]) {
        let mut facts: Vec<(Fact, Option<bool>)> = facts.to_vec();
        while !facts.is_empty() {
            let mut beta_maytrigger: u128 = 0;
            for &(k, v) in &facts {
                if !self.tell(k, v) {
                    continue;
                }
                let Some(v) = v else { continue };
                let idx = 2 * k as usize + v as usize;
                let (t, fl) = FULL_IMPLICATIONS[idx];
                for f in facts_in(t) {
                    self.tell(f, Some(true));
                }
                for f in facts_in(fl) {
                    self.tell(f, Some(false));
                }
                for &b in BETA_TRIGGERS[idx] {
                    beta_maytrigger |= 1 << b;
                }
            }
            facts.clear();
            // SymPy iterates a set of rule indices, whose order can differ
            // from this ascending one. The closure is the same either way;
            // only which contradiction is reported first could differ.
            for (bidx, &(t, fl, f, v)) in BETA_RULES.iter().enumerate() {
                if beta_maytrigger >> bidx & 1 == 0 {
                    continue;
                }
                let ok = facts_in(t).all(|k| self.get(k) == Some(true))
                    && facts_in(fl).all(|k| self.get(k) == Some(false));
                if ok {
                    facts.push((f, Some(v)));
                }
            }
        }
    }

    /// `StdFactKB(facts)`: a KB holding `facts` and all their implications.
    pub fn deduced(facts: &[(Fact, Option<bool>)]) -> Kb {
        let mut kb = Kb::EMPTY;
        kb.deduce_all_facts(facts);
        kb
    }
}

impl std::fmt::Debug for Kb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut m = f.debug_map();
        for (k, v) in self.items(true) {
            m.entry(&k.name(), &v);
        }
        m.finish()
    }
}

/// A node's `_assumptions`, shared by every handle to the node. Facts are
/// only ever added, so concurrent writers can merge with `fetch_or`.
pub(crate) struct FactCell(AtomicU64);

impl FactCell {
    pub(crate) fn new(kb: Kb) -> FactCell {
        FactCell(AtomicU64::new(kb.0))
    }

    pub(crate) fn load(&self) -> Kb {
        Kb(self.0.load(Ordering::Acquire))
    }

    /// Adds what `kb` knows: a definite value beats a stored `None`, which
    /// beats absence. (`kb` is normally this cell's KB plus deductions.)
    pub(crate) fn merge(&self, kb: Kb) {
        let mut old = self.0.load(Ordering::Acquire);
        loop {
            let merged = merge_states(old, kb.0);
            match self
                .0
                .compare_exchange_weak(old, merged, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => return,
                Err(cur) => old = cur,
            }
        }
    }
}

/// Per 2-bit slot: `new` if definite, else `old` if definite, else
/// whichever of the two is `None`.
fn merge_states(old: u64, new: u64) -> u64 {
    const HI: u64 = 0xAAAA_AAAA_AAAA_AAAA;
    let spread = |x: u64| (x & HI) | ((x & HI) >> 1);
    let nd = spread(new);
    let od = spread(old);
    (new & nd) | (!nd & ((old & od) | (!od & (old | new))))
}

impl Clone for FactCell {
    fn clone(&self) -> FactCell {
        FactCell::new(self.load())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_implies_the_usual() {
        let kb = Kb::deduced(&[(Fact::Positive, Some(true))]);
        assert_eq!(kb.get(Fact::Real), Some(true));
        assert_eq!(kb.get(Fact::Nonzero), Some(true));
        assert_eq!(kb.get(Fact::Negative), Some(false));
        assert_eq!(kb.get(Fact::Integer), None);
    }

    #[test]
    fn merge_replaces_none_with_a_value() {
        let mut none = Kb::EMPTY;
        none.tell(Fact::Rational, None);
        none.tell(Fact::Real, Some(true));
        let cell = FactCell::new(none);
        let mut deduced = none;
        deduced.tell(Fact::Rational, Some(false));
        deduced.tell(Fact::Odd, None);
        cell.merge(deduced);
        let kb = cell.load();
        assert_eq!(kb.lookup(Fact::Rational), Some(Some(false)));
        assert_eq!(kb.lookup(Fact::Real), Some(Some(true)));
        assert_eq!(kb.lookup(Fact::Odd), Some(None));
        // A stale KB holding None never erases a value.
        cell.merge(none);
        assert_eq!(cell.load().lookup(Fact::Rational), Some(Some(false)));
    }

    #[test]
    fn beta_rule_integer_not_odd_is_even() {
        let kb = Kb::deduced(&[(Fact::Integer, Some(true)), (Fact::Odd, Some(false))]);
        assert_eq!(kb.get(Fact::Even), Some(true));
    }
}
