#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::f64::consts::PI;

use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use winit::window::CursorIcon::Help;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    texture_width: u32,
    texture_height: u32,
    texture: Vec<Vec<u32>>,
    distances: Vec<Vec<u32>>,
    angles: Vec<Vec<u32>>,
    animation: f64,
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("tunnel-rs")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let mut hidpi_factor = window.scale_factor();

    let mut pixels = {
        let surface = Surface::create(&window);
        let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, surface);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Adjust high DPI factor
            if let Some(factor) = input.scale_factor_changed() {
                hidpi_factor = factor;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
}


fn generate_texture(width: usize, height: usize) -> Vec<Vec<u32>> {
    let mut texture = vec![vec![0u32; width]; height];
    for y in 0..height {
        for x in 0..width {
            texture[y][x] = ((x * 256 / width) ^ (y * 256 / height)) as u32;
        }
    }
    texture
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        let texture_width = 128u32;
        let texture_height = 128u32;

        let mut distances = vec![vec![0u32; WIDTH as usize]; HEIGHT as usize];
        let mut angles = vec![vec![0u32; WIDTH as usize]; HEIGHT as usize];

        let w = WIDTH as f64;
        let h = HEIGHT as f64;
        let tw = texture_width as f64;
        let th = texture_height as f64;

        let ratio = 64.0;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {

                let xf = x as f64;
                let yf = y as f64;
                let distance = (ratio * th / ((xf - w / 2.0) * (xf - w / 2.0) + (yf - h / 2.0) * (yf - h / 2.0)).sqrt()) as u32 % texture_height;
                let angle = (0.5 * tw * (yf - h / 2.0).atan2(xf - w / 2.0) / PI) as u32;
                distances[y as usize][x as usize] = distance;
                angles[y as usize][x as usize] = angle;
            }
        }

        Self {
            texture_width,
            texture_height,
            texture: generate_texture(texture_width as usize, texture_height as usize),
            distances,
            angles,
            animation: 0.0,
        }
    }


    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        self.animation += 0.1;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        let shift_x = (self.texture_width as f64 * self.animation * 0.5) as u64;
        let shift_y = (self.texture_height as f64 * self.animation * 0.1) as u64;
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = i % WIDTH as usize;
            let y = i / WIDTH as usize;

            let tex_x = ((self.distances[y][x] as u64 + shift_x) as u32 % self.texture_width) as usize;
            let tex_y = ((self.angles[y][x] as u64 + shift_y) as u32 % self.texture_height) as usize;

            let color = self.texture[tex_y][tex_x];

            let rgba = [
                0u8,
                color as u8,
                0u8,
                0xFF
            ];

            pixel.copy_from_slice(&rgba);
        }
    }
}