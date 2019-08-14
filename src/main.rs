#![feature(async_await)]

use std::marker::PhantomData;
use std::pin::Pin;

use futures::future::join_all;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080".parse()?;
    let mut listener = TcpListener::bind(&addr)?;

    loop {
        let (mut socket, _) = listener.accept().await?;
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

        //
        //
        //

        let mut nums: [i32; 5] = [1, 2, 3, 4, 5];
        let iter = nums.iter_mut();

        // This code will be handled by Scope.
        scope(|s| {
            for num in iter {
                s.spawn(async {
                    cpu_intesive_work(num);
                });
            }
        });

        //
        //
        //

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
    futures: Vec<Pin<Box<dyn Future<Output = ()> + 'static>>>,
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
        F: Future<Output = ()> + 'env,
        'env: 'scope,
    {
        let func: Pin<Box<dyn Future<Output = ()> + 'env>> = Box::pin(f);
        let func: Pin<Box<dyn Future<Output = ()> + 'static>> =
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

fn scope<'env, F, R>(f: F) -> R
where
    F: FnOnce(&mut Scope<'env>) -> R,
{
    let mut scope = Scope::new();
    let result = f(&mut scope);
    scope.join_all();
    result
}
