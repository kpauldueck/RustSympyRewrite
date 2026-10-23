//! `sympy.parsing.sympy_parser`: tokenizing, the token transformations and
//! evaluating the resulting code without Python's `eval`.
//!
//! SKELETON: signatures are the contract; bodies to be ported.

use crate::error::not_ported;
use crate::expr::Expr;

/// The transformations of `sympy_parser`, by their SymPy names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transformation {
    LambdaNotation,
    AutoSymbol,
    RepeatedDecimals,
    AutoNumber,
    FactorialNotation,
    ImplicitMultiplicationApplication,
    ImplicitMultiplication,
    ImplicitApplication,
    SplitSymbols,
    ConvertXor,
    ConvertEqualsSigns,
    Rationalize,
    FunctionExponentiation,
}

/// `standard_transformations`.
pub const STANDARD_TRANSFORMATIONS: [Transformation; 5] = [
    Transformation::LambdaNotation,
    Transformation::AutoSymbol,
    Transformation::RepeatedDecimals,
    Transformation::AutoNumber,
    Transformation::FactorialNotation,
];

/// `parse_expr(s, transformations=..., evaluate=...)` with SymPy's default
/// global namespace (`from sympy import *`).
pub fn parse_expr(s: &str, transformations: &[Transformation], evaluate: bool) -> Expr {
    let _ = (s, transformations, evaluate);
    not_ported("parse_expr")
}
