
use std::io::{self, Write};

use windows_capture::settings::ColorFormat;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct Canvas {
    width: usize,
    height: usize,
    color_format: ColorFormat,
    buffer: Vec<u8>,
}

impl Canvas {
    pub fn new(width: usize, height: usize, color_format: ColorFormat) -> Self {
        let multiplier = match color_format {
            ColorFormat::Rgba16F => 8,
            ColorFormat::Rgba8 => 4,
            ColorFormat::Bgra8 => 4,
        };

        Self {
            width,
            height,
            color_format,
            buffer: vec![0; width * height * multiplier],
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        let multiplier = match self.color_format {
            ColorFormat::Rgba16F => 8,
            ColorFormat::Rgba8 => 4,
            ColorFormat::Bgra8 => 4,
        };
        self.buffer = vec![0; width * height * multiplier];
    }

    pub fn clear(&mut self) {
        self.buffer.iter_mut().for_each(|x| *x = 0);
        // Clear image
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel_data: &[u8]) {
        let index = (y * self.width + x) * 4;
        let r = index;
        let g = index + 1;
        let b = index + 2;
        let a = index + 3;

        self.buffer[r] = pixel_data[0];
        self.buffer[g] = pixel_data[1];
        self.buffer[b] = pixel_data[2];
        self.buffer[a] = pixel_data[3];
    }

    pub fn quantize_buffer(width: u32, height: u32, buffer: &[u8], amount: u32) -> (Vec<u8>, u32, u32) {
        // Now we lower the resolution of the image, and by doing so the image will be lower sized
        // but the colors will be more uniform

        // We will do this by taking the average of the colors in a block of pixels
        // and then setting all the pixels in that block to that average color
        let mut new_buffer = vec![]; // Dont know the size yet

        let block_width = width / amount;
        let block_height = height / amount;

        for y in 0..block_height {
            for x in 0..block_width {
                let mut total_r = 0;
                let mut total_g = 0;
                let mut total_b = 0;
                let mut total_a = 0;

                for block_y in 0..amount {
                    for block_x in 0..amount {
                        let index: usize = (((y * amount + block_y) * width + x * amount + block_x) * 4) as usize;
                        total_r += buffer[index] as u32;
                        total_g += buffer[index + 1] as u32;
                        total_b += buffer[index + 2] as u32;
                        total_a += buffer[index + 3] as u32;
                    }
                }

                let average_r = (total_r / (amount * amount)) as u8;
                let average_g = (total_g / (amount * amount)) as u8;
                let average_b = (total_b / (amount * amount)) as u8;
                let average_a = (total_a / (amount * amount)) as u8;

                for block_y in 0..amount {
                    for block_x in 0..amount {
                        new_buffer.push(average_r);
                        new_buffer.push(average_g);
                        new_buffer.push(average_b);
                        new_buffer.push(average_a);
                    }
                }
            }
        }
        (new_buffer, block_width, block_height)
    }
    
    pub fn push_buffer(&mut self, buffer: &[u8], buffer_width: usize, buffer_height: usize) {
        let mut resized_buffer = vec![0; self.width * self.height * 4];

        for y in 0..self.height {
            for x in 0..self.width {
                let src_x = x * buffer_width / self.width;
                let src_y = y * buffer_height / self.height;
                let src_index = (src_y * buffer_width + src_x) * 4;

                let dst_index = (y * self.width + x) * 4;
                resized_buffer[dst_index] = buffer[src_index];
                resized_buffer[dst_index + 1] = buffer[src_index + 1];
                resized_buffer[dst_index + 2] = buffer[src_index + 2];
                resized_buffer[dst_index + 3] = buffer[src_index + 3];
            }
        }

        self.buffer = resized_buffer;
    }

    pub fn render(&self) {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        let mut color_spec = ColorSpec::new();

        // ANSI escape code to move the cursor to the top-left corner
        print!("{esc}c", esc = 27 as char);

        for y in 0..1 {
            let mut line = String::new();
            for x in 0..1 {
                let index = (y * self.width + x) * 4;
                let r = self.buffer[index];
                let g = self.buffer[index + 1];
                let b = self.buffer[index + 2];
                // let a = self.buffer[index + 3];

                color_spec.set_fg(Some(Color::Rgb(r, g, b)));
                stdout.set_color(&color_spec).unwrap();
                line.push('█');
            }
            println!("{}", line);
        }

        io::stdout().flush().unwrap();

        // Reset the color to default
        stdout.reset().unwrap();
    }
}