"""Parsing LaTeX with SymPy's Lark backend (op "parse_latex"), as SymPy-parity cases.

Every case is the single call

    parse_latex(args[0], backend="lark")

from sympy.parsing.latex, the backend the homework app uses; options always
records {"backend": "lark"}. args[0] is the LaTeX as a student would type it.
The expected answer is SymPy's str() of whatever the call returns, or, when it
raises, the exception's class name (here always lark's UnexpectedCharacters).

The inputs are the ones where the Lark parser does something a reader might
not expect, which is where a port is most likely to diverge:

  - commands the grammar does not have raise UnexpectedCharacters: \\Delta,
    \\pi, \\sigma, \\circ, \\degree, \\angle, \\vec, \\hat, \\mathbf, \\in, ...;
    so do some subscripts (x_{12}, v_{0x}, a_{n-1}), a power followed by a
    letter (x^2y; x^{2}y fails the same way, since no implicit-product rule
    takes a power as its left factor) and a leading-dot decimal;
  - \\mathrm and \\text exist only inside the fixed differential tokens
    \\mathrm{d} and \\text{d}, so \\int x \\mathrm{d}x parses while
    \\mathrm{m/s} and \\text{ cm} raise UnexpectedCharacters;
  - an ambiguous input does not raise or pick a reading: the transformer has
    no rule for Lark's _ambig node, so parse_latex returns a lark Tree,
    Tree('_ambig', [reading, reading]), with the readings in Lark's order
    (e.g. \\ln x^{2}, \\sin(x)^{2}, 2\\sin x\\cos x, x(x+1));
  - a fraction whose denominator is d followed by an implicit-product
    factor returns the Python tuple ('derivative', factor) whatever the
    numerator is, printed (derivative, factor): \\frac{dy}{dx} gives
    ('derivative', x);
  - e is the plain Symbol e (not E), \\log is the natural log, typed decimals
    become Floats, \\overline is conjugate, a subscripted name is normalized
    to base_{sub} with braces added when none were typed (x_1 gives x_{1}),
    \\lim builds an unevaluated Limit and a pmatrix becomes a Matrix.
"""

from sympy.parsing.latex import parse_latex

from caselib import outcome

TOPIC = "latex_parse"
OP = "parse_latex"
BACKEND = "lark"


def _parse(src, note=None):
    return outcome(
        OP, [src], lambda: parse_latex(src, backend=BACKEND),
        options={"backend": BACKEND}, note=note,
    )


def _missing(src, command, note=None):
    """An input that uses a command the Lark grammar does not define."""
    return _parse(src, note or f"The Lark grammar has no {command} command.")


def deltas_and_derivatives():
    """Change and rate notation from physics and calculus items."""
    return [
        _missing(r"\Delta x", "\\Delta", "The Lark grammar knows \\delta but has no \\Delta."),
        _missing(r"\frac{\Delta y}{\Delta x}", "\\Delta", "The unknown \\Delta sits inside a fraction."),
        _missing(r"\Delta x^{2}", "\\Delta"),
        _missing(r"\Delta v_{x}", "\\Delta"),
        _missing(r"F\Delta t = m\Delta v", "\\Delta",
                 "The unknown \\Delta follows an implicit product, in an equation."),
        _parse(r"\frac{dy}{dx}",
               "A bare Leibniz fraction returns the Python tuple ('derivative', x), not a SymPy object."),
        _parse(r"\frac{d}{dx}x^{2}", "d/dx in front of an expression builds an unevaluated Derivative."),
        _parse(r"\frac{d^{2}y}{dx^{2}}",
               "Fails at the y after d^{2}: a power cannot be followed by a letter, "
               "so the input never reaches the fraction."),
        _parse(r"\frac{d}{t}",
               "The denominator t does not start with d, so the numerator is the plain "
               "Symbol d and the result is the quotient d/t. Only a denominator of d "
               "followed by an implicit-product factor gives the ('derivative', factor) "
               "tuple, whatever the numerator."),
    ]


def geometry_notation():
    return [
        _parse(r"\overline{AB}", "\\overline is complex conjugation, applied to the product A*B."),
        _parse("AB", "A run of capital letters is an implicit product of symbols."),
        _parse(r"AB^{2} + BC^{2} = AC^{2}", "Each exponent binds to the last letter only, giving A*B**2."),
        _parse(r"\frac{1}{2}bh", "An implicit product follows the fraction."),
        _missing(r"\angle ABC", "\\angle"),
        _missing(r"m\angle ABC = 90^{\circ}", "\\angle"),
        _missing(r"\triangle ABC", "\\triangle"),
        _missing(r"AB \cong CD", "\\cong"),
        _missing(r"AB \parallel CD", "\\parallel"),
    ]


