//use termcolor::Color;
use crossterm::style::Color;
use crate::processing::{ImageProcess, Scaling, Energy, Kernel};
use crate::{
    pixel,
    vector2,
    pixel::{PxData, Vector2},
};

#[derive(Copy, Clone, Default)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Default, Clone)]
pub struct Image {
    pub pixel_data: Vec<PxData>,
    pub size: Size,
    multiplier: u8,
}

#[derive(Default, Clone)]
pub struct FrameData {
    buffer_size: Size,
    raw_data: Vec<u8>,
    image: Image,
    multiplier: u8,
}


impl FrameData {
    pub fn new(width: u32, height: u32, multiplier: u8) -> Self {
        let buffer_width = 0;
        let buffer_height = 0;

        Self {
            buffer_size: Size{width: buffer_width, height: buffer_height},
            raw_data: Vec::new(),
            image: Image::default(),
            multiplier,
        }
    }
    
    pub fn get_buffer_size(&self) -> Size {
        self.buffer_size
    }

    pub fn set_multiplier(&mut self, multiplier: u8) {
        self.multiplier = multiplier;
    }

    pub fn set_raw_data(&mut self, data: Vec<u8>, buffer_size: Size) {
        let step = self.multiplier as usize;
        let mut pixel_data = Vec::new();

        self.raw_data = data;
        self.buffer_size = buffer_size;

        for i in (0..self.raw_data.len()).step_by(step) {
            let quotient = (i / step) as u32;
            let x = quotient % self.buffer_size.width;
            let y = quotient / self.buffer_size.height;
            let pixel = pixel!(self.raw_data[i], self.raw_data[i + 1], self.raw_data[i + 2], x, y);

            pixel_data.push(pixel);
        }

        self.image = Image::new(pixel_data, self.buffer_size, self.multiplier);
    }

    pub fn raw_buffer(&self) -> &Vec<u8> {
        &self.raw_data
    }

    pub fn clone_image(&self) -> Image {
        self.image.clone()
    }

    pub fn get_image(&self) -> &Image {
        &self.image
    }

    pub fn get_image_mut(&mut self) -> &mut Image {
        &mut self.image
    }

    pub fn set_image(&mut self, image: Image) {
        self.image = image;
    }
}

impl Image {
    pub fn new(pixel_data: Vec<PxData>, size: Size, multiplier: u8) -> Self {
        Self {
            pixel_data,
            size,
            multiplier,
        }
    }

    pub fn black(size: Size) -> Self {
        let mut pixel_data = Vec::new();

        for y in 0..size.height {
            for x in 0..size.width {
                pixel_data.push(pixel!(0, 0, 0, x, y));
            }
        }

        Self {
            pixel_data,
            size,
            multiplier: 4,
        }
    }

    pub fn get_image_size(&self) -> Size {
        self.size
    }

    pub fn set_image_size(&mut self, size: Size) {
        self.size = size;
    }

    fn get_pixel_index(&self, position: Vector2) -> usize {
        (position.get_y() * self.get_image_size().width + position.get_x()) as usize
    }

    pub fn get_pixel(&self, position: Vector2) -> PxData {
        let index = self.get_pixel_index(position);
        self.pixel_data[index]
    }

    pub fn get_pixel_mut(&mut self, position: Vector2) -> &mut PxData {
        let index = self.get_pixel_index(position);
        &mut self.pixel_data[index]
    }

    pub fn get_pixel_data(&self) -> &Vec<PxData>{
        &self.pixel_data
    }

    pub fn get_pixel_data_mut(&mut self) -> &mut Vec<PxData> {
        &mut self.pixel_data
    }
}

impl ImageProcess for Image {
    fn flip(&mut self, horizontal: bool, vertical: bool) -> &mut Self {
        let size: Size = self.size;
        let pixel_data = self.get_pixel_data();
        
        match pixel_data.len() > 0 && size.width > 0 && size.height > 0 {
            true => {
                let mut flipped_data = vec![PxData::default(); pixel_data.len()];
                let width = size.width;
                let height = size.height;

                for pixel in pixel_data.iter() {
                    let x = pixel.get_x();
                    let y = pixel.get_y();
                    let flipped_x = if horizontal { width - x - 1 } else { x };
                    let flipped_y = if vertical { height - y - 1 } else { y };
                    let flipped_index = (flipped_y * width + flipped_x) as usize;

                    flipped_data[flipped_index] = self.get_pixel(vector2!(x, y));
                }

                self.pixel_data = flipped_data;
            },
            false => {
                panic!("Could not flip, invalid frame data");
            }
        }

        self
    }

