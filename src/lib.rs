//! Core abstractions for streaming technical-analysis indicators.
//!
//! `nova-core` defines the shared building blocks that Nova indicator
//! implementations are written against:
//!
//! - [`Indicator`]: a stateful, incremental computation driven by `update`,
//!   with `ready` for warm-up and `reset` to start over.
//! - [`Num`]: a numeric abstraction with fallible arithmetic, implemented
//!   for `f64`.
//! - [`Ohlc`]: a candle with prices, volume and timestamp.
//! - [`Window`]: a fixed-capacity rolling buffer.
//! - [`math_support!`] / [`math_impl!`]: macros that write fallible [`Num`]
//!   arithmetic as ordinary infix expressions.
//! - [`real_support!`] / [`real_impl!`]: the same for [`Real`], adding float
//!   literals, `**`, `exp`, `ln`, `sin`, `cos`, `log` and the constants.
//!
//! The crate contains no concrete indicators.
//!
//! # Design
//!
//! - **Fallible arithmetic.** [`Num`] operations return `Result`, so
//!   checked backends (for example fixed-point decimals) can report overflow
//!   or division by zero as errors instead of panicking. Infallible types
//!   such as `f64` use [`Infallible`](core::convert::Infallible).
//! - **Incremental updates.** An indicator consumes one sample per call and
//!   keeps its own state; no history is passed in.
//! - **Warm-up is a query.** `update` always returns a value and
//!   [`Indicator::ready`] says when it is meaningful, so there is no
//!   `Option` on the hot path.
//!
//! # Writing arithmetic with `math_support!`
//!
//! Every [`Num`] operation returns a `Result`, so a formula such as
//! `(n * xy - x * y) / (n * x2 - x * x)` turns into a pile of nested
//! `add(&mul(..)?, ..)?` calls. [`math_support!`] defines a local macro that
//! lets you write the formula directly:
//!
//! ```
//! use nova_ta_core::{Num, math_support};
//!
//! #[derive(Debug, PartialEq)]
//! enum MyError {
//!     Number(String),
//! }
//!
//! impl MyError {
//!     fn number(e: impl core::fmt::Debug) -> Self {
//!         MyError::Number(format!("{e:?}"))
//!     }
//! }
//!
//! // Defines `math!`, bound to the number type `T` and the error mapper.
//! math_support!(math, MyError::number, T);
//!
//! fn slope<T: Num>(n: usize, xy: T, x: T, y: T, x2: T) -> Result<T, MyError> {
//!     let n = T::from_usize(n);
//!     Ok(math!((n * xy - x * y) / (n * x2 - x * x)))
//! }
//!
//! assert_eq!(slope(2, 5.0, 3.0, 4.0, 5.0), Ok(-2.0));
//! ```
//!
//! The generated macro expands to a plain `T`, with a `?` after every
//! operation. Use it inside a function that returns `Result`, usually wrapped
//! as `Ok(math!(..))`. Rules:
//!
//! - **Operators.** `+`, `-`, `*`, `/`, unary `-` and parentheses, with the
//!   usual precedence. Binary operators call [`Num::add`], [`Num::sub`],
//!   [`Num::mul`] and [`Num::div`], and each error goes through the mapper
//!   before `?`. Unary `-` calls the infallible [`Num::neg`]. Anything else
//!   is a compile error.
//! - **Operands.** Any expression that evaluates to `T`: variables, fields,
//!   indexing, function and method calls. [`Num`] is `Copy`, so no `&` is
//!   needed.
//! - **`usize` values.** A macro cannot see types, so mark them. Integer
//!   literals and `expr as usize` become [`Num::from_usize`], and the inner
//!   `usize` expression stays ordinary Rust: `math!(a * 2 + 1)` and
//!   `math!((n * (n + 1) / 2) as usize)`. Float literals are rejected.
//! - **Error mapper.** `math_support!(name, mapper, Type)` takes any
//!   expression usable as `map_err(mapper)`, for example `MyError::number`
//!   or [`core::convert::identity`] when the caller already uses
//!   `T::Error`.
//! - **Type.** Resolved where `name!` is used, so it can be a generic
//!   parameter such as `T`, or a concrete type such as `f64`.
//!
//! [`math_impl!`] is the underlying macro,
//! `math_impl!(mapper, Type, expr)`. Prefer [`math_support!`] unless you need
//! a different mapper or type for a single expression.
//!
//! `math!` also accepts the [`Num`] functions `sqrt(x)`, `abs(x)`,
//! `max(a, b)` and `min(a, b)`, written either as calls or as methods
//! (`x.sqrt()`, `a.max(b)`).
//!
//! # Writing `Real` formulas with `real_support!`
//!
//! [`real_support!`] has the same shape and accepts everything `math!` does,
//! plus the [`Real`] vocabulary:
//!
//! ```
//! use nova_ta_core::{Real, real_support};
//!
//! real_support!(real, core::convert::identity, T);
//!
//! /// Standard normal density.
//! fn pdf<T: Real>(x: T) -> Result<T, T::Error> {
//!     Ok(real!(exp(-0.5 * x ** 2) / sqrt(2 * PI)))
//! }
//!
//! assert!((pdf(0.0_f64).unwrap() - 0.3989422804014327).abs() < 1e-15);
//! ```
//!
//! - **Float literals and `expr as f64`** become [`Real::from_f64`]; integer
//!   literals and `expr as usize` are still [`Num::from_usize`].
//! - **`**`** is [`Real::powf`]. It binds tighter than `*`, `/` and unary
//!   `-` and is right-associative, as in Python: `2 * x ** 2` is
//!   `2 * (x ** 2)` and `-x ** 2` is `-(x ** 2)`. `pow(x, p)` and
//!   `x.pow(p)` are the same thing spelled as a call.
//! - **Functions**: `exp`, `ln`, `sin`, `cos`, `pow`/`powf`, `log(x, base)`,
//!   `eml(x, y)`, as calls or methods (`x.ln()`, `x.log(b)`). Arguments are
//!   themselves `real!` expressions.
//! - **Constants**: `PI`, `E`, `TAU`, `LN_2` call [`Real::pi`] and friends,
//!   so a variable with one of those names is shadowed inside `real!`.
//! - Unknown function names are left untouched and treated as operands, so
//!   your own `fn half(x: T) -> T` still works inside the expression.
//!
//! [`real_impl!`] is the one-shot form, `real_impl!(mapper, Type, expr)`.
//! Using a `Real`-only feature inside `math!` is a compile error pointing at
//! the offending token.

