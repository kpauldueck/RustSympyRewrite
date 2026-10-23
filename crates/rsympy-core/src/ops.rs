//! Python's operators on SymPy objects.
//!
//! `a + b` in Python calls `a.__add__(b)`, chosen by the class of `a`
//! (Number subclasses evaluate number-with-number arithmetic directly;
//! everything else builds `Add(a, b)`). These functions reproduce that
//! dispatch, and the `std::ops` impls at the bottom route Rust's operators
//! through them.
//!
//! All `Expr`s share `_op_priority = 10.0`, so `call_highest_priority`
//! never hands the operation to the right operand.

use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::Expr;
use crate::generated::{ClassId as C, Tag};
use crate::numbers::{self, s};

/// Python's operator protocol when an operand is not an `Expr`.
/// `Expr.__add__` and friends (and Number's, which end there) return
/// `NotImplemented` for a non-`Expr` other (`@sympify_return`), so Python
/// asks `other.__radd__`. `BooleanAtom` defines every arithmetic dunder as
/// `_noop`, which raises TypeError; other Booleans (Relational, And, ...)
/// define none, so Python raises TypeError itself. Other non-`Expr`
/// objects (Tuple, sets) have their own operators, not ported yet.
fn check_operands(a: &Expr, b: &Expr, op: &str) {
    if a.is_a(Tag::Expr) && b.is_a(Tag::Expr) {
        return;
    }
    for x in [a, b] {
        if x.is_a(Tag::BooleanAtom) {
            raise(
                ErrorKind::TypeError,
                "BooleanAtom not allowed in this context.",
            );
        }
        if !x.is_a(Tag::Expr) && !x.is_a(Tag::Boolean) {
            not_ported(format!("{}.__{op}__ / __r{op}__", x.class_name()));
        }
    }
    raise(
        ErrorKind::TypeError,
        format!(
            "unsupported operand type(s) for {op}: '{}' and '{}'",
            a.class_name(),
            b.class_name()
        ),
    )
}

/// The classes whose `__add__` etc. are defined in `core.numbers` (Number
/// subclasses). `ComplexInfinity` and the NumberSymbols are not Numbers.
fn numeric(a: &Expr) -> bool {
    a.is_Number()
}

/// `a + b`.
pub fn add(a: &Expr, b: &Expr) -> Expr {
    check_operands(a, b, "add");
    if numeric(a) {
        return numbers::add(a, b);
    }
    crate::add::add(vec![a.clone(), b.clone()])
}

/// `a - b`.
pub fn sub(a: &Expr, b: &Expr) -> Expr {
    check_operands(a, b, "sub");
    if numeric(a) {
        return numbers::sub(a, b);
    }
    crate::add::add(vec![a.clone(), neg(b)])
}

/// `a * b`.
pub fn mul(a: &Expr, b: &Expr) -> Expr {
    check_operands(a, b, "mul");
    if numeric(a) {
        return numbers::mul(a, b);
    }
    crate::mul::mul(vec![a.clone(), b.clone()])
}

/// `a / b`.
pub fn div(a: &Expr, b: &Expr) -> Expr {
    check_operands(a, b, "truediv");
    if numeric(a) {
        return numbers::truediv(a, b);
    }
    let denom = crate::power::pow(b, &s::negative_one());
    if a.is_cls(C::One) {
        return denom;
    }
    crate::mul::mul(vec![a.clone(), denom])
}

/// `a ** b`.
pub fn pow(a: &Expr, b: &Expr) -> Expr {
    check_operands(a, b, "pow");
    crate::power::pow(a, b)
}

/// `-a`.
pub fn neg(a: &Expr) -> Expr {
    match a.cls() {
        C::Integer
        | C::Zero
        | C::One
        | C::NegativeOne
        | C::Rational
        | C::Half
        | C::Float
        | C::Infinity
        | C::NegativeInfinity
        | C::NaN
        | C::ComplexInfinity => numbers::neg(a),
        C::Add => crate::add::neg(a),
        C::Mul => crate::mul::neg(a),
        // Expr.__neg__: a 2-arg Mul with -1 in the coefficient slot.
        _ => crate::mul::from_args(vec![s::negative_one(), a.clone()], Some(a.is_commutative())),
    }
}

/// `abs(a)`.
pub fn abs(a: &Expr) -> Expr {
    if let Some(r) = numbers::abs(a) {
        return r;
    }
    crate::functions::call(C::Abs, vec![a.clone()])
}

/// `a % b`.
pub fn rem(a: &Expr, b: &Expr) -> Expr {
    check_operands(a, b, "mod");
    if numeric(a)
        && let Some(r) = numbers::rem(a, b)
    {
        return r;
    }
    crate::functions::call(C::Mod, vec![a.clone(), b.clone()])
}

/// `a // b`: `floor(a / b)`, or Python's `//` for two Integers.
pub fn floordiv(a: &Expr, b: &Expr) -> Expr {
    if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
        let q = x.floor_div(y).unwrap_or_else(|_| {
            crate::error::raise(
                crate::error::ErrorKind::ZeroDivisionError,
                "integer division or modulo by zero",
            )
        });
        return Expr::integer(q);
    }
    crate::functions::call(C::Floor, vec![div(a, b)])
}

macro_rules! binop {
    ($trait:ident, $method:ident, $f:path) => {
        impl std::ops::$trait<&Expr> for &Expr {
            type Output = Expr;
            fn $method(self, rhs: &Expr) -> Expr {
                $f(self, rhs)
            }
        }
        impl std::ops::$trait<Expr> for Expr {
            type Output = Expr;
            fn $method(self, rhs: Expr) -> Expr {
                $f(&self, &rhs)
            }
        }
        impl std::ops::$trait<&Expr> for Expr {
            type Output = Expr;
            fn $method(self, rhs: &Expr) -> Expr {
                $f(&self, rhs)
            }
        }
        impl std::ops::$trait<Expr> for &Expr {
            type Output = Expr;
            fn $method(self, rhs: Expr) -> Expr {
                $f(self, &rhs)
            }
        }
        impl std::ops::$trait<i64> for &Expr {
            type Output = Expr;
            fn $method(self, rhs: i64) -> Expr {
                $f(self, &Expr::integer(rhs))
            }
        }
        impl std::ops::$trait<i64> for Expr {
            type Output = Expr;
            fn $method(self, rhs: i64) -> Expr {
                $f(&self, &Expr::integer(rhs))
            }
        }
    };
}

binop!(Add, add, add);
binop!(Sub, sub, sub);
binop!(Mul, mul, mul);
binop!(Div, div, div);
binop!(Rem, rem, rem);

impl std::ops::Neg for &Expr {
    type Output = Expr;
    fn neg(self) -> Expr {
        neg(self)
    }
}

impl std::ops::Neg for Expr {
    type Output = Expr;
    fn neg(self) -> Expr {
        neg(&self)
    }
}

impl Expr {
    /// `self ** other`.
    pub fn pow(&self, other: &Expr) -> Expr {
        pow(self, other)
    }

    /// `self ** n`.
    pub fn powi(&self, n: i64) -> Expr {
        pow(self, &Expr::integer(n))
    }
}
