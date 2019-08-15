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

## TODO

I've noticed that Tokio doesn't do a great job of utilizing all CPU cores when executing the
futures to render different bands in the Mandelbrot set being generated. A comment on the
threadpool implementation, in fact, states:

> The Tokio thread pool supports scheduling futures and processing them on multiple CPU cores. It
> is optimized for the primary Tokio use case of many independent tasks with limited computation
> and with most tasks waiting on I/O.

Consequently, I want to explore using a dedicated CPU pool for rendering the Mandelbrot set so
we can allow the Tokio threadpool to focus on performing short IO tasks.
