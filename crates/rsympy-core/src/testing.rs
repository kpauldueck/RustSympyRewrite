//! The oracle's expression encoding, for differential tests.
//!
//! Test vectors carry SymPy objects as text: `srepr(e, order='none')`
//! (args in storage order rather than printing order), with two
//! extensions so the encoding is lossless:
//!
//! - an undefined function prints its assumptions,
//!   `Function('f', real=True, ...)(x)`, listing all of `_kwargs` in
//!   alphabetical order (with `nargs=...` when it was given);
//! - nothing else: `Float('...', precision=p)` round-trips exactly because
//!   `srepr` prints `repr_dps(p)` digits.
//!
//! [`parse`] builds the tree without evaluating anything (each node is
//! `Basic.__new__(cls, *args)`, or `_from_args` for Add and Mul), so a
//! module can be tested on exactly the inputs SymPy handed it. [`encode`]
//! is the inverse; `Debug` for [`Expr`] uses it. The oracle side is
//! `oracle/exprenc.py`.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use rsympy_num::Int;

use crate::error::{ErrorKind, not_ported, raise};
use crate::expr::{Data, Expr};
use crate::facts::Fact;
use crate::generated::{ALL_CLASSES, CLASSES, ClassId as C};
use crate::numbers::{self, s};
use crate::symbol;

/// Parses the oracle's encoding of a SymPy object into a raw tree.
///
/// Raises `ValueError` for malformed text and `NotPortedYet` for classes
/// rsympy does not have yet.
pub fn parse(text: &str) -> Expr {
    let mut p = Parser {
        src: text.as_bytes(),
        pos: 0,
    };
    let v = p.expr();
    p.skip_ws();
    if p.pos != p.src.len() {
        p.fail("trailing text");
    }
    v
}

/// Encodes `e` the way the oracle does (see the module docs).
pub fn encode(e: &Expr) -> String {
    let mut out = String::new();
    write_expr(&mut out, e);
    out
}

// ---------------------------------------------------------------------------
// parsing

static CLASS_BY_NAME: LazyLock<HashMap<&'static str, C>> = LazyLock::new(|| {
    CLASSES
        .iter()
        .zip(ALL_CLASSES)
        .map(|(c, id)| (c.name, id))
        .collect()
});

/// A name, its `fact=bool` keywords and its integer keywords.
type NamedArgs = (String, Vec<(Fact, bool)>, Vec<(String, Int)>);

/// A `Function`'s name, its `fact=bool` keywords and its `nargs=`.
type FunctionArgs = (
    String,
    Vec<(Fact, bool)>,
    Option<crate::functions::NargsArg>,
);

enum Val {
    Expr(Expr),
    Int(Int),
    Str(String),
    Bool(bool),
    None,
    /// A Python tuple, `(1,)` or `(1, 2)`.
    Tuple(Vec<Val>),
}

