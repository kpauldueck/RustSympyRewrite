//! `Expr` methods that SymPy overrides per class, dispatched on the node's
//! class the way Python's MRO lookup would (`Mul.as_coeff_Mul` before
//! `Expr.as_coeff_Mul`, and so on). The per-class bodies live with their
//! class; the `Expr`/`Basic` defaults live here.

use std::collections::HashSet;

use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::Expr;
use crate::generated::{ClassId as C, Method, Tag};
use crate::numbers::s;

impl Expr {
    /// `bool(self)`: Python's truthiness of a SymPy object. `Basic` has no
    /// `__bool__`, so most objects are true.
    pub fn truthy(&self) -> bool {
        match self.method_owner(Method::Bool) {
            "Zero" | "BooleanFalse" => false,
            "BooleanTrue" => true,
            "Float" => *self.as_float().expect("Float").0 != rsympy_mpf::FZERO,
            "Relational" => raise(
                ErrorKind::TypeError,
                "cannot determine truth value of Relational",
            ),
            "" => match self.method_owner(Method::Len) {
                "" => true,
                "Tuple" | "FiniteSet" => !self.args().is_empty(),
                "EmptySet" => false,
                o => not_ported(format!("{o}.__len__ for truthiness")),
            },
            o => not_ported(format!("{o}.__bool__")),
        }
    }

    /// `self.is_comparable`.
    pub fn is_comparable(&self) -> bool {
        match self.method_owner(Method::IsComparable) {
            "Zero" | "Infinity" | "NegativeInfinity" => true,
            "NaN" => false,
            "Basic" => {
                if self.is_extended_real() == Some(false) {
                    return false;
                }
                if !self.is_number() {
                    return false;
                }
                // n, i = self.as_real_imag(), evaluated to 2 digits
                if self.is_Rational() {
                    return true;
                }
                if let Some((_, prec)) = self.as_float() {
                    return prec != 1;
                }
                if self.is_NumberSymbol() {
                    // re(c) = c and im(c) = 0 for the real constants, and c
                    // evaluates to a Float with 2 digits.
                    return true;
                }
                not_ported(format!("{}.is_comparable (needs evalf)", self.class_name()))
            }
            o => not_ported(format!("{o}.is_comparable")),
        }
    }
}

impl Expr {
    /// The fallback of a method dispatched below: `Expr`'s default, unless
    /// the class has no such method (a non-`Expr` Basic such as a Boolean
    /// or a Tuple), where Python raises AttributeError.
    fn expr_default(&self, method: Method, name: &str) {
        if self.method_owner(method).is_empty() {
            raise(
                ErrorKind::AttributeError,
                format!("'{}' object has no attribute '{name}'", self.class_name()),
            );
        }
    }
}

#[allow(non_snake_case)]
impl Expr {
    /// `self.as_coeff_Mul(rational)`: `(coeff, rest)` with a Number coeff.
    pub fn as_coeff_Mul(&self, rational: bool) -> (Expr, Expr) {
        if self.is_Rational() {
            return (self.clone(), s::one());
        }
        if self.is_Number() {
            return if rational {
                (s::one(), self.clone())
            } else {
                (self.clone(), s::one())
            };
        }
        if self.is_Mul() {
            return crate::mul::as_coeff_Mul(self, rational);
        }
        self.expr_default(Method::AsCoeff_Mul, "as_coeff_Mul");
        (s::one(), self.clone())
    }

    /// `self.as_coeff_Add(rational)`: `(coeff, rest)` with a Number coeff.
    pub fn as_coeff_Add(&self, rational: bool) -> (Expr, Expr) {
        if self.is_Rational() {
            return (self.clone(), s::zero());
        }
        if self.is_Number() {
            return if rational {
                (s::zero(), self.clone())
            } else {
                (self.clone(), s::zero())
            };
        }
        if self.is_Add() {
            return crate::add::as_coeff_Add(self, rational);
        }
        self.expr_default(Method::AsCoeff_Add, "as_coeff_Add");
        (s::zero(), self.clone())
    }

    /// `self.as_coeff_mul()` (no deps, `rational=True`).
    pub fn as_coeff_mul(&self) -> (Expr, Vec<Expr>) {
        if self.is_Number() {
            if self.is_Rational() {
                return (self.clone(), vec![]);
            }
            if self.is_negative() == Some(true) {
                return (s::negative_one(), vec![crate::ops::neg(self)]);
            }
            return (s::one(), vec![self.clone()]);
        }
        if self.is_Mul() {
            return crate::mul::as_coeff_mul(self, true);
        }
        self.expr_default(Method::AsCoeffMul, "as_coeff_mul");
        (s::one(), vec![self.clone()])
    }

    /// `self.as_coeff_add()` (no deps).
    pub fn as_coeff_add(&self) -> (Expr, Vec<Expr>) {
        if self.is_Number() {
            if self.is_Rational() {
                return (self.clone(), vec![]);
            }
            return (s::zero(), vec![self.clone()]);
        }
        if self.is_Add() {
            return crate::add::as_coeff_add(self);
        }
        self.expr_default(Method::AsCoeffAdd, "as_coeff_add");
        (s::zero(), vec![self.clone()])
    }

