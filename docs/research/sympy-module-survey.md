# SymPy 1.14 module survey for the Rust rewrite

Generated from a code-reading survey of every planning unit in SymPy 1.14.0 (eight parallel surveys, one per cluster, 2026-10-22). Line and test counts come from `oracle/inventory.py` (`oracle/sympy_inventory.json`). Difficulty assumes a skilled team porting to idiomatic Rust with SymPy as the oracle: S under a person-week, M 1–4 weeks, L 1–3 months, XL over 3 months. This is research input for [`../PLAN.md`](../PLAN.md); the plan wins where they differ.

## Summary

| Unit | Cluster | Src lines | Test fns | Difficulty | Strategy | App |
|---|---|---:|---:|:-:|---|---|
| `polys` | polys | 49,017 | 907 | XL | port faithfully | serve |
| `core` | core | 34,576 | 2,066 | XL | port faithfully | serve |
| `combinatorics` | discrete | 21,444 | 175 | L | port faithfully | none |
| `printing` | io | 20,048 | 1,068 | XL | port faithfully | serve |
| `matrices` | solvers_matrices | 19,187 | 624 | L | port faithfully | roadmap |
| `integrals` | calculus | 17,783 | 486 | XL | port faithfully | none |
| `stats` | discrete | 16,651 | 225 | L | port faithfully | roadmap |
| `physics.quantum` | physics | 16,457 | 298 | XL | port faithfully | serve |
| `solvers` | solvers_matrices | 13,559 | 441 | XL | port faithfully | compile |
| `tensor` | physics | 13,225 | 239 | XL | port faithfully | none |
| `simplify` | calculus | 13,214 | 318 | XL | port faithfully | serve |
| `functions.special` | functions | 12,938 | 151 | XL | port faithfully | roadmap |
| `utilities` | core | 12,661 | 830 | L | redesign | serve |
| `functions.elementary` | functions | 12,105 | 358 | L | port faithfully | serve |
| `geometry` | discrete | 12,060 | 117 | L | port faithfully | roadmap |
| `solvers.ode` | solvers_matrices | 11,837 | 144 | XL | keep in Python on the Rust core | none |
| `physics.mechanics` | physics | 11,177 | 124 | L | port faithfully | none |
| `polys.matrices` | polys | 10,762 | 315 | L | port faithfully | compile |
| `plotting` | io | 10,530 | 134 | L | keep in Python on the Rust core | none |
| `ntheory` | polys | 9,742 | 89 | L | port faithfully | serve |
| `assumptions` | core | 8,589 | 157 | L | port faithfully | roadmap |
| `polys.domains` | polys | 6,947 | 57 | L | redesign | serve |
| `physics.continuum_mechanics` | physics | 6,861 | 34 | M | port faithfully | none |
| `polys.numberfields` | polys | 6,852 | 123 | L | port faithfully | serve |
| `parsing.autolev` | io | 6,601 | 0 | L | keep in Python on the Rust core | none |
| `sets` | core | 6,574 | 224 | L | port faithfully | compile |
| `series` | calculus | 6,329 | 489 | L | port faithfully | roadmap |
| `physics.control` | physics | 6,153 | 49 | M | port faithfully | none |
| `logic` | core | 6,121 | 127 | L | port faithfully | serve |
| `parsing.latex` | io | 5,954 | 0 | L | redesign | serve |
| `matrices.expressions` | solvers_matrices | 5,906 | 266 | L | port faithfully | none |
| `physics` | physics | 5,299 | 85 | L | port faithfully | none |
| `vector` | physics | 5,009 | 67 | M | port faithfully | none |
| `physics.vector` | physics | 4,931 | 112 | M | port faithfully | none |
| `codegen` | io | 4,468 | 105 | L | port faithfully | none |
| `functions.combinatorial` | functions | 4,330 | 56 | M | port faithfully | roadmap |
| `concrete` | functions | 4,251 | 160 | L | port faithfully | roadmap |
| `printing.pretty` | io | 4,217 | 156 | L | port faithfully | none |
| `physics.biomechanics` | physics | 4,162 | 0 | M | port faithfully | none |
| `solvers.diophantine` | solvers_matrices | 3,985 | 51 | L | port faithfully | none |
| `categories` | physics | 3,591 | 12 | M | port faithfully | none |
| `testing` | core | 3,447 | 33 | M | keep in Python on the Rust core | none |
| `crypto` | polys | 3,403 | 51 | M | port faithfully | none |
| `physics.units` | physics | 3,346 | 72 | M | redesign | serve |
| `holonomic` | functions | 3,272 | 32 | L | defer or drop | none |
| `physics.optics` | physics | 2,984 | 16 | M | port faithfully | none |
| `polys.agca` | polys | 2,935 | 37 | M | port faithfully | compile |
| `parsing` | io | 2,788 | 75 | L | redesign | serve |
| `calculus` | calculus | 2,714 | 47 | M | port faithfully | compile |
| `diffgeom` | physics | 2,432 | 24 | M | port faithfully | none |
| `liealgebras` | physics | 1,948 | 12 | S | port faithfully | none |
| `algebras` | solvers_matrices | 1,669 | 20 | M | port faithfully | none |
| `external` | core | 1,508 | 70 | S | reuse a Rust crate | serve |
| `discrete` | polys | 1,208 | 14 | S | port faithfully | none |
| `interactive` | io | 1,098 | 8 | S | keep in Python on the Rust core | none |
| `parsing.c` | io | 1,060 | 0 | M | keep in Python on the Rust core | none |
| `strategies` | core | 790 | 50 | S | port faithfully | serve |
| `core-misc` | core | 775 | 0 | S | port faithfully | serve |
| `physics.hep` | physics | 716 | 5 | M | port faithfully | none |
| `multipledispatch` | core | 680 | 37 | S | redesign | serve |
| `polys.benchmarks` | polys | 634 | 0 | S | redesign | none |
| `benchmarks` | discrete | 478 | 0 | S | redesign | none |
| `unify` | core | 428 | 33 | S | defer or drop | none |
| `stats.sampling` | discrete | 371 | 17 | M | redesign | none |
| `parsing.fortran` | io | 348 | 0 | M | defer or drop | none |
| `functions` | functions | 115 | 0 | M | redesign | serve |
| `sandbox` | discrete | 80 | 1 | S | port faithfully | none |
| `integrals.benchmarks` | calculus | 34 | 0 | S | defer or drop | none |
| `matrices.benchmarks` | solvers_matrices | 21 | 0 | S | defer or drop | none |
| `solvers.benchmarks` | solvers_matrices | 12 | 0 | S | defer or drop | none |

## Core: expressions, assumptions, logic, sets, utilities

### Suggested port order inside the cluster

1. testing first: a PyO3 + SymPy-test-runner harness with per-module conformance counters, pinned PYTHONHASHSEED, ground types=python and mpmath 1.3.0. Nothing else can be accepted without the oracle.
1. external next: choose the bigint/rational representation with an i64 fast path and port the BPSW/iroot/gcdext primitives. Every number in core sits on it.
1. core phase A: numbers (Integer/Rational/singletons, Float on a ported mpmath libmpf), the FactKB bitset plus the 40 generated rules, Basic/Symbol/Dummy/Wild nodes, hash-consing, Basic.compare/class names, sort_key/default_sort_key/ordered, Tuple/Dict, Ctx(evaluate/distribute), and sympify of primitives. Everything else builds on this.
1. core-misc alongside phase A: generate the 921-name namespace and the module layout for the PyO3 package and the parser. It is cheap and the harness imports it.
1. multipledispatch replacement: dump the oracle's Dispatcher orderings into Rust match tables. relational, kind and sets need it before they can work.
1. core phase B: Add/Mul/Pow flatten, Integer/Rational/Float _eval_power (with the limited factorint from ntheory), exprtools.factor_terms/gcd_terms, intfunc, Mod, and the old-assumption handlers of numbers, Add, Mul and Pow. This is the canonical-form spec that every test depends on.
1. logic boolalg (true/false, And/Or/Not, LatticeOp) together with core relational (Eq/Lt auto-evaluation via is_eq/is_ge). Relational is a Boolean and these two are mutually dependent.
1. core phase C: subs/_eval_subs, xreplace, replace, has/atoms/free_symbols, match/Wild/AssocOp matching, traversal. Downstream modules (simplify, integrals, solvers, ode) use them in about 2,500 call sites.
1. core phase D: the Function/Application vtable, Lambda, Derivative, Subs, expand hint dispatch, count_ops, and evalf with the mpmath elementary functions, hypsum and quad. The functions cluster plugs into this.
1. utilities iterables/misc/exceptions: port early as tiny helpers when the first callers need them; lambdify and codegen come later with printing.
1. strategies: needed as soon as simplify (fu/trigsimp greedy) and matrices.expressions are ported.
1. sets phase 1: Interval, FiniteSet, Union, Intersection, Complement, Range, the fancy singletons and Contains. These need only core, logic and Min/Max, and the app's compile-time figure lint uses them.
1. logic SAT (dpll2, lra_theory, inference) followed by the new assumptions (ask/Q/refine/satask), which consume them. Roadmap priority.
1. sets phase 2: ImageSet, ConditionSet, imageset and the function-image handlers, ported jointly with solvers.solveset and diophantine because they form a genuine cycle.
1. unify last, or drop it: nothing else in SymPy uses it.

### Risks

- The core is not a separable layer. Lazy imports hide hard dependencies of Pow.__new__, evalf, the assumptions and sort_key on functions.elementary (log/sign/im/exp_polar/Abs/re/floor/Piecewise), ntheory (perfect_power, factorint(limit=2**15), isprime), simplify.radsimp.fraction/clear_coefficients, polys.numberfields.minimal_polynomial and printing (str of atoms). Without explicit late-binding hooks, the core cannot be tested to parity until large parts of four other clusters exist.
- Canonical-form drift cascades. Tests across all of SymPy compare structurally against constructor-built expected values and check .args order and str output. Any deviation in Add/Mul/Pow flatten, Basic.compare ordering, radical extraction or Float precision breaks tests in unrelated modules, and the cause is hard to localise. Mitigate with differential fuzzing of the constructors against the oracle from day one.
- mpmath bit-exactness. Float equality includes the last mantissa bit and the precision, and test_evalf compares digit-exact strings. Using a correctly rounded library (astro-float) in place of a port of mpmath's libmpf/libelefun/libhyper will produce rare last-bit and last-digit mismatches. The mpmath port (about 9k lines of libmp, more with special functions) is large and on the critical path.
- Size of the hook protocol. There are about 419 _eval_is_* handlers, 528 _eval_rewrite_as_*, 163 derivative hooks, 127 series hooks and 66 expand hooks, plus an open set of user-defined Function classes. The Rust FunctionDef/Head trait must be designed once and early; reworking it later touches every function.
- Interop with Python subclasses during the transition. Unported Python classes (Matrix, physics.quantum, user Function subclasses with eval) must live inside Rust Add/Mul/Pow (constructor postprocessors, _op_priority) through an opaque-node and callback path. Doing this without GIL contention or semantic gaps is a real design risk, as Symbolica's PyO3 global-state issues showed.
- Nondeterminism in the oracle. _ask shuffles facts with an unseeded RNG, Expr.is_constant and equals use random points, and the sets/solvers loops iterate Python sets in hash order (PYTHONHASHSEED is randomised by default). The oracle must pin seeds, and some divergences will be 'oracle-unstable' rather than port bugs.
- Hash-consing memory and concurrency. A global interner can leak (strong refs) or contend under threads. Weak interning or session arenas are needed, and assumption caches on shared nodes must be race-free (atomic bitsets).
- Repeating SymEngine's mistake. The Rust API must expose non-evaluating raw constructors and keep auto-evaluation in a separate canonicalise layer. Otherwise the 'faster core' cannot serve evaluate=False workloads, which the app needs for radical form checks, or future non-evaluating SymPy designs.
- Sets and solvers form a genuine cycle (ImageSet/imageset handlers call solveset, diophantine, limit and singularities, and solveset returns ImageSet/ConditionSet). Sets cannot be fully finished before solvers.

### What other clusters and the core design must know

- Oracle configuration: the venv runs GROUND_TYPES=python (no gmpy2 or python-flint) and mpmath 1.3.0 with its python backend. Polys and ntheory outputs must be compared under these ground types. SymPy's test runner randomises PYTHONHASHSEED, so the oracle should pin it (for example to 0) and call sympy.core.random.seed. Probes of Union/solveset/solve/simplify/roots outputs across 8 hash seeds were identical, but set-iteration order dependence exists in simplify_union/simplify_intersection.
- The mpmath replacement must be one shared crate with mpmath's exact semantics. Its users: core.evalf (libmpf/libmpc/libelefun, hypsum, quad); functions and functions.special, through Function._eval_evalf's lookup of mpmath functions by class name or MPMATH_TRANSLATIONS (gamma, zeta, polylog, bessel*, erf, Ei, li, hyper, meijerg, fresnel...); polys (RealField/ComplexField domains are mpmath contexts, nroots uses mpmath.polyroots); printing (Float str uses mlib.to_str with dps, min_fixed and max_fixed); integrals (mpmath.quad). The libmp core alone is 8,705 lines; all of mpmath is about 47.7k lines.
- functions.elementary must deliver exp, log, sign, re, im, Abs, exp_polar, floor, ceiling, Min/Max, Piecewise, sin/cos/tan/atan early, because core's Pow.__new__, evalf_table and sets need them. E**x is represented as exp(x) (type(E**x) is exp), and 2**(x/log(2)) auto-evaluates to exp(x).
- ntheory must reproduce factorint(n, limit=2**15) and perfect_power bit-for-bit, because Integer._eval_power's radical simplification depends on the factors they find: sqrt(2*1000003**2) -> 1000003*sqrt(2), but sqrt(3*1000003*1000033**3) stays unsimplified. Integer.is_prime calls isprime.
- polys.numberfields: Expr._eval_is_extended_positive_negative and Expr.equals fall back to minimal_polynomial when evalf(2) gives no significance. (sqrt(2)+sqrt(3)-sqrt(5+2*sqrt(6))).is_zero -> True goes through polys.numberfields, simplify and printing. Until minpoly is ported, the core should answer None and record the divergence. AlgebraicNumber, in core/numbers.py, really belongs with polys.numberfields.
- printing: core ordering depends on the StrPrinter. Atom.sort_key uses str(self) for non-Symbol, non-Number atoms (pi, E, I, oo, BooleanTrue...), and default_sort_key orders by it. Core either needs a small atom printer, or printing has to provide sstr early.
- simplify: core uses simplify.radsimp.fraction in Pow.__new__ and Expr.expand, simplify.simplify.clear_coefficients in is_eq, and simplify/nsimplify in Expr.equals. The core and simplify units therefore form a cycle that needs late-bound hooks.
- parsing: sympify on a string calls parse_expr, which uses Python eval with the namespace 'from sympy import *' (921 names). The Rust parser must reproduce that table, including non-Symbol names (E, I, S, N, O, Q, beta, gamma, zeta, li, Ei, ln->log, rf, ff, sign, re, im, test). It must also reproduce the evaluate=False AST transform: '2/4' -> Mul(2, Pow(4,-1)); 'x - y' -> Add(x, Mul(-1,y)); nested, unflattened Adds. Removing eval is a security improvement that the brief's 'more secure' goal relies on.
- Every unit that uses multipledispatch (assumptions predicates, sets handlers, matrices, AccumBounds, functions.integers, tensor.indexed, physics.quantum, geometry.entity, combinatorics.permutations, polys.rootoftools) can obtain its effective resolution order by dumping each Dispatcher's ordering from the oracle, then encoding it as Rust match tables. MDNotImplementedError fallthrough must be emulated.
- core/tests/test_args.py has 1,007 of core's 2,066 test functions. It instantiates every Basic subclass in all of SymPy, so it should be owned by the project-level harness, not counted as core progress. utilities/tests/test_wester.py has 397 cross-domain CAS tests (153 XFAIL) and belongs in the integration suite.
- Performance: in a warm, app-like workload profile, the old assumptions (core/assumptions.py + core/facts.py) were the single largest SymPy self-time cost (about 20%). A u64-bitset FactKB with compile-time rule tables, plus atomic per-node caches, is a cross-cutting win for simplify, solvers and the app's grading path.
- Licences: malachite is LGPL-3.0 (verify the crate metadata) and therefore excluded under the permissive-only decision. Use dashu (MIT/Apache) or num-bigint/num-rational. mpmath is BSD-3, so porting its algorithms is permitted. splr (SAT) is MPL-2.0, so prefer porting dpll2 or using varisat/batsat (MIT/Apache).
- Python semantics every Rust module must emulate: floored // and % (Integer(-7)//2 == -4, Mod(-7,3) == 2); dict insertion order (use IndexMap); `is` identity on singletons (S.One, S.true...); three-valued fuzzy logic (core/logic.py fuzzy_and/fuzzy_or/fuzzy_not); Expr.round's decimal rounding (Float(2.675).round(2) == 2.68).
- The global evaluation parameters (evaluate, distribute, exp_is_pow) are thread-local mutable state in SymPy. Every constructor-calling module must take an explicit Ctx in Rust. The app needs evaluate=False trees for its radical-form checks.
- Function.nargs is canonicalised to a sets FiniteSet or Naturals0 at construction. Core should use its own small nargs type so that it does not depend on sets, and expose a FiniteSet view only through the API.
- Old assumptions call evalf: is_positive and similar on number expressions evaluate at precision 2, and _prec==1 means 'unknown'. The functions cluster's evalf hooks therefore influence the answers of assumption queries, so evalf and assumptions parity must be tested together.

### `core`

34,576 source lines, 2,066 test functions. Difficulty **XL**. Strategy: **port faithfully**. App relevance: serve.

The expression kernel (34,576 source lines in 45 files; 24,953 test lines; 2,066 test functions, 1,007 of them in test_args.py). It holds Basic/Atom/Expr, Symbol/Dummy/Wild, numbers.py (Integer, Rational, Float, singletons, NumberSymbols, I, oo/-oo/zoo/nan, AlgebraicNumber), AssocOp/LatticeOp, Add/Mul/Pow with constructor-time auto-evaluation, Function/Application/Lambda/Derivative/Subs/WildFunction, Relational (Eq/Ne/Lt/Le/Gt/Ge), the old assumptions system (facts.py FactRules/FactKB, assumptions.py _ask, assumptions_generated.py), cacheit, sympify, sorting (default_sort_key/ordered), subs/xreplace/replace/has/atoms/match, evalf (evalf.py on mpmath), exprtools (factor_terms/gcd_terms), traversal, containers (Tuple/Dict), kind, parameters (evaluate/distribute), Mod, intfunc (igcd, integer_nthroot), singleton registry S, and _op_priority binary-operator dispatch.

**Why this strategy.** Every other module's tests and outputs depend on the core's canonical forms (args order, flatten heuristics, root extraction, Float precision, the assumption closure), so these semantics must be reproduced exactly with SymPy as the oracle. The representation should still be redesigned, not transliterated: an immutable hash-consed DAG, separate raw (evaluate=False) and canonicalising constructors, bitset facts, trait vtables in place of getattr hook lookup, and no lru_cache. This keeps auto-evaluation in a separate layer, as the SymEngine lesson requires.

**Key algorithms.**

- Add.flatten (core/add.py:198): collects terms into a dict term->Number coefficient with the coefficient in slot 0; checks for NaN early; prunes terms absorbed by oo/-oo/zoo using is_extended_nonnegative/is_real/is_finite queries; absorbs into Order; has hooks for AccumBounds, MatrixExpr and TensExpr; ends with _addsort = sort by cmp_to_key(Basic.compare) (basic.py:2225).
- Mul.flatten (core/mul.py ~226-760): a 2-arg fast path that distributes a Rational over an Add (2*(x+1) -> 2*x+2, gated by global_parameters.distribute); _gather of (base, exp) pairs keyed by exp.as_coeff_Mul; merges numeric bases with rational exponents through pnum_rat/num_rat and gcd splitting; accumulates powers of -1 and I in neg1e; NC_Marker handles non-commutative runs; oo and zoo sign handling; _mulsort by Basic.compare.
- Pow.__new__ (core/power.py:137): special exponents (zoo, oo, 0, 1, -1); sign extraction for numeric Mul bases with even or odd exponents; NaN and 1**oo rules; recognises b**(c*n/log(b)) -> exp(c*n) through factor_terms and simplify.radsimp.fraction on every non-atomic exponent; then delegates to base._eval_power.
- Integer._eval_power (numbers.py:2022): root extraction through integer_nthroot, then ntheory.perfect_power, then Integer.factors(limit=2**15), splitting out integer and radical parts (sqrt(8)=2*sqrt(2); sqrt(3*1000003*1000033**3) is left unsimplified); Rational, Float, Infinity, NegativeOne, ImaginaryUnit and Exp1 each have their own _eval_power.
- Basic.compare/_cmp_name with the ordering_of_classes table (basic.py:49,88,366): orders by class-name index or name string, then by the length of _hashable_content, then element-wise. This defines the .args order of Add and Mul.
- Sort keys: Basic.sort_key, Expr.sort_key (expr.py:150, via as_coeff_Mul and as_ordered_terms), Atom.sort_key (uses str(self)), Number.sort_key (numeric), Function.class_key table (function.py:507, exp=10 log=11 sin=20 ...), default_sort_key, ordered() and _nodes (sorting.py).
- Old assumptions: FactRules compiles 40 rule strings (assumptions.py:237) into full_implications, beta_rules and prereq tables (facts.py:380); FactKB.deduce_all_facts does alpha/beta forward chaining (facts.py:599); _ask (assumptions.py ~520) runs a breadth-first search over prereq facts that calls _eval_is_<fact> handlers and caches the answers per object; _prepare_class_assumptions builds is_* properties per class through __init_subclass__.
- Sign of numeric expressions: Expr._eval_is_extended_positive_negative (expr.py:895) uses evalf(2), treats _prec==1 as 'no significance', and falls back to polys.numberfields.minimal_polynomial. Expr.equals and is_constant use random numeric points (_random).
- subs (basic.py:965-1300): unordered dict input is sorted by (-_nodes, default_sort_key) with infinities first; simultaneous=True goes through Dummy*m placeholders; _subs is cached; the fallback rebuilds args, and a 2-arg Mul hack is applied under hack2; per-class _eval_subs implements algebraic substitution (Mul._eval_subs, Pow._eval_subs ((x**4).subs(x**2,y)=y**2, exp(2x).subs(exp(x),y)=y**2), Add._eval_subs (sub-sum matching), bound variables in Integral and Sum, Derivative -> Subs).
- xreplace/_xreplace (exact node replacement); replace() bottom-up walk with type, pattern or callable queries and an exact= heuristic (basic.py:1542); has/has_free/atoms/free_symbols/find/count.
- Pattern matching: Basic.matches, AssocOp._matches_commutative, Mul.matches/_matches_noncomm, Add.matches, Pow.matches, Wild(exclude, properties) and old=True mode. The result is order-heuristic: (x*y).match(p*q) -> {p: x, q: y}.
- evalf (evalf.py): adaptive-precision quad tuples (re, im, re_acc, im_acc); evalf_add/add_terms cancellation detection with precision raised up to maxprec (501-636); evalf_mul, evalf_pow, evalf_trig and evalf_log with argument reduction; hypsum for Sum (1239); do_integral via mpmath.quad (1076); dispatch through evalf_table (1392); otherwise Function._eval_evalf looks up an mpmath function by class name or MPMATH_TRANSLATIONS (function.py:534). EvalfMixin.evalf uses prec=dps_to_prec(n)+4 and handles chop, strict and subs.
- Relational evaluation: is_eq (relational.py:1422) tries _eval_Eq, multipledispatch _eval_is_eq, structural equality, infinity and real/imaginary splitting, (lhs-rhs).is_zero, Float/integer checks, _n2 (evalf(2) of the difference), then a ratio test via simplify.clear_coefficients; is_ge/is_lt use _eval_is_ge dispatch, then _n2, then is_extended_nonnegative(lhs-rhs).
- Number arithmetic: Rational normalisation; Float = mpmath mpf with _prec, where arithmetic takes max(prec) and rounds to nearest (numbers.py:594-1078); Float string parsing sets dps = max(15, digits) (Float('1e30') has prec 106, Float('0.1') has prec 53); Float(x)==Float(x, other prec) is False; Expr.round decimal rounding (expr.py:3828).
- exprtools: Factors/Term, gcd_terms, factor_terms (exprtools.py 919-1270), _monotonic_sign. These are used by Pow.__new__ and by simplify.
- Expr.expand (expr.py:3650): hint dispatch by method-name string _eval_expand_<hint> with deep recursion (_expand_hint); multinomial is ordered before mul.
- Function machinery: Application.__new__ calls cls.eval and canonicalises nargs into FiniteSet/Naturals0; Function._new_ auto-evalfs when every arg is a Float (sin(0.5) is a Float at the arg's precision); UndefinedFunction creates a class at runtime; Derivative canonicalises the variable order and the count tuples; Lambda and Subs handle bound variables.

**Data structures.**

- Expr = Arc<Node>, hash-consed in a sharded weak interner so that == is pointer equality and hashing is O(1). Node { head: Head, args: Box<[Expr]> (SmallVec for 0-2 args), hash: u64, flags: u16 (is_commutative, is_number, has_float, has_wild, canonical), facts: AtomicU64 (30 known bits + 30 value bits, filled lazily), OnceLock caches for free_symbols and sort_key }.
- Head enum. Atoms: Int(SmallOrBig), Rat(Int,Int), Float{mpf: Mpf, prec: u32}, Sym(SymId), Dummy(SymId, u64 index), Wild(WildId), Const(Pi|E|EulerGamma|Catalan|GoldenRatio|Tribonacci), ImagUnit, Oo, NegOo, Zoo, NaN, True, False. Compound: Add, Mul, Pow, Apply(FuncId), Derivative, Subs, Lambda, Rel(Eq|Ne|Lt|Le|Gt|Ge), Bool(And|Or|Not|...), Tuple, Dict, Set(SetHead), Opaque(PyTypeId) for unported Python subclasses.
- Int = enum { Small(i64), Big(Arc<IBig>) } normalised to Small whenever the value fits. Rational is (Int, Int) with q>0 and gcd 1, and becomes Int when q==1, mirroring Rational(4,2) -> Integer. Mpf {sign, man: UBig, exp: i64, bc: u64} replicates mpmath's normalized tuple and rounding.
- sympy_class_name(&Node) -> &'static str returns the SymPy class-name string used by Basic.compare and srepr: Integer 0 -> 'Zero', 1 -> 'One', -1 -> 'NegativeOne', 1/2 -> 'Half', and so on.
- SymbolTable: SymId -> (name: Arc<str>, FactSet closure, original assumptions kept for srepr).
- FuncId registry: name, class_key index, nargs as a small range/bitset instead of a FiniteSet, and a FunctionDef trait vtable (eval, fdiff, evalf, eval_is(Fact), rewrite(target), expand hints, nseries, subs).
- Fact enum (30 facts) and FactSet {known: u32, value: u32}. The rule tables are generated at build time from the 40 rule strings (a port of FactRules), or taken from assumptions_generated.py.
- Ctx { evaluate, distribute, exp_is_pow } is passed explicitly in the Rust API; the PyO3 shim mirrors the thread-local global_parameters.
- IndexMap/IndexSet with a fixed-seed hasher wherever SymPy uses dicts. Never iterate an unordered set without sorting it first.

**Hard dependencies.**

- external
- utilities
- multipledispatch
- logic
- functions.elementary
- ntheory
- simplify
- printing

**Python features that are hard to port.**

- __init_subclass__ generates is_<fact> properties and a default_assumptions FactKB per class (assumptions.py _prepare_class_assumptions); handlers are found by getattr('_eval_is_'+fact). There are 419 _eval_is_* methods across SymPy (98 in core, 231 in functions).
- String-named hook protocols are resolved by getattr: _eval_expand_<hint> (66 methods), _eval_rewrite_as_<Class> (528), _eval_subs, _eval_power, _eval_evalf/_eval_mpmath/_imp_, _eval_derivative/fdiff (163), _eval_nseries/_eval_as_leading_term (127), _eval_Eq, _eval_refine.
- The Singleton metaclass and the S registry, together with pervasive `is S.One` identity checks.
- Binary-operator dispatch through _op_priority and call_highest_priority (Matrix 10.01, sets 11, logic 11, units 13), plus _sympifyit/NotImplemented fallthrough.
- _constructor_postprocessor_mapping lets MatrixExpr, Vector, TensExpr and quantum classes post-process Add/Mul/Pow.
- Function('f') creates a new class at runtime (UndefinedFunction metaclass). Applied functions are instances of that class, and func is a class object.
- Lazy imports inside function bodies hide import cycles (core -> functions, simplify, polys, series, printing).
- lru_cache on __new__, _subs, expand, has and sort_key, with typed=True and maxsize 1000. Env vars SYMPY_USE_CACHE and SYMPY_CACHE_SIZE.
- A thread-local mutable global_parameters, with the context managers evaluate() and distribute().
- Duck typing through is_Add/is_Mul/is_Number class flags and hasattr checks. A few args are non-Basic (frozenset in compare, Python bools in some classes).
- Python int arbitrary precision, floor semantics and hash randomisation; pickling via __getnewargs_ex__/__reduce_ex__.

**Behaviour a port must reproduce.**

- The .args order of Add and Mul is sorted by Basic.compare, which ranks class-name strings via ordering_of_classes and then the _hashable_content tuple. It is not default_sort_key. Singleton numbers have their own class names ('Zero', 'One', 'NegativeOne', 'Half', 'Infinity', ...), so a Rust Integer 1/2 must compare as class 'Half'. (x**2+x+1).args == (1, x, x**2), while str() gives 'x**2 + x + 1'.
- Distribution: 2*(x+1) -> 2*x+2 and Mul(2, x+1) -> 2*x+2, but y*(x+1)*2 -> 2*y*(x+1) and Mul(2, x+1, y) -> 2*y*(x+1). The first is a 2-arg-only hack; 2*(x+1)*y gives y*(2*x+2).
- Radical extraction depends on the exact behaviour of factorint(limit=2**15). sqrt(2*1000003**2) -> 1000003*sqrt(2), but sqrt(3*1000003*1000033**3) stays unsimplified. Integer roots of negatives: (-8)**(1/3) -> 2*(-1)**(1/3); sqrt(-4) -> 2*I; (-1)**(1/2) -> I.
- Float vs exact (changed in 1.13): Float(2.0)==Integer(2) is False and Rational(1,2)==0.5 is False, yet hash(Float(2.0))==hash(2). FiniteSet(1, 2.0, 2) keeps both 2.0 and 2. A Float's identity includes its precision: Float(0.5)==Float(0.5,30) is False, while Float(1.0)==Float(1.0,15) is True because both have prec 53.
- Float precision rules: the string dps is max(15, significant digits); integer-like strings get dps >= num_digits (Float('12345678901234567890') has prec 70); dps_to_prec(16) == 56; arithmetic uses the max precision of the operands (Float(1,5)+Float(1,30) has prec 103); 0.0*x -> Integer 0 but 1.0*x stays 1.0*x; x+0.5*x+x/2 -> 2.0*x.
- Infinity algebra: oo-oo=nan, 0*oo=nan, zoo+1=zoo, 1/S(0)=zoo, x*oo stays oo*x, p*oo -> oo when p is positive, x+oo stays x+oo.
- Old assumptions are three-valued and the closure is computed from 40 fixed rules. Symbol identity is (name, sorted deduced closure), so Symbol('x') != Symbol('x', real=True). Answers are cached on each node. _ask shuffles the prereq order with an unseeded RNG; the port must be deterministic and must not depend on that order.
- is_positive/is_zero on number expressions evaluates numerically at precision 2 and, if that is inconclusive, calls minimal_polynomial. (sqrt(2)+sqrt(3)-sqrt(5+2*sqrt(6))).is_zero -> True reaches polys.numberfields, simplify and printing.
- Eq auto-evaluates: Eq(x,x) -> true, Eq(1,2) -> false, Eq(nan,nan) -> false although nan==nan is True. Lt/Gt raise TypeError for non-real arguments or NaN. bool(x<1) raises TypeError. Results are BooleanTrue/BooleanFalse singletons, not Python bools.
- evaluate=False produces nested, unflattened, unsorted trees: with evaluate(False), x+x+2*3 -> Add(Add(x,x),6); sympify('2/4',evaluate=False) -> Mul(2, Pow(4,-1)); 'x - y' -> Add(x, Mul(-1, y)). global_parameters (evaluate, distribute, exp_is_pow) are thread-local.
- subs: dict input is reordered by (-node count, default_sort_key); a list input is applied sequentially ((x+y).subs([(x,y),(y,z)]) -> 2*z, while simultaneous=True gives y+z); (x**3).subs(x**2,y) is unchanged; (2*x+2*y).subs(x+y,a) is unchanged; bound variables in Integral are untouched; Derivative(f(x),x).subs(x,0) -> Subs(...).
- sort_key differs from compare. Atom.sort_key uses str(atom), so ordering depends on the StrPrinter. Number.sort_key compares numerically. Dummy sort_key includes dummy_index, a global creation counter.
- Function evaluation: sin(0.5) evaluates to a Float at 53 bits; sin(Float('0.5',30)) evaluates at 30 digits; gamma(2) -> 1; E**x is the class exp; exp(I*pi/2) -> I; 2**(x/log(2)) -> exp(x); cos(pi/5) -> 1/4+sqrt(5)/4.
- Expr.round rounds the exact binary value in decimal: Float(2.675).round(2) -> 2.68, whereas Python's round gives 2.67. Float(2.5).round() -> 2 (round half to even).
- Python integer semantics: floor division and floored modulo (Integer(-7)//2 == -4, Mod(-7,3) == 2). Rust's / and % truncate.
- Python dicts are insertion-ordered and SymPy depends on that (for example Mul.flatten's pnew/inv_exp_dict, then a sort). Python sets of Basic objects iterate in hash order, and string hashes are randomized per process.
- SymPy holds per-object mutable caches (_assumptions, _mhash, Mul/Add is_commutative), and cacheit is lru_cache(1000, typed=True), so Integer(500) is Integer(500). Correctness must not depend on identity except for the S.* singletons, which SymPy compares with `is` throughout.

**Rust prior art.**

- dashu-int / dashu-ratio (MIT OR Apache-2.0): bigints and rationals; add our own i64 fast path.
- num-bigint / num-rational (MIT/Apache): an alternative, but very slow on huge numbers.
- hashconsing crate (MIT/Apache) and internment (MIT/Apache) for hash-consed Arc nodes; lasso (MIT/Apache) as a string interner.
- smallvec (MIT/Apache) and indexmap (MIT/Apache) for Python dict ordering; foldhash/rustc-hash (MIT/Apache) for deterministic hashing.
- mpmath 1.3.0 (BSD-3, Python) is the porting source for libmpf/libmpc/libelefun so that Float bits match.
- astro-float (MIT): correctly rounded, but not bit-compatible with mpmath; use it for cross-checking, not as the engine.
- SymEngine (MIT, C++) as a design reference for RCP Basic and dict-based Add/Mul, keeping in mind that it bakes in auto-evaluation.
- Symbolica: source-available, not usable; reference only for its append-only symbol table.
- malachite is LGPL-3.0 (verify the crate metadata), so it is excluded under the permissive-only rule. rug/GMP/MPFR are LGPL and also excluded.

**How its tests check results.** Mostly structural == against expected values built through the same auto-evaluating constructors, so a port with a consistent but different canonical form passes some tests and fails others. Many tests are order-sensitive: .args tuple checks (test_arit), str/srepr/sstr string comparisons, and test_evalf's NS() digit-exact strings (for example NS('pi**(10**20)',10) == '1.339148777e+49714987269413385435'). Assumption properties are checked with `is True/False/None`. Errors are checked with raises(TypeError/ValueError/...). Core has about 45 XFAILs. test_args.py (1,007 functions) instantiates every Basic subclass in all of SymPy and checks that its args are Basic and that func(*args)==expr, so it is a whole-project invariant suite, not a core suite. test_cache, test_priority, test_compatibility and test_constructor_postprocessor test Python-specific mechanisms. Doctests in the core modules are also string comparisons.

**Notes.** Measured with the pinned oracle. In a warm profile of an app-like workload (parse_expr + simplify/expand/factor/limit/solve/solveset/N/latex), the old-assumptions machinery (core/assumptions.py + core/facts.py) was the largest single cost, about 20% of SymPy self-time, ahead of basic.py, numbers.py, mul.py, sympify.py and cache.py. A u64 bitset FactKB is therefore the biggest early performance win. Pow.__new__ runs factor_terms and fraction on every non-atomic exponent, which is a further hot spot.

The core genuinely needs functions.elementary (log, sign, im, exp_polar in Pow.__new__; exp as the representation of E**x; Abs/re/im/floor/ceiling/Piecewise/atan in evalf_table), ntheory (perfect_power, limited factorint, isprime), simplify.radsimp.fraction, and printing (str of atoms in sort_key; Float formatting). Order (series), AccumBounds (calculus), MatrixExpr and TensExpr are only hooks inside flatten. minimal_polynomial (polys.numberfields) can return 'unknown' until it is ported, at the cost of documented divergences. sets is needed only for Function.nargs.

Plan core as about four sub-phases: A numbers/Basic/Symbol/facts/compare; B Add/Mul/Pow/exprtools/intfunc; C relational/subs/xreplace/replace/match/sorting; D Function/Derivative/expand/evalf. The mpmath replacement (libmpf + elementary functions + to_str/from_str + hypsum + quad) is about 9k lines of mpmath libmp alone. It is a shared crate needed by core.evalf, functions.special (by-name lookup), polys (RealField/ComplexField are mpmath contexts; nroots uses polyroots), printing (to_str) and integrals (quad). core/backend.py (the USE_SYMENGINE switch) and core/benchmarks can be dropped; the benchmarks are useful as performance probes.

### `core-misc`

775 source lines, 0 test functions. Difficulty **S**. Strategy: **port faithfully**. App relevance: serve.

Top-level package files (775 lines). sympy/__init__.py defines the 921-name public namespace; parse_expr and sympify evaluate strings in exactly this namespace (exec 'from sympy import *'). abc.py provides pre-made Symbols plus the _clash/_clash1/_clash2 tables of names that collide with SymPy objects (Q, S, N, O, I, E, beta, gamma, zeta, pi). release.py holds __version__='1.14.0'. this.py is the Zen text. galgebra.py is a stub. conftest.py contains the pytest --split option and the ground-types report header.

**Why this strategy.** This unit is a name table and module layout, not logic. It should be generated from the oracle (dump sympy.__all__ and abc._clash) into the Rust parser's namespace and into the PyO3 package __init__.

**Key algorithms.**

- The namespace table: sympy.__all__ (921 names) is the default global_dict of parse_expr.
- abc._clash computation: exec('from sympy import *') then intersection with the latin and greek name lists.
- conftest.process_split shards the tests (--split a/b).

**Data structures.**

- A static name -> NamespaceEntry map: Function(FuncId) | Constant(Expr) | Class(ctor) | PyOnly. It is generated from the oracle and shared by the parser and the PyO3 module.

**Hard dependencies.**

- core

**Python features that are hard to port.**

- exec('from sympy import *') to build namespaces.
- Lazy module attributes (LazyFunction for 'test').

**Behaviour a port must reproduce.**

- parse_expr resolves names through this namespace, not as Symbols: E->Exp1, I->ImaginaryUnit, S->SingletonRegistry, N->evalf function, O->Order, Q->AssumptionKeys, beta/gamma/zeta/Ei/li/sec/re/im/sign->function classes, ln->log, rf/ff->RisingFactorial/FallingFactorial, test->LazyFunction. C and lambda_ become Symbols. So 'N*2', 'S*2', 'beta*2' and 'Q*x' raise TypeError.
- from sympy.abc import pi, gamma, beta and the like gives Symbols that shadow the constants. The parser's _clash handling depends on this table.
- The module layout (sympy.core.numbers, sympy.core.add, ...) must be preserved by the PyO3 shim, because tests and users import from deep paths.

**Rust prior art.**

- phf (MIT) for a compile-time perfect-hash name table.

**How its tests check results.** There are no tests of its own. The whole suite exercises these files through imports, and doctests of sympy/__init__ are string compares.

**Notes.** This unit matters for the app's student-input parsing. Any namespace entry that changes between SymPy versions changes whether a letter is read as a symbol or a function, which is the kind of silent misread the app audit warns about. Pin the table to 1.14.0.

### `assumptions`

8,589 source lines, 157 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: roadmap.

The new-style assumptions system: about 58 predicate classes behind Q (predicates/*.py and relation/binrel and equality), about 276 handler registrations through multipledispatch (handlers/order, sets, ntheory, calculus, common, matrices), the ask() pipeline, satask (CNF plus a SAT check over relevant class facts from sathandlers.py), lra_satask (linear real arithmetic), refine() with per-function handlers, AssumptionsWrapper (bridging an assumptions context to the old is_* API) and global_assumptions. 8,589 source lines, 157 test functions.

**Why this strategy.** The unit is self-contained logic over the core, but its answers must match the oracle's True/False/None exactly. Porting the handlers and reusing the generated fact tables is far cheaper than designing a new inference engine. It is a later-phase unit because almost nothing in the core or the app path calls ask (only refine, the matrix expressions, codegen.rewriting and unify.rewrite).

**Key algorithms.**

- ask (ask.py): sympify the proposition, map Eq/Lt/... to Q.eq/Q.lt, CNF.from_prop(assumptions) plus context, _extract_all_facts, an EncodedCNF consistency check via satisfiable (ValueError on inconsistency), _ask_single_fact against the generated get_known_facts_dict, then Predicate._eval_ask (multipledispatch handler with MDNotImplementedError fallthrough), then satask, then lra_satask.
- satask (satask.py:17): get_all_relevant_facts iterates class facts for subexpressions (sathandlers ClassFactRegistry: allargs/anyarg/exactlyonearg rules for Add, Mul, Pow, Abs, ...) and check_satisfiability of prop and ~prop.
- CNF/EncodedCNF and to_NNF/distribute_AND_over_OR (cnf.py).
- lra_satask (lra_satask.py) feeds binary relations to logic/algorithms/lra_theory.LRASolver.
- refine (refine.py) does a bottom-up handler_map plus an _eval_refine hook (Pow, exp, Abs, re, im, arg, sign, atan2, MatrixElement).
- Generated tables: ask_generated.py (get_all_known_facts, get_known_facts_dict) and facts.py (generate_known_facts_dict).

**Data structures.**

- Predicate ids (an enum of about 58 predicates); AppliedPredicate as a core Head; handler tables keyed by (Predicate, Head kind) with an ordered fallback list.
- CNF as Vec<Vec<Lit>> with an EncodedCNF symbol encoding; the known-facts implication tables loaded as const arrays.

**Hard dependencies.**

- core
- logic
- multipledispatch
- matrices
- ntheory

**Python features that are hard to port.**

- multipledispatch registrations with MDNotImplementedError fallthrough.
- The AssumptionsWrapper Basic subclass that proxies is_* properties.
- The global_assumptions mutable context (AssumptionsContext set).

**Behaviour a port must reproduce.**

- ask returns True, False or None (Python bool identity is tested with `is`), and raises ValueError('inconsistent assumptions') when the given facts are unsatisfiable.
- Handlers for Expr delegate to the old system (expr.is_negative) and raise MDNotImplementedError on None to fall through to the next most specific registration. The port needs an ordered fallback chain.
- For number arguments, the order predicates evaluate at precision 2 with the _prec!=1 significance rule (handlers/order.py _NegativePredicate_number). Their answers therefore inherit evalf semantics.
- Q.positive(x) and similar are AppliedPredicate Basic nodes with args (predicate, x), so they hash, compare and print as expressions.
- The known-facts tables are generated and must be reused verbatim. _ask_single_fact can answer without any SAT call, so performance and results depend on that shortcut.
- lra_satask raises UnhandledInput for non-linear or non-real input and ask turns that into None. Do not over-answer.
- refine(sqrt(x**2), Q.positive(x)) -> x and refine(Abs(x), Q.negative(x)) -> -x. Pow._eval_refine rewrites (-1)**(even/odd) forms.

**Rust prior art.**

- varisat (MIT/Apache) and batsat (MIT) as CDCL SAT solvers, although porting dpll2 is simpler because it needs the LRA theory hook.
- rustsat (MIT) for CNF utilities.

**How its tests check results.** `assert ask(Q.p(expr), assumptions) is True/False/None` dominates test_query.py (2,541 lines). refine results are compared structurally. raises(ValueError) checks inconsistent assumptions. test_satask exercises the SAT path directly. There are about 14 XFAILs. Some tests use matrices (test_matrices.py).

**Notes.** The app's per-item assumptions (Symbol(positive=True)) go through the old system in core, not this unit. refine and Q would only matter if the roadmap adds an assumptions knob on an existing expression. Inside the core, ask is reached only through Pow._eval_refine and exp._eval_refine.

### `logic`

6,121 source lines, 127 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: serve.

Boolean algebra and propositional reasoning. boolalg.py (3,587 lines) holds Boolean, the BooleanAtom true/false singletons, And/Or (LatticeOp), Not, Xor, Nand, Nor, Implies, Equivalent, ITE and Exclusive with constructor canonicalisation; to_cnf/to_dnf/to_nnf/to_anf; simplify_logic (Quine-McCluskey SOP/POS plus pattern-based relational simplification); bool_map; truth tables. inference.py holds satisfiable, valid, pl_true, entails and PropKB. algorithms/ holds dpll and dpll2 (watched literals, VSIDS, DPLL(T) hook), lra_theory (a simplex-based linear real arithmetic solver) and wrappers for pycosat, minisat22 and z3. utilities/dimacs.py is a DIMACS reader. 6,121 source lines, 127 test functions.

**Why this strategy.** Boolean canonicalisation is on the hot path for Relational, Piecewise and solveset results and must match the oracle structurally. The SAT and LRA parts are small (about 1,600 lines) and need SymPy's DPLL(T) theory interface, so a direct port is simpler and more faithful than adapting an external SAT crate.

**Key algorithms.**

- And._new_args_filter / Or._new_args_filter (boolalg.py:~585): binary_check_and_simplify, LatticeOp absorption, relational canonical dedupe and contradiction detection (x<1 & x>=1 -> false).
- LatticeOp.__new__ (core/operations.py:528): frozenset args, ShortCircuit on zero, args ordered by default_sort_key.
- Not.eval pushes into relationals (Not(x<1) -> x>=1, Not(Eq) -> Ne) and applies De Morgan in to_nnf.
- simplify_logic (boolalg.py:2777): _find_predicates sorted, truth table, SOPform/POSform via _simplified_pairs/_rem_redundancy, and _apply_patternbased_simplification for relationals (3082-3476).
- to_cnf/to_dnf with distribute_and_over_or, and a simplify=True threshold (force for more than 8 variables).
- dpll2.SATSolver (algorithms/dpll2.py:82-679): two watched literals, VSIDS, no clause learning by default, an LRA theory callback, and an all_models generator.
- LRASolver (algorithms/lra_theory.py:142): dual simplex in the style of Dutertre and de Moura, with Boundary and LRARational supporting strict bounds through infinitesimals.
- satisfiable dispatch (inference.py:35) and pl_true evaluation.

**Data structures.**

- Boolean heads in the core DAG (Bool(And|Or|Not|Xor|...)) with sorted, deduplicated args.
- A SAT clause DB with watched-literal lists and a VSIDS heap; the LRA tableau with exact rationals (dashu-ratio) and (value, delta) pairs for strict bounds.

**Hard dependencies.**

- core
- utilities

**Python features that are hard to port.**

- LatticeOp's ShortCircuit exception for control flow.
- Generator-based all_models.
- Optional imports of pycosat, minisat and z3 (drop them or put them behind features).

**Behaviour a port must reproduce.**

- And/Or/Xor args are deduplicated and ordered by default_sort_key, not by insertion order. And(x, true) -> x; And(x, false) -> false.
- S.true and S.false are BooleanTrue/BooleanFalse singletons, distinct from Python True/False, but true == True holds. Relational subclasses Boolean, so Eq(...) objects participate in And/Or.
- Relational contradictions are detected only through .canonical and .negated equality, not by any general reasoning.
- simplify_logic output depends on variable ordering (sorted predicates) and the QM tie-breaking. Results are compared structurally in tests.
- satisfiable returns a dict model or False. all_models yields a generator, and the model order follows the dpll2 search. Tests accept alternatives for multiple models, but unique models must match exactly.
- ITE(c, a, b) auto-evaluation rules, Xor parity normalisation, and Implies/Equivalent eval with relationals.

**Rust prior art.**

- varisat (MIT/Apache), batsat (MIT), rustsat (MIT). splr is MPL-2.0 (weak copyleft), so avoid it.
- good_lp/minilp (MIT/Apache) are floating-point LP solvers and unsuitable for exact LRA, so port lra_theory.py instead.

**How its tests check results.** Structural == of boolean expressions (to_cnf(...) == And(...), simplify_logic(...) == ...). satisfiable(...) == {A: True, B: False} or `in (alternatives)`. raises(TypeError/ValueError). test_lra_theory uses seeded random linear systems and compares feasibility. test_dimacs parses files. About 6 XFAILs.

**Notes.** The app uses BooleanAtom and Relational (which inherits from Boolean). solveset and inequality results return And/Or conditions. The SAT and LRA parts serve assumptions.satask, solvers.polysys and sets.fancysets, which makes them roadmap priority.

### `multipledispatch`

680 source lines, 37 test functions. Difficulty **S**. Strategy: **redesign**. App relevance: serve.

A vendored copy of the multipledispatch library (680 lines). Dispatcher registers implementations by type signature, resolves them in MRO-based specificity order (conflict.supercedes, a topological ordering), warns or ignores ambiguities, and supports MethodDispatcher. It underlies about 550 registrations in SymPy: relational _eval_is_eq/_eval_is_ge, Kind dispatch for Add/Mul/Pow, sets handlers (intersection, union, issubset, comparison, functions, add, mul, power), about 276 assumption predicate handlers, AccumBounds, matrices, tensor.indexed and functions.integers.

**Why this strategy.** Rust has no runtime class hierarchy. Each SymPy Dispatcher becomes a statically written match over (Head, Head) with an explicit ordered fallback list, which is faster and auditable. The resolution order can be generated by dumping each Dispatcher's ordering from the oracle (Dispatcher.ordering). Only the PyO3 extension path for Python-defined classes needs a small runtime registry.

**Key algorithms.**

- conflict.supercedes / ambiguous / ordering (conflict.py): a topological sort of signatures by specificity.
- Dispatcher.dispatch_iter over the product of MROs, with a resolution cache.
- MDNotImplementedError: fall through to the next candidate implementation.

**Data structures.**

- Per-operation fn tables: match (a.kind(), b.kind()) { ... } returning Option<T>, with a Vec of fallbacks to emulate MDNotImplementedError.
- A registry HashMap<(TypeId, TypeId), Vec<Handler>> only for opaque Python classes.

**Python features that are hard to port.**

- Signature resolution over Python MROs and abstract classes.
- Decorator-based registration at import time, so the final tables depend on module import order.

**Behaviour a port must reproduce.**

- The most specific signature wins according to the Python MRO, including abstract bases. The Rust replacement must reproduce the same winner for every (type, type) pair that SymPy registers.
- MDNotImplementedError makes the dispatcher try the next most specific implementation, and the assumption handlers rely on this.
- is_eq calls _eval_is_eq(rhs, lhs) when dispatch(type(lhs), type(rhs)) differs from the reverse. Symmetric fallbacks are part of the semantics.
- ambiguity_register_error_ignore_dup suppresses duplicate-registration errors; registration order matters where a rule is registered twice.

**Rust prior art.**

- Plain enum match. Crates such as `enum_dispatch` (MIT/Apache) cover single dispatch only.

**How its tests check results.** Unit tests of the dispatcher on Python builtin types (int, float, custom classes): resolve, dispatch identity, ambiguity warnings. They are irrelevant to a Rust port except through the PyO3 shim.

**Notes.** Extract the effective dispatch tables mechanically from the oracle by iterating each Dispatcher's funcs and ordering, so the Rust match arms have a checked spec.

### `strategies`

790 source lines, 50 test functions. Difficulty **S**. Strategy: **port faithfully**. App relevance: serve.

A rule-combinator library (790 lines). core.py has exhaust, condition, chain, do_one, switch, typed, minimize, memoize, null_safe, tryit, debug and identity. rl.py has rm_id, glom, sort, distribute, subs, unpack, flatten and rebuild. tree.py has treeapply, greedy, allresults and brute. traverse.py has top_down, bottom_up and sall. branch/ has generator-based branching versions.

**Why this strategy.** The module is tiny and its users (simplify.fu/trigsimp, matrices.expressions, integrals.manualintegrate, solvers.rebuild, tensor.functions) need the identical rule-application order. Rust closures and iterators map one to one.

**Key algorithms.**

- tree.greedy and brute: choose the minimal result by objective over a tree of alternative rule lists. simplify/fu.py's futrig uses this for TR rule sequences.
- exhaust (repeat until a fixed point under ==), do_one (first rule that changes the expression), condition and switch dispatch.
- rl.glom/rm_id/flatten/unpack/sort/distribute: canonicalisation of MatAdd/MatMul and similar in matrices.expressions.

**Data structures.**

- Rule = Arc<dyn Fn(&Expr) -> Expr>; BranchRule = Arc<dyn Fn(&Expr) -> Box<dyn Iterator<Item = Expr>>>.

**Hard dependencies.**

- core

**Python features that are hard to port.**

- Higher-order functions and generators; memoize uses a dict keyed by the expression.

**Behaviour a port must reproduce.**

- Ties in minimize and greedy are broken by list order (Python min returns the first minimal element), and the objective is usually count_ops or a tuple key.
- exhaust compares with structural == and can loop forever on non-confluent rules, so keep the same rule order.
- Branch strategies yield in a defined depth-first order, which allresults and brute outputs depend on.

**Rust prior art.**

- None needed; std closures and iterators suffice.

**How its tests check results.** Small structural == tests on toy trees and Basic objects, comparing lists of results.

**Notes.** simplify calls trigsimp, which calls futrig, which calls strategies.tree.greedy, so this unit is on the app's serve path for any trig expression.

### `unify`

428 source lines, 33 test functions. Difficulty **S**. Strategy: **defer or drop**. App relevance: none.

Associative-commutative unification (428 lines). core.py has Compound, Variable and CondVariable with a unify generator that handles assoc/comm through allcombinations and partition. usympy.py has deconstruct, construct and rebuild adapters for SymPy Basic. rewrite.py has rewriterule, which combines unify with ask conditions.

**Why this strategy.** No other SymPy module imports sympy.unify (grep finds only its own files). It is an isolated feature that can be ported last as a small iterator module, or dropped with a documented divergence.

**Key algorithms.**

- unify/core.py unify: occur_check, unify_var, and enumeration of associative and commutative splittings via allcombinations/partition/index.
- usympy.deconstruct/construct: maps Basic to and from Compound, consulting is_associative/is_commutative per class.

**Data structures.**

- Compound {op: Head, args}, Variable(Expr), and an iterator-based unifier.

**Hard dependencies.**

- core
- assumptions

**Python features that are hard to port.**

- Recursive generators; class-attribute-based associativity checks.

**Behaviour a port must reproduce.**

- A generator yields all unifiers in a specific enumeration order, and tests compare the full list.
- construct rebuilds with evaluate=False for some classes (Add, Mul, MatAdd...). Rebuilding with evaluation changes the results.

**Rust prior art.**

- None that is directly reusable. The egg (MIT) e-graph pattern matcher solves a different problem.

**How its tests check results.** list(unify(...)) == [expected dicts] (order-sensitive); structural == for deconstruct and construct.

**Notes.** This is the lowest priority in the cluster.

### `sets`

6,574 source lines, 224 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: compile.

Set algebra as Basic objects (6,574 source lines, 224 test functions). sets.py has Set (union, intersect, complement, contains, is_subset, inf/sup, measure, boundary, as_relational), Interval, FiniteSet, Union, Intersection, Complement, SymmetricDifference, DisjointUnion, ProductSet, and the EmptySet/UniversalSet singletons. fancysets.py has Naturals, Naturals0, Integers, Rationals, Reals, Complexes, Range, ImageSet and ComplexRegion. Also conditionset.py, powerset.py, contains.py, setexpr.py (set arithmetic), ordinals.py, and handlers/* (pairwise multipledispatch rules for intersection, union, issubset, comparison, add, mul, power and functions).

**Why this strategy.** Set results are compared structurally and feed solveset, the app's interval keys and figure lint. The handlers' rule order defines the canonical outputs. Port in two phases: the base sets and interval logic first (they depend only on core, logic and Min/Max), then the ImageSet/ConditionSet/function-image handlers together with solvers, which form a genuine cycle with solveset.

**Key algorithms.**

- simplify_union and simplify_intersection (sets.py:2562-2695): merge FiniteSets, then iterate pairwise rules to a fixed point over a Python set of args; for intersections, distribute over Union and rewrite Complement.
- Intersection._handle_finite_sets: fuzzy membership sifting.
- Interval.__new__ evaluation rules (sets.py:1035).
- Union and Intersection arg ordering via Set._infimum_key; FiniteSet dedupe and ordering via ordered().
- imageset() (sets.py:2416) and handlers/functions._set_function: images of intervals under monotonic functions using diff, singularities, solveset and limit.
- Intersection handlers (handlers/intersection.py): ImageSet with Integers via diop_linear/diophantine (150-300), Range arithmetic, ImageSet with Reals via solve_linear/denoms and invert_real.
- Range normalisation and slicing; ComplexRegion and normalize_theta_set; ConditionSet substitution and dummy handling; Complement reductions.

**Data structures.**

- Set heads in the core DAG: Interval(start, end, lopen, ropen), FiniteSet(sorted unique args), Union, Intersection, Complement, ProductSet, Range(start, stop, step), ImageSet(Lambda, base sets), ConditionSet(sym, cond, base), and singletons Reals/Integers/Naturals/Naturals0/Rationals/Complexes/EmptySet/UniversalSet.
- Pairwise rule tables (match on the set head pair) that replace the multipledispatch handlers.

**Hard dependencies.**

- core
- logic
- multipledispatch
- functions.elementary
- simplify
- solvers
- solvers.diophantine
- calculus
- series

**Python features that are hard to port.**

- multipledispatch handler registration across modules.
- Python set iteration order in the simplify_* fixed-point loops.
- __contains__ raising TypeError to signal an undecidable result; __iter__ for FiniteSet and Range.

**Behaviour a port must reproduce.**

- Interval args are (start, end, left_open, right_open) with BooleanTrue/False atoms. Infinite ends are forced open (Interval(0,oo).args == (0, oo, False, True)). Interval(1,1) -> {1}; Interval(2,1) -> EmptySet; non-real endpoints raise ValueError.
- Membership `x in Interval(0,1)` raises TypeError('did not evaluate to a bool') when undetermined, while .contains(x) returns a Boolean expression ((0 <= x) & (x <= 1)).
- FiniteSet(1, 2.0, 2) == {1, 2.0, 2}: Float and Integer are distinct members. Args are sorted by ordered() (FiniteSet(x,1,y).args == (1, x, y)).
- Union with overlapping or touching intervals merges them (Union(Interval.Ropen(0,1), Interval(1,2)) -> Interval(0,2)). Complement(Reals, {0}) -> Union of two open intervals. Intersection(Interval(0,5), Integers) -> Range(0, 6, 1).
- imageset(x, 2*x+3, Integers) normalises to ImageSet(Lambda(x, 2*x + 1), Integers), but the direct ImageSet(Lambda(n, 2*n+3), Integers) constructor does not.
- is_subset, is_empty and is_finite_set are fuzzy (None allowed); tests check them with `is`.
- Pairwise reduction iterates a Python set, so the reduction order is hash-dependent. Probes across 8 PYTHONHASHSEED values gave identical outputs for typical cases, but the port must choose a deterministic order that reproduces the canonical results.

**Rust prior art.**

- None for symbolic sets. The intervals and inari (MIT/Apache) crates do numeric interval arithmetic only and are not applicable.

**How its tests check results.** Structural == of resulting sets; `is True/False/None` for predicates; raises(TypeError/ValueError); `in` membership; imageset and solveset-produced sets are compared structurally. About 11 XFAILs. Many tests depend on solveset and diophantine behaviour through the handlers.

**Notes.** The app uses Interval and FiniteSet at compile time (figure lint, singularities) and plans solveset/Interval/Union for inequality keys. Core needs only a nargs representation instead of FiniteSet/Naturals0, so the core can ship before sets.

### `utilities`

12,661 source lines, 830 test functions. Difficulty **L**. Strategy: **redesign**. App relevance: serve.

A grab-bag of 12,661 source lines and 830 test functions (397 of them in test_wester.py, which tests the whole CAS). iterables.py (3,179 lines) has flatten, sift, subsets, variations, permutations, multiset_permutations/partitions/combinations, partitions, ordered_partitions, necklaces, generate_bell, topological_sort, strongly_connected_components, numbered_symbols, cartes, rotations and more. misc.py has filldedent, as_int, func_name, replace/translate and Undecidable. exceptions.py has sympy_deprecation_warning and ignore_warnings. Also decorator.py (public, threaded, memoize_property, deprecated, doctest_depends_on, conserve_mpmath_dps), memoization.py (recurrence_memo), enumerative.py (Knuth multiset partitions), lambdify.py (code generation plus exec over math/numpy/mpmath/scipy/... namespaces), codegen.py (C/Fortran/Octave/Julia/Rust routines), autowrap.py and _compilation/ (calling external compilers), matchpy_connector.py, mathml/ (XSLT), pkgdata, source, timeutils, and deprecated shims.

**Why this strategy.** The parts differ, so split the unit. Port iterables, misc, exceptions, enumerative and memoization faithfully into a small util crate, because their enumeration orders are load-bearing. Redesign lambdify as a Rust expression-to-closure compiler (and optionally a Cranelift JIT), keeping Python lambdify via PyO3 for numpy users. Port codegen together with printing. Defer autowrap, _compilation, matchpy_connector and mathml, or keep them in Python.

**Key algorithms.**

- iterables.partitions / multiset_partitions / ordered_partitions / multiset_permutations: specific enumeration orders.
- enumerative.MultisetPartitionTraverser (Knuth TAOCP 7.1.2.5 algorithm M) with counting variants.
- iterables.topological_sort and strongly_connected_components (Tarjan).
- lambdify: printer selection by module (NumPyPrinter, MpmathPrinter, ...), namespace merge priority, then source text compiled with exec; also cse support.
- codegen: Routine construction (argument and result inference) and per-language CodeGen emitters.

**Data structures.**

- Iterator adaptors for the combinatorial generators; a compiled numeric closure (Vec<Instr> stack VM or Cranelift) for lambdify.

**Hard dependencies.**

- core
- printing
- external

**Python features that are hard to port.**

- exec/eval code generation in lambdify; optional import_module for numpy/scipy/tensorflow/jax/cupy/numexpr.
- Decorators that mutate function attributes (public adds to __all__; doctest_depends_on).
- A mutated dict reused between generator yields (partitions).

**Behaviour a port must reproduce.**

- Generator yield order of partitions, multiset_partitions, permutations and subsets is relied on by combinatorics, ntheory, solvers and their tests.
- partitions() yields the same mutated dict object each time, so callers copy it (a Python-ism to keep in mind when porting callers).
- as_int(1.0) raises ValueError unless strict=False; sift returns a defaultdict or a binary tuple.
- lambdify executes generated Python source (a security surface); its module-namespace priority decides which implementation (math vs numpy vs mpmath) is called.
- sympy_deprecation_warning and its stacklevel are part of test expectations (warns_deprecated_sympy).

**Rust prior art.**

- itertools (MIT/Apache) for generic combinators.
- cranelift (Apache-2.0 WITH LLVM-exception) for an optional lambdify JIT.
- evalexpr/meval (MIT) are not suitable as a spec source.

**How its tests check results.** Iterables: list equality of generator outputs. lambdify: numeric checks against math/mpmath/numpy (heavily skipped without numpy, with 433 numpy/scipy/... mentions). codegen: exact string compare of generated source. pickling round-trips. test_wester: 397 cross-domain CAS tests with 153 XFAILs, an integration benchmark rather than a utilities test.

**Notes.** Only iterables, misc and exceptions are on the app's serve path, as helpers used everywhere. lambdify is roadmap (plotting and fast numeric checks). Move test_wester.py to the project-level integration suite.

### `testing`

3,447 source lines, 33 test functions. Difficulty **M**. Strategy: **keep in Python on the Rust core**. App relevance: none.

The test infrastructure (3,447 lines). pytest.py has raises, warns, warns_deprecated_sympy, XFAIL, SKIP, slow, skip_under_pyodide and ignore_warnings. runtests.py is SymPy's own test and doctest runner (PYTHONHASHSEED randomisation, split, timeouts, doctest flags, a dependency-aware doctest skipper). It also contains runtests_pytest.py, quality_unicode.py (source code-quality checks), tmpfiles.py, randtest.py (verify_numerically, random_complex_number) and matrices.py helpers.

**Why this strategy.** This unit is the oracle harness. The plan is to run SymPy's own test files and doctests against the rsympy PyO3 module, so the existing Python runner should be kept and extended with per-module conformance counters, pinned seeds and divergence classification (like oracle/divergences.json), not rewritten.

**Key algorithms.**

- runtests.run_in_subprocess_with_hash_randomization: chooses a random PYTHONHASHSEED if none is set.
- SymPyDocTestFinder/SymPyDocTestRunner: doctest collection with dependency checks and output normalisation.
- XFAIL/XPASS bookkeeping and the slow/split selection.

**Data structures.**

- Per-test results keyed by (module, test) with a pass/fail/xfail/divergence class, stored alongside oracle/baseline.json.

**Hard dependencies.**

- core

**Python features that are hard to port.**

- pytest and doctest machinery, and subprocess re-exec for hash seeding.

**Behaviour a port must reproduce.**

- The runner randomises PYTHONHASHSEED when it is unset, so oracle runs must pin it to make any hash-order-dependent output reproducible.
- SYMPY_USE_CACHE and SYMPY_CACHE_SIZE change runtime behaviour; the oracle should run with the defaults (yes, 1000).
- Doctests compare printed strings exactly (str and pretty printers), so doctest parity depends on printing parity.
- Test files import deep module paths (sympy.core.numbers ...), so the PyO3 shim must preserve the module layout.

**Rust prior art.**

- PyO3 (MIT/Apache) and maturin (MIT/Apache) to build the module that the Python harness imports; insta (Apache-2.0) for Rust-side snapshot tests of oracle cases.

**How its tests check results.** Meta-tests of the harness itself (test_pytest, test_runtests_pytest, test_module_imports). test_code_quality scans Python source for style issues; it is Python-specific and can be dropped for Rust.

**Notes.** Build this first: no other unit can be accepted without a strong executable oracle (lesson from prior ports). Pin GROUND_TYPES=python, mpmath 1.3.0 and PYTHONHASHSEED=0, and seed sympy.core.random.

### `external`

1,508 source lines, 70 test functions. Difficulty **S**. Strategy: **reuse a Rust crate**. App relevance: serve.

The optional-backend layer (1,508 lines). gmpy.py selects ground types (flint, gmpy2 or python; the oracle venv has python) and exports MPZ/MPQ and integer functions (gcd, lcm, sqrt, iroot, is_square, factorial, invert, legendre, jacobi, kronecker, PRP tests). ntheory.py provides pure-Python fallbacks: bit_scan0/1, remove, gcdext, iroot, is_fermat/euler/strong_prp, Lucas and Selfridge sequences, and BPSW. pythonmpq.py is a pure-Python rational used as the QQ dtype. importtools.import_module does version-gated optional imports.

**Why this strategy.** Ground types become one Rust bigint and rational choice, dashu-int and dashu-ratio (MIT/Apache) with an i64 fast path, which is the main performance lever for a CAS according to the prior-art research. The small number-theory primitives are ported faithfully on top of it. import_module is replaced by Cargo features.

**Key algorithms.**

- is_strong_bpsw_prp / is_strong_selfridge_prp / _lucas_sequence (ntheory.py:337-611).
- iroot (Newton integer root) and is_square.
- PythonMPQ arithmetic with gcd normalisation and Fraction-compatible hashing (pythonmpq.py:46).

**Data structures.**

- Int = Small(i64) | Big(IBig), and Rat with a small/big split, shared by core numbers and polys ZZ/QQ.

**Python features that are hard to port.**

- Runtime backend selection via an environment variable and try-import.
- Duck-typed MPZ, which can be int, gmpy2.mpz or flint.fmpz.

**Behaviour a port must reproduce.**

- The oracle runs with GROUND_TYPES=python (no gmpy2, no python-flint) and mpmath's python backend. Some downstream outputs, such as flint-backed polys paths, differ under other ground types, so compare only against the python ground types.
- Python int semantics: arbitrary size, floor division and modulo, bit_length, and negative shifts raising errors.
- PRP and BPSW results must agree exactly with sympy.ntheory.isprime for all inputs, because they are deterministic below 2**64 and probabilistic above it.
- The PythonMPQ hash equals the Fraction and int hash for equal values (dict-key compatibility).

**Rust prior art.**

- dashu (MIT/Apache).
- num-bigint/num-integer/num-rational (MIT/Apache).
- malachite (LGPL-3.0: excluded).
- rug (LGPL via GMP: excluded).

**How its tests check results.** test_ntheory compares the pure-Python primitives against known values. test_pythonmpq checks arithmetic and hashing. test_numpy, test_autowrap and test_codegen are skipped without numpy or compilers. test_importtools covers import gating.

**Notes.** Benchmark the small-int fast path early: rustsmith's SymPy experiment found a generic BigInt rational about 3x slower than CPython until it added an i64 path.

## Polynomials, domains, number theory

### Suggested port order inside the cluster

1. 1. Ground arithmetic and core domains ZZ/QQ/GF(p) (polys.domains subset): one Int type (i64 fast path, dashu fallback) with Python floor-division semantics, a normalized Rational, and ModInt. Everything else is generic over Domain, and core.numbers and Matrix storage need the same types.
1. 2. ntheory-base (isprime, nextprime, perfect_power, factorint/factorrat with exact limit= semantics and factor_cache, multiplicity, multinomial_coefficients, integer CRT moved out of galoistools, sqrt_mod): core's Integer powers (sqrt(50) -> 5*sqrt(2)) and the polys algorithms call these.
1. 3. polys dense layer: densebasic, densearith, densetools, monomials and orderings. This is the foundation of every algorithm, and its tests are raw integer lists that translate mechanically to Rust.
1. 4. galoistools (GF(p)[x] arithmetic, sqf, Cantor-Zassenhaus, gf_csolve). Zassenhaus over ZZ and ntheory's polynomial_congruence need it.
1. 5. euclidtools, heuristicgcd and sqfreetools (heugcd, subresultant PRS, resultants, cancel, sqf_norm). gcd/cancel is the serve-path hot spot, and porting the real algorithms avoids rustsmith's 0.17x GCD regression.
1. 6. factortools (Zassenhaus, cyclotomic, Wang EEZ, Gaussian). factor is on the serve path (require_form factored, simplify).
1. 7. polyclasses DMP/DMF, polyutils (_sort_gens), polyoptions, constructor, polytools Poly plus rationaltools, partfrac and polyfuncs. This is the Expr-facing API used by simplify, cancel, together, apart and the Phase-3 milestone.
1. 8. rings/fields (PolyElement/FracElement) and the PolynomialRing/FractionField domains. construct_domain produces ZZ[y]/ZZ(x,y), and groebner, polys.solvers and minpoly all use sparse rings.
1. 9. polys.matrices DDM/SDM/DomainMatrix over ZZ/QQ/EXRAW (rref, rref_den, det, inv_den, charpoly, nullspace, linsolve). sympy.matrices.Matrix stores its data here, and solve/linsolve use it. It can run in parallel from step 1 because it only needs domains and dup_mul.
1. 10. rootisolation, rootoftools and polyroots, plus the mpmath-replacement pieces polyroots and findroot. real_roots/nroots/roots/CRootOf serve solve, solveset and figure lint.
1. 11. AlgebraicField/ANP, numberfields.minpoly and subfield, and dup_ext_factor. minimal_polynomial is called by simplify and Expr.equals/is_positive; AlgebraicField is needed by extension= and eigenvects.
1. 12. agca.extensions.FiniteExtension and polys.matrices.eigen. Matrix.eigenvects (app Markov steady state) needs them.
1. 13. groebnertools, fglmtools and polys.solvers. solve_poly_system, the minpoly fallback and solve() for linear systems need them.
1. 14. RR/CC/EX/EXRAW and the Gaussian domains in full: inexact and symbolic coefficient paths, once the float layer and core cancel are stable.
1. 15. polys.benchmarks as criterion benches, introduced from step 5 onward to track speed against SymPy at each layer.
1. 16. numberfields advanced (round_two, prime_decomp, modules with HNF from normalforms, galois groups that need combinatorics.galois). Pure roadmap scope with heavy dependencies.
1. 17. ntheory-upper (ecm, qs, discrete_log variants, npartitions on the float layer, continued_fraction, egyptian_fraction, elliptic_curve, bbp_pi, digits). Self-contained, after the CPython-compatible RNG exists.
1. 18. Low-use polys modules (ring_series, puiseux, agca modules/ideals/homomorphisms with old_polynomialring and distributedmodules, modulargcd, subresultants_qq_zz, multivariate_resultants, dispersion, polymatrix). Few or no consumers outside tests; they can run as Python on the Rust core until then.
1. 19. discrete, then crypto. Thin layers over core Expr/trig, Matrix, FF/Poly and ntheory; no app use.

### Risks

- Several outputs depend on the algorithm, not just the mathematics: isolation intervals, CRootOf internal intervals, primitive_element coefficients, rref_den denominators, EX/EXRAW expression forms, LLL bases, prime_decomp generators, factorint(limit=) partial results and SIQS internals. A faster algorithm (FLINT-style) can only replace SymPy's in a function whose output is canonical (gcd, factor_list, sqf, resultant, reduced Gröbner basis, rref, HNF/SNF, minpoly), and only after the faithful port passes the oracle.
- Global history-dependent state breaks oracle reproducibility. ntheory.factor_cache (an LRU of 1000 entries) changes core auto-evaluation: on the pinned oracle, sqrt(4*40009**2*40013) differs depending on whether factorint ran earlier. Other sources: the lazily extended global sieve, the CRootOf _reals_cache/_complexes_cache (intervals refined in place), _randint(seed) reseeding the global rng, and @cacheit. Either replicate these per context or document the divergence, and make the oracle harness clear caches or use fresh processes.
- Tests in qs, crypto, Wang and gf_random need a CPython-identical random.Random: MT19937, init_by_array seeding from abs(seed), getrandbits-based _randbelow, 53-bit random() and float uniform.
- Dependency cycles cannot map onto Rust crate boundaries directly. core->polys: EX elements call Expr.cancel, Poly is a Basic, and Expr.equals/_eval_is_extended_positive_negative call minimal_polynomial. polys<->ntheory: gf_crt hosts integer CRT, and factortools uses factorint. polys<->numberfields: AlgebraicField construction needs factoring over algebraic fields. numberfields<->polys.matrices: HNF one way, eigenvects with extension=True the other. This needs a single crate or trait-object callbacks.
- Performance traps. Bigints without a small-int fast path lose to CPython (rustsmith measured 3x slower). Dense recursive DMP allocates heavily for sparse multivariate inputs (the Expr gcd path took 98 ms against 13.6 ms for the sparse heugcd on the same inputs). Every EX operation runs a full core cancel. heugcd needs fast big multiplication and division. Reaching FLINT's 30-100x without FLINT means building native modular algorithms at the same seams SymPy hands to python-flint (DUP over ZZ/QQ/GF(p), dense ZZ/QQ matrices, univariate factor).
- mpmath parity: nroots (polyroots, extraprec 10*deg then 15*deg), CRootOf.evalf (findroot), RR/CC printing and to_rational/max_denom, field_isomorphism_pslq and npartitions all need bit-identical mpmath algorithms, or the Float digits that tests assert will drift.
- Python semantics that compile in Rust but give wrong answers: floor division and modulo (Rust truncates), the float('-inf') vs S.NegativeInfinity degree sentinels, Python list/tuple ordering used for factor sorting, dict insertion order in printed factor dicts, and set iteration order in sparse algorithms.
- Size and test-harness cost. About 1,600 test functions across these units call internal APIs: dup_* on raw Python lists, R.dup_* through compatibility.IPolys, DDM/SDM internals, qs internals and verbose output capture. Either the PyO3 surface gets very large, or the regular test files must be translated to Rust mechanically.
- Exception parity: about 25 polyerrors classes plus the DM* matrix errors are asserted with raises(), so the Rust error enums must map one-to-one onto the Python classes in the PyO3 layer.
- Security items to remove or gate: polyconfig.configure() eval()s SYMPY_* environment variables, and FactorCache.get_external is a documented hook for network calls (factordb). crypto key generation uses non-cryptographic Mersenne Twister.

### What other clusters and the core design must know

- Every explicit sympy Matrix stores its data as a polys.matrices DomainMatrix: RepMatrix._rep is an SDM over ZZ, QQ or EXRAW, and Matrix([[1.5]]) and Matrix([[sqrt(2)]]) use EXRAW. Equality goes through DomainMatrix.unify_eq. inv defaults to DomainMatrix inv_den unless the domain is EX, rref uses DomainMatrix for ZZ/QQ, charpoly always goes through DomainMatrix.charpoly, eigenvects goes through from_Matrix(field=True, extension=True) and dom_eigenvects (with agca FiniteExtension and AlgebraicField), and M**n for n > 10000 uses discrete.linrec_coeffs. polys.domains (ZZ/QQ/EXRAW) and polys.matrices are therefore hard prerequisites for the matrices cluster.
- Core numbers need ntheory early. Integer._eval_power uses perfect_power, then Integer(b).factors(limit=2**15) (trial division only) for surd extraction. Integer.is_prime uses isprime, Mul uses multiplicity, log eval uses multiplicity/perfect_power/factorint, trig special angles use factorint on pi-coefficient denominators, and power expansion uses multinomial_coefficients. ntheory-base has to be in the core phase, not the polys phase.
- Core assumptions and simplify call polys.numberfields: Expr.equals and Expr._eval_is_extended_positive_negative call minimal_polynomial when evalf has no significance, and simplify imports _is_sum_surds/_minimal_polynomial_sq. Measured: simplify(sqrt(2)+sqrt(3)-sqrt(5+2*sqrt(6))) makes 4 minimal_polynomial calls. minpoly in turn needs evalf at up to 200 digits (_choose_factor), groebner, resultants and factoring, so this is an unavoidable core<->polys cycle.
- Global state that changes results: ntheory.factor_cache (LRU of 1000) makes auto-evaluation history-dependent; reproduced on the pinned oracle, sqrt(4*40009**2*40013) is 80018*sqrt(40013) after a prior factorint(40009**2*40013) and stays unsimplified in a fresh process. Other sources: the global sieve, CRootOf caches, and the global rng reseeded by _randint(seed) inside factorint, pollard_rho, ecm, qs and dmp_zz_wang. The oracle harness (oracle/gen_cases.py) should run each case with cleared caches or in a fresh process, and record any intended divergence in divergences.json.
- A CPython-compatible random.Random is required for test parity: MT19937, init_by_array seeding from abs(int seed), getrandbits, _randbelow rejection, randint/randrange, 53-bit random(), uniform, shuffle and choice, plus sympy.core.random._randint/_randrange accepting list seeds. SIQS tests assert g.a == 1133107 with seed 1234, and crypto key generation uses seeds. rand_mt (MIT/Apache) provides the MT19937 core.
- Float-layer (mpmath replacement) requirements from this cluster: polyroots (Durand-Kerner with extraprec and cleanup, used by nroots; tests compare Floats exactly), findroot (CRootOf._eval_evalf), pslq (field_isomorphism_pslq), MPContext with mpf/mpc at arbitrary prec (RR/CC domains, galois_resolvents), to_rational plus limit_denominator with max_denom=max(2**prec//200, 99) (RR->QQ conversion), and the libmp primitives mpf_add/mul/div/sqrt/pi/cos/sin/cosh_sinh, from_int and from_rational (npartitions).
- Licence settled: malachite is LGPL-3.0 and therefore excluded, which answers the 'Malachite's licence' unknown in docs/research/rust-rewrites.md. Use dashu-int/dashu-ratio (MIT OR Apache-2.0) behind an Int enum with an i64 fast path. Coordinate one Int/Rational/ModInt type across core.numbers, the external ground types (PythonMPQ semantics including cross-cancelling mul), polys.domains and ntheory.
- The rules for Poly generator order and domain names are visible in printing and in every factor, cancel and collect output. _sort_gens ranks x,y,z (124-126), then p..w (216-223), then a..o (301-315), then the rest (1000), then name, then numeric suffix. Domain names print as 'QQ<sqrt(2) + sqrt(3)>', 'ZZ[x,y]', 'ZZ(x,y)', 'GF(7)', 'EX'. Poly with modulus prints symmetric coefficients (x**2 + 5*x + 6 mod 7 prints x**2 - 2*x - 1). The printing cluster needs Poly, PurePoly, PolyElement, FracElement, ANP, DomainMatrix and GroebnerBasis printers.
- Solvers depend on this cluster. solve() for linear systems uses polys.solvers.solve_lin_sys (sparse PolyRing plus DomainMatrix). solveset.linsolve uses polys.matrices.linsolve._linsolve. Polynomial solving uses polyroots.roots (Cardano/Ferrari radical forms compared structurally in solver tests) and CRootOf. solve_poly_system uses groebner. ode uses _lin_eq2dict. Their output forms depend on a faithful polyroots port.
- The functions cluster depends on polys and ntheory. functions.special.polynomials (legendre, chebyshev, hermite, jacobi, laguerre) and integrals.quadrature use polys.orthopolys dup generators. functions.combinatorial.numbers (bernoulli, euler, genocchi, andre) uses polys.appellseqs. The Function classes primepi, totient, mobius, divisor_sigma, legendre_symbol, jacobi_symbol and partition now live in functions.combinatorial.numbers; the ntheory versions are integer helpers plus 11 @deprecated shims.
- polys.numberfields.galois_group depends on combinatorics.galois (S3..S6TransitiveSubgroups, Permutation), and galois_resolvents uses utilities.lambdify for numeric evaluation, so those clusters must provide equivalents (direct evaluation in Rust).
- The EX and EXRAW domains need from core: Expr.cancel and expand with deep=False/mul=True flags (every EX operation runs them), could_extract_minus_sign (EX sign normalization in factor and gcd), and an n-ary Add(*items) constructor (K.sum, used in sdm_matmul_exraw). Core must expose these as a callable interface to polys without a crate cycle.
- FLINT seams in SymPy 1.14 mark where native fast paths should go. In the ground types: ZZ=fmpz, QQ=fmpq, GF(p)=nmod/fmpz_mod. polyclasses.DUP_Flint backs univariate ZZ/QQ/GF(p) polynomials, factortools line 685 calls fmpz_poly.factor, and polys.matrices._dfm uses fmpz_mat/fmpq_mat/nmod_mat. Multivariate polynomials (fmpz_mpoly) are not used by default. The normalizations written into DUP_Flint and _dfm.py (monic vs primitive factors over QQ, division semantics) list exactly what a fast path must do to match python-ground outputs. The oracle venv has no gmpy2 or flint, so python ground types are the reference.

### `polys`

49,017 source lines, 907 test functions. Difficulty **XL**. Strategy: **port faithfully**. App relevance: serve.

SymPy's polynomial engine (49,017 src lines, 907 test functions). It has four layers. (1) The user and Expr API in polytools.py (7,960 lines): Poly/PurePoly (Basic subclasses), plus gcd/lcm/cancel/factor/factor_list/sqf/resultant/discriminant/groebner/real_roots/nroots/intervals/apart. Around it sit polyutils.py (_dict_from_expr, _sort_gens), polyoptions.py, constructor.py (construct_domain), rationaltools.together, partfrac.apart, polyfuncs, polyroots.roots and rootoftools (CRootOf/RootSum). (2) Wrapper classes in polyclasses.py: DMP (DMP_Python, or DUP_Flint when GROUND_TYPES=flint), DMF and ANP. (3) The dense recursive algorithm layer, about 14.3k lines of dup_*/dmp_* free functions on nested lists: densebasic, densearith, densetools, euclidtools, sqfreetools, factortools, galoistools, rootisolation. (4) The sparse layer: rings.PolyRing/PolyElement (a dict keyed by exponent tuples), fields.FracField/FracElement, heuristicgcd, groebnertools and fglmtools, ring_series and puiseux. compatibility.IPolys (1,152 lines) exposes every dense function as R.dup_*/R.dmp_* on sparse rings: it converts to dense, calls the function and converts back. About 6k lines are rarely used. modulargcd.py (2,278) and multivariate_resultants.py (473) are imported only by tests. subresultants_qq_zz.py (2,558) is educational and only referenced in docstrings. distributedmodules.py (739) is used only by agca.

**Why this strategy.** The dense dup/dmp algorithms are already free functions over flat data and port cleanly. Several outputs are algorithm-dependent (isolation intervals, EX/EXRAW forms, factor ordering, exception types), so only a faithful port reproduces the oracle. The rustsmith precedent shows naive replacements regress: a naive GCD ran 0.17x the speed of SymPy. Redesign only the representation underneath: a Domain enum with monomorphized ZZ/QQ/GF(p) fast paths, packed monomials in PolyElement and small-int Int. Later, swap in faster algorithms (sparse modular GCD, van Hoeij, F4) only where the output is canonical, behind the same normalization.

**Key algorithms.**

- GCD dispatch, euclidtools._dmp_inner_gcd: inexact domains go to get_exact(). QQ tries dmp_qq_heu_gcd then falls back to dmp_ff_prs_gcd. ZZ tries dmp_zz_heu_gcd then falls back to dmp_rr_prs_gcd. Every other domain uses PRS directly. Controlled by polyconfig USE_HEU_GCD=True.
- Heuristic GCD (Char/Geddes/Gonnet), euclidtools.dup_zz_heu_gcd/dmp_zz_heu_gcd: HEU_GCD_MAX=6 tries. The start point is x=max(min(B,99*isqrt(B)), 2*min(norm/|LC|)+2) and the step is x=73794*x*isqrt(isqrt(x))//27011. Interpolation is _dup_zz_gcd_interpolate with symmetric remainders. heuristicgcd.heugcd is the sparse twin, used by PolyElement._gcd_ZZ.
- Subresultant PRS, euclidtools.dup_inner_subresultants/dmp_inner_subresultants: drives dup_prs_resultant/dmp_prs_resultant, dmp_discriminant and the PRS gcds. Collins' modular resultant (dmp_zz_collins_resultant) exists but is off by default (USE_COLLINS_RESULTANT=False).
- Square-free decomposition, sqfreetools.dup_sqf_list/dmp_sqf_list (Yun) and dup_gf_sqf_list. dup_sqf_norm (shift x -> x - s*a until square-free) feeds the algebraic factoring and primitive_element.
- Zassenhaus, factortools.dup_zz_zassenhaus: Mignotte bound (dup_zz_mignotte_bound), modular factoring via galoistools.gf_factor_sqf, multifactor Hensel lift dup_zz_hensel_lift/dup_zz_hensel_step, and subset recombination with the _test_pl pruning. dup_zz_cyclotomic_factor handles x^n±1 (USE_CYCLOTOMIC_FACTOR=True). Dispatch goes through dup_zz_factor -> dup_factor_list.
- Wang EEZ, factortools.dmp_zz_wang: dmp_zz_wang_non_divisors, dmp_zz_wang_test_points, dmp_zz_wang_lead_coeffs, dmp_zz_diophantine and dmp_zz_wang_hensel_lifting. It builds EEZ_NUMBER_OF_CONFIGS=3 configurations from evaluation points drawn by the global RNG (sympy.core.random._randint) and restarts with mod+1 on ExtraneousFactors.
- Trager algebraic factoring, factortools.dup_ext_factor/dmp_ext_factor: sqf_norm, then factor over QQ, then gcd over QQ<a>. Gaussian factoring (dup_qq_i_factor/dup_zz_i_factor) goes through QQ<I>. Factoring over RR converts to QQ, then rescales by max_norm.
- GF(p)[x] in galoistools: gf_* arithmetic, gf_sqf_list, gf_ddf_zassenhaus + gf_edf_zassenhaus (Cantor-Zassenhaus is the default, GF_FACTOR_METHOD), plus gf_berlekamp and gf_ddf_shoup/gf_edf_shoup. Irreducibility test: gf_irred_p_rabin. gf_csolve/csolve_prime solve polynomial congruences. gf_crt/gf_crt1/gf_crt2 do integer CRT (used by ntheory).
- Real root isolation, rootisolation.dup_isolate_real_roots_sqf: continued fractions in the Vincent-Akritas-Strzebonski style (dup_inner_isolate_positive_roots with Möbius transforms _mobius_to_interval, dup_root_lower_bound). Refinement is dup_inner_refine_real_root/dup_refine_real_root. Complex roots: dup_isolate_complex_roots_sqf uses Sturm-sequence quadrant counting (_winding_number, _traverse_quadrants, _vertical_bisection/_horizontal_bisection). dup_count_real_roots/dup_count_complex_roots count roots.
- Gröbner bases, groebnertools.groebner: _buchberger (Gebauer-Möller pair criteria, selection by normal strategy) is the default (GROEBNER='buchberger'); _f5b is the alternative; red_groebner reduces. fglmtools.matrix_fglm does order conversion for GroebnerBasis.fglm.
- Root formulas, polyroots.roots: roots_linear/quadratic/cubic (Cardano with special-case branches)/quartic (Ferrari, plus a Descartes/Euler variant), roots_binomial, roots_cyclotomic, roots_quintic (uses the polyquinticconst tables). preprocess_roots and decompose come first.
- CRootOf (rootoftools.ComplexRootOf): indexing is reals ascending, then complex roots. Results are memoized in the global _reals_cache/_complexes_cache (_pure_key_dict). _eval_evalf calls mpmath.findroot inside the refined interval.
- nroots (Poly.nroots): mpmath.polyroots (Durand-Kerner) with extraprec=10*deg, retried at 15*deg. Roots are sorted with key (is_complex, re, |im|, sign(im)).
- Partial fractions: partfrac.apart (undetermined coefficients) and apart_full_decomposition / apart_list (Bronstein). rationaltools.together.
- Expr to poly: polyutils._dict_from_expr, _sort_gens and _parallel_dict_from_expr; constructor.construct_domain (_construct_simple, _construct_algebraic via primitive_element, _construct_composite, EX fallback).
- Sparse arithmetic, rings.PolyElement: __mul__ via monomial_mul from exec-generated MonomialOps, __pow__ via multinomial_coefficients, div/rem against a list of divisors, _gcd_QQ via ZZ heugcd. FracElement keeps num/den cancelled with gcd.
- Series in ring_series.rs_*: truncated power series (rs_exp, rs_log, rs_series, rs_compose_add used by minpoly), with puiseux.PuiseuxRing for fractional exponents.
- Special polynomials: orthopolys dup_jacobi/chebyshevt/legendre/... and appellseqs dup_bernoulli/euler/genocchi/andre. specialpolys has swinnerton_dyer_poly (via minimal_polynomial) and cyclotomic_poly.

**Data structures.**

- Dense recursive list (DMP): Vec of coefficients, highest degree first, with level u giving the nesting. The zero polynomial is u nested empty lists.
- DMP/DMF/ANP wrappers that carry (rep, dom, lev). ANP carries (rep, mod, dom) for algebraic numbers mod a minimal polynomial.
- PolyRing (symbols, domain, order, ngens, generated monomial ops) and PolyElement, a dict from exponent tuple to coefficient with a cached hash.
- FracField/FracElement (numer, denom PolyElements kept cancelled).
- Monomial tuples and orderings (lex, grlex, grevlex, ilex…) as key functions.
- Options dictionary with typed options and their dependencies (gens, domain, extension, modulus, symmetric, field, greedy, wrt, sort, order, auto, frac, formal…).
- RealInterval/ComplexInterval objects (mutable refinement) and the global CRootOf caches keyed by PurePoly.
- GroebnerBasis (Basic) holding exprs, gens, options.

**Hard dependencies.**

- polys.domains: every algorithm is generic over a Domain object (ZZ, QQ, GF(p), ZZ[y], QQ<a>, RR, EX…), and composite domains (ZZ[y], ZZ(x,y)) store sparse PolyElement/FracElement coefficients inside dense DMP.
- Ground integers and rationals from the external cluster: python int, PythonMPQ, and the sympy.external.gmpy helpers isqrt/gcdex/invert.
- core Expr: Poly is a Basic; _dict_from_expr and expr_from_dict; CRootOf/RootSum are Expr; EX elements call Expr.cancel and expand on every operation.
- ntheory base: nextprime (euclidtools, factortools, specialpolys), isprime/factorint/divisors (factortools, polyroots).
- Float layer (the mpmath replacement): polyroots for nroots, findroot for CRootOf.evalf, mpf/mpc for RR/CC.
- polys.numberfields: AlgebraicField construction and extension= factoring need minimal_polynomial/primitive_element. specialpolys.swinnerton_dyer_poly uses minpoly.
- polys.matrices: polys.solvers.solve_lin_sys/_solve_lin_sys and fglm-style linear algebra.

**Python features that are hard to port.**

- Duck typing: the same dup_* code runs on int, PythonMPQ, ModularInteger, ANP, PolyElement, FracElement, mpf/mpc and Expression via K.one/K.zero and Python operators.
- monomials.MonomialOps generates monomial_mul/div/lcm source code and exec()s it, memoized with @cacheit.
- PolyElement subclasses dict (in-place construction, cached _hash). PolyRing.__new__ setattr's generator names onto the ring (R.x) and builds dtype as a bound constructor.
- polyoptions uses an OptionType metaclass registry plus a topological sort over option requires/excludes.
- polyconfig is global mutable configuration whose configure() runs eval() on SYMPY_* environment variables. Drop the eval in Rust.
- Exceptions are used as control flow: HeuristicGCDFailed falls back to PRS, ExtraneousFactors restarts Wang, CoercionFailed drives conversion attempts.
- GROUND_TYPES selects DUP_Flint in DMP.__new__ for univariate ZZ/QQ/GF(p), and factortools line 685 calls fmpz_poly.factor.
- Global caches: CRootOf _reals_cache/_complexes_cache, @cacheit on Basic constructors, and mpmath global precision mutated inside nroots.
- Unbounded Python ints everywhere; ordering of lists and tuples is relied on for sorting.

**Behaviour a port must reproduce.**

- Generator order (polyutils._sort_gens) is ranked by str(gen): x,y,z first (124-126), then p..w (216-223), then a..o (301-315), then everything else (1000). Ties break on name, then on the trailing integer suffix parsed numerically (x2 < x10). opt.wrt and opt.sort override this. Every Poly repr and factor/cancel output form depends on it.
- Python floor division and modulo: ZZ quo/rem and gf_* reduce `% p` into [0,p), and heugcd uses symmetric remainders. Rust's truncating `/` and `%` silently change results, so the Int type needs div_floor/rem_euclid wrappers.
- Zero and degree conventions: dup_degree([]) returns float('-inf') while Poly(0,x).degree() returns S.NegativeInfinity. dmp_zero(u) is u nested empty lists. dmp_strip and dmp_validate rules apply.
- Factor ordering: _sort_factors sorts by key (len(f), multiplicity, coefficient lists) using Python list/tuple ordering of domain elements. For GF(p) that compares the stored value in [0,p) and raises a deprecation warning under modulus=. Ordering of ANP/PolyElement elements also matters. The coeff/content sign normalization in dup_factor_list and dmp_factor_list feeds Expr-level factor output.
- Normalization: ZZ gcd has positive LC and field gcds are monic (dup_qq_heu_gcd rescales the cofactors). cancel((2*x**2-2)/(4*x-4)) gives x/2 + 1/2. Expr-level cancel/factor rebuild expressions, so they rely on core Mul/Add canonical ordering.
- Inexact domains: factor/gcd over RR convert to QQ via RealField.to_rational with limit_denominator max_denom=max(2**prec//200, 99), not the exact binary value. factor(0.1*x**2+0.3*x+0.2) gives 0.2*(0.5*x + 1.0)*(1.0*x + 1.0).
- EX domain: every Expression op runs ex.cancel().expand(...), and sign comes from could_extract_minus_sign. EXRAW does no simplification at all, so result forms depend on the exact operation order.
- Root isolation intervals are algorithm-specific exact rationals (Poly(x**3-2).intervals() == [((1, 2), 1)]), and tests assert endpoints. The CRootOf global caches refine intervals in place, so a .interval value depends on earlier evaluations.
- nroots mutates mpmath.mp.dps globally and tests compare Floats exactly ([-1.0, 1.0]). Matching needs bit-identical polyroots/cleanup.
- Wang EEZ and gf_random consume the global sympy.core.random rng (gf_random calls int(uniform(0, p)) with float uniform). Outputs are canonical after sorting, but the work done is history-dependent.
- Exception parity: tests assert about 25 polyerrors classes via raises() (167 in test_polytools alone): PolificationFailed, GeneratorsNeeded, CoercionFailed, ExactQuotientFailed, NotInvertible, DomainError, UnificationFailed, MultivariatePolynomialError…
- Canonical vs algorithm-dependent outputs. gcd, cofactors, factor_list, sqf_list, resultant, reduced Gröbner bases and roots are canonical, so faster algorithms may be swapped in later. Isolation intervals, CRootOf internals, EXRAW forms and the partial-fraction intermediate forms are not.

**Rust prior art.**

- dashu-int/dashu-ratio (MIT OR Apache-2.0): bigint and rational base. Wrap them in an enum Int{Small(i64),Big} fast path.
- feanor-math (believed MIT, licence to verify): polynomial rings, factoring over Z/Q/finite fields/number fields, Gröbner. Use as a cross-check or design reference, not as a drop-in, because its normalizations differ.
- FLINT (LGPL, excluded): use its algorithms (fmpz_poly_factor van Hoeij, fmpz_mpoly_gcd Brown/Zippel, nmod_poly) as the performance target at the seams SymPy hands to python-flint.
- Symbolica (source-available, not OSS, excluded): performance reference only.
- rustsmith sympy-rust (PR #98): measured dup_mul 5x and dmp_mul 8.75x faster than SymPy, but gcd 0.17x, until SymPy's subresultant PRS was ported.
- rand_mt (MIT OR Apache-2.0): MT19937, if CPython-identical RNG behaviour is wanted for Wang and gf_random.

**How its tests check results.** Structural == on Poly, Expr and Python lists. test_densearith/densebasic/densetools/galoistools call dup_*(…, ZZ) on raw nested int lists (about 1,200 asserts that are easy to translate mechanically). test_euclidtools, test_factortools, test_rings, test_rootisolation, test_sqfreetools and test_groebnertools go through ring('x,y', ZZ) and R.dup_*/R.dmp_* (the compatibility IPolys surface). test_polytools has 1,616 asserts and 167 raises(). There are few str/repr checks, @slow in four files, and test_hypothesis.py uses hypothesis. All tests assume python ground types (the oracle venv has no gmpy2 or flint).

**Notes.** Measured on the pinned oracle (python ground types). Expr-level gcd of two dense 3-variable polynomials of degree 10 and 13 took 98 ms through the dense DMP path, versus 13.6 ms for the same pair as sparse PolyElement.gcd (heugcd), so a sparse-first multivariate GCD in Rust is a free win because the output is canonical. Also measured: factor_list of an irreducible degree-29 polynomial 19 ms; 3-variable Wang factor 38 ms; cancel((x**2-1)/(x-1)) 0.84 ms (Expr round trip dominates); groebner cyclic-3 1 ms. Suggested Rust crate split: rsympy-polys-dense (dup/dmp generic over Domain), rsympy-polys-sparse (PolyRing/PolyElement/FracField, groebner, series), and rsympy-polys (Poly API + Expr conversion), with core callbacks through a trait so the crates do not form a cycle. Port compatibility.IPolys mechanically, or translate the ring-based tests to Rust directly. Defer modulargcd, subresultants_qq_zz and multivariate_resultants to the end, or run them as Python on the Rust core.

### `polys.domains`

6,947 source lines, 57 test functions. Difficulty **L**. Strategy: **redesign**. App relevance: serve.

The coefficient-domain system (6,947 lines, 32 files). Domain base class (domain.py, 1,382 lines): convert/convert_from, unify, is_positive/is_negative/canonical_unit, gcd/lcm/exquo/quo/rem, get_field/get_ring/get_exact, poly_ring/frac_field/algebraic_field constructors. Concrete domains: ZZ (IntegerRing: ZZ_python/ZZ_gmpy/flint), QQ (RationalField, PythonMPQ), GF(p) (FiniteField with ModularInteger classes from a factory), ZZ_I/QQ_I (gaussiandomains), AlgebraicField QQ<a> (ANP elements), PolynomialRing (sparse PolyElement), FractionField (FracElement), RR/CC (mpmath-backed RealElement/ComplexElement), EX (ExpressionDomain, cancel on every op), EXRAW (raw Expr), plus old_polynomialring/old_fractionfield (DMP-based, used by agca) and QuotientRing.

**Why this strategy.** The Python design (dynamic dtype classes, ModularIntegerFactory, name-based from_* dispatch, the gmpy/flint/python ground-type switch, mpmath context subclasses) cannot be transliterated. In Rust it becomes a closed Domain enum plus generic element types: ZZ/QQ/GF(p) monomorphized with small-int fast paths, composite and algebraic domains behind Arc'd parameters. The observable semantics (the unify table, conversions, sign and canonical_unit rules, RR max_denom, printing names) must still be reproduced exactly from the oracle.

**Key algorithms.**

- Domain.unify (domain.py): characteristic check, then EXRAW > EX > FiniteExtension > Composite (unify_composite) > CC/RR by precision > AlgebraicField (rebuilt over _unify_gens of orig_ext, which re-runs primitive_element) > Gaussian > QQ > ZZ, with EX as the fallback.
- Domain.convert_from dispatches on the method name 'from_' + K0.alias (or the class name). That makes an N×N conversion matrix: from_ZZ, from_QQ_python, from_RealField, from_AlgebraicField, from_PolynomialRing, from_FractionField, from_ExpressionDomain…
- RealField.to_rational / to_rational() (realfield.py): mpmath to_rational, then a continued-fraction limit_denominator with max_denom=max(2**prec//200, 99).
- AlgebraicField.__init__: to_number_field(ext) builds the primitive element, mod = minpoly.rep. Also maximal_order/integral_basis/discriminant/primes_above/galois_group, which delegate to numberfields.
- GaussianIntegerRing gcd (Euclid with rounded division) and canonical_unit via quadrant; QQ_I as_AlgebraicField.
- FiniteField(mod, symmetric=True): elements stored in [0,p), printed and converted to_sympy symmetrically. ModularIntegerFactory caches one class per (mod, dom, symmetric).

**Data structures.**

- Domain enum: ZZ, QQ, GF{p, symmetric}, ZZ_I, QQ_I, Alg(Arc<AlgField{ext, mod: DMP, orig_ext, alias}>), PolyRing(Arc<PolyRing>), FracField(Arc<FracField>), RR{prec}, CC{prec}, EX, EXRAW, plus OldPolyRing/QuotientRing for agca.
- Element types: Int (small or big), Rational (normalized p/q, q > 0), ModInt (u64 or bigint plus modulus), GaussianInt/GaussianRat, ANP, PolyElement, FracElement, Float/Complex at a given precision, Expr.

**Hard dependencies.**

- Ground integers and rationals (sympy.external.gmpy: MPZ/MPQ, PythonMPQ, isqrt/gcdex/invert).
- Float layer replacing mpmath (MPContext, mpf/mpc, to_rational, almosteq) for RR/CC.
- polys (rings/fields for PolynomialRing/FractionField; polyclasses ANP/DMP).
- polys.numberfields (AlgebraicField construction: to_number_field/minpoly/primitive_element).
- core Expr (EX/EXRAW, to_sympy/from_sympy).
- ntheory.isprime (FiniteField validity).

**Python features that are hard to port.**

- ModularIntegerFactory builds a new class per modulus, cached in _modular_integer_cache.
- getattr(self, 'from_' + alias) dispatch, with alias strings such as ZZ_python and QQ_gmpy.
- Ground types chosen at import time from the environment (python/gmpy/flint).
- RealElement/ComplexElement subclass mpmath _mpf/_mpc and carry a context.
- ZZ[x] __getitem__ syntax for ring construction; Domain.__call__ for element construction.

**Behaviour a port must reproduce.**

- AlgebraicField.is_positive/is_negative look at the sign of a.LC(), not the real value: K.is_negative(K.from_sympy(1 - sqrt(2))) is True. This drives gcd and factor normalization over QQ<a>.
- EX sign: is_positive uses ex.as_coeff_mul()[0].is_positive and is_negative uses ex.could_extract_minus_sign(), so it depends on core heuristics. ExpressionDomain.gcd returns 1, but element .gcd calls polys gcd.
- GF(p) is symmetric by default: Poly(x**2+5*x+6, x, modulus=7) prints x**2 - 2*x - 1, yet int(GF(7)(5)) == 5 and comparisons use the stored value in [0,p) (with a deprecation warning).
- RR to QQ/ZZ conversion is approximate (limit_denominator with max_denom tied to prec), so QQ.convert(RR(0.1)) == 1/10. Core's Rational(0.1) is the exact binary value instead.
- construct_domain picks the domain: [1/2, sqrt(2)] gives EX unless extension=True (then QQ<sqrt(2)>); x + y*pi in x gives ZZ[y,pi]; x/y + 1 gives ZZ(y); sin(y)*x + 1 gives ZZ[sin(y)]; floats give RR with the maximum precision.
- Unify symmetry hack for two finite extensions (ordered moduli), and GaussianRing with QQ gives QQ_I.
- Domain equality and hash are structural (QQ<sqrt(2)> created twice compares equal) and feed caching. Domains print as 'QQ<sqrt(2) + sqrt(3)>', 'ZZ[x,y]', 'ZZ(x,y)', 'GF(7)', 'RR', 'EX'.

**Rust prior art.**

- dashu-int and dashu-ratio (MIT OR Apache-2.0) for ZZ/QQ.
- ibig (MIT OR Apache-2.0), dashu's predecessor.
- num-bigint/num-rational (MIT OR Apache-2.0): reported far slower than GMP-class libraries on huge operands (10M-digit e benchmark: 482 s vs 2.8 s for rug); fine for small sizes.
- malachite: LGPL-3.0, excluded (this settles the 'Malachite licence' unknown in docs/research/rust-rewrites.md).
- rug: LGPL (GMP), excluded.
- dashu-float / astro-float (MIT) as possible bases for RR/CC, but mpmath-identical rounding and printing must be implemented on top.

**How its tests check results.** test_domains.py has 791 asserts: structural == on domains and elements, plus raises(UnificationFailed/CoercionFailed/…) for the unify and convert tables. test_polynomialring.py and test_quotientring.py test old_polynomialring/QuotientRing.

**Notes.** Ground numbers (ZZ int, QQ PythonMPQ) actually live in sympy.external and core, so coordinate one Int/Rational type across core.numbers, polys.domains and ntheory. A generic BigInt rational was about 3x slower than CPython until an i64 fast path was added (rustsmith). PythonMPQ cross-cancels (gcd(ap,bq), gcd(bp,aq)) in __mul__; copy that. Port old_polynomialring/old_fractionfield/quotientring together with agca.

### `polys.matrices`

10,762 source lines, 315 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: compile.

DomainMatrix (10,762 lines): exact linear algebra over any Domain. It is also the internal storage of every explicit sympy Matrix (RepMatrix._rep, an SDM over ZZ, QQ or EXRAW). Formats: DDM (dense list-of-lists), SDM (sparse dict-of-dicts) and DFM (python-flint fmpz_mat/fmpq_mat/nmod_mat; absent under the oracle's python ground types). Operations: arithmetic and matmul, rref, rref_den, det, inv/inv_den, adjugate, lu/lu_solve, fflu, qr, nullspace, columnspace, solve_den, charpoly with charpoly_factor_blocks/charpoly_berk, scc, clear_denoms, lll, Smith/Hermite normal forms (normalforms.py), eigenvectors (eigen.py), linsolve (linsolve.py).

**Why this strategy.** Several outputs depend on the algorithm (rref_den denominators, EXRAW forms, LLL basis, the format each method chooses), and the sympy Matrix class sits on top of these semantics. No permissive Rust crate does exact linear algebra generic over a runtime domain (nalgebra and faer are floating point). Port DDM and SDM faithfully, then add native dense fast paths for ZZ/QQ/GF(p) in place of DFM, normalized to the python-ground outputs.

**Key algorithms.**

- rref method choice, rref._dm_rref_choose_method_ZZ/_QQ: GJ (Gauss-Jordan over the field), FF (fraction-free Bareiss rref_den) or CD (clear denominators, then FF), picked from row density, nonzero row count, bit lengths and denominator lcm (for example 'CD' if denom_lcm.bit_length() < 50).
- Dense kernels in dense.py: ddm_irref (partial pivoting for RR/CC), ddm_irref_den, ddm_idet (Bareiss with exquo for rings, plain elimination for fields), ddm_iinv, ddm_ilu/ddm_ilu_solve, ddm_berk (Berkowitz).
- Sparse kernels in sdm.py: sdm_irref, sdm_rref_den, sdm_matmul plus sdm_matmul_exraw (keeps 0*oo -> nan and uses K.sum), sdm_nullspace_from_rref, sdm_berk. SDM.det converts to dense (to_dfm_or_ddm().det()).
- charpoly = product of charpoly_factor_blocks: strongly connected components give block-triangular structure, then Berkowitz per block, multiplied with dup_mul.
- normalforms: smith_normal_form/invariant_factors, hermite_normal_form (_hermite_normal_form and the modulo-D variant _hermite_normal_form_modulo_D, which uses ntheory symmetric_residue).
- lll._ddm_lll: exact rational LLL with delta=3/4 (QQ Gram-Schmidt), optionally returning the transform.
- eigen.dom_eigenvects: charpoly, then dup_factor_list. Linear factors give nullspace over K. Higher-degree factors build FiniteExtension(minpoly) from agca and take the nullspace over K[x]/(m).
- linsolve._linsolve: equations to a sparse augmented matrix, sdm_irref, then the parametric solution (used by solveset.linsolve).

**Data structures.**

- DDM: Vec<Vec<K>> plus shape and domain.
- SDM: map row -> map col -> nonzero K (BTreeMap or sorted-vec rows; keep zeros out).
- DomainMatrix {rep: DDM|SDM|native dense, shape, domain, fmt}.
- DomainScalar.
- Pivot lists, permutation swaps (LU) and denominators (rref_den, inv_den).

**Hard dependencies.**

- polys.domains (every element op; ZZ/QQ/EXRAW for Matrix storage; AlgebraicField for extension=True).
- polys dense layer (dup_mul for charpoly, dup_factor_list for eigen and charpoly_factor_list).
- polys.agca.extensions.FiniteExtension (eigen).
- core Expr (EXRAW/EX elements, Add(*items) via K.sum).
- ntheory.modular.symmetric_residue (normalforms).

**Python features that are hard to port.**

- DDM subclasses list and SDM subclasses dict, and both are mutated in place by ddm_i*/sdm_i* functions.
- Format switching via to_dfm_or_ddm/to_sparse, with duck-typed rep objects.
- Method selection by density and bit-length heuristics computed from Python ints.
- Python set iteration order in the sparse algorithms.

**Behaviour a port must reproduce.**

- rref_den is not canonical: dense and sparse, FF and GJ give results scaled differently (the comment at the top of tests/test_rref.py says so). The method heuristics therefore change observable output and must be copied exactly.
- EXRAW and EX results are not canonical: charpoly, inv and rref forms depend on the exact operation order and on dense vs sparse format. Matrix.charpoly for symbolic matrices runs DomainMatrix.charpoly on EXRAW.
- sdm_matmul_exraw keeps nan when a zero meets oo; a plain sparse matmul that skips zeros loses it.
- SDM omits zero entries. Iteration order (dict and set intersections) feeds Add argument order, which core canonicalizes afterwards.
- DFM (flint) outputs differ for non-canonical operations (LLL basis, rref_den). The oracle is python ground types, so DDM/SDM semantics are the reference.
- Every Matrix construction and equality goes through DomainMatrix (RepMatrix.__eq__ calls unify_eq). The domain chosen (ZZ, QQ or EXRAW; Matrix([[1.5]]) is EXRAW) must match.
- Exceptions DMNonSquareMatrixError, DMNonInvertibleMatrixError, DMShapeError, DMDomainError, DMRankError, DMBadInputError are asserted by tests.

**Rust prior art.**

- None for exact runtime-domain matrices. nalgebra (Apache-2.0) and faer (MIT) are float-only and possibly useful for RR/CC numeric helpers only.
- FLINT fmpz_mat/fmpq_mat/nmod_mat (LGPL, excluded) are the performance target for the dense ZZ/QQ/GF(p) path.

**How its tests check results.** 315 test functions. test_xxm.py is parametrized over DDM/SDM/DFM (62 parametrize, 68 flint-skip mentions). test_rref.py has a 16-case RREF_EXAMPLES table compared after cancel_denom. test_domainmatrix.py has 334 asserts and 93 raises(). Everything is structural == on DomainMatrix (shape, domain and entries). test_lll uses raises for invalid input.

**Notes.** Matrix.inv defaults to DomainMatrix inv_den whenever the domain is not EX (matrices/inverse.py). Matrix.rref uses DomainMatrix for ZZ/QQ (reductions._to_DM_ZZ_QQ). Matrix.eigenvects uses DomainMatrix.from_Matrix(M, field=True, extension=True).to_dense() and dom_eigenvects. Measured: Matrix.det 8x8 over ZZ 2.4 ms, charpoly 0.67 ms. The ZZ/QQ/EXRAW subset can be ported early in parallel with the polys dense layer, because it needs only domains plus dup_mul.

### `polys.agca`

2,935 source lines, 37 test functions. Difficulty **M**. Strategy: **port faithfully**. App relevance: compile.

Commutative algebra in the style of Greuel and Pfister (2,935 lines): free modules, submodules and quotient modules over polynomial rings, including local rings with ilex orderings (modules.py). Also ideals and quotient rings (ideals.py), module homomorphisms with kernel and image (homomorphisms.py), and extensions.py with FiniteExtension/MonogenicFiniteExtension (K[x]/(f)) and ExtensionElement.

**Why this strategy.** Small and self-contained, with no Rust prior art. Results are presented in generator-dependent forms, so a faithful port is the only way to match the oracle. Port extensions.py (356 lines) early, because Matrix.eigenvects needs FiniteExtension. Modules, ideals and homomorphisms plus old_polynomialring and distributedmodules can come late; until then they can run as Python on the Rust core through the planned PyO3 shim.

**Key algorithms.**

- Module Gröbner bases and normal forms via distributedmodules.sdm_groebner and sdm_nf_mora (Mora's tangent-cone algorithm for local orderings) and sdm_spoly. Called from SubModulePolyRing._groebner, in_terms_of_generators and syzygy_module.
- Ideal operations (intersection, quotient, saturation, radical membership) implemented via module computations.
- FiniteExtension arithmetic: an ExtensionElement holds a DMP polynomial reduced modulo the modulus. Inverse via gcdex. Used for algebraic eigenvectors.

**Data structures.**

- FiniteExtension {modulus: Poly, domain, symbol}; ExtensionElement {rep: DMP, ext}.
- SDM module vectors: lists of ((component, monomial…), coeff) terms (distributedmodules).
- FreeModule/SubModule/QuotientModule class hierarchy with generator lists and cached Gröbner bases.
- ModuleHomomorphism with a matrix representation.

**Hard dependencies.**

- polys.domains old_polynomialring (GlobalPolynomialRing/GeneralizedPolynomialRing with DMP elements) and quotientring.
- polys.distributedmodules (sdm_* Mora/Buchberger for modules).
- polys dense layer (DMP, gcdex) for FiniteExtension.
- polys.matrices consumes FiniteExtension (eigen.py).

**Python features that are hard to port.**

- Deep class hierarchies with operator overloading (+, *, in, ==) and mixins (FreeModulePolyRing, SubModuleQuotientRing…).
- Lazy cached _gb attributes.
- Relies on old DMP-based domains that the main polys code no longer uses.

**Behaviour a port must reproduce.**

- FiniteExtension is on the Matrix.eigenvects path (polys.matrices.eigen.dom_eigenvects). Element representation and to_sympy form decide the printed eigenvector entries, which are then substituted with roots()/CRootOf.
- Local rings with ilex/igrlex orderings change which generators count as units, so containment and normal forms differ from the global case.
- Results such as syzygies and homomorphism kernels are presented in generator-dependent forms that tests compare structurally.

**Rust prior art.**

- None permissive. Singular (GPL) and Macaulay2 (GPL) are algorithm references only.

**How its tests check results.** 4 test files, 37 test functions: 334 asserts, 47 raises(), 7 str/repr. Structural == on modules, ideals and homomorphisms, membership tests (x in M) and is_* predicates.

**Notes.** Only extensions.FiniteExtension matters for the app roadmap (Markov eigenvectors with irrational eigenvalues). Everything else here is full-rewrite scope with no consumer outside polys.domains.ring.ideal/quotient_ring.

### `polys.numberfields`

6,852 source lines, 123 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: serve.

Algebraic number theory (6,852 lines). minpoly.py covers minimal_polynomial/minpoly for algebraic expressions. subfield.py covers primitive_element, field_isomorphism and to_number_field (which build every AlgebraicField). Further files: basis.py (round_two maximal order), primes.py (prime_decomp, prime_valuation, PrimeIdeal), modules.py (PowerBasis/Submodule/ModuleHomomorphism with HNF), galoisgroups.py with galois_resolvents.py and resolvent_lookup.py (galois_group for degree 6 or less), and utilities.py (isolate, AlgIntPowers, coeff_search, extract_fundamental_discriminant).

**Why this strategy.** Minpoly results are canonical, but primitive elements, ideal generators and the numeric factor choice are not. There is no permissive Rust number-field library: PARI is GPL and Hecke is Julia. The code is a straightforward composition of polys primitives, so a faithful port that tracks the oracle is the low-risk route. Port the minpoly/subfield subset early, since core assumptions, simplify and AlgebraicField depend on it; port round_two, prime_decomp, modules and galois groups late.

**Key algorithms.**

- minimal_polynomial via _minpoly_compose. Addition and multiplication use resultant composition (_minpoly_op_algebraic_element, with ring_series.rs_compose_add for addition). Powers use _minpoly_pow. Rational multiples of pi in sin, cos, tan and exp go through Chebyshev and cyclotomic constructions (_minpoly_sin/_cos/_tan/_exp). CRootOf uses _minpoly_rootof.
- _choose_factor: evaluates each candidate factor numerically at increasing precision (prec1 from 10 doubling up to 200) and accepts one when the second-smallest |f(v)| exceeds the smallest by 10^6.
- _minimal_polynomial_sq handles sums of surds and nth roots. Also used directly by simplify.nthroot via _is_sum_surds and _minimal_polynomial_sq.
- _minpoly_groebner: the compose=False path and fallback. It substitutes generators, runs a lex Gröbner basis, then factor_list.
- primitive_element: iterative sqf_norm. Factor the minpoly over the previous field, take sqf_norm to get s, set gen += s*ext; ex=True also returns the representations. The result is _canonicalized (primitive, positive LC).
- field_isomorphism: field_isomorphism_pslq (mpmath pslq) when fast=True, otherwise field_isomorphism_factor.
- round_two (Zassenhaus Round Two with the Dedekind criterion and nilradical_mod_p). prime_decomp (easy case via factoring T mod p; otherwise kernel and splitting in the style of Buchmann-Lenstra). Hermite normal forms via polys.matrices.normalforms.
- galois_group: degree 3 via discriminant; degree 4 via root approximation or lookup; degree 5 hybrid/ext-factor; degree 6 via resolvent lookup tables. tschirnhausen_transformation retries draw from random.randint.

**Data structures.**

- AlgebraicNumber (core) and ANP (rep mod minpoly over QQ).
- PowerBasis/Submodule: a DomainMatrix basis over QQ with a denominator. ModuleElement/PowerBasisElement.
- PrimeIdeal {ZK, p, alpha, f, e}.
- Resolvent objects with numerically evaluated coefficient functions. Precomputed resolvent_lookup tables (456 lines of data).

**Hard dependencies.**

- polys (Poly, factor_list over QQ and algebraic fields, resultant, sqf_norm, groebner, roots, CRootOf, ring_series.rs_compose_add, specialpolys.cyclotomic_poly, orthopolys.dup_chebyshevt).
- polys.domains (AlgebraicField, ANP, FF).
- polys.matrices (DomainMatrix, hermite_normal_form) for modules, primes and basis.
- core Expr plus evalf: _choose_factor uses .n(prec) and subs; Expr.is_* queries.
- Float layer: MPContext in galois_resolvents and subfield, pslq in field_isomorphism_pslq.
- ntheory (divisors, sieve, factorint, is_square).
- combinatorics.galois (S3..S6TransitiveSubgroups, Permutation) for galois_group.
- utilities.lambdify (galois_resolvents builds numeric resolvent evaluators).

**Python features that are hard to port.**

- lambdify-generated numeric evaluators (galois_resolvents).
- mpmath MPContext instances held per computation.
- Module classes with mixins and operator overloading (IntegerPowerable).
- Global random.randint inside tschirnhausen_transformation.

**Behaviour a port must reproduce.**

- Core depends on this module. Expr.equals and Expr._eval_is_extended_positive_negative (core/expr.py around lines 771-929) call minimal_polynomial when evalf has no significance. Measured: simplify(sqrt(2)+sqrt(3)-sqrt(5+2*sqrt(6))) makes 4 minimal_polynomial calls (17 ms in total).
- Primitive element choice (the s values from sqf_norm) is algorithm-specific and visible: QQ.algebraic_field(sqrt(2), sqrt(3)).ext == sqrt(2) + sqrt(3), and it appears in printed domains and AlgebraicNumber.
- Minpoly normalization: primitive integer coefficients with positive LC. With no x given it uses Dummy x and PurePoly. NotAlgebraic is raised for transcendental input. The domain= argument allows QQ<...> or QQ(y).
- _choose_factor depends on evalf precision behaviour. If evalf digits differ, a different factor may be chosen, or NotImplementedError('multiple candidates') raised.
- prime_decomp generators and the order of the output ideals are algorithm-dependent.
- galois_group with randomize=False must stay deterministic (it uses its own coefficient history). randomize=True uses Python's random module.

**Rust prior art.**

- feanor-math (licence to verify) has number fields and orders. Use it only as a cross-check.
- PARI/GP (GPL, excluded) as the reference for round2 and prime decomposition.
- None that reproduce SymPy's primitive-element choices.

**How its tests check results.** 8 test files, 123 test functions: 608 asserts, 68 raises(), 1 @slow. Structural == on minpoly Exprs and PurePoly, on (poly, coeffs, reps) tuples from primitive_element, and on galois_group(T, by_name=True) == (S4TransitiveSubgroups.X, alt). test_numbers.py exercises core AlgebraicNumber.

**Notes.** app_relevance is serve only for minimal_polynomial and _minimal_polynomial_sq (simplify, equals and sign of algebraic constants). AlgebraicField/primitive_element are compile-time (Matrix.eigenvects with extension=True; factor with extension=). The rest is roadmap. Measured: minimal_polynomial(sqrt(2)+sqrt(3)+sqrt(5)) 9.9 ms. The FLINT blog's minpoly case is over 5 minutes in pure SymPy versus 300 ms with FLINT, so composition via resultants needs fast QQ[x] resultants and factoring.

### `polys.benchmarks`

634 source lines, 0 test functions. Difficulty **S**. Strategy: **redesign**. App relevance: none.

Three benchmark scripts (634 lines, no tests). bench_solvers.py runs large sparse linear systems through polys.solvers.solve_lin_sys over PolyRing/FracField domains. bench_galoispolys.py covers GF(p) factoring (gf_factor_sqf) with large primes from nextprime. bench_groebnertools.py computes groebner of a classic system.

**Why this strategy.** Turn these into Rust criterion/divan benchmarks with identical inputs, plus a script that times the same workloads on the pinned SymPy. This makes the speed goal measurable at the seams where FLINT normally gives 30-100x (GF(p) factoring, sparse linear solve, Gröbner).

**Key algorithms.**

- Workload definitions only: solve_lin_sys on systems such as eqs_10x8/165x165/189x49 with symbolic coefficients, gf_factor_sqf on random dense GF(p) polynomials, and groebner via buchberger/f5b.

**Data structures.**

- Benchmark input tables (equation lists, prime moduli, polynomial seeds).

**Hard dependencies.**

- polys (solvers, galoistools, groebnertools, rings)
- ntheory.nextprime

**Python features that are hard to port.**

- Module-level ring construction with xring/vring injecting names into globals.

**Behaviour a port must reproduce.**

- None functional. Timings only. Inputs must be reproduced exactly so the Rust/SymPy comparison is fair.

**Rust prior art.**

- criterion (MIT OR Apache-2.0)
- divan (MIT OR Apache-2.0)

**How its tests check results.** No tests. Plain timing functions (timeit_* naming) that the asv/bench harness picks up.

**Notes.** Extend with the hot spots measured during this survey: 3-variable gcd (98 ms Expr path vs 13.6 ms sparse), Wang factor (38 ms), simplify on nested surds (17 ms), factorint(2**128+1) (4.05 s), cancel round trip (0.84 ms).

### `ntheory`

9,742 source lines, 89 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: serve.

Integer number theory (9,742 lines, 15 files). Primality (primetest.isprime, mr, Lucas/BPSW variants, proth_test, is_mersenne_prime, is_gaussian_prime). Prime generation (generate: the global Sieve, prime, primepi, nextprime/prevprime, primerange, randprime, primorial, composite, compositepi, cycle_length). Factoring (factor_: factorint, factorrat, perfect_power, multiplicity, pollard_rho/pm1, divisors/totient/divisor_sigma and arithmetic functions, plus the global FactorCache; ecm.py; qs.py SIQS). Residues (residue_ntheory: n_order, primitive_root, sqrt_mod, nthroot_mod, discrete_log, legendre/jacobi, mobius, polynomial_congruence, binomial_mod). Also partitions_.npartitions (Hardy-Ramanujan-Rademacher), modular.crt/solve_congruence, multinomial, continued_fraction, egyptian_fraction, digits, bbp_pi and elliptic_curve.

**Why this strategy.** Most functions are short integer algorithms whose outputs are mathematically canonical (complete factorization, primality, primepi, npartitions), so any correct implementation would match. The observable edge cases (limit= partial results, factor_cache history, seeded qs/ecm internals, verbose text) only match with a faithful port on a CPython-compatible RNG. Permissive crates (primal, num-prime) are useful as cross-checks or fast backends for the canonical functions only.

**Key algorithms.**

- isprime: trial division by primes up to 47, an Euler test below 65077 with 5 known exceptions, bisection in the sieve, a factor_cache lookup, then deterministic Miller-Rabin base sets up to 3.317e24 (with the Forisek-Jancina hashed single base for n < 4296595241), then is_strong_bpsw_prp from sympy.external.ntheory.
- factorint: _factorint_small trial division to 2**15 with fail_max=600, then factor_cache, then 3 Fermat steps, then rounds of trial/pollard_pm1(B=low, seed=high_)/pollard_rho(max_steps=low, seed=high_) on doubling ranges, then ECM (_ecm_one_factor: B1=10000, B2=100*B1, 50 curves, scaled ×5 and ×4) once n has at least 24 digits after 3 iterations. With limit= only partial work is done and composite cofactors are returned.
- perfect_power/_perfect_power (integer root search), multiplicity, factorrat.
- ecm: Montgomery-curve ECM with Suyama parametrization (sigma from randint(6, n-1), seeded). qs/qs_factor: self-initializing quadratic sieve (_generate_factor_base, _generate_polynomial with seeded randint, _gen_sieve_array, _trial_division_stage with partial relations, _find_factor via fast GF(2) Gaussian elimination).
- Sieve: a segmented, lazily extended array of primes with totientrange/mobiusrange. _primepi is an O(n^(3/4)) sieve-of-counts. prime(nth) bisects with li(m).evalf() bounds.
- residue_ntheory: _sqrt_mod_tonelli_shanks (random non-residue), _sqrt_mod_prime_power (Hensel lifting), composite moduli via CRT, _nthroot_mod1, discrete_log dispatching to trial_mul, shanks_steps (BSGS), pollard_rho (rseed), index_calculus or pohlig_hellman, plus n_order and primitive_root (smallest).
- npartitions: Hardy-Ramanujan-Rademacher via mpmath.libmp mpf_* at a computed precision (_a, _d with mpf_cosh_sinh), and _partition_rec for small n.
- crt (modular.py) delegates to polys.galoistools.gf_crt/gf_crt1/gf_crt2 (Garner-style).

**Data structures.**

- Global Sieve {_list primes, _tlist totients, _mlist mobius} with lazy extension (needs a Mutex/RwLock in Rust).
- FactorCache: an LRU map from n to its largest known prime factor (maxsize 1000).
- SIQS structures: FactorBaseElem {prime, tmem_p, log_p, soln1/soln2, b_ainv}, SievePolynomial {a, b, coefficients}, relation lists.
- Factor dict prime -> exponent (Python dict insertion order is visible when printed).

**Hard dependencies.**

- Ground integers with fast isqrt/iroot/gcdext/invert/jacobi/kronecker/is_strong_prp/_lucas_sequence (sympy.external.ntheory and gmpy shim).
- sympy.core.random (_randint(seed) reseeds the global rng) for pollard_rho/pm1, ecm, qs, discrete_log and randprime.
- polys.galoistools (gf_crt*, gf_csolve, linear_congruence) for modular.crt and polynomial_congruence. polys Poly/FF for elliptic_curve and residue_ntheory.
- Float layer (mpmath.libmp) for npartitions. core functions li/log evalf for prime(nth) and composite(nth).
- solvers.solve for elliptic_curve.

**Python features that are hard to port.**

- Global mutable singletons: sieve and factor_cache, plus a pluggable get_external network hook.
- recurrence_memo memoization (partitions, binomial helpers).
- Generators and yield APIs (sqrt_mod_iter, primerange, continued_fraction_iterator, divisors(generator=True)).
- array.array for sieve storage.
- Seeds accepted as int, None or a list (deterministic draws for tests).
- Unbounded ints throughout, plus float math.log/exp heuristics in qs and ecm.

**Behaviour a port must reproduce.**

- Core auto-evaluation depends on this module. Integer._eval_power (core/numbers.py) uses perfect_power and then Integer(b).factors(limit=2**15), which is factorrat with use_rho=False and use_pm1=False. sqrt(40009**2*40013) therefore stays sqrt(64049612601053), while sqrt(10007**2*3) becomes 10007*sqrt(3).
- The global factor_cache (an OrderedDict LRU with maxsize=1000) makes results history-dependent. After a full factorint(n), sqrt(4*n) for n=40009**2*40013 evaluates to 80018*sqrt(40013); in a fresh process it stays unsimplified (reproduced on the pinned oracle). factor_cache.get_external is a user hook that can make network calls; drop it or make it explicit opt-in.
- Partial factorization with limit= depends on the exact algorithm order and the seeds (seed=high_ for rho and pm1). Tests assert, for example, factorint(1951*15013*15053, limit=2000) == {225990689: 1, 1951: 1} and factorint(13*17*19, limit=15) == {13: 1, 17*19: 1}.
- Tests check SIQS internals with seed 1234: sieve_poly.eval_v, factor_base[i].tmem_p/log_p, g.a == 1133107, g.b == 682543. Matching needs a CPython-identical random.Random(seed).randint (MT19937 with init_by_array seeding and getrandbits-based _randbelow) and IEEE-identical math.log/exp thresholds.
- _randint(int) reseeds sympy's global rng as a side effect of factorint, pollard_rho, ecm and qs, which changes later random draws elsewhere.
- The global sieve extends lazily (sieve._list) and tests inspect it. isprime consults sieve and factor_cache, which changes speed but not results.
- Edge conventions: factorint(0) == {0: 1}, factorint(1) == {}, negative input adds -1: 1. visual=True returns an unevaluated Mul. multiple=True returns a list. Generator variants exist (divisors(generator=True), sqrt_mod_iter).
- sqrt_mod returns the smallest root unless all_roots=True (sorted). primitive_root returns the smallest. discrete_log returns the least exponent.
- Several functions are deprecated shims (11 @deprecated) whose Function versions now live in functions.combinatorial.numbers (primepi, totient, mobius, divisor_sigma, legendre_symbol, jacobi_symbol…).

**Rust prior art.**

- primal (MIT OR Apache-2.0): fast segmented sieve, prime counting and nth-prime estimates. Good for Sieve and primepi.
- num-prime (Apache-2.0): BPSW primality, factorization (rho, SQUFOF), nt functions. Use it as a cross-check.
- machine-prime (fast u64/u128 deterministic primality; licence to verify).
- num-integer (MIT OR Apache-2.0): div_floor, Roots, extended gcd.
- rand_mt (MIT OR Apache-2.0): MT19937 core for a CPython-compatible random.Random.
- GMP-ECM and msieve (GPL/LGPL, excluded) as algorithm references.

**How its tests check results.** 89 test functions: 963 asserts, 76 raises(), 4 @slow, 8 seed uses. Mostly integer ==, dict/set equality for factor outputs and list equality. qs/ecm tests assert sets of factors and internal structures. Verbose tests capture() printed text ('1000' in output). test_hypothesis.py runs property tests.

**Notes.** Split it into ntheory-base (primetest, generate, factor_, multinomial, integer CRT moved out of polys.galoistools, sqrt_mod) below polys and core.numbers, and ntheory-upper (polynomial_congruence, elliptic_curve, partitions, continued_fraction, egyptian_fraction, digits, bbp_pi, ecm, qs). Core uses perfect_power, factorrat, isprime, multiplicity and multinomial_coefficients. Measured: factorint(2**128+1) takes 4.05 s in pure-Python SymPy, which is a large-speedup target. isprime(2**127-1) takes 0.07 ms.

### `discrete`

1,208 source lines, 14 test functions. Difficulty **S**. Strategy: **port faithfully**. App relevance: none.

Exact discrete transforms and convolutions (1,208 lines). transforms.py: fft/ifft (symbolic radix-2 DFT over Expr with exact cos/sin of 2*pi/n), ntt/intt (number-theoretic transform modulo a prime of the form k*2^m + 1), fwht/ifwht (Walsh-Hadamard), mobius_transform/inverse_mobius_transform. convolutions.py: convolution dispatcher, convolution_fft, convolution_ntt, convolution_fwht, convolution_subset, covering_product, intersecting_product, and convolution_int (Kronecker substitution packing into one big integer). recurrences.py: linrec (linear recurrences) and linrec_coeffs.

**Why this strategy.** Small and mechanical. All the difficulty sits in core (Expr canonicalization, trig special values). Numeric FFT crates (rustfft) do not apply because the transforms are exact and symbolic.

**Key algorithms.**

- _fourier_transform: pad to a power of two, bit-reversal permutation (ibin), iterative butterflies with twiddles w = cos(ang*i) + I*sin(ang*i) and expand_mul on every product. With dps, evalf(dps+2) is applied to the angle.
- _number_theoretic_transform: requires isprime(p) and n | p-1, uses the primitive_root(p) generator.
- convolution dispatch: if every entry is an int or Rational, use convolution_int scaled by the denominator lcm. Otherwise use convolution_fft. Cyclic folding via cycle=.
- convolution_int: Kronecker packing with bound (1+min(deg))·max|a|·max|b| and signed digit unpacking (Fateman's 'polysbyGMP').
- linrec_coeffs: computes coefficients of x^n mod the characteristic polynomial (used by Matrix._eval_pow_by_cayley).

**Data structures.**

- Vec<Expr> sequences; Vec<Int> for ntt/fwht/convolution_int.

**Hard dependencies.**

- core Expr (sympify, expand_mul, pi, I, Rational, Add/Mul canonical form).
- functions.elementary trig (exact cos/sin at rational multiples of pi) and evalf.
- ntheory.isprime/primitive_root (ntt).

**Python features that are hard to port.**

- sympify of arbitrary sequences; isinstance-based dispatch between int and Rational.
- Bit-reversal through string formatting (ibin(i, b, str=True)[::-1]).

**Behaviour a port must reproduce.**

- Outputs are exact Expr lists compared structurally, e.g. fft(range(6))[1] == -7*sqrt(2)/2 - 4 - sqrt(2)*I/2 + 2*I. Matching depends on core trig special values and expand_mul canonical form.
- ifft divides by n and applies evalf(dps) only when dps is given. Padding with S.Zero changes output length.
- Errors: TypeError for non-iterables, ValueError for symbolic entries in fft, and errors for a prime not suitable for the NTT length.
- convolution returns exact Rationals when the inputs are rational (via integer packing), so it must not detour through floats.

**Rust prior art.**

- rustfft (MIT OR Apache-2.0): not applicable, it is floating point only.
- An NTT is a few lines over the project's own modular arithmetic; no crate needed.

**How its tests check results.** 14 test functions: 158 asserts, 40 raises(). List equality of Expr/ints, transform/inverse round trips, cross-checks between convolution methods.

**Notes.** Only consumers: series.formal (convolution) and Matrix powers with n > 10000 (linrec_coeffs). Port after core and trig exist.

### `crypto`

3,403 source lines, 51 test functions. Difficulty **M**. Strategy: **port faithfully**. App relevance: none.

Educational classical and public-key cryptography (3,403 lines in crypto.py): shift/affine/atbash/substitution/Vigenère/Hill/bifid (5x5, 6x6)/rail-fence ciphers, RSA (multi-prime, CRT decryption, kid RSA), Morse code, LFSR sequences with autocorrelation and the Berlekamp-Massey connection polynomial, ElGamal, Diffie-Hellman, Goldwasser-Micali and Blum-Goldwasser.

**Why this strategy.** Simple deterministic string and integer code with exact expected outputs. A port reproduces it directly once ntheory, Matrix and FF exist. RustCrypto crates are the wrong fit: these are textbook variants with SymPy-specific conventions, not secure primitives.

**Key algorithms.**

- Text preparation (check_and_join, _prep, padded_key, translate-based substitution).
- encipher_hill/decipher_hill: key matrix times plaintext blocks mod N, inverse via Matrix.inv_mod.
- rsa_private_key via totient or reduced totient of the multi-prime modulus. _decipher_rsa_crt uses ntheory.modular.crt.
- lfsr_connection_polynomial: Berlekamp-Massey over FF(p), returning a Poly in x.
- Key generation for ElGamal, DH, GM and BG via isprime/nextprime/primitive_root and seeded _randrange/_randint.

**Data structures.**

- Strings and char tables (alphabet symbols, bifid squares).
- Tuples of keys (n, e) and (p, r, e); Vec<ModInt> for LFSR states.

**Hard dependencies.**

- ntheory (isprime, nextprime, primitive_root, factorint, crt, totient via functions.combinatorial.numbers).
- sympy.matrices Matrix (Hill cipher, bifid squares).
- polys FF and Poly (LFSR).
- sympy.core.random (_randrange and _randint with int or list seeds).

**Python features that are hard to port.**

- str.translate and maketrans tables; itertools.cycle; uniq/multiset helpers.
- Seed parameters accepting int, None or a list via _randrange/_randint.
- doctest_depends_on decorators.

**Behaviour a port must reproduce.**

- Seeded key generation: seeds may be ints (CPython MT19937 sequence) or lists (deterministic values in order). The values must match the oracle exactly.
- String normalization: uppercasing, stripping non-alphabet characters, the default symbols set (AZ()), and padding behaviour of the bifid and Hill ciphers.
- rsa_private_key returns False for invalid parameters rather than raising, and warns for non-squarefree or duplicate primes (tests use warns).
- lfsr_sequence returns lists of FF(p) elements. lfsr_connection_polynomial returns a Poly with domain-specific printing.

**Rust prior art.**

- None needed. Do not substitute RustCrypto, because it does not reproduce these conventions.

**How its tests check results.** 51 test functions: 164 asserts, 21 raises(). String equality for ciphers, tuple and int equality for keys, list equality for LFSR outputs, and raises/warns checks.

**Notes.** For the project's 'more secure' goal: document that these are educational, not secure, primitives. When seed=None, drawing the seed from OS entropy is a safe improvement with no effect on the tests. Port last.

## Functions, sums and products

### Suggested port order inside the cluster

1. 1. functions protocol: hook trait, head registry, the auto-eval and auto-evalf rule, rewrite/expand dispatch. Co-design with core/function.py, because every family plugs into it.
1. 2. functions.elementary kernel: complexes (re, im, Abs, sign, arg, conjugate), exponential (exp, log), integers (floor, ceiling), miscellaneous (sqrt/root builders, Min/Max). core/power.py, mul.py and sets cannot work without them.
1. 3. functions.combinatorial exact numbers: factorial, binomial, rf/ff, bernoulli, harmonic, euler, fibonacci. Small and exact, with trivial oracle comparison; trig taylor terms and Faulhaber sums need them.
1. 4. Minimal functions.special subset: chebyshevt/u (cos.eval at multiples of pi/5, expand_trig), KroneckerDelta, Heaviside/DiracDelta, gamma. This unblocks elementary, simplify gates and concrete.
1. 5. functions.elementary trig, inverse trig and hyperbolic eval with their special-value tables. This is the app's serve path: special angles at parse time, asin/acos/atan endpoints, expand_trig.
1. 6. Piecewise (collapse, piecewise_fold) together with logic and relational canonicalisation. Piecewise integration waits for the integrals milestone.
1. 7. concrete containers: ExprWithLimits/AddWithLimits/ExprWithIntLimits, Sum/Product objects, the Karr convention, direct summation. Integral inherits from these.
1. 8. Series hooks across elementary and special (taylor_term, _eval_nseries, _eval_as_leading_term, _eval_aseries, tractable rewrites), developed jointly with the series/gruntz unit.
1. 9. concrete summation algorithms, after polys (apart, resultant, factor), solve and simplify: Faulhaber, geometric, telescoping, Gosper and delta sums first, then hyper and residue sums and is_convergent.
1. 10. The remaining symbolic rules of functions.special (bessel, error functions, zeta, hyper/meijerg with HyperRep for hyperexpand and meijerint, elliptic, Ynm, bsplines, mathieu), in step with the numeric crate that ports mpmath.
1. 11. holonomic last, or keep it in Python on the Rust core.

### Risks

- Numerics are the long pole. About 75 mpmath kernels are needed, from elementary functions through hyp2f1, meijerg, lerchphi and besseljzero, all with complex arguments and arbitrary precision. Tests assert str(evalf(n=10)) strings and exact Float equality (L.n() == E.n()), so results must round as mpmath does. A faithful port of mpmath 1.3.0 (BSD-3) is safer than adapting astro-float.
- Exact shapes are the oracle. Tests assert structural == on unexpanded, unsimplified forms (gosper_sum(k,(k,0,n)) == n*(1 + n)/2; digamma(1/3) with an unsimplified log(sqrt(3))). Special-value tables are hash lookups on canonical radicals. Any difference in core canonicalisation (Add/Mul ordering, sqrt normal form) silently turns table hits into misses or changes output trees.
- Eval paths call heavy subsystems:
- exp.eval -> simplify.logcombine;
- Abs.eval -> simplify.signsimp;
- log.eval -> ntheory.multiplicity, simplify.ratsimp and polys.cancel;
- cos.eval -> special.chebyshevt -> polys.orthopolys;
- floor.eval -> adaptive-precision evalf.
These cycles force early ports of those helpers or temporary PyO3 callbacks during the transition.
- About 243 _eval_is_* handlers, and eval branches gated on three-valued facts (e.g. pi_coeff.is_even is False). The Rust assumption engine must give identical fuzzy answers, or evaluation diverges.
- The 102 series hooks and 57 leading-term hooks call limit() and Order, while series/gruntz calls back into them and into 20 tractable rewrites. The functions and series units cannot be validated independently.
- Python features to replace: runtime class creation (Function('f')), the ExpMeta isinstance hack, getattr string dispatch for about 440 rewrite methods, @cacheit memoisation, class-level mutable caches (bernoulli._cache, recurrence_memo), the global evaluate flag, and multipledispatch on floor/ceiling/frac comparisons.
- Hotspots with exponential-size outputs or slow paths: cos(pi/257).rewrite(sqrt) builds 16k ops in about 21 s; test_binomial_Mod_slow takes about 27 s; Sum.is_convergent tests about 25 s; meijerg evalf tests about 19 s. These need budgets or timeouts in the Rust harness.
- Breadth: about 37k source lines, about 200 function classes, 757 test functions (about 4.7 min of Python run time) and about 2.2k doctest lines, which are a second exact-string oracle.

### What other clusters and the core design must know

- Core cannot be completed without a kernel of functions.elementary. core/power.py, mul.py, add.py, expr.py and numbers.py have about 82 lazy imports of exp, log, re, im, Abs, sign, arg, floor, ceiling, conjugate, sin, cos, atan2, Piecewise, factorial, binomial, ff, totient and polygamma. Pow._eval_power, for example, uses floor, arg, exp and log. Plan that kernel inside the core milestone.
- core/evalf.py handles exp, sin, cos, tan, log, atan, Abs, re, im, floor, ceiling, Piecewise, Sum, Product and Integral itself. Every other function goes through Function._eval_evalf, which looks up an mpmath function by class name via MPMATH_TRANSLATIONS (utilities/lambdify.py:70). mpmath is not one of the 70 SymPy planning units, so it needs its own planning unit and crate (XL). The existing rsympy-num crate is scoped to 'exact numbers ... arbitrary-precision floats' and should not absorb the special-function kernels.
- The canonicalisation layer must reproduce the auto-evalf rule: a function evaluates numerically only when ALL its arguments are Floats or pure-complex Float Adds (Function._should_evalf). gamma(2.5) becomes a Float, but besselj(0, 1.0) and polylog(2, 0.5) stay symbolic.
- integrals.Integral subclasses concrete.expr_with_limits.AddWithLimits (integrals/integrals.py:35), so integrals depends on concrete's limit processing, change_index and bound-variable subs. Port that part of concrete before integrals.
- series/limits (gruntz) depend on 20 _eval_rewrite_as_tractable plus intractable rewrites and on the function series hooks. Those hooks call limit() themselves (exp._eval_nseries does). The two units must be co-developed against the oracle.
- sets (Interval, Union) depend on Min/Max from functions.elementary.miscellaneous (MinMaxBase, LatticeOp).
- simplify() unconditionally calls hyperexpand(expr). It gates trigsimp, combsimp, besselsimp, kroneckersimp and piecewise_fold on isinstance checks against this cluster's classes (TrigonometricFunction, HyperbolicFunction, CombinatorialFunction, gamma, BesselBase, KroneckerDelta, Piecewise, ExpBase). Those class markers must exist before the families are fully ported.
- solveset uses f.inverse() (24 definitions here), and calculus.util.periodicity uses f.period(symbol). The app roadmap's trig-inequality intervals and continuous_domain depend on these hooks.
- integrals.meijerint and simplify.hyperexpand use the meijerg/hyper parameter API (.an, .bm, .aother, .bother, .argument, .delta, .nu, get_period; about 140 uses) and the HyperRep_* classes. Port special.hyper early as a class layer, without numerics.
- The app subclasses sympy.Function for custom log printing, and sympy's own test_interface.py checks series and limit of user subclasses. The PyO3 layer needs 'foreign function heads' that call Python eval/fdiff from inside Rust series, limit and diff.
- log(x, b) auto-evaluates to log(x)/log(b); no node carries the base. This bears on the app's \log_b printing and on any evaluate=False parse path. The app's other wish, keeping trig at special angles from evaluating during parse, needs the raw non-evaluating constructor to cover function application, i.e. skip eval().
- holonomic is the only user of polys' old_poly_ring/DMF API outside polys.agca. Deferring holonomic lets polys defer old_polynomialring.py and old_fractionfield.py (except for agca).
- concrete outputs depend on polys output shapes: gosper_sum ends with factor(result); eval_sum_symbolic calls apart() and factor(); Product uses roots(). polys must be faithful in form, not just correct.
- The oracle venv runs GROUND_TYPES=python (no gmpy2, no python-flint) with mpmath 1.3.0. The oracle therefore takes the pure-Python paths (e.g. factorial._recursive/_swing rather than gmpy.fac).
- pytest is not installed in the oracle venv, and sympy.test() in 1.14 refuses to run without it. I timed this cluster's tests with a minimal runner (a small per-function runner, not committed). All modules pass; total about 284 s of Python time.
- Licence note for the bigint choice: malachite (which has fast factorial and binomial) is LGPL-3.0, so the settled licence decision excludes it. Use dashu, ibig or num-bigint and port SymPy's prime-swing factorial and binomial.

### `functions`

115 source lines, 0 test functions. Difficulty **M**. Strategy: **redesign**. App relevance: serve.

Package facade (functions/__init__.py, 115 lines) that re-exports about 190 function classes from the three subpackages, plus `ln = log`. For planning it stands for the shared function-class protocol every family plugs into. That protocol lives in core/function.py (core unit, 3423 lines: Application, Function, DefinedFunction, AppliedUndef, UndefinedFunction, WildFunction) and in the rewrite dispatch in core/basic.py.

**Why this strategy.** Python's metaclass protocol with hooks found by getattr on method names must become a Rust trait: eval, fdiff, rewrite(target enum), expand(hint), nseries, leading_term, evalf and assumption handlers, plus a registry of function heads. Behaviour is ported faithfully; only the dispatch mechanism is redesigned. A PyO3 'foreign head' adapter is needed for Python-defined subclasses.

**Key algorithms.**

- Auto-evaluation: Application.__new__ (core/function.py:295) calls cls.eval(*args) unless evaluate=False. A None result builds the node, and nargs is canonicalised to FiniteSet or Naturals0.
- Auto-evalf: Function._new_/_should_evalf (core/function.py:453-505) calls evalf only when EVERY argument is a Float or a pure-complex Float Add. Precision is the largest argument _prec.
- Numeric fallback: Function._eval_evalf (core/function.py:534) looks up mpmath.<ClassName> or MPMATH_TRANSLATIONS (utilities/lambdify.py:70, e.g. Abs->fabs, elliptic_k->ellipk, assoc_legendre->legenp, dirichlet_eta->altzeta). It converts arguments at prec+5, runs under mpmath.workprec, and otherwise falls back to the _imp_ callable.
- core/evalf.py evalf_table bypasses that lookup for exp, sin, cos, tan, log, atan, Abs, re, im, floor, ceiling, Piecewise, Sum, Product and Integral.
- Chain rule: Function._eval_derivative plus fdiff (core/function.py:600,765). With no fdiff the result is Subs(Derivative(f(_xi_i), _xi_i), _xi_i, arg).
- Generic series: Function._eval_nseries (core/function.py:657) uses taylor_term, or repeated differentiation plus limit(), with _eval_aseries for infinite arguments.
- Rewrite dispatch: Basic.rewrite/_rewrite (core/basic.py:1984-2110) builds the method name '_eval_rewrite_as_' + target. This cluster defines about 440 such methods across about 60 targets, including tractable and intractable.
- Expand dispatch: each expand() hint maps to _eval_expand_<hint> (func: 25 defs; trig, complex, log, power_exp, diracdelta).
- Assumption handlers: about 243 _eval_is_<fact> methods in this cluster, consumed by the core assumption engine.

**Data structures.**

- Function-head registry: name, nargs set, class_key, hook table
- Applied-function node: head plus Arc'd args
- Rewrite-target enum (about 60 targets)
- Expand-hint flags
- Name-to-numeric-kernel table (today MPMATH_TRANSLATIONS)

**Hard dependencies.**

- core
- utilities

**Python features that are hard to port.**

- FunctionClass metaclass with an nargs property
- UndefinedFunction creates classes at runtime (Function('f', real=True)), equal by name and assumptions
- ExpMeta isinstance hack
- String dispatch via getattr('_eval_rewrite_as_%s') and '_eval_expand_%s'
- @cacheit on __new__ and on taylor_term
- Thread-global global_parameters.evaluate / exp_is_pow
- _imp_ attribute from implemented_function

**Behaviour a port must reproduce.**

- Auto-evalf rule: gamma(2.5), zeta(2.0), erf(0.5) and LambertW(1.0) become Floats, but besselj(0, 1.0), polylog(2, 0.5), polygamma(1, 2.5), expint(1, 2.0) and hyper([1],[2],0.5) stay symbolic, because one Integer or Tuple argument blocks it.
- Classes with no mpmath mapping never evaluate numerically: erf2, erfcinv, erf2inv, mathieus/c, marcumq, KroneckerDelta. For example, erfcinv(1/2).evalf() returns erfcinv(1/2).
- With evaluate=False, eval is skipped but nargs is still canonicalised.
- fdiff of an undefined head yields Subs(Derivative(f(_xi_1), _xi_1), _xi_1, x**2). The dummy is named xi_<argindex> with dummy_index=hash(arg), and the name appears in printed test output.
- isinstance(E**x, exp) is True via ExpMeta.__instancecheck__; exp(x) is its own node unless global_parameters.exp_is_pow is set.
- The Function.class_key table (exp=10, log=11, sin=20 ... arg=43, unknown 0 or 10000) feeds canonical ordering of arguments.
- Python subclasses must get Rust series and limit behaviour by calling back into their Python eval/fdiff (functions/elementary/tests/test_interface.py). The app also subclasses Function for its log printing.

**Rust prior art.**

- SymEngine (C++, MIT): Function/FunctionSymbol design, reference only; it bakes in auto-evaluation, which the plan avoids
- Symbolica (Rust, source-available, not under a permissive licence): reference only

**How its tests check results.** The facade has no tests of its own. The protocol is exercised by every function test, plus test_interface.py (series and limit of user subclasses). Tests check structural == after auto-evaluation, identity of singletons (`is S.Zero`, `is nan`) and three-valued facts (`.is_real is None`).

**Notes.** The real protocol code is core/function.py in the core unit. Recommendation: this cluster co-owns the hook-trait definition with core, and the name-to-numeric-kernel table moves out of lambdify into the numeric crate.

### `functions.elementary`

12,105 source lines, 358 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: serve.

Elementary functions, 12.1k lines and 60 classes:
- trigonometric.py: sin, cos, tan, cot, sec, csc, sinc, the inverse trig functions and atan2, plus the Fermat-prime radicals in _trigonometric_special.py.
- hyperbolic.py: 12 hyperbolic and inverse hyperbolic functions.
- exponential.py: exp, exp_polar, log, LambertW.
- complexes.py: re, im, sign, Abs, arg, conjugate, polar_lift, periodic_argument, principal_branch, transpose, adjoint, polarify/unpolarify.
- integers.py: floor, ceiling, frac.
- miscellaneous.py: sqrt, root, cbrt and real_root as Pow builders, Min/Max, Rem, Id.
- piecewise.py: Piecewise, piecewise_fold, piecewise_simplify, piecewise_exclusive and Piecewise integration.

**Why this strategy.** Each eval method is a dense case analysis whose exact output tree is the oracle. Only a faithful translation that keeps branch order and table keys reproduces the expectations. The lookup tables should be built lazily at runtime with the Rust canonicaliser, not hard-coded as trees.

**Key algorithms.**

- _pi_coeff/_peeloff_pi (trigonometric.py:131,168): normalise rational multiples of pi, recast binary-fraction Floats as Rationals, and decide parity through assumptions.
- cos.eval (trigonometric.py:607-703): sign and period reduction; _table2 products for q in {12,15,20,24,30,40,60,120}; q=3 and q=5 via chebyshevt(p, c).expand(); half-angle recursion for even q<=12. sin.eval converts to cos.
- cos.rewrite(sqrt): Gauss-period radicals for the Fermat primes 3, 5, 17 and 257 (_trigonometric_special.py: migcdex, ipartfrac, fermat_coords, cos_257).
- Inverse-trig lookup tables keyed by canonical radicals (_asin_table, _atan_table, _acsc_table at trigonometric.py:2035+), and log's _log_atan_table.
- exp.eval (exponential.py:276): reduces exp(k*pi*I), extracts a single log factor via simplify.logcombine, and splits Adds.
- log.eval (exponential.py:661): base handling via ntheory multiplicity(), log(-x) -> I*pi + log(x), and logs of Gaussian algebraic numbers via the atan table plus ratsimp.
- Abs.eval (complexes.py ~470): signsimp, splitting of Muls, rules for Pow, then sqrt(expand_mul(arg*conjugate)).
- RoundFunction.eval (integers.py): splits the argument into integer, numeric and symbolic parts; exact constants go through core.evalf.get_integer_part with adaptive precision.
- MinMaxBase._collapse_arguments/_find_localzeros/_is_connected (miscellaneous.py:411-620): lattice reduction using is_ge comparisons.
- _piecewise_collapse_arguments (piecewise.py:1245): canonicalises conditions and removes repeated or negated ones.
- piecewise_fold.
- Piecewise._intervals/_eval_integral/_eval_interval (piecewise.py:365-750): piecewise antiderivatives with Min/Max clipping, using solvers.inequalities.
- taylor_term recurrences: factorial for sin/cos, bernoulli for tan/cot, euler for sec/sech, RisingFactorial for asin.
- 102 series hooks (_eval_nseries, _eval_aseries, taylor_term) and 32 _eval_as_leading_term.
- expand_trig: sin(n*x) via chebyshevt/chebyshevu, and the tan(x+y) addition formula.

**Data structures.**

- Lazy special-value tables (HashMap<Expr, Expr>) for asin, atan, acsc and log-atan, plus the cached Fermat-prime cos radicals
- Ordered ExprCondPair list for Piecewise
- Argument set of the Min/Max lattice operation
- Interplay with AccumBounds and SetExpr arguments

**Hard dependencies.**

- core
- logic
- polys
- simplify
- ntheory
- functions.combinatorial
- functions.special
- calculus
- sets
- series
- solvers

**Python features that are hard to port.**

- @cacheit tables (_table2, _asin_table, cos_257)
- multipledispatch on (floor, Expr) etc. for _eval_is_ge/_eval_is_lt (integers.py:291,491,699)
- ExpMeta isinstance hack
- IdentityFunction as a Singleton Lambda
- LatticeOp's ShortCircuit exception used as control flow
- hasattr(arg, '_eval_Abs') duck typing
- A MatrixBase branch inside exp.eval

**Behaviour a port must reproduce.**

- Special angles: cos(pi/7), sin(pi/17) and cos(pi/257) stay unevaluated, but cos(pi/120) auto-expands into a large nested radical. cos(pi/257).rewrite(sqrt) builds a 16,088-op tree in about 21 s.
- Exact output shapes are the oracle: cot(pi/5) -> sqrt(2*sqrt(5)/5 + 1); csc(pi/10) -> 1/(-1/4 + sqrt(5)/4); tan(5*pi/12) -> sqrt(3) + 2; sin(pi/12) -> -sqrt(2)/4 + sqrt(6)/4.
- Table lookups hit only when the radical's canonical form is identical. asin(1/sqrt(2)) hits because 1/sqrt(2) canonicalises to sqrt(2)/2.
- Results depend on assumptions: cos(n*pi) -> (-1)**n for integer n. sin(2*k*pi + x) -> sin(x) requires k even. _pi_coeff returns 0 or 2 depending on whether parity is known.
- Infinities and branches: sin(oo) -> AccumBounds(-1, 1); sin(zoo) -> nan; log(0) -> zoo; log(-8) -> log(8) + I*pi; exp(I*pi/3) and exp(2*I*pi/5) stay unevaluated.
- log(x, 2) returns log(x)/log(2); there is no node that carries the base. log(8, 2) -> 3. log(exp(x)) stays unless x is known real; exp(log(x)) -> x always.
- sqrt, root and cbrt are not classes: srepr(sqrt(x)) == Pow(x, 1/2), and sqrt(8*x**2) -> 2*sqrt(2)*sqrt(x**2).
- Abs(x)**2 stays, but Abs(r)**2 -> r**2 for real r; Abs(I*x) -> Abs(x); re(x + I*y) -> re(x) - im(y).
- Min/Max: Max(1, x, 2) -> Max(2, x); Min(x, x + 1) -> x; Max(x, -x) stays.
- Piecewise((1, x>0), (1, True)) -> 1, but Piecewise((x, x<1), (2, x<0), (3, True)) keeps the unreachable branch. Conditions are canonicalised (-x < -1 becomes x > 1).
- Rewrite outputs are specific: tan(x).rewrite(sin) == 2*sin(x)**2/sin(2*x); asinh(1) -> log(1 + sqrt(2)).
- Floats: sin(Float(1,30)) keeps 30 digits; floor(2.5) -> Integer 2.

**Rust prior art.**

- astro-float (MIT): correctly rounded arbitrary-precision exp, ln, sin, cos, tan, asin, acos, atan and sinh..atanh; a possible numeric base layer, but its rounding may not match mpmath bit for bit
- dashu-float (MIT OR Apache-2.0)
- num-complex (MIT OR Apache-2.0)

**How its tests check results.** 358 test functions and about 4.1k asserts (test_trigonometric 1237, test_hyperbolic 813). They check:
- structural == against auto-evaluated expressions, and identity of singletons;
- series(...) == ... + O(x**n), about 300 times;
- rewrite results, about 370 times;
- integrate, 129 calls in test_piecewise;
- a few str/srepr outputs and toleranced N() checks.
Python run time is about 92 s; test_sincos_rewrite_sqrt_257 alone takes 23 s.

**Notes.** Core cannot finish without a kernel of this unit: about 82 lazy imports in core/power.py, mul.py, add.py and expr.py pull in exp, log, re, im, Abs, sign, arg, floor, ceiling, conjugate, sin, cos, atan2 and Piecewise. Narrow dependencies: solvers (Piecewise._intervals uses _solve_inequality), series (Order/limit, series hooks only), calculus (AccumBounds), sets (SetExpr branches). Piecewise integration (about 400 lines) can wait for the integrals milestone. App use: special-angle evaluation during parse, asin/acos/atan endpoints, expand_trig/trigsimp, and log printing.

### `functions.combinatorial`

4,330 source lines, 56 test functions. Difficulty **M**. Strategy: **port faithfully**. App relevance: roadmap.

factorials.py: factorial, factorial2, subfactorial, RisingFactorial/rf, FallingFactorial/ff, binomial.
numbers.py has 25 function classes:
- sequences: fibonacci, lucas, tribonacci, bernoulli, bell, harmonic, euler, catalan, genocchi, andre, partition, motzkin;
- number-theory wrappers: carmichael, divisor_sigma, udivisor_sigma, legendre/jacobi/kronecker symbols, mobius, primenu, primeomega, totient, reduced_totient, primepi.
numbers.py also has the counting helpers nP, nC, nT, stirling and nD.

**Why this strategy.** Exact integer and rational algorithms translate directly and gain the most from Rust bigints. Outputs are exact, so comparing with the oracle is trivial.

**Key algorithms.**

- factorial._swing/_recursive prime-swing algorithm (factorials.py:92-136) over sieve.primerange. gmpy.fac is used only when gmpy is present, and the oracle venv has none.
- factorial._facmod/_eval_Mod (Wilson's theorem) and binomial._eval_Mod (factorials.py:992-1070) via ntheory binomial_mod.
- Exact binomial: binomial._eval, plus _eval_expand_func to a polynomial.
- bernoulli._calc_bernoulli (numbers.py:523-575): Ramanujan recurrence memoised per residue mod 6. For n > 500 it uses mpmath bernfrac. bernoulli(n, x) uses polys.appellseqs.bernoulli_poly.
- fibonacci via mpmath.libmp.ifib (fast doubling); lucas and tribonacci via recurrence_memo and closed forms.
- Exact Euler numbers via mpmath eulernum(exact=True); Euler and Genocchi polynomials via appellseqs.
- Bell numbers, Bell polynomials and incomplete Bell via recurrence_memo; harmonic gives exact rationals, and expand_func applies Gauss's digamma theorem.
- nT/nC/nP/stirling multiset counting via _AOP_product and utilities.enumerative.MultisetPartitionTraverser.
- _eval_evalf of non-integer arguments: mp.bernoulli/bernpoly/zeta, mp.harmonic, mp.eulernum/eulerpoly, mp.dirichlet with sinpi/cospi (andre).

**Data structures.**

- Bigint plus the prime sieve from ntheory
- Memo tables behind OnceLock/Mutex, or recomputation
- Multiset histogram for nP/nC/nT

**Hard dependencies.**

- core
- ntheory
- polys
- functions.elementary
- utilities

**Python features that are hard to port.**

- recurrence_memo decorator with growing class-level lists
- bernoulli._cache dict mutated across calls
- @cacheit on _stirling1, _stirling2, _nT and _AOP_product
- Python int's implicit arbitrary precision everywhere

**Behaviour a port must reproduce.**

- bernoulli(1) == 1/2 and bernoulli(1, x) == x - 1/2 (the convention since SymPy 1.12).
- factorial: factorial(-1) -> zoo; factorial(1/2) stays; factorial(5.0) -> Float 120.000000000000; factorial(4.5) -> a Float via gamma.
- binomial(-1, 3) -> -1 and binomial(1/2, 2) -> -1/8. binomial(n, n) -> 1 only for nonnegative integer n; binomial(x, x) stays.
- harmonic(4) -> 25/12; harmonic(n).rewrite(digamma) -> polygamma(0, n + 1) + EulerGamma; harmonic(-1.0, 1).evalf() is nan.
- genocchi(4) == 1 under SymPy's G_n = 2(1-2^n)B_n convention.
- Exact numeric strings are asserted, e.g. str(bernoulli(I).evalf(n=10)) == '0.9268485643 - 0.5821580598*I'.
- Class-level memo tables (bernoulli._cache/_highest, recurrence_memo lists) must not make results depend on call order.

**Rust prior art.**

- num-bigint (MIT OR Apache-2.0)
- dashu-int (MIT OR Apache-2.0)
- ibig (MIT OR Apache-2.0)
- primal (MIT OR Apache-2.0): prime sieve
- malachite has fast factorial/binomial but is LGPL-3.0, so it is excluded by the licence decision

**How its tests check results.** 56 test functions and about 790 asserts:
- structural == on exact integers, rationals and rewritten forms;
- 85 raises() for invalid arguments;
- 36 exact-string checks of str(...evalf(n=10));
- nC/nT compared exhaustively against enumeration.
test_binomial_Mod_slow takes about 27 s in Python.

**Notes.** Other units need it earlier than app priority suggests:
- trig and hyperbolic taylor_term use factorial, bernoulli, euler and RisingFactorial;
- concrete's Faulhaber sums use bernoulli polynomials and harmonic;
- core/power.py imports totient, factorial, ff and binomial;
- stats (probability roadmap) uses binomial and factorial.
The number-theory wrappers are thin over ntheory. binomial(100000, 50000) takes 0.84 s in pure Python, an easy Rust speedup.

### `functions.special`

12,938 source lines, 151 test functions. Difficulty **XL**. Strategy: **port faithfully**. App relevance: roadmap.

Special functions, 12.9k lines and 105 classes:
- gamma family: gamma, lowergamma, uppergamma, polygamma, loggamma, digamma, trigamma, multigamma; beta, betainc;
- error and integral functions: erf, erfc, erfi, erf2, the three inverses, Ei, expint/E1, li, Li, Si, Ci, Shi, Chi, fresnels/c, and the _erfs/_eis helpers;
- zeta family: zeta, dirichlet_eta, polylog, lerchphi, stieltjes, riemann_xi;
- Bessel: besselj/y/i/k, hankel1/2, jn, yn, hn1, hn2, jn_zeros, the Airy functions and their derivatives, marcumq;
- hyper, meijerg, appellf1 and 12 HyperRep_* classes;
- 13 orthogonal-polynomial classes; Ynm/Znm; four elliptic integrals;
- DiracDelta, Heaviside, SingularityFunction, KroneckerDelta, LeviCivita;
- B-splines; four Mathieu functions.

**Why this strategy.** The symbolic eval, rewrite and series rules are case tables that must reproduce exact shapes. The numeric half cannot be ported inside this unit at all: it needs a separate Rust crate equivalent to mpmath, which is the real long pole.

**Key algorithms.**

- Gamma family (gamma_functions.py:222-560): exact gamma values at integers and half-integers; lowergamma/uppergamma eval to exp/erfc/expint closed forms.
- polygamma.eval applies Gauss's digamma theorem for rationals. Its _eval_evalf needs the Hurwitz-zeta derivative mp.zeta(s+1, z, 1) (gamma_functions.py:851).
- zeta.eval via Bernoulli numbers for even and negative integers, plus Hurwitz at rational shifts; polylog closed forms for s<=0 and the _dilogtable (zeta_functions.py:773).
- Orthogonal polynomials: an explicit degree routes to polys.orthopolys (legendre_poly, chebyshevt_poly, jacobi_poly ...); a symbolic degree uses symmetry rules.
- Bessel: half-integer orders expand to sin/cos via the spherical _jn/_yn; order and argument sign rules; jn_zeros via mpmath besseljzero.
- hyper/meijerg (hyper.py): canonicalises parameter tuples (TupleArg); radius_of_convergence, convergence_statement, get_period, _diff_wrt_parameter; meijerg._eval_evalf wraps mpmath.meijerg with a polar-branch trick (hyper.py:694).
- Series: _eval_aseries asymptotic expansions, plus the tractable rewrites (_erfs, _eis) that gruntz uses.
- DiracDelta and Heaviside: the H0 convention; _eval_expand_diracdelta via polys roots.
- bspline_basis builds Piecewise recursively; interpolating_spline uses linsolve.
- Numeric kernels it needs (about 60): gamma, rgamma, loggamma, digamma, psi, gammainc, beta, betainc, erf/erfc/erfi/erfinv, ei, e1, expint, li, si, ci, shi, chi, fresnel; besselj/y/i/k, hankel1/2, besseljzero, airyai/bi with derivative; hyper (pFq), meijerg, appellf1; ellipk/e/f/pi; zeta (Riemann, Hurwitz and its derivative), altzeta, polylog, lerchphi, stieltjes; legendre, legenp, chebyt/u, gegenbauer, hermite, jacobi, laguerre, spherharm. All need complex arguments.

**Data structures.**

- Parameter tuples (TupleArg) for hyper/meijerg, with an/aother/bm/bother views
- HyperRep piecewise analytic-continuation representations
- Polar numbers (exp_polar) and branch tracking
- Numeric backend: mpf/mpc with a precision field, and hypercomb term lists

**Hard dependencies.**

- core
- functions.elementary
- functions.combinatorial
- polys
- simplify
- series
- concrete

**Python features that are hard to port.**

- TupleArg overrides as_leading_term and limit
- Per-subclass classmethods _expr_small/_expr_big on HyperRep
- @assume_integer_order decorator
- mpmath imported lazily inside _eval_evalf, under workprec context managers
- Numeric kernel chosen by class name via MPMATH_TRANSLATIONS

**Behaviour a port must reproduce.**

- Unsimplified shapes are the oracle: digamma(1/3) -> -log(3) - sqrt(3)*pi/6 - EulerGamma - log(sqrt(3)) and polygamma(0, 1/4) -> -pi/2 - log(4) - log(2) - EulerGamma.
- zeta: zeta(3) stays; zeta(2) -> pi**2/6; zeta(-3, 1/2) -> -7/960; zeta(1) -> zoo.
- polylog: polylog(1, x) stays, but polylog(0, x) -> x/(1 - x).
- Auto-evalf is asymmetric: polylog(2, 0.5), polygamma(1, 2.5), besselj(0, 1.0) and expint(1, 2.0) stay symbolic.
- Li(2.0) -> 1.06164034811860e-16: leftover rounding error from li(2.0) - li(2.0), not 0.
- Heaviside(0) -> 1/2 (the H0 default); DiracDelta(0) stays; DiracDelta(x).diff(x) -> DiracDelta(x, 1).
- Identity tests evaluate at random complex points via verify_numerically/test_derivative_numerically with sympy's seeded RNG. Numerics must handle complex arguments with mpmath's branch cuts.
- Exact strings and exact Float equality: str(polygamma(I, I).evalf(n=10)) == '6.332362889 + 45.92828268*I', and L.n() == E.n() compares 53-bit results exactly.

**Rust prior art.**

- mpmath 1.3.0 (BSD-3-Clause, Python) is the algorithm source to port: libmp/gammazeta.py (2167 lines), libmp/libhyper.py (1150), functions/hypergeometric.py (1413), bessel.py (1108), zeta.py (1154), elliptic.py (1431), expintegrals.py (425), orthogonal.py (493).
- astro-float (MIT): arbitrary-precision elementary layer only.
- statrs (MIT), puruspe and special (verify licences): f64-only gamma/erf/beta/bessel, usable only as fast paths or cross-checks.
- Excluded as LGPL: Arb/FLINT, and MPFR via rug/gmp-mpfr-sys.

**How its tests check results.** 151 test functions and about 2k asserts:
- structural == for eval, rewrite, expand_func and diff;
- 107 series asserts in test_bessel alone;
- about 60 numeric identity checks via verify_numerically/td at random complex points;
- some exact str(evalf(n=10)) strings and exact Float equality.
test_meijerg_eval takes about 19 s and test_hyperrep about 6 s.

**Notes.** Subsets are needed far earlier than the rest:
- KroneckerDelta: concrete delta sums, matrices, kroneckersimp.
- Heaviside and DiracDelta: Max/Min/Abs/sign derivatives and rewrites, and integrals.
- gamma and polygamma: gammasimp/combsimp; core/power.py imports polygamma.
- chebyshevt/chebyshevu: cos.eval at multiples of pi/5, and expand_trig.
- hyper/meijerg plus HyperRep_*: simplify.hyperexpand, and integrals.meijerint (about 140 uses of .an/.bm/.aother/.bother).
matrices and solvers are needed only by bsplines.interpolating_spline and jn_zeros.
Suggested split: a core subset (M); the full symbolic rules (L); and the mpmath-port numerics (XL, its own crate).

### `concrete`

4,251 source lines, 160 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: roadmap.

Base classes ExprWithLimits, AddWithLimits and ExprWithIntLimits, which are also the base of integrals.Integral. Sum and Product objects with doit, evalf hooks, is_convergent, euler_maclaurin, reverse_order and change_index. The summation() and product() algorithms (direct, symbolic, telescoping, Gosper, hypergeometric, residue). KroneckerDelta sums and products (delta.py). Sequence guessing (guess.py).

**Why this strategy.** summation is a cascade of heuristics, and the order of the cascade decides the output form, so only a faithful port reproduces it. ExprWithLimits must be ported early because Integral inherits from it.

**Key algorithms.**

- _process_limits/_common_new (expr_with_limits.py:22-198): normalise limits, handle dummies, compute free_symbols, and do _eval_subs that respects the bound variable.
- Karr convention: when a > b, eval_sum(f, (i, a, b)) evaluates (i, b+1, a-1) with a sign change, so Sum(i,(i,5,1)).doit() == -9 and Product(i,(i,5,1)).doit() == 1/24.
- eval_sum (summations.py:1016): folds Piecewise; sends KroneckerDelta to deltasummation; sums fewer than 100 terms directly; otherwise calls eval_sum_symbolic.
- eval_sum_symbolic (summations.py:1129) cascades through:
- linearity, apart() and telescopic();
- Faulhaber via bernoulli(n+1, b+1), and harmonic for negative powers;
- a geometric pattern via Wild match -> Piecewise((l, Eq(q, 1)), (r, True));
- gosper_sum, then eval_sum_hyper, then eval_sum_residue, then a factor() fallback.
- Gosper (gosper.py): hypersimp ratio; gosper_normal via resultant plus ground_roots; a linear solve for the certificate; final factor().
- eval_sum_hyper builds hyper(...) and runs hyperexpand, with convergence conditions.
- eval_sum_residue: cot/csc residues at polynomial roots.
- Product._eval_product (products.py:291): polynomial roots -> RisingFactorial, conversion between exp and log sums, deltaproduct.
- Sum.is_convergent (summations.py:380-670): ratio, root, integral, limit-comparison, Dirichlet and alternating tests via limit() and is_decreasing.
- Numerics in core/evalf.py: evalf_sum uses hypsum (term ratio plus mpmath nsum Richardson) or Euler-Maclaurin; evalf_prod.
- guess.py: find_simple_recurrence by rational linear algebra, guess_generating_function, and continued-fraction rationalize.

**Data structures.**

- Limits as Vec<(Symbol, Option<Expr>, Option<Expr>)>
- Scoping of bound variables for subs and free_symbols
- Poly over ZZ/QQ for gosper_normal
- Wild patterns (c1**wexp, c2*i + c3)

**Hard dependencies.**

- core
- functions.elementary
- functions.combinatorial
- functions.special
- polys
- simplify
- solvers
- series
- logic
- sets
- calculus

**Python features that are hard to port.**

- Pattern recognition in eval_sum_symbolic via Wild/match
- Exceptions as control flow (PolynomialError, NotImplementedError, ReorderError)
- _dummy_with_inherited_properties_concrete creates Dummies with copied assumptions

**Behaviour a port must reproduce.**

- Geometric sums return Piecewise: summation(r**i, (i,0,n)) -> Piecewise((n + 1, Eq(r, 1)), ((1 - r**(n + 1))/(1 - r), True)), and Sum(x**i,(i,0,oo)).doit() -> Piecewise((1/(1 - x), Abs(x) < 1), (Sum(x**i, (i, 0, oo)), True)).
- summation(binomial(n,i),(i,0,n)) returns a Piecewise containing hyper((1, 1), (n + 2,), -1), not 2**n. The port must not 'improve' this.
- gosper_sum(k, (k, 0, n)) == n*(1 + n)/2 holds structurally (factored, not expanded) because gosper ends with factor().
- summation(i**3,(i,1,n)) -> n**4/4 + n**3/2 + n**2/4 (expanded Faulhaber).
- summation(1/(i**2+1),(i,-oo,oo)) -> pi/tanh(pi).
- Sum(i,(i,1,n)).subs(n,3) stays an unevaluated Sum(i, (i, 1, 3)), and the bound variable must never leak through subs.

**Rust prior art.**

- No Rust prior art known. References: A=B (Petkovsek-Wilf-Zeilberger) for Gosper/Zeilberger; Maxima nusum (GPL, reference only).

**How its tests check results.** 160 test functions and about 970 asserts:
- structural == on closed forms and Piecewise results;
- simplify(result - expected) == 0, 61 calls across test_sums_products and test_gosper;
- some evalf checks.
Python times: test_is_convergent about 25 s; the two residue tests about 10 s each; the whole package about 79 s.

**Notes.** Split into two parts:
- (a) The ExprWithLimits/Sum/Product containers, the Karr convention and direct evaluation (S-M). Needed early by integrals and the printers.
- (b) The summation algorithms (L), after polys (apart, resultant, roots, factor), solvers.solve, simplify (hypersimp, combsimp, hyperexpand) and series (limit, residue).
Faulhaber and geometric sums cover the app roadmap's arithmetic/geometric sequence keys and partial sums; rsolve lives in solvers.

### `holonomic`

3,272 source lines, 32 test functions. Difficulty **L**. Strategy: **defer or drop**. App relevance: none.

Holonomic functions:
- DifferentialOperatorAlgebra and DifferentialOperator: an Ore algebra over the QQ[x] old polynomial ring.
- HolonomicFunction with closure operations (+, *, composition, integrate, diff).
- Conversions: expr_to_holonomic, from_hyper, from_meijerg, to_hyper, to_meijerg, to_expr.
- Power series and recurrences: to_sequence, _frobenius, series; RecurrenceOperator and HolonomicSequence.
- Numerical Euler/RK4 integration (numerical.py).

**Why this strategy.** A leaf: nothing else in SymPy imports it, and the app does not use it. It needs the old poly ring API, DomainMatrix, meijerint and hyperexpand. Keep it as Python on the Rust core via PyO3 until everything below it is ported, then port it faithfully.

**Key algorithms.**

- DifferentialOperator multiplication by the Leibniz rule over DMF coefficients (holonomic.py:226-270).
- Closure under + and * via DomainMatrix nullspace (holonomic.py:548, 904).
- to_sequence/_frobenius (holonomic.py:1183-1570): recurrences for series coefficients, and roots of the indicial equation.
- expr_to_holonomic (holonomic.py:2199): table lookup (_create_table), a recursive build, and the meijerint fallback (_convert_meijerint).
- to_hyper, to_meijerg and to_expr, all via hyperexpand.
- _rk4/_euler (numerical.py): ODE integration on mpmath mpf at mp.prec.

**Data structures.**

- Ore polynomial as a Vec of DMF/PolyElement coefficients
- HolonomicFunction{annihilator, x, x0, y0}
- RecurrenceOperator over QQ[n]

**Hard dependencies.**

- core
- polys
- polys.domains
- polys.matrices
- functions.special
- simplify
- integrals
- solvers
- series

**Python features that are hard to port.**

- __rmul__/__radd__ overloading that mixes sympy expressions with ring elements
- DMF objects from polys.polyclasses
- Mutable y0 lists

**Behaviour a port must reproduce.**

- Tests compare sstr() of RK4/Euler results, so mpf add and mul must round bit-identically at 53 bits.
- Operators print through sstr of old_poly_ring/DMF coefficients, so that representation leaks into expected strings.
- The y0 initial-condition lists and x0 shifts must match exactly (assert sstr(p.y0) == y0).

**Rust prior art.**

- No Rust prior art. ore_algebra (SageMath, GPL) and Koutschan's HolonomicFunctions (Mathematica) are references only.

**How its tests check results.** 32 test functions and about 190 asserts:
- mostly == between HolonomicFunction objects (normalised annihilator plus x0/y0);
- 20 sstr() string comparisons, including numerical ODE output;
- series equality.
About 16 s in Python.

**Notes.** Apart from polys.agca, holonomic is the only user of polys' old_poly_ring/DMF (polys/domains/old_polynomialring.py, old_fractionfield.py). Deferring holonomic lets polys defer those as well.

## Series, limits, integrals, simplification

### Suggested port order inside the cluster

1. 1. Core-hosted pieces, inside the core milestone: series/order.py Order, calculus/accumulationbounds.py AccumBounds, radsimp fraction/numer/denom/collect/collect_const, simplify.signsimp, and powsimp/powdenest. Core's Add.flatten, Pow.__new__, expand_mul and the pattern matcher call them, so they cannot wait.
1. 2. simplify basics: ratsimp, radsimp/rad_rationalize, logcombine/expand_log, combsimp/gammasimp, sqrtdenest, separatevars/posify, then nsimplify (once pslq/identify exist in the float library). They are small and backed by polys, and both limit() and simplify() need them.
1. 3. fu.py TR rules, then trigsimp/futrig (needs strategies.tree.greedy) and exptrigsimp. trigsimp_groebner and ratsimpmodprime come later, once groebner exists.
1. 4. The simplify() pipeline, with the hyperexpand/besselsimp/sum/product/Quantity stages stubbed. This unlocks the app's serve path (simplify(a - b) == 0) with exact SymPy behaviour on its vocabulary.
1. 5. cse (cse_main, cse_opts), at any time after core. It is an independent leaf that codegen and lambdify need.
1. 6. The series protocol (Expr.series/nseries/leadterm in core, plus the 127 _eval_nseries/_eval_as_leading_term and 21 _eval_rewrite_as_tractable methods in core and functions), then limits.py + gruntz.py. Gate on test_nseries, test_series, test_order, test_limits and test_gruntz (about 360 functions).
1. 7. calculus.util/singularities, built together with solvers.solveset and inequalities because they call each other. finite_diff and euler are leaves that can go at any time.
1. 8. The Integral class (AddWithLimits) and the integrate() dispatch skeleton, early, because stats, concrete, evalf and ode build unevaluated Integrals. Then ratint, trigintegrate, deltaintegrate, singularityintegrate and manualintegrate (its step tree is also useful to a homework app).
1. 9. Risch (risch/rde/prde) once polys supports Poly over towers of fraction fields. Then heurisch and heurisch_wrapper (needs solve_lin_sys over parameter rings, root_factors and solve).
1. 10. hyperexpand together with meijerint: meijerint passes every result through hyperexpand, and the hyperexpand tests use meijerg. Then transforms.py (Mellin/Fourier/sine/cosine/Hankel) and laplace.py.
1. 11. The series parts that need integrate or rsolve: fourier.py, formal.py (fps), sequences.py and limitseq.py, plus residues, approximants, aseries, acceleration and kauers.
1. 12. Leaves: intpoly (after geometry), quadrature (after RootOf and arbitrary-precision evalf), and integrals.benchmarks rewritten as criterion benches.

### Risks

- Limit and series results depend on 127 _eval_nseries/_eval_as_leading_term methods spread over core and functions (trigonometric.py alone has 25). The series unit cannot reach conformance on its own, and its L estimate does not include that protocol.
- The oracle is not deterministic: integrate() results with parametric special cases change with PYTHONHASHSEED (the Piecewise branches built by heurisch_wrapper). The harness must pin PYTHONHASHSEED and compare such cases by equivalence (a derivative check), not by structure.
- Outputs hinge on exact core behaviour: default_sort_key/ordered (heurisch's ansatz order, powsimp's base sort, Order's variable sort), count_ops (the measures in simplify and fu), and match()/Wild semantics (meijerint, laplace, manualintegrate, trigintegrate). Porting this cluster before core ordering and matching are exact will produce mass diffs that are hard to trace.
- Units call each other in cycles: limit <-> simplify (nsimplify, ratsimp, powsimp, together), calculus.util <-> solveset, integrate <-> simplify, meijerint <-> hyperexpand <-> transforms, and fps -> integrate/rsolve. The Rust crate graph must allow this, either one crate with modules or function tables bound late.
- Some global mutable state exists for correctness, not only for speed: the manualintegrate _integral_cache recursion guard and _parts_u_cache depth limit, the lazily built meijerint/hyperexpand/trigsimp tables, and @cacheit on gruntz sign/limitinf/mrv_leadterm. Rust needs an explicit per-call context and thread-safe lazy statics.
- A faithful port keeps the inherent blowups: series(exp(sin(x))*log(1+x)/cos(x), x, 0, 16) takes 10.4 s (0.8 s at n=10), several tests take 10-36 s, and heurisch builds dense linear systems. The Rust speedup will come from the core, not from these algorithms, unless a fast series kernel is added that emits identical canonical forms.
- Much of the code is research-grade with thin documentation: Risch after Bronstein, Gruntz, Koepf's formal power series, Roach's hyperexpand, the Saxena/Prudnikov conditions in meijerint, Lazard-Rioboo-Trager, and Fu's trig rules. Divergences need mathematically trained reviewers, and agents will spend most of their time investigating.
- Tests are coarse: test_laplace.py is 4 @slow functions spanning 774 lines, test_risch and test_prde assert on internal Poly and DifferentialExtension objects, and test_failing_integrals has 27 XFAILs. Conformance must be counted per assert, and the internal-API tests need a PyO3 shim for Poly.
- App risk: the grading decision simplify(a - b) == 0 must match SymPy exactly, failures included (sqrt(x**2) - x, Float exponents), or grades change silently. simplify stays the most expensive call in the submit path.

### What other clusters and the core design must know

- core must host Order and AccumBounds. Add.flatten and Mul.flatten absorb O-terms (O(x) + x**2 + 1 -> 1 + O(x)), AssocOp takes order_symbols, and core/numbers.py and core/function.py import Order. AccumBounds is handled in Add/Mul/Pow arithmetic and returned by sin(oo) and by floor/frac.
- Core auto-evaluation calls simplify helpers: Pow.__new__ uses factor_terms + radsimp.fraction (2**(x/log(2)) evaluates to exp(x)), AssocOp._matches_commutative uses radsimp.collect and signsimp (_combine_inverse), Mul._eval_expand_mul and core/function.py use fraction, and Mul/Pow nseries use powsimp/powdenest. fraction, numer, denom, collect, signsimp, powsimp and powdenest belong in the core crate.
- The functions cluster owns the series protocol: 104 of the 127 _eval_nseries/_eval_as_leading_term implementations, and the 21 _eval_rewrite_as_tractable methods that Gruntz needs. Limit and series conformance waits on them.
- The float library that replaces mpmath must include pslq and identify (mpmath/identification.py, 844 lines) for nsimplify. limit() nsimplifies every Float input (limit((1 + 0.5/x)**x, x, oo) == exp(1/2)), so limit depends on them too. The hyperexpand tests also need numeric hyper/meijerg evaluation at random complex points.
- solvers: calculus.util (continuous_domain, function_range, periodicity) and solvers.solveset/inequalities call each other, so plan them as one milestone. ratint_ratpart and heurisch_wrapper call solve(), and fps calls rsolve and linsolve.
- polys must provide for this cluster: resultant(includePRS=True), subresultants, roots(filter='R'), count_roots, root_factors, RootSum/Lambda, apart(full=True), groebner, solve_lin_sys over a PolyRing with parameter domains, construct_domain, itermonomials, and Poly over towers such as ZZ(t0)[t1] for Risch. The risch tests compare Poly objects with their domains.
- The strategies unit is a hard dependency: futrig and fu use strategies.tree.greedy and the core combinators, and manualintegrate uses do_one/switch/condition/null_safe.
- Other clusters need the unevaluated Integral class early: concrete/summations (the integral test, and AddWithLimits), core/evalf (numeric integration), stats crv, solvers.ode and physics. Ship the class well before the integration engines.
- Oracle harness: pin PYTHONHASHSEED, because integrate outputs vary by seed. sympy.test() in 1.14 requires pytest, and test_cse_diff.py imports pytest directly, but the pinned venv has no pytest. Install it, or use a plain per-function runner (the one used here ran 1,331 functions in the cluster with no failures outside XFAIL).
- printing: srepr(-x**3/6) prints Mul(Integer(-1), Rational(1, 6), Pow(...)) although the Mul's args are (Rational(-1, 6), x**3). This matters for the app's srepr-based radical-form spike and for the printing port.
- concrete: Sum convergence (is_convergent) relies on series.limitseq.limit_seq, calculus.singularities.is_decreasing and series.residues, and summation uses hypersimp, hyperexpand and combsimp.
- App: calculus.singularities is used at compile time today (figure lint). continuous_domain is available only from sympy.calculus.util. It returns unions of open intervals that exclude removable holes, which is what the roadmap's function-equivalence option needs, and it raises NotImplementedError for any function outside a fixed whitelist. singularities defaults to S.Complexes for non-real symbols.

### `series`

6,329 source lines, 489 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: roadmap.

Limits (limit/Limit: heuristics in front of the Gruntz MRV algorithm), big-O Order objects, the glue behind Expr.series/nseries, formal power series (fps), sequences (SeqFormula/SeqPer/RecursiveSeq), Fourier series, limit_seq/difference_delta, residue, Pade approximants, Richardson/Shanks acceleration, aseries helpers and Kauers finite differences.

**Why this strategy.** Tests check limit, Order and series outputs by structural equality. Other units consume the exact limits: Sum convergence, solveset via function_range, and the endpoints of definite integrals. Gruntz plus limits is about 1100 compact lines once the nseries protocol exists. A Rust truncated-power-series kernel for Taylor cases is allowed only as an internal fast path that emits the same canonical Add.

**Key algorithms.**

- Gruntz MRV limit algorithm in series/gruntz.py: mrv(), mrv_max1/mrv_max3, compare(), sign(), limitinf(), mrv_leadterm(), rewrite() with SubsSet (a dict subclass that creates Dummies on lookup), build_expression_tree, and moveup2 when x is in Omega
- limit() front end in series/limits.py Limit.doit: '+-' two-sided check, substitution at oo, nsimplify of Floats, set_signs for Abs/sign/arg, an is_meromorphic+leadterm fast path, leadterm on the shifted expression, pow_heuristics, gruntz, then term-wise heuristics() falling back to factor_terms/together/factor/ratsimp; factorial is rewritten to gamma
- Order in series/order.py: Order.__new__ (shift the point to 0, extract_leading_order/as_leading_term loop, expand_power_base/expand_log, combining powers of x), contains() (@cacheit, ratio/limit test), _eval_power, as_expr_variables. The absorption of O-terms itself happens in core Add.flatten
- The series driver lives in core/expr.py (series, nseries, lseries, aseries, leadterm, as_leading_term). It retries _eval_nseries up to 8 times to reach the requested order, then returns collect(s1, x) + O
- Formal power series (Koepf's algorithm) in series/formal.py: rational_algorithm (apart), simpleDE/exp_re/hyper_re (DE to recurrence), rsolve_hypergeometric, _solve_explike_DE, solve_de, hyper_algorithm, compute_fps; FormalPowerSeries product/compose/inverse via discrete.convolution and Bell polynomials
- sequences.py: SeqFormula/SeqPer/SeqAdd/SeqMul/RecursiveSeq and find_linear_recurrence (Matrix LUsolve + simplify)
- fourier.py: fourier_series built from integrate() of cos/sin terms; finite_check uses fu TR2/TR1/sincos_to_sum; sigma_approximation
- limitseq.py limit_seq (dominance via difference_delta, uses Sum); approximants.py (Pade via continued fractions); residues.py (the 1/x series coefficient); acceleration.py (richardson, shanks); aseries.py; kauers.py

**Hard dependencies.**

- core
- functions.elementary
- functions.special
- functions.combinatorial
- polys
- simplify
- calculus
- sets
- solvers
- integrals
- concrete
- matrices

**Python features that are hard to port.**

- SubsSet(dict) whose __getitem__ creates a Dummy on demand
- @cacheit, @debug and @timeit decorators on the gruntz internals
- rewrite('tractable'/'intractable') dispatches by name to _eval_rewrite_as_* methods
- getattr(w, 'normal', lambda: w)() duck typing in rewrite()
- Exceptions (PoleError, ValueError, NotImplementedError) used for control flow in Limit.doit
- Generators for lseries; SeriesBase/SeqBase operator overloading with call_highest_priority

**Behaviour a port must reproduce.**

- The default is dir='+'. When z0=oo the direction becomes '-'. dir='+-' raises ValueError when the two sides differ, but returns zoo when both are infinite: limit(1/x, x, 0, '+-') == zoo
- limit nsimplifies any Float before it computes, so limit((1 + 0.5/x)**x, x, oo) == exp(1/2), an exact result from a float input
- Oscillating limits return AccumBounds instead of failing: limit(sin(1/x), x, 0) == AccumBounds(-1, 1). limit(sin(x)/x, x, oo) == 0 is reached through the AccumBounds heuristics
- When gruntz raises PoleError/ValueError and heuristics() returns None, limit returns the unevaluated Limit (self) rather than raising
- The gruntz result goes through rewrite('intractable'). limitinf swaps x for a positive Dummy when x is not known positive, or is an integer
- Order absorption: O(x) + x**2 + 1 -> 1 + O(x); O(x**2) + x**3 -> O(x**2); O(1/x, (x, oo)) + 1/x**2 -> O(1/x, (x, oo)). Order about x0 != 0 keeps the shifted form O((x - pi)**3, (x, pi)). Order variables are sorted by default_sort_key
- series n is the exponent of the O-term, not a count of terms: series(1/(x*(1-x)), x, 0, 3) == 1/x + 1 + x + x**2 + O(x**3). Exact results may have no O-term at all: (1/x).series(x) == 1/x and series(log(x), x, 0, 3) == log(x)
- Log factors appear in terms and in O: series(x**x, x, 0, 3) == 1 + x*log(x) + x**2*log(x)**2/2 + O(x**3*log(x)**3)
- series(n=None) returns a generator (lseries). series at oo substitutes x -> 1/x and falls back to aseries on PoleError
- fps returns a FormalPowerSeries whose ak.formula holds a fresh Dummy k (fps(log(1+x)).ak.formula == -1/((-1)**_k*_k)), so tests compare through truncate()
- gruntz.sign, limitinf and mrv_leadterm are @cacheit. SubsSet makes fresh Dummies on every call, so the mrv tests compare through a helper (mmrv) that strips them

**Rust prior art.**

- SymEngine series module (C++, MIT): a design reference for a truncated univariate/Puiseux series kernel
- Diofant (Python, BSD-3): reworked Gruntz limits, as a reference only (its outputs differ from SymPy's)
- Symbolica (Rust) has series expansion but is source-available, not open source, so it cannot be used
- No Rust crate implements Gruntz or Koepf formal power series

**How its tests check results.** Structural == against expressions that contain explicit O() terms (sin(x).series(x, 0, 4) == x - x**3/6 + O(x**4)); identity checks such as `is oo` and `is S.NaN`; raises(PoleError/ValueError/NotImplementedError); fps checked through .truncate(); test_gruntz has @slow tests behind runslow, and tests of the internal mrv/rewrite. I ran them with a plain per-function runner on 1.14 (pytest is not installed): 489 test functions, 470 pass, 5 @slow skipped, about 14 XFAIL, about 235 s. The slowest limit tests take 13-16 s each (test_issue_16722, test_issue_17431, test_issue_10801).

**Notes.** Order (order.py, 522 lines) must live in the core crate. Most of the real cost of limits sits outside this unit: there are 127 _eval_nseries/_eval_as_leading_term implementations (104 in functions.*, about 18 in core; trigonometric.py alone has 25) and 21 _eval_rewrite_as_tractable methods, and Gruntz needs all of them. On performance: series(exp(sin(x))*log(1+x)/cos(x), x, 0, 16) takes 10.4 s in SymPy 1.14 (0.8 s at n=10). Expression-tree series blows up, and a faithful port keeps that unless a coefficient-ring kernel takes over. Sub-unit sizes: formal.py 1863 lines, sequences.py 1239, fourier.py 811, gruntz.py 701, order.py 522, limits.py 394, limitseq.py 257. formal, fourier and limitseq depend on integrate, rsolve/linsolve and Sum respectively, so they can be ported late. App: limit is on the roadmap for compile-time figure lint (end behaviour), and Sum/summation goes through limit_seq.

### `calculus`

2,714 source lines, 47 test functions. Difficulty **M**. Strategy: **port faithfully**. App relevance: compile.

AccumBounds interval objects; singularities and monotonicity tests; continuous_domain, function_range, periodicity, not_empty_in, stationary points, maximum/minimum and is_convex; finite-difference weights; Euler-Lagrange equations.

**Why this strategy.** util.py is a thin layer over solveset, sets and limit, and its exact Set outputs are tested. AccumBounds has to be exact because core arithmetic and function evaluation produce it.

**Key algorithms.**

- AccumulationBounds (accumulationbounds.py, 804 lines): interval arithmetic for + * / ** with oo, plus comparison handlers registered through multipledispatch. sin(oo), floor and others produce these objects
- singularities.py singularities(): rewrites sec/csc/cot/tan to cos (and sech/csch/coth/tanh to cosh), runs solveset on the bases of negative powers and on the arguments of log/asech/acsch, and on atanh/acoth arguments at +-1
- singularities.py monotonicity_helper with is_increasing/is_decreasing/is_strictly_*/is_monotonic: solveset of the derivative predicate over the interval
- util.py continuous_domain(): a fixed whitelist `implemented` (Pow, exp, log, Abs, frac, trig, hyperbolic and their inverses); domain constraints through solve_univariate_inequality; the acot jump at 0; frac jumps through function_range; the result is the domain minus singularities()
- util.py function_range(): reduces by periodicity, finds critical points with solveset of diff, takes limits at interval ends and at singularities
- util.py periodicity()/_periodicity/lcim: per-function period rules, decompogen/compogen, Mod and Abs handling
- util.py stationary_points, maximum, minimum, is_convex (hessian when multivariate), not_empty_in
- finite_diff.py finite_diff_weights (Fornberg 1988), apply_finite_diff, differentiate_finite, and _as_finite_diff (behind Derivative.as_finite_difference)
- euler.py euler_equations: diff over combinations_with_replacement

**Hard dependencies.**

- core
- functions.elementary
- sets
- solvers
- series
- polys
- logic
- multipledispatch
- matrices

**Python features that are hard to port.**

- multipledispatch handlers for AccumBounds comparisons (_eval_is_le and friends)
- func-identity checks against whitelist lists; constraint tables keyed by function class, holding relations in a placeholder Symbol('x') and applied with subs
- Exceptions used for control flow (NotImplementedError rewrapped with filldedent)

**Behaviour a port must reproduce.**

- singularities uses S.Reals as the default domain only if symbol.is_real, and S.Complexes otherwise: singularities(1/(x**2+1), x) == {-I, I} for a plain x, and EmptySet for x real
- continuous_domain raises NotImplementedError('Unable to determine the domain of the given function.') for any function outside its whitelist, gamma for example. It is not exported from top-level sympy, only from sympy.calculus.util
- continuous_domain((x**2-1)/(x-1), x, S.Reals) == Union(Interval.open(-oo, 1), Interval.open(1, oo)): removable holes are excluded
- continuous_domain(acot(x), x, S.Reals) excludes 0; continuous_domain(1/x + sqrt(x), x, Interval(-5, 5)) == Interval.Lopen(0, 5)
- singularities(tan(x), x) returns a Union of ImageSets with Lambda(_n, 2*_n*pi + pi/2) dummies
- 1/AccumBounds(-1, 2) == AccumBounds(-oo, oo). AccumBounds comparisons return True/False/None through multipledispatch, and core Add/Mul/Pow invoke AccumBounds arithmetic
- periodicity(sin(2*x) + cos(3*x), x) == 2*pi (the lcim of the periods) and returns None for aperiodic input; function_range(1/x, x, S.Reals) == Union(Interval.open(-oo, 0), Interval.open(0, oo))

**Rust prior art.**

- inari and similar Rust interval crates do floating-point interval arithmetic only; they cannot replace symbolic AccumBounds
- Fornberg weights are textbook material; no crate is needed

**How its tests check results.** Structural == on Sets (Interval, Union, FiniteSet, ImageSet) and on AccumBounds; raises(NotImplementedError) for unsupported functions; finite-difference weights compared as lists of exact rationals. Measured: 47 test functions, 42 pass, 1 @slow, 4 XFAIL, about 20 s in total, of which test_periodicity takes 7.6 s.

**Notes.** AccumBounds belongs in the core layer: core/add.py, mul.py, power.py and expr.py import it, and so do functions/elementary trigonometric, exponential, integers and hyperbolic. util.py and solvers are mutually recursive: solveset calls continuous_domain (solveset.py:1153), function_range (:1201) and periodicity (:2637), and inequalities.py calls them too, while they call solveset and solve_univariate_inequality. Build them together. finite_diff and euler are leaves. App: figure lint already uses singularities at compile time. The roadmap moves continuous_domain/singularities into the serve path so that removable holes count for function equivalence.

### `integrals`

17,783 source lines, 486 test functions. Difficulty **XL**. Strategy: **port faithfully**. App relevance: none.

The Integral object and the integrate() front end with its algorithm dispatch, plus every integration engine: rational (ratint), trigonometric (trigintegrate), DiracDelta/SingularityFunction, transcendental Risch, heurisch (Risch-Norman), Meijer G (meijerint), rule-based manualintegrate with steps, the integral transforms (Mellin, Laplace, Fourier, sine/cosine, Hankel), polytope integration, Gaussian quadrature nodes, and line integrals.

**Why this strategy.** The engine order and each engine's internal choices fix the form of the output, and the tests in stats, concrete, ode and physics consume those exact forms. Phase the work: ship the unevaluated Integral class early, port the engines one at a time, and keep the Python meijerint/laplace running on the Rust core during the transition.

**Key algorithms.**

- Dispatch order in integrals.py Integral._eval_integral: Poly.integrate -> Piecewise.piecewise_integrate -> constant -> Poly conversion -> full risch_integrate -> then per Add term: Order handling, (a*x+b)**c, ratint, trigintegrate, deltaintegrate, singularityintegrate, risch again, heurisch_wrapper, meijerint_indefinite, manualintegrate, and finally a retry after sincos_to_sum + expand
- integrals.py Integral.doit: Dummy substitution by the sign of the limits, Abs/sign rewritten to Piecewise, meijerint_definite tried first for infinite limits, the atan/tan branch correction with floor, _eval_interval (limits at the endpoints), and conds='piecewise'|'separate'|'none'
- rationaltools.py ratint: Horowitz-Ostrogradsky in ratint_ratpart (solve for the coefficients) plus Lazard-Rioboo-Trager in ratint_logpart (resultant with includePRS, subresultants), log_to_atan/log_to_real for the real form, and RootSum when the roots are not radicals
- risch.py: the DifferentialExtension tower builder (rewriting exp/log/pow, handle_first), hermite_reduce, polynomial_reduce, residue_reduce, integrate_primitive/hyperexponential/hypertangent and NonElementaryIntegral, following Bronstein's Symbolic Integration I. rde.py rischDE (weak_normalizer, normal_denom, special_denom, bound_degree, spde, no_cancel_*/cancel_*). prde.py: param_rischDE, limited_integrate, is_deriv_k, is_log_deriv_k_t_radical
- heurisch.py, the parallel Risch-Norman ansatz: components(), DiffCache derivations, candidate denominators, an ansatz from itermonomials, root_factors for the log terms, solve_lin_sys over a PolyRing. heurisch_wrapper re-solves at special parameter values (solve(denoms)) and builds a Piecewise
- meijerint.py: _create_lookup_table (a formula -> G-function table keyed by the _mytype class tuple), _rewrite_single/_rewrite1/_rewrite2, the Saxena/Prudnikov convergence conditions _check_antecedents/_check_antecedents_1/_check_antecedents_inversion, _int0oo, meijerint_indefinite/definite/inversion. Every result then goes through hyperexpand
- manualintegrate.py: 60 Rule dataclasses (PartsRule, URule, TrigSubstitutionRule, ...) built by integral_steps through the strategies combinators do_one/switch/condition/null_safe. Includes find_substitutions, parts_rule with a _parts_u_cache depth limit, trig_substitution_rule and sqrt_quadratic_rule; Rule.eval() yields the antiderivative
- trigonometry.py trigintegrate: sin**m*cos**n patterns with the reduction formulas _sin_pow_integrate/_cos_pow_integrate. deltafunctions.py deltaintegrate/change_mul. singularityfunctions.py singularityintegrate
- laplace.py: a rule table from _laplace_build_rules applied with match(); program rules (heaviside, delta, timescale, exp shift, trig, diff, sdiff); _laplace_expand; a fallback to _laplace_transform_integration. inverse_laplace_transform uses its own rule table, apart and _complete_the_square_in_denom; also laplace_correspondence and laplace_initial_conds
- transforms.py: the IntegralTransform base class, _mellin_transform (strip found from conditions), _rewrite_gamma + _inverse_mellin_transform (gamma products to meijerg), and Fourier/sine/cosine/Hankel through integrate
- intpoly.py polytope_integrate (homogeneous decomposition by Chin, Lasserre and Sukumar). quadrature.py gauss_legendre/laguerre/hermite/jacobi/chebyshev/lobatto through RootOf + evalf at n_digits

**Hard dependencies.**

- core
- polys
- polys.domains
- functions.elementary
- functions.special
- simplify
- solvers
- series
- concrete
- strategies
- logic
- sets
- matrices
- geometry

**Python features that are hard to port.**

- A Rule dataclass hierarchy with eval() and contains_dont_know(), driven by strategies combinators
- Module-global mutable caches: _integral_cache, _parts_u_cache, the meijerint _lookup_table built lazily, heurisch _symbols_cache, BesselTable and DiffCache
- DifferentialExtension uses __slots__ with a __getattr__ that returns None for unset attributes, and setattr from an extension dict
- Wild(properties=[lambda ...]) predicates and match(old=True)
- The DEBUG_WRAP and _noconds_ decorators change the return shape (tuple vs expression); integrate() takes tri-state hint kwargs
- Exceptions used for control flow: NotImplementedError, PolynomialError, NonElementaryIntegralException, IntegralTransformError, MellinTransformStripError

**Behaviour a port must reproduce.**

- The engine that succeeds first decides the form of the answer. integrate(sin(x)**3*cos(x)**2, x) == cos(x)**5/5 - cos(x)**3/3 (from trigintegrate), while heurisch alone gives -sin(x)**2*cos(x)**3/3 - 2*cos(x)**5/15. The dispatch order and the tri-state hints (risch/manual/meijerg/heurisch = None/True/False) must be reproduced
- The oracle itself is not deterministic. integrate(exp(a*x)*cos(b*x), x) and integrate(sin(a*x)*cos(b*x), x) return Piecewise branches for Eq(a, I*b) and Eq(a, b) whose constants of integration change with PYTHONHASHSEED. I probed seeds 0-5: 4 of 5 parametric integrals varied, while simplify, trigsimp, factor, cancel, limit, series and cse outputs did not
- The default conds='piecewise' makes definite integrals with parameters return Piecewise((value, convergence_cond), (Integral(...), True)). heurisch_wrapper adds special-case branches, and integrate(1/(a+b*cos(x)), x) contains a literal zoo*(...) branch for Eq(a, 0) & Eq(b, 0)
- Results can carry dummies inside RootSum(Lambda(_i, ...)): integrate(exp(x)/(1+exp(2*x)), x) == RootSum(4*_z**2 + 1, Lambda(_i, _i*log(2*_i + exp(x))))
- risch_integrate(exp(-x**2), x) returns a NonElementaryIntegral, while integrate() goes on to reach sqrt(pi)*erf(x)/2 through other engines
- manualintegrate keeps module-level recursion guards that are cleared on every call: _integral_cache is set to None while an integral is in progress, and _parts_u_cache caps repeated integration by parts with the same u at 2. manualintegrate(x*exp(x)*sin(x), x) returns an unexpanded x*(...) form that integrate() never produces
- meijerint looks up formulas by tuples of classes sorted by class_key, matches with f.match(formula, old=True), and relies on polar numbers (exp_polar, polar_lift, unpolarify); core pattern matching and polar handling must be exact
- Definite-integral post-processing adds sign(coeff)*pi*floor((a - pi/2)/pi) to atan(...tan...) antiderivatives

**Rust prior art.**

- No symbolic integrator exists in Rust
- FriCAS (BSD-3, SPAD): a Risch implementation to use as an algorithmic reference
- Rubi rule set (MIT): a possible later rule source (SymPy dropped its own RUBI port)
- gauss-quad crate: f64 nodes only, so it cannot give n_digits Gauss nodes (licence to verify)
- Symbolica cannot be used (source-available)

**How its tests check results.** Mostly structural == on exact antiderivatives and Piecewise results. Some tests check by differentiation (diff_test: (i.diff(s).doit() - i.doit().diff(s)).expand() == 0), and meijerint uses verify_numerically. The risch/rde/prde tests assert on internal Poly tuples with explicit domains (Poly(..., domain='QQ')) and on DifferentialExtension(extension={...}) dicts. test_failing_integrals.py has 41 functions, 27 of them XFAIL. test_laplace.py is 4 giant @slow functions over 774 lines. Measured: 486 functions, 413 pass, 45 @slow, about 387 s (test_issue_15925b takes 36 s, test_issue_21034 18 s).

**Notes.** Sizes: laplace.py 2377 lines, meijerint.py 2191, manualintegrate.py 2174, risch.py 1851, integrals.py 1640, transforms.py 1590, prde.py 1333, intpoly.py 1302, rde.py 800, heurisch.py 781, quadrature.py 617, rationaltools.py 445, trigonometry.py 335. Research-grade parts: risch/rde/prde (Bronstein), the meijerint antecedents (Prudnikov et al., Saxena), Lazard-Rioboo-Trager in ratint. Heuristic piles: heurisch_wrapper, manualintegrate, laplace, the doit post-processing. Measured timings: integrate(1/(x**3+1)) 66 ms, x*exp(x)*sin(x) 242 ms, sin(x)/x 302 ms, fourier_transform(exp(-x**2)) 270 ms. Other clusters need the Integral class long before the engines: core/evalf (numeric integration), concrete/summations, stats crv, solvers.ode. intpoly needs geometry (Point, Polygon, Segment), and quadrature needs RootOf + arbitrary-precision evalf.

### `integrals.benchmarks`

34 source lines, 0 test functions. Difficulty **S**. Strategy: **defer or drop**. App relevance: none.

Two tiny timing scripts (bench_integrate.py: integrate(x**n*sin(x)) for n = 0..3; bench_trigintegrate.py: trigintegrate(sin(x)**3) and trigintegrate(x**2)). No tests.

**Why this strategy.** Re-create them as criterion benchmarks once trigintegrate and integrate() exist; there is nothing to port.

**Key algorithms.**

- No algorithms; they exercise trigintegrate and the integrate() dispatch

**Hard dependencies.**

- integrals

**Python features that are hard to port.**

- Benchmark discovery by function-name prefix (bench_/timeit_)

**Behaviour a port must reproduce.**

- trigintegrate(x**2, x) returns None (no match) rather than raising

**Rust prior art.**

- criterion (MIT/Apache-2.0) for the replacement benches

**How its tests check results.** No tests: 34 lines of benchmark functions that assert nothing.

**Notes.** Add the series benchmarks too (series/benchmarks: limit(1/x, x, oo); Add of 1000 powers of x plus O(x**1001), 0.22 s in SymPy) as a regression guard for Order absorption cost.

### `simplify`

13,214 source lines, 318 test functions. Difficulty **XL**. Strategy: **port faithfully**. App relevance: serve.

SymPy's simplification layer: the simplify() heuristic pipeline, the Fu trig rules and trigsimp, radsimp/collect/fraction, powsimp/powdenest, common subexpression elimination, hyperexpand (closed forms of hypergeometric and Meijer G functions), nsimplify, ratsimp/ratsimpmodprime, sqrtdenest, combsimp/gammasimp, logcombine, separatevars/posify, besselsimp, kroneckersimp, epath and nc_simplify.

**Why this strategy.** These are heuristic piles, and their exact outputs are the specification: 108 test files outside the cluster call simplify() and 16 call trigsimp. A 'better' simplifier, such as one built on e-graphs, would change grading outcomes in the app and break structural tests. Port faithfully, and offer any different engine only as an opt-in later.

**Key algorithms.**

- simplify() in simplify.py runs this pipeline: signsimp+collect_abs -> inversecombine -> recurse into args that are not Add/Mul/Pow -> nc_simplify -> nsimplify(rational) -> _bottom_up(normal) -> powsimp + as_content_primitive -> choose among cancel/_mexpand/together -> factor_terms -> sign to Abs -> Piecewise (piecewise_fold, piecewise_simplify) -> hyperexpand, kroneckersimp, besselsimp, trigsimp(deep), expand_log vs logcombine, combsimp, sum/product simplify, quantity_simplify -> powsimp variants -> exptrigsimp -> hollow_mul -> radsimp of the denominator -> signsimp. `shorter` picks by count_ops and keeps the first on ties; the original is restored if the measure grows past ratio=1.7; doit() runs at the end
- fu.py, the Fu-Zhong-Zeng trig transformation rules: TR0-TR22, TR2i, TR10i, TR12i, TRmorrie, TRpower, L(), the rule lists RL1/RL2/CTR1-CTR4, and fu() with measure (L, count_ops); also trig_split, hyper_as_trig (Osborne) and sincos_to_sum
- trigsimp.py: trigsimp(method='matching') by default runs futrig. _futrig is a greedy tree search (strategies.tree.greedy) over TR chains with objective (L, count_ops, _nodes, len(args), is_Add). Also trigsimp_groebner (an ideal of trig identities, groebner, then ratsimpmodprime), exptrigsimp, and trigsimp_old with the _trigpats pattern table
- radsimp.py: collect/rcollect/collect_const/collect_sqrt/collect_abs; radsimp with rad_rationalize (rationalises denominators with up to 4 surds); fraction/numer/denom (exact flag); split_surds
- powsimp.py: powsimp (combine='all'|'exp'|'base', deep, force) and powdenest/_denest_pow with polar handling
- cse_main.py cse: preprocessing with cse_opts sub_pre/sub_post, opt_cse (match_common_args uses FuncArgTracker to find shared Add/Mul argument subsets), tree_cse, reps_toposort, order='canonical'|'none', and numbered_symbols x0, x1, ...
- hyperexpand.py, closed forms in the style of Roach: FormulaCollection/MeijerFormulaCollection tables (add_formulae, add_meijerg_formulae), Operator shift/unshift/ReduceOrder, devise_plan/devise_plan_meijer, try_polynomial, try_lerchphi, try_shifted_sum, build_hypergeometric_formula, with the formula bases stored as Matrix ODE systems
- simplify.py nsimplify: evalf to 30 digits, mpmath.pslq for rationals, mpmath.identify with the user's constants, _real_to_rational (base10 vs exact), and tolerance handling
- sqrtdenest.py: _sqrtdenest0/_sqrtdenest_rec/_sqrtdenest1, _sqrt_match, _denester (Borodin-Fagin-Hopcroft-Tompa depth reduction), sqrt_biquadratic_denest, _sqrt_ratcomb
- ratsimp.py: ratsimp, and ratsimpmodprime (Groebner based, Monagan-Pearce). combsimp/gammasimp: gamma reflection and duplication, rf rewriting. Also logcombine, separatevars, posify, hypersimp/hypersimilar, besselsimp, kroneckersimp, nthroot, signsimp, epath

**Hard dependencies.**

- core
- polys
- functions.elementary
- functions.special
- functions.combinatorial
- strategies
- ntheory
- matrices
- external
- solvers
- concrete
- logic

**Python features that are hard to port.**

- kwargs plumbing with measure callables (measure=count_ops, ratio)
- Duck-typed dispatch through getattr(expr, '_eval_simplify', None) and getattr(w, 'normal', ...)
- Strategies combinators where a Python list means alternatives and a tuple means a chain inside greedy trees
- Transform objects used with xreplace (hollow_mul)
- Lazily built module globals (_trigpat, the hyperexpand _collection/_meijercollection) and @cacheit constants (_ROOT2, _ROOT3, _invROOT3)
- nsimplify mutates and then restores mpmath's global precision (mp.dps)
- cse uses Unevaluated wrappers and a numbered_symbols generator; hyperexpand Formula objects hold closures

**Behaviour a port must reproduce.**

- simplify chooses among candidates with `shorter` (the smallest count_ops, the first on ties) and returns the original whenever count_ops(result) > 1.7*count_ops(original), so any difference in count_ops changes outputs
- For plain Symbols, simplify leaves identities that depend on assumptions untouched: simplify(sqrt(x**2) - x), simplify(log(x**2) - 2*log(x)) and simplify(Abs(x)**2 - x**2) all return their input. Float exponents never equal Rationals: simplify(400*exp(log(0.75)*x) - 400*(3/4)**x) is not 0
- Odd forms that are tested must be reproduced: simplify(sin(x)**6 + cos(x)**6 + 3*sin(x)**2*cos(x)**2) == sin(x)**6 + cos(x)**6 - 3*cos(4*x)/8 + 3/8 (not 1); simplify((1/x+1/y)/(1/x-1/y)) == (-x - y)/(x - y); sqrtdenest(sqrt(3+2*sqrt(2)) + sqrt(2)) == sqrt(2)*(sqrt(2)/2 + 2)
- simplify calls doit() at the end by default, so an unevaluated Integral, Sum, Limit or Derivative inside the expression gets evaluated
- trigsimp's default method 'matching' runs futrig, not fu(), and the two can disagree: trigsimp(sin(x)**4 - cos(x)**4) == fu(...) == -cos(2*x), but the paths differ
- cse names its symbols x0, x1, ... in canonical (ordered) traversal order: cse([(x+y)**2 + sin(x+y), exp(x+y)]) == ([(x0, x + y)], [x0**2 + sin(x0), exp(x0)])
- nsimplify relies on the order of mpmath.identify's heuristics: nsimplify(0.1+0.2) == 3/10, nsimplify(pi.evalf(15), [pi]) == pi, nsimplify(1.4142135623730951) == sqrt(2)
- Core evaluation calls into this unit: Pow.__new__ uses radsimp.fraction + factor_terms (2**(x/log(2)) evaluates to exp(x) automatically); AssocOp._matches_commutative uses collect, and signsimp through _combine_inverse; Mul._eval_expand_mul uses fraction

**Rust prior art.**

- egg/egglog (MIT): e-graphs, only for an optional simplifier that does not follow SymPy
- No crate reproduces SymPy's simplify, fu or trigsimp. SymEngine (MIT) has no comparable simplify, which confirms porting from the source
- Symbolica cannot be used (source-available)

**How its tests check results.** Structural == on exact simplified forms (simplify(e) == -2*y, trigsimp(...) == ...). cse tests compare replacement lists, symbol names included. hyperexpand tests are mostly numeric: can_do/can_do_meijer/tn evaluate at random complex parameters through mpmath, about 250 uses, so they check correctness more than form. fu and trigsimp have a few verify_numerically checks. Measured: 318 functions in total; 309 ran because test_cse_diff.py imports pytest directly; about 277 pass, 15 @slow, about 19 XFAIL (8 in cse), about 100 s. Slowest: test_trigsimp_groebner 7.7 s, test_lerchphi 7.1 s, test_nthroot1 4.9 s.

**Notes.** App serve path: the grading decision is simplify(a - b) == 0, then expand. simplify's later stages only run when expr.has(...) finds their heads: hyperexpand is the identity when no hyper or meijerg is present; besselsimp, kroneckersimp, sum/product simplify and quantity_simplify only fire on those heads. So a serve milestone can ship the pipeline with those stages stubbed and still be exact for the app's vocabulary. App-like simplify calls take 10-150 ms in SymPy 1.14. Sizes: hyperexpand 2494 lines, simplify.py 2164, fu 2112, trigsimp 1252, radsimp 1234, cse_main 945, powsimp 718, sqrtdenest 678, gammasimp 493, cse_diff 291, ratsimp 222. Research-grade: hyperexpand (Roach), ratsimpmodprime/trigsimp_groebner, sqrtdenest. Heuristic piles: simplify(), fu/futrig, collect, powsimp, combsimp/gammasimp, nsimplify. nsimplify (compile time, through applyfunc(nsimplify)) and fu/trigsimp (roadmap: trig identity checking) are also on the app's list.

## Solvers and matrices

### Suggested port order inside the cluster

1. matrices (ZZ/QQ fast path first): RepMatrix over polys.matrices DomainMatrix, construction/indexing/arithmetic, det/rref/inv/nullspace/charpoly/eigenvects through DomainMatrix, together with a minimal MatrixExpr base. solvers (linear_eq_to_matrix, nsolve jacobian, simplex), ODE systems, diophantine and quaternions all build on it, and it is the first matrix slice the app roadmap (Unit 4) needs.
1. solvers.diophantine: pure integer number theory once ntheory exists. The sets cluster needs it for ImageSet intersection, and its Univariate case can wait for solveset_real.
1. solvers, polynomial core: polysys (Groebner + roots back-substitution), linsolve/solve_linear_system over polys solve_lin_sys, decompogen, simplex, solve_poly_inequality/solve_rational_inequalities. These need polys, not simplify, so they can land early and give fast oracle wins.
1. matrices (EXRAW generic paths): bareiss/berkowitz, _find_reasonable_pivot with simplify retry, GE/LU/QR/Cholesky/LDL, jordan_form, exp/log, pow heuristics with dotprodsimp. These need simplify (dotprodsimp, simplify) for faithful output forms.
1. solvers, solveset stack: the _invert_* tables, _solveset, _solve_as_poly/_solve_radical (unrad)/_solve_abs/_solve_trig/_transolve/_solve_modular, then solve_univariate_inequality and reduce_inequalities. This needs sets (ImageSet/ConditionSet), calculus.util and simplify. The app roadmap needs continuous_domain and inequality keys from here.
1. solvers, legacy solve: solve/_solve/_solve_system/checksol/_tsolve/bivariate LambertW, nonlinsolve/substitution, nsolve (needs bigfloat findroot). After simplify and nsimplify, because every solution is simplified and checked.
1. solvers, rsolve: rsolve_poly/rsolve_ratio early (polys only); rsolve_hyper after hypersimp and concrete.product. rsolve is on the app roadmap for sequence keys.
1. matrices.expressions (full): strategies-based MatMul/MatAdd canonicalize, block_collapse, special matrices; then derivatives once tensor.array.expressions is ported.
1. algebras: Quaternion, after trigsimp and integrate.
1. solvers.ode + pdsolve: run as Python on the Rust core through PyO3 until integrate/simplify/solve are at parity; then port hint by hint, constant-coefficient linear and linear systems first, Lie group heuristics last.
1. solvers.benchmarks / matrices.benchmarks: recreate as criterion benches whenever the measured feature lands.

### Risks

- Output-form fidelity. solve, solveset, dsolve and matrix tests assert exact structural forms (or dummy_eq), and those forms are byproducts of simplify, nsimplify, radsimp, powsimp, roots and the polys domain to_sympy conversions. A mathematically correct port fails the oracle unless every heuristic call happens in the same order with bit-identical sub-results, so the simplify cluster is the critical path for most of this cluster.
- Dependency cycle. solvers <-> sets (fancysets ImageSet uses _solveset_multi; handlers/intersection uses diophantine and solveset); calculus.util <-> inequalities <-> solveset; simplify (nthroot) -> solve; integrals (heurisch, rationaltools, manualintegrate) -> solve; core Expr.equals/is_constant -> solve/denoms; Relational.as_set -> solve_univariate_inequality. The Rust crates need late-bound function tables, or these pieces have to live in one crate.
- Two solver architectures with deliberately different semantics: solve (assumption-filtered, principal roots, polymorphic outputs, simplifies solutions) and solveset (ignores assumptions, real roots in the Reals domain, Set outputs with ImageSet/ConditionSet). Both must be ported, and they cannot be merged without breaking the oracle.
- Matrix results depend on the path taken. Integer/Rational matrices go through DomainMatrix; symbolic ones through construct_domain domains (inv/charpoly/eigenvects) or the EXRAW generic algorithms that call iszerofunc and simplify for pivots; Float matrices through mpmath.eig. Each path gives differently shaped answers, for example normalised versus unnormalised eigenvectors and expanded versus factored entries.
- The ODE subsystem is 11.8k lines of heuristics with about 150 Wild/.match sites and heavy integrate use. Its slow tests are skipped by default (43 skipped in test_single plus test_systems), so default-run parity understates the work. Until integrate is at parity it should stay as Python on the Rust core.
- nsolve and float eigen paths need a full mpmath replacement: findroot solvers (secant, MDNewton and others) with identical iteration semantics, plus arbitrary-precision eig with a precision-doubling loop. Otherwise the returned digits differ from the oracle.
- Test infrastructure. pytest is not installed in the pinned venv, so sympy.test() fails; sympy.testing.runtests.test works. matrices/tests/test_solvers.py imports pytest. About 4.75k test lines (test_matrices.py, test_commonmatrix.py) only cover deprecated mixins and should be excluded from the parity target.

### What other clusters and the core design must know

- polys.matrices is a hard prerequisite for matrices. Every explicit Matrix class (dense or sparse, mutable or immutable) stores a DomainMatrix in _rep, always the SDM (sparse dict of dicts) flavour, with domain limited to ZZ, QQ or EXRAW (repmatrix.py:23-60; probe: Matrix([[1,2],[3,4]])._rep is SDM over ZZ, Matrix([[x,1],[1,x]])._rep is SDM over EXRAW). The polys port must provide EXRAW, unify/unify_eq, convert_to, rref_den, inv_den, charpoly, dom_eigenvects and to_Matrix.
- Matrix.inv(), charpoly(), eigenvects() and det(method='domain-ge') call M.to_DM(), which goes through construct_domain (ZZ[x], ZZ(x), QQ<sqrt2>, EX). The printed results come from domain.to_sympy (inv([[x,1],[1,x]]) -> [[x/(x**2 - 1), -1/(x**2 - 1)], ...]), so to_sympy output in polys.domains must match SymPy exactly.
- Linear systems in solve, linsolve and solve_linear_system are delegated to polys: polys/solvers.py (sympy_eqs_to_ring, solve_lin_sys) and polys/matrices/linsolve.py (_linsolve, _linear_eq_to_dict, _lin_eq2dict). nonlinsolve and polysys depend on groebner(lex/grevlex), fglm, is_zero_dimensional, roots(cubics/quartics/quintics, filter='Z', strict=) and Poly.all_roots/real_roots/CRootOf.
- Core: ImmutableDenseMatrix is both a RepMatrix and a MatrixExpr/Expr/Basic, with args (rows, cols, Tuple(flat entries)), and sympify(Matrix) returns it. The expression DAG therefore needs a matrix node kind; MatrixKind takes part in Mul._kind_dispatcher (matrices/kind.py).
- Core: Basic._constructor_postprocessor_mapping is a global registry that rewrites Add/Mul when an argument is a MatrixExpr (-> MatAdd/MatMul), a Vector, a TensExpr or a quantum Operator/State. The Rust canonicalize layer needs an extension hook keyed on argument type. MatMul and MatAdd also subclass Mul and Add (is_Mul/is_Add are True).
- Core: the ODE classifier uses Wild + Expr.match with exclude= about 150 times, and solveset/test_solveset relies on dummy_eq (57 asserts) because ImageSet results use a fresh Dummy _n. Matching semantics and alpha-equivalence must be exact.
- Core and sets both call back into solvers: Expr.equals -> solve; Expr.is_constant -> denoms; Relational._eval_as_set -> solve_univariate_inequality; relational/boolalg -> linear_coeffs; sets/fancysets -> _solveset_multi; sets/handlers/intersection -> diop_linear, diophantine, solveset, invert_real; calculus.util (continuous_domain, function_range) and calculus.singularities -> solveset and solve_univariate_inequality.
- Simplify: solve, checksol, rsolve, odesimp, nonlinsolve (_separate_poly_nonpoly calls unrad(simplify(eq))) and the matrix pivot search (_find_reasonable_pivot) all call simplify; matrices also call dotprodsimp. The default dotprodsimp switch is a thread-local; Matrix**n expands intermediate products, but Matrix*Matrix does not.
- mpmath replacement: nsolve uses lambdify(modules='mpmath') + findroot (secant for 1-D, MDNewton with a Jacobian for systems). Float-matrix eigenvals/eigenvects use mpmath.eig in a precision-doubling loop until results stabilise (matrices/eigen.py:33-77) and return unit-normalised vectors.
- Tensor: MatrixExpr.diff goes through tensor.array.expressions (convert_matrix_to_array, array_derive, convert_array_to_matrix), and hadamard, applyfunc and matpow derivatives build ArrayContraction/ArrayDiagonal/ArrayTensorProduct. That tensor subpackage must exist before matrix-expression derivatives pass.
- Assumptions: matrices.expressions refine handlers call ask(Q.invertible/orthogonal/unitary/...) (matmul.py, inverse.py, determinant.py, blockmatrix.py), and the matrix fact handlers live in sympy/assumptions/handlers/matrices.py (assumptions cluster). The strategies unit (rm_id, unpack, flatten, exhaust, typed, do_one, glom, sort, bottom_up) is the rewrite engine behind MatMul/MatAdd/BlockMatrix canonicalization.
- ntheory must provide sqrt_mod/sqrt_mod_iter, factorint, divisors, is_square, perfect_power, nextprime, jacobi, symmetric_residue, iroot/isqrt (used by diophantine), plus discrete_log, nthroot_mod and totient (used by solveset._solve_modular).
- Deprecated code can be dropped from the parity target: matrices/common.py (3258 lines) and matrices/matrices.py (687) are mixins kept only so isinstance() still works, and they emit deprecation warnings. test_commonmatrix.py (1266 lines) and test_matrices.py (3487 lines; 178 of its 197 test names duplicate test_matrixbase.py) cover only those mixins.
- App priority: the app already reaches solveset at compile time (figure lint calls singularities). Its roadmap needs the real-domain solveset/inequality path (continuous_domain, on the serve path), rsolve, solve/nsolve, and exact Rational Matrix det/inv/pow/eigenvects (Unit 4). It needs neither ODE, diophantine, matrix expressions nor quaternions.

### `solvers`

13,559 source lines, 441 test functions. Difficulty **XL**. Strategy: **port faithfully**. App relevance: compile.

All equation-solving front ends except ODE and diophantine: solve (the older API; output is a list or dict, filtered by symbol assumptions), solveset (the newer API; output is a Set, aware of the domain), linsolve/nonlinsolve/substitution, solve_poly_system/factor_system, inequalities (solve_univariate_inequality, reduce_inequalities), rsolve (recurrences), nsolve (numeric), decompogen/compogen, unrad, LambertW bivariate solving, the exact LP simplex (lpmin/lpmax/linprog), pdsolve/classify_pde and the shared deutils (_desolve, ode_order).

**Why this strategy.** The oracle asserts exact output forms, and those forms come from a fixed sequence of heuristics (simplify, nsimplify, roots, unrad, invert tables), so a redesign would fail the tests even when it is mathematically right. Both architectures, solve and solveset, have to be ported side by side because their semantics differ on purpose. The polynomial cores are small and well specified (polysys, linsolve over polys, simplex, poly inequalities, rsolve_poly/ratio) and can be ported first.

**Key algorithms.**

- solve() pipeline (solvers/solvers.py:362-1290). Preprocessing: Eq->expr, hyperbolic->exp, Abs->Piecewise via separatevars, split into re/im for real symbols, recast_to_symbols, mask non-invertible subtrees with Dummy, Float->Rational via nsimplify, piecewise_fold, TR10/TR11 trig expansion. Then dispatch to _solve_undetermined, _solve or _solve_system. Postprocessing: simplify each solution, checksol, check_assumptions filter, nfloat restore, sort by default_sort_key.
- _solve, the single-equation dispatcher (solvers.py:1309-1745). Splits a Mul into factors; branches on Piecewise; tries solve_linear; looks at Poly generators (same base with different powers -> change of variable; several trig generators -> rewrite to tan); roots(cubics/quartics/quintics) with all_roots() CRootOf fallback; then unrad fallback, then _tsolve; finally a denominator check and checksol.
- _solve_system (solvers.py:1763-2028). Splits into connected components; linear systems go to solve_linear_system -> polys solve_lin_sys; polynomial systems go to solve_poly_system (tried over subsets of the symbols when underdetermined); failed equations are handled by successive substitution with _vsolve.
- checksol (solvers.py:186): subs, then is_constant, then _mexpand of the numerator, then simplify + posify, then a radical/function scan, then a numeric test abs(val.n(18).n(12, chop=True)) < 1e-9.
- unrad radical elimination (solvers.py:3299-3670): change of variables for a common base; isolate and raise to a power for 2-4 radical terms.
- _tsolve transcendental solving (solvers.py:2641): _invert, factor(powdenest()), logcombine, Pow rewrite, then bivariate._solve_lambert / bivariate_type (bivariate.py:193, 417) for LambertW solutions.
- nsolve (solvers.py:2905): lambdify(modules='mpmath') + mpmath.findroot (secant for 1-D, MDNewton with a symbolic jacobian for systems).
- solveset/_solveset dispatcher (solvers/solveset.py:1244-1400, 2337). Ignores assumptions by swapping in Dummy('R', real=True) or Dummy('C', complex=True); masks Abs and rewrites to Piecewise; branches for Mul -> Union of factors (only when the factors are finite), trig/hyperbolic -> _solve_trig (rewrite to exp, solve as a rational function), Piecewise -> as_expr_set_pairs, relational -> solve_univariate_inequality, modular -> _solve_modular (discrete_log, nthroot_mod); otherwise inverts via _invert_real/_invert_complex/_invert_trig_hyp_* (which return (expr, Set) with ImageSet images), then _solve_radical (unrad), _solve_abs, _solve_as_rational/_solve_as_poly (roots with all radicals, then all_roots), _transolve (_solve_exponential/_solve_logarithm/lambert), and a factor() fallback; domain_check filtering at the end.
- linsolve / linear_eq_to_matrix / linear_coeffs (solveset.py:2675-3175): a thin layer over polys _linsolve (sparse RREF over a ring) and solve_lin_sys; free parameters renamed tau0...
- nonlinsolve (solveset.py:3899): separates polynomial from non-polynomial equations (_separate_poly_nonpoly, which calls unrad(simplify(eq))); groebner(grevlex) -> is_zero_dimensional -> fglm('lex') -> solve_poly_system; positive-dimensional systems use groebner lex + substitution() (solveset.py:3184, about 550 lines of heuristics that keep ImageSet/Complement/Intersection bookkeeping).
- solve_poly_system (polysys.py:39-360): solve_biquadratic (two equations of degree <= 2: groebner + roots) or solve_generic (lex Groebner basis, take the last univariate poly, roots, substitute back recursively; zero-dimensional only); solve_triangulated; factor_system / factor_system_bool (polysys.py:477-870).
- Inequalities (inequalities.py): solve_poly_inequality (Poly.real_roots + sign intervals); solve_rational_inequalities; solve_univariate_inequality (inequalities.py:383-676), which uses periodicity/function_range, critical points from solvify, singularities from denoms, continuous_domain, a test point per interval via _pt evaluated with n(2), and an imaginary-part filter; reduce_inequalities/_reduce_inequalities; reduce_abs_inequality.
- rsolve (recurr.py): rsolve_poly (Abramov's polynomial solutions, undetermined coefficients, roots filter='Z'); rsolve_ratio (Abramov's rational solutions via resultant); rsolve_hyper (Petkovsek's Hyper algorithm: pairs of factors of the leading and trailing coefficients, characteristic-polynomial roots, hypersimp); initial conditions solved with solve.
- decompogen (decompogen.py): structural decomposition of function compositions plus polys.decompose for polynomial parts.
- Exact two-phase simplex with Bland's rule on a sympy Matrix (simplex.py:117-400); _primal_dual; lpmin/lpmax/linprog with bounds handling.
- pdsolve/classify_pde (pde.py): first-order linear PDEs with constant or variable coefficients by the method of characteristics (calls dsolve and integrate); pde_separate_add/mul separation of variables.

**Data structures.**

- Internal solution form list[dict[Expr, Expr]] (solve) converted to the polymorphic public outputs
- Set algebra results (FiniteSet, ImageSet(Lambda(_n, ...), Integers), ConditionSet, Union/Intersection/Complement, Interval, ProductSet) for solveset/nonlinsolve
- flags: dict threaded through recursive calls
- PolyRing/PolyElement equations from sympy_eqs_to_ring for linear systems; lists of Poly for Groebner bases
- Simplex tableau as a sympy Matrix with rational entries
- Interval lists for polynomial and rational inequalities

**Hard dependencies.**

- core
- functions.elementary
- polys
- polys.domains
- polys.matrices
- sets
- simplify
- calculus
- matrices
- logic
- ntheory
- concrete
- integrals
- utilities
- mpmath-replacement (bigfloat findroot)

**Python features that are hard to port.**

- **flags kwargs dict mutated and passed down recursions as hidden state
- Exceptions (NotImplementedError, PolynomialError, ValueError, UnsolvableFactorError) used as the normal fallback signal between strategies
- Polymorphic return types (list, dict, list of tuples, Relational, Set)
- Lazy imports inside functions to break the solvers/sets/calculus/simplify cycle
- Module-level lazily cached inverse tables (_trig_inverses, _hyp_inverses in solveset.py)
- nsolve uses lambdify (code generation + exec) with modules='mpmath'
- warnings.warn for unchecked assumptions

**Behaviour a port must reproduce.**

- The return type of solve() varies: a list of values (one symbol, bare expression); a dict (a linear system with one solution); a list of dicts; a list of tuples (symbols given in order); a Relational (inequalities); dict=True or set=True make it uniform (solvers.py:1225-1290). The PyO3 layer must reproduce each shape.
- solve filters solutions by the symbols' assumptions (positive p: solve(p**2-4,p) -> [2]). solveset ignores assumptions (returns {-2, 2}) because it swaps the symbol for a Dummy.
- Principal roots versus real roots: solve(x**(1/3)+2, x) == [] but solveset(x**(1/3)+2, x, S.Reals) == {-8}.
- solve simplifies every solution by default, but quietly turns simplify off when there are more than 2 roots or after unrad, and _solve_system only simplifies when a nonlinear fallback was used (default_simplify = bool(failed)). The output form therefore depends on simplify being bit-identical.
- Floats are rationalized with nsimplify and restored with nfloat when rational=None (the default). rational=False skips both steps.
- Outputs are sorted by default_sort_key. solveset ImageSets use a fresh Dummy _n, and 57 assertions in test_solveset use dummy_eq, so alpha-equivalence must be supported.
- solveset calls roots(..., cubics=True, quartics=True, quintics=True); when roots() is incomplete it falls back to CRootOf via all_roots and finally to a ConditionSet. Which one comes back is observable.
- checksol's numeric fallback uses 18-digit evalf and the threshold 1e-9; it returns None when it cannot decide, and solve keeps such solutions.
- rsolve names its arbitrary constants C0, C1, ... (dsolve starts at C1).
- flags is a mutable dict threaded through the recursion ('check', 'simplify', 'tsolve', '_unrad', 'tsolve_saw' memo, '_denominators'); which branches fire depends on these mutations.
- The Mul branch of _solveset only unions the factors when every factor is finite (_is_finite_with_finite_vars). Otherwise the equation goes through the general path.

**Rust prior art.**

- SymEngine solve.cpp (C++, MIT): polynomial/trig solving returning sets; usable as a structural reference, not as a dependency
- polys-cluster Rust ports (DomainMatrix sparse RREF, Groebner/FGLM) are the real backbone
- argmin (MIT/Apache-2.0): float-only root finding; not a substitute for arbitrary-precision findroot
- minilp / good_lp (MIT/Apache-2.0): float LP only; the exact rational simplex has to be ported

**How its tests check results.** Mostly structural == on lists, dicts and sets (solve(x**2-4,x) == [-2, 2]; solveset(...) == FiniteSet(...)); dummy_eq for ImageSet/ConditionSet results; many raises(NotImplementedError/ValueError/TypeError); nsolve checks numerically with tolerances; some tests call checksol/simplify themselves. XFAILs: 13 in test_solveset and 5 in test_solvers. pytest is not installed in the venv, but sympy.testing.runtests.test (the legacy runner) works. With 4 cores and 18 runs at once: test_solvers 133 passed (134 s), test_solveset 181 passed (232 s), test_inequalities 28 (30 s), test_recurr 14 (84 s), test_pde 11 (43 s), test_simplex 4 (9 s); test_polysys alone took 1.4 s.

**Notes.** About 13.6k source lines and 8.4k test lines. Light parts that can be ported early: polysys (872), simplex (1104), decompogen (126), linsolve/linear_eq_to_matrix, solve_poly_inequality/solve_rational_inequalities, rsolve_poly/rsolve_ratio. Heavy heuristic parts: solve/_solve/_tsolve/unrad/checksol (solvers.py, 3674 lines), _solveset and the invert tables plus nonlinsolve substitution (solveset.py, 4131 lines), solve_univariate_inequality. Hits on simplify-family helpers in solvers.py alone: 15 simplify, 5 TR/fu. solveset's results depend on how the sets cluster simplifies ImageSet/Union/Intersection. App: figure lint already calls singularities(), which uses solveset at compile time. The roadmap adds solveset/Interval for inequality keys, rsolve, and solve/nsolve at compile time, plus continuous_domain on the serve path, which calls solve_univariate_inequality and solveset_real. So port solveset's real-domain polynomial and rational path and the univariate inequalities first.

### `solvers.ode`

11,837 source lines, 144 test functions. Difficulty **XL**. Strategy: **keep in Python on the Rust core**. App relevance: none.

Symbolic ODE solving: dsolve and classify_ode for single ODEs (about 45 hints including _Integral variants, implemented by SingleODESolver subclasses), plus the power-series hints; linear and nonlinear ODE systems (linodesolve, dsolve_system, special 2- and 3-equation types); Lie-symmetry infinitesimal heuristics; rational Riccati solutions; 2F1 hypergeometric matching; checkodesol; constantsimp/constant_renumber; solving for initial conditions.

**Why this strategy.** The 11.8k lines are heuristics whose outputs are byproducts of integrate, simplify and solve. None of the tests can pass in Rust until those three are bit-identical, so the module should keep running as SymPy's Python code on the Rust core through PyO3. Port it later one hint class at a time (each hint's example table is a ready-made oracle), starting with the constant-coefficient linear hints and the linear systems that only need polys and matrices.

**Key algorithms.**

- classify_ode (ode/ode.py:819-1140): builds a SingleODEProblem, then iterates solver_map (ode/single.py) calling matches() on each class. Hint order is the allhints tuple (ode.py:277-323). Power-series hints (1st_power_series, 2nd_power_series_ordinary/regular) are matched inline with Wild/collect/cancel.
- dsolve -> deutils._desolve (solvers/deutils.py:134) -> _helper_simplify (ode.py:643) -> get_general_solution() -> odesimp (ode.py:1570). odesimp integrates, solves for f(x) with solve(force=True), applies logcombine for homogeneous hints, then constantsimp and constant_renumber; _remove_redundant_solutions follows, then solve_ics.
- Pattern-matched first-order solvers (single.py): separable (separatevars), 1st_exact, 1st_linear (integrating factor), Bernoulli, almost_linear, 1st_homogeneous_coeff_* (with _best chosen by ode_sol_simplicity), linear_coefficients, separable_reduced, Riccati_special_minus2, factorable, nth_algebraic.
- nth_linear_constant_coeff_homogeneous: characteristic polynomial roots, including CRootOf (nonhomogeneous.py:175). Undetermined coefficients: trial set from _undetermined_coefficients_match (nonhomogeneous.py:274-484). Variation of parameters via the wronskian (nonhomogeneous.py:124). Euler equations, homogeneous and nonhomogeneous (single.py:2387-2660).
- Bessel and Airy equation matching (single.py:2667-2840); Liouville; 2nd_nonlinear_autonomous_conserved; nth_order_reducible.
- Rational Riccati (riccati.py:795 solve_riccati): pole and valuation analysis (construct_c, construct_d), rational Laurent series, an auxiliary linear system, and a general solution built from particular solutions.
- 2F1 hypergeometric equivalence (hypergeometric.py:62-272).
- Lie group method (lie_group.py): infinitesimals via the heuristics abaco1_simple/product, bivariate, chi, function_sum, abaco2_similar/unique_unknown/unique_general and linear, then canonical coordinates. Heavy use of simplify, integrate and solve (21/16/9 call sites).
- Linear systems (systems.py): canonical_odes, linear_ode_to_matrix, _classify_linear_system, linodesolve types 1-6 using matrix_exp_jordan_form (systems.py:566), and a component split by strongly and weakly connected components (systems.py:1696-1784). Higher-order systems are reduced to first order.
- Power-series and Frobenius solutions (ode.py:2287-2610); checkodesol (subscheck.py:48), which substitutes and simplifies and falls back to series checks.

**Data structures.**

- SingleODEProblem (eq, func, sym, order, cached preprocessed forms)
- SingleODESolver subclasses holding Wild-match dicts; solver_map: hint -> class
- matching_hints ordered by the allhints tuple
- Numbered constant Symbols C1..Cn
- Coefficient matrices (A0, A1, ..., b) and Jordan-form matrix exponentials for systems

**Hard dependencies.**

- core
- integrals
- simplify
- solvers
- polys
- matrices
- series
- functions
- functions.special
- sets
- logic

**Python features that are hard to port.**

- Class registry solver_map and ClassVar hint names
- cached_property on SingleODEProblem
- globals()['ode_' + hint] dynamic function lookup (ode.py:658)
- Exceptions (ODEMatchError, NotImplementedError) used as the signal that a match failed
- Pattern dicts keyed by Wild objects

**Behaviour a port must reproduce.**

- Hint priority is the fixed allhints tuple. The default hint is the first match, so a port must reproduce every matches() predicate exactly, or dsolve picks a different method and returns a different form.
- Constants: C1..Cn come from get_numbered_constants. constantsimp absorbs constant subexpressions (2*C1 -> C1, exp(C1) -> C1, C1 + C2 -> C1) and constant_renumber reorders them, so tests depend on that exact order.
- dsolve returns an Eq or a list of Eq; *_Integral hints return unevaluated Integral objects; simplify=False changes the output a lot.
- odesimp calls solve(eq, f(x), force=True) and then applies powsimp(expand(), combine='exp') with a rational/float switch. Its output forms inherit every solve/simplify trap.
- Matching relies on Wild with exclude= and on collect(..., exact=True).match(); about 150 .match/Wild uses (ode.py: 77 .match and 57 Wild; single.py: 43 Wild).
- Slow tests are skipped by default (test_single: 25 skipped; test_systems: 18 skipped), so parity under the default run hides much of the surface.

**Rust prior art.**

- No permissive Rust symbolic ODE solver exists
- diffsol / ode_solvers (MIT/Apache-2.0): numeric only; useful only for cross-checking solutions numerically

**How its tests check results.** Example tables in test_single.py (_get_examples_* dicts with keys eq, sol, XFAIL, slow, simplify_flag, checkodesol_XFAIL) run by _ode_solver_test. Each one asserts the hint appears in classify_ode, that dsolve(eq, hint=...) dummy_eq-matches an expected solution, and that checkodesol(...) == (True, 0). test_ode.py also compares classify_ode tuples exactly. Timings under 4-way contention: test_single 14 passed/25 skipped (116 s), test_systems 22/18 skipped (119 s), test_ode 32 (86 s), test_riccati 15 (30 s), test_lie_group 7 (43 s).

**Notes.** Port this last. Its dependencies are integrate (the backbone of separable, 1st_linear, exact, variation of parameters and the Lie hints: 38 integrate/Integral sites in ode.py, 31 in single.py), simplify, solve and series. PDE code lives in the solvers unit but reuses deutils._desolve and dsolve. Not on the app roadmap.

### `solvers.diophantine`

3,985 source lines, 51 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: none.

Integer solutions of polynomial Diophantine equations. classify_diop assigns an equation type (Univariate, Linear, BinaryQuadratic, HomogeneousTernaryQuadratic(+Normal), GeneralPythagorean, GeneralSumOfSquares, GeneralSumOfEvenPowers; CubicThue and the inhomogeneous ternary/general quadratics are recognised but have no solver). The module also provides number-theory helpers: Pell/diop_DN, cornacchia, sum_of_squares, power_representation.

**Why this strategy.** Almost all of it is pure integer number theory with a small expression surface (Poly coefficient extraction plus building parametric tuples). It fits Rust well and gains a lot from fast bigint. It is also a dependency of the sets cluster (ImageSet intersection calls diop_linear and diophantine).

**Key algorithms.**

- diophantine() (diophantine/diophantine.py:1252): factor_list, then diop_solve per factor, merge_solution, then permute/signed_permutations expansion.
- Linear: base_solution_linear via igcdex; n-variable reduction with parameters t_0, t_1, ... (diophantine.py:256-450, 1704-1812).
- BinaryQuadratic (Alpern's method, diophantine.py:454-675): splits on the discriminant (B^2-4AC zero, square or non-square); transformation_to_DN (2441); diop_DN for Pell-type x^2-Dy^2=N (1925) via PQa continued fractions (2214), diop_bf_DN brute force (2270) and _special_diop_DN (2064); fundamental solutions mapped through the transformation matrix.
- Ternary quadratic: transformation_to_normal (2719), sqf_normal, Legendre descent (ldescent 3067, descent 3135), gaussian_reduce lattice reduction (3189), holzer minimisation (3277), parametrize_ternary_quadratic (2815).
- Sums of squares: prime_as_sum_of_two_squares (3511), cornacchia (2141), sum_of_three_squares (3581), sum_of_four_squares (3674), power_representation (3741, recursive generator), sum_of_squares (3902), GeneralPythagorean.

**Data structures.**

- Coefficient dict keyed by monomials (from Poly) per equation type
- DiophantineEquationType subclasses with matches()/solve()
- DiophantineSolutionSet (set of tuples plus symbols and parameters)
- Plain integer tuples and 2x2/3x3 integer transformation matrices

**Hard dependencies.**

- core
- ntheory
- polys
- matrices
- functions.elementary
- solvers

**Python features that are hard to port.**

- set subclass with extra attributes
- Generators (yield) in power_representation, sum_of_squares and sqrt_mod_iter consumers
- Mixing of int and Integer through as_int

**Behaviour a port must reproduce.**

- List-returning helpers have a fixed order and tests compare lists: diop_DN(-19, 23) == [(2, 1)], diop_DN(9, 0) == [(3*t, t)].
- Parametric solutions use integer Symbols t_0, t_1, ... (and m for the Pythagorean case). Tuple order follows default_sort_key of the variables unless syms= is given.
- permute=True expands sign and position permutations only for certain equation types and coefficient patterns (diophantine.py:1252-1500).
- DiophantineSolutionSet subclasses Python set, so tests compare unordered sets of tuples.
- Python ints and sympy Integers are mixed throughout (as_int); results must come back as sympy Integers inside Tuples.
- The Univariate type calls solveset_real(...).intersect(S.Integers).

**Rust prior art.**

- num-integer / num-bigint (MIT/Apache-2.0): ext_gcd, isqrt, nth_root
- dashu (MIT/Apache-2.0): bigint candidate named in the brief
- primal (MIT/Apache-2.0): prime sieving for helpers
- No permissive Rust Diophantine solver exists; PARI/GP is GPL and excluded

**How its tests check results.** Structural == on sets and lists of tuples (diop_solve((2*x + y + 1)**2) == {(t_0, -2*t_0 - 1)}). A check_solutions helper substitutes each solution back and tests for zero. 48 passed, 1 skipped, 2 xfail (33 s under contention).

**Notes.** 3980 source lines in one file and 1071 test lines. Port it right after ntheory (sqrt_mod, factorint, divisors, is_square, perfect_power, jacobi). The Univariate case can delegate to solveset_real once that exists. Helpers sum_of_three_squares, power_representation and prime_as_sum_of_two_squares are also used outside the solver.

### `solvers.benchmarks`

12 source lines, 0 test functions. Difficulty **S**. Strategy: **defer or drop**. App relevance: none.

A single asv-style timing function, timeit_linsolve_trivial: solve_linear_system on an 8x9 identity augmented matrix (solvers/benchmarks/bench_solvers.py).

**Why this strategy.** Twelve lines. Recreate it as a criterion benchmark when the linear solver lands. There is nothing to port.

**Key algorithms.**

- None. It is a benchmark harness around solve_linear_system -> polys solve_lin_sys.

**Data structures.**

- None

**Hard dependencies.**

- solvers
- matrices

**Python features that are hard to port.**

- asv naming convention timeit_*

**Behaviour a port must reproduce.**

- None

**Rust prior art.**

- criterion (MIT/Apache-2.0)

**How its tests check results.** No tests; asv timeit_ functions only.

**Notes.** Can serve as a seed for a performance regression suite comparing the Rust port with SymPy.

### `matrices`

19,187 source lines, 624 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: roadmap.

Explicit matrices. MatrixBase API (about 286 methods), the RepMatrix storage layer over DomainMatrix, Mutable/Immutable Dense/Sparse classes. Algorithms: determinants and charpoly, rref/echelon/rank, inverse (DM, GE, LU, ADJ, CH, LDL, QR, BLOCK), LU/QR/Cholesky/LDL/SVD/Hessenberg/rank decompositions, eigenvals/eigenvects/diagonalize/jordan_form, exp/log/analytic_func, nullspace/columnspace/rowspace/orthogonalize, linear solvers (GJ, LU, QR, CH, LDL, pinv, Cramer, least squares), graph components, Smith/Hermite normal forms, sparse tools (banded), and the dotprodsimp switch.

**Why this strategy.** The algorithms are classic and well specified. The ZZ/QQ fast paths are thin wrappers over DomainMatrix, which the polys cluster ports anyway. Outputs must match the oracle exactly, so port each algorithm, pivot rule and method-selection heuristic as written. Redesign only the class hierarchy: one Rust matrix value over DomainMatrix, with mutable/immutable and dense/sparse kept as thin views in the PyO3 layer, and drop the deprecated mixins.

**Key algorithms.**

- RepMatrix delegation (matrices/repmatrix.py:23-560). Storage is always a DomainMatrix of SDM type (sparse dict of dicts), even for 'dense' Matrix, with domain ZZ, QQ or EXRAW. _unify_element_sympy promotes the whole matrix ZZ -> QQ -> EXRAW. _eval_add and _eval_matrix_mul go straight to DomainMatrix + and *.
- det dispatch (determinant.py:569-712). n <= 3 uses explicit cofactor formulas with dotprodsimp; larger matrices are split into strongly_connected_components blocks; the default per-block method is Bareiss (_det_bareiss, determinant.py:716, iszerofunc=_is_zero_after_expand_mul); berkowitz, LU, bird, laplace and domain-ge (DomainMatrix) are alternatives.
- charpoly through DomainMatrix.charpoly (Berkowitz), with the domain from to_DM() (determinant.py:332-440).
- _find_reasonable_pivot (determinant.py:19-122): partial pivoting for all-Float columns; otherwise the first entry with iszerofunc False; undecidable entries are retried after simplify.
- rref (reductions.py:295-387): ZZ/QQ go through DomainMatrix rref_den then to_field/den; otherwise _row_reduce with iszerofunc/simpfunc and normalize_last.
- inv (inverse.py:383-524): to_DM() with construct_domain (ZZ[x], QQ(x), QQ<sqrt2>, ...) then inv_den and cancel; GE (dense) or LDL (sparse) only when the domain is EX; explicit methods available on request.
- Eigen (eigen.py). Floats go to mpmath.eig with a precision-doubling loop (eigen.py:33-77). Exact matrices go to dom_eigenvects over QQ or algebraic fields (eigen.py:299, polys/matrices/eigen.py), or to charpoly roots + nullspace (_eigenspace) for EX. jordan_form (eigen.py:1029) builds nullspace chains; diagonalize, singular_values, bidiagonalization (Householder).
- Matrix power (matrixbase.py:2894-2990). Integer exponents use dotprodsimp recursion by default; jordan_pow for 2x2 matrices with exp > 100000; cayley for exp > 10000 when dotprodsimp is off; symbolic exponents use jordan_pow, falling back to MatPow. exp/log go through Jordan blocks (_eval_matrix_exp_jblock).
- Decompositions (decompositions.py): LUdecomposition(_Simple, FF), QR by Gram-Schmidt (1375), cholesky/LDL dense plus sparse versions with the symbolic row structure (_liupc), SVD via eigendecomposition of A^H A (1127), upper Hessenberg (1557), rank_decomposition.
- Tarjan strongly connected components (graph.py:43) and block decomposition, used by det and eigenvals.
- Normal forms (normalforms.py): smith_normal_form and hermite_normal_form delegate to polys.matrices.normalforms.

**Data structures.**

- RepMatrix._rep: DomainMatrix whose rep is SDM (dict of dicts) over ZZ/QQ/EXRAW (DDM/DFM used inside polys)
- shape (rows, cols)
- ImmutableDenseMatrix args (Integer rows, Integer cols, Tuple of flat entries)
- DotProdSimpState thread-local flag
- Pivot tuples (offset, value, assumed_nonzero, newly_determined)

**Hard dependencies.**

- core
- polys.matrices
- polys.domains
- polys
- simplify
- functions.elementary
- matrices.expressions
- utilities
- mpmath-replacement (bigfloat eig)

**Python features that are hard to port.**

- Deep mixin MRO plus a deprecation metaclass (_MatrixDeprecatedMeta.__instancecheck__ in common.py)
- Thread-local contextmanager dotprodsimp() changing arithmetic globally
- call_highest_priority / _op_priority operator dispatch with MatrixExpr and other Exprs
- Duck-typed constructor inputs and __array__ numpy interop
- Callables (iszerofunc, simpfunc, simplify=function) passed as parameters
- Mutable versus hashable class split

**Behaviour a port must reproduce.**

- M*M with symbolic entries is not expanded ([[x+1,1],[2,x-1]]**2 via * gives (x + 1)**2 + 2), but M**2 is expanded (x**2 + 2*x + 3), because pow uses _eval_pow_by_recursion_dotprodsimp by default. The global dotprodsimp switch is a thread-local in utilities.py.
- Symbolic inv() goes through construct_domain and ZZ(x) cancellation; the result form (Matrix([[x/(x**2 - 1), -1/(x**2 - 1)], ...])) is fixed by polys domain to_sympy. The GE/iszerofunc path runs only when the domain is EX or a method is named.
- eigenvects with Float entries returns unit-normalised mpmath vectors (e.g. [-0.928, 0.371]). Exact matrices return an unnormalised nullspace basis (last free entry 1, e.g. [2/3, 1/3, 1]), sorted by default_sort_key of the eigenvalue. With rational=True, Floats are passed through nsimplify first.
- Matrix == Matrix is structural (DomainMatrix unify_eq) and does no simplification; .equals() uses Expr.equals per entry.
- ImmutableDenseMatrix is a Basic/Expr (MatrixExpr subclass) with args (rows, cols, Tuple(flat entries)) and is hashable; sympify(Matrix) returns ImmutableDenseMatrix; mutable matrices are unhashable; Matrix ops return MutableDenseMatrix.
- __getitem__/__setitem__ accept flat ints, negative indices, slices, tuples and lists of indices; slices return matrices. _handle_creation_inputs accepts nested lists, (r, c, list), (r, c, function), other matrices, numpy-like objects (__array__) and empty shapes.
- iszerofunc defaults differ: _iszero (x.is_zero) for rref, LU and inv; _is_zero_after_expand_mul for bareiss. A None from iszerofunc triggers a simplify retry.
- Non-Expr entries trigger a deprecation warning but are still accepted (repmatrix.py:_dod_to_DomainMatrix).

**Rust prior art.**

- SymEngine DenseMatrix/SparseMatrix (C++, MIT): symbolic LU, Bareiss/Berkowitz det, inverse, jacobian; usable as an algorithm reference
- nalgebra (Apache-2.0) and faer (MIT): float-only, useful for cross-checking numeric paths and float eig
- FLINT fmpz_mat/fmpq_mat (LGPL): excluded by licence; port polys.matrices instead

**How its tests check results.** Structural == between Matrix objects, eigenvect lists, decomposition tuples and dicts; many raises(ShapeError/NonSquareMatrixError/NonInvertibleMatrixError/ValueError/IndexError) (about 147 each in test_matrixbase.py and the deprecated test_matrices.py); some simplify(...) == 0 checks. Under contention: test_matrixbase 215 passed (69 s), test_eigen 18 (15 s), test_decompositions 11 (6 s). matrices/tests/test_solvers.py imports pytest directly. test_matrices.py and test_commonmatrix.py exist only to exercise deprecated classes (178 of the 197 test names in test_matrices.py duplicate test_matrixbase.py).

**Notes.** About 19.2k source lines, of which common.py (3258) and matrices.py (687) are deprecated mixins kept only for isinstance compatibility; drop them, plus about 4.75k deprecated test lines. The difficulty grade (L) assumes polys.matrices (DomainMatrix with SDM/DDM, rref_den, inv_den, charpoly, dom_eigenvects) is ported by the polys cluster. Without that it is XL. App roadmap (Unit 4, compile time): Matrix(key), det, inv, A*B, .T, is_invertible/det != 0, (M**n)*v, eigenvects for Markov steady state, applyfunc(nsimplify). All of these go through the ZZ/QQ DomainMatrix fast path when keys are Rational, so port that slice first. Steer the app away from Float matrices, which route to mpmath.eig and return normalised float vectors.

### `matrices.expressions`

5,906 source lines, 266 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: none.

Symbolic matrix algebra with symbolic shapes. MatrixExpr base, MatrixSymbol and MatrixElement, MatAdd/MatMul/MatPow, Transpose/Adjoint/Inverse/Trace/Determinant/Permanent, Identity/ZeroMatrix/OneMatrix/GenericIdentity, BlockMatrix/BlockDiagMatrix with block_collapse, Hadamard and Kronecker products and powers, DiagMatrix/DiagonalMatrix/DiagonalOf, FunctionMatrix, MatrixSlice, PermutationMatrix/MatrixPermute, DFT/IDFT, CompanionMatrix, LU/QR factor placeholders, ElementwiseApplyFunction, MatrixSet. Also matrix calculus (.diff) and index notation (_entry, from_index_summation).

**Why this strategy.** The canonicalization rule tables are small and declarative. Porting the strategies combinators and the rules in their original order reproduces the oracle's normal forms. The derivative machinery waits on the tensor.array.expressions port.

**Key algorithms.**

- MatMul canonicalize (matmul.py:443-447): exhaust(typed({MatMul: do_one(distribute_monom, any_zeros, remove_ids, combine_one_matrices, combine_powers, unpack, rm_id(1), merge_explicit, factor_in_front, flatten, combine_permutations)})).
- MatAdd canonicalize (matadd.py:147-157): rm_id(zero), unpack, flatten, glom(matrix_of, factor_of, combine), merge_explicit, sort(default_sort_key).
- Constructor postprocessors (matexpr.py:486-528) registered in Basic._constructor_postprocessor_mapping turn core Add/Mul nodes that contain MatrixExprs into MatAdd/MatMul.doit(deep=False), absorbing scalars into explicit matrices.
- block_collapse (blockmatrix.py:683-830) with the bc_* rules (bc_matadd, bc_block_plus_ident, bc_dist, bc_matmul, bc_transpose, bc_inverse); 2x2 block inversion by Schur complement with _choose_2x2_inversion_formula (blockmatrix.py:835-895), which uses ask(Q.invertible); blockcut/deblock/reblock_2x2.
- _entry index expansion: MatMul._entry builds Sum over Dummy indices i_1, i_2, ... (matmul.py:72-120); from_index_summation via tensor array conversions (matexpr.py:460).
- Matrix derivative (matexpr.py:531-594): convert_matrix_to_array, then array_derive, then convert_array_to_matrix (tensor.array.expressions); _matrix_derivative_old_algorithm with _LeftRightArgs (matexpr.py:739-875).
- Refine handlers via handlers_dict and ask(Q.orthogonal/unitary/invertible/...) for MatMul, Inverse, Determinant, Transpose; shape validation (_shape.py validate_matadd_integer/validate_matmul_integer).

**Data structures.**

- Basic nodes with shape (Expr, Expr); MatrixSymbol(Str name, rows, cols)
- MatMul args: optional scalar coefficient followed by matrices
- BlockMatrix wrapping an ImmutableMatrix of blocks
- _LeftRightArgs (lines of first/second pointers) for the old derivative algorithm

**Hard dependencies.**

- core
- matrices
- strategies
- assumptions
- tensor
- concrete
- combinatorics
- functions
- multipledispatch
- sets

**Python features that are hard to port.**

- Multiple inheritance MatMul(MatrixExpr, Mul) / MatAdd(MatrixExpr, Add)
- Import-time mutation of the global Basic._constructor_postprocessor_mapping and the refine handlers_dict
- multipledispatch for _eval_is_eq
- Higher-order strategy combinators (exhaust, typed, do_one, glom, condition)

**Behaviour a port must reproduce.**

- Operators canonicalize but raw constructors do not: A*B calls MatMul(...).doit(), while MatMul(A, Identity(n), B) stays as A*I*B until .doit(). This fits the planned raw-constructor/canonicalize split.
- MatMul subclasses Mul and MatAdd subclasses Add (is_Mul / is_Add are True), so generic core code that tests is_Mul sees them. is_commutative=False, _op_priority=11, kind=MatrixKind.
- A*A -> MatPow(A, 2); (A*B).T -> B.T*A.T; (A*B).I -> B**(-1)*A**(-1) with no invertibility check; 2*A + B from A+A+B; MatrixSymbol('A',2,2)*ImmutableMatrix(eye(2)) -> A.
- Derivatives must reproduce exact forms such as (x.T*A*x).diff(x) == A*x + A.T*x and Trace(A*B).diff(A) == B.T, which depend on the array-expression round-trip.
- _entry produces Dummy indices; tests use dummy_eq.
- Shape mismatches raise ShapeError only when the dimensions are provably different (symbolic dimensions pass).

**Rust prior art.**

- egg (MIT): an e-graph engine could host the rules, but would not reproduce the strategy order; a faithful port of strategies is closer to the oracle
- No Rust crate does symbolic matrix-expression algebra

**How its tests check results.** Structural == and dummy_eq on expression trees; raises(ShapeError/TypeError); explicit-matrix comparisons via as_explicit(). 258 passed, 2 skipped, 6 xfail across 25 test files (68 s under contention); test_derivatives.py (477 lines) is the heaviest cross-cluster test.

**Notes.** 5.9k source lines and 3.9k test lines. Split the port: the minimal MatrixExpr base (shape, kind, operator dispatch, postprocessor hook) must ship with matrices, because ImmutableDenseMatrix inherits it and Matrix**y falls back to MatPow. The rest of the rules, block_collapse and the derivatives (which need tensor.array.expressions) can follow later. Not used by the app.

### `matrices.benchmarks`

21 source lines, 0 test functions. Difficulty **S**. Strategy: **defer or drop**. App relevance: none.

Four asv timing functions: M[3,3], M[Integer(3), Integer(3)], M[:, :] on eye(100), and zeros(100, 100) (matrices/benchmarks/bench_matrix.py).

**Why this strategy.** Twenty-one lines. Recreate them as criterion benchmarks for the Rust matrix type and the PyO3 indexing overhead.

**Key algorithms.**

- None. These are timing harnesses for indexing, slicing and construction.

**Data structures.**

- None

**Hard dependencies.**

- matrices

**Python features that are hard to port.**

- asv naming convention timeit_*

**Behaviour a port must reproduce.**

- None

**Rust prior art.**

- criterion (MIT/Apache-2.0)

**How its tests check results.** No tests; asv timeit_ functions only.

**Notes.** Useful for measuring the PyO3 boundary cost on __getitem__, which SymPy tests hammer.

### `algebras`

1,669 source lines, 20 test functions. Difficulty **M**. Strategy: **port faithfully**. App relevance: none.

The Quaternion class, an Expr subclass. Hamilton product, add/sub/div, norm, normalize, inverse, pow/pow_cos_sin, exp/log. Conversions to and from rotation matrices, axis-angle and Euler angles (12 sequences, intrinsic or extrinsic). Also rotate_point, integrate/diff, and predicates (is_pure, is_zero_quaternion, parallel, orthogonal, arc_coplanar, vector_coplanar), mensor, index_vector, product_matrix_left/right.

**Why this strategy.** Small (1.7k lines) and self-contained formula code. It only becomes testable once trigsimp and simplify are ported, so schedule it late, or run it as Python on the Rust core until then.

**Key algorithms.**

- _generic_mul Hamilton product, with norm propagation and real_field complex-scalar handling (algebras/quaternion.py:797-870).
- from_euler (quaternion.py:375-436): composes elementary rotation quaternions and applies trigsimp.
- to_euler (quaternion.py:438-580): Bernardes-Viollet generic method with angle_addition and avoid_square_root options.
- from_rotation_matrix (quaternion.py:624-666) using Max/sign/sqrt; to_rotation_matrix with an optional homogeneous translation (quaternion.py:1186-1260).
- norm (quaternion.py:877-886) as sqrt(trigsimp(a^2+b^2+c^2+d^2)) unless a norm was preset; pow via repeated multiply; exp/log in polar form.

**Data structures.**

- Quaternion args (a, b, c, d) plus side fields _real_field: bool and _norm: Option<Expr>

**Hard dependencies.**

- core
- functions.elementary
- matrices
- simplify
- integrals

**Python features that are hard to port.**

- Per-instance mutable state outside args (set_norm)
- Operator overloading through Expr dunder methods

**Behaviour a port must reproduce.**

- Instances carry _real_field and _norm outside args. They are not part of hash or equality, but they propagate through multiplication; set_norm mutates an existing instance.
- Multiplying by a complex scalar when real_field=True converts it to Quaternion(re, im, 0, 0) before the Hamilton product.
- norm() and from_euler() call trigsimp, so output forms depend on trigsimp parity.
- Non-commutative arguments raise ValueError; arguments are sympified.

**Rust prior art.**

- nalgebra::Quaternion / UnitQuaternion (Apache-2.0): numeric only
- quaternion crate (MIT/Apache-2.0): numeric only

**How its tests check results.** Structural == on Quaternions and Matrices, often after trigsimp/simplify; expand(...) == 0 identity checks. 18 passed, 2 skipped (8 s under contention).

**Notes.** Low priority and not on the app roadmap. The 16 trigsimp call sites are the main blocker.

## Printing, parsing, code generation, plotting

### Suggested port order inside the cluster

1. 1. printing core framework plus StrPrinter and ReprPrinter for core node kinds: Printer trait, settings, 'virtual MRO' dispatch, precedence, split_super_sub. The PyO3 oracle harness needs it, 32k doctest lines compare str(), core's atom sort_key uses str(), and the app uses srepr/str.
1. 2. parsing (secure parse_expr/sympify): sympify(str) and S('...') appear in about 473 test lines and back the srepr round-trip. The app's fallback and template parsing sit on the serve path. Build the raw-tree plus separate canonicalize design here.
1. 3. LaTeX printer: app display path (answer keys, inequality endpoints) and 1297 golden asserts. Expose an override API early because the app subclasses LatexPrinter.
1. 4. parsing.latex (new hand-written parser with lark-compat/antlr-compat/strict profiles): the app's main serve path and a large speed win (88-140 ms per parse in SymPy+Lark).
1. 5. printing.pretty, stringPict engine first: self-contained, golden-tested, used by pprint doctests.
1. 6. codegen AST, then text code printers (C, Python, NumPy first so Python-side lambdify works unchanged; then Fortran, C++, Rust, JS, Julia, Octave, R, GLSL, Maple, Mathematica, SMT-LIB). Codegen AST must exist before the code printers.
1. 7. MathML printers (string output plus a minidom adapter in the PyO3 shim for DOM-inspecting tests).
1. 8. parse_mathematica (already a hand-written precedence parser) and parse_maxima (rebased on the secure parser).
1. 9. Python-side glue on the PyO3 module: interactive, plotting (backed by a Rust numeric evaluator), lambdify, preview. Port textplot and intervalmath to Rust opportunistically.
1. 10. Deferred: parsing.autolev (keep Python plus antlr4 runtime), parsing.c (keep Python plus libclang), parsing.fortran (drop or defer), Aesara/Theano/TF/Torch/llvmjit printers (stay Python), pygletplot (drop).

### Risks

- Ordering parity is emergent. Printed term order comes from as_terms/_parse_order/monomial_key/decompose_power/sort_key/class_key, with float tie-breaks on coefficients. Small deviations break thousands of string asserts and doctests. Mitigation: differential-fuzz str()/latex() against the SymPy oracle from day one, starting before most printers exist.
- Breadth over depth. More than 1,300 _print_ methods cover types from about 20 packages, so printer parity cannot finish before those types are ported. Plan staged coverage and keep a Python-fallback printer path in the shim.
- There are two disagreeing LaTeX oracles, and neither is a clean spec. Lark returns ambiguity Trees and tuples, depends on the global evaluate flag, lacks \pi, and is slow. The ANTLR backend is not installed in the pinned venv, so its 185-pair suite and the autolev tests never run unless antlr4-python3-runtime 4.11 is added. Decide per construct whether to replicate or fix, and record each decision.
- Security versus compatibility. SymPy's parse_expr is eval of arbitrary Python: verified code execution and unbounded integer DoS. Its tests use Python semantics ('x is None', pow with 3 args, arbitrary local_dict objects, user transformations). The secure Rust parser cannot pass all of these, so the PyO3 shim needs an explicit opt-in legacy-eval path. Never make that path the default for untrusted input.
- Silent misreads in implicit multiplication (verified: 'x1' -> x, 'x12' -> 12*x, 'xtheta' -> a*e*h*t**2*x, 'f(x)' -> f*x, \overline{AB} -> conjugate(A)*conjugate(B)). Faithful compatibility conflicts with the app's 'never silently misread' goal, so use profiles.
- Under-covered oracle. Many tests in this cluster are skipped in the pinned venv (numpy, jax, tensorflow, torch, cupy, aesara, theano, llvmlite, antlr4, clang, lfortran, matplotlib, IPython). Build a second, 'full extras' oracle venv, or accept unverified ports.
- Non-string outputs: MathML tests walk minidom DOM trees, and some printers return framework graphs. These need adapter layers rather than string ports.
- Environment and global-state coupling: terminal width (pretty wrap), global unicode flag, Printer._global_settings, global evaluate(); Python-version-dependent tokenization ('!' token since 3.12).

### What other clusters and the core design must know

- Core must reproduce exactly Expr.as_ordered_terms/as_terms/_parse_order (core/expr.py:1091-1262), Mul.as_ordered_factors (core/mul.py:2091), Expr.sort_key (core/expr.py:150), the class_key tuples (Number (1,0), Atom (2,0), Mul (3,0), Add (3,1), Pow (3,2), Function (4, i, name) with its special numbering table in core/function.py:507), and exprtools.decompose_power. All printers order terms through these at print time. The tie-break casts coefficients to Python complex floats.
- Atom.sort_key (core/basic.py:2266) uses str(self), and 34 non-test sorts use key=str. Canonical ordering therefore depends on the StrPrinter for atoms. Port StrPrinter for atoms together with the core ordering.
- Raw (evaluate=False) constructors must keep argument order and identity elements, because printers print unevaluated Mul args in stored order (Mul(3,x,2) -> '3*x*2', Mul(x,1/2) -> 'x*(1/2)'). The unevaluated-detection rule (args[0] is One, or a Number among args[1:]) is part of the contract. Printed Add order does NOT depend on stored order unless order='none'.
- The mpf crate replacing mpmath must reproduce mpmath.libmp.to_str and prec_to_dps exactly, including strip_zeros, min_fixed/max_fixed and exponent formatting. StrPrinter/LatexPrinter/PrettyPrinter call it for every Float, and full_prec='auto' depends on printer recursion depth.
- sympify(str) (core/sympify.py:490-511) is just parse_expr with standard_transformations + convert_xor (+ rationalize). The secure parser therefore sits on the critical path for running SymPy's own suite through PyO3: about 473 S('..')/sympify('..') test lines and the srepr round-trips.
- The node model needs a per-kind class-name chain (a 'virtual MRO': sin -> TrigonometricFunction -> Function -> Application -> Expr -> Basic) so printers can emulate '_print_<Class>' fallback dispatch. The PyO3 shim must let Python subclasses of printers and Functions override _print_<Class> and _latex/_sympystr hooks. The app subclasses LatexPrinter and Function, and physics.quantum/vector/units rely on object-side hooks.
- Avoid a printing -> everything dependency. Each domain crate (sets, matrices, logic, polys, physics, tensor, stats) should register printer implementations for its own node kinds through a trait or registry. SymPy's latex.py alone imports from about 20 packages.
- polys.orderings.monomial_key (lex/grlex/grevlex and rev- variants) is needed by the printers' order= setting. simplify.radsimp.fraction(exact=True) is called by LatexPrinter._print_Mul for every product. Printer._compare_pretty (order='old') needs Wild pattern matching.
- A global or contextual evaluate flag is observable through parsers: the Lark LaTeX transformer uses evaluating constructors, so results change under evaluate(False). Core needs an explicit evaluate context usable by parsers.
- codegen.ast nodes are Basic subclasses with non-Expr atoms (String, Type with nbits/nmant/nexp). Code printers, parsing.c/fortran and utilities.codegen/autowrap consume them. Give codegen its own crate depending only on core.
- lambdify (utilities/lambdify.py) execs Python source produced by printing.pycode/numpy/lambdarepr printers. Keep lambdify in Python, require the Rust code printers to emit byte-identical strings, and provide a separate native compiled numeric evaluator in Rust. Plotting needs it, and the app needs it for sampling (inequality_2d, figure lint).
- Security for any service using rsympy: parse_expr, sympify(str), parse_maxima, parsing.ast_parser and plotting.experimental_lambdify all reach eval/exec in SymPy. Verified: parse_expr("__import__('os').getpid()") executes. The Rust API must expose only the closed-grammar parser with resource limits. Lambdify-style exec belongs only in the opt-in Python layer.
- Oracle environment: install antlr4-python3-runtime==4.11, numpy, matplotlib and IPython in an extras venv. Otherwise test_latex.py (ANTLR), test_autolev.py, the numpy printer tests, plotting and interactive tests are silently skipped and give no answer key.

### `printing`

20,048 source lines, 1,068 test functions. Difficulty **XL**. Strategy: **port faithfully**. App relevance: serve.

All non-2D printers: Printer base/dispatch (printer.py), StrPrinter/sstr (str.py), ReprPrinter/srepr (repr.py), LatexPrinter (latex.py, 3318 lines), MathML content+presentation (mathml.py, 2157), precedence and symbol-name conventions, about 11k lines of code printers (codeprinter.py base plus C89/C99/C11, C++98/11/17, Fortran, JS, Julia, Octave, R, Rust, GLSL, Maple, Mathematica, SMT-LIB, Python/mpmath/cmath/SymPy, NumPy/SciPy/CuPy/JAX, NumExpr, Torch, TensorFlow, Aesara, Theano, llvmlite JIT), plus tree/dot/tableform/python/preview/gtk utilities. str() and repr() of every SymPy object go through StrPrinter via core/_print_helpers.Printable.

**Why this strategy.** SymPy's tests and doctests compare printed strings, so output must match byte for byte. Two parts of the design change. Dispatch becomes a trait per printer, using a 'virtual MRO' class-name chain per node kind, plus a registry so each domain crate supplies printers for its own nodes. Without that, printing would depend on every crate. Printers that build Python objects or call external programs stay in Python on the PyO3 layer: Aesara, Theano (deprecated since 1.8), TF, Torch, llvmjit, preview (subprocess latex/dvipng) and gtk.

**Key algorithms.**

- Printer._print dispatch (printing/printer.py ~291-330): first an object hook named by printmethod (_sympystr/_latex/_pretty/_sympyrepr), then '_print_'+cls.__name__ over type(expr).__mro__. The MRO is cut at AppliedUndef/UndefinedFunction. A subclass of Function keeps only classes with the same name or names ending in 'Base'. Falls back to emptyPrinter.
- Term ordering is computed at print time, not stored. Printer._as_ordered_terms -> Expr.as_ordered_terms (core/expr.py:1157) -> Expr.as_terms (core/expr.py:1211; uses exprtools.decompose_power, coefficients cast to Python complex) -> Expr._parse_order (core/expr.py:1091; polys.orderings.monomial_key, negated lex monomial, then ncpart sort_key, then the float tie-break ((bool(im),im),(re,im))). There is a special case Number + negative Mul -> '1 - x', and Order terms go last.
- Mul.as_ordered_factors (core/mul.py:2091) sorts the commutative part by Expr.sort_key (core/expr.py:150: class_key, args, exp, coeff; E**x keyed as exp(x)).
- Printer._compare_pretty for order='old' (printer.py ~350): Wild-pattern match p1*p2**p3, then Basic.compare.
- StrPrinter._print_Add/_print_Mul (str.py:52, 262): sign hoisting, numerator/denominator split for negative exponents, Rational split into p and q, the pow_paren special case for issue 14160, and unevaluated-Mul detection (args[0] is One or any Number in args[1:]), which prints the stored args verbatim.
- StrPrinter._print_Float (str.py:738): mpmath.libmp.to_str with dps from prec_to_dps. full_prec='auto' strips zeros only when _print_level > 1, plus the '.0'/'-.0'/'+inf' fix-ups.
- precedence()/precedence_traditional() tables plus PRECEDENCE_FUNCTIONS (printing/precedence.py); parenthesize(strict).
- split_super_sub (printing/conventions.py:12): parses ^, __ and _ markers, and trailing digits become a subscript.
- LatexPrinter._print_Mul (latex.py:520): calls simplify.radsimp.fraction(exact=True) and uses fold_short_frac, long_frac_ratio, mul_symbol tables, the _between_two_numbers_p regexes, and sifts physics units via hasattr(_scale_factor/is_physical_constant). translate() (latex.py:2936) handles greek and modifier suffixes, longest first, with a stable sort.
- CodePrinter.doprint (codeprinter.py): assign_to/Assignment, tensor index contraction loops via tensor.get_contraction_structure, known_functions tables (str or a list of (predicate, name)), _rewriteable_functions, strict/human modes that collect unsupported exprs sorted by str, reserved-word suffixing, Piecewise multi-line ternaries.
- MathML printers build xml.dom.minidom trees (mathml.py); _print returns DOM Elements.
- ReprPrinter (repr.py): srepr with Symbol assumptions, Float('..', precision=53), and eval(srepr(e)) == e round-trip.

**Data structures.**

- Settings map per printer (_default_settings merged with _global_settings, validated keys)
- Mutable _context dict and _print_level counter (affect Float stripping and codegen printing)
- PRECEDENCE / PRECEDENCE_VALUES / PRECEDENCE_FUNCTIONS tables
- known_functions tables per language: name -> str or [(predicate, name)]; _rewriteable_functions; reserved_words sets
- tex_greek_dictionary, greek_letters_set, other_symbols, modifier_dict (ordered), accepted_latex_functions
- CodePrinter _not_supported set and _number_symbols set; pycode module_imports registry
- minidom Document/Element tree for MathML

**Hard dependencies.**

- core
- simplify
- polys
- codegen

**Python features that are hard to port.**

- getattr-based '_print_<ClassName>' dispatch over the Python MRO, with special-case MRO filtering
- Duck-typed object hooks (hasattr(expr, printmethod)) and units detection by hasattr(_scale_factor/is_physical_constant)
- _PrintFunction rewrites __signature__ via inspect so help() shows the settings
- Class-level global settings mutated at runtime (set_global_settings / init_printing)
- Printers that construct live Python objects (minidom, aesara/tf graphs, llvmlite IR)
- Multiple inheritance mixins (ArrayPrinter + PythonCodePrinter, _CXXCodePrinterBase + C99CodePrinter, IntervalPrinter(MpmathPrinter, LambdaPrinter))

**Behaviour a port must reproduce.**

- Probed order rules: str(sqrt(x)+x**2+x)=='sqrt(x) + x**2 + x'. The generators are sorted by sort_key, so sqrt(x) becomes its own generator ahead of x.
- More probed order rules: 'x + x**2.0'; '1 + sqrt(2)' but 'sqrt(2) + 2'; '3 + pi' but 'pi + 4'. Constants are ordered by their float value.
- str(x**2/3+sin(x)*exp(-y)+S(1)/2+sqrt(x)+Abs(x)) == 'sqrt(x) + x**2/3 + Abs(x) + 1/2 + exp(-y)*sin(x)'. The constant lands in the middle because exp(-y) gives a negative exponent in the monomial.
- Unevaluated nodes print in stored order. Mul(3,x,2,evaluate=False) -> '3*x*2'; Mul(x,1/2,evaluate=False) -> 'x*(1/2)'. Unevaluated Add is still sorted ('x + 3'). Raw constructors must therefore preserve argument order.
- Float: str(Float(1.5)) == '1.50000000000000' but str(1.5*x) == '1.5*x', because the printer's recursion level matters. str(N(pi,1)) == '3.'. Exponent switch: '1.00000000000000e-5', '1.23456789012346e+17'. LaTeX gives '1.0 \cdot 10^{-5}'.
- The printmethod hook wins over class dispatch. Objects in physics.quantum, vector, units, tensor and codegen define about 38 _sympystr, 60 _latex, 28 _pretty and 16 _sympyrepr methods.
- Unknown settings raise TypeError. Printer._global_settings is class-level mutable state changed by init_printing.
- LaTeX symbol names: 'x1' -> x_{1}; 'alphahatdotprime' -> "{\dot{\hat{\alpha}}}'"; greek names are case-folded through greek_letters_set; the symbol_names override exists.
- latex(log(x)) == '\log{\left(x \right)}' (ln_notation off by default). The app overrides this through a LatexPrinter subclass, so Python subclass overrides must keep working.
- Code printers: Rational prints '1.0/2.0' in C/C++/R/GLSL, '1/2' in Python/JS, '1 // 2' in Julia, '1_f64/2.0' in Rust. gamma -> tgamma in C99. fcode refuses an inline Piecewise before F95. MCodePrinter raises PrintMethodNotImplementedError for Piecewise.
- Some printers do not return strings. MathML returns minidom nodes, and tests walk childNodes/nodeName (453 lines in test_mathml.py). Aesara/Theano/TF/Torch build framework graphs. llvmjitcode emits llvmlite IR.
- emptyPrinter in StrPrinter returns repr() for Basic and str() otherwise. Container printing (_print_dict) sorts keys by default_sort_key.

**Rust prior art.**

- SymEngine (MIT, C++): StrPrinter, LatexPrinter, MathMLPrinter, C/JS code printers. A lesson: its term order differs from SymPy's ('1 + x' vs 'x + 1'), which is one reason its str output never matched SymPy.
- quick-xml (MIT) to write MathML; roxmltree (MIT/Apache-2.0) for a DOM adapter in the PyO3 test shim
- pyo3 (MIT/Apache-2.0) for Python-subclassable printers
- Symbolica has printers and parsers but is not permissively licensed, so it is reference only

**How its tests check results.** Exact string equality: 489 asserts in test_str.py, 1297 in test_latex.py, 221 in test_c.py, and many per language. srepr round-trip. MathML: 453 lines inspect the DOM (childNodes/nodeName) and 183 compare toxml strings. raises() is used for bad settings or unsupported functions. The numpy/jax/tensorflow/torch/cupy/aesara/theano/llvmlite test files are gated by import_module and are SKIPPED in the pinned venv, which has only mpmath and lark. About 32k '>>>' doctest lines across SymPy also compare str() output, and outside printing 1240 assert lines compare str/sstr/repr.

**Notes.** Size: str 1021 lines, repr 339, latex 3318, mathml 2157, code printers about 11k, of which about 8.9k are text-emitting and worth porting. Printing tests: about 1068 functions. Order of work: Str+Repr for core types first (oracle harness, doctests, atom sort keys), then LaTeX, then text code printers (C/Python/NumPy first because lambdify needs them), then MathML. Keep in Python: theanocode (deprecated), aesaracode, tensorflow, pytorch, llvmjitcode, preview, gtk.

### `printing.pretty`

4,217 source lines, 156 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: none.

2D ASCII/Unicode pretty printer (pprint/pretty). Contains the stringPict/prettyForm layout engine (stringpict.py), the glyph and symbol tables (pretty_symbology.py), and PrettyPrinter with 218 _print_ methods (pretty.py, 2937 lines).

**Why this strategy.** The output is pure strings with a deterministic layout algorithm and an exhaustive golden test file. stringPict is a small self-contained engine (537 lines) that maps cleanly to a Rust struct of Vec<String> plus baseline. Global flags become explicit config, and the PyO3 shim keeps the globals.

**Key algorithms.**

- stringPict.next/stack/below/above/parens/root/leftslash (pretty/stringpict.py): box composition with a baseline. equalLengths centres lines.
- prettyForm operator algebra (stringpict.py:354-520): __add__, __mul__, __truediv__, __pow__, apply, with binding levels ATOM/FUNC/DIV/POW/MUL/NEG/ADD/OPEN deciding parentheses.
- stringPict.render (stringpict.py:249): line wrapping into chunks of num_columns or terminal width, with overflow markers ('↪ ' in unicode, '> ' in ASCII).
- line_width/center/center_pad (pretty_symbology.py:676-730): width is the codepoint count after removing combining marks in U+0300..U+036F.
- pretty_symbol (pretty_symbology.py:550): greek names, unicode sub/superscript tables, fallback when a subscript is not representable. xobj/vobj/hobj build extensible brackets, integral and sum signs from glyph pieces.
- PrettyPrinter._print_Add (pretty.py:1904): negative-term handling with pretty_negative; Rationals stacked as fractions when any term is taller than one line.
- PrettyPrinter._print_Mul, _print_Pow (root notation, use_unicode_sqrt_char), _print_Integral/_print_Sum/_print_Product (sign sizing), _print_Matrix, _print_Piecewise

**Data structures.**

- stringPict { picture: Vec<String>, baseline, binding }
- prettyForm binding precedence constants
- Unicode glyph tables (greek, sub/superscript maps, bracket pieces, atoms_table)

**Hard dependencies.**

- printing
- core

**Python features that are hard to port.**

- Operator overloading on prettyForm (+ * / **) used as a layout DSL
- Module-level mutable globals for unicode and wrapping
- unicodedata.lookup by character name at import time
- String methods on str subclasses (render returns type(self.picture[0]))

**Behaviour a port must reproduce.**

- Exact multi-line strings with trailing-space and centring rules. center_pad rounding (left = pad//2) must match.
- Width counts codepoints minus combining marks. This is not East-Asian width (unicode-width would give different results) and not grapheme clusters.
- Global state: pretty_symbology._use_unicode (pprint_use_unicode), stringpict._GLOBAL_WRAP_LINE, unicode_warnings. use_unicode=None means 'use the global flag'.
- wrap_line=True with num_columns=None queries shutil.get_terminal_size, which depends on the environment. Tests force wrap_line=False.
- Glyphs are looked up by Unicode name via unicodedata.lookup (U()), falling back to ASCII forms when the lookup fails.
- Symbol rendering: upretty(Symbol('beta_1_2')) == 'β₁ ₂', 'F_1_2^3^4' -> 'F³ ⁴₁ ₂'. Subscripts not representable in Unicode fall back to the '_' text form.
- imaginary_unit must be 'i' or 'j'. Anything else raises ValueError, and a non-str raises TypeError.
- Term order inherited from as_ordered_terms is the same as str (see printing).

**Rust prior art.**

- No direct crate implements 2D math layout. unicode-width and unicode-segmentation (MIT/Apache-2.0) exist but encode different rules from SymPy's.
- SymEngine has no 2D pretty printer

**How its tests check results.** pretty/tests/test_pretty.py: 156 functions, 1238 asserts of exact multi-line strings for both ASCII (pretty(), use_unicode=False) and Unicode (upretty()), all with wrap_line=False. A few tests exercise num_columns wrapping. Only 6 raises.

**Notes.** Port stringpict first; it is S to M on its own. Then port PrettyPrinter methods in step with the node kinds available in core. Not on the app path, but doctests in several modules use pprint.

### `parsing`

2,788 source lines, 75 test functions. Difficulty **L**. Strategy: **redesign**. App relevance: serve.

String-to-expression parsing. sympy_parser.parse_expr (1270 lines) backs sympify(str) and S('...'): Python tokenize, then a list of token transformations, then untokenize and eval(). Also includes parse_mathematica (1085-line hand-written tokenizer and operator-precedence parser to FullForm), parse_maxima (regex then sympify), ast_parser (ast then eval), and sym_expr.SymPyExpression (front end for the C/Fortran parsers).

**Why this strategy.** Porting eval faithfully is impossible and also undesirable. Build a Rust parser over a closed grammar: a lexer compatible with Python tokenize for the needed subset, the 12 transformations re-implemented as token passes (byte-compatible with SymPy's token output), and a Pratt parser for Python expression precedence. It resolves names against a whitelist registry, not `from sympy import *`. It builds a raw unevaluated tree, and evaluation is a separate budgeted step. The PyO3 shim keeps a legacy eval path, opt-in only, for custom transformations or arbitrary global_dict objects. parse_mathematica ports directly because it is already a hand-written precedence parser.

**Key algorithms.**

- stringify_expr (sympy_parser.py:880): tokenize.generate_tokens, then each transformation over (toknum, tokval) lists, then untokenize.
- eval_expr/parse_expr (sympy_parser.py:899-1090): eval(code, global_dict, local_dict). The default global_dict is exec('from sympy import *') plus every builtin function, with max/min mapped to Max/Min. The local_dict null-sentinel bookkeeping is restored afterwards.
- auto_symbol (sympy_parser.py:533): undefined NAME becomes Symbol('n') or Function('n') (when followed by '('). Names in the namespace are kept if they are Basic, a type or callable.
- auto_number (754): int -> Integer, float -> Float('lit'), 'j' suffix -> *I. repeated_decimals (666): '0.[3]' -> Integer+Rational sums. factorial_notation (624). convert_xor (651). rationalize (788). lambda_notation (584). convert_equals_signs (836).
- split_symbols_custom/_token_splittable (364/29), implicit_multiplication/_implicit_multiplication (196), implicit_application/_implicit_application (261), function_exponentiation (312). They use ParenthesisGroup/AppliedFunction grouping (86-195).
- EvaluateFalseTransformer (sympy_parser.py:1105): an ast.NodeTransformer. a-b becomes Add(a, Mul(-1,b)), a/b becomes Mul(a, Pow(b,-1)), with a left/right swap when the left side is unary. Nested Add/Mul are flattened. evaluate=False is added to 36 named functions. Comparison chains become And(...). BitXor maps to 'Not'.
- T/_transformation registry (sympy_parser.py:1225-1270): index-addressable groups ('all', 'implicit' = T[:6]).
- MathematicaParser (mathematica.py): _from_mathematica_to_tokens, _from_tokens_to_fullformlist (precedence table _mathematica_op_precedence), _from_fullformlist_to_sympy (atoms through sympify).

**Data structures.**

- Token stream of (kind, text) with Python token kinds NAME/NUMBER/OP/STRING/ERRORTOKEN
- ParenthesisGroup / AppliedFunction intermediate groupings
- Transformation registry indexed 0..11 with group aliases
- Name-resolution table (whitelist registry replacing the global_dict)
- Mathematica operator precedence table and FullForm list tree

**Hard dependencies.**

- core
- functions.elementary

**Python features that are hard to port.**

- eval/compile/exec of generated Python source; exec('from sympy import *') to build the namespace
- tokenize/untokenize round-trip; ast.NodeTransformer rewriting
- User-supplied callables as transformations; local_dict mutation with a sentinel key
- Python-version-dependent tokenizer behaviour

**Behaviour a port must reproduce.**

- Arbitrary code execution: parse_expr("__import__('os').getpid()") ran and returned an int. parse_expr('().__class__.__mro__') returns Python classes. sympify(str), parse_maxima and ast_parser all reach eval.
- DoS: parse_expr('2**2**2**2**2') computes 2**65536 as an Integer before failing on str conversion. No limits on size, depth or time.
- Namespace capture under auto_symbol: 'E' -> exp(1) (so 'E^x' -> exp(x)), 'I' -> imaginary unit, 'N' -> the function N, 'S' -> SingletonRegistry, 'Q' -> AssumptionKeys, 'beta'/'gamma' -> FunctionClass, 'Integer' -> a class. 'e' stays Symbol('e').
- split_symbols (part of implicit_multiplication_application) silently misreads input. All probed: 'x1' -> Symbol('x')*Number('1') == x; 'x12' -> 12*x; 'x2y' -> 2*x*y; 'xtheta' -> a*e*h*t**2*x. 'x_1' and 'theta' are kept.
- Implicit application probed results: 'sin x^2' -> sin(x**2); 'sin^2 x' raises TypeError unless function_exponentiation is on; 'f(x)' with an undefined f -> f*x; '2(x+1)' -> 2*x + 2 (evaluated).
- sympify(s) == parse_expr(s, standard_transformations + convert_xor), so '^' is power by default. 'x = 2' is a SyntaxError unless convert_equals_signs is on.
- Python-tokenizer semantics are part of the contract: 1_000, 0x1F and '2j' are numbers; '!' is an OP token from Python 3.12 but an ERRORTOKEN before that; unicode identifiers ('α'); unbalanced '(()' raises TokenError.
- Tests depend on Python semantics passing straight through: parse_expr('some_parameter is None', {'some_parameter': None}) is True; pow(38, -1, 97) == 23; local_dict may hold any Python object; transformations are arbitrary user callables on token lists.
- parse_expr('sqrt(50)', evaluate=False) stays sqrt(50); 'x - x' stays '-x + x'; '2/4' stays '2/4'. The evaluate=False AST shape (Mul(-1, b), Pow(b, -1)) is observable through srepr.
- test_parsing_definitions asserts the exact transformation table order and T[...] slicing.

**Rust prior art.**

- ruff_python_parser (MIT, git dependency) and rustpython-parser (MIT, crates.io, superseded) as references for a Python-compatible lexer and number grammar
- logos (MIT/Apache-2.0) lexer; chumsky (MIT) or winnow (MIT) combinators; a hand-written Pratt parser is simplest
- SymEngine's parser (MIT, C++) as an example of a non-eval sympify

**How its tests check results.** Structural == against constructed expressions, including evaluate=False forms (test_sympy_parser.py 29 tests, test_implicit_multiplication_application.py 6). raises(TokenError/SyntaxError/TypeError/ValueError). Mathematica tests compare against sympify of expected strings. test_c_parser.py and test_fortran_parser.py are gated on clang/lfortran and skipped in the pinned venv.

**Notes.** Requirements for a secure parser: (1) A closed grammar. Literals (int/float/imag/underscore/hex/oct/bin), names, calls with positional args and literal-valued kwargs (srepr round-trip needs Symbol('x', positive=True), Float('1.5', precision=53), Function('f')), tuples and lists as call arguments (Matrix([[...]]), Piecewise((a,c),...)), unary and binary ops, comparisons, & | ~, factorial, lambda. No attribute access, subscripts, comprehensions or dunder names; strings only as constructor arguments. (2) Name resolution from a whitelist registry. (3) Limits on input length, nesting depth (no recursion overflow), integer literal digits, integer power bit growth, factorial/binomial arguments, and an evaluation budget. (4) Parse to a raw tree, then canonicalize separately, so evaluate=False parity and form checks work. (5) Typed errors mirroring TokenError/SyntaxError/SympifyError. (6) cargo-fuzz plus differential tests against SymPy on the safe subset. Also add a 'strict' profile that refuses split_symbols misreads ('x1' -> x) for the app.

### `parsing.latex`

5,954 source lines, 0 test functions. Difficulty **L**. Strategy: **redesign**. App relevance: serve.

parse_latex(s, strict, backend). Two backends. ANTLR is the default: LaTeX.g4 (312 lines), a generated 3652-line latexparser.py, and the _parse_latex_antlr.py visitor (607 lines) that builds evaluate=False trees. Lark is experimental and is the one the app uses: lark/grammar/latex.lark (403 lines) with an Earley parser, ambiguity='explicit', and lark/transformer.py (730 lines). Also check_matrix_delimiters regex preflight (__init__.py).

**Why this strategy.** Porting an Earley parser with explicit ambiguity and a mutable rule-name transformer would copy its bugs (ambiguity Trees, tuples) and its slowness. Write one hand-written recursive-descent/Pratt LaTeX parser in Rust with explicit precedence and a vocabulary table. Give it profiles: 'lark-compat' (checked against the 442 pairs, ambiguities resolved deterministically and recorded), 'antlr-compat' (185 pairs), and 'strict' (refuse unknown commands by name, needed by the app). Return the raw unevaluated tree and canonicalize separately.

**Key algorithms.**

- check_matrix_delimiters (parsing/latex/__init__.py): regex scan pairing \begin/\end and \left/\right matrix delimiters, with error messages quoting an index window.
- LarkLaTeXParser (lark/latex_parser.py): Lark(parser='earley', lexer='auto', ambiguity='explicit', keep_all_tokens=True), then TransformToSymPyExpr.transform.
- TransformToSymPyExpr (lark/transformer.py): rule-named methods. adjacent_expressions handles implicit multiplication, a heuristic where a Symbol('d') prefix means a differential, Ket*Bra -> OuterProduct, and tuple -> Derivative. Also superscript (primes/stars/transpose), fraction/_handle_division, normal_integral and integral_with_special_fraction, limit_dir_expr, matrix/determinant/trace/adjugate, log with base, and the trig _power forms.
- ANTLR visitor (_parse_latex_antlr.py): convert_relation/convert_add/convert_mp/convert_unary/convert_postfix_list (implicit multiplication), convert_frac (d/dx derivative detection), convert_func (inverse trig and powers, log bases), handle_integral (differential detection), handle_sum_or_prod, handle_limit. All constructors are called with evaluate=False.

**Data structures.**

- Token kinds (commands, greek, operators, braces, \left/\right pairs)
- Command vocabulary table: name -> {arity, sub/sup handling, builder} or a refusal reason
- Raw parse tree, kept apart from the canonicalized expression

**Hard dependencies.**

- core
- functions.elementary
- concrete
- integrals
- series

**Python features that are hard to port.**

- Lark Transformer dispatch by grammar-rule method name and Token.type strings
- ANTLR-generated code plus visitor; import_module optional-dependency gating
- Global evaluate() context changing outputs
- Regex tables built with f-strings for delimiter checks

**Behaviour a port must reproduce.**

- Lark returns a lark Tree('_ambig', [...]) rather than an Expr for ambiguous input, because the transformer has no _ambig handler. Probed: '\sin x^2' -> _ambig[sin(x**2), sin(x)**2]; 'f(x)' -> _ambig[f(x), f*x]; '\ln\sqrt{x}^{2}' -> _ambig[log(x), log(sqrt(x))**2].
- Lark '\frac{dy}{dx}' returns the Python tuple ('derivative', x), not an Expr. '\frac{d}{dx} x^2' -> Derivative(x**2, x).
- Lark uses evaluating constructors, so results depend on the global evaluate() context: '1+2' -> 3, '3\times 4' -> 12, '\frac{1}{2}+\frac{1}{3}' -> 5/6. Under evaluate(False) these become Add(1,2) and 1/3**1 + 1/2**1. ANTLR always builds evaluate=False trees.
- Lark vocabulary gaps (probed): '\pi' raises UnexpectedCharacters, 'x_{10}' fails (a multi-digit braced subscript), and 'e^{x}' -> Symbol('e')**x, not exp(x). \overline{AB} -> conjugate(A)*conjugate(B). '\log_2 x' -> log(x)/log(2).
- Symbol naming conventions: x_0 -> Symbol('x_{0}') (braces inside the name), h_\theta -> Symbol('h_{theta}'), primes kept in names ("a'_b" -> Symbol("a'_{b}")), \mathit{HELLO world} -> Symbol('HELLO world').
- The ANTLR backend needs antlr4-python3-runtime 4.11. It is NOT installed in the pinned venv (ImportError), so test_latex.py (185 pairs plus BAD_STRINGS) is disabled. The ANTLR oracle has to be installed separately, and the two backends disagree.
- Spacing and ignore rules: \, \; \quad \left \right \displaystyle \limits are ignored. 'strict' exists only for ANTLR.
- Performance: 88-140 ms per parse with Lark and 0.2 s to build the grammar at import, measured on the pinned stack.

**Rust prior art.**

- Parser tooling: pest (MIT/Apache-2.0), lalrpop (Apache-2.0/MIT), logos (MIT/Apache-2.0), chumsky (MIT). Hand-written is recommended.
- LaTeX math tokenization references: latex2mathml (MIT) and pulldown-latex (MIT). Both output MathML, not CAS trees.
- earlgrey (MIT) Earley parser, only if Lark ambiguity behaviour must be reproduced (not recommended)
- antlr-rust (BSD-3-Clause) targets ANTLR 4.8 and is stale. Not recommended.

**How its tests check results.** Table-driven. test_latex_lark.py has 442 (latex, expected) pairs, compared with == inside `with evaluate(False)`. Known failures are skipped by list index (expected_failures = {6, 7}, {0, 3, 4}, ...). test_latex.py (ANTLR, gated) has 185 pairs plus BAD_STRINGS/FAILING_BAD_STRINGS raising LaTeXParsingError. test_custom_latex.py checks custom grammar_file/transformer hooks.

**Notes.** Highest app priority with parsing. The app works around \pi, e, \left/\right and unknown commands, and lists silent misreads to fix: \frac{dy}{dx}, \Delta x, \overline{AB}, compound subscripts, the e^{...} exponent binding, and power on log. A Rust parser should take microseconds. Keep the lark-compat outputs as a measured profile, not as the default meaning.

### `parsing.autolev`

6,601 source lines, 0 test functions. Difficulty **L**. Strategy: **keep in Python on the Rust core**. App relevance: none.

Source-to-source translator from the Autolev dynamics language to Python code that targets sympy.physics.mechanics: Autolev.g4 (118 lines), ANTLR-generated lexer and parser (about 3.7k lines), and _listener_autolev_antlr.py (2083 lines of string-building listener).

**Why this strategy.** It is a niche text-to-text translator with no core dependency. Its output is Python that runs against physics.mechanics through the PyO3 API. Rewriting it in Rust only pays off if a Rust-native mechanics API needs Rust output, which is a separate product decision.

**Key algorithms.**

- AutolevListener (parsing/autolev/_listener_autolev_antlr.py): symbol table with case-folded IDs, type/sign maps, declaration handlers (VARIABLES/MOTIONVARIABLES/SPECIFIED/CONSTANTS with indexed ranges like Q{3}, C{1:2,1:2} and derivative primes), frames/points/particles/bodies writers, Kane's method code emission, include_numeric numeric code section
- parse_autolev (autolev/__init__.py): reads a file or string, runs the ANTLR parser and walker, returns generated Python text

**Data structures.**

- Symbol table dicts (symbol_table, type, sign, var_list, bodies, inertia maps)
- Output line buffer

**Python features that are hard to port.**

- ANTLR-generated listener with dynamic dispatch by context type
- String-built Python code

**Behaviour a port must reproduce.**

- The output is Python text compared line by line (after rstrip) against checked-in .py files. Names (x_d, y_dd, q1..q3), declaration order, and the exact spelling of '_me.dynamicsymbols._t' and '_sm.S(...)' all matter.
- IDs are lowercased. Indexed declarations expand in row-major order (c11, c12, c21, c22).
- There is no runtime dependency on the SymPy core. The listener only concatenates strings, and the emitted code imports sympy.physics.mechanics, sympy, math and numpy.

**Rust prior art.**

- antlr-rust (BSD-3-Clause, stale) or a hand-written parser for a 118-line grammar

**How its tests check results.** test_autolev.py: 12 rule tests (ruletest1-12 .al -> .py) and 4 pydy examples, compared line by line as text. All are skipped when antlr4 is missing, which is the case in the pinned venv.

**Notes.** Keeping it costs about S: install antlr4-python3-runtime 4.11 in the oracle venv so its 16 golden tests actually run. A Rust port is L, roughly 4-8 weeks, because the 2k-line listener carries a lot of state.

### `parsing.c`

1,060 source lines, 0 test functions. Difficulty **M**. Strategy: **keep in Python on the Rust core**. App relevance: none.

parse_c / SymPyExpression for C. It turns C source into sympy.codegen.ast nodes (Variable, Declaration, FunctionDefinition, While, aug_assign, Pre/PostIncrement...) by walking libclang's AST through the clang.cindex Python bindings (c_parser.py, 1059 lines).

**Why this strategy.** It is a thin adapter over libclang that only needs the codegen AST constructors exposed through PyO3. A Rust port would need FFI to libclang (only allowed in an FFI crate) or a new C front end, for little value.

**Key algorithms.**

- CCodeConverter.transform dispatch by cursor kind (c_parser.py:196 onward): var_decl, function_decl, parm_decl, integer/floating/string/character literals, call_expr, compound_stmt, decl_stmt, unary/binary/compound-assignment operators, while_stmt
- Binary operators are rebuilt from the token stream with an explicit precedence table (priority_of/perform_operation, c_parser.py:939-1006), because cindex does not expose operator kinds
- Literal and type mapping to codegen types (intc, int8..uint64, float32/64/80, bool_)

**Data structures.**

- Clang cursor tree; codegen AST output

**Hard dependencies.**

- codegen
- core
- logic

**Python features that are hard to port.**

- Dynamic method lookup 'transform_' + cursor kind name
- clang.cindex Python bindings

**Behaviour a port must reproduce.**

- Needs libclang at runtime. Behaviour depends on the clang version's cursor kinds and token spelling.
- Operator precedence is re-derived from tokens. Mistakes there change the tree shape that tests compare structurally.
- Supported subset only: declarations, assignment, functions, calls, return, while, ++/--, compound assignment.

**Rust prior art.**

- clang + clang-sys (Apache-2.0) bindings to libclang (Apache-2.0 WITH LLVM-exception); FFI
- tree-sitter + tree-sitter-c (MIT); lang-c (MIT/Apache-2.0, verify) as pure-Rust C parsers

**How its tests check results.** test_c_parser.py: 5248 lines of structural == on codegen AST nodes. Fully gated on clang.cindex, so skipped in the pinned venv.

**Notes.** Defer. To get an oracle, install libclang Python bindings in a second venv. Otherwise these tests never run.

### `parsing.fortran`

348 source lines, 0 test functions. Difficulty **M**. Strategy: **defer or drop**. App relevance: none.

src_to_sympy for Fortran (fortran_parser.py, 347 lines). It walks an LFortran ASR (lfortran.asr) with ASTVisitor and emits codegen AST: Variable/Declaration/Assignment/FunctionDefinition, BinOp -> Add/Mul/..., Num, Sequence.

**Why this strategy.** Its only dependency is a stale external Python API, the feature scope is tiny, and it has no users in the app. Re-implementing it needs a Fortran front end. Revisit if there is demand, using tree-sitter-fortran.

**Key algorithms.**

- ASR2PyVisitor.visit_* (fortran_parser.py:68-300): TranslationUnit, Assignment, BinOp, Variable (type mapping to intc/float32...), Sequence, Num, Function
- call_visitor / src_to_sympy (fortran_parser.py:302-347)

**Data structures.**

- LFortran ASR nodes; codegen AST output

**Hard dependencies.**

- codegen
- core

**Python features that are hard to port.**

- Visitor dispatch by node class name over a foreign AST

**Behaviour a port must reproduce.**

- Depends on the old `lfortran` Python package API (lfortran.asr, asr_to_ast). This is very likely not maintained alongside current LFortran (unverified), so the test oracle may not be installable.
- Only a tiny subset is supported (declarations, assignments, arithmetic, simple functions).

**Rust prior art.**

- tree-sitter-fortran (MIT); LFortran itself (BSD-3, C++) as an external tool

**How its tests check results.** test_fortran_parser.py: structural == on codegen AST nodes. Gated on lfortran, so skipped in the pinned venv.

**Notes.** Lowest priority in the cluster.

### `codegen`

4,468 source lines, 105 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: none.

Language-neutral code-generation AST plus helpers. ast.py (1906 lines): Token base, Assignment and augmented assignments, CodeBlock with topological_sort/cse, For, While, Scope, Variable/Pointer/Element/Declaration, Type/IntType/FloatType/ComplexType with numeric limits, FunctionPrototype/Definition/Call, Return, Print. Also cnodes/fnodes/cxxnodes/pynodes/numpy_nodes/scipy_nodes/matrix_nodes, C99 math functions (cfunctions.py: expm1, log1p, exp2, log2, fma, log10, Sqrt, Cbrt, hypot, isnan, isinf), rewriting.py optimizations, approximations.py (SumApprox/SeriesApprox) and algorithms.py (Newton's method AST generators). Printed by the code printers in printing.

**Why this strategy.** An AST of tagged nodes maps naturally to Rust enums and structs. The surface is small and well tested, and the code printers depend on it. Put it in its own crate that depends only on core. Code printers then depend on codegen, and codegen depends on printers only in tests.

**Key algorithms.**

- Token.__new__ (codegen/ast.py:168-330): _fields-driven construction with _construct_<attr> converters, defaults, not_in_args, pass-through of Dummy; _sympystr via printer_context with indentation
- CodeBlock.topological_sort and CodeBlock.cse (ast.py:666, 746): dependency ordering through utilities.iterables.topological_sort, then simplify.cse with re-sorting
- FloatType limits (ast.py:1195-1314): max_exponent, min_exponent, max, tiny, eps, dig, decimal_dig as exact SymPy numbers; Type.cast_check raising on overflow or precision loss
- rewriting.optimize with ReplaceOptim/FuncMinusOneOptim (codegen/rewriting.py:52-357): Wild-pattern replace with a cost_function; optims_c99/optims_numpy/optims_scipy sets; create_expand_pow_optimization; matinv_opt (uses ask(Q.fullrank))
- approximations.SumApprox/SeriesApprox (codegen/approximations.py): bounds analysis with calculus.singularities.is_increasing/is_decreasing and series truncation
- algorithms.newtons_method/newtons_method_function (codegen/algorithms.py): builds While/Assignment AST with optional debug Print

**Data structures.**

- Token node structs with ordered _fields and defaults
- Type hierarchy (IntBaseType/Signed/Unsigned/FloatType/ComplexType) carrying nbits/nmant/nexp
- CodeBlock with left_hand_sides/free symbols
- Optimization objects (pattern, replacement, cost function)

**Hard dependencies.**

- core
- functions.elementary
- printing
- assumptions
- matrices
- calculus
- series

**Python features that are hard to port.**

- Class-attribute metaprogramming (_fields, defaults, _construct_<attr> lookup by getattr)
- __slots__ plus Basic subclassing with Python str payloads
- Wild properties as Python lambdas in rewriting rules

**Behaviour a port must reproduce.**

- Fields listed in not_in_args are excluded from .args, which changes ==/hash. `arg == None` treats NoneToken as None.
- String/QuotedString/Comment are Atom Tokens holding Python str, so non-Basic payloads live in the tree.
- FloatType('f16', 16, nmant=10, nexp=5).max == 65504 is exact; dig uses floor(nmant*log(2)/log(10)).
- Rewriting outcomes depend on Wild matching order and Expr.count cost functions. Results are compared structurally.
- cfunctions follow SymPy Function auto-evaluation: expm1(0) -> 0; _eval_rewrite_as_* and fdiff are tested.
- Emitted code order comes from topological_sort and cse; tests compare printed code.

**Rust prior art.**

- cranelift (Apache-2.0 WITH LLVM-exception) as a JIT target replacing printing/llvmjitcode; inkwell (Apache-2.0) LLVM bindings (FFI only)

**How its tests check results.** 105 tests. Structural == on AST nodes, printed-code string compares via ccode/fcode/pycode, numeric checks. test_applications/test_algorithms/test_rewriting/test_fnodes/test_matrix_nodes compile and run generated C/Fortran (or need numpy/scipy) through sympy.utilities._compilation and are skipped without compilers.

**Notes.** Related pieces outside this unit, in utilities: lambdify.py (1592 lines), the older codegen.py (2237: make_routine/CCodeGen/FCodeGen/RustCodeGen) and autowrap.py (1178). lambdify generates Python source through pycode/numpy printers and exec()s it, so keep lambdify in Python on top of Rust printers that emit identical strings. Separately, offer a native Rust compiled numeric evaluator (bytecode VM over f64/complex, with Cranelift JIT optional).

### `interactive`

1,098 source lines, 8 test functions. Difficulty **S**. Strategy: **keep in Python on the Rust core**. App relevance: none.

Interactive-session glue. init_printing (printing.py) installs sys.displayhook or IPython formatters (plain/LaTeX/PNG/SVG through preview or matplotlib mathtext) and sets Printer global settings. init_session (session.py) runs the preexec imports, can wrap int literals with Integer via an AST/token rewrite (int_to_Integer), and can auto-create Symbols on NameError. interactive_traversal (traversal.py) is a console expression browser.

**Why this strategy.** This is pure Python/IPython integration that calls into printing. It stays a thin Python layer over the PyO3 module. Rust users could get an evcxr Jupyter integration later.

**Key algorithms.**

- _init_ipython_printing (interactive/printing.py:33): registers formatters per type with for_type, _can_print container recursion, latex rendering through preview (dvipng/dvisvgm) or matplotlib
- int_to_Integer (session.py:85): tokenize pass wrapping NUMBER ints in Integer(...)
- enable_automatic_symbols (session.py:161): custom exception handler re-runs a cell after defining the missing Symbol

**Data structures.**

- IPython formatter registrations; settings passed through to printers

**Hard dependencies.**

- printing
- core

**Python features that are hard to port.**

- Monkeypatching IPython shell methods and sys.displayhook
- tokenize-based source rewriting

**Behaviour a port must reproduce.**

- Mutates global state: Printer._global_settings, Printable._repr_latex_ (or _repr_disabled), sys.displayhook, IPython run_cell.
- The auto-symbols handler re-executes the whole cell, which can repeat side effects.

**Rust prior art.**

- pyo3 (MIT/Apache-2.0); evcxr (MIT/Apache-2.0) Jupyter kernel for a future Rust-native REPL

**How its tests check results.** 8 tests. test_interactive.py does a string compare of int_to_Integer output. test_ipython.py needs IPython (skipped in the pinned venv).

**Notes.** About 1 person-week to adapt to the PyO3 module once printers and Printable hooks exist.

### `plotting`

10,530 source lines, 134 test functions. Difficulty **L**. Strategy: **keep in Python on the Rust core**. App relevance: none.

Plotting front end and data series. plot.py (plot, plot_parametric, plot3d*, plot_contour, PlotGrid). series.py (2591 lines): LineOver1DRange, Parametric2D/3D, Surface, Contour, ImplicitSeries; uniform and adaptive sampling, pole detection, exclusion points; evaluates through lambdify with numpy. Backends: matplotlib, text. textplot (ASCII). plot_implicit with intervalmath (interval arithmetic) and experimental_lambdify. Legacy pygletplot (OpenGL).

**Why this strategy.** The value lies in matplotlib/numpy integration, which belongs in Python. Rust should supply the hot path: a fast compiled numeric evaluator for expressions (vectorized f64/complex) and symbolic singularity detection. textplot (S) and intervalmath (M) can move to Rust cheaply. Drop pygletplot (legacy OpenGL).

**Key algorithms.**

- BaseSeries._evaluate / lambdify wrapping (plotting/series.py:356-400): lambdify(modules=numpy) with a fallback to the 'sympy' module (dummify=True) on failure
- LineOver1DRangeSeries._adaptive_sampling_helper (series.py:1382): recursive midpoint flatness test with a random point in [0.45, 0.55] (np.random.rand), depth limit, log-scale handling
- _detect_poles_numerical_helper / _detect_poles_symbolic_helper (series.py:931-990): gradient threshold or calculus singularities
- ImplicitSeries with intervalmath (plotting/intervalmath/interval_arithmetic.py, lib_interval.py) and experimental_lambdify (exec of generated code) for adaptive implicit plots
- textplot_str (plotting/textplot.py): sampling with lambdify, rescale into a W x H character grid
- Plot/PlotGrid object model and backend registry plot_backends = {'matplotlib', 'text'}

**Data structures.**

- Series objects (expr, ranges, sampling params, cached numeric data)
- Plot/PlotGrid containers; backend objects
- Interval (start, end, is_valid) for intervalmath

**Hard dependencies.**

- core
- printing
- sets
- calculus

**Python features that are hard to port.**

- numpy vectorization and broadcasting, np.random
- lambdify/experimental_lambdify exec of generated code
- Backend registry dict and duck-typed backend objects; matplotlib artists

**Behaviour a port must reproduce.**

- Adaptive sampling is nondeterministic (np.random.rand at series.py:1404 and 1679), so tests cannot compare exact arrays there.
- Backend choice depends on whether matplotlib is importable, otherwise text.
- textplot output is an exact ASCII grid compared line by line. Its rounding and rescale must match.
- intervalmath semantics: interval objects carry is_valid tri-state (True/False/None) used by plot_implicit refinement.
- There are many deprecated-but-supported keyword paths in plot.py (documented as 'considered deprecated').

**Rust prior art.**

- plotters (MIT) for native SVG/bitmap backends; textplots (MIT) terminal plots
- inari (MIT) interval arithmetic for plot_implicit (check that optional GMP features stay off)

**How its tests check results.** About 134 tests. Most need matplotlib/numpy and are skipped in the pinned venv. test_series compares numpy arrays with tolerance. test_textplot compares exact line lists. The intervalmath tests are numeric interval checks.

**Notes.** The app does not use sympy.plotting: its graph_plot grader uses .subs at lattice points. It does need fast numeric sampling (inequality_2d sample points, figure lint), which the Rust numeric evaluator also serves. A full port is L; adapting it to the Rust core is M.

## Physics, vectors, tensors, other domains

### Suggested port order inside the cluster

1. 1. physics.units: the app's quantity grader uses it in the serve path. It needs only core arithmetic, Rational/Float, sqrt/pi, printing hooks and a tiny exact Gauss-Jordan solver. Its data tables can be generated from the oracle.
1. 2. physics.quantum slice (state.Bra/Ket, InnerProduct, OuterProduct, transforms.py postprocessors): the Lark parse_latex transformer constructs these. It is also the first real client of the core's Mul postprocessor and kind hooks, which keeps that core design honest.
1. 3. liealgebras: tiny and self-contained (Matrix + Rational); an onboarding task to shake down the oracle harness.
1. 4. tensor.indexed + index_methods, then tensor.array (NDimArray, arrayop): other clusters consume them (codegen, printers, concrete Sum, lambdify, Derivative's ArrayDerivative dispatch, stats), so they cannot wait until the end.
1. 5. physics.vector: the foundation for mechanics and biomechanics; needs Function/Derivative, dense matrices, trigsimp and printer subclassing.
1. 6. physics.mechanics, then physics.biomechanics: thin layers over vector and matrices. Biomechanics also waits for the code printers and lambdify.
1. 7. physics top-level wigner.py, small modules and paulialgebra: wigner is a prerequisite for quantum cg/spin; paulialgebra needs the nc and operator-overload semantics settled.
1. 8. sympy.vector: needs core support for Add/Mul-derived node kinds and the Vector Add postprocessor, plus solve/integrate/diophantine.
1. 9. tensor.array.expressions: ship together with matrices.expressions calculus (MatrixExpr.diff goes through it).
1. 10. diffgeom: after solve, simplify and tensor.array.
1. 11. physics.control: after polys roots/CRootOf parity, solveset, limit, linodesolve and inverse Laplace; matplotlib plots deferred.
1. 12. physics.continuum_mechanics: after integrate on SingularityFunction/Piecewise, linsolve, dsolve and units; plotting deferred.
1. 13. physics.optics: after geometry 3D (Ray3D, Plane), units and quantum TensorProduct.
1. 14. tensor.tensor + toperators (needs combinatorics perm_groups and tensor_can from the combinatorics cluster), then physics.hep (Kahane algorithm).
1. 15. rest of physics.quantum (operators, qapply, represent, spin/cg, gates/qubits/circuits, qasm): the largest unit; needs the pretty printer and wigner.
1. 16. physics secondquant.py: last in the physics unit, because its canonical dummy ordering keys on exact str-printer output.
1. 17. categories: lowest priority and niche; can run as Python on the Rust core until ported.

### Risks

- Core extension points must be designed before any of quantum, sympy.vector, tensor or matrix-expression calculus is ported: constructor postprocessors on Add/Mul/Pow, the Mul kind dispatcher, custom expand hints, Add.flatten special cases, and node kinds that reuse Add/Mul flattening (VectorAdd is an Add). SymEngine shows the cost of retrofitting a closed core.
- Exact-form coupling to other clusters: control depends on roots/CRootOf ordering, continuum_mechanics on integrate(SingularityFunction)/dsolve forms, mechanics on LUsolve pivoting, and diffgeom/vector on the first solve() solution. These units cannot reach parity before their upstream clusters do.
- Printing parity dominates the tests: quantum (246 printing asserts in one file), tensor (dummy names in str), secondquant (repr/latex plus KeyPrinter-based sorting), categories (Xy-pic strings), biomechanics (code-printer strings). The str, pretty, latex and code printers must be byte-exact first.
- Global mutable registries make tests order-dependent and need an explicit Rust design: physics.units class dicts, TensorManager, physics.vector frame caches and the Vector.simp flag, the wigner factorial cache, the tensor .data substitution dict.
- Optional numpy/scipy/matplotlib/jax paths (quantum represent formats, identitysearch, control plots, beam plots, biomechanics lambdify tests) skip without those packages, so missing behaviour can hide. Decide drop versus PyO3 bridge per path.
- Volume versus value: this cluster is about 30k code lines excluding docstrings and about 43k test lines. Apart from units and a small quantum slice, the app does not use it. It must be scheduled late without letting the core design ignore it.
- Matching units for parity conflicts with app strictness: SymPy's convert_to silently no-ops on a dimension mismatch, _collect_factor_and_dimension has bug-compatible outputs (exp(meter) -> (E, length)), and there are no affine temperatures. The Rust API should offer a parity mode plus a strict mode for the app, kept outside the oracle surface.
- Test inventory under-counts pytest-class tests: biomechanics shows 0 test functions but has about 215 test methods, so planning and coverage tooling must count methods.

### What other clusters and the core design must know

- Core constructor postprocessors: Basic._constructor_postprocessor_mapping is consulted in AssocOp.__new__ (core/operations.py:98,111) and Pow.__new__ (core/power.py:229) and keyed by the class of each direct argument. Registrants: MatrixExpr (matrices/expressions/matexpr.py:525), Vector (vector/vector.py:432), StateBase/TensorProduct/Operator (physics/quantum/transforms.py:279-291) and TensExpr (tensor/tensor.py:5263). The postprocessors are collected in a Python set (unordered). The Rust core needs a typed hook registry for Add/Mul/Pow keyed by node kind.
- Add.flatten (core/add.py:217,277) imports tensor.tensor and special-cases TensExpr via TensAdd(o, coeff).doit(deep=False), alongside MatrixExpr and AccumBounds. Core has explicit knowledge of tensor kinds.
- Kind dispatch: Mul._kind_dispatcher registrations come from matrices/kind.py:73,91, vector/kind.py:58 and physics/quantum/kind.py:88-104 (KetKind/BraKind/OperatorKind); custom KindDispatchers exist in quantum TensorProduct/Commutator/AntiCommutator. The Rust kind lattice must be open, or a closed enum that already includes Matrix, Vector, Ket, Bra, Operator, Array and Tensor kinds.
- Noncommutative support the core must provide: is_commutative as a fuzzy flag on every node; a Mul layout of {coeff, sorted commutative part, ordered nc part}, with adjacent equal nc bases merged into powers (A*A -> A**2, A**2*A**-1 -> A) and no distribution of nc powers ((A*B)**2 stays); args_cnc; Mul._eval_subs subsequence substitution ((A*B*C).subs(A*B, x)); Mul._matches_noncomm for nc Wild (core/mul.py:1080); factor_nc/_mask_nc (core/exprtools.py:1274,1398); nc_simplify (simplify/simplify.py:1645); expand with order-preserving multinomials; printing puts commutative factors first.
- Binary operators differ from constructors: Pauli, Prefix, Dimension, TransferFunction, RayTransferMatrix, physics.vector.Vector, tensors and NDimArray override __mul__ etc., guided by _op_priority (Prefix/Dimension 13.0, Vector 12.0, VectorZero 12.1, Dyadic 13.0, DyadicZero 13.1, TensExpr 12.0, NDimArray 11.0). Pauli(1)*Pauli(2) == I*sigma3, but Mul(Pauli(1), Pauli(2)) stays sigma1*sigma2. The PyO3 layer and the Rust operator traits must reproduce _op_priority dispatch separately from canonical constructors.
- Custom expand hints: Expr.expand dispatches to methods named '_eval_expand_' + hint (core/expr.py:3703); quantum adds commutator and tensorproduct hints. The Rust expand needs an extensible hint registry.
- simplify() calls physics.units.util.quantity_simplify whenever expr.has(Quantity) (simplify/simplify.py:733), so the simplify cluster carries a units hook. For the app, simplify(a - b) on unit expressions is unit-aware: simplify(km/hour - meter/second) -> -13*kilometer/(5*hour).
- The Lark LaTeX parser (the app's serve path) imports sympy.physics.quantum on the first implicit multiplication (parsing/latex/lark/transformer.py:191 adjacent_expressions; also bra/ket/inner_product at :521-531), which also registers the quantum Mul postprocessors. parse_latex(r'\langle a|b\rangle', backend='lark') returns InnerProduct. The ANTLR backend imports Bra/Ket at module top (parsing/latex/_parse_latex_antlr.py:8). The parsing cluster needs Bra/Ket/InnerProduct/OuterProduct node kinds, or must refuse bra-ket syntax by name.
- tensor is not a leaf. core/function.py _derivative_dispatch (~L1933) sends any diff involving MatrixBase/MatrixExpr/NDimArray/list to tensor.array.array_derivatives.ArrayDerivative. MatrixExpr.diff, Trace, MatPow, HadamardProduct and applyfunc derivatives route through tensor.array.expressions (matrices/expressions/matexpr.py:460,540-551; trace.py:60; hadamard.py:124,433; matpow.py:106; applyfunc.py:138). Indexed/IndexedBase are imported by codegen, all code printers (printing/codeprinter.py, c/rust/julia/octave/rcode/pycode/llvmjitcode), concrete/expr_with_limits.py, integrals/integrals.py, utilities/codegen.py/autowrap.py, stats and series/sequences.py; core/sympify.py imports tensor.array.Array.
- tensor.tensor canon_bp hard-depends on combinatorics.tensor_can.canonicalize (Butler-Portugal, 442 code lines) and on perm_groups/Schreier-Sims. diffgeom and array expressions use Permutation. The combinatorics cluster must deliver these with exact BSGS conventions.
- Sort keys embed Python class-name strings: Quantity.class_key() == (2, 0, 'Quantity') ties with Symbol (2, 0, 'Symbol') and is broken by name, so str(meter*x) == 'meter*x' while Mul.args == (x, meter) (Basic.compare uses a different ordering). Every node type in this cluster needs its SymPy class name and class_key in the Rust ordering tables.
- Matrix.solve(method='GJ') semantics feed physics.units.convert_to: a tall consistent system solves, while wide, parametric or inconsistent systems raise NonInvertibleMatrixError and the expression is returned unchanged (matrices/solvers.py ~L827-838).
- Unevaluated construction is part of tested output: physics.control to_expr builds Mul(num, Pow(den, -1, evaluate=False), evaluate=False). Biomechanics doit(evaluate=False) keeps UnevaluatedExpr inside exp. The core's raw (non-evaluating) constructors must be public and printable.
- Physics code subclasses core containers and atoms: BasisDependentAdd(Add), BasisDependentMul(Mul), TensAdd/TensMul(AssocOp), Pauli(Symbol), Quantity/BaseScalar/BaseVector(AtomicExpr), RayTransferMatrix(MutableDenseMatrix), and CharacteristicCurveFunction(Function) with eval/fdiff/doit/_eval_evalf/inverse/_print_code. The Rust Expr model needs extensible node kinds that reuse Add/Mul/Function machinery and printer hooks.
- Name-based dynamic dispatch in quantum (qexpr.dispatch_method uses '%s_%s' % (basename, arg.__class__.__name__), plus _apply_operator_<Class>, _represent_<Basis>, _eval_innerproduct_<Bra>) needs typed double-dispatch tables in Rust and string-name shims in PyO3.
- mpmath leaks into this cluster: physics.control imports mpmath.libmp.libmpf.prec_to_dps, liealgebras uses mpmath.fac (so WeylGroup('A3').group_order() == 24.0 is a float). The Rust float layer must reproduce both.
- parsing.autolev (another cluster) emits code against physics.mechanics and physics.vector, so it is blocked on those ports.

### `physics.units`

3,346 source lines, 72 test functions. Difficulty **M**. Strategy: **redesign**. App relevance: serve.

Dimensional analysis and symbolic unit conversion: Dimension/DimensionSystem (base and derived dimensions with dependency maps), Quantity/PhysicalConstant atoms, Prefix, UnitSystem registry (SI, MKS, MKSA, cgs_gauss, natural, length_weight_time), convert_to, quantity_simplify, check_dimensions, find_unit, plus about 400 unit and constant definitions (unit_definitions.py has 120 Quantity(...) calls and 67 set_global_* calls; si.py adds prefixed units through prefix_unit at import). About 2.2k code lines excluding docstrings, much of it data.

**Why this strategy.** The maths is small: exponent-vector linear algebra plus scale-factor products. The Python design does not port well: class-level mutable dicts as global registries, Dimension as an Expr with overridden operators, and a module namespace scanned by find_unit. A Rust version should use static tables generated from SymPy and checked against the oracle, typed dimension vectors, and an explicit registry/context with a copy-on-write overlay for user set_global_* calls. A PyO3 shim exposes a process-global context for API parity. Printed and structural outputs must stay identical.

**Key algorithms.**

- DimensionSystem._get_dimensional_dependencies_for_name (dimensions.py ~L330-385): recursive reduction of a Dimension's name expression to {base Dimension: exponent}. Mul sums exponents. Add requires identical maps, else TypeError. Pow allows only a dimensionless or Symbol exponent. A Function's args must be dimensionless, or for TrigonometricFunction dimensionless or angle.
- _QuantityMapper.get_quantity_dimension / get_quantity_scale_factor (dimensions.py ~L85-115): layered lookup: unit-system map, then dimension-system map, then the class-global map, then the dimensional-equivalence chain (recursive through the reference unit), falling back to Dimension(quantity.name). Scale factors multiply along the chain of relative scale factors.
- UnitSystem.get_dimensional_expr and _collect_factor_and_dimension (unitsystem.py ~L130-200): structural recursion over Mul/Pow/Add/Derivative/Function/Quantity giving a dimension expression or a (scale factor, Dimension) pair; Add checks equivalent_dims.
- util._get_conversion_matrix_for_expr + convert_to (util.py ~L20-130): build the exponent matrix of the target units over canonical dimensions and solve it with Matrix.solve (Gauss-Jordan over Q). If the expression's dimensions are not a subset, or the system is singular or underdetermined, the input comes back unchanged. Adds and Pow-of-Add are converted per term; nested Quantities are replaced first.
- quantity_simplify (util.py ~L135-195): replace prefixes with their numbers, then rewrite every quantity of a dimension in terms of the first one in ordered() order, with an optional across_dimensions lookup into UnitSystem.derived_units. simplify() calls it whenever expr.has(Quantity) (simplify/simplify.py:733).
- check_dimensions (util.py ~L200-265), find_unit (units/__init__.py ~L200-268: scans dir(module) and sorts by (len, name)), prefix_unit (prefixes.py ~L125-152), and Prefix.__mul__/__truediv__, which combine prefixes by scanning the PREFIXES table

**Hard dependencies.**

- core (AtomicExpr/Expr node kinds, Mul/Pow/Add canonicalization, Rational and 53-bit Float, replace/xreplace/atoms/has, default_sort_key/ordered)
- matrices (Matrix.solve with the GJ method on small rational matrices; a units-local exact solver can replace it if outcomes match)
- functions.elementary (sqrt, pi, the TrigonometricFunction class test, Abs)
- printing (str/latex/pretty hooks for Quantity/Prefix/Dimension)

**Python features that are hard to port.**

- class-level mutable dicts as global registries (_QuantityMapper._quantity_*_global, UnitSystem._unit_systems)
- `cls._is_prefixed = is_prefixed` HACK in Quantity.__new__ mutates a class attribute
- operator overloading with _op_priority=13 on Prefix and Dimension (binary * and / differ from Mul(...))
- find_unit introspects dir(sympy.physics.units)
- import-time generation of prefixed Quantities (prefix_unit) and module-level alias chains (m = meter = meters)

**Behaviour a port must reproduce.**

- convert_to returns the input unchanged on a dimension mismatch, with no error (convert_to(5*meter, second) -> 5*meter). It is also not symmetric: convert_to(newton*meter, joule) -> joule, but convert_to(joule, newton*meter) -> meter*newton.
- kilo*meter evaluates to 1000*meter (a Mul), not kilometer. A Prefix times a Prefix searches PREFIXES for a matching scale factor. All six binary prefixes have abbrev 'Y' (an upstream bug that must be reproduced if the oracle exposes it).
- Quantity is positive, real, nonzero and commutative, with empty free_symbols and _diff_wrt=True: sqrt(meter**2) == meter, diff(meter**2, meter) == 2*meter, Abs(q) == q. Quantity._eval_subs keeps a Quantity when the replacement is a different Quantity.
- Equality goes by args (name, abbrev): Quantity('v') == Quantity('v'), so set_global_relative_scale_factor on one silently redefines the other through class-level dicts. Tests rely on this global mutation (test_quantities.test_mul_div).
- Dimension algebra is deliberately non-algebraic: -length == length, 3*length == length, length+length == length. length+time builds a plain Add that only fails later in get_dimensional_dependencies. Exponents may be Rational (cgs_gauss uses S.Half) or even a Symbol.
- _collect_factor_and_dimension(exp(meter)) returns (E, Dimension(length)) without error (the Function branch returns (func(*factors), *dims)). Bug-compatible behaviour is needed for parity.
- Constants are Python float literals turned into 53-bit Floats (1.602176634e-19, 1.66053906660e-24, 9.80665), so convert_to(12.0*km/hour, meter/second) -> 3.33333333333333*meter/second, and Float digits show in outputs.
- Ordering: Quantity.class_key() is (2, 0, 'Quantity') and sorts against Symbol by the class-name string, so str(meter*x) is 'meter*x' while Mul args are (x, meter). The Rust ordering tables must carry these class names.
- srepr(meter) prints 'meter', and Dimension prints as 'Dimension(length, L)'. `from sympy.physics.units import *` shadows N (newton), m, s, g, c, G.
- SymPy has no affine units (no degC/degF) and no offset conversion.

**Rust prior art.**

- uom (MIT OR Apache-2.0): compile-time typed units; useful only to cross-check definitions, since symbolic dynamic units are needed
- dimensioned (MIT OR Apache-2.0): type-level dimensions, reference only
- rink-core (MPL-2.0, reference only, not linkable under the licence policy): runtime unit parsing and conversion ideas for the app's unit-string front end

**How its tests check results.** 72 tests, 1225 lines. Mostly structural == on convert_to results (convert_to(mile, kilometer) == 25146*kilometer/15625), dict equality of dimensional dependencies, exact find_unit lists, str/latex compares, raises(ValueError/TypeError). Tests mutate the global registries (set_global_relative_scale_factor, SI.set_quantity_dimension), so test order matters.

**Notes.** Proposed Rust design. Quantity and Prefix become interned atom variants in the expression DAG (QuantityId into a table of {name, abbrev, latex, pretty, is_prefixed, is_physical_constant}). Dimension stays an Expr node that keeps its name expression for printing and equality. DimensionSystem computes SmallVec<(BaseDimId, Exponent)>, where Exponent is a Rational fast path with an Expr fallback. convert_to gets a dedicated exact Gauss-Jordan over Q for matrices of at most 7 x n, which must reproduce Matrix.solve(GJ) outcomes: a tall consistent system succeeds; a wide, singular or inconsistent one means no conversion. App needs: the quantity grader must check dimensions itself through get_dimensional_expr + get_dimensional_dependencies, because convert_to fails silently. simplify(a - b) on unit expressions already goes through quantity_simplify (simplify(km/hour - meter/second) -> -13*kilometer/(5*hour)). degC/degF would be an extension outside the oracle. Note that importing sympy.physics.units runs physics/__init__.py, which also imports physics.matrices. Estimate: 2-3 person-weeks once core arithmetic and printing exist.

### `physics.quantum`

16,457 source lines, 298 test functions. Difficulty **XL**. Strategy: **port faithfully**. App relevance: serve.

Dirac-notation quantum mechanics on noncommutative expressions: states (Ket/Bra/TimeDep/Orthogonal/Wavefunction), operators (Operator/Hermitian/Unitary/Identity/OuterProduct/DifferentialOperator), InnerProduct, Dagger, Commutator/AntiCommutator, TensorProduct, HilbertSpace algebra, qapply, represent, density matrices, partial trace, spin (J operators, WignerD, coupled/uncoupled states), Clebsch-Gordan objects with cg_simp, SHO/particle-in-box/cartesian X/P, boson/fermion ops and normal ordering, Pauli operators, qubits/gates/circuits (QFT, Grover, Shor, gate simplification, identity search, QASM parser, circuit plotting). 38 files, about 7.7k code lines excluding docstrings.

**Why this strategy.** The algorithms are domain-specific and have no Rust prior art, so the oracle-driven port should stay structurally close. Python name-based dispatch becomes typed double-dispatch tables. Two parts should be deferred or dropped: circuitplot (matplotlib) and the numpy/scipy.sparse represent formats (expose them through PyO3 if needed). The Bra/Ket/InnerProduct/OuterProduct slice must come early for the LaTeX parser.

**Key algorithms.**

- qapply / qapply_Mul (qapply.py): expand(commutator=True, tensorproduct=True), split with args_cnc, then reduce right-to-left. Pow peeling, OuterProduct splitting, Commutator.doit expansion, TensorProduct pairwise application, Sum distribution, then lhs._apply_operator(rhs) or rhs._apply_from_right_to(lhs), falling back to InnerProduct(Bra, Ket).
- transforms.py (new in 1.14, marked experimental): Mul and Pow constructor postprocessors registered for StateBase, Operator and TensorProduct. _postprocess_state_mul walks adjacent pairs through the multipledispatch _transform_state_pair (Bra*Ket -> InnerProduct, Ket*Bra -> OuterProduct, OuterProduct*OuterProduct -> InnerProduct*OuterProduct). Pow of Bra/Ket raises TypeError; TensorProduct**n distributes.
- Commutator/AntiCommutator.eval (commutator.py, anticommutator.py): canonical argument ordering with a sign, and _eval_expand_commutator (Leibniz expansion) as a custom expand hint.
- TensorProduct (tensorproduct.py): matrix Kronecker product (matrix_tensor_product), _eval_expand_tensorproduct, and a KindDispatcher for kinds.
- represent / rep_innerproduct / rep_expectation / integrate_result (represent.py): basis-dependent matrix representation through name-built dispatch (_represent_<BasisClass>), with integration over continuous bases.
- operatorordering.normal_ordered_form / normal_order: boson/fermion reordering with a recursion limit.
- cg.py: Wigner3j/CG/Wigner6j/Wigner9j objects; cg_simp applies Varshalovich identities (8.7.1, 8.7.2, 8.7.4, 8.7.9) by matching term lists inside Add and Sum.
- spin.py: J operators acting in Jx/Jy/Jz bases through WignerD rotations; couple/uncouple expand through CG coefficients (_couple, _uncouple, _build_coupled).
- gate.py/qubit.py: gate matrices in the z basis (represent_zbasis), gate_simp/gate_sort (commutation rules), measure_all/measure_partial. qft.py, grover.py, shor.py (period finding with igcd and continued fractions). circuitutils.py (KMP find_subcircuit, random_reduce). identitysearch.py (BFS over gate sequences using numpy matrices). qasm.py (a small parser).
- density.py (entropy, fidelity) and trace.py (Tr with partial trace over TensorProduct).

**Hard dependencies.**

- core: noncommutative Mul with an ordered nc part, args_cnc, constructor postprocessors (Basic._constructor_postprocessor_mapping), the Mul._kind_dispatcher plus custom KindDispatchers, custom expand hints (_eval_expand_<hint>), and Wild/match
- matrices (Kronecker product, dense matrices for represent, eigenvals for density)
- concrete (Sum) and integrals (represent over continuous bases)
- functions.special (KroneckerDelta, DiracDelta) and functions.combinatorial
- physics (wigner.py for CG/3j/6j/9j values; constants hbar)
- printing + printing.pretty (each class has _print_contents/_pretty/_latex built on stringPict/prettyForm)
- multipledispatch (Dispatcher for transforms), ntheory (shor), simplify

**Python features that are hard to port.**

- getattr-based dispatch on class-name strings (dispatch_method, _apply_operator, _represent_*)
- multipledispatch Dispatcher and KindDispatcher registered at import time
- global constructor-postprocessor registration as an import side effect (transforms.py:279-291)
- __slots__ state outside args (hilbert_space)
- optional numpy/scipy/matplotlib imports through import_module
- random in random_circuit, random_reduce and shor

**Behaviour a port must reproduce.**

- Because of the 1.14 postprocessors, multiplying already-constructed objects changes the result type: Bra('b')*Ket('k') is an InnerProduct and Ket*Bra an OuterProduct; TensorProduct(A,B)*TensorProduct(C,D) becomes (A*C)x(B*D); Ket**2 raises TypeError. Postprocessors live in a Python set, so their order is unspecified.
- Commutator(b, a) canonicalizes to -[a,b], and Commutator(a*b, a).expand(commutator=True) == -a*[a,b]. Signs and order follow default_sort_key.
- Dagger(a*b) == Dagger(b)*Dagger(a); Dagger of a commutative scalar is its conjugate.
- QExpr.hilbert_space is a __slots__ attribute derived from args and not part of args, so equality and hashing ignore it.
- Dispatch is built from Python class names: dispatch_method uses '%s_%s' % (basename, arg.__class__.__name__) (qexpr.py:397), alongside _apply_operator_<Class>, _represent_<Basis> and _eval_innerproduct_<Bra>.
- represent(format='numpy'|'scipy.sparse') returns third-party matrix types, and matrixcache.py caches numpy matrices.
- The Lark LaTeX parser imports this package on the first implicit multiplication (parsing/latex/lark/transformer.py:191, adjacent_expressions) and builds Bra/Ket/InnerProduct/OuterProduct for \langle a|, |a\rangle and \langle a|b\rangle (checked: parse_latex(r'\langle a|b\rangle', backend='lark') -> <a|b>).
- tensor_product_simp is deprecated in 1.14 but still works and emits a warning that tests check.

**Rust prior art.**

- roqoqo (Apache-2.0): numeric quantum-circuit gate matrices; reference only, no symbolic capability
- q1tsim (Apache-2.0, verify): numeric circuit simulator, reference for gate conventions
- no symbolic Dirac-algebra crate exists; the port follows SymPy

**How its tests check results.** 298 tests, 10143 lines. Structural == on qapply/represent/expand results and Matrix ==. Heavy exact string checks: test_printing.py alone has 246 asserts over str/pretty/upretty/latex/srepr, with 2D ASCII art for CG, Wigner symbols and kets. Some tests skip without numpy/scipy/matplotlib (circuitplot, identitysearch, matrixutils, represent, sho1d, density). 6 slow/XFAIL markers.

**Notes.** app_relevance is 'serve' only because of a small slice: state.py Bra/Ket, innerproduct.py, operator.OuterProduct and transforms.py are imported by the Lark parse_latex transformer the app uses. The other ~95% of this unit is not used by the app. That slice (~600 code lines with printing) should land with the LaTeX parser port, or the Rust parser should refuse bra-ket syntax by name, consistent with the app's 'refuse rather than misread' policy. The full unit is about 3+ person-months. It splits into: core states/operators/qapply/represent (L), spin+cg (M), and gates/qubits/circuits/qasm/shor/grover/identitysearch (M). It is the main client that tests the core's nc and postprocessor design, so that design must be settled before this port starts.

### `physics.mechanics`

11,177 source lines, 124 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: none.

Multibody dynamics on top of physics.vector: Particle/RigidBody, inertia, loads, KanesMethod, LagrangesMethod, the System API, joints (Pin/Prismatic/Cylindrical/Planar/Spherical/Weld), JointsMethod (deprecated), actuators (force/torque/spring/damper/Duffing/Coulomb friction), pathways and wrapping geometry, Linearizer, and helpers (msubs, find_dynamicsymbols, energies, Lagrangian). About 3.8k code lines.

**Why this strategy.** This is a thin orchestration layer over matrices (Jacobians, LU solves) and physics.vector, with no deep algorithms of its own. It can be ported mechanically once those exist.

**Key algorithms.**

- KanesMethod (kane.py): _initialize_kindiffeq_matrices takes Jacobians of the kinematic differential equations in u and qdot. Nonholonomic constraint elimination uses linear_eq_to_matrix and Ars = -B_dep.LUsolve(B_ind). _form_fr/_form_frstar form generalized active and inertia forces from partial velocities. rhs() comes from mass_matrix_full.LUsolve(forcing_full) or an inv(method).
- LagrangesMethod.form_lagranges_equations (lagrange.py): Jacobians of L in qdot and q, Lagrange multipliers from holonomic and nonholonomic constraints, mass_matrix_full/forcing_full.
- Linearizer (linearize.py): general linearization in the Peterson/Gede/Hubbard style with constraint partitioning, a permutation matrix perm_mat, and op-point substitution through msubs; A/B matrices through LU or a user-supplied linear_solver.
- System (system.py): collects bodies, joints, loads and actuators; forms EOMs through Kane or Lagrange with validation.
- Joints (joint.py): frame alignment by axis (_alignment_rotation), automatic kinematic equations and generalized speeds; SphericalJoint uses body/space-fixed rotation sequences.
- Actuators/pathways (actuator.py, pathway.py) and wrapping_geometry.py: geodesic_length and geodesic_end_vectors on WrappingSphere/WrappingCylinder.
- functions.msubs (smart substitution that skips derivatives) and find_dynamicsymbols (AppliedUndef of t, with exclusions)

**Hard dependencies.**

- physics.vector (ReferenceFrame, Point, Vector, Dyadic, dynamicsymbols)
- matrices (jacobian, LUsolve, inv with methods, ImmutableMatrix, zeros/eye)
- core (AppliedUndef/Function, Derivative, Dummy, xreplace/subs)
- solvers.solveset.linear_eq_to_matrix
- simplify (trigsimp/simplify, used optionally on results)

**Python features that are hard to port.**

- mutable objects with property setters and validation
- deprecation shims (Body, JointsMethod)
- duck-typed load tuples
- functools.wraps decorators in System
- a global class flag (Vector.simp) shared with physics.vector

**Behaviour a port must reproduce.**

- The exact forms of mass matrices, forcing vectors and rhs depend on LUsolve's elimination order and pivoting, and on the order of q/u lists.
- find_dynamicsymbols returns sets; any ordered output goes through default_sort_key.
- The deprecated Body and JointsMethod paths are still tested with warns_deprecated_sympy.
- Vector.simp (a physics.vector class flag) changes outputs.
- Loads are duck-typed tuples (Point or ReferenceFrame, Vector) normalized by loads._parse_load.

**Rust prior art.**

- no symbolic multibody crate exists; PyDy (BSD-3, Python) is the conceptual reference
- nalgebra (Apache-2.0) only for downstream numeric use

**How its tests check results.** 124 tests, 8127 lines. Mostly expand(a) == expand(b), simplify(KM.rhs() - expected) == zeros, Matrix == explicit, eigenvals dicts, raises, and deprecation warnings. Comparisons through expand/simplify reduce sensitivity to exact form. A few slow tests (bicycle model in test_kane3).

**Notes.** Volume, not depth, makes this L (about 1.5-2 person-months). parsing.autolev (another cluster) generates code against this API, so its port depends on this unit.

### `physics.continuum_mechanics`

6,861 source lines, 34 test functions. Difficulty **M**. Strategy: **port faithfully**. App relevance: none.

Structural analysis: Beam and Beam3D (singularity-function loads, reactions, shear/moment/slope/deflection, influence lines, plotting), Truss (method of joints), Cable, and three-hinged Arch. About 3.1k code lines; beam.py is 3903 raw lines.

**Why this strategy.** Glue over integrate, linsolve, dsolve and SingularityFunction. The risk lies in upstream output forms, not in this code. Plotting is deferred along with the plotting cluster.

**Key algorithms.**

- Beam.apply_load/apply_support build a load as a sum of SingularityFunction terms (beam.py).
- Beam.solve_for_reaction_loads integrates the load twice with integration constants, applies boundary conditions, and solves with linsolve. Shear force, bending moment, slope and deflection come from integrate over SingularityFunction.
- max_shear_force/max_bmoment piece-wise solve over intervals; solve_for_ild_* compute influence lines.
- Beam3D: coupled equations solved with dsolve.
- Truss (truss.py): nodal equilibrium matrix assembly and solve; Cable (cable.py) and Arch (arch.py) are closed-form statics through solve.
- Plotting (plot/PlotGrid, matplotlib/numpy) for loads, shear, moment and deflection.

**Hard dependencies.**

- integrals (integrate of SingularityFunction and Piecewise)
- solvers (linsolve, solve, ode.dsolve)
- series.limit
- functions (SingularityFunction, Piecewise, factorial)
- matrices
- physics.units (Quantity values in loads; tests use newton and meter)

**Python features that are hard to port.**

- mutable Beam state built up through apply_* calls
- optional numpy/matplotlib
- sstr-based labels

**Behaviour a port must reproduce.**

- reaction_loads dict values must match exactly, units included ({R1: -13750*newton, R2: -31250*newton}).
- Names and placement of integration constants (C3, C4) and SingularityFunction exponent conventions (-1, -2 for point loads and moments) appear in results.
- Mixed Float/Rational inputs propagate into outputs.
- Plotting paths need numpy/matplotlib.

**Rust prior art.**

- none symbolic; reference only

**How its tests check results.** 34 tests, 1362 lines. Structural == against explicit SingularityFunction expressions and dicts, with expand/simplify of differences in some places.

**Notes.** Port late, after integrals (SingularityFunction), solvers (linsolve/dsolve) and physics.units. About 3-4 person-weeks without plotting.

### `physics.control`

6,153 source lines, 49 test functions. Difficulty **M**. Strategy: **port faithfully**. App relevance: none.

Linear time-invariant control systems: TransferFunction, PIDController, Series/Parallel/Feedback block algebra and their MIMO counterparts, TransferFunctionMatrix, StateSpace, discretization (gbt/bilinear/forward_diff/backward_diff), phase/gain margins, plus numeric data and matplotlib plots for pole-zero, step, impulse, ramp, Bode, Nyquist and Nichols. About 1.6k code lines (lti.py is 5001 raw lines, mostly docstrings).

**Why this strategy.** The block algebra and state-space formulas are short. Everything difficult (roots, CRootOf, solveset, limits, linodesolve, inverse Laplace) belongs to other clusters. Port the data functions and defer the matplotlib wrappers.

**Key algorithms.**

- TransferFunction (lti.py:439): unevaluated num/den expressions in var. poles()/zeros() use _roots, which calls roots(multiple=True) and falls back to CRootOf when roots are incomplete (lti.py:34). is_stable = fuzzy_and(re(pole).is_negative). dc_gain uses limit(..., var, 0). Constructors from_rational_expression, from_coeff_lists and from_zpk.
- Block algebra: __add__/__mul__/__neg__ return Parallel/Series objects. doit() combines them with cancel/expand options. Feedback gives the closed loop G/(1 - sign*G*H). MIMOFeedback uses a matrix inverse (lti.py:3052).
- StateSpace (lti.py:4184): controllability and observability matrices, rank-based is_controllable/is_observable, rewrite to TF as C*(sI-A)^-1*B + D, output_response through ode.systems.linodesolve.
- gbt discretization (lti.py:42-206) and phase_margin/gain_margin through solveset on Interval(0, oo, left_open=True) (lti.py:208-362).
- control_plots.py: numeric response data through integrals.laplace._fast_inverse_laplace + evalf, and pole-zero data through Poly.all_roots/nroots.

**Hard dependencies.**

- polys (Poly, roots, rootof/CRootOf, cancel, degree, numer)
- solvers (solveset, linsolve, linear_eq_to_matrix, ode.systems.linodesolve)
- series.limit
- matrices + matrices.expressions (MatMul/MatAdd for MIMO)
- integrals.laplace (_fast_inverse_laplace)
- mpmath prec_to_dps (becomes the Rust float layer)

**Python features that are hard to port.**

- operator overloading that returns block-diagram objects
- an is_StateSpace_object duck-typing flag
- decorators _check_other_SISO/_check_other_MIMO
- optional numpy/matplotlib

**Behaviour a port must reproduce.**

- Equality is structural: TransferFunction(s-1, s**2-2*s+1, s) is not simplified, and factor(tf) gives TransferFunction(s - 1, (s - 1)**2, s).
- to_expr builds unevaluated trees, Mul(num, Pow(den, -1, evaluate=False), evaluate=False), and tests compare them exactly.
- zeros()/poles() list order and CRootOf indices come from roots/rootof, and str() of float roots is compared exactly.
- tf1 + tf2 == Parallel(tf1, tf2) structurally; operators raise ValueError on mismatched variables.

**Rust prior art.**

- python-control (BSD-3, Python) as a numeric reference
- no symbolic LTI crate in Rust; numeric Rust control crates exist but are not needed (verify licences before any use)

**How its tests check results.** 49 tests (574 asserts in test_lti.py). Structural == on TransferFunction/Series/Parallel trees and Matrix ==. Root lists use exact expressions with CRootOf, or str() of Floats. Plot-data tests need numpy/matplotlib.

**Notes.** Exact outputs depend on polys.roots ordering and CRootOf, so port only after the polys root-finding parity work. About 3-4 person-weeks.

### `physics.vector`

4,931 source lines, 112 test functions. Difficulty **M**. Strategy: **port faithfully**. App relevance: none.

Classical-mechanics vector algebra with frames: ReferenceFrame (orientations, DCMs, angular velocity and acceleration), Vector (a plain Python object holding (3x1 Matrix, frame) pairs), Dyadic, Point (position/velocity/acceleration graph and two-point theorems), dynamicsymbols, kinematic equations, partial velocities, field functions, and vector printers (q', \dot{q}). About 1.8k code lines.

**Why this strategy.** A thin layer over matrices and core functions. The only design work is mapping the mutable, cyclic frame/point graph to Rust (an arena of frames with ids and interior-mutable caches) while keeping the Python-visible mutation API through PyO3.

**Key algorithms.**

- ReferenceFrame.orient_axis/orient_body_fixed/orient_space_fixed/orient_quaternion/orient_dcm/orient_explicit (frame.py): build DCMs from rotation sequences ('123', 'zxz', ...), store them in adjacency dicts (_dcm_dict), and derive angular velocity from the DCM (_w_diff_dcm).
- ReferenceFrame.dcm (frame.py:482): path search across the frame graph (_dict_list, BFS) and a product of adjacent DCMs, memoized in _dcm_cache on both frames (with the transpose); re-orientation invalidates caches across linked frames. ang_vel_in/ang_acc_in chain sums along the path.
- Vector.express/dot/cross/outer/dt/diff (vector.py): transport-theorem time derivative and frame re-expression through DCMs; variable_map remaps coordinate symbols.
- Point (point.py): pos/vel/acc dict graph with BFS (_pdict_list), v2pt_theory/a2pt_theory/v1pt_theory/a1pt_theory, and automatic velocity computation through position derivatives.
- functions.py: kinematic_equations (body/space/quaternion), partial_velocity, get_motion_params (integration), time_derivative; fieldfunctions.py: curl/divergence/gradient/scalar_potential.

**Hard dependencies.**

- core (Function/AppliedUndef as dynamicsymbols of a global t, Derivative, Symbol, Dummy, subs/xreplace)
- matrices (dense Matrix arithmetic, rotation matrices)
- simplify (trigsimp when Vector.simp is set)
- integrals (scalar_potential, get_motion_params)
- printing + printing.pretty (VectorStrPrinter/VectorLatexPrinter/VectorPrettyPrinter)

**Python features that are hard to port.**

- mutable objects whose caches reference each other (cyclic graph)
- operator overloading of & ^ | and *
- class-level flag Vector.simp
- attributes on the dynamicsymbols function object (_t, _str)
- lazy properties

**Behaviour a port must reproduce.**

- Vector is not a Basic, so it has no immutable args tuple. __eq__ re-expresses in the first frame and tests expand(dot) != 0 per basis vector. Printing order follows dict insertion order of frames.
- DCM cache invalidation on re-orientation must match, or results go stale.
- Vector.simp is a class-wide flag; it changes outputs and is part of the _var_dict cache keys.
- Printing: derivatives of dynamicsymbols print as q' (str) and \dot{q} (latex), and & ^ | mean dot, cross and outer.

**Rust prior art.**

- none symbolic; nalgebra/glam (Apache-2.0 / MIT-Apache) only for numeric rotations as a cross-check

**How its tests check results.** 112 tests, 2610 lines. Structural == through Vector.__eq__ (expand-based) and Matrix ==, plus str/latex/pretty compares in test_printing.py.

**Notes.** Foundation for physics.mechanics and physics.biomechanics. Its printers subclass the core printers, so the printing cluster must expose an extensible printer trait. About 3-4 person-weeks.

### `physics.optics`

2,984 source lines, 16 test functions. Difficulty **M**. Strategy: **port faithfully**. App relevance: none.

Gaussian optics (ABCD ray transfer matrices, geometric rays, complex beam parameter), polarization (Jones/Stokes vectors, Mueller matrices, retarders/polarizers), media with permittivity/permeability, transverse waves (TWave), and utilities (refraction angle, Fresnel coefficients, Brewster/critical angles, lens/mirror formulas, hyperfocal distance). About 750 code lines.

**Why this strategy.** Formula-level code over matrices, geometry and units with no algorithmic depth.

**Key algorithms.**

- RayTransferMatrix subclasses MutableDenseMatrix with A/B/C/D accessors and __mul__ dispatch to GeometricRay/BeamParameter (gaussopt.py). Includes gaussian_conj and conjugate_gauss_beams (solves beam conjugation).
- Polarization (polarization.py): mueller_matrix(J) via the TensorProduct Kronecker product of J and its conjugate, plus Jones/Stokes constructions.
- utils.refraction_angle/deviation: vector Snell's law with Ray3D/Plane intersection (geometry). lens_formula/mirror_formula solve for the missing quantity.
- Medium/TWave: symbolic defaults built from physics.units constants (speed_of_light, u0, e0).

**Hard dependencies.**

- matrices (Matrix subclassing)
- geometry (Ray3D, Plane, intersection)
- physics.units (Medium, TWave)
- physics.quantum (TensorProduct over matrices for mueller_matrix)
- polys (together, cancel), series (Limit), functions.elementary

**Python features that are hard to port.**

- subclassing MutableDenseMatrix
- __mul__ overrides that return different classes
- isinstance-based argument polymorphism

**Behaviour a port must reproduce.**

- RayTransferMatrix is a mutable Matrix subclass and compares equal to a plain Matrix (lens == Matrix([[1, 0], [-1/f, 1]])).
- Medium speed/refractive-index expressions embed exact unit scale factors.
- refraction_angle returns a Ray3D or an angle depending on its input types.

**Rust prior art.**

- none

**How its tests check results.** 16 tests, 491 lines. Structural == against Matrix and expressions, with simplify in places.

**Notes.** About 1-2 person-weeks, after geometry 3D and units.

### `physics.biomechanics`

4,162 source lines, 0 test functions. Difficulty **M**. Strategy: **port faithfully**. App relevance: none.

Musculotendon models for mechanics: De Groote 2016 characteristic curves as SymPy Function subclasses (tendon force-length and inverse, passive/active fiber force-length, fiber force-velocity and inverses), activation dynamics (zeroth-order, first-order De Groote 2016), and MusculotendonDeGroote2016 built on mechanics ForceActuator and pathways. About 1.2k code lines.

**Why this strategy.** Thin: a set of Function subclasses plus mechanics glue. It mainly needs the core to support user-defined Function kinds with eval/fdiff/doit/evalf/inverse hooks and printer hooks.

**Key algorithms.**

- CharacteristicCurveFunction subclasses (curve.py): eval returns None (never auto-evaluates); doit(evaluate=...) gives closed forms in exp/log/sinh/cosh with UnevaluatedExpr when evaluate=False; fdiff per argument; inverse(); _eval_evalf; custom _latex and _print_code for code printers; with_defaults Float constants.
- FirstOrderActivationDeGroote2016 (activation.py): tanh-smoothed activation ODE; state/input/constants matrices and rhs.
- MusculotendonDeGroote2016 (musculotendon.py): rigid or elastic tendon formulations (an IntEnum) that assemble curve functions into force and state equations through ForceActuator and pathways.

**Hard dependencies.**

- physics.mechanics (ForceActuator, pathways)
- physics.vector (dynamicsymbols)
- core Function subclass protocol (eval/fdiff/doit/_eval_evalf/inverse)
- matrices
- printing (code printers for Python, NumPy, JAX, C, Fortran, Octave), plus utilities.lambdify in tests

**Python features that are hard to port.**

- Function subclasses with classmethod eval and overridden doit
- dataclass-based curve collections
- IntEnum formulations
- pytest classes with parametrize

**Behaviour a port must reproduce.**

- Default Float constants (e.g. 33.93669377311689, 0.995) must round-trip exactly through the code printers.
- doit(evaluate=False) keeps UnevaluatedExpr(l_T_tilde - c1) inside exp.
- str(curve) shows all constants as arguments.

**Rust prior art.**

- none

**How its tests check results.** About 215 test methods in pytest classes with parametrize (inventory reports 0 test functions because it counts module-level defs only), 2928 lines. Exact string compares of str/latex/code-printer output, structural ==, and pytest.approx numeric checks through lambdify (numpy/jax optional).

**Notes.** Depends on the code-printing cluster for most test assertions. About 2-3 person-weeks.

### `physics.hep`

716 source lines, 5 test functions. Difficulty **M**. Strategy: **port faithfully**. App relevance: none.

Dirac gamma-matrix algebra in tensor-index notation: GammaMatrix as a TensorHead over LorentzIndex (dim 4), Kahane simplification of contracted gamma products, traces, and p-slash simplification. About 340 code lines.

**Why this strategy.** Small but intricate and entirely built on tensor.tensor. Port it directly after tensor.

**Key algorithms.**

- kahane_simplify (gamma_matrices.py:337): Kahane's 1968 algorithm. It builds a link graph between free and contracted index positions, walks the cycles, and emits a factor of (-2)^k with reversed free-index chains.
- gamma_trace / _trace_single_line / _gamma_trace1 (gamma_matrices.py:170-336): recursive trace of gamma products into metric contractions.
- simplify_gpgp (gamma_matrices.py:81): p-slash product reduction; extract_type_tens.

**Hard dependencies.**

- tensor (tensor.tensor: TensorIndexType, TensorHead, TensMul/TensAdd, contract_metric, canon_bp, index structure)
- matrices (trace, eye)

**Python features that are hard to port.**

- module-level TensorIndexType/TensorHead singletons created at import

**Behaviour a port must reproduce.**

- Results depend on tensor auto-dummy naming (L_0, L_1, ...) and canon_bp term ordering.
- kahane_simplify(G(mu)*G(nu)*G(-mu)) == -2*GammaMatrix(nu), and gamma_trace(G(mu)*G(nu)) == 4*metric(mu, nu) (checked).

**Rust prior art.**

- none (FORM is GPL, Cadabra2 is GPL-3.0; reference only)

**How its tests check results.** 5 long tests, 427 lines: structural == on tensor expressions after simplification.

**Notes.** About 1-2 person-weeks once tensor.tensor reaches parity.

### `vector`

5,009 source lines, 67 test functions. Difficulty **M**. Strategy: **port faithfully**. App relevance: none.

sympy.vector: 3D vector calculus with CoordSys3D (Cartesian and curvilinear, rotations and translations), BaseScalar/BaseVector, Vector/Dyadic algebra (BasisDependent classes subclassing core Add/Mul), Del operators, express, scalar potentials, orthogonalize, ParametricRegion/ImplicitRegion with vector_integrate. About 2.2k code lines.

**Why this strategy.** Mostly glue over matrices, solve and integrate. The real cost is core design: vector node kinds that reuse Add/Mul flattening while keeping a component map. Without those core extension points it cannot be ported faithfully.

**Key algorithms.**

- CoordSys3D (coordsysrect.py): built from location/rotation/transformation (a Lambda or 'spherical'/'cylindrical'); _get_lame_coeff; inverse transformation equations through solve (_calculate_inv_trans_equations); rotation_matrix/position_wrt through _path to a common ancestor.
- Orienters (orienters.py): Axis, Body/Space ThreeAngle, Quaternion -> rotation matrices (_rot).
- BasisDependentAdd/BasisDependentMul (basisdependent.py:183-312): component-dict normalization layered on core Add/Mul constructors; a VectorAdd constructor postprocessor (vector.py:419-434); VectorKind kind dispatch.
- operators.py: gradient/divergence/curl/laplacian in curvilinear coordinates using Lame coefficients (_diff_conditional).
- functions.py: express (with variables=True substitution), scalar_potential (successive integration), is_conservative/is_solenoidal, orthogonalize (Gram-Schmidt), directional_derivative.
- integrals.py (ParametricIntegral via Jacobian or cross-product magnitude + integrate) and implicitregion.py (regular_point for conics via diophantine and squarefree core, singular_points via solveset/nonlinsolve, multiplicity, rational_parametrization by projection from a regular point).

**Hard dependencies.**

- core (subclassing Add/Mul, constructor postprocessor, Mul kind dispatcher)
- matrices
- simplify/trigsimp
- solvers (solve, solveset, nonlinsolve, diophantine)
- integrals
- polys (gcd, total_degree), ntheory (core), geometry (Point in implicitregion)
- printing.pretty

**Python features that are hard to port.**

- multiple inheritance from core Add/Mul with a custom __new__
- class attributes wiring _add_func/_mul_func/_zero_func
- a postprocessor registered at import time
- an _op_priority ladder (12.0, 12.1, 13.0, 13.1) between Vector and Dyadic

**Behaviour a port must reproduce.**

- VectorAdd is an instance of Add (isinstance(C.i + C.j, Add) is True), so generic Add code walks vector sums.
- Printing quirk: curl(C.y*C.i) prints '(-1)*C.k' (VectorMul._sympystr parenthesizes the measure number); the zero vector is a VectorZero singleton.
- Adding a scalar to a vector goes through the Add postprocessor and raises or misbehaves exactly as upstream does.
- Inverse transformations take the first solve() solution.

**Rust prior art.**

- none symbolic

**How its tests check results.** 67 tests, 2002 lines. Structural == against explicit vectors and matrices, simplify, and str/pretty/latex printing compares.

**Notes.** The app's physics vectors (components, magnitude, atan2 direction) need only core functions, not this package. About 3-4 person-weeks.

### `diffgeom`

2,432 source lines, 24 test functions. Difficulty **M**. Strategy: **port faithfully**. App relevance: none.

Differential geometry: Manifold/Patch/CoordSystem with coordinate transformations, scalar, vector and form fields, Differential, WedgeProduct/TensorProduct of forms, Commutator, LieDerivative, covariant derivatives, integral curves, and Christoffel/Riemann/Ricci from a metric. Predefined R2/R3 systems in rn.py. About 930 code lines.

**Why this strategy.** Thin over solve, simplify and arrays.

**Key algorithms.**

- CoordSystem.transform/jacobian (diffgeom.py ~L400-560): path search across relation graphs; inverse transformations via solve, cached with @cacheit (_inverse_transformation).
- WedgeProduct/TensorProduct evaluation: antisymmetrization over permutations using Permutation signatures and factorial normalization.
- LieDerivative, BaseCovarDerivativeOp/CovarDerivativeOp; intcurve_series/intcurve_diffequ.
- metric_to_Christoffel_1st/2nd, metric_to_Riemann_components, metric_to_Ricci_components (ImmutableDenseNDimArray).

**Hard dependencies.**

- core (Lambda, Dummy, Str, cacheit)
- matrices
- solvers (solve)
- simplify
- tensor (tensor.array ImmutableDenseNDimArray)
- combinatorics (Permutation, for the sign only)
- functions (factorial)

**Python features that are hard to port.**

- @cacheit on classmethods
- deprecation shims
- module-level manifold singletons with attribute injection (R2.x = ...)

**Behaviour a port must reproduce.**

- Inverse transformations take the first solution returned by solve, so their form follows solve's output.
- Deprecated connect_to paths are still used in rn.py under warnings suppression.
- rcall semantics on fields applied to points.

**Rust prior art.**

- none

**How its tests check results.** 24 tests, 611 lines. Matrix == on transforms (C2.transform(C1) == Matrix([c/2, (d - 1)/3])) and trigsimp/simplify(expr) == 0.

**Notes.** About 2-3 person-weeks after solve and arrays.

### `tensor`

13,225 source lines, 239 test functions. Difficulty **XL**. Strategy: **port faithfully**. App relevance: none.

Four sub-packages. (a) tensor.indexed (Indexed/IndexedBase/Idx) and index_methods (get_indices, get_contraction_structure): Einstein-summation bookkeeping used by codegen, printers, concrete and lambdify. (b) tensor.array: NDimArray dense/sparse mutable/immutable, tensorproduct/tensorcontraction/tensordiagonal/permutedims/derive_by_array, ArrayComprehension, ArrayDerivative. (c) tensor.array.expressions: ArrayTensorProduct/ArrayContraction/ArrayDiagonal/PermuteDims/ArrayAdd and conversion to and from matrix expressions; this powers MatrixExpr.diff. (d) tensor.tensor: abstract index notation (TensorIndexType, TensorHead, TensMul/TensAdd, canon_bp, contract_metric, replace_with_arrays, WildTensor) plus toperators.PartialDerivative. About 7.3k code lines in total (tensor.py 3.0k, arrays 3.7k of which expressions 2.7k).

**Why this strategy.** These are SymPy's own algorithms with no Rust equivalent (Butler-Portugal canonicalization and contraction-to-matrix recognition). The ndarray crate can serve as storage for NDimArray, but the semantics must be ported. The sub-packages should be scheduled separately because their consumers differ.

**Key algorithms.**

- canon_bp (tensor.py:4997, TensMul.canon_bp): delegates to combinatorics.tensor_can.canonicalize (tensor_can.py:633), the Butler-Portugal algorithm. double_coset_can_rep is built on BSGS of slot symmetries and dummy (metric) symmetries, and canonical_free handles free indices. Returns 0 when symmetries force cancellation.
- TensMul._tensMul_contract_indices and _IndexStructure (tensor.py:98): free/dummy index bookkeeping with automatic dummy renaming to <dummy_name>_<n> (L_0, L_1).
- contract_metric/contract_delta: raising and lowering through metrics, dimension traces. TensAdd._tensAdd_collect_terms. riemann_cyclic. replace_with_arrays: component evaluation through arrays.
- Arrays (arrayop.py): row-major flat storage, contraction by index arithmetic, derive_by_array, permutedims.
- Array expressions (array_expressions.py, from_array_to_matrix.py): _EditArrayContraction graph edits, canonicalization of nested contraction and tensor-product forms, and recognition of matmul chains, traces, transposes and Hadamard products (convert_array_to_matrix). array_derive gives matrix calculus ((a.T*X*b).diff(X) == a*b.T, checked).
- get_contraction_structure (index_methods.py): Einstein-summation analysis for codegen.

**Hard dependencies.**

- core (Add.flatten special-cases TensExpr; constructor postprocessor for TensExpr at tensor.py:5263)
- combinatorics (Permutation, perm_groups Schreier-Sims, tensor_can) for tensor.tensor and array expressions
- matrices + matrices.expressions (array <-> matrix conversion)
- functions.special (KroneckerDelta)
- multipledispatch (indexed)

**Python features that are hard to port.**

- global singletons (TensorManager, _tensor_data_substitution_dict)
- AssocOp subclassing for TensAdd/TensMul
- memoize_property, deprecated decorators
- __getitem__ overloading to build Indexed and tensors with index signs (-a)
- multipledispatch in indexed

**Behaviour a port must reproduce.**

- Auto-generated dummy names show up in str output and tests compare against them ('A(b, -L_0)*A(L_0, a) + A(b, -L_0)*B(L_0, a)'). A(a,b)*B(-a,-b) prints with renamed dummies.
- canon_bp ordering and signs must match the Butler-Portugal implementation exactly; any other canonical form breaks str-compare tests.
- TensorManager holds global commutation groups, and the deprecated .data API uses a global _tensor_data_substitution_dict.
- NDimArray has mutable and immutable variants, rank-0 behaviour, and list-like equality. Matrix-calculus outputs depend on the heuristics in from_array_to_matrix.py.
- core/function.py _derivative_dispatch sends any diff involving a MatrixBase, MatrixExpr, NDimArray or list to ArrayDerivative.

**Rust prior art.**

- ndarray (MIT OR Apache-2.0): storage and stride arithmetic for NDimArray of Expr
- xPerm/xAct and Cadabra2 (GPL): reference only, not linkable; no Rust tensor-canonicalization crate exists

**How its tests check results.** 239 tests, 6987 lines. tensor.tensor relies on str() compares and structural == after canon_bp. Arrays are compared with == against lists and arrays. Array expressions use structural == against ArrayContraction/ArrayTensorProduct trees and matrix-expression == after conversion.

**Notes.** This is not a leaf. Split it into: tensor.indexed (S-M, early, consumed by codegen/printing/concrete/lambdify/stats); tensor.array (M, consumed by Derivative/ArrayDerivative, diffgeom and stats); tensor.array.expressions (L, ships with matrices.expressions calculus); tensor.tensor + toperators (L, needs combinatorics.tensor_can, perm_groups and Schreier-Sims from the combinatorics cluster). The XL rating covers all four (about 4+ person-months).

### `liealgebras`

1,948 source lines, 12 test functions. Difficulty **S**. Strategy: **port faithfully**. App relevance: none.

Simple Lie algebras of types A-G: CartanType factory, per-type root data (simple and positive roots in an orthogonal basis), Cartan matrices, Dynkin/Coxeter diagram strings, RootSystem utilities, and WeylGroup (order, name, element order, reflection matrices). About 870 code lines.

**Why this strategy.** Tiny, table-driven and self-contained.

**Key algorithms.**

- CartanType.__new__ (cartan_type.py): parses a 'A3'-style string and dispatches to TypeA..TypeG (type_*.py), which hardcode simple_root, positive_roots, cartan_matrix, basis, dimension and dynkin_diagram.
- RootSystem (root_system.py): all_roots (positive and negative), root_space string, add_simple_roots/add_as_roots.
- WeylGroup (weyl_group.py): group_order through closed formulas (factorials via mpmath.fac, constants for E/F/G); element_order by repeated multiplication of reflection matrices until the identity; matrix_form builds reflection matrices per type; group_name/coxeter_diagram strings.

**Hard dependencies.**

- core (Basic/Atom, Rational)
- matrices (Matrix, eye)

**Python features that are hard to port.**

- string-dispatched factory with lazy imports
- mpmath fac returning mpf/float
- sympy.core.backend indirection

**Behaviour a port must reproduce.**

- group_order returns a float for some types (WeylGroup('A3').group_order() == 24.0 because of mpmath.fac) but an int for others (E6 -> 51840, checked).
- Diagram strings, group_name strings and root dict keys/order must match exactly.
- Imports go through sympy.core.backend (a USE_SYMENGINE environment switch) and the module is its only user.

**Rust prior art.**

- none needed

**How its tests check results.** 12 tests, 221 lines: == against hardcoded strings, ints/floats and Matrix.

**Notes.** Good onboarding task to shake down the oracle harness; it needs only Matrix and Rational.

### `categories`

3,591 source lines, 12 test functions. Difficulty **M**. Strategy: **port faithfully**. App relevance: none.

Elementary category theory: Object, Morphism (Named, Composite, Identity), Category, Diagram (premises closed under composition, conclusions, properties), and diagram layout (DiagramGrid) with Xy-pic LaTeX output (XypicDiagramDrawer). About 1.25k code lines, of which diagram_drawing.py is about 1.0k.

**Why this strategy.** Self-contained heuristics with no mathematical dependencies beyond FiniteSet and latex. It is niche, so it is the lowest-priority port; running the Python module on the Rust core via PyO3 is an acceptable interim.

**Key algorithms.**

- Diagram._add_morphism_closure / Diagram.__new__ (baseclasses.py): closure of premises under composition with property propagation as FiniteSets; CompositeMorphism flattens and drops identities; subdiagram_from_objects; hom.
- DiagramGrid (diagram_drawing.py:175): skeleton construction (_build_skeleton), triangle enumeration (_list_triangles), triangle welding (_weld_triangle), pseudopod growth (_grow_pseudopod), and sequential/grouped layouts on a _GrowableGrid.
- XypicDiagramDrawer (diagram_drawing.py:1495): arrow curving/looping and label placement heuristics producing Xy-pic strings; ArrowStringDescription.

**Hard dependencies.**

- core (Basic, Dict, Tuple, Symbol, Str)
- sets (FiniteSet, EmptySet)
- printing.latex

**Python features that are hard to port.**

- heavy use of Python set/dict iteration order inside layout heuristics
- FiniteSet as a property container
- string building for Xy-pic

**Behaviour a port must reproduce.**

- Layout depends on default_sort_key order, FiniteSet iteration order and dict insertion order; tests assert exact cell placement (grid[1, 1] == A1, grid[0, 3] is None).
- Xy-pic output strings are compared byte for byte.

**Rust prior art.**

- none

**How its tests check results.** 12 tests (296 asserts in test_drawing.py): exact grid-cell identity checks and full Xy-pic string compares; test_baseclasses uses structural ==.

**Notes.** About 2-3 person-weeks, mostly reproducing ordering-sensitive layout decisions exactly.

### `physics`

5,299 source lines, 85 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: none.

Top-level physics helpers: secondquant.py (second quantization: fermion/boson operators, Fock states, normal ordering, Wick's theorem, dummy canonicalization), wigner.py (Wigner 3j/6j/9j, Clebsch-Gordan, Racah, Gaunt, real Gaunt, Wigner small-d/D), paulialgebra.py (Pauli symbols with product evaluation), hydrogen.py/sho.py/qho_1d.py/pring.py (wavefunctions and energies), matrices.py (Dirac/Pauli matrices, Minkowski tensor, deprecated mdft). physics/__init__.py imports units and matrices. About 1.8k code lines (secondquant 1.24k, wigner 0.27k).

**Why this strategy.** These are algorithms of SymPy's own design (Wick contraction, string-keyed dummy canonicalization, Racah sums) that must reproduce outputs exactly. There is no Rust prior art usable under the licence policy.

**Key algorithms.**

- wicks (secondquant.py): Wick's theorem. _get_contractions recursively pairs fermion operators with signs; keep_only_fully_contracted is an option.
- NO normal-ordered products with _sort_anticommuting_fermions (a bubble sort with a sign count keyed by _sqkey_operator); contraction gives KroneckerDelta with fermi-level assumptions (above_fermi/below_fermi on Dummy).
- substitute_dummies with _get_ordered_dummies/_determine_ambiguous: canonical dummy relabeling. Sort keys are built from masked string representations of factors produced by KeyPrinter (a StrPrinter subclass), with recursion for ambiguous groups. evaluate_deltas and simplify_index_permutations (PermutationOperator).
- wigner_3j / clebsch_gordan / racah / wigner_6j / wigner_9j / gaunt / real_gaunt (wigner.py): Racah-formula sums over a factorial table (_calc_factlist, a global growing list), exact sqrt of rationals, and an optional prec= path through evalf; wigner_d_small/wigner_d closed forms; dot_rot_grad_Ynm.
- Pauli.__mul__ / evaluate_pauli_product (paulialgebra.py): sigma_i*sigma_j = delta_ij + I*eps_ijk*sigma_k, including inside TensorProduct.
- hydrogen R_nl/Psi_nlm/E_nl/E_nl_dirac (assoc_laguerre, Ynm), sho R_nl, qho_1d psi_n (hermite), pring wavefunction.

**Hard dependencies.**

- core (Dummy with custom assumptions, noncommutative Symbol subclass, KroneckerDelta interplay)
- functions (factorial, binomial, KroneckerDelta, assoc_laguerre, Ynm, hermite)
- concrete (Sum)
- matrices
- printing (StrPrinter subclass KeyPrinter; latex)
- physics.quantum (TensorProduct, hbar)

**Python features that are hard to port.**

- a Symbol subclass overriding __mul__/_eval_power (Pauli)
- printer-string-based sort keys (KeyPrinter)
- a global factorial cache list
- Dummy assumptions as custom keyword flags (above_fermi)

**Behaviour a port must reproduce.**

- substitute_dummies relies on exact str-printer output, including str.replace on printed factors, so it needs a byte-exact StrPrinter.
- Pauli(1)*Pauli(2) evaluates to I*sigma3 through Python __mul__, but Mul(Pauli(1), Pauli(2)) stays sigma1*sigma2 (checked); Pauli**n reduces modulo 2.
- Wigner/CG closed forms (clebsch_gordan(3/2, 1/2, 1, -1/2, 1/2, 0) == -sqrt(2)/2) and the factorial-cache behaviour; prec= returns Floats.
- secondquant defines its own Dagger, Commutator and InnerProduct, distinct from physics.quantum's.
- repr/latex of NO, Fd and F objects is tested exactly ('NO(CreateFermion(a)*CreateFermion(i))').

**Rust prior art.**

- wigner-symbols (Rufflewind): exact Wigner symbols, but depends on rug/GMP (LGPL), so reference only
- wigners (verify licence): f64 Wigner 3j/CG with caching; numeric cross-check only

**How its tests check results.** 85 tests, 1903 lines. Structural == on wicks/substitute_dummies/evaluate_deltas outputs, many direct asserts on _sort_anticommuting_fermions tuples, repr/latex compares, and exact values for Wigner/CG.

**Notes.** wigner.py (about 1 person-week) should be ported before physics.quantum cg/spin. secondquant (about 4-6 person-weeks) should be the last part of this unit, after the str printer is byte-exact. The rest is a few days.

## Combinatorics, statistics, geometry

### Suggested port order inside the cluster

1. 1. combinatorics/permutations.py kernel (array-form ops, Cycle, ranks): integer-only, depends on nothing in the other units; unblocks matrices (PermutationMatrix, permute, graph), tensor.array permutedims, printing and every group algorithm.
1. 2. combinatorics leaf objects (prufer, graycode, subsets, partitions/RGS, group_numbers, schur_number): independent and parallelisable starter tasks that validate the oracle harness cheaply.
1. 3. combinatorics util.py plus the perm_groups BSGS core (orbits, transversals, Schreier vectors, schreier_sims_incremental, coset_factor, order, contains) plus named_groups/generators/group_constructs, behind an injectable RNG trait with replay mode: the foundation of all group theory.
1. 4. combinatorics tensor_can and galois: unblock tensor.tensor (canon_bp) and polys.numberfields.galoisgroups; cross-check against the butler-portugal crate.
1. 5. combinatorics fp stack: free_groups, then coset_table (tests inspect raw intermediate tables), rewritingsystem with fsm, fp_groups, homomorphisms, pc_groups; then perm_groups.presentation, strong_presentation and polycyclic_group, which need it.
1. 6. perm_groups advanced routines (subgroup_search, centralizer, normal_closure, sylow, minimal_blocks, composition_series, conjugacy_classes) plus Polyhedron: the hardest CGT code, after the BSGS core is stable.
1. 7. stats finite slice: rv.py framework, symbolic_probability, rv_interface, frv.py, frv_types.py. This is the app roadmap item (FiniteRV, P, given, Binomial) and needs only core, Piecewise, FiniteSet and Relational, so it can come early.
1. 8. geometry point.py, line.py, polygon.py (Triangle centres, transforms, Polygon canonical equality) and util.py (convex_hull, closest/farthest points), with a Rational fast path: the app's compile-time figure kit, needing only Matrix rank/rref, nsimplify and simplify on rationals.
1. 9. geometry ellipse.py/Circle (needs solve on polynomial systems), plane.py (linsolve), parabola.py, curve.py (integrate): after solvers reach parity.
1. 10. stats drv/drv_types (summation, series for the MGF path) and stochastic_process_types (Markov chains need matrices and linsolve).
1. 11. stats crv/crv_types, joint_rv(_types), compound_rv, matrix_distributions, random_matrix_models, error_prop: last, because parity is gated on integrate (including meijerint), solveset and reduce_rational_inequalities.
1. 12. stats.sampling native backend (rand_distr plus generic inverse-transform) once the distributions and a numeric evaluator exist.
1. 13. sandbox IndexedIntegral once Integral and Indexed exist. benchmarks: build the cross-implementation harness early (symbench needs only core, simplify and factor) and grow it as units land.

### Risks

- stats test parity depends on the exact output forms of integrate (including meijerint), summation, series, solveset and simplify. Porting stats before those units are at parity yields many failures that are not stats bugs. CI should gate stats by slice: finite first, continuous last.
- Geometry outputs are simplify's canonical radical forms (incenter, bisectors, Ellipse intersections), so every change to simplify or radsimp moves geometry expectations. Its 'exactness' also relies on a 2-digit numeric zero test in affine_rank, which reports false collinearity for near-degenerate input. Decide deliberately whether to reproduce it (parity) or fix it (a documented deviation).
- Randomised group algorithms (product replacement, schreier_sims_random, Monte Carlo is_alt_sym, randomised is_primitive/minimal_blocks, FpGroup.random with stdlib random) need a pluggable RNG with sequence-replay injection to run the _random_prec and seed=list tests. Bit-exact MT19937 parity is not required.
- Mutable state on supposedly immutable Basic objects (PermutationGroup lazy caches with history-dependent int/Integer return types, Polyhedron.rotate in-place, GrayCode skip() during iteration, Prufer/Permutation lazy ranks) clashes with an immutable hash-consed DAG. These need side caches (OnceCell) and explicit mutable handle types.
- Performance traps that come from other units: P(U>V) for two uniforms does not finish in 60 s; Triangle.bisectors takes 1.6 s; the stats suite needs about 264 s. A Rust port that reproduces the control flow inherits these unless integrate/simplify get faster. Finite product-space enumeration in stats is exponential by design.
- Big integers are needed throughout combinatorics (|S_n| = n!, permutation ranks above u64 from n = 21, group_numbers), so small-int fast paths must overflow cleanly into bigints.
- Oracle tooling: SymPy 1.14's sympy.test() requires pytest, which the pinned venv lacks, as it does numpy and scipy. All 17 sampling tests and several stats tests therefore skip, leaving the sampling oracle weak. A per-function runner was used for this survey.

### What other clusters and the core design must know

- Core: Permutation must be an atom node in the expression DAG (args=(), payload = array form, size part of identity), because matrices.expressions.PermutationMatrix, AppliedPermutation, tensor.array permutedims/PermutationArray and diffgeom put Permutations inside expressions. The node design needs an opaque leaf-payload variant (Vec<u32>) or a registry of external atom kinds.
- Core: FreeGroup, FreeGroupElement, FpGroup, FpSubgroup, CosetTable, Collector and RewritingSystem are not Basic (DefaultPrinting only), so the whole finitely-presented-group stack can live outside the expression DAG. Only generator names are Symbols.
- Core: PermutationGroup, Polyhedron, Prufer, GrayCode and Permutation hold mutable lazy caches or do in-place mutation on Basic. The core should offer a per-node memo facility (OnceCell side table) and let some types expose a separate mutable handle.
- Core/utilities: a shared deterministic RNG facility equivalent to sympy.core.random (randrange/choice/shuffle/uniform, seed(), and _randrange(seed) where seed may be a list replayed in order) is needed by combinatorics, geometry random_point, ntheory and the test helpers (verify_numerically). Tests rely on injection hooks, not on matching MT19937 output.
- polys: polys.polytools.galois_group and polys.numberfields.galoisgroups/galois_resolvents import combinatorics.galois (S1..S6TransitiveSubgroups, PGL2F5) and Permutation, so polys needs PermutationGroup for Galois groups of degree 6 or less.
- tensor: tensor.tensor needs combinatorics.tensor_can (canonicalize, get_symmetric_group_sgs, bsgs_direct_product, riemann_bsgs) and therefore the BSGS machinery; butler-portugal (MIT) is Rust prior art.
- matrices: Matrix.permute (matrixbase.py:2294, common.py:2269) and matrices/graph.py (connected components) use combinatorics Permutation; matrices does not need groups.
- sets: Partition subclasses FiniteSet, and GeometrySet subclasses Set and registers intersection_sets/union_sets handlers through multipledispatch, so the Set framework must let external Set subclasses register binary-operation handlers.
- integrals/vector/physics: integrals/intpoly.py (polytope_integrate) needs geometry Polygon, Segment2D and Point; vector/parametricregion.py and implicitregion.py, physics/optics/utils.py (Ray3D, Plane, intersection), plotting/pygletplot and physics/continuum_mechanics/beam.py also import geometry, so geometry is not a leaf.
- stats is a true leaf (nothing outside stats imports it), but it uses almost everything: integrals (incl. meijerint), concrete.summation, series (moment-generating-function Taylor path), polys.poly, solveset, reduce_rational_inequalities, linsolve, sets (Interval/Union/Intersection/Complement/Range/ConditionSet), Piecewise, logic, matrices plus expressions, Indexed, and special functions (erf, erfinv, gamma, lowergamma, uppergamma, beta, besseli/besselk, hyper, meijerg, zeta, elliptic). Its test pass rate is a lagging integration metric for those clusters.
- App priority: the finite stats slice (FiniteRV, Die, Coin, Bernoulli, Binomial, Hypergeometric, P, E, given, density) needs only core, Relational evaluation, Piecewise, FiniteSet/Dict/Lambda and xreplace, plus solveset for given(Eq). It can ship long before integrals, and the app needs nothing else from stats.
- App priority: the geometry figure kit with rational coordinates needs Matrix rank/rref over QQ, sqrt and core arithmetic. simplify/nsimplify are identities on Rationals, so a semantics-preserving fast path removes Paul's 'slow, simplifies eagerly' cost without waiting for a full simplify port. Circles need solve on degree-2 systems.
- printing: printers needed for Permutation/Cycle (cyclic notation with trailing (size-1), perm_cyclic flag, deprecated print_cyclic), PermutationGroup, FreeGroupElement (r**2*s*r**-1), Partition, RandomDomain and Probability/Expectation/Variance (str/latex/pretty/repr).
- Testing infrastructure: SymPy 1.14's sympy.test() delegates to pytest (testing/runtests_pytest.py) and raises NoPytestError without it. The pinned venv has no pytest, numpy or scipy, so the planned PyO3 test run needs pytest installed, and stats sampling tests will skip unless scipy/numpy are added.
- Benchmarks: top-level sympy/benchmarks gives a ready perf corpus. bench_symbench R1-R11 covers core arithmetic, subs, expand and simplify; bench_meijerint has about 130 integrate/transform cases, many of them probability densities and exactly the stats hot path; bench_discrete_log covers ntheory bigint. It should seed the cross-implementation perf harness.

### `combinatorics`

21,444 source lines, 175 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: none.

Permutations and computational group theory: Permutation/Cycle; PermutationGroup with base and strong generating set (BSGS) algorithms; finitely presented groups (free groups, Todd-Coxeter coset enumeration, Knuth-Bendix rewriting, Reidemeister-Schreier, low-index subgroups); homomorphisms; polycyclic groups; Butler-Portugal tensor canonicalisation; tables of transitive subgroups of S1-S6 used for Galois groups; Polyhedron; and combinatorial objects (set and integer partitions, RGS, Prufer codes, Gray codes, Subset ranking, group counts, Schur numbers). About 21.4k source lines, of which about 8.2k are code once docstrings are removed (perm_groups.py 2235, permutations.py 824, fp_groups.py 705, free_groups.py 546, coset_table.py 513, tensor_can.py 442, polyhedron.py 352, homomorphisms.py 301).

**Why this strategy.** The algorithms are deterministic, integer-only and textbook (Holt's Handbook of Computational Group Theory), and they map directly onto Vec<u32> and Vec<Vec<Option<u32>>>. Tests check exact intermediate coset tables and exact canonical forms, so a structurally close port is the cheapest way to pass them. No permissive Rust library covers Schreier-Sims, Todd-Coxeter or Knuth-Bendix; algebraeon-groups is GPL-3.0 and GAP is GPL.

**Key algorithms.**

- Array-form permutation kernel: _af_rmul/_af_rmuln (composition), _af_invert, _af_pow (exponentiation by squaring), _af_parity, _af_commutes_with (permutations.py:21-243)
- Permutation ranking: lexicographic rank/unrank_lex (permutations.py:1873, 2926), Myrvold-Ruskey rank_nonlex/unrank_nonlex (1777-1845), Trotter-Johnson rank/unrank/next (2502-2635), inversion vector and from_inversion_vector (2450, 2879), inversions counted by merge sort (_merge 3031), josephus (2830), precedence/adjacency distances (2637-2830)
- Schreier-Sims: deterministic schreier_sims/_schreier_sims (perm_groups.py:3494-3550); schreier_sims_incremental with straight-line programs (3551, Holt's SCHREIERSIMS); randomised schreier_sims_random (3736); baseswap (476); sifting via util._strip and coset_factor (1136); coset_rank/coset_unrank (1268/1315)
- Orbit machinery: _orbit, _orbits, _orbit_transversal, _stabilizer (perm_groups.py:5032-5256), schreier_vector (3868), orbit_rep (2848), union-find minimal_block (349-440, 2529)
- Product-replacement random elements: random_pr and _random_pr_init (3426, 289); Monte Carlo is_alt_sym (1928)
- Backtrack subgroup_search with base changes (3992, about 250 lines), centralizer (970), normal_closure (2709), derived and lower central series, sylow_subgroup with _sylow_alt_sym and _p_elements_group (4296-4575), composition_series (724), abelian_invariants (1807), conjugacy_classes (2678), generate_dimino and generate_schreier_sims (1547, 1605), strong_presentation and presentation through fp groups (4754, 4876), polycyclic_group (4987)
- Todd-Coxeter coset enumeration: HLT coset_enumeration_r with look_ahead (coset_table.py:968, 653); Felsch coset_enumeration_c with deduction stack (1203); coincidence/merge/rep with path compression (425-596); the 'modified' variants that track words for Reidemeister-Schreier (880-1200); standardize and compress (751-833)
- Low-index subgroups (fp_groups.py:720-946); Reidemeister-Schreier presentation with Tietze elimination_technique_1/2 and simplify_presentation (947-1350)
- Knuth-Bendix completion RewritingSystem.make_confluent with an automaton (rewritingsystem.py, rewritingsystem_fsm.StateMachine)
- Free-group word algebra: reduction, cyclic_reduction, eliminate_word, subword, power_of (free_groups.py)
- Polycyclic Collector collection algorithm, induced pcgs, exponent vectors and depth (pc_groups.py)
- Homomorphism kernel and image computed through coset tables, plus group_isomorphism (homomorphisms.py)
- Butler-Portugal canonicalisation: double_coset_can_rep, canonical_free, canonicalize, dummy_sgs, bsgs_direct_product, get_symmetric_group_sgs, riemann_bsgs (tensor_can.py)
- Hard-coded transitive subgroup tables S1..S6TransitiveSubgroups, including PGL2F5 (galois.py)
- Prufer encode/decode to_prufer/to_tree with rank/unrank (prufer.py); Gray code generate/rank/unrank, bin_to_gray/gray_to_bin, graycode_subsets (graycode.py); binary, lexicographic and Gray-order subset ranks (subsets.py)
- RGS_enum/RGS_rank/RGS_unrank with a generalised RGS count table, IntegerPartition next_lex/prev_lex, as_ferrers, random_integer_partition (partitions.py)
- groups_count from Holder's formula and OEIS tables (group_numbers.py); Schur number bound and schur_partition (schur_number.py); pgroups of the platonic solids built at import time (polyhedron.py:_pgroup_calcs, _pgroup_of_double)

**Data structures.**

- Permutation: array form Vec<u32> whose length is part of the identity, with lazily cached cyclic_form, rank and cycle_structure
- Cycle: sparse map from point to image
- PermutationGroup: generators plus a lazily built BSGS (base Vec<u32>, strong_gens, basic_orbits, transversals as per-level maps from point to perm, Schreier vectors as i32 arrays, straight-line-program dicts) and property flags
- CosetTable: table Vec<Vec<Option<usize>>> with columns for generators and inverses, parent array p, omega, deduction stack, max_cosets
- FreeGroup/FreeGroupElement: words as Vec<(generator index, i64 exponent)> (array form) or letter form
- RewritingSystem: rules map, reduction automaton states
- Collector: relative orders, pc presentation dict, pcgs
- Polyhedron: corners tuple, faces set, pgroup
- IntegerPartition (Integer and Tuple), Partition (a FiniteSet of FiniteSets), Prufer (prufer_repr and tree_repr), GrayCode (n, current), Subset (subset and superset)

**Hard dependencies.**

- core
- ntheory
- functions.combinatorial
- utilities
- sets

**Python features that are hard to port.**

- Operator overloading with unusual meanings: __xor__/__rxor__ for conjugation and image, __add__/__sub__ for rank arithmetic, __invert__, list*Permutation coercion through __rmul__
- Cycle is a dict subclass with __missing__ and chained __call__
- Mutable caches on Basic subclasses (PermutationGroup._order and _base, Permutation._rank and _cyclic_form, Prufer._rank) and in-place mutation (Polyhedron.rotate, GrayCode iteration with skip())
- Generators as public API: generate_dimino, generate_schreier_sims, generate_gray, RGS_enum
- Test hooks injected as keyword arguments (_random_prec, _random_prec_n, seed=list)
- Return type depends on path (int vs Integer from order())
- multipledispatch _eval_is_eq registration for Permutation
- utilities.magic.pollute (vfree_group/vfp_group) writes into the caller's globals
- Module-level computation at import (_pgroup_calcs builds the five platonic solids)

**Behaviour a port must reproduce.**

- Permutation products compose left to right: (p*q)(i) == q(p(i)) (permutations.py:1323), while _af_rmul(a,b) is the reverse (a[b[i]]). p^q is the conjugate q**-1*p*q, i^p == p(i), p+n and p-n mean unrank_lex(rank±n), and ~p is the inverse.
- Size is part of a permutation's identity: Permutation(1,2) != Permutation(1,2,size=5) and Permutation([0,1]) != Permutation([0,1,2]) (probed). Multiplication pads the shorter operand. The PermutationGroup constructor resizes every generator to the largest degree, removes duplicates (dups=True) and drops identities when there is more than one generator.
- Permutation is an Atom with args == () and _hashable_content == tuple(array_form). p(x) with a symbolic x raises ValueError, but p.apply(x) returns an unevaluated AppliedPermutation(Expr).
- Constructor rules: array form must contain 0..n-1, otherwise 'Integers 0 through %s must be present.'; cyclic form may omit singletons; Permutation(n) is the identity of size n+1; Permutation([[1,2]], size=10) is allowed.
- Default printing is cyclic: 'Permutation(1, 2)', and a trailing '(size-1)' is shown for fixed points, e.g. '(9)(0 1 2 3 4)'. init_printing(perm_cyclic=False) switches to array form.
- PermutationGroup.order() returns a SymPy Integer (factorial(n), or factorial(n)/2) when _is_sym/_is_alt is already known, and a Python int from prod(basic_transversals) otherwise. The result is cached, so the type depends on call history (probed: SymmetricGroup(5).order() gives Integer, a generic group gives int).
- PermutationGroup keeps mutable lazy caches (_base, _strong_gens, _order, _is_sym, and others). Tests check BSGS only through testutil._verify_bsgs, so a port may choose its own base.
- Randomised paths are on by default: random_pr, schreier_sims_random, is_alt_sym, is_primitive(randomized=True), minimal_blocks(randomized=True), normal_closure, sylow_subgroup, and FpGroup.random (which uses stdlib random, not sympy.core.random). Tests make them deterministic by injecting _random_prec dicts or seed=list(...) into _randrange, so a pluggable RNG with a replay mode is required.
- CosetTable tests inspect intermediate state after each scan_and_fill/define: c.table with None holes, the parent array c.p, c.omega and c.n (test_coset_table.py:17-60). Column order is [x, x**-1, y, y**-1, ...] and coset definition order must match HLT and Felsch exactly. Exceeding max_cosets (default 4096000) raises ValueError.
- Cycle is a dict subclass: __missing__ returns i, and calls chain, as in Cycle(1,2)(2,3).
- Polyhedron.rotate mutates _corners in place (a mutable Basic), and reset() restores it. Faces are minlex-normalised and stored as a FiniteSet.
- GrayCode.generate_gray is a generator whose state changes when skip() is called during iteration.
- Partition subclasses FiniteSet: Partition([1,2],[3]) prints as 'Partition({3}, {1, 2})'.
- FreeGroupElement is a tuple subclass of (symbol, exp) pairs and is not a Basic. free_group() caches groups globally. vfree_group injects names into the caller's globals through utilities.magic.pollute.
- tensor_can.canonicalize uses fixed conventions: the last two array entries encode the sign, and msym must be 0, 1 or None, otherwise ValueError.

**Rust prior art.**

- butler-portugal 0.3.0 (MIT), Butler-Portugal tensor canonicalisation: reference and cross-check for tensor_can
- permutation 0.4.1 (MIT/Apache-2.0): basic permutation utilities, probably simpler to write our own
- integer-partitions 0.1.1 (MIT), gray-codes 0.1.1 (MIT/Apache-2.0): small enumeration crates
- rand_mt 6.1.0 (MIT/Apache-2.0): MT19937, only if bit-exact reproduction of Python's random is ever wanted
- num-bigint / dashu (MIT/Apache-2.0): group orders and ranks above u64 (|S_21| > 2^64)
- Not usable: algebraeon-groups (GPL-3.0-only), GAP (GPL)

**How its tests check results.** About 1197 asserts over 175 test functions. They use structural == on Permutations, lists and ints (616 references to Permutation(...)), exact list-of-list coset tables including intermediate states, exact canonical forms from tensor_can, property checks through testutil (_verify_bsgs, _verify_centralizer, _verify_normal_closure, 37 uses), raises(ValueError/TypeError) (72), and str/srepr of permutations (26). There are no symbolic simplify comparisons. In the pinned environment 172 pass, 3 xfail and 0 time out, in 128 s total. The slowest are test_coset_enumeration (71 s), test_presentation (12 s), test_order in fp_groups (8 s), test_induced_pcgs (6 s) and test_rewriting (6 s).

**Notes.** The cluster splits into four sub-units that can run in parallel once the kernel exists: (a) permutations.py plus the leaf objects (prufer, graycode, subsets, partitions, group_numbers, schur_number), about 2 person-weeks; (b) util plus perm_groups BSGS core and advanced routines plus named_groups/generators/group_constructs, about 6-8 weeks; (c) free_groups, coset_table, rewritingsystem, fp_groups, homomorphisms and pc_groups, about 4-6 weeks; (d) tensor_can, galois and polyhedron, about 2-3 weeks. The total is roughly 3 person-months, at the upper end of L. Except for Polyhedron corners and free-group generator names (Symbols) and Partition (FiniteSet), none of this touches symbolic expressions. These are the best early, parallel porting targets in SymPy: integer-only, deterministic, well tested, and fast to verify against the oracle. The pure-Python coset enumeration test takes 71 s, so large speed-ups are likely. The app does not use combinatorics (its 'binomial' is functions.combinatorial).

### `stats`

16,651 source lines, 225 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: roadmap.

Symbolic probability. Provides RandomSymbol and probability spaces (finite, discrete, continuous, joint, compound, matrix, random-matrix ensembles); the queries P, E, variance, density, cdf, quantile, median, moments, entropy, given and where; unevaluated Probability/Expectation/Variance/Covariance/Moment objects; about 50 continuous, 11 discrete and 10 finite named distributions; stochastic processes (discrete and continuous Markov chains, Bernoulli, Poisson, Wiener and Gamma processes); and error propagation. About 16.7k source lines and 6.9k code lines (crv_types 1134, stochastic_process_types 1064, rv.py 766, crv.py 370, joint_rv_types 357, symbolic_probability 348, frv 341, frv_types 330).

**Why this strategy.** Every output is a SymPy expression or set produced by composing other units, so parity requires the same control flow: the linearity split, the MGF path, closed-form overrides, and solveset or inequality reduction for where(). The multiple-inheritance PSpace/Domain hierarchy (e.g. SingleContinuousPSpace(ContinuousPSpace, SinglePSpace)) should become Rust enums and traits, and the distribution catalogue a data table (argnames, check, pdf, set, optional closed forms), but behaviour should follow SymPy line by line.

**Key algorithms.**

- Probability-space plumbing: pspace() merges distinct pspaces into IndependentProductPSpace, independence assumed (rv.py:388, 624); given() builds a conditional space and swaps symbols with rs_swap (rv.py:686-770); an Eq condition on one RV is resolved with solveset plus substitution; dependent/independent/pspace_independent (1482-1570)
- Expectation.doit distributes over Add (linearity) before passing to the pspace; Probability.doit handles Not as 1 - P, a given condition via given(), and a RandomSymbol as the given condition giving a BernoulliDistribution (symbolic_probability.py:74-125, 252-292)
- Finite spaces enumerate the itertools.product of all domains: compute_density sums probabilities into a FiniteDensity dict, compute_expectation sums Piecewise((p*val, cond)), probability filters ConditionalFiniteDomain, compute_cdf sorts keys with SymPy's < (frv.py:234-512). The symbolic-size path builds a Sum (frv.py:443)
- Continuous: compute_expectation integrates pdf*expr over the domain, nesting Integrals for ConditionalContinuousDomain with limits from reduce_rational_inequalities (crv.py:54-137, 551-570); where() goes through reduce_rational_inequalities_wrap; density of g(X) by change of variables, solveset(expr - y) and the sum of fx(g)*|dg/dy| (crv.py:526-541); a Ne condition becomes 1 - P(Eq)
- Moments by Taylor series of the moment generating function: when the expression is polynomial in X and _moment_generating_function exists, E uses series(mgf) coefficients times k! instead of integrating (crv.py:257-277, drv.py:150-176)
- Closed-form overrides per distribution (_cdf 29, _characteristic_function 24, _moment_generating_function 28, _quantile 5 in crv_types). Otherwise CDF, CF and MGF come from integrate/summation and are cached with @cacheit (crv.py:186-300, drv.py:57-145)
- Discrete: summation of the pmf with floor(z) for the CDF (drv.py)
- Joint: marginal_distribution by integrate/summation over the other components (joint_rv.py:89-108); MultivariateNormal/T/Beta/Dirichlet/Multinomial densities using Matrix det and inv (joint_rv_types.py)
- Compound distributions by marginalising the parameter RV (compound_rv.py)
- Markov chains: communication_classes via strongly_connected_components, fundamental_matrix, absorbing_probabilities, decompose/canonical_form, stationary_distribution via linsolve (stochastic_process_types.py:964-1475); multistep probabilities via matrix powers; Poisson/Wiener/Gamma process queries
- error_prop: first-order (Taylor) variance propagation (error_prop.py); random-matrix normalisation constants and joint eigenvalue densities (random_matrix_models.py)

**Data structures.**

- RandomSymbol(symbol, pspace) node, plus RandomIndexedSymbol and RandomMatrixSymbol
- PSpace kinds: Single, IndependentProduct and Conditional, crossed with Finite, Discrete, Continuous, Joint, Compound and Matrix
- RandomDomain kinds: SingleDomain, ProductDomain, ConditionalDomain (the condition as a Boolean), FiniteDomain over a FiniteSet of (symbol, value) tuples
- Distribution nodes with named args; Lambda densities over Dummy variables
- FiniteDensity (value to probability map)
- Markov chain: state space (FiniteSet or Range), transition or generator matrix (ImmutableMatrix or MatrixSymbol)

**Hard dependencies.**

- core
- sets
- logic
- functions.elementary
- functions.special
- functions.combinatorial
- integrals
- concrete
- series
- polys
- solvers
- simplify
- matrices
- matrices.expressions
- tensor

**Python features that are hard to port.**

- NamedArgsMixin.__getattr__ gives dynamic parameter attributes
- functools.singledispatch for is_random and the sampling backends
- @cacheit on methods with **kwargs (compute_cdf, compute_quantile, and others) uses the global SymPy cache
- Class attribute 'set' overridden per distribution (Interval, Range, FiniteSet)
- property(lambda self: self.args[i]) accessors; FiniteDensity(dict) is callable
- Runtime optional imports (import_module for scipy/numpy/pymc) and deprecation warnings
- Generators for sample_iter

**Behaviour a port must reproduce.**

- RandomSymbol equality is structural over (symbol, pspace), so two identically built Normal('X',0,1) are the same variable: E(X1*X2) == 1, not 0 (probed). Hash-consing reproduces this automatically. Same-name RVs with different parameters are silently treated as independent (E(X1+X3) == 1).
- RandomSymbol sets is_Atom=True and free_symbols == {self}, yet it has args (symbol, pspace). Generic tree walkers must not descend into the pspace.
- Polynomial expectations take the MGF series path when the distribution defines _moment_generating_function. E(Normal(mu,sigma)) == mu in 0.03 s comes from that, not from integration. Non-polynomial expressions integrate.
- Results are whatever integrate/summation/solveset return verbatim. P(Normal(0,1) > 1) gives sqrt(2)*(-sqrt(2)*sqrt(pi)*erf(sqrt(2)/2) + sqrt(2)*sqrt(pi))/(4*sqrt(pi)) unsimplified (probed). quantile returns sets such as Complement(Intersection(FiniteSet(...), Interval(-oo, oo)), FiniteSet(mu)). median(Die) returns FiniteSet(3, 4). An unevaluable E returns Sum/Integral objects, e.g. E(Binomial(n symbolic)) gives Sum(Piecewise(...), (_k, 0, n)).
- Finite results are Python dicts: density(X).dict, and cdf(Binomial) returns a plain dict sorted by key using SymPy <. FiniteDensity is a callable dict subclass.
- Product-space enumeration is exponential in the number of finite RVs. That is algorithmic and must be kept for parity, but it is a cost risk in the submit path.
- _value_check raises ValueError only when the fuzzy condition is definitely False; symbolic parameters pass silently.
- Distribution parameters are read as attributes through NamedArgsMixin.__getattr__ and _argnames (dist.mean, dist.std).
- given(expr, Eq(rv, v)) with a single RV short-circuits to substitution via solveset. Otherwise it builds a ConditionalDomain and swaps in new RandomSymbols.
- P(U > V) for two independent Uniform(0,1) did not finish within 60 s in the pinned SymPy (probed; P(..., evaluate=False) is instant). Multivariate conditional integration is a performance trap.
- Unevaluated forms: E(X**2, evaluate=False) gives Expectation(Y**2), and Probability(X>1).rewrite(Integral) gives Integral(... (_z, 1, oo)) with Dummy variables. Tests compare these with dummy_eq.
- sample() defaults to library='scipy' and raises ValueError('Failed to import scipy') when scipy is missing (probed).

**Rust prior art.**

- statrs 0.19.1 (MIT): numeric distributions and special functions, useful as a numeric oracle or cross-check, not symbolic
- cdflib 0.4.4 (MIT): numeric CDFs and inverses
- puruspe 0.4.4 / special 0.14.2 (MIT/Apache-2.0): numeric special functions for spot checks
- No symbolic-probability prior art in Rust

**How its tests check results.** About 1411 asserts over 225 test functions. They use structural == against closed forms involving erf, gamma, lowergamma, besseli, hyper, meijerg, Piecewise, Integral and Sum. Many comparisons wrap results in simplify(...) (98) or dummy_eq (40); raises appears 211 times, with a few numeric .n()/N checks (23) and verify_numerically (2). Sampling tests skip without scipy/numpy/pymc. In the pinned environment 212 pass, 3 xfail and 9 skip, plus one harness artefact (an XFAIL pymc test reported as XPass), in about 264 s total. The slowest are test_GeometricDistribution (33.5 s), test_JointPSpace_marginal_distribution (20.6 s), test_exponential and test_gamma (15.3 s each), test_normal_CompoundDist (14.7 s) and test_ideal_soliton (14.4 s).

**Notes.** stats is the furthest-downstream unit in SymPy: nothing outside stats imports it, but it needs integrals (including meijerint), concrete.summation, series, polys.poly, solveset/inequalities/linsolve, sets, matrices and special functions. Its tests will fail for reasons owned by other units until those are at parity, so it should be ported late and gated in CI by slices. The exception is the finite slice (rv.py framework, symbolic_probability, rv_interface, frv.py, frv_types.py, about 1.8k code lines). That slice needs only core arithmetic, Relational evaluation, Piecewise, FiniteSet/Dict/Lambda and xreplace (plus solveset for given(Eq)), it is exactly what the app roadmap asks for (FiniteRV, P, given, Binomial, Rational), and it can be ported early (about 3-4 weeks). For the app, Markov steady state is better served by matrices (eigenvects or linsolve); DiscreteMarkovChain.stationary_distribution is an optional alternative.

### `stats.sampling`

371 source lines, 17 test functions. Difficulty **M**. Strategy: **redesign**. App relevance: none.

Numerical sampling backends for sympy.stats distributions. sample_scipy.py (the default), sample_numpy.py and sample_pymc.py map each distribution to a scipy/numpy/pymc sampler; arbitrary continuous pdfs are lambdified into a scipy rv_continuous subclass. Distribution.sample (rv.py:1599) dispatches on library= and seed. About 371 source lines.

**Why this strategy.** The Python module only glues SymPy to scipy/numpy/pymc, which a pure-Rust library cannot depend on. Replace it with native samplers on rand and rand_distr for the standard distributions, plus generic inverse-transform or rejection sampling for handmade pdfs, using a compiled numeric evaluator for the pdf and numeric CDF inversion. Keep library= as a compatibility keyword. During the transition the PyO3 layer may still delegate to scipy/pymc, because tests only check membership.

**Key algorithms.**

- singledispatch do_sample_scipy/do_sample_numpy/do_sample_pymc registries keyed on distribution class, with parameter re-mapping (e.g. scipy expon scale = 1/rate) (sampling/*.py)
- Generic continuous fallback: lambdify the pdf into scipy.stats.rv_continuous._pdf, so scipy does numerical CDF integration and root-finding for ppf (sample_scipy.py:24-40)
- Finite distributions sample from the sorted CDF table (FinitePSpace.sample using sorted_cdf)
- sample_iter / sampling_P / sampling_E / sampling_density do Monte Carlo estimates (rv.py:1240-1480)

**Hard dependencies.**

- stats
- utilities

**Behaviour a port must reproduce.**

- The default library is 'scipy', and a missing scipy raises ValueError('Failed to import scipy'). Unsupported library names raise NotImplementedError.
- seed accepts an int or a numpy Generator. Exact sample streams cannot match numpy and are not tested for exact values.
- size=() returns a scalar; any other size returns an ndarray shape. pymc returns draws reshaped to size.

**Rust prior art.**

- rand 0.10 and rand_distr 0.6 (MIT OR Apache-2.0): standard samplers
- statrs 0.19 (MIT): inverse CDFs for inverse-transform sampling
- cdflib 0.4 (MIT): CDF inverses

**How its tests check results.** 17 test functions that only check membership (sam in X.pspace.domain.set), range or shape, plus raises(NotImplementedError). They all skip when scipy/numpy/pymc are missing; in the pinned venv 16 skip and 1 passes.

**Notes.** Most of the effort is the generic sampler for arbitrary symbolic pdfs, which needs fast numeric evaluation of expressions (a lambdify equivalent, shared with plotting and codegen) plus numeric integration and root-finding. The per-distribution mappings are quick.

### `geometry`

12,060 source lines, 117 test functions. Difficulty **L**. Strategy: **port faithfully**. App relevance: roadmap.

Exact symbolic Euclidean geometry in 2D and 3D: Point/Point2D/Point3D; Line, Ray and Segment in 2D and 3D; Polygon, Triangle and RegularPolygon; Ellipse and Circle; Parabola; Plane; parametric Curve; and utilities (intersection, convex_hull, closest_points, farthest_points, centroid, are_similar, idiff). Entities are Basic and GeometrySet participates in Set algebra. About 12.1k source lines and 3.6k code lines (polygon 861, line 737, ellipse 474, point 352, plane 316, entity 306, util 290).

**Why this strategy.** Results are exact SymPy objects compared structurally, and their forms come from simplify, solve and rref, so control flow must match SymPy, including the eager simplify and the Float-to-Rational conversion. Fast paths that keep the semantics are allowed: when all coordinates are Rational, simplify and nsimplify are identities, so they can be skipped and exact rational arithmetic used for orientation, rank and rref. That is where Paul's 'slow, simplifies eagerly' complaint is fixed without changing outputs.

**Key algorithms.**

- Point constructor: Float coordinates go through nsimplify(rational=True) then simplify when evaluate=True; padding and dimension morphing in _normalize_dimension; every arithmetic op calls simplify() per coordinate (point.py:109-296)
- affine_rank and is_collinear via Matrix.rank with iszerofunc abs(x.n(2)) < 1e-12 for numbers (point.py:315-336); is_concyclic; are_coplanar
- Linear-entity intersection: case split on affine rank (collinear overlap of rays and segments via _span_test, coplanar solve via rref(simplify=True) of [d1, -d2 | p2-p1], skew means empty), with a solve()-based parametric containment check when Floats are present (line.py:390-570)
- Polygon construction removes consecutive duplicates and collinear or folded vertices and returns Triangle, Segment or Point when degenerate (polygon.py:122-164). Also: signed shoelace area (166), centroid (358), first and second moments of area, is_convex, ray-casting encloses_point (707), intersection (901), cut_section (958), polygon distance (1093)
- Polygon equality up to rotation and reversal: _hashable_content relabels vertices with ordered() and takes the least rotation (Booth's algorithm, utilities.iterables.least_rotation) (polygon.py:1312)
- Triangle centres and lines: circumcenter, incenter (side-length weights), orthocenter, centroid, medians, altitudes, bisectors, excenters, exradii, medial, nine_point_circle, eulerline; _sss/_sas/_asa constructors with angles in degrees (polygon.py:2053-2890)
- Ellipse and Circle: intersection by solve([conic, other], [x, y], set=True) with ordered() output (ellipse.py:615-695), tangent_lines (1224), normal_lines (859), is_tangent (697), Circle through 3 points (1541), circumference via elliptic_e, area moments
- Utilities: convex_hull by Andrew's monotone chain with _orientation (util.py:360), closest_points by sweep line with an exact hypot (284), farthest_points by rotating calipers (474), idiff (562), multi-entity intersection (635)
- Plane intersections and projections via linsolve/solve (plane.py), Parabola (parabola.py), Curve with length via integrate (curve.py)

**Data structures.**

- Point2D/Point3D: a Tuple of Expr coordinates
- LinearEntity(p1, p2) variants Line, Ray and Segment in 2D and 3D (direction = p2 - p1)
- Polygon(vertices...), Triangle, RegularPolygon(center, radius, n, rot)
- Ellipse(center, hradius, vradius), Circle(center, radius)
- Parabola(focus, directrix), Plane(p1, normal_vector), Curve(functions, (t, a, b))

**Hard dependencies.**

- core
- simplify
- matrices
- solvers
- functions.elementary
- sets
- utilities
- multipledispatch
- integrals

**Python features that are hard to port.**

- __new__ returning instances of other classes (Polygon to Triangle/Segment/Point, Circle to Segment, Point to Point2D/Point3D)
- Keyword-driven alternative constructors (sss=, asa=, sas=, eccentricity=, normal_vector=, slope=, angle=, direction_ratio=, n= for regular polygons)
- multipledispatch registrations into sets' intersection_sets/union_sets
- Use of global_parameters.evaluate; warnings.warn for dimension morphing
- _svg/_repr_svg_ rendering methods (Jupyter)
- __cmp__ with a class-name ordering table

**Behaviour a port must reproduce.**

- Float handling is inconsistent: Point(0.5, 0.1) gives Point2D(1/2, 1/10), but Point(1,2) + (0.1,0.2) gives Point2D(1.10000000000000, 2.20000000000000), because the tuple is converted with evaluate=False, while translate() and Point+Point rationalise (probed).
- Every coordinate operation calls simplify, so outputs are simplify's exact forms. incenter of (0,0),(4,0),(1,3) is Point2D(4*(1 + sqrt(10))/(sqrt(10) + 4 + 3*sqrt(2)), 12/(sqrt(10) + 4 + 3*sqrt(2))), and Triangle.bisectors() takes 1.6 s and produces large nested radicals (probed).
- Constructors return other classes: Polygon with collinear or duplicate vertices becomes a Triangle, Segment or Point; Circle through 3 collinear points becomes a Segment2D; Point(...) becomes Point2D or Point3D; Triangle(asa=(30, 1, 30)) takes degrees (all probed).
- Equality is structural and ordered: Line(a,b) != Line(b,a) (use .equals), and Segment(p,q) != Segment(q,p), with endpoints not normalised. Polygon == is up to rotation and reversal (probed).
- Collinearity uses a numeric heuristic, abs(x.n(2)) < 1e-12. Points 1e-13 off a line are reported collinear (Point.is_collinear((0,0),(1,1),(2,2+10**-13)) is True, probed), so results are not exact for near-degenerate input.
- Line intersection with a symbolic slope returns the generic solution without side conditions: Line((0,0),(1,x)) ∩ Line((0,1),(1,0)) gives [Point2D(1/(x+1), x/(x+1))] (probed).
- Ellipse/Ellipse intersections come back as unnested radicals in ordered() order, e.g. sqrt(30*sqrt(2377)/101761 + 4623/101761) (probed), and depend on solve's exact output.
- Polygon.area is signed: clockwise vertex order gives a negative area (probed: -1).
- tangent_lines returns Line2D through the given point and an arbitrary second point on the tangent, e.g. Line2D(Point2D(3, 4), Point2D(103, -71)) (probed). Tests compare these Line objects structurally.
- Imaginary coordinates raise ValueError and non-Expr coordinates raise TypeError. Mixing 2D and 3D points pads with zeros and emits UserWarning (on_morph='warn'); tests use warns().
- GeometrySet hooks into Set algebra through multipledispatch intersection_sets/union_sets registrations (entity.py:555-595). GeometryEntity.__cmp__ sorts by an ordering_of_classes list.
- random_point(seed) uses sympy.core.random.

**Rust prior art.**

- robust 1.2 (MIT/Apache-2.0, georust) and geometry-predicates 0.3 (MIT): floating-point robust predicates, useful only for a float fast path, not for exact symbolic results
- geo (MIT/Apache-2.0): reference float implementations of convex hull and closest pair
- No exact symbolic-geometry crate

**How its tests check results.** About 1311 asserts over 117 test functions. They use structural == on entity objects and on ordered lists of Points, raises (101), warns(UserWarning) (17), and a few numeric .n()/evalf checks (21). There is no dummy_eq and little simplify (6). In the pinned environment 111 pass, in about 129 s total. The slowest are test_polygon (25.3 s), test_bisectors (17.0 s), test_free_symbols in curve (13.6 s), test_second_moment_of_area (13.0 s) and test_ellipse_geom (9.7 s).

**Notes.** The app roadmap's figure kit (Point, Segment, Line, Ray, Circle, Triangle, Polygon, intersection, distance, angle_between, is_parallel, is_perpendicular, is_similar, tangent_lines, triangle centres, reflect/rotate/translate/scale, Polygon equality up to vertex order) is mostly point.py, line.py, polygon.py and parts of ellipse.py. With rational coordinates these need only core arithmetic, sqrt, Matrix rank and rref, and the Rational fast path, so they can be ported before simplify and solve are complete. Circle intersections and tangent_lines need solve on degree-2 systems; Ellipse/Ellipse needs quartics. Downstream users: integrals/intpoly.py (polytope_integrate uses Polygon, Segment2D, Point), vector/parametricregion.py and implicitregion.py, physics/optics/utils.py (Ray3D, Plane), plotting, and physics/continuum_mechanics/beam.py (isinstance only).

### `sandbox`

80 source lines, 1 test functions. Difficulty **S**. Strategy: **port faithfully**. App relevance: none.

Experimental IndexedIntegral (sandbox/indexed_integrals.py, 36 code lines): a subclass of Integral that replaces Indexed integration variables (A[i]) with Dummies, integrates, and substitutes back.

**Why this strategy.** It is a 36-line subclass and a straightforward port once Integral and Indexed exist in Rust. Until then the Python file can run unchanged on the Rust core through PyO3.

**Key algorithms.**

- _indexed_process_limits maps each Indexed limit variable to a Dummy; doit() calls Integral.doit and then xreplaces the Dummies back (indexed_integrals.py)

**Hard dependencies.**

- integrals
- tensor

**Behaviour a port must reproduce.**

- Printing shows the Dummy names, e.g. Integral(_A[i], _A[i]). Distinct indices are distinct variables: IndexedIntegral(A[j], A[i]).doit() == A[i]*A[j].
- Works with Idx indices as well as integer Symbols.

**How its tests check results.** 1 test function with 12 structural == asserts on doit() results (A[i]**2/2, -cos(A[i])). It passes in under 0.1 s.

**Notes.** Port it last. Its only value is completeness and its single test.

### `benchmarks`

478 source lines, 0 test functions. Difficulty **S**. Strategy: **redesign**. App relevance: none.

Standalone timing scripts with no tests: bench_symbench.py (SymPy 'symbench' R1-R11 and S1 micro-benchmarks: nested complex arithmetic and as_real_imag, recursive Hermite with expand, equality, set uniqueness, 100 simplify calls, 10^4 subs with random floats, a Riemann sum of 10^4 subs, srange with evalf, random complex numbers, expand((x+y+z+1)**7 * (...+1))); bench_meijerint.py (about 130 integrate and Laplace/Fourier/Mellin transform calls, many on probability densities: normal, exponential, beta, chi, Dagum, F, Rice, Laplace); and bench_discrete_log.py (ntheory discrete_log: trial multiplication, Shanks, Pollard rho, Pohlig-Hellman on prime fields up to about 10^24 and 2**118).

**Why this strategy.** There is no library behaviour to port. Turn the workloads into (a) Rust criterion benchmarks and (b) a cross-implementation harness that runs the same workloads against SymPy 1.14 and rsympy through the PyO3 shim, checking results against the oracle while timing. That gives the 'faster, less memory' goal a fixed corpus.

**Key algorithms.**

- Harness loops with time() and clear_cache() (bench_meijerint.py main), exec of benchmark strings
- Workload definitions only. The algorithms under test live in core, simplify, polys, integrals (meijerint, transforms) and ntheory (residue_ntheory)

**Hard dependencies.**

- core
- simplify
- polys
- integrals
- ntheory
- functions.special

**Behaviour a port must reproduce.**

- bench_meijerint runs clear_cache() before each case, so the timings measure cold-cache cost; Rust comparisons should do the same.
- bench_symbench R7 and R11 use sympy.core.random.random, so the inputs are nondeterministic unless seeded.

**How its tests check results.** No tests (0 test functions). These are scripts run by hand and report timings only.

**Notes.** Natural seed corpus for the performance dashboard. symbench measures core arithmetic, subs, expand and simplify; meijerint measures the integration paths that stats depends on; discrete_log measures ntheory big-integer paths. Similar benchmark folders exist under core, integrals, matrices, polys and solvers and belong to those units.