struct Arg {
    key: Option<String>,
    val: Val,
}

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn fail(&self, what: &str) -> ! {
        let text = String::from_utf8_lossy(self.src);
        raise(
            ErrorKind::ValueError,
            format!(
                "bad expression encoding at byte {}: {what}: {text}",
                self.pos
            ),
        )
    }

    fn skip_ws(&mut self) {
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_ws();
        self.src.get(self.pos).copied()
    }

    fn eat(&mut self, c: u8) -> bool {
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, c: u8) {
        if !self.eat(c) {
            self.fail(&format!("expected '{}'", c as char));
        }
    }

    fn ident(&mut self) -> String {
        self.skip_ws();
        let start = self.pos;
        while self.pos < self.src.len()
            && (self.src[self.pos].is_ascii_alphanumeric() || self.src[self.pos] == b'_')
        {
            self.pos += 1;
        }
        if start == self.pos {
            self.fail("expected a name");
        }
        String::from_utf8_lossy(&self.src[start..self.pos]).into_owned()
    }

    fn int(&mut self) -> Int {
        self.skip_ws();
        let start = self.pos;
        if self.src.get(self.pos) == Some(&b'-') {
            self.pos += 1;
        }
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        let text = std::str::from_utf8(&self.src[start..self.pos]).expect("ascii");
        text.parse()
            .unwrap_or_else(|_| self.fail("expected an integer"))
    }

    /// A Python string literal, `'...'` or `"..."`.
    fn string(&mut self) -> String {
        let quote = self
            .peek()
            .unwrap_or_else(|| self.fail("expected a string"));
        self.pos += 1;
        let mut bytes = Vec::new();
        loop {
            let Some(&c) = self.src.get(self.pos) else {
                self.fail("unterminated string")
            };
            self.pos += 1;
            if c == quote {
                break;
            }
            if c != b'\\' {
                bytes.push(c);
                continue;
            }
            let Some(&e) = self.src.get(self.pos) else {
                self.fail("unterminated escape")
            };
            self.pos += 1;
            let ch = match e {
                b'\\' => '\\',
                b'\'' => '\'',
                b'"' => '"',
                b'n' => '\n',
                b't' => '\t',
                b'r' => '\r',
                b'x' | b'u' | b'U' => {
                    let n = match e {
                        b'x' => 2,
                        b'u' => 4,
                        _ => 8,
                    };
                    let hex = self
                        .src
                        .get(self.pos..self.pos + n)
                        .unwrap_or_else(|| self.fail("short escape"));
                    self.pos += n;
                    let v = u32::from_str_radix(std::str::from_utf8(hex).unwrap_or(""), 16)
                        .unwrap_or_else(|_| self.fail("bad escape"));
                    char::from_u32(v).unwrap_or_else(|| self.fail("bad code point"))
                }
                _ => self.fail("unknown escape"),
            };
            let mut buf = [0u8; 4];
            bytes.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
        }
        String::from_utf8(bytes).unwrap_or_else(|_| self.fail("string is not UTF-8"))
    }

    fn args(&mut self) -> Vec<Arg> {
        self.expect(b'(');
        let mut out = Vec::new();
        if self.eat(b')') {
            return out;
        }
        loop {
            out.push(self.arg());
            if self.eat(b')') {
                return out;
            }
            self.expect(b',');
        }
    }

    fn arg(&mut self) -> Arg {
        let save = self.pos;
        if self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
            let name = self.ident();
            if self.eat(b'=') {
                let val = self.value();
                return Arg {
                    key: Some(name),
                    val,
                };
            }
            self.pos = save;
        }
        Arg {
            key: None,
            val: self.value(),
        }
    }

    fn value(&mut self) -> Val {
        match self.peek() {
            Some(b'(') => {
                self.pos += 1;
                let mut items = Vec::new();
                if self.eat(b')') {
                    return Val::Tuple(items);
                }
                loop {
                    items.push(self.value());
                    if self.eat(b')') {
                        if items.len() == 1 {
                            self.fail("a 1-tuple needs its trailing comma");
                        }
                        return Val::Tuple(items);
                    }
                    self.expect(b',');
                    if self.eat(b')') {
                        return Val::Tuple(items);
                    }
                }
            }
            Some(b'\'' | b'"') => Val::Str(self.string()),
            Some(c) if c.is_ascii_digit() => Val::Int(self.int()),
            Some(b'-') if self.src.get(self.pos + 1).is_some_and(u8::is_ascii_digit) => {
                Val::Int(self.int())
            }
            _ => {
                let save = self.pos;
                if self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
                    match self.ident().as_str() {
                        "True" => return Val::Bool(true),
                        "False" => return Val::Bool(false),
                        "None" => return Val::None,
                        _ => self.pos = save,
                    }
                }
                Val::Expr(self.expr())
            }
        }
    }

    fn expr(&mut self) -> Expr {
        if self.eat(b'-') {
            let e = self.expr();
            if e.is_cls(C::Infinity) {
                return s::negative_infinity();
            }
            self.fail("unary minus is only written for -oo");
        }
        let name = self.ident();
        match name.as_str() {
            "oo" => return s::infinity(),
            "zoo" => return s::complex_infinity(),
            "nan" => return s::nan(),
            "I" => return s::imaginary_unit(),
            "E" => return s::exp1(),
            "pi" => return s::pi(),
            "EulerGamma" => return s::euler_gamma(),
            "Catalan" => return s::catalan(),
            "GoldenRatio" => return s::golden_ratio(),
            "TribonacciConstant" => return s::tribonacci_constant(),
            "true" => return s::true_(),
            "false" => return s::false_(),
            "EmptySet" => return s::empty_set(),
            _ => {}
        }
        if self.peek() != Some(b'(') {
            self.fail(&format!("unknown name {name}"));
        }
        let args = self.args();
        match name.as_str() {
            "Integer" => match args.as_slice() {
                [
                    Arg {
                        key: None,
                        val: Val::Int(p),
                    },
                ] => Expr::integer(p.clone()),
                _ => self.fail("Integer(p)"),
            },
            "Rational" => match args.as_slice() {
                [
                    Arg {
                        key: None,
                        val: Val::Int(p),
                    },
                    Arg {
                        key: None,
                        val: Val::Int(q),
                    },
                ] => Expr::rational(p.clone(), q.clone()),
                _ => self.fail("Rational(p, q)"),
            },
            "Float" => match args.as_slice() {
                [
                    Arg {
                        key: None,
                        val: Val::Str(t),
                    },
                    Arg {
                        key: Some(k),
                        val: Val::Int(prec),
                    },
                ] if k == "precision" => {
                    let prec = prec.as_i64().unwrap_or_else(|| self.fail("precision"));
                    numbers::float_from_str(t, None, Some(prec))
                }
                _ => self.fail("Float('digits', precision=p)"),
            },
            "Str" => match args.as_slice() {
                [
                    Arg {
                        key: None,
                        val: Val::Str(t),
                    },
                ] => Expr::make(C::Str, Data::Str(Arc::from(t.as_str())), vec![]),
                _ => self.fail("Str('name')"),
            },
            "Symbol" | "Dummy" | "Wild" => self.symbol(&name, &args),
            "WildFunction" => match args.as_slice() {
                // WildFunction('F'): the constructor, as the oracle's
                // decoder calls it (it sympifies the name)
                [
                    Arg {
                        key: None,
                        val: Val::Str(t),
                    },
                ] => crate::functions::application::wild_function(t, None),
                _ => self.fail("WildFunction('name')"),
            },
            "Function" => {
                let (fname, assumptions, nargs) = self.function_args(&args);
                let f =
                    crate::functions::application::undefined_function(&fname, &assumptions, nargs);
                let applied = self.args();
                let applied = self.exprs(applied);
                let kb = f.assumptions;
                Expr::make_with_kb(C::AppliedUndef, Data::Undef(f), applied, kb)
            }
            _ => {
                let Some(&cls) = CLASS_BY_NAME.get(name.as_str()) else {
                    not_ported(format!("class {name} in an encoded expression"))
                };
                let args = self.exprs(args);
                crate::construct::raw(cls, args)
            }
        }
    }

    fn exprs(&self, args: Vec<Arg>) -> Vec<Expr> {
        args.into_iter()
            .map(|a| match a {
                Arg {
                    key: None,
                    val: Val::Expr(e),
                } => e,
                _ => self.fail("expected an expression argument"),
            })
            .collect()
    }

    /// `Function`'s `('name', fact=bool, ..., nargs=...)`.
    fn function_args(&self, args: &[Arg]) -> FunctionArgs {
        use crate::functions::NargsArg;
        let Some(Arg {
            key: None,
            val: Val::Str(name),
        }) = args.first()
        else {
            self.fail("expected a name string")
        };
        let small = |i: &Int| i.as_i64().unwrap_or_else(|| self.fail("nargs"));
        let mut facts = Vec::new();
        let mut nargs = None;
        for a in &args[1..] {
            match (a.key.as_deref(), &a.val) {
                (Some("nargs"), Val::None) => nargs = Some(NargsArg::None),
                (Some("nargs"), Val::Int(i)) => nargs = Some(NargsArg::Int(small(i))),
                (Some("nargs"), Val::Tuple(items)) => {
                    let v = items
                        .iter()
                        .map(|x| match x {
                            Val::Int(i) => small(i),
                            _ => self.fail("nargs=(int, ...)"),
                        })
                        .collect();
                    nargs = Some(NargsArg::Tuple(v));
                }
                (Some(k), Val::Bool(b)) => {
                    let f = Fact::from_name(k).unwrap_or_else(|| self.fail("unknown fact"));
                    facts.push((f, *b));
                }
                _ => self.fail("expected fact=True/False or nargs=..."),
            }
        }
        (name.clone(), facts, nargs)
    }

    /// `('name', fact=bool, ...)`, plus the integer keywords in `extra`.
    fn name_and_assumptions(&self, args: &[Arg], extra: &[&str]) -> NamedArgs {
        let Some(Arg {
            key: None,
            val: Val::Str(name),
        }) = args.first()
        else {
            self.fail("expected a name string")
        };
        let mut facts = Vec::new();
        let mut ints = Vec::new();
        for a in &args[1..] {
            match (&a.key, &a.val) {
                (Some(k), Val::Bool(b)) => {
                    let f = Fact::from_name(k).unwrap_or_else(|| self.fail("unknown fact"));
                    facts.push((f, *b));
                }
                (Some(k), Val::Int(i)) if extra.contains(&k.as_str()) => {
                    ints.push((k.clone(), i.clone()));
                }
                _ => self.fail("expected fact=True/False"),
            }
        }
        (name.clone(), facts, ints)
    }

    fn symbol(&self, cls: &str, args: &[Arg]) -> Expr {
        let (name, facts, ints) = self.name_and_assumptions(args, &["dummy_index"]);
        match cls {
            "Symbol" => symbol::symbol_with(&name, &facts),
            "Wild" => symbol::wild(&name, &facts),
            _ => {
                let [(_, index)] = ints.as_slice() else {
                    self.fail("Dummy needs dummy_index")
                };
                let index = index
                    .as_i64()
                    .and_then(|i| u64::try_from(i).ok())
                    .unwrap_or_else(|| self.fail("dummy_index"));
                symbol::dummy_with_index(&name, index, &facts)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// encoding

fn write_expr(out: &mut String, e: &Expr) {
    let name = match e.cls() {
        C::Integer | C::Zero | C::One | C::NegativeOne => {
            out.push_str(&format!("Integer({})", e.p()));
            return;
        }
        C::Rational | C::Half => {
            out.push_str(&format!("Rational({}, {})", e.p(), e.q()));
            return;
        }
        C::Float => {
            let (m, prec) = e.as_float().expect("a Float");
            let digits = rsympy_mpf::to_str(m, rsympy_mpf::repr_dps(prec), true, None, None, false);
            out.push_str(&format!("Float('{digits}', precision={prec})"));
            return;
        }
        C::Infinity => "oo",
        C::NegativeInfinity => "-oo",
        C::ComplexInfinity => "zoo",
        C::NaN => "nan",
        C::ImaginaryUnit => "I",
        C::Exp1 => "E",
        C::Pi => "pi",
        C::BooleanTrue => "true",
        C::BooleanFalse => "false",
        C::Symbol | C::Dummy | C::Wild => {
            let d = e.symbol_data().expect("a symbol");
            out.push_str(e.class_name());
            out.push('(');
            out.push_str(&crate::py::repr_str(&d.name));
            for &(f, v) in &d.assumptions_orig {
                out.push_str(&format!(", {}={}", f.name(), py_bool(v)));
            }
            if let Some(i) = d.dummy_index {
                out.push_str(&format!(", dummy_index={i}"));
            }
            out.push(')');
            return;
        }
        C::Str => {
            let name = e.name().expect("Str has a name");
            out.push_str(&format!("Str({})", crate::py::repr_str(name)));
            return;
        }
        C::AppliedUndef => {
            let u = e.undef_data().expect("AppliedUndef has its function");
            out.push_str("Function(");
            out.push_str(&crate::py::repr_str(&u.name));
            for (k, v) in crate::functions::application::undefined_kwargs_repr(u) {
                out.push_str(&format!(", {k}={v}"));
            }
            out.push(')');
            write_args(out, e.args());
            return;
        }
        C::WildFunction => {
            // ReprPrinter._print_WildFunction: "%s('%s')" % (cls, expr.name)
            use crate::expr::WildFunctionName as N;
            let d = e.wild_function_data().expect("a WildFunction");
            let name = match &d.name {
                N::Str(t) => t.to_string(),
                N::Expr(x) if x.is_Symbol() => x.name().expect("a Symbol's name").to_string(),
                N::Expr(x) => not_ported(format!("str({x:?}) as a WildFunction name")),
                N::Unset => raise(
                    ErrorKind::AttributeError,
                    "'WildFunction' object has no attribute 'name'",
                ),
            };
            out.push_str(&format!("WildFunction('{name}')"));
            return;
        }
        _ => e.class_name(),
    };
    out.push_str(name);
    // Atoms (EulerGamma, ...) and the EmptySet singleton print bare;
    // everything else as a call, even with no args (`IdentityFunction()`).
    if !(e.is_Atom() || e.is_cls(C::EmptySet)) {
        write_args(out, e.args());
    }
}

fn write_args(out: &mut String, args: &[Expr]) {
    out.push('(');
    for (i, a) in args.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        write_expr(out, a);
    }
    out.push(')');
}

fn py_bool(v: bool) -> &'static str {
    if v { "True" } else { "False" }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Strings printed by SymPy 1.14's `srepr(e, order='none')`.
    const SAMPLES: &[&str] = &[
        "Integer(0)",
        "Integer(-1)",
        "Integer(123456789012345678901234567890)",
        "Rational(1, 2)",
        "Rational(-3, 7)",
        "Float('1.5', precision=53)",
        "Float('0.0999999999999999999999999999999951', precision=103)",
        "Float('0.0', precision=53)",
        "-oo",
        "zoo",
        "nan",
        "I",
        "pi",
        "EulerGamma",
        "Symbol('y', positive=True)",
        "Symbol('z', commutative=False)",
        "Symbol(\"it's\")",
        "Symbol('a\\\\b')",
        "Dummy('d', dummy_index=1708795)",
        "Wild('w')",
        "Function('f')(Symbol('x'))",
        "exp(Symbol('x'))",
        "Tuple(Integer(1), Symbol('x'))",
        "Str('s')",
        "Equality(Symbol('x'), Integer(1))",
        "Add(Symbol('x'), Integer(1))",
        "Mul(Integer(-1), Symbol('x'))",
        "Pow(Symbol('x'), Integer(2))",
        "Integral(Symbol('x'), Tuple(Symbol('x'), Integer(0), Integer(1)))",
        "Piecewise(ExprCondPair(Symbol('x'), StrictGreaterThan(Symbol('x'), Integer(0))), \
         ExprCondPair(Integer(0), true))",
        "Interval(Integer(0), Integer(1), false, false)",
        "EmptySet",
    ];

    #[test]
    fn round_trips() {
        for &s in SAMPLES {
            assert_eq!(encode(&parse(s)), s);
        }
    }

    #[test]
    fn builds_raw_nodes() {
        let e = parse("Add(Integer(1), Integer(1))");
        assert!(e.is_Add());
        assert_eq!(e.args().len(), 2);
        let m = parse("Mul(Symbol('A', commutative=False), Symbol('x'))");
        assert_eq!(m.is_commutative(), Some(false));
        assert_eq!(parse("Integer(1)").cls(), C::One);
        assert_eq!(parse("Rational(1, 2)").cls(), C::Half);
    }

    #[test]
    fn function_assumptions() {
        let e = parse("Function('g', real=True)(Symbol('x'))");
        assert_eq!(e.is_real(), Some(true));
        assert_eq!(e.is_commutative(), Some(true));
        // Same deduced kwargs, so the same class.
        assert_eq!(
            parse("Function('g', complex=True, real=True)(Symbol('x'))"),
            e
        );
        // An explicit commutative=True is kept in _kwargs, so SymPy's
        // Function('g', real=True) != Function('g', real=True, commutative=True).
        assert_ne!(
            parse("Function('g', commutative=True, real=True)(Symbol('x'))"),
            e
        );
        assert_eq!(
            encode(&e),
            "Function('g', complex=True, extended_real=True, finite=True, hermitian=True, \
             imaginary=False, infinite=False, real=True)(Symbol('x'))"
        );
    }
}
