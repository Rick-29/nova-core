//! Procedural macros behind `nova-ta-core`'s fallible arithmetic helpers.
//!
//! Depend on `nova-ta-core` and use its re-exports instead of this crate
//! directly: the generated code refers to `::nova_ta_core::Num` and
//! `::nova_ta_core::Real`.
//!
//! ```ignore
//! nova_ta_core::math_support!(math, NovaError::number, T);
//! nova_ta_core::real_support!(real, NovaError::number, T);
//!
//! let m = math!((n_t * xy_sum - x_sum * y_sum) / (n_t * x_sq2 - x_sum2));
//! let r = real!(exp(-0.5 * x ** 2) / sqrt(2 * PI));
//! ```
//!
//! Both expand to a plain `T`, with every fallible call mapped through the
//! error mapper and propagated with `?`. `math!` accepts the [`Num`] subset:
//! `+ - * /`, unary `-`, integer literals and `expr as usize`
//! (`Num::from_usize`), and the functions `sqrt`, `abs`, `max`, `min`.
//! `real!` adds the [`Real`] vocabulary: float literals and `expr as f64`
//! (`Real::from_f64`), `**` for `powf`, the functions `exp`, `ln`, `sin`,
//! `cos`, `pow`, `log`, `eml` (callable as `f(x, ..)` or `x.f(..)`) and the
//! constants `PI`, `E`, `TAU`, `LN_2`.
//!
//! [`Num`]: https://docs.rs/nova-ta-core/latest/nova_ta_core/trait.Num.html
//! [`Real`]: https://docs.rs/nova-ta-core/latest/nova_ta_core/trait.Real.html

mod expand;
mod func;
mod node;

use node::Level;

/// Expands a `Num` expression into fallible trait calls.
///
/// `math_impl!(NovaError::number, T, a + b * 2)`
#[proc_macro]
pub fn math_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand::expand(input, Level::Num)
}

/// Defines a local macro `name!(expr)` bound to a number type and an error
/// mapper: `math_support!(math, NovaError::number, T);` then `math!(a + b * 2)`
/// is `math_impl!(NovaError::number, T, a + b * 2)`.
///
/// `Type` is resolved where `name!` is used, so it can be a generic parameter.
/// The calling crate needs `nova-ta-core` as a dependency, which re-exports
/// both macros.
#[proc_macro]
pub fn math_support(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand::support(input, "math_impl")
}

/// Expands a `Real` expression into fallible trait calls.
///
/// `real_impl!(NovaError::number, T, exp(-0.5 * x ** 2) / sqrt(2 * PI))`
#[proc_macro]
pub fn real_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand::expand(input, Level::Real)
}

/// Defines a local macro `name!(expr)` bound to a `Real` type and an error
/// mapper, like [`math_support!`] but forwarding to [`real_impl!`].
#[proc_macro]
pub fn real_support(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand::support(input, "real_impl")
}