def vector_notation():
    return [
        _missing(r"\vec{v}", "\\vec"),
        _missing(r"\hat{i}", "\\hat"),
        _missing(r"\mathbf{v}", "\\mathbf"),
        _missing(r"\overrightarrow{AB}", "\\overrightarrow"),
        _missing(r"3\hat{i} - 4\hat{j}", "\\hat", "The unknown \\hat follows a number, in a sum."),
    ]


def degree_marks():
    return [
        _missing(r"30^{\circ}", "\\circ", "The grammar has no \\circ, so a braced degree mark does not parse."),
        _missing(r"30^\circ", "\\circ", "The unbraced degree mark does not parse either."),
        _missing(r"30\degree", "\\degree"),
        _missing(r"\sin(30^{\circ})", "\\circ", "The degree mark is inside a function's parentheses."),
        _missing(r"(2x+10)^{\circ}", "\\circ", "The degree mark is on a parenthesised expression."),
        _missing(r"f \circ g", "\\circ", "\\circ as function composition is not in the grammar either."),
    ]


def other_commands():
    """Units, text, limits, sets and matrices."""
    return [
        _parse(r"\mathrm{m/s}",
               "The grammar has \\mathrm only in the fixed differential token \\mathrm{d}, "
               "so \\mathrm{m/s} matches no terminal."),
        _parse(r"9.8\,\mathrm{m/s^2}",
               "The \\, spacing parses; the parse fails at \\mathrm{m/s^2}, which is not the token \\mathrm{d}."),
        _parse(r"5\text{ cm}",
               "The grammar has \\text only in the fixed differential token \\text{d}, "
               "so \\text{ cm} matches no terminal."),
        _parse(r"\int x \mathrm{d}x", "\\mathrm{d} is the upright differential of an integral."),
        _parse(r"\lim_{x \to 0} \frac{\sin x}{x}",
               "A limit is built unevaluated, two-sided (dir='+-'), not computed."),
        _missing(r"x\in[0,1]", "\\in"),
        _parse(r"\begin{pmatrix}1&2\\3&4\end{pmatrix}", "A pmatrix environment becomes a Matrix."),
    ]


def subscripts():
    return [
        _parse("x_1", "x_1 becomes one Symbol named x_{1}: the transformer always "
                      "writes the subscript in braces, even when none were typed."),
        _parse(r"v_{0x}", "A braced subscript of a digit then a letter does not tokenize."),
        _parse(r"F_{net}", "A braced all-letter subscript is part of one Symbol's name."),
        _parse(r"a_{c}", "A braced one-letter subscript."),
        _parse(r"x_{12}", "A braced multi-digit subscript does not tokenize, though x_1 does."),
        _parse(r"v_{0}t + \frac{1}{2}at^{2}", "Subscripted symbols take part in implicit products."),
        _parse(r"v_x^{2}", "The exponent applies to the subscripted symbol."),
        _parse("a_n"),
        _parse(r"a_{n-1}", "A subscript containing an operator does not tokenize."),
        _parse(r"a_{n+1} = 2a_n", "A recurrence: the subscript with an operator fails first."),
    ]


def binding():
    """Where exponents and function arguments attach, and ambiguous parses."""
    ambiguous = "Ambiguous: Lark returns a Tree('_ambig') holding both readings."
    return [
        _parse(r"400e^{\ln(0.75)t}",
               "e is the Symbol e, and the argument of \\ln(0.75) runs on over the following t."),
        _parse(r"\ln\sqrt{x}^{2}", ambiguous),
        _parse(r"\sin 2x", "An unparenthesised argument runs over the implicit product 2x."),
        _parse(r"\ln x^{2}", ambiguous),
        _parse(r"\sin x^{2}", ambiguous),
        _parse(r"\sin(x)^{2}", "Ambiguous even with parentheses: both readings come back in a Tree('_ambig')."),
        _parse(r"\ln x + \ln y", ambiguous + " One reading is log(x + log(y))."),
        _parse(r"2\sin x\cos x", ambiguous + " One reading is 2*sin(x*cos(x))."),
        _parse(r"\sin^{2}x+\cos^{2}x", ambiguous + " One reading is sin(x + cos(x)**2)**2."),
        _parse(r"\sqrt{x}^{2}", "sqrt(x)**2 evaluates to x at construction."),
        _parse(r"x^2y", "A power followed by a letter does not parse: no implicit-product rule "
               "takes a power as its left factor."),
        _parse(r"x^{2}y", "Bracing the exponent does not help: a power still cannot be followed by a letter."),
        _parse(r"x(x+1)", ambiguous + " One reading applies an undefined function x to x + 1."),
        _parse(r"-x^2", "Unary minus versus the power."),
        _parse(r"-2^2", "Unary minus versus the power, on numbers."),
    ]


def constants_and_decimals():
    no_pi = "The Lark grammar has no \\pi command."
    return [
        _parse("e", "A lone e is the plain Symbol e, not Euler's number E."),
        _parse(r"e^{x}", "A power of the Symbol e, not exp(x)."),
        _parse(r"e^{\ln 5}", "With e a Symbol, e**log(5) does not reduce to 5."),
        _parse(r"\pi", no_pi),
        _parse(r"2\pi r", no_pi),
        _parse(r"\frac{3\pi}{4}", no_pi),
        _parse(r"\cos\left(\frac{\pi}{3}\right)", no_pi),
        _parse("0.75", "A typed decimal becomes a Float."),
        _parse("0.1+0.2", "The Floats are added at construction."),
        _parse("1.5x", "A Float coefficient."),
        _parse(".5", "A decimal without a leading digit does not tokenize."),
        _parse("-0.5", "A negative Float."),
        _parse(r"x^{0.5}", "A Float exponent, not sqrt(x)."),
        _parse(r"(0.75)^{t}", "A Float base."),
    ]


def implicit_multiplication():
    return [
        _parse("2x"),
        _parse("3(x+1)", "SymPy distributes a number over a sum at construction."),
        _parse("xy"),
    ]


def fractions_and_roots():
    return [
        _parse(r"\frac12", "Brace-less \\frac with two digits."),
        _parse(r"\frac{x^2-1}{x-1}", "Parsing does not cancel the removable factor."),
        _parse(r"\frac{1}{1+\frac{1}{x}}", "A nested fraction stays nested."),
        _parse(r"\sqrt{50}", "SymPy simplifies the radical at construction."),
        _parse(r"\sqrt[3]{x}"),
        _parse(r"\sqrt{x^{2}}", "Without assumptions sqrt(x**2) does not simplify."),
    ]


def operators_and_grouping():
    return [
        _parse(r"\left(x+1\right)^2"),
        _parse(r"2\cdot3"),
        _parse(r"2 \times x"),
        _parse(r"6\div 3"),
        _parse("|x|"),
        _parse(r"\left|x\right|"),
        _parse(r"|2x+1| = 5"),
    ]


def functions():
    return [
        _parse(r"\sin^{-1} x", "^{-1} on a trig function is the inverse function, not a reciprocal."),
        _parse(r"\arcsin x"),
        _parse(r"\cos^{-1}\left(\frac{1}{2}\right)", "acos(1/2) evaluates to pi/3 at construction."),
        _parse(r"\log_2 x"),
        _parse(r"\log_{10}(x)"),
        _parse(r"\log_2 8", "An exact integer logarithm evaluates."),
        _parse(r"\log x", "An unsubscripted \\log is the natural log, the same as \\ln."),
        _parse(r"\log(100)", "The natural log of 100 stays unevaluated."),
        _parse(r"\ln x"),
        _parse(r"2\ln x"),
        _parse(r"x^{-1/2}"),
        _missing(r"\sigma", "\\sigma", "The grammar's Greek letters leave out \\sigma (and \\pi)."),
        _parse(r"\sin\theta", "A Greek letter is a valid unparenthesised function argument."),
    ]


def signs_relations_infinity():
    return [
        _parse("-2x+5"),
        _parse("-(x-3)"),
        _parse("x = 3"),
        _parse(r"2x+1 \le 5"),
        _parse(r"2x - 1 > 3", "A strict inequality is a StrictGreaterThan."),
        _parse(r"x \geq -2"),
        _parse(r"x \ne 0"),
        _parse(r"\infty"),
        _parse(r"-\infty"),
    ]


def generate():
    return (
        deltas_and_derivatives()
        + geometry_notation()
        + vector_notation()
        + degree_marks()
        + other_commands()
        + subscripts()
        + binding()
        + constants_and_decimals()
        + implicit_multiplication()
        + fractions_and_roots()
        + operators_and_grouping()
        + functions()
        + signs_relations_infinity()
    )
