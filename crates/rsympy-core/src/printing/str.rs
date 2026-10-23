//! `sympy.printing.str.StrPrinter`.
//!
//! [`StrPrinter::print`] is `Printer._print`: it counts `_print_level` and
//! dispatches to the `_print_<Class>` method that `Printer._print` picks
//! for the node's class (recorded per class in `generated_compare.rs`,
//! including the `AppliedUndef`/`Function` MRO exceptions). Each method is
//! one `print_<Class>` function here. The atoms are ported; the other
//! methods raise `NotPortedYet` naming the method.

use std::cell::Cell;

use rsympy_mpf as mlib;

use crate::error::not_ported;
use crate::expr::Expr;
use crate::generated_compare::COMPARE_INFO;

/// `StrPrinter`'s `full_prec` setting: `True`, `False` or `"auto"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FullPrec {
    True,
    False,
    Auto,
}

/// `StrPrinter._default_settings`, with values as given to `sstr(expr,
/// **settings)`.
#[derive(Clone, Debug, PartialEq)]
pub struct StrSettings {
    /// `order`: a monomial order name, `'none'`, `'old'`, or `None`.
    pub order: Option<String>,
    pub full_prec: FullPrec,
    pub sympy_integers: bool,
    pub abbrev: bool,
    pub perm_cyclic: bool,
    /// `min`: `min_fixed` for Floats.
    pub min: Option<i64>,
    /// `max`: `max_fixed` for Floats.
    pub max: Option<i64>,
    /// `dps`: the digits Floats print with (default: from their precision).
    pub dps: Option<i64>,
}

impl Default for StrSettings {
    fn default() -> Self {
        StrSettings {
            order: None,
            full_prec: FullPrec::Auto,
            sympy_integers: false,
            abbrev: false,
            perm_cyclic: true,
            min: None,
            max: None,
            dps: None,
        }
    }
}

/// A `StrPrinter(settings)`.
pub struct StrPrinter {
    pub settings: StrSettings,
    /// `_print_level`: how deep `_print` calls are nested.
    print_level: Cell<usize>,
}

/// Restores `_print_level` when a `_print` call ends (the `finally:`).
struct LevelGuard<'a>(&'a Cell<usize>);

impl Drop for LevelGuard<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}

#[allow(non_snake_case)]
impl StrPrinter {
    pub fn new(settings: StrSettings) -> StrPrinter {
        StrPrinter {
            settings,
            print_level: Cell::new(0),
        }
    }

    /// `self._print_level`.
    pub fn print_level(&self) -> usize {
        self.print_level.get()
    }

    /// `self._print_level = level`.
    pub fn set_print_level(&self, level: usize) {
        self.print_level.set(level);
    }

    /// `self.doprint(expr)`.
    pub fn doprint(&self, e: &Expr) -> String {
        self.print(e)
    }

    /// `self._print(expr)`: `Printer._print` for a SymPy object.
    pub fn print(&self, e: &Expr) -> String {
        self.print_level.set(self.print_level.get() + 1);
        let _guard = LevelGuard(&self.print_level);
        self.dispatch(e)
    }

    /// `self._print(s)` for a Python `str` (`_print_str`).
    pub fn print_py_str(&self, s: &str) -> String {
        self.print_level.set(self.print_level.get() + 1);
        let _guard = LevelGuard(&self.print_level);
        self.print_str(s)
    }

    /// The method `Printer._print` finds for `e`'s class.
    fn dispatch(&self, e: &Expr) -> String {
        let method = COMPARE_INFO[e.cls() as usize].str_print;
        match method {
            "_print_Integer" => self.print_Integer(e),
            "_print_Zero" => self.print_Zero(e),
            "_print_Rational" => self.print_Rational(e),
            "_print_Float" => self.print_Float(e),
            "_print_Infinity" => self.print_Infinity(e),
            "_print_NegativeInfinity" => self.print_NegativeInfinity(e),
            "_print_NaN" => self.print_NaN(e),
            "_print_ComplexInfinity" => self.print_ComplexInfinity(e),
            "_print_ImaginaryUnit" => self.print_ImaginaryUnit(e),
            "_print_Exp1" => self.print_Exp1(e),
            "_print_Pi" => self.print_Pi(e),
            "_print_EulerGamma" => self.print_EulerGamma(e),
            "_print_Catalan" => self.print_Catalan(e),
            "_print_GoldenRatio" => self.print_GoldenRatio(e),
            "_print_TribonacciConstant" => self.print_TribonacciConstant(e),
            "_print_Symbol" => self.print_Symbol(e),
            "_print_Dummy" => self.print_Dummy(e),
            "_print_Wild" => self.print_Wild(e),
            "_print_BooleanTrue" => self.print_BooleanTrue(e),
            "_print_BooleanFalse" => self.print_BooleanFalse(e),
            "_print_Str" => self.print_Str(e),
            // An object printing itself (`Indexed._sympystr`, ...).
            m if !m.starts_with("_print_") && m != "emptyPrinter" => not_ported(m),
            m => not_ported(format!("StrPrinter.{m}")),
        }
    }

