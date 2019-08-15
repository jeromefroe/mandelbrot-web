#![feature(async_await)]

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
        let mut futures = Vec::with_capacity(shards.len());

        for shard in shards.into_iter() {
            futures.push(async move {
                let millis = rand::thread_rng().gen_range(0, 100);
                let mut interval = Interval::new_interval(Duration::from_millis(millis));
                interval.next().await;

                cpu_intesive_work(shard);
            })
        }

        join_all(futures).await;

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
