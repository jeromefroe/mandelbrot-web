# mandelbrot-web

A WIP example server for producing images of Mandelbrot sets using Rust's new [async/await syntax].
The server is a derivation of the [echo server example in the Romio crate].

To run the server:

```bash
# We need to use nightly since async/await is currently only available on nightly.
rustup override set nightly

cargo run
```

We can then interact with the server from another terminal window:

```bash
nc localhost 7878
```

[async/await syntax]: https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html
[echo server example in the romio crate]: https://github.com/withoutboats/romio/blob/master/examples/echo.rs