    /// `_print_Integer`.
    pub fn print_Integer(&self, e: &Expr) -> String {
        if self.settings.sympy_integers {
            // "S(%s)" % (expr): str(expr) with default settings
            return format!("S({})", sstr(e));
        }
        e.p().to_string()
    }

    /// `_print_Zero`.
    pub fn print_Zero(&self, e: &Expr) -> String {
        if self.settings.sympy_integers {
            return "S(0)".to_string();
        }
        // self._print_Integer(Integer(0))
        self.print_Integer(e)
    }

    /// `_print_Rational`.
    pub fn print_Rational(&self, e: &Expr) -> String {
        let (p, q) = (e.p(), e.q());
        if q.is_one() {
            return p.to_string();
        }
        if self.settings.sympy_integers {
            return format!("S({p})/{q}");
        }
        format!("{p}/{q}")
    }

    /// `_print_Float`.
    pub fn print_Float(&self, e: &Expr) -> String {
        let (mpf, prec) = e.as_float().expect("a Float");
        let dps = match self.settings.dps {
            Some(d) => d,
            None if prec < 5 => 0,
            None => mlib::prec_to_dps(prec),
        };
        let strip = match self.settings.full_prec {
            FullPrec::True => false,
            FullPrec::False => true,
            FullPrec::Auto => self.print_level() > 1,
        };
        let rv = mlib::to_str(mpf, dps, strip, self.settings.min, self.settings.max, false);
        let rv = if let Some(rest) = rv.strip_prefix("-.0") {
            format!("-0.{rest}")
        } else if let Some(rest) = rv.strip_prefix(".0") {
            format!("0.{rest}")
        } else {
            rv
        };
        // e.g., +inf -> inf
        match rv.strip_prefix('+') {
            Some(rest) => rest.to_string(),
            None => rv,
        }
    }

    /// `_print_Infinity`.
    pub fn print_Infinity(&self, _e: &Expr) -> String {
        "oo".to_string()
    }

    /// `_print_NegativeInfinity`.
    pub fn print_NegativeInfinity(&self, _e: &Expr) -> String {
        "-oo".to_string()
    }

    /// `_print_NaN`.
    pub fn print_NaN(&self, _e: &Expr) -> String {
        "nan".to_string()
    }

    /// `_print_ComplexInfinity`.
    pub fn print_ComplexInfinity(&self, _e: &Expr) -> String {
        "zoo".to_string()
    }

    /// `_print_ImaginaryUnit`.
    pub fn print_ImaginaryUnit(&self, _e: &Expr) -> String {
        "I".to_string()
    }

    /// `_print_Exp1`.
    pub fn print_Exp1(&self, _e: &Expr) -> String {
        "E".to_string()
    }

    /// `_print_Pi`.
    pub fn print_Pi(&self, _e: &Expr) -> String {
        "pi".to_string()
    }

    /// `_print_EulerGamma`.
    pub fn print_EulerGamma(&self, _e: &Expr) -> String {
        "EulerGamma".to_string()
    }

    /// `_print_Catalan`.
    pub fn print_Catalan(&self, _e: &Expr) -> String {
        "Catalan".to_string()
    }

    /// `_print_GoldenRatio`.
    pub fn print_GoldenRatio(&self, _e: &Expr) -> String {
        "GoldenRatio".to_string()
    }

    /// `_print_TribonacciConstant`.
    pub fn print_TribonacciConstant(&self, _e: &Expr) -> String {
        "TribonacciConstant".to_string()
    }

    /// `_print_Symbol`.
    pub fn print_Symbol(&self, e: &Expr) -> String {
        e.name().expect("a Symbol").to_string()
    }

    /// `_print_Dummy`.
    pub fn print_Dummy(&self, e: &Expr) -> String {
        format!("_{}", e.name().expect("a Dummy"))
    }

    /// `_print_Wild`.
    pub fn print_Wild(&self, e: &Expr) -> String {
        format!("{}_", e.name().expect("a Wild"))
    }

    /// `_print_BooleanTrue`.
    pub fn print_BooleanTrue(&self, _e: &Expr) -> String {
        "True".to_string()
    }

    /// `_print_BooleanFalse`.
    pub fn print_BooleanFalse(&self, _e: &Expr) -> String {
        "False".to_string()
    }

    /// `_print_Str`: `self._print(s.name)`.
    pub fn print_Str(&self, e: &Expr) -> String {
        self.print_py_str(e.name().expect("a Str"))
    }

    /// `_print_str`.
    pub fn print_str(&self, s: &str) -> String {
        s.to_string()
    }
}

/// `sstr(expr, **settings)`.
pub fn sstr_with(e: &Expr, settings: StrSettings) -> String {
    StrPrinter::new(settings).doprint(e)
}

/// `sstr(expr)` (`str(expr)`) with default settings.
pub fn sstr(e: &Expr) -> String {
    sstr_with(e, StrSettings::default())
}
