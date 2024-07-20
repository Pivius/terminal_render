use std::process::{Command};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::{Instant};
use std::{thread};
use windows_capture::{
    capture::GraphicsCaptureApiHandler,
    encoder::{ImageEncoder},
    frame::{Frame, ImageFormat},
    graphics_capture_api::InternalCaptureControl,
    window::Window,
    settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings},
};
use term_size;
mod canvas;
use canvas::Canvas;
use glyphon::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer,
};

use wgpu::{
    CommandEncoderDescriptor, CompositeAlphaMode, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations, PresentMode,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, SurfaceConfiguration,
    TextureFormat, TextureUsages, TextureViewDescriptor,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

struct FrameData {
    buffer: Vec<u8>,
    width: usize,
    height: usize,
}

impl FrameData {
    fn new(buffer: Vec<u8>, width: usize, height: usize) -> Self {
        Self {
            buffer,
            width,
            height,
        }
    }
}

struct Capture {
    fps: usize,
    last_output: Instant,
    encoder: Option<ImageEncoder>,
    canvas: Canvas,
    tx: Sender<FrameData>,
}

impl GraphicsCaptureApiHandler for Capture {
    type Flags = Option<Sender<FrameData>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(message: Self::Flags) -> Result<Self, Self::Error> {
        let encoder = ImageEncoder::new(ImageFormat::Png, ColorFormat::Rgba8);
        let canvas = Canvas::new(1920, 1080, ColorFormat::Rgba8);
        if message.is_none() {
            return Err("No message channel provided".into());
        }
        let tx = message.clone().unwrap();

        Ok(Self {
            fps: 0,
            encoder: Some(encoder),
            last_output: Instant::now(),
            canvas,
            tx,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        let width = frame.width() as usize;
        let height = frame.height() as usize;
        let mut data = frame.buffer()?;
        let term_size = term_size::dimensions().unwrap();
        let term_width = term_size.0;
        let term_height = term_size.1;
        let buffer_data = data.as_raw_buffer().to_vec();
        let frame_data = FrameData::new(buffer_data.clone(), width, height);
        self.tx.send(frame_data).expect("Failed to send frame data");
        self.canvas.resize(term_width, term_height);
        self.canvas.push_buffer(data.as_raw_buffer(), width, height);
        self.canvas.render();
        self.fps += 1;
        // Stop the capture after 6 seconds
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
    let (frame_tx, frame_rx): (Sender<FrameData>, Receiver<FrameData>) = std::sync::mpsc::channel();
    let capture_thread = thread::spawn(move || {
        let window = Window::from_contains_name("Garry's Mod").expect("Failed to find Garry's Mod window");
        let settings = Settings::new(
            window,
            CursorCaptureSettings::Default,
            DrawBorderSettings::Default,
            ColorFormat::Rgba8,
            Some(frame_tx),
        );

        ///let mut capture = Capture::new(Some(frame_tx)).expect("Failed to create capture handler");
        Capture::start(settings).unwrap(); // Start capture
    });

    //pollster::block_on(run(frame_rx));
}


async fn run(frame_rx: Receiver<FrameData>) {
    // Set up window
    let (width, height) = (800, 600);
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_inner_size(LogicalSize::new(width as f64, height as f64))
        .with_title("renderer")
        .build(&event_loop)
        .unwrap());
    let size = window.inner_size();
    let scale_factor = window.scale_factor();

    // Set up surface
    let instance = Instance::new(InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                required_limits: Limits::downlevel_defaults(),
            },
            None,
        )
        .await
        .unwrap();

    let surface = instance.create_surface(window.clone()).expect("Create surface");
    let swapchain_format = TextureFormat::Bgra8UnormSrgb;
    let mut config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Opaque,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    // Set up text renderer
    let mut font_system = FontSystem::new();
    let mut cache = SwashCache::new();
    let mut atlas = TextAtlas::new(&device, &queue, swapchain_format);
    let mut text_renderer =
        TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);
    
    let mut cache = SwashCache::new();

    let physical_width = (width as f64 * scale_factor) as f32;
    let physical_height = (height as f64 * scale_factor) as f32;

    let mut test = 0;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut text_buffers: Vec<Buffer> = vec![];
    let mut font_size = 30.0;
    let mut line_height = 42.0;
    event_loop
        .run(move |event, target| {
            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                match event {
                    WindowEvent::Resized(size) => {
                        config.width = size.width;
                        config.height = size.height;
                        surface.configure(&device, &config);
                        window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        // Receive the buffer
                        let buffer_data = frame_rx.try_recv();
                        if buffer_data.is_ok() {
                            let buffer = buffer_data.unwrap();
                            let frame_width = buffer.width;
                            let frame_height = buffer.height;
                            let window_width = (config.width as f64 * scale_factor) as usize;
                            let window_height = (config.height as f64 * scale_factor) as usize;
                            let mut raw_buffer = buffer.buffer;
                            let (new_buffer, new_width, new_height) = Canvas::quantize_buffer(frame_width as u32, window_height as u32, &raw_buffer, line_height as u32);

                            let mut index = 0;
                            for y in 0..new_width {
                                for x in 0..new_height {
                                    if index >= new_buffer.len() {
                                        break;
                                    }
                                    println!("index: {}", index);
                                    // If text_buffers at this position is not set, create a new buffer
                                    if text_buffers.len() <= index {
                                        let mut buffer = Buffer::new(&mut font_system, Metrics::new(font_size, line_height));
                                        text_buffers.push(buffer);
                                    }
                                    
                                    let mut buffer = &mut text_buffers[index / 4];
                                    buffer.set_size(&mut font_system, font_size, line_height);
                                    buffer.set_text(&mut font_system, "█", Attrs::new().family(Family::SansSerif).color(
                                        Color::rgb(
                                            new_buffer[index],
                                            new_buffer[index + 1],
                                            new_buffer[index + 2],
                                        )
                                    ), Shaping::Advanced);

                                    index += 4;
                                }
                            }
                        }
                        //let mut buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));
                        //buffer.set_size(&mut font_system, 30.0, 42.0);
                        //buffer.set_text(&mut font_system, "1", Attrs::new().family(Family::SansSerif), Shaping::Advanced);
                        //buffer.set_text(&mut font_system, "2", Attrs::new().family(Family::SansSerif), Shaping::Advanced);
                        let text_areas = text_buffers.iter().enumerate().map(|(index, buffer)| {
                            TextArea {
                                buffer: &buffer,
                                left: 0.0,
                                top: 0.0,
                                scale: 1.0,
                                bounds: TextBounds {
                                    left: index as i32 * font_size as i32,
                                    top: index as i32 * line_height as i32,
                                    right: index as i32 * font_size as i32,
                                    bottom: index as i32 * line_height as i32,
                                },
                                default_color: Color::rgb(255, 255, 255),
                            }
                        }).collect::<Vec<_>>();
                        text_renderer
                            .prepare(
                                &device,
                                &queue,
                                &mut font_system,
                                &mut atlas,
                                Resolution {
                                    width: config.width,
                                    height: config.height,
                                },
                                text_areas,
                                &mut cache,
                            )
                            .unwrap();

                        let frame = surface.get_current_texture().unwrap();
                        let view = frame.texture.create_view(&TextureViewDescriptor::default());
                        let mut encoder = device
                            .create_command_encoder(&CommandEncoderDescriptor { label: None });
                        {
                            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                                label: None,
                                color_attachments: &[Some(RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: Operations {
                                        load: LoadOp::Clear(wgpu::Color::BLACK),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });

                            text_renderer.render(&atlas, &mut pass).unwrap();
                        }

                        queue.submit(Some(encoder.finish()));
                        frame.present();

                        atlas.trim();

                        window.request_redraw();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                }
            }
        })
        .unwrap();
}
