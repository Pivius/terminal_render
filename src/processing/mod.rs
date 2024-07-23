use crossterm::style::Color;
use crate::{frame::{Size, Image}, pixel::{PxData, Vector2}, pixel, vector2};
pub enum Scaling {
    Nearest,
    Bilinear,
    //Bicubic,
    //Lanczos,
}
pub enum Kernel {
    Sobel,
    Prewitt
}

pub struct Energy {
    pixels: Vec<Vec<i32>>,
    kernel: Kernel
}

pub trait ImageProcess {
    fn flip(&mut self, horizonal: bool, vertical: bool) -> &mut Self;
    fn quantize(&mut self, shades: u8) -> &mut Self;
    fn grayscale(&mut self, shades: u8) -> &mut Self;
    fn scale(&mut self, size: Size, scaling: Scaling, original_size: Size) -> &mut Self;
    fn seam_carve(&mut self, direction: bool, remove: bool) -> &mut Self;
    fn gradient_magnitude(&mut self, kernel: Kernel) -> &mut Self;
    fn mask_ontop(&mut self, other: &Image, color_mask: Color, threshold: u8) -> &mut Self;
}

pub struct Threshold {
    data: Vec<(usize, Color)>,
}

impl Threshold {
    pub fn new(data: Vec<(usize, Color)>) -> Self {
        Self {
            data
        }
    }

    pub fn get_data(&self) -> Vec<(usize, Color)> {
        self.data.clone()
    }

    pub fn get_data_mut(&mut self) -> &mut Vec<(usize, Color)> {
        &mut self.data
    }
}

impl Energy {
    pub fn new(pixel_data: Vec<PxData>, width: u32, height: u32, kernel: Kernel) -> Self {
        let mut grayscale_map: Vec<Vec<i32>> = vec!(vec!(0; width as usize); height as usize);

        for y in 0..height {
            for x in 0..width {
                let index = (y * width + x) as usize;
                let pixel = pixel_data[index];
                let (r, g, b) = pixel.get_color_raw();
                let average = ((r as i32 + g as i32 + b as i32) / 3);

                grayscale_map[y as usize][x as usize] = average;
            }
        }

        Self {
            pixels: grayscale_map,
            kernel
        }
    }

    pub fn get_pixels(&self) -> Vec<Vec<i32>> {
        self.pixels.clone()
    }

    pub fn get_pixels_mut(&mut self) -> &mut Vec<Vec<i32>> {
        &mut self.pixels
    }

    fn get_kernel_matrix(&self) -> (Vec<Vec<i32>>, Vec<Vec<i32>>) {
        match self.kernel {
            Kernel::Sobel => {
                let x = vec![
                    vec![-1, 0, 1],
                    vec![-2, 0, 2],
                    vec![-1, 0, 1]
                ];
                let y = vec![
                    vec![-1, -2, -1],
                    vec![0, 0, 0],
                    vec![1, 2, 1]
                ];
                (x, y)
            },
            Kernel::Prewitt => {
                let x = vec![
                    vec![-1, 0, 1],
                    vec![-1, 0, 1],
                    vec![-1, 0, 1]
                ];
                let y = vec![
                    vec![-1, -1, -1],
                    vec![0, 0, 0],
                    vec![1, 1, 1]
                ];
                (x, y)
            }
        }
    }

    fn compute_gradient(&self, pixel: i32, x: usize, y: usize) -> (i32, i32) {
        let (kx, ky) = self.get_kernel_matrix();
        let mut gx = 0;
        let mut gy = 0;

        for i in 0..3 {
            for j in 0..3 {
                let x_index = x as i32 + i - 1;
                let y_index = y as i32 + j - 1;

                if x_index < 0 || x_index >= self.pixels[0].len() as i32 || y_index < 0 || y_index >= self.pixels.len() as i32{
                    continue;
                }

                gx += kx[i as usize][j as usize] * self.pixels[y_index as usize][x_index as usize];
                gy += ky[i as usize][j as usize] * self.pixels[y_index as usize][x_index as usize];
            }
        }

        (gx, gy)

    }

