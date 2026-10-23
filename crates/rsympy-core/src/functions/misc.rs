//! `sympy.functions.elementary.miscellaneous`: `IdentityFunction` (`Id`).
//!
//! Later: `sqrt`, `root`, `cbrt`, `real_root`, `Min`, `Max`, `Rem`.

use std::sync::LazyLock;

use crate::expr::{Data, Expr};
use crate::generated::ClassId as C;

/// `IdentityFunction._symbol`: `Dummy('x')`, made once when SymPy imports
/// the module. Its index is random in SymPy, so it must not appear in any
/// result the oracle compares; it only has to differ from every other
/// Dummy's.
static ID_SYMBOL: LazyLock<Expr> =
    LazyLock::new(|| crate::symbol::dummy_with_index("x", (i64::MAX - 1) as u64, &[]));

/// `S.IdentityFunction`: `Basic.__new__(IdentityFunction)`, with no args.
static IDENTITY: LazyLock<Expr> =
    LazyLock::new(|| Expr::make(C::IdentityFunction, Data::None, vec![]));

/// `S.IdentityFunction` (`Id`).
pub fn identity_function() -> Expr {
    IDENTITY.clone()
}

/// `IdentityFunction._symbol`.
pub fn identity_symbol() -> Expr {
    ID_SYMBOL.clone()
}

/// `IdentityFunction.signature`: `Tuple(self._symbol)`.
pub fn identity_signature() -> Expr {
    crate::construct::raw(C::Tuple, vec![identity_symbol()])
}

/// `IdentityFunction.expr`: `self._symbol`.
pub fn identity_expr() -> Expr {
    identity_symbol()
}
