//! The old assumptions system: `obj.is_<fact>` (`sympy.core.assumptions`).
//!
//! [`Expr::is`] reads the node's KB and otherwise runs `_ask`: try the
//! class's `_eval_is_<fact>` handler, then the facts that could decide it,
//! breadth first, deducing implications into the KB as values turn up.
//!
//! SymPy visits each batch of prerequisite facts in set order and then
//! shuffles it with a seeded RNG. Set order of strings depends on the hash
//! seed, so it can't be reproduced in general; rsympy visits them in fact
//! order. With handlers consistent with the rules (SymPy's intent), the
//! order does not change any answer. A few SymPy handlers are not (e.g.
//! `Mul._eval_is_even` calls `Mul(p, 0)` not even), so some answers, such
//! as `Mul(p, 0, evaluate=False).is_hermitian`, vary with SymPy's hash
//! seed; rsympy gives SymPy's answer for fact order, and the oracle leaves
//! those cases out (oracle/gen_diff_arith_facts.py).

use crate::expr::Expr;
use crate::facts::{Fact, Kb, facts_in};
use crate::generated::{ClassId, PREREQ};

impl Expr {
    /// `self.is_<fact>`.
    pub fn is(&self, fact: Fact) -> Option<bool> {
        if fact == Fact::Commutative
            && let Some(c) = self.0.commutative_slot
        {
            return c;
        }
        if let Some(v) = self.assumptions_kb().lookup(fact) {
            return v;
        }
        ask(fact, self)
    }

    /// Records facts (and what they imply) in this node's KB.
    pub(crate) fn deduce(&self, facts: &[(Fact, Option<bool>)]) {
        let mut kb = self.assumptions_kb();
        kb.deduce_all_facts(facts);
        self.merge_kb(kb);
    }
}

/// `_ask(fact, obj)`.
fn ask(fact: Fact, obj: &Expr) -> Option<bool> {
    let handlers = obj.info().handlers;
    let mut to_check = vec![fact];
    let mut queued = fact.bit();
    let mut i = 0;
    while i < to_check.len() {
        let fact_i = to_check[i];
        i += 1;
        if obj.assumptions_kb().contains(fact_i) {
            continue;
        }
        let value = if handlers & fact_i.bit() != 0 {
            handler(obj, fact_i)
        } else {
            None
        };
        if let Some(v) = value {
            obj.deduce(&[(fact_i, Some(v))]);
        }
        if let Some(v) = obj.assumptions_kb().get(fact) {
            return Some(v);
        }
        let new = PREREQ[fact_i as usize] & !queued;
        to_check.extend(facts_in(new));
        queued |= new;
    }
    if let Some(v) = obj.assumptions_kb().lookup(fact) {
        return v;
    }
    let mut kb: Kb = obj.assumptions_kb();
    kb.tell(fact, None);
    obj.merge_kb(kb);
    None
}

/// Calls `type(obj)._prop_handler[fact](obj)`. Only reached for facts the
/// generated table says the class handles.
fn handler(obj: &Expr, fact: Fact) -> Option<bool> {
    use ClassId as C;
    match obj.cls() {
        C::Integer | C::Zero | C::One | C::NegativeOne | C::Rational | C::Half => {
            crate::numbers::rational_handler(obj, fact)
        }
        C::Float => crate::numbers::float_handler(obj, fact),
        C::Add => crate::add::handler(obj, fact),
        C::Mul => crate::mul::handler(obj, fact),
        C::Pow => crate::power::handler(obj, fact),
        C::Symbol | C::Dummy | C::Wild => None,
        _ => crate::functions::handler(obj, fact),
    }
}

macro_rules! fact_getters {
    ($($name:ident => $fact:ident),* $(,)?) => {
        impl Expr {
            $(
                #[doc = concat!("`self.", stringify!($name), "`.")]
                pub fn $name(&self) -> Option<bool> {
                    self.is(Fact::$fact)
                }
            )*
        }
    };
}

fact_getters! {
    is_algebraic => Algebraic,
    is_antihermitian => Antihermitian,
    is_commutative => Commutative,
    is_complex => Complex,
    is_composite => Composite,
    is_even => Even,
    is_extended_negative => ExtendedNegative,
    is_extended_nonnegative => ExtendedNonnegative,
    is_extended_nonpositive => ExtendedNonpositive,
    is_extended_nonzero => ExtendedNonzero,
    is_extended_positive => ExtendedPositive,
    is_extended_real => ExtendedReal,
    is_finite => Finite,
    is_hermitian => Hermitian,
    is_imaginary => Imaginary,
    is_infinite => Infinite,
    is_integer => Integer,
    is_irrational => Irrational,
    is_negative => Negative,
    is_noninteger => Noninteger,
    is_nonnegative => Nonnegative,
    is_nonpositive => Nonpositive,
    is_nonzero => Nonzero,
    is_odd => Odd,
    is_polar => Polar,
    is_positive => Positive,
    is_prime => Prime,
    is_rational => Rational,
    is_real => Real,
    is_transcendental => Transcendental,
    is_zero => Zero,
}

/// `fuzzy_not(v)`.
pub fn fuzzy_not(v: Option<bool>) -> Option<bool> {
    v.map(|b| !b)
}

/// `fuzzy_and(iterable)`.
pub fn fuzzy_and(vs: impl IntoIterator<Item = Option<bool>>) -> Option<bool> {
    let mut rv = Some(true);
    for v in vs {
        match v {
            None => rv = None,
            Some(false) => return Some(false),
            Some(true) => {}
        }
    }
    rv
}

/// `fuzzy_or(iterable)`.
pub fn fuzzy_or(vs: impl IntoIterator<Item = Option<bool>>) -> Option<bool> {
    let mut rv = Some(false);
    for v in vs {
        match v {
            Some(true) => return Some(true),
            None => rv = None,
            Some(false) => {}
        }
    }
    rv
}

/// `_torf(args)`: `True` if all are true, `False` if all are false, else
/// `None`.
pub fn torf(vs: impl IntoIterator<Item = Option<bool>>) -> Option<bool> {
    let (mut saw_t, mut saw_f) = (false, false);
    for v in vs {
        match v {
            Some(true) => {
                if saw_f {
                    return None;
                }
                saw_t = true;
            }
            Some(false) => {
                if saw_t {
                    return None;
                }
                saw_f = true;
            }
            None => return None,
        }
    }
    Some(saw_t)
}

/// `fuzzy_xor(args)`: `None` if any is `None`, else the parity of trues.
pub fn fuzzy_xor(vs: impl IntoIterator<Item = Option<bool>>) -> Option<bool> {
    let mut t = 0;
    for v in vs {
        match v {
            None => return None,
            Some(true) => t += 1,
            Some(false) => {}
        }
    }
    Some(t % 2 == 1)
}

/// `fuzzy_nand(args)`.
pub fn fuzzy_nand(vs: impl IntoIterator<Item = Option<bool>>) -> Option<bool> {
    fuzzy_not(fuzzy_and(vs))
}

/// `_fuzzy_group(args, quick_exit)`.
pub fn fuzzy_group(vs: impl IntoIterator<Item = Option<bool>>, quick_exit: bool) -> Option<bool> {
    let mut saw_other = false;
    for a in vs {
        match a {
            Some(true) => continue,
            None => return None,
            Some(false) => {
                if quick_exit && saw_other {
                    return None;
                }
                saw_other = true;
            }
        }
    }
    Some(!saw_other)
}
