// See https://www.codeproject.com/Articles/10248/Motion-Detection-Algorithms

use std::io::Error;
use std::{
    fs::{create_dir, read_dir},
    time::Instant,
};

// use rayon::prelude::*;

use anyhow::{Context, Result};
use image::{DynamicImage, GrayImage, ImageBuffer};

enum MotionDetectionStep {
    MotionDetectionFrame {
        motion_frame: DynamicImage,
        motion_level: f32,
    },
    MotionNotDetected,
}

fn merge(image: &GrayImage, overlay: &GrayImage) -> GrayImage {
    let now = Instant::now();
    let (width, height) = image.dimensions();

    // let mut result = ImageBuffer::new(width, height);

    // result
    //     .enumerate_rows_mut()
    //     /* .par_bridge() */
    //     .for_each(|(_y, pixels)| {
    //         pixels.for_each(|(x, y, pixel): (u32, u32, &mut image::Luma<u8>)| {
    //             let src = image.get_pixel(x, y)[0];
    //             let ovr = overlay.get_pixel(x, y)[0];

    //             if src > ovr {
    //                 pixel[0] = src
    //             } else {
    //                 pixel[0] = ovr
    //             }
    //         })
    //     });

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

fn move_towards(img1: &GrayImage, img2: &GrayImage, step_size: u8) -> GrayImage {
    let now = Instant::now();
    let (width, height) = img1.dimensions();

    // let mut result = ImageBuffer::new(width, height);

    // result
    //     .enumerate_rows_mut()
    //     /* .par_bridge() */
    //     .for_each(|(_y, pixels)| {
    //         pixels.for_each(|(x, y, pixel): (u32, u32, &mut image::Luma<u8>)| {
    //             // res = src + Min( Abs( ovr - src ), step ) * Sign( ovr - src )
    //             let src = img1.get_pixel(x, y)[0];
    //             let ovr = img2.get_pixel(x, y)[0];

    //             let sign = if ovr > src { 1 } else { -1 };

    //             let abs_difference = if sign > 0 { ovr - src } else { src - ovr };
    //             let abs_difference = if step_size < abs_difference {
    //                 step_size
    //             } else {
    //                 abs_difference
    //             };

    //             let p = if sign > 0 {
    //                 src.saturating_add(abs_difference)
    //             } else {
    //                 src.saturating_sub(abs_difference)
    //             };

    //             pixel[0] = p;
    //         })
    //     });

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

// TODO mutate in place?
// TODO why u8? do the math
fn difference(img1: &GrayImage, img2: &GrayImage, threshold: u8) -> GrayImage {
    let now = Instant::now();

    let (width, height) = img1.dimensions();

    // let mut result = ImageBuffer::new(width, height);

    // result
    //     .enumerate_rows_mut()
    //     /* .par_bridge() */
    //     .for_each(|(_y, pixels)| {
    //         pixels.for_each(|(x, y, pixel): (u32, u32, &mut image::Luma<u8>)| {
    //             let first = img1.get_pixel(x, y)[0];
    //             let second = img2.get_pixel(x, y)[0];

    //             let difference = if first >= second {
    //                 first - second
    //             } else {
    //                 second - first
    //             };

    //             if difference > threshold {
    //                 pixel[0] = 255;
    //             } else {
    //                 pixel[0] = 0;
    //             }
    //         })
    //     });

    // result.enumerate_pixels_mut()./* par_bridge(). */for_each(
    //     |(x, y, pixel): (u32, u32, &mut image::Luma<u8>)| {
    //         let first = img1.get_pixel(x, y)[0];
    //         let second = img2.get_pixel(x, y)[0];

    //         let difference = if first >= second {
    //             first - second
    //         } else {
    //             second - first
    //         };

    //         if difference > threshold {
    //             pixel[0] = 255;
    //         } else {
    //             pixel[0] = 0;
    //         }
    //     },
    // );

    let result = ImageBuffer::from_fn(width, height, |x, y| {
        // TODO spiegarlo....
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

    println!("difference took {} micros", now.elapsed().as_micros());

    result
}

#[derive(Debug, Default)]
struct MotionDetector {
    background: Option<GrayImage>,
    counter: usize,
}

impl MotionDetector {
    fn new() -> Self {
        Default::default()
    }

    fn process_frame(&mut self, frame: &DynamicImage) -> MotionDetectionStep {
        let current_frame = frame.grayscale().into_luma8();

        if self.background.is_none() {
            self.background = Some(current_frame);

            return MotionDetectionStep::MotionNotDetected;
        }

        self.counter += 1;
        if self.counter == 2 {
            self.background = Some(move_towards(
                self.background.as_ref().unwrap(),
                &current_frame,
                5,
            ));
            self.counter = 0;
        }

        let background = self.background.as_ref().unwrap();

        let motion_frame: GrayImage = difference(&current_frame, background, 15);
        // let motion_frame =
        //     imageproc::morphology::erode(&motion_frame, imageproc::distance_transform::Norm::L1, 1);

        let (width, height) = motion_frame.dimensions();

        let mut white_pixels: i32 = 0;
        for pixel in motion_frame.pixels() {
            white_pixels += (pixel[0] >> 7) as i32;
        }
        let motion_level = white_pixels as f32 / (height as f32 * width as f32);

        MotionDetectionStep::MotionDetectionFrame {
            motion_frame: DynamicImage::ImageLuma8(motion_frame),
            motion_level,
        }
    }
}

fn main() -> Result<()> {
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(4)
    //     .build_global()
    //     .unwrap();

    let mut entries = read_dir("data/Candela_m1.10")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, Error>>()?;

    entries.sort();

    let mut motion_detector = MotionDetector::new();

    // TODO delete if exists
    let _r = create_dir("out");

    for (index, frame_path) in entries.iter().enumerate() {
        dbg!(frame_path);
        let img = image::io::Reader::open(frame_path)?.decode()?;

        let now = Instant::now();
        let step = motion_detector.process_frame(&img);
        println!("Frame processesd in {} ms", now.elapsed().as_millis());

        let mut current_frame = img;
        if let MotionDetectionStep::MotionDetectionFrame {
            motion_level,
            motion_frame,
        } = step
        {
            if motion_level > 0.001 {
                // let open = imageproc::morphology::open(
                //     &motion_frame.to_luma8(),
                //     imageproc::distance_transform::Norm::LInf,
                //     1,
                // );

                // let edges = imageproc::edges::canny(&open, 20_f32, 100_f32);

                let red_channel: GrayImage = imageproc::map::red_channel(&current_frame.to_rgb8());
                let merged = merge(&red_channel, &motion_frame.to_luma8());

                current_frame = DynamicImage::ImageRgb8(imageproc::map::map_colors2(
                    &current_frame,
                    &merged,
                    |i, r| image::Rgb([r[0], i[1], i[2]]),
                ));
            }
        }

        current_frame
            .save(format!("./out/{:05}.png", index))
            .context("Unable to save image")?;
    }

    Ok(())
}
