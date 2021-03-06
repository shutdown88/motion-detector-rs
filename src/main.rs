// See https://www.codeproject.com/Articles/10248/Motion-Detection-Algorithms

use std::io::Error;
use std::{
    fs::{create_dir, read_dir},
    time::Instant,
};

use anyhow::{Context, Result};
use image::{DynamicImage, GrayImage};
// use ops::{difference, merge, move_towards};
use ops::parralel::{difference, merge, move_towards};

mod ops;

enum MotionDetectionStep {
    MotionDetectionFrame {
        motion_frame: DynamicImage,
        motion_level: f32,
    },
    MotionNotDetected,
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

    let main_start_instant = Instant::now();

    let mut entries = read_dir("data/frames_0")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, Error>>()?;

    entries.sort();

    let mut motion_detector = MotionDetector::new();

    // TODO delete if exists
    let _r = create_dir("out");

    for (index, frame_path) in entries.iter().enumerate() {
        let start_frame_instant = Instant::now();
        dbg!(frame_path);

        let open_image_instant = Instant::now();
        let img = image::io::Reader::open(frame_path)?.decode()?;
        println!(
            "image read in {} ms",
            open_image_instant.elapsed().as_millis()
        );

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

        let write_image_instant = Instant::now();
        current_frame
            .save(format!("./out/{:05}.png", index))
            .context("Unable to save image")?;
        println!(
            "image written in {}",
            write_image_instant.elapsed().as_millis()
        );

        println!(
            "frame created in {}",
            start_frame_instant.elapsed().as_millis()
        );
    }

    println!("main took {} ms", main_start_instant.elapsed().as_millis());

    Ok(())
}
