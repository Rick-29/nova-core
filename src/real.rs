use crate::Num;

/// A [`Num`] that converts to and from `f64` and supports the exponential
/// and the natural logarithm.
///
/// Implementors provide only [`from_f64`](Real::from_f64),
/// [`to_f64`](Real::to_f64), [`exp`](Real::exp) and [`ln`](Real::ln). Every
/// other method has a default built from them (plus the [`Num`] operations)
/// and can be overridden when a backend has a more accurate or faster
/// version, as `f64` does.
///
/// Like [`Num`], the operations are fallible, and implementations must be
/// deterministic.
///
/// # The `eml` operator
///
/// [`eml`](Real::eml) is `exp(x) - ln(y)`. Together with the constant 1 it
/// is enough to build the elementary functions, and it is provided here as a
/// default so backends and indicators can use it directly. It is **not** a
/// required method: defaulting `exp` and `ln` to `eml` while `eml`
/// defaults to them would be circular. The other defaults go through
/// `exp` and `ln` rather than long chains of `eml`, which keeps the
/// error bounded.
///
/// # Example
///
/// ```
/// use nova_ta_core::Real;
///
/// // Geometric mean of two positive numbers, generic over the backend,
/// // using a floating-point literal as a constant.
/// fn geo_mean<R: Real>(a: R, b: R) -> Result<R, R::Error> {
///     let half = R::from_f64(0.5)?;
///     a.powf(half)?.mul(b.powf(half)?)
/// }
///
/// let g = geo_mean(4.0_f64, 9.0).unwrap();
/// assert!((g - 6.0).abs() < 1e-12);
/// ```
pub trait Real: Num {
    /// Converts an `f64` into `Self`, e.g. for constants such as `0.5`.
    ///
    /// The conversion may round to the nearest representable value.
    ///
    /// # Errors
    /// Backend-defined. Checked backends should reject NaN, infinities and
    /// values outside their range.
    fn from_f64(x: f64) -> Result<Self, Self::Error>;

    /// Returns `e` raised to the power `self`.
    ///
    /// # Errors
    /// Backend-defined, e.g. overflow.
    fn exp(self) -> Result<Self, Self::Error>;

    /// Returns the natural logarithm of `self`.
    ///
    /// # Errors
    /// Backend-defined. Checked backends should return an error for
    /// `self <= 0`; `f64` follows IEEE (`-inf` / NaN).
    fn ln(self) -> Result<Self, Self::Error>;

    /// Returns `exp(self) - ln(y)`.
    ///
    /// # Errors
    /// Any error from [`exp`](Real::exp), [`ln`](Real::ln) or
    /// [`sub`](Num::sub).
    fn eml(self, y: Self) -> Result<Self, Self::Error> {
        self.exp()?.sub(y.ln()?)
    }

    /// Returns `self` raised to the power `exp`, as `exp(exp * ln(self))`.
    ///
    /// The default is only defined for `self > 0`; it fails or returns NaN
    /// wherever [`ln`](Real::ln) does. Backends with a native `pow` should
    /// override it.
    fn powf(self, exp: Self) -> Result<Self, Self::Error> {
        self.ln()?.mul(exp)?.exp()
    }

    /// Returns the logarithm of `self` in the given `base`, as
    /// `ln(self) / ln(base)`.
    ///
    /// # Errors
    /// Fails where either [`ln`](Real::ln) does, and when `base` is 1
    /// (division by zero on checked backends).
    fn log(self, base: Self) -> Result<Self, Self::Error> {
        self.ln()?.div(base.ln()?)
    }
}

/// Delegates to the standard library; never returns `Err`.
impl Real for f64 {
    fn from_f64(x: f64) -> Result<Self, Self::Error> {
        Ok(x)
    }
    fn exp(self) -> Result<Self, Self::Error> {
        Ok(f64::exp(self))
    }
    fn ln(self) -> Result<Self, Self::Error> {
        Ok(f64::ln(self))
    }
    fn powf(self, exp: Self) -> Result<Self, Self::Error> {
        Ok(f64::powf(self, exp))
    }
    fn log(self, base: Self) -> Result<Self, Self::Error> {
        Ok(f64::log(self, base))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Backend that only supplies `exp` and `ln`, so the defaults run.
    #[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
    struct Wrap(f64);

    impl Num for Wrap {
        type Error = core::convert::Infallible;
        fn zero() -> Self { Wrap(0.0) }
        fn one() -> Self { Wrap(1.0) }
        fn from_usize(n: usize) -> Self { Wrap(n as f64) }
        fn add(self, r: Self) -> Result<Self, Self::Error> { Ok(Wrap(self.0 + r.0)) }
        fn sub(self, r: Self) -> Result<Self, Self::Error> { Ok(Wrap(self.0 - r.0)) }
        fn mul(self, r: Self) -> Result<Self, Self::Error> { Ok(Wrap(self.0 * r.0)) }
        fn div(self, r: Self) -> Result<Self, Self::Error> { Ok(Wrap(self.0 / r.0)) }
        fn sqrt(self) -> Result<Self, Self::Error> { Ok(Wrap(self.0.sqrt())) }
        fn neg(self) -> Self { Wrap(-self.0) }
    }
    impl Real for Wrap {
        fn from_f64(x: f64) -> Result<Self, Self::Error> { Ok(Wrap(x)) }
        fn exp(self) -> Result<Self, Self::Error> { Ok(Wrap(self.0.exp())) }
        fn ln(self) -> Result<Self, Self::Error> { Ok(Wrap(self.0.ln())) }
    }

    #[test]
    fn eml_identities() {
        let one = Wrap(1.0);
        // exp(x) = eml(x, 1)
        assert_eq!(Wrap(2.0).eml(one).unwrap(), Wrap(2.0).exp().unwrap());
        // ln(x) = eml(1, eml(eml(1, x), 1))
        let x = Wrap(5.0);
        let ln = one.eml(one.eml(x).unwrap().eml(one).unwrap()).unwrap();
        assert!((ln.0 - 5.0_f64.ln()).abs() < 1e-12);
    }

    #[test]
    fn defaults_match_std() {
        assert!((Wrap(2.0).powf(Wrap(10.0)).unwrap().0 - 1024.0).abs() < 1e-9);
        assert!((Wrap(8.0).log(Wrap(2.0)).unwrap().0 - 3.0).abs() < 1e-12);
    }
}