    /// `self.as_base_exp()`: `b, e` with `self == b**e`.
    pub fn as_base_exp(&self) -> (Expr, Expr) {
        match self.cls() {
            C::Pow => crate::power::as_base_exp(self),
            C::Mul => crate::mul::as_base_exp(self),
            C::ImaginaryUnit => (s::negative_one(), s::half()),
            C::Exp => crate::functions::exp_as_base_exp(self),
            _ => {
                self.expr_default(Method::AsBaseExp, "as_base_exp");
                (self.clone(), s::one())
            }
        }
    }

    /// `self.as_numer_denom()`.
    pub fn as_numer_denom(&self) -> (Expr, Expr) {
        match self.cls() {
            C::Integer | C::Zero | C::One | C::NegativeOne => (self.clone(), s::one()),
            C::Rational | C::Half => (Expr::integer(self.p()), Expr::integer(self.q())),
            C::Add => crate::add::as_numer_denom(self),
            C::Mul => crate::mul::as_numer_denom(self),
            C::Pow => crate::power::as_numer_denom(self),
            C::Exp => crate::functions::exp_as_numer_denom(self),
            _ => {
                self.expr_default(Method::AsNumerDenom, "as_numer_denom");
                (self.clone(), s::one())
            }
        }
    }

    /// `self.could_extract_minus_sign()`.
    pub fn could_extract_minus_sign(&self) -> bool {
        if self.is_Number() {
            return self.is_extended_negative() == Some(true);
        }
        match self.cls() {
            C::Add => crate::add::could_extract_minus_sign(self),
            C::Mul => crate::mul::could_extract_minus_sign(self),
            _ => false,
        }
    }

    /// `Mul.make_args(self)`.
    pub fn mul_args(&self) -> Vec<Expr> {
        if self.is_Mul() {
            self.args().to_vec()
        } else {
            vec![self.clone()]
        }
    }

    /// `Add.make_args(self)`.
    pub fn add_args(&self) -> Vec<Expr> {
        if self.is_Add() {
            self.args().to_vec()
        } else {
            vec![self.clone()]
        }
    }

    /// `self.args_cnc(split_1=split_1)`: commutative and non-commutative
    /// factors, keeping order.
    pub fn args_cnc(&self, split_1: bool) -> (Vec<Expr>, Vec<Expr>) {
        let args = self.mul_args();
        let split = args
            .iter()
            .position(|a| a.is_commutative() != Some(true))
            .unwrap_or(args.len());
        let mut c = args[..split].to_vec();
        let nc = args[split..].to_vec();
        if split_1
            && !c.is_empty()
            && c[0].is_Number()
            && c[0].is_extended_negative() == Some(true)
            && !c[0].is_cls(C::NegativeOne)
        {
            let n = crate::ops::neg(&c[0]);
            c.splice(0..1, [s::negative_one(), n]);
        }
        (c, nc)
    }

    /// `self.free_symbols`, as a list in first-seen order (SymPy returns a
    /// set; callers that iterate it in SymPy depend on hash order).
    pub fn free_symbols(&self) -> Vec<Expr> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        collect_free_symbols(self, &mut out, &mut seen);
        out
    }

    /// `self.has(pattern)` for an expression pattern: whether `pattern`
    /// occurs as a subtree.
    pub fn has(&self, pattern: &Expr) -> bool {
        self == pattern || self.args().iter().any(|a| a.has(pattern))
    }

    /// `self.has(cls)`: whether any subtree is an instance of `tag`.
    pub fn has_a(&self, tag: Tag) -> bool {
        self.is_a(tag) || self.args().iter().any(|a| a.has_a(tag))
    }

    /// `self.has(cls)` for a concrete class.
    pub fn has_cls(&self, cls: C) -> bool {
        self.cls() == cls || self.args().iter().any(|a| a.has_cls(cls))
    }

    /// `preorder_traversal(self)`.
    pub fn preorder(&self) -> Vec<Expr> {
        let mut out = Vec::new();
        let mut stack = vec![self.clone()];
        while let Some(e) = stack.pop() {
            for a in e.args().iter().rev() {
                stack.push(a.clone());
            }
            out.push(e);
        }
        out
    }
}

fn collect_free_symbols(e: &Expr, out: &mut Vec<Expr>, seen: &mut HashSet<Expr>) {
    if e.is_Symbol() {
        if seen.insert(e.clone()) {
            out.push(e.clone());
        }
        return;
    }
    if e.is_a(Tag::ExprWithLimits) || matches!(e.cls(), C::Lambda | C::Derivative | C::Subs) {
        crate::error::not_ported(format!("{}.free_symbols", e.class_name()));
    }
    for a in e.args() {
        collect_free_symbols(a, out, seen);
    }
}
