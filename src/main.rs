#![feature(async_await)]

use std::marker::PhantomData;
use std::pin::Pin;
use std::time::Duration;

use futures::future::join_all;
use rand::Rng;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::timer::Interval;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080".parse()?;
    let mut listener = TcpListener::bind(&addr)?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(handler(socket));
    }
}

async fn handler(mut socket: TcpStream) {
    let mut buf = [0; 1024];

    // In a loop, read data from the socket and write the data back.
    loop {
        let n = match socket.read(&mut buf).await {
            // socket closed
            Ok(n) if n == 0 => return,
            Ok(n) => n,
            Err(e) => {
                println!("failed to read from socket; err = {:?}", e);
                return;
            }
        };

        // Simulate performing a computationally intensive task.
        let mut pixels: [i32; 5] = [1, 2, 3, 4, 5];

        let shards: Vec<&mut i32> = pixels.iter_mut().collect();

        scope(|s| {
            for shard in shards.into_iter() {
                s.spawn(async move {
                    let millis = rand::thread_rng().gen_range(0, 100);
                    let mut interval = Interval::new_interval(Duration::from_millis(millis));
                    interval.next().await;

                    cpu_intesive_work(shard);
                });
            }
        })
        .await;

        // Write the data back
        if let Err(e) = socket.write_all(&buf[0..n]).await {
            println!("failed to write to socket; err = {:?}", e);
            return;
        }
    }
}

fn cpu_intesive_work(num: &mut i32) {
    *num *= 2;
    println!("num: {}", *num);
}

struct Scope<'env> {
    futures: Vec<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>,
    _marker: PhantomData<&'env ()>,
}

impl<'env> Scope<'env> {
    fn new() -> Scope<'env> {
        Scope {
            futures: Vec::new(),
            _marker: PhantomData,
        }
    }

    fn spawn<'scope, F>(&'scope mut self, f: F)
    where
        F: Future<Output = ()> + Send + 'env,
        'env: 'scope,
    {
        // Use transmute to change the lifetime bound of the Future to 'static. The lifetime
        // bound 'env: 'scope ensures that the lifetime 'env is longer than the lifetime 'scope.
        // And our call to `join_all` below ensures that no future has a lifetime longer 'scope.
        // Thus, even though we erase the 'env lifetime of the future we manually ensure that
        // it's lifetime is less than 'env.
        let func: Pin<Box<dyn Future<Output = ()> + Send + 'env>> = Box::pin(f);
        let func: Pin<Box<dyn Future<Output = ()> + Send + 'static>> =
            unsafe { std::mem::transmute(func) };
        self.futures.push(func);
    }

    async fn join_all(self) {
        // Ideally we would implement Drop for Scope and call join_all there so we can guarantee
        // the function is called when Scope goes out of scope. As best as I can tell though that
        // is not possible at the moment since we need to call await.
        join_all(self.futures).await;
    }
}

async fn scope<'env, F, R>(f: F) -> R
where
    F: FnOnce(&mut Scope<'env>) -> R,
{
    let mut scope = Scope::new();
    let result = f(&mut scope);
    scope.join_all().await;
    result
}
