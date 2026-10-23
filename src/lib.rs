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

mod num;
mod window;

pub use num::Num;
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
/// use nova_core::{Indicator, Window};
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
/// use nova_core::Ohlc;
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
