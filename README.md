
<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/nova-dark-text.jpg">
    <img alt="Nova: stateful indicators for real-time data" src="assets/nova-light-text.jpg" width="360">
  </picture>
</p>

[![Discord](https://img.shields.io/badge/Discord-Join%20the%20community-5865F2?logo=discord&logoColor=white)](https://discord.gg/QwKmaaeW56)
# nova-core


Core traits and state primitives for streaming technical-analysis indicators in Rust.

`nova-core` is the foundation of the Nova indicator ecosystem. It defines the shared abstractions that indicator implementations are written against, and contains no concrete indicators itself.

| Item | Purpose |
| --- | --- |
| `Indicator` | Stateful, incremental computation: `update`, `ready`, `reset`. |
| `Num` | Numeric abstraction with fallible arithmetic; implemented for `f64`. |
| `Ohlc<N>` | A candle: open, high, low, close, volume and timestamp. |
| `Window<T>` | Fixed-capacity rolling buffer that returns the evicted sample on push. |

## Example

A rolling sum implemented once against `Indicator`:

```rust
use core::convert::Infallible;
use nova_core::{Indicator, Window};

struct Sum {
    window: Window<f64>,
    total: f64,
}

impl Indicator for Sum {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn update(&mut self, input: f64) -> Result<f64, Infallible> {
        self.total += input;
        if let Some(old) = self.window.push(input) {
            self.total -= old;
        }
        Ok(self.total)
    }

    fn ready(&self) -> bool {
        self.window.is_full()
    }

    fn reset(&mut self) {
        self.window.clear();
        self.total = 0.0;
    }
}

let mut sum = Sum { window: Window::new(3), total: 0.0 };
for price in [1.0, 2.0, 3.0, 4.0] {
    let value = sum.update(price).unwrap();
    println!("{value} (ready: {})", sum.ready());
}
```

## Incremental processing and warm-up

An indicator consumes one sample per `update` call and keeps its own state, so it fits candle-by-candle execution on live data.

- `update` always returns a value. Until `ready()` is `true` the value is a partial result whose seeding is defined by the implementation.
- `ready()` reports that warm-up is complete. The caller decides whether to skip or use partial outputs.
- If `update` returns an error, the state is unspecified; call `reset()` before reuse.
- `reset()` restores the just-constructed state and restarts warm-up.

## Installation

`nova-core` is not yet published on crates.io. Until it is, depend on it by path:

```toml
[dependencies]
nova-core = { path = "../nova-core" }
```

## Multi-language goal and Chipa-Flow

Nova aims to implement each indicator once and expose consistent behavior across Rust, Python, TypeScript and possibly other languages, potentially through bindings such as UniFFI. No bindings are provided by this crate.

Nova is being built for [Chipa](https://chipaeditor.com), in particular the new Chipa-Flow language, where indicators must work with real-time market data and candle-by-candle execution. The integration is in development; the version of Chipa-Flow currently described on the public website does not use Nova but the beta version available in [Chipa Studio](https://studio.chipaeditor.com).

### Using Nova?

Nova is free to use in personal, commercial, and closed-source projects. If you use it in something you're building, we'd love to hear about it. Share your project name or a link if you're comfortable doing so — it helps us understand how Nova is used in the real world and where we can improve.

<<<<<<< HEAD
No fees, no obligations, and no need to share proprietary details.

## Help and community

Questions or feedback? Join the [Nova Discord](https://discord.gg/QwKmaaeW56).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
=======
No fees, no obligations, and no need to share proprietary details.
>>>>>>> e40a7003eb1e2bec44a7ad25c3b1423ffdf79eac
