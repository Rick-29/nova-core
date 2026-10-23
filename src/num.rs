/// A number type that indicators can compute with.
///
/// Arithmetic is fallible so that checked backends can report overflow or
/// division by zero as errors. Backends that cannot fail set
/// [`Error`](Num::Error) to [`core::convert::Infallible`].
///
/// Implementors must uphold two requirements:
///
/// - [`from_usize`](Num::from_usize) is exact for the small integers
///   indicators use (periods, weights, constants such as 2, 50 and 100),
///   realistically anything up to 10⁶.
/// - Operations are deterministic: the same operands give the same result
///   on any machine.
///
/// `Copy` is required so indicator code does not need to clone values.
///
/// [`abs`](Num::abs), [`max_of`](Num::max_of) and [`min_of`](Num::min_of)
/// have default implementations built on [`PartialOrd`] and
/// [`neg`](Num::neg).
///
/// # Example
///
/// ```
/// use nova_ta_core::Num;
///
/// fn mean<N: Num>(a: N, b: N) -> Result<N, N::Error> {
///     a.add(b)?.div(N::from_usize(2))
/// }
///
/// assert_eq!(mean(1.0_f64, 4.0), Ok(2.5));
/// ```
pub trait Num: Copy + PartialOrd {
    /// Error returned by failing operations.
    type Error: core::fmt::Debug;

    /// The additive identity.
    fn zero() -> Self;
    /// The multiplicative identity.
    fn one() -> Self;
    /// Converts a small non-negative integer exactly. See the trait-level
    /// requirements.
    fn from_usize(n: usize) -> Self;

    /// Returns `self + rhs`.
    ///
    /// # Errors
    /// Backend-defined, e.g. overflow.
    fn add(self, rhs: Self) -> Result<Self, Self::Error>;
    /// Returns `self - rhs`.
    ///
    /// # Errors
    /// Backend-defined, e.g. overflow.
    fn sub(self, rhs: Self) -> Result<Self, Self::Error>;
    /// Returns `self * rhs`.
    ///
    /// # Errors
    /// Backend-defined, e.g. overflow.
    fn mul(self, rhs: Self) -> Result<Self, Self::Error>;
    /// Returns `self / rhs`.
    ///
    /// # Errors
    /// Backend-defined, e.g. division by zero or overflow.
    fn div(self, rhs: Self) -> Result<Self, Self::Error>;
    /// Returns the square root of `self`.
    ///
    /// # Errors
    /// Backend-defined, e.g. a negative operand.
    fn sqrt(self) -> Result<Self, Self::Error>;
    /// Returns `-self`. Infallible by contract.
    fn neg(self) -> Self;

    /// Returns the absolute value.
    fn abs(self) -> Self {
        if self < Self::zero() { self.neg() } else { self }
    }
    /// Returns the larger of `self` and `other`; `self` if they compare
    /// equal or are unordered.
    fn max_of(self, other: Self) -> Self {
        if other > self { other } else { self }
    }
    /// Returns the smaller of `self` and `other`; `self` if they compare
    /// equal or are unordered.
    fn min_of(self, other: Self) -> Self {
        if other < self { other } else { self }
    }
}

/// `f64` backend with plain IEEE 754 semantics. Operations never return
/// `Err`: `x / 0.0` yields infinity or NaN, and `sqrt` of a negative number
/// yields NaN. Callers are responsible for not feeding non-finite inputs.
impl Num for f64 {
    type Error = core::convert::Infallible;

    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
    fn from_usize(n: usize) -> Self {
        n as f64
    }
    fn add(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self + rhs)
    }
    fn sub(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self - rhs)
    }
    fn mul(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self * rhs)
    }
    fn div(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self / rhs)
    }
    fn sqrt(self) -> Result<Self, Self::Error> {
        Ok(f64::sqrt(self))
    }
    fn neg(self) -> Self {
        -self
    }
}