pub use nova_core_macros::{math_impl, math_support, real_impl, real_support};

mod num;
mod real;
mod window;

pub use num::Num;
pub use real::Real;
pub use window::Window;

/// A streaming indicator: it consumes one sample at a time and owns its state.
///
/// # Contract
///
/// - [`update`](Indicator::update) always returns a value, even during
///   warm-up. Until [`ready`](Indicator::ready) is `true` that value is a
///   *partial* result whose seeding is defined by the implementation.
///   Callers decide whether to skip or consume partial outputs.
/// - [`ready`](Indicator::ready) reports whether warm-up is complete, i.e.
///   whether outputs carry the meaning the indicator advertises.
/// - [`reset`](Indicator::reset) returns the indicator to its
///   just-constructed state, including warm-up.
///
/// # Composition
///
/// An indicator built from other indicators holds them as ordinary fields
/// and calls their `update` itself. The caller only drives the outermost
/// indicator, and the owner decides when each inner one advances.
///
/// # Example
///
/// A running sum over the last `n` samples, built on [`Window`]:
///
/// ```
/// use nova_ta_core::{Indicator, Window};
/// use core::convert::Infallible;
///
/// struct Sum {
///     window: Window<f64>,
///     total: f64,
/// }
///
/// impl Indicator for Sum {
///     type Input = f64;
///     type Output = f64;
///     type Error = Infallible;
///
///     fn update(&mut self, input: f64) -> Result<f64, Infallible> {
///         self.total += input;
///         if let Some(old) = self.window.push(input) {
///             self.total -= old;
///         }
///         Ok(self.total)
///     }
///
///     fn ready(&self) -> bool {
///         self.window.is_full()
///     }
///
///     fn reset(&mut self) {
///         self.window.clear();
///         self.total = 0.0;
///     }
/// }
///
/// let mut sum = Sum { window: Window::new(3), total: 0.0 };
/// assert_eq!(sum.update(1.0), Ok(1.0));
/// assert!(!sum.ready());
/// sum.update(2.0).unwrap();
/// assert_eq!(sum.update(3.0), Ok(6.0));
/// assert!(sum.ready());
/// assert_eq!(sum.update(4.0), Ok(9.0)); // 2 + 3 + 4
/// ```
pub trait Indicator {
    /// The sample consumed by each [`update`](Indicator::update), for
    /// example a price or an [`Ohlc`] candle.
    type Input;
    /// The value produced by each [`update`](Indicator::update).
    type Output;
    /// The error returned when an update fails. Infallible indicators
    /// use [`core::convert::Infallible`].
    type Error;

    /// Consumes one sample and returns the current output.
    ///
    /// Before [`ready`](Indicator::ready) is `true` the output is a partial
    /// result; each implementation documents how it is seeded.
    ///
    /// # Errors
    ///
    /// Returns an error when the computation fails, typically because a
    /// checked [`Num`] reported overflow or division by zero. After an
    /// error the indicator's state is unspecified: call
    /// [`reset`](Indicator::reset) before using it again.
    fn update(&mut self, input: Self::Input) -> Result<Self::Output, Self::Error>;

    /// Returns `true` once warm-up is complete and outputs carry their
    /// advertised meaning.
    fn ready(&self) -> bool;

    /// Restores the just-constructed state, discarding all history and
    /// restarting warm-up.
    fn reset(&mut self);
}

/// One market candle: open, high, low, close, volume and timestamp.
///
/// Price fields use the numeric type `N` (see [`Num`]). Indicators that only
/// need prices ignore `volume` and `time`.
///
/// # Example
///
/// ```
/// use nova_ta_core::Ohlc;
///
/// let bar = Ohlc::prices(10.0, 12.0, 9.5, 11.0);
/// assert_eq!(bar.close, 11.0);
/// assert_eq!(bar.volume, 0.0);
/// assert_eq!(bar.time, 0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ohlc<N> {
    /// Opening price.
    pub open: N,
    /// Highest price of the bar.
    pub high: N,
    /// Lowest price of the bar.
    pub low: N,
    /// Closing price.
    pub close: N,
    /// Traded volume over the bar. Zero when the feed has none.
    pub volume: N,
    /// Bar timestamp as a Unix epoch value. The resolution (seconds,
    /// milliseconds, ...) is chosen by the host and must be used
    /// consistently. It is an `i64` regardless of `N`.
    pub time: i64,
}

impl<N: Num> Ohlc<N> {
    /// Creates a price-only candle with `volume` set to zero and `time` set
    /// to `0`.
    pub fn prices(open: N, high: N, low: N, close: N) -> Self {
        Self { open, high, low, close, volume: N::zero(), time: 0 }
    }
}
