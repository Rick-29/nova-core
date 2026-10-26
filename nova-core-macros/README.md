# nova-core-macros

Procedural macros for [`nova-ta-core`](https://crates.io/crates/nova-ta-core).

**You normally don't depend on this crate directly.** `nova-ta-core` re-exports
`math_support!` and `math_impl!`, and the generated code refers to
`::nova_ta_core::Num`, so `nova-ta-core` must be a dependency of the crate that
uses them.

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

`math_support!(name, mapper, Type)` defines the local macro `name!`;
`math_impl!(mapper, Type, expr)` is the underlying one-shot form.

## License

MIT
