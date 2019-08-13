#![feature(async_await)]

use std::io;

use futures::executor::{self, ThreadPool};
use futures::io::AsyncReadExt;
use futures::io::AsyncWriteExt;
use futures::task::SpawnExt;
use futures::StreamExt;

use romio::{TcpListener, TcpStream};

fn main() -> io::Result<()> {
    executor::block_on(async {
        let mut threadpool = ThreadPool::new()?;

        let mut listener = TcpListener::bind(&"127.0.0.1:7878".parse().unwrap())?;
        let mut incoming = listener.incoming();

        println!("Listening on 127.0.0.1:7878");

        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            let addr = stream.peer_addr()?;

            threadpool
                .spawn(async move {
                    println!("Accepting stream from: {}", addr);

                    echo_on(stream).await.unwrap();

                    println!("Closing stream from: {}", addr);
                })
                .unwrap();
        }

        Ok(())
    })
}

async fn echo_on(stream: TcpStream) -> io::Result<()> {
    let (mut reader, mut writer) = stream.split();

    let mut buf = [0u8; 1024];
    reader.read(&mut buf).await?;

    // TODO: Parse input for parameters of Mandelbrot set image we are generating.

    // TODO: What I want to emulate in this request handler is a CPU intensive task. For example,
    // suppose our server is used to compute graphs of Mandelbrot sets (this example was used in
    // the O'Reilly book Programming Rust). Our handler function would then accept different
    // parameters (for example, the dimensions of the graph we want to generate) and return a
    // corresponding image of the Mandelbrot set. What's interesting about this problem is that
    // since computing the Mandelbrot set can be a computationally expensive task we want to
    // divide the image into subregions and calculate those subregions in parallel. That way, if
    // our server is lightly loaded (more CPUs than requests), we can ensure that we compute the
    // set as quickly as we can. In other words, we want our request handler to run on a thread
    // pool, but we also want it to send the CPU intensive work that it needs to perform to a
    // thread pool as well. The calculation of each subregion that we have divided the image of
    // the Mandelbrot set into should be turned into a future that is run on the threadpool. Our
    // request handler is responsible for the coordination of the work: it divides the images
    // into subregions, creates a future for the calculation of each subregion, and is then
    // responsible for joining all those futures together (for example, with futures::join!) and
    // waiting for that joined future to complete which indicates that each subregion has been
    // calculated and so the image is done. The critical point though is that since we are calling
    // await in the request on the joined future we aren't blocking any thread from waiting while
    // we wait for the work that has been sent out to the threadpool to complete.
    let mut num = 0;
    cpu_intesive_work(&mut num).await;

    println!("num is {}", num);

    // TODO: Currently we are just writing "Hello World!", once we finish the handler we will
    // be sending the pixels of the Mandelbrot set image.
    writer
        .write_all(&[72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 33])
        .await?;

    Ok(())
}

async fn cpu_intesive_work(num: &mut i32) {
    for i in 1..1000 {
        *num += i
    }
}
