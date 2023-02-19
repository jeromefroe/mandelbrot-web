use std::str::FromStr;

use image::buffer::EnumeratePixelsMut;
use image::Luma;
use num::Complex;

/// Parse the string `s` as a coordinate pair, like `"400x600"` or `"1.0,0.5"`.
///
/// Specifically, `s` should have the form <left><sep><right>, where <sep> is
/// the character given by the `separator` argument, and <left> and <right> are both
/// strings that can be parsed by `T::from_str`.
///
/// If `s` has the proper form, return `Some<(x, y)>`. If it doesn't parse
/// correctly, return `None`.
pub fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
    }
}

/// Return the point on the complex plane corresponding to a given pixel in the
/// bitmap.
///
/// `bounds` is a pair giving the width and height of the bitmap. `pixel` is a
/// pair indicating a particular pixel in that bitmap. The `upper_left` and
/// `lower_right` parameters are points on the complex plane designating the
/// area our bitmap covers.
pub fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (u32, u32),
    upper_left: (f64, f64),
    lower_right: (f64, f64),
) -> (f64, f64) {
    // It might be nicer to find the position of the *middle* of the pixel,
    // instead of its upper left corner, but this is easier to write tests for.
    let (width, height) = (lower_right.0 - upper_left.0, upper_left.1 - lower_right.1);
    (
        upper_left.0 + pixel.0 as f64 * width / bounds.0 as f64,
        upper_left.1 - pixel.1 as f64 * height / bounds.1 as f64,
    )
}

/// Try to determine whether the complex number `c` is in the Mandelbrot set.
///
/// A number `c` is in the set if, starting with zero, repeatedly squaring and
/// adding `c` never causes the number to leave the circle of radius 2 centered
/// on the origin; the number instead orbits near the origin forever. (If the
/// number does leave the circle, it eventually flies away to infinity.)
///
/// If after `limit` iterations our number has still not left the circle, return
/// `None`; this is as close as we come to knowing that `c` is in the set.
///
/// If the number does leave the circle before we give up, return `Some(i)`, where
/// `i` is the number of iterations it took.
fn escapes(c: Complex<f64>, limit: u32) -> Option<u32> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        z = z * z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }

    return None;
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels.
///
/// The `upper_left` and `lower_right` arguments specify points on the complex plane
/// corresponding to the upper left and lower right corners of the pixel buffer.
pub fn render(buf: EnumeratePixelsMut<Luma<u8>>, upper_left: (f64, f64), lower_right: (f64, f64)) {
    let width = buf.len();
    let pixels = buf.enumerate();

    for (col, pixel) in pixels.into_iter() {
        let point = pixel_to_point((width, 1), (col as u32, 0), upper_left, lower_right);
        let val = match escapes(
            Complex {
                re: point.0,
                im: point.1,
            },
            255,
        ) {
            None => 0,
            Some(count) => 255 - count as u8,
        };
        *pixel.2 = Luma([val]);
    }
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("", ','), None);
    assert_eq!(parse_pair::<i32>("10,", ','), None);
    assert_eq!(parse_pair::<i32>(",10", ','), None);
    assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x", 'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(
        pixel_to_point((100, 100), (25, 75), (-1.0, 1.0), (1.0, -1.0)),
        (-0.5, -0.5)
    );
}
