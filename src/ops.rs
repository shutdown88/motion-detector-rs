use std::time::Instant;

use image::{GrayImage, ImageBuffer};

pub fn merge(image: &GrayImage, overlay: &GrayImage) -> GrayImage {
    let now = Instant::now();

    let (width, height) = image.dimensions();
    let result = ImageBuffer::from_fn(width, height, |x, y| {
        let src = image.get_pixel(x, y)[0];
        let ovr = overlay.get_pixel(x, y)[0];

        if src > ovr {
            image::Luma([src])
        } else {
            image::Luma([ovr])
        }
    });
    println!("merge took {} ms", now.elapsed().as_millis());
    result
}

pub fn move_towards(img1: &GrayImage, img2: &GrayImage, step_size: u8) -> GrayImage {
    let now = Instant::now();

    let (width, height) = img1.dimensions();
    let result = ImageBuffer::from_fn(width, height, |x, y| {
        // res = src + Min( Abs( ovr - src ), step ) * Sign( ovr - src )
        let src = img1.get_pixel(x, y)[0];
        let ovr = img2.get_pixel(x, y)[0];

        let sign = if ovr > src { 1 } else { -1 };

        let abs_difference = if sign > 0 { ovr - src } else { src - ovr };
        let abs_difference = if step_size < abs_difference {
            step_size
        } else {
            abs_difference
        };

        let p = if sign > 0 {
            src.saturating_add(abs_difference)
        } else {
            src.saturating_sub(abs_difference)
        };

        image::Luma([p])
    });

    println!("move_towards took {} ms", now.elapsed().as_millis());
    result
}

pub fn difference(img1: &GrayImage, img2: &GrayImage, threshold: u8) -> GrayImage {
    let now = Instant::now();

    let (width, height) = img1.dimensions();
    let result = ImageBuffer::from_fn(width, height, |x, y| {
        let first = img1.get_pixel(x, y)[0];
        let second = img2.get_pixel(x, y)[0];

        let difference = if first >= second {
            first - second
        } else {
            second - first
        };

        if difference > threshold {
            image::Luma([255])
        } else {
            image::Luma([0])
        }
    });

    println!("difference took {} ms", now.elapsed().as_millis());

    result
}

pub mod parralel {
    use std::time::Instant;

    use image::{GrayImage, ImageBuffer};
    use rayon::prelude::*;

    pub fn merge(image: &GrayImage, overlay: &GrayImage) -> GrayImage {
        let now = Instant::now();

        let (width, height) = image.dimensions();
        let mut result = ImageBuffer::new(width, height);
        result
            .enumerate_rows_mut()
            .par_bridge()
            .for_each(|(_y, pixels)| {
                pixels.for_each(|(x, y, pixel): (u32, u32, &mut image::Luma<u8>)| {
                    let src = image.get_pixel(x, y)[0];
                    let ovr = overlay.get_pixel(x, y)[0];

                    if src > ovr {
                        pixel[0] = src
                    } else {
                        pixel[0] = ovr
                    }
                })
            });

        println!("parallel merge took {} ms", now.elapsed().as_millis());

        result
    }

    pub fn move_towards(img1: &GrayImage, img2: &GrayImage, step_size: u8) -> GrayImage {
        let now = Instant::now();

        let (width, height) = img1.dimensions();
        let mut result = ImageBuffer::new(width, height);

        result
            .enumerate_rows_mut()
            .par_bridge()
            .for_each(|(_y, pixels)| {
                pixels.for_each(|(x, y, pixel): (u32, u32, &mut image::Luma<u8>)| {
                    // res = src + Min( Abs( ovr - src ), step ) * Sign( ovr - src )
                    let src = img1.get_pixel(x, y)[0];
                    let ovr = img2.get_pixel(x, y)[0];

                    let sign = if ovr > src { 1 } else { -1 };

                    let abs_difference = if sign > 0 { ovr - src } else { src - ovr };
                    let abs_difference = if step_size < abs_difference {
                        step_size
                    } else {
                        abs_difference
                    };

                    let p = if sign > 0 {
                        src.saturating_add(abs_difference)
                    } else {
                        src.saturating_sub(abs_difference)
                    };

                    pixel[0] = p;
                })
            });

        println!(
            "parallel move_towards took {} ms",
            now.elapsed().as_millis()
        );

        result
    }

    pub fn difference(img1: &GrayImage, img2: &GrayImage, threshold: u8) -> GrayImage {
        let now = Instant::now();

        let (width, height) = img1.dimensions();
        let mut result = ImageBuffer::new(width, height);
        result
            .enumerate_rows_mut()
            .par_bridge()
            .for_each(|(_y, pixels)| {
                pixels.for_each(|(x, y, pixel): (u32, u32, &mut image::Luma<u8>)| {
                    let first = img1.get_pixel(x, y)[0];
                    let second = img2.get_pixel(x, y)[0];

                    let difference = if first >= second {
                        first - second
                    } else {
                        second - first
                    };

                    if difference > threshold {
                        pixel[0] = 255;
                    } else {
                        pixel[0] = 0;
                    }
                })
            });

        println!("parallel difference took {} ms", now.elapsed().as_millis());

        result
    }
}
