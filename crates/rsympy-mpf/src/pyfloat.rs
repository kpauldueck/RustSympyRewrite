//! The piece of CPython's complex arithmetic that mpmath's pure-Python
//! code relies on: `complex ** float` (`mpc_nthroot_fixed` takes its
//! starting estimate from it). The libm calls are the platform's, as
//! CPython's are.

/// Python raised `OverflowError`.
#[derive(Debug)]
pub(crate) struct Overflow;

/// `complex(re, im) ** e` for a float exponent `e` that is not a small
/// integer (CPython's `_Py_c_pow(a, (e, 0.0))`; `complex_pow` raises
/// `OverflowError` when the result overflows, which here means `hypot`
/// did). `e > 0` is assumed, so the `EDOM` case of a zero base with a
/// negative exponent does not arise.
pub(crate) fn complex_pow_float(re: f64, im: f64, e: f64) -> Result<(f64, f64), Overflow> {
    debug_assert!(
        e > 0.0 && !(e == e.floor() && e.abs() <= 100.0),
        "small integer exponents take c_powi"
    );
    if re == 0.0 && im == 0.0 {
        return Ok((0.0, 0.0));
    }
    let vabs = re.hypot(im);
    let len = vabs.powf(e);
    let at = im.atan2(re);
    let phase = at * e;
    let (r, i) = (len * phase.cos(), len * phase.sin());
    // _Py_ADJUST_ERANGE2: an infinite part (or hypot's ERANGE) raises.
    if r.is_infinite() || i.is_infinite() || vabs.is_infinite() {
        return Err(Overflow);
    }
    Ok((r, i))
}
