# nova-core-macros

Procedural macros for [`nova-ta-core`](https://crates.io/crates/nova-ta-core).

**You normally don't depend on this crate directly.** `nova-ta-core` re-exports
`math_support!` / `math_impl!` and `real_support!` / `real_impl!`, and the
generated code refers to `::nova_ta_core::Num` and `::nova_ta_core::Real`, so
`nova-ta-core` must be a dependency of the crate that uses them.

## Why

Every `Num` operation returns a `Result`, so a formula turns into nested calls:

```rust
add(&mul(&m, &n_t)?, &b)
```

The macros let you write the formula as ordinary infix code and propagate each
error with `?`:

```rust
use nova_ta_core::{Num, math_support};

math_support!(math, MyError::number, T);

fn line<T: Num>(m: T, n: usize, b: T) -> Result<T, MyError> {
    let n = T::from_usize(n);
    Ok(math!(m * n + b))
}
```

## Rules

- `+`, `-`, `*`, `/`, unary `-` and parentheses, with normal precedence. Other
  operators are a compile error.
- Operands are any expression of type `T` (variables, fields, indexing, calls).
- Integer literals and `expr as usize` become `Num::from_usize`; the inner
  `usize` expression stays plain Rust. Float literals are rejected.
- Binary operations map their error with the given mapper, then `?`. The macro
  evaluates to a plain `T`, so wrap it as `Ok(math!(..))`.

- The `Num` functions `sqrt`, `abs`, `max`, `min` work as `sqrt(x)` or
  `x.sqrt()`. Unknown function names are left alone and used as operands.

`math_support!(name, mapper, Type)` defines the local macro `name!`;
`math_impl!(mapper, Type, expr)` is the underlying one-shot form.

## `real_support!`

Same shape, for types implementing `Real`. Everything above plus:

```rust
real_support!(real, MyError::number, T);

fn pdf<T: Real>(x: T) -> Result<T, MyError> {
    Ok(real!(exp(-0.5 * x ** 2) / sqrt(2 * PI)))
}
```

- Float literals and `expr as f64` become `Real::from_f64`.
- `**` is `Real::powf`, binding tighter than `*`, `/` and unary `-` and
  right-associative (Python rules): `2 * x ** 2` is `2 * (x ** 2)`,
  `-x ** 2` is `-(x ** 2)`. `pow(x, p)` / `x.pow(p)` are the call forms.
  Note that `rustfmt` prints `x ** 2` as `x * *2`; both parse the same.
- Functions `exp`, `ln`, `sin`, `cos`, `pow`, `log(x, base)`, `eml(x, y)`,
  as calls or methods.
- Constants `PI`, `E`, `TAU`, `LN_2`.
- Using any of these inside `math!` is a compile error.

## License

MIT