    fn quantize(&mut self, shades: u8) -> &mut Self {
        for pixel in self.get_pixel_data_mut().iter_mut() {
            pixel.quantize(shades);
        }

        self
    }

    fn grayscale(&mut self, shades: u8) -> &mut Self {
        match shades >= 2 {
            true => {
                let factor = 255 / (shades - 1) as u8;

                for pixel in self.get_pixel_data_mut().iter_mut() {
                    let (r, g, b) = pixel.get_color_raw();
                    let average = (r as f64 + g as f64 + b as f64) / 3.0;
                    let gray = (average / factor as f64).ceil() * factor as f64;

                    pixel.set_color_raw(gray as u8, gray as u8, gray as u8);
                }
            },
            false => {
                panic!("Shades must be between 2 and 255")
            }
        }
        self
    }

    fn scale(&mut self, size: Size, scaling: Scaling, original_size: Size) -> &mut Self {
        let (width, height) = (size.width, size.height);

        let buffer_width = original_size.width;
        let buffer_height = original_size.height;
        let mut resized_pixel_data = Vec::new();

        match scaling {
            Scaling::Nearest => {
                for y in 0..height {
                    for x in 0..width {
                        let src_x = x * buffer_width / width;
                        let src_y = y * buffer_height / height;
                        let index = (src_y * buffer_width + src_x) as usize;

                        resized_pixel_data.push(self.get_pixel_data()[index]);
                    }
                }
                self.set_image_size(size);
                self.pixel_data = resized_pixel_data;
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
                        let q11 = self.get_pixel_data()[index11];
                        let q21 = self.get_pixel_data()[index21];
                        let q12 = self.get_pixel_data()[index12];
                        let q22 = self.get_pixel_data()[index22];

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

                self.set_image_size(size);
                self.pixel_data = resized_pixel_data;
            },
            _ => {
                panic!("Invalid scaling type");
            }
        }

        self
    }

    // Direction: true = horizontal, false = vertical
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

    fn gradient_magnitude(&mut self, kernel: Kernel) -> &mut Self {
        let mut energy_map = Energy::new(
            self.get_pixel_data().clone(), 
            self.get_image_size().width, 
            self.get_image_size().height, 
            kernel
        );

        energy_map.compute_gradient_magnitude();
        self.pixel_data = energy_map.get_pixel_data();

        self
    }

    // Combine two images based on a threshold and a color
    fn mask_ontop(&mut self, other: &Image, color_mask: Color, threshold: u8) -> &mut Self {
        let mut new_pixel_data = Vec::new();
        let (width, height) = (self.get_image_size().width, self.get_image_size().height);

        for y in 0..height {
            for x in 0..width {
                let index = (y * width + x) as usize;
                let pixel = self.get_pixel(vector2!(x, y));
                let other_pixel = other.get_pixel(vector2!(x, y));
                let (r, g, b) = pixel.get_color_raw();
                let (r2, g2, b2) = other_pixel.get_color_raw();
                let average = ((r as i32 + g as i32 + b as i32) / 3) as u8;
                let average2 = ((r2 as i32 + g2 as i32 + b2 as i32) / 3) as u8;
                let diff = (average as i32 - average2 as i32).abs() as u8;

                if diff > threshold {
                    new_pixel_data.push(pixel);
                } else {
                    new_pixel_data.push(other_pixel);
                }
            }
        }

        self.pixel_data = new_pixel_data;
        self
    }

    fn get_ascii(&self, shades: String) -> String {
        let mut ascii_image = String::new();
        let shades = shades.chars().collect::<Vec<char>>();
        let shades_len = shades.len() as f64;
        let pixel_data = self.get_pixel_data();

        for pixel in pixel_data.iter() {
            let (r, g, b) = pixel.get_color_raw();
            let average = ((r as f64 + g as f64 + b as f64) / 3.0) as f64;
            let shade_index = (average / 255.0 * shades_len).floor() as usize;
            let shade = shades[shade_index.min(shades_len as usize - 1)];

            ascii_image.push(shade);
        }

        ascii_image
    }

    fn brightness(&mut self, value: i32) -> &mut Self {
        for pixel in self.get_pixel_data_mut().iter_mut() {
            let (r, g, b) = pixel.get_color_raw();
            let r = (r as i32 + value).max(0).min(255) as u8;
            let g = (g as i32 + value).max(0).min(255) as u8;
            let b = (b as i32 + value).max(0).min(255) as u8;

            pixel.set_color_raw(r, g, b);
        }

        self
    }
}