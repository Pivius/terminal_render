// # Stdout Raw handle
extern crate kernel32;
extern crate winapi;
use std::os::windows::io::FromRawHandle;
use std::fs::File;
use std::mem;
use crossterm::style::Color;
use winapi::um::winnt::HANDLE;
use winapi::um::wincon::{SetConsoleOutputCP, GetConsoleScreenBufferInfo, COORD};
use winapi::um::winnls::CP_UTF8;

// # Terminal handling
use std::io::Write;
use crossterm::{cursor, execute, style::Stylize, terminal};
use windows_capture::settings::ColorFormat;

// # FrameData
use crate::frame::{FrameData, Size};
use crate::processing::{ImageProcess, Kernel, Scaling};

pub struct Canvas {
    frame_data: FrameData,
}

impl Canvas {
    pub fn new(color_format: ColorFormat) -> Self {
        let multiplier = match color_format {
            ColorFormat::Rgba16F => 8,
            ColorFormat::Rgba8 => 4,
            ColorFormat::Bgra8 => 4,
        };
        let mut frame_data = FrameData::default();
        frame_data.set_multiplier(multiplier);
        terminal::enable_raw_mode().unwrap();
        
        Self {
            frame_data,
        }
    }

    pub fn get_frame_data(&mut self) -> &mut FrameData {
        &mut self.frame_data
    }

    pub fn push_buffer(&mut self, buffer: Vec<u8>, buffer_width: u32, buffer_height: u32) {
        self.frame_data.set_raw_data(buffer, Size {width: buffer_width, height: buffer_height});
    }

    pub unsafe fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Windows terminal stuff
        SetConsoleOutputCP(CP_UTF8);
        let h = winapi::um::winbase::STD_OUTPUT_HANDLE;
        let console_handle = kernel32::GetStdHandle(h);
        let mut stdout = File::from_raw_handle(console_handle as *mut _);
        let mut screen_buffer_info = winapi::um::wincon::CONSOLE_SCREEN_BUFFER_INFO {
            dwSize: COORD { X: 1 as i16, Y: 2 as i16 },
            dwCursorPosition: COORD { X: 0, Y: 0 },
            wAttributes: 0,
            srWindow: winapi::um::wincon::SMALL_RECT { Left: 0, Top: 0, Right: 1 as i16, Bottom: 2 as i16 },
            dwMaximumWindowSize: COORD { X: 1 as i16, Y: 2 as i16 },
        };
        unsafe{GetConsoleScreenBufferInfo(console_handle as HANDLE, &mut screen_buffer_info)};
        let cols = screen_buffer_info.srWindow.Right - screen_buffer_info.srWindow.Left + 1;
        let rows = screen_buffer_info.srWindow.Bottom - screen_buffer_info.srWindow.Top + 1;
        // End of Windows terminal stuff
        let buffer_size: Size = self.frame_data.get_buffer_size();
        let pixel_data = self.frame_data.get_image_mut()
            .scale(Size {width: cols as u32, height: rows as u32}, Scaling::Bilinear, buffer_size);
            //.seam_carve(false, false)
            //.get_pixel_data();
        let mut sobel_image = pixel_data.clone();
        sobel_image.gradient_magnitude(Kernel::Sobel)
        .mask_ontop(pixel_data, Color::Rgb{r: 0, g: 0, b: 0}, 50)
        .get_pixel_data();
        
        let pixel_data = sobel_image.get_pixel_data();
        let mut colored_string = String::with_capacity(pixel_data.len());


        //execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        for (index, pixel) in pixel_data.iter().enumerate() {
            let pixel = pixel.clone();
            // From color to cross color
            
            //if self.last_buffer.len() > 0 && self.last_buffer.len() == pixel_data.len() {
            //    let last_pixel = self.last_buffer[index].clone();
            //    if last_pixel == pixel {
            //        continue;
            //    }
            //}
            // If almost black no render
            let (r, g, b) = pixel.get_color_raw();

            if r < 10 && g < 10 && b < 10 {
                colored_string.push_str(&format!("{}", " "));

                continue;
            }
            //execute!(stdout, 
            //    cursor::MoveTo(x as u16, y as u16),
            //    SetForegroundColor(pixel),
            //    style::Print("█")
            //)?;
            
            
            //if index % width == 0 {
            //    //execute!(stdout, cursor::MoveToNextLine(1)).unwrap();
            //    stdout.flush()?;
            //}

            colored_string.push_str(&format!("{}", "█".stylize().with(pixel.get_color())));


            //colored_string.push_str(&format!("{}", "█".stylize().with(pixel.get_color())));
            // Write to stdout
            //stdout.write_all(&format!("{}", "█".stylize().with(pixel)).as_bytes())?;
        }

        execute!(stdout, 
            cursor::MoveTo(0, 0),
            crossterm::style::Print(colored_string),
            cursor::MoveTo(0, 0),
        )?;
        stdout.flush()?;
        mem::forget(stdout);
        Ok(())
        //print!("{esc}c", esc = 27 as char); // Clears terminal
    }
}