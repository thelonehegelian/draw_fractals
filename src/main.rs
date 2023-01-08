use image::{ImageBuffer, Pixel, Rgb, RgbImage};
use num::complex::Complex;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;

fn main() {
    // Save the image to a file or display it in some way...
    let (width, height) = (1920, 1080);
    let mut img = ImageBuffer::new(width, height);
    let complex = Complex::new(-0.7269, 0.1889);
    let iterations = 300;

    // create a thread pool
    let pool = ThreadPool::new(num_cpus::get());

    let (sender, receiver) = channel();

    // divide the task with threads
    for y in 0..height {
        let sender = sender.clone();
        pool.execute(move || {
            for x in 0..width {
                let i = julia(complex, x, y, width, height, iterations);
                let pixel = wavelength_to_rgb(380 + i * 400 / iterations);
                sender.send((x, y, pixel)).expect("Error");
            }
        });
    }

    for _ in 0..(width * height) {
        let (x, y, pixels) = receiver.recv().unwrap();
        img.put_pixel(x, y, pixels);
    }
    img.save("fractal.png").unwrap();
}

// julia set creator
fn julia(c: Complex<f32>, x: u32, y: u32, width: u32, height: u32, max_iter: u32) -> u32 {
    let width = width as f32;
    let height = height as f32;

    let mut z = Complex {
        // scale and translate the point to image coordinates
        re: 3.0 * (x as f32 - 0.5 * width) / width,
        im: 2.0 * (y as f32 - 0.5 * height) / height,
    };
    let mut i = 0;
    for t in 0..max_iter {
        if z.norm() >= 2.0 {
            break;
        }
        z = z * z + c;
        i = t;
    }
    i
}

// Function converting intensity values to RGB
// Based on http://www.efg2.com/Lab/ScienceAndEngineering/Spectra.htm
fn wavelength_to_rgb(wavelength: u32) -> Rgb<u8> {
    // Normalizes color intensity values within RGB range
    fn normalize(color: f32, factor: f32) -> u8 {
        ((color * factor).powf(0.8) * 255.) as u8
    }
    let wave = wavelength as f32;

    let (r, g, b) = match wavelength {
        380..=439 => ((440. - wave) / (440. - 380.), 0.0, 1.0),
        440..=489 => (0.0, (wave - 440.) / (490. - 440.), 1.0),
        490..=509 => (0.0, 1.0, (510. - wave) / (510. - 490.)),
        510..=579 => ((wave - 510.) / (580. - 510.), 1.0, 0.0),
        580..=644 => (1.0, (645. - wave) / (645. - 580.), 0.0),
        645..=780 => (1.0, 0.0, 0.0),
        _ => (0.0, 0.0, 0.0),
    };

    let factor = match wavelength {
        380..=419 => 0.3 + 0.7 * (wave - 380.) / (420. - 380.),
        701..=780 => 0.3 + 0.7 * (780. - wave) / (780. - 700.),
        _ => 1.0,
    };

    let (r, g, b) = (
        normalize(r, factor),
        normalize(g, factor),
        normalize(b, factor),
    );
    Rgb::from_channels(r, g, b, 0)
}
