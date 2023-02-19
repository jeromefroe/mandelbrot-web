mod mandelbrot;

use mandelbrot::{parse_pair, pixel_to_point, render};

use futures::future::join_all;
use image::{GrayImage, ImageEncoder};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(handler(socket));
    }
}

async fn handler(mut socket: TcpStream) {
    let mut buf = [0; 1024];

    let n = match socket.read(&mut buf).await {
        Ok(n) if n == 0 => return, // The socket was closed.
        Ok(n) => n,
        Err(e) => {
            println!("failed to read from socket; err = {:?}", e);
            return;
        }
    };

    let input = match std::str::from_utf8(&buf[0..n]) {
        Ok(s) => s,
        Err(e) => {
            println!("unable to parse input as string: {:?}", e);
            return;
        }
    };

    let args: Vec<&str> = input.split_whitespace().collect();
    if args.len() != 3 {
        // Input should be of the form <PIXELS> <UPPERLEFT> <LOWERRIGHT>. For example:
        //   "1000x750 -1.20,0.35 -1,0.20".
        println!(
            "input string is invalid, should contain 3 arguments, found {}",
            args.len()
        );
    }

    let bounds = match parse_pair(&args[0], 'x') {
        Some(pair) => pair,
        None => {
            println!("{} is not a valid bounds", &args[0]);
            return;
        }
    };

    let upper_left = match parse_pair(&args[1], ',') {
        Some(pair) => pair,
        None => {
            println!("{} is not a valid coordinate", &args[1]);
            return;
        }
    };

    let lower_right = match parse_pair(&args[2], ',') {
        Some(pair) => pair,
        None => {
            println!("{} is not a valid coordinate", &args[2]);
            return;
        }
    };

    let mut pixels = GrayImage::new(bounds.0 as u32, bounds.1 as u32);

    // Divde pixels into horizontal bands. We could probably fine-tune the size of the bands.
    // For example, if the image is small we may want to compute multiple in a single unit of
    // work. I'll leave that as a future exercise for now.
    let bands = pixels.enumerate_rows_mut();
    let mut futures = Vec::with_capacity(bands.len());

    for (row, band) in bands.into_iter() {
        futures.push(async move {
            let band_upper_left = pixel_to_point(bounds, (0, row), upper_left, lower_right);
            let band_lower_right =
                pixel_to_point(bounds, (bounds.0 as u32, row + 1), upper_left, lower_right);
            render(band, band_upper_left, band_lower_right);
        })
    }

    join_all(futures).await;

    let mut img = Vec::with_capacity(pixels.len());

    let encoder = image::codecs::png::PngEncoder::new(&mut img);
    encoder
        .write_image(
            &pixels,
            bounds.0 as u32,
            bounds.1 as u32,
            image::ColorType::L8,
        )
        .unwrap();

    if let Err(e) = socket.write_all(&img).await {
        println!("failed to write image to socket: {:?}", e);
        return;
    }
}
