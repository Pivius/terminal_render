use crossterm::style::Color;
use crate::{frame::{Size, Image, FrameData}, pixel::{PxData, Vector2}, pixel, vector2};

use super::{Energy, Kernel, Scaling};

trait Filter {
    fn apply(&mut self, frame_data: &mut FrameData);
}

pub struct Flip {
    pub horizontal: bool
}

impl Filter for Flip {
    fn apply(&mut self, frame_data: &mut FrameData) {
        let image = frame_data.get_image_mut();
        let size: Size = image.size;
        let pixel_data = image.get_pixel_data();
        
        match pixel_data.len() > 0 && size.width > 0 && size.height > 0 {
            true => {
                let mut flipped_data = vec![PxData::default(); pixel_data.len()];
                let width = size.width;
                let height = size.height;

                for pixel in pixel_data.iter() {
                    let x = pixel.get_x();
                    let y = pixel.get_y();
                    let flipped_x = if self.horizontal { width - x - 1 } else { x };
                    let flipped_y = if !self.horizontal { height - y - 1 } else { y };
                    let flipped_index = (flipped_y * width + flipped_x) as usize;

                    flipped_data[flipped_index] = image.get_pixel(vector2!(x, y));
                }

                image.pixel_data = flipped_data;
            },
            false => {
                panic!("Could not flip, invalid frame data");
            }
        }
    }
}

pub struct Quantize {
    pub shades: u8
}

impl Filter for Quantize {
    fn apply(&mut self, frame_data: &mut FrameData) {
        let image = frame_data.get_image_mut();

        for pixel in image.get_pixel_data_mut().iter_mut() {
            pixel.quantize(self.shades);
        }
    }
}

pub struct Grayscale {
    pub shades: u8
}

impl Filter for Grayscale {
    fn apply(&mut self, frame_data: &mut FrameData) {
        let image = frame_data.get_image_mut();

        match self.shades >= 2 {
            true => {
                let factor = 255 / (self.shades - 1) as u8;

                for pixel in image.get_pixel_data_mut().iter_mut() {
                    let (r, g, b) = pixel.get_color_raw();
                    let average = (r as f64 + g as f64 + b as f64) / 3.0;
                    let gray = ((average / factor as f64).ceil() * factor as f64) as u8;

                    pixel.set_color_raw(gray , gray, gray);
                }
            },
            false => {
                panic!("Shades must be between 2 and 255")
            }
        }
    }
}

pub struct Scale {
    pub size: Size,
    pub scaling: Scaling,
}

impl Filter for Scale {
    fn apply(&mut self, frame_data: &mut FrameData) {
        let image_size = frame_data.get_image().get_image_size();
        let image = frame_data.get_image_mut();

        let (width, height) = (self.size.width, self.size.height);
        let (buffer_width, buffer_height) = (image_size.width, image_size.height);
        let mut resized_pixel_data = Vec::new();

        match self.scaling {
            Scaling::Nearest => {
                for y in 0..height {
                    for x in 0..width {
                        let src_x = x * buffer_width / width;
                        let src_y = y * buffer_height / height;
                        let index = (src_y * buffer_width + src_x) as usize;

                        resized_pixel_data.push(image.get_pixel_data()[index]);
                    }
                }
                image.set_image_size(self.size);
                image.pixel_data = resized_pixel_data;
            },
            Scaling::Bilinear => { //https://x-engineer.org/bilinear-interpolation/
                macro_rules! bilinear_interpolate {
                    ($x:expr, $y:expr, $x1:expr, $y1:expr, $x2:expr, $y2:expr, $q11:expr, $q21:expr, $q12:expr, $q22:expr) => {{
                        let x1 = $x1 as f64;
                        let x2 = $x2 as f64;
                        let y1 = $y1 as f64;
                        let y2 = $y2 as f64;

                        let r1 = ($q11 as f64 * (x2 - $x) + $q21 as f64 * ($x - x1)) / (x2 - x1);
                        let r2 = ($q12 as f64 * (x2 - $x) + $q22 as f64 * ($x - x1)) / (x2 - x1);
                        let p = (r1 * (y2 - $y) + r2 * ($y - y1)) / (y2 - y1);

                        p.round() as u8
                    }};
                }

                for y in 0..height {
                    for x in 0..width {
                        let src_x = (x as f64 * (buffer_width - 1) as f64 / (width - 1) as f64) as f64;
                        let src_y = (y as f64 * (buffer_height - 1) as f64 / (height - 1) as f64) as f64;
                        let x1 = src_x.floor() as u32;
                        let x2 = (x1 + 1).min(buffer_width - 1);
                        let y1 = src_y.floor() as u32;
                        let y2 = (y1 + 1).min(buffer_height - 1);
                        
                        let index11 = (y1 * buffer_width + x1) as usize;
                        let index21 = (y1 * buffer_width + x2) as usize;
                        let index12 = (y2 * buffer_width + x1) as usize;
                        let index22 = (y2 * buffer_width + x2) as usize;
                        let q11 = image.get_pixel_data()[index11];
                        let q21 = image.get_pixel_data()[index21];
                        let q12 = image.get_pixel_data()[index12];
                        let q22 = image.get_pixel_data()[index22];

                        let r = bilinear_interpolate!(
                            src_x, src_y, x1, y1, x2, y2, q11.get_r(), q21.get_r(), q12.get_r(), q22.get_r());
                        let g = bilinear_interpolate!(
                            src_x, src_y, x1, y1, x2, y2, q11.get_g(), q21.get_g(), q12.get_g(), q22.get_g());
                        let b = bilinear_interpolate!(
                            src_x, src_y, x1, y1, x2, y2, q11.get_b(), q21.get_b(), q12.get_b(), q22.get_b());

                        let new_pixel = pixel!(r, g, b, x, y);
                        resized_pixel_data.push(new_pixel);
                    }
                }

                image.set_image_size(self.size);
                image.pixel_data = resized_pixel_data;
            },
            _ => {
                panic!("Invalid scaling type");
            }
        }
    }
}

