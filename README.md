# mandelbrot-web

An example server for producing images of Mandelbrot sets using Rust's new [async/await syntax].
The server is a straightforward combination of the [echo server example in the tokio crate] and
the [Mandelbrot set plotter] from the _Programming Rust_ book.

## Usage

Start the server:

```bash
cargo run
```

We can then interact with the server from another terminal window:

```bash
nc localhost 8080 > mandelbrot.png
1000x750 -1.20,0.35 -1,0.20
```

[async/await syntax]: https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html
[echo server example in the tokio crate]: https://tokio.rs/blog/2019-08-alphas/
[mandelbrot set plotter]: https://github.com/ProgrammingRust/mandelbrot
