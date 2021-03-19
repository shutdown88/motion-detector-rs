// See https://www.codeproject.com/Articles/10248/Motion-Detection-Algorithms

use std::{env, fs::write, io::Error};
use std::{
    fs::{create_dir, read_dir},
    time::Instant,
};

use anyhow::Result;
use image::{DynamicImage, GrayImage};
use rayon::prelude::*;
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
        let convert_greyscale_instant = Instant::now();
        // let current_frame = frame.grayscale().into_luma8();
        let current_frame = ops::parralel::dyn_grayscale(frame);
        println!(
            "converting image grayscale took {} ms",
            convert_greyscale_instant.elapsed().as_millis()
        );

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

        let count_pixel_instant = Instant::now();
        let mut white_pixels: i32 = 0;
        for pixel in motion_frame.pixels() {
            white_pixels += (pixel[0] >> 7) as i32;
        }
        println!(
            "counting motion pixels took {} ms",
            count_pixel_instant.elapsed().as_millis()
        );

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

    let dir = env::args().nth(1).unwrap();

    let mut entries = read_dir(dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, Error>>()?;

    entries.sort();

    let mut motion_detector = MotionDetector::new();

    let _r = std::fs::remove_dir_all("./out");
    let _r = create_dir("out");

    let mut index: usize = 0;

    let mut out = String::new();

    entries.chunks(10).for_each(|chunk| {
        let images_chunk: Vec<DynamicImage> = chunk
            .par_iter()
            .map(|frame_path| {
                let open_image_instant = Instant::now();
                println!("Start decoding frame {:?}", frame_path);
                let img = image::io::Reader::open(frame_path)
                    .unwrap()
                    .decode()
                    .unwrap();
                println!(
                    "Frame {:?} decoded in {} ms",
                    frame_path,
                    open_image_instant.elapsed().as_millis()
                );
                img
            })
            .collect();

        let mut processed: Vec<DynamicImage> = Vec::with_capacity(10);

        for img in images_chunk {
            let now = Instant::now();
            let step = motion_detector.process_frame(&img);
            println!("Frame processesd in {} ms", now.elapsed().as_millis());

            let mut current_frame = img;
            if let MotionDetectionStep::MotionDetectionFrame {
                motion_level,
                motion_frame,
            } = step
            {
                out += format!("{}\n", motion_level).as_str();
                if motion_level > 0.006 {
                    // let open = imageproc::morphology::open(
                    //     &motion_frame.to_luma8(),
                    //     imageproc::distance_transform::Norm::LInf,
                    //     1,
                    // );

                    // let edges = imageproc::edges::canny(&open, 20_f32, 100_f32);

                    let red_channel: GrayImage =
                        imageproc::map::red_channel(&current_frame.to_rgb8());
                    let merged = merge(&red_channel, &motion_frame.to_luma8());

                    current_frame = DynamicImage::ImageRgb8(imageproc::map::map_colors2(
                        &current_frame,
                        &merged,
                        |i, r| image::Rgb([r[0], i[1], i[2]]),
                    ));

                    processed.push(current_frame);
                }
            }

            // processed.push(current_frame);

            // let write_image_instant = Instant::now();
            // current_frame
            //     .save(format!("./out/{:05}.png", index))
            //     .expect("Unable to save image");
            // println!(
            //     "image written in {}",
            //     write_image_instant.elapsed().as_millis()
            // );
        }

        processed
            .par_iter()
            .enumerate()
            .for_each(|(i, current_frame)| {
                let write_image_instant = Instant::now();
                let name = format!("./out/{:05}.png", index + i);
                println!("Start writing image {}", name);
                current_frame.save(&name).expect("Unable to save image");
                println!(
                    "image {} written in {}",
                    name,
                    write_image_instant.elapsed().as_millis()
                );
            });

        index += processed.len();
    });

    write("out.csv", out).expect("error writing out csv");

    println!("main took {} ms", main_start_instant.elapsed().as_millis());

    Ok(())
}