    pub fn compute_gradient_magnitude(&mut self) {
        let mut result = vec![vec![0; self.pixels[0].len()]; self.pixels.len()];

        for i in 0..self.pixels.len() {
            for j in 0..self.pixels[0].len() {
                let (gx, gy) = self.compute_gradient(self.pixels[i][j], j, i);
                let magnitude = ((gx.pow(2) + gy.pow(2)) as f64).sqrt();

                result[i][j] = magnitude as i32;
            }
        }

        self.pixels = result;
    }

    pub fn get_pixel_data(&self) -> Vec<PxData> {
        let mut pixel_data = Vec::new();

        for i in 0..self.pixels.len() {
            for j in 0..self.pixels[0].len() {
                let pixel_value = self.pixels[i][j] as u8;
                pixel_data.push(pixel!(pixel_value, pixel_value, pixel_value, j as u32, i as u32));
            }
        }

        pixel_data
    }
    pub fn find_seams(&self, seam_count: usize) -> Vec<usize> {
        let mut result: Vec<usize> = Vec::new();
        let mut energy_map = self.pixels.clone();

        for _ in 0..seam_count {
            let mut seam = Vec::new();
            let mut min_energy = i32::MAX;
            let mut min_index = 0;

            for i in 0..energy_map[0].len() {
                let mut energy = energy_map[0][i];

                for j in 1..energy_map.len() {
                    let left = if i > 0 {energy_map[j - 1][i - 1]} else {i32::MAX};
                    let right = if i < energy_map[0].len() - 1 {energy_map[j - 1][i + 1]} else {i32::MAX};

                    energy += left.min(energy_map[j - 1][i]).min(right);
                }

                if energy < min_energy {
                    min_energy = energy;
                    min_index = i;
                }
            }

            seam.push(min_index);
            let mut index = min_index;

            for i in (1..energy_map.len()).rev() {
                let left = if index > 0 {energy_map[i - 1][index - 1]} else {i32::MAX};
                let right = if index < energy_map[0].len() - 1 {energy_map[i - 1][index + 1]} else {i32::MAX};
                let mut min_energy = energy_map[i - 1][index];
                let mut min_index = index;

                if left < min_energy {
                    min_energy = left;
                    min_index = index - 1;
                }

                if right < min_energy {
                    min_index = index + 1;
                }

                seam.push(min_index);
                index = min_index;
            }

            result.push(min_index);
            let mut new_energy_map = Vec::new();

            for i in 0..energy_map.len() {
                let mut row = Vec::new();

                for j in 0..energy_map[0].len() {
                    if j != seam[i] {
                        row.push(energy_map[i][j]);
                    }
                }

                new_energy_map.push(row);
            }

            energy_map = new_energy_map;
        }

        result
    }

    // If scale is true, then the missing pixels will be interpolated
    pub fn remove_seams(&mut self, seams: Vec<usize>, scale: bool) {
        let mut result = Vec::new();

        for i in 0..self.pixels.len() {
            let mut row = Vec::new();
            let mut seam = seams.clone();
            let mut index = 0;

            for j in 0..self.pixels[0].len() {
                if seam.len() > 0 && seam[0] == j {
                    seam.remove(0);
                } else {
                    row.push(self.pixels[i][index]);
                    index += 1;
                }
            }

            result.push(row);
        }
        self.pixels = result;
    }

    pub fn add_seams(&mut self, seams: Vec<Vec<usize>>) {
        let mut result = Vec::new();

        for i in 0..self.pixels.len() {
            let mut row = Vec::new();
            let mut seam = seams[i].clone();
            let mut index = 0;

            for j in 0..self.pixels[0].len() {
                if seam.len() > 0 && seam[0] == j {
                    row.push(self.pixels[i][index]);
                    row.push(self.pixels[i][index]);
                    seam.remove(0);
                } else {
                    row.push(self.pixels[i][index]);
                }

                index += 1;
            }

            result.push(row);
        }

        self.pixels = result;
    }
}