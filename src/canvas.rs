// # Stdout Raw handle
extern crate kernel32;
extern crate winapi;
use std::os::windows::io::FromRawHandle;
use std::fs::File;
use std::mem;
use std::sync::{Arc, Mutex};
use std::{time::Duration, thread, time::Instant};
use crossterm::style::{self, Color};
use winapi::um::wincon::SetConsoleOutputCP;
use crossterm::event::{Event, poll, read};
use winapi::um::winnls::CP_UTF8;

// # Terminal handling
use std::io::Write;
use crossterm::{cursor, execute, queue, style::Stylize, terminal};
use windows_capture::settings::ColorFormat;

// # FrameData
use crate::frame::{FrameData, Size};
use crate::processing::{ImageProcess, Kernel, Scaling};

#[derive(Clone)]
pub struct Canvas {
    frame_data: FrameData,
    term_size: Arc<Mutex<Size>>,
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
        terminal::disable_raw_mode().unwrap();
        let ts = crossterm::terminal::size().unwrap();
        let (cols, rows) = (ts.0 as u32, ts.1 as u32);
        
        let term_size = Arc::new(Mutex::new(Size { width: cols, height: rows }));
        let term_size_clone = Arc::clone(&term_size);

        thread::spawn(move || {
            loop {
                if poll(Duration::from_millis(500)).unwrap() {
                    match read().unwrap() {
                        Event::Resize(width, height) => {
                            let mut term_size = term_size_clone.lock().unwrap();
                            term_size.width = width as u32;
                            term_size.height = height as u32;
                        },
                        _ => {}
                    }
                }
            }
        });

        Self {
            frame_data,
            term_size,
        }
    }

    pub fn get_frame_data(&mut self) -> &mut FrameData {
        &mut self.frame_data
    }

    pub fn push_buffer(&mut self, buffer: Vec<u8>, buffer_width: u32, buffer_height: u32) {
        self.frame_data.set_raw_data(buffer, Size {width: buffer_width, height: buffer_height});
    }

    pub fn get_colored_output(character: char, color: Color) -> String {
        let (r, g, b) = match color {
            Color::Rgb{r, g, b} => (r, g, b),
            _ => (0, 0, 0),
        };
        format!("{}[38;2;{};{};{}m{}", 27 as char, r, g, b, character)
    }

    pub unsafe fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Windows terminal stuff
        SetConsoleOutputCP(CP_UTF8);
        let h = winapi::um::winbase::STD_OUTPUT_HANDLE;
        let console_handle = kernel32::GetStdHandle(h);
        let mut stdout = File::from_raw_handle(console_handle as *mut _);
        let term_size = self.term_size.lock().unwrap();
        // End of Windows terminal stuff
        let buffer_size: Size = self.frame_data.get_buffer_size();
        let pixel_data = self.frame_data.get_image_mut()
            .scale(*term_size, Scaling::Bilinear, buffer_size)
            //.seam_carve(false, false)
            .get_pixel_data();
        //let mut sobel_image = pixel_data.clone();
        //sobel_image
        //.grayscale(3)
        //.gradient_magnitude(Kernel::Prewitt);
        //.mask_ontop(pixel_data.grayscale(10 as u8).brightness(-50), Color::Rgb{r: 0, g: 0, b: 0}, 130);
        //.get_pixel_data();

        //let ascii_data = sobel_image.get_ascii("  `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@".to_string());
        
        //let pixel_data = sobel_image.get_pixel_data();
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

            //colored_string.push_str(&format!("{}", "█".stylize().with(pixel.get_color())));

            colored_string.push_str(&Self::get_colored_output(pixel.get_character(), pixel.get_color()));
            // Write to stdout
            //stdout.write_all(&format!("{}", "█".stylize().with(pixel)).as_bytes())?;
        }
        let start = Instant::now();
        execute!(stdout, 
            cursor::MoveTo(0, 0),
            style::Print(colored_string),
            //cursor::MoveTo(0, 0),
        )?;

        //write!(stdout, "{}", "\x1B[H")?;
        //write!(stdout, "{}", colored_string)?;
        
        let duration = start.elapsed();
        println!("Render processing time: {:?}", duration);
        stdout.flush()?;

        mem::forget(stdout);

        Ok(())
        //print!("{esc}c", esc = 27 as char); // Clears terminal
    }
}


//Render processing time: 459.2432ms-925.8014ms Max windows terminal
//Render processing time: 9.417ms Max windows terminal no color
//Render processing time: 517.901ms

