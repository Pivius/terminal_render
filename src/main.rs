extern crate kernel32;
extern crate winapi;

use std::process::Command;
use std::time::{Instant};
use windows_capture::{
    capture::GraphicsCaptureApiHandler,
    encoder::ImageEncoder,
    frame::{Frame, ImageFormat},
    graphics_capture_api::InternalCaptureControl,
    window::Window,
    settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings},
};
pub mod pixel;
pub mod frame;
pub mod processing;
pub mod term;
mod canvas;
use canvas::Canvas;

struct Capture {
    fps: usize,
    last_output: Instant,
    canvas: Canvas,
}

impl GraphicsCaptureApiHandler for Capture {
    type Flags = Option<String>;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(message: Self::Flags) -> Result<Self, Self::Error> {
        let canvas = Canvas::new(ColorFormat::Rgba8);

        Ok(Self {
            fps: 0,
            last_output: Instant::now(),
            canvas,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        let mut data = frame.buffer()?;
        self.canvas.push_buffer(data.as_raw_buffer().to_vec(), frame.width(), frame.height());
        unsafe{self.canvas.render().unwrap()};

        self.fps += 1;

        if self.last_output.elapsed().as_secs() >= 1 {
            //println!("FPS: {}", self.fps);
            self.fps = 0;
            self.last_output = Instant::now();
        }


        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("Capture Session Closed");

        Ok(())
    }
}


fn open_gmod() {
    let handle = Command::new("C:\\Program Files (x86)\\Steam\\steamapps\\common\\GarrysMod\\hl2.exe")
        .arg("-console")
        .spawn()
        .expect("Failed to start Garry's Mod");
}

fn main() {
    let window = Window::from_contains_name("Garry's Mod").expect("Failed to find Garry's Mod window");
    let settings = Settings::new(
        window,
        CursorCaptureSettings::Default,
        DrawBorderSettings::Default,
        ColorFormat::Rgba8,
        None,
    );

    Capture::start(settings).unwrap(); // Start capture
}
