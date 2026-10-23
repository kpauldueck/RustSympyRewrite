//! Python exceptions.
//!
//! SymPy signals errors by raising, and parts of it catch and carry on
//! (`except PrecisionExhausted: pass`). The port keeps that control flow:
//! [`raise`] unwinds with a [`SympyError`] payload and [`catch`] turns it
//! back into a `Result`, the way `try`/`except` does. Public entry points
//! of the facade wrap their work in [`catch`], so callers get a `Result`.
//!
//! `NotPortedYet` is not a Python exception: it marks a SymPy code path
//! that this port has not reached yet, so a missing piece shows up as a
//! named error and never as a wrong answer.

use std::fmt;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Once;

/// The Python exception class, or `NotPortedYet`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    TypeError,
    ValueError,
    ZeroDivisionError,
    NotImplementedError,
    AttributeError,
    NameError,
    SyntaxError,
    TokenError,
    SympifyError,
    PrecisionExhausted,
    /// `InconsistentAssumptions` (a `ValueError` subclass in SymPy).
    InconsistentAssumptions,
    /// mpmath's `ComplexResult` (a `ValueError` subclass).
    ComplexResult,
    PolificationFailed,
    RecursionError,
    OverflowError,
    IndexError,
    KeyError,
    /// An `assert` in SymPy's code failing.
    AssertionError,
    /// `sympy.core.function.BadSignatureError` (a `TypeError` subclass).
    BadSignatureError,
    /// `sympy.core.function.BadArgumentsError` (a `TypeError` subclass).
    BadArgumentsError,
    /// `sympy.core.function.ArgumentIndexError` (a `ValueError` subclass).
    ArgumentIndexError,
    /// `sympy.polys.polyerrors.PolynomialError` (`PolificationFailed`'s
    /// base class).
    PolynomialError,
    NotPortedYet,
}

impl ErrorKind {
    /// The Python class name, as the golden cases record it.
    pub fn python_name(self) -> &'static str {
        match self {
            ErrorKind::TypeError => "TypeError",
            ErrorKind::ValueError => "ValueError",
            ErrorKind::ZeroDivisionError => "ZeroDivisionError",
            ErrorKind::NotImplementedError => "NotImplementedError",
            ErrorKind::AttributeError => "AttributeError",
            ErrorKind::NameError => "NameError",
            ErrorKind::SyntaxError => "SyntaxError",
            ErrorKind::TokenError => "TokenError",
            ErrorKind::SympifyError => "SympifyError",
            ErrorKind::PrecisionExhausted => "PrecisionExhausted",
            ErrorKind::InconsistentAssumptions => "InconsistentAssumptions",
            ErrorKind::ComplexResult => "ComplexResult",
            ErrorKind::PolificationFailed => "PolificationFailed",
            ErrorKind::RecursionError => "RecursionError",
            ErrorKind::OverflowError => "OverflowError",
            ErrorKind::IndexError => "IndexError",
            ErrorKind::KeyError => "KeyError",
            ErrorKind::AssertionError => "AssertionError",
            ErrorKind::BadSignatureError => "BadSignatureError",
            ErrorKind::BadArgumentsError => "BadArgumentsError",
            ErrorKind::ArgumentIndexError => "ArgumentIndexError",
            ErrorKind::PolynomialError => "PolynomialError",
            ErrorKind::NotPortedYet => "NotPortedYet",
        }
    }

    /// `isinstance(err, ValueError)` and the like, for `except` clauses
    /// that name a base class.
    pub fn is_subclass_of(self, base: ErrorKind) -> bool {
        self == base
            || matches!(
                (self, base),
                (ErrorKind::InconsistentAssumptions, ErrorKind::ValueError)
                    | (ErrorKind::ComplexResult, ErrorKind::ValueError)
                    | (ErrorKind::SympifyError, ErrorKind::ValueError)
                    | (ErrorKind::TokenError, ErrorKind::SyntaxError)
                    | (ErrorKind::BadSignatureError, ErrorKind::TypeError)
                    | (ErrorKind::BadArgumentsError, ErrorKind::TypeError)
                    | (ErrorKind::ArgumentIndexError, ErrorKind::ValueError)
                    | (ErrorKind::PolificationFailed, ErrorKind::PolynomialError)
            )
    }
}

/// Re-raises an mpmath error as the Python exception mpmath raised.
pub fn raise_mpf(e: rsympy_mpf::MpfError) -> ! {
    use rsympy_mpf::MpfError as M;
    let kind = match e {
        M::ZeroDivision => ErrorKind::ZeroDivisionError,
        M::ComplexResult(_) => ErrorKind::ComplexResult,
        M::Value(_) => ErrorKind::ValueError,
        M::NotImplemented => ErrorKind::NotImplementedError,
    };
    raise(kind, e.to_string())
}

/// A raised exception.
#[derive(Clone, Debug)]
pub struct SympyError {
    pub kind: ErrorKind,
    pub message: String,
}

impl fmt::Display for SympyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind.python_name(), self.message)
    }
}

impl std::error::Error for SympyError {}

pub type SympyResult<T> = Result<T, SympyError>;

/// `raise kind(message)`.
pub fn raise(kind: ErrorKind, message: impl Into<String>) -> ! {
    install_quiet_hook();
    panic::panic_any(SympyError {
        kind,
        message: message.into(),
    })
}

/// Marks a SymPy code path that is not ported yet.
pub fn not_ported(what: impl Into<String>) -> ! {
    raise(ErrorKind::NotPortedYet, what)
}

/// Re-raises a caught error.
pub fn reraise(err: SympyError) -> ! {
    install_quiet_hook();
    panic::panic_any(err)
}

/// `try: f() except Exception as e: ...`: runs `f`, returning the raised
/// [`SympyError`] instead of unwinding. Other panics (bugs) keep unwinding.
pub fn catch<T>(f: impl FnOnce() -> T) -> SympyResult<T> {
    install_quiet_hook();
    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => Ok(v),
        Err(payload) => match payload.downcast::<SympyError>() {
            Ok(err) => Err(*err),
            Err(other) => panic::resume_unwind(other),
        },
    }
}

/// `try: f() except kind: ...`: catches only `kind` (and its subclasses).
pub fn catch_kind<T>(kind: ErrorKind, f: impl FnOnce() -> T) -> Option<T> {
    match catch(f) {
        Ok(v) => Some(v),
        Err(e) if e.kind.is_subclass_of(kind) => None,
        Err(e) => reraise(e),
    }
}

/// Raised errors are control flow, so they must not print a panic message.
/// Everything else goes to the previous hook.
fn install_quiet_hook() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if info.payload().downcast_ref::<SympyError>().is_none() {
                previous(info);
            }
        }));
    });
}
