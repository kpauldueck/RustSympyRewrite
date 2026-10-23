//! `sympy.core.sympify`.

use crate::expr::Expr;
use crate::parsing::{STANDARD_TRANSFORMATIONS, parse_expr};

/// `sympify(s)` for a string: `parse_expr` with the standard
/// transformations.
pub fn sympify_str(s: &str, evaluate: bool) -> Expr {
    parse_expr(s, &STANDARD_TRANSFORMATIONS, evaluate)
}