pub struct GradientMagnitude {
    pub kernel: Kernel
}

impl Filter for GradientMagnitude {
    fn apply(&mut self, frame_data: &mut FrameData) {
        let size = frame_data.get_image().get_image_size();
        let image = frame_data.get_image_mut();
        let mut energy_map = Energy::new(
            image.get_pixel_data().clone(), 
            size.width, 
            size.height, 
            self.kernel.clone()
        );

        energy_map.compute_gradient_magnitude();
        image.pixel_data = energy_map.get_pixel_data();
    }
}

struct MaskOntop {
    other: Image,
    color_mask: Color,
    threshold: u8
}

impl Filter for MaskOntop {
    fn apply(&mut self, frame_data: &mut FrameData) {
        let size = frame_data.get_image().get_image_size();
        let mut new_pixel_data = Vec::new();
        let (width, height) = (size.width, size.height);
        let image = frame_data.get_image_mut();

        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(vector2!(x, y));
                let other_pixel = self.other.get_pixel(vector2!(x, y));
                let (r, g, b) = pixel.get_color_raw();
                let (r2, g2, b2) = other_pixel.get_color_raw();
                let average = ((r as i32 + g as i32 + b as i32) / 3) as u8;
                let average2 = ((r2 as i32 + g2 as i32 + b2 as i32) / 3) as u8;
                let diff = (average as i32 - average2 as i32).abs() as u8;

                if diff > self.threshold {
                    new_pixel_data.push(pixel);
                } else {
                    new_pixel_data.push(other_pixel);
                }
            }
        }

        image.pixel_data = new_pixel_data;
    }
}

struct Ascii {
    shades: String
}

impl Filter for Ascii {
    fn apply(&mut self, frame_data: &mut FrameData) {
        let image = frame_data.get_image_mut();

        let mut ascii_image = String::new();
        let shades = self.shades.chars().collect::<Vec<char>>();
        let shades_len = shades.len() as f64;
        let pixel_data = image.get_pixel_data_mut();

        for pixel in pixel_data.iter_mut() {
            let (r, g, b) = pixel.get_color_raw();
            let average = ((r as f64 + g as f64 + b as f64) / 3.0) as f64;
            let shade_index = (average / 255.0 * shades_len).floor() as usize;
            let shade = shades[shade_index.min(shades_len as usize - 1)];

            pixel.set_character(shade);
        }
    }
}

/*
/ Direction: true = horizontal, false = vertical
    fn seam_carve(&mut self, direction: bool, remove: bool) -> &mut Self {
        // Get the energy map
        let mut energy_map = Energy::new(
            self.get_pixel_data().clone(), 
            self.get_image_size().width, 
            self.get_image_size().height, 
            Kernel::Sobel
        );
        let size = self.get_image_size();

        energy_map.compute_gradient_magnitude();

        //self.pixel_data = energy_map.get_pixel_data();

        // Dynamic programming
        // Find seams
        let mut seams = energy_map.find_seams(100);

        energy_map.remove_seams(seams, true);

        self.pixel_data = energy_map.get_pixel_data();
        self
    }
*/