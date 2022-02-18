#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::f64::consts::PI;
use std::time::SystemTime;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

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

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    world.draw(pixels.get_frame());

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

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

fn now() -> f64 {
    let now = SystemTime::now();
    let duration = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards!");
    duration.as_secs_f64()
}

impl World {
    fn new() -> Self {
        let texture_width = 256u32;
        let texture_height = 256u32;

        let mut distances = vec![vec![0u32; (WIDTH * 2) as usize]; (HEIGHT * 2) as usize];
        let mut angles = vec![vec![0u32; (WIDTH * 2) as usize]; (HEIGHT * 2) as usize];

        let w = WIDTH as f64;
        let h = HEIGHT as f64;
        let tw = texture_width as f64;
        let th = texture_height as f64;

        let ratio = 64.0;

        for y in 0..HEIGHT * 2 {
            for x in 0..WIDTH * 2 {
                let xf = x as f64;
                let yf = y as f64;
                let distance = (ratio * th / ((xf - w) * (xf - w) + (yf - h) * (yf - h)).sqrt())
                    as u32
                    % texture_height;
                let angle = ((0.5 * tw * (yf - h).atan2(xf - w) / PI) as i32) as u32;
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
            animation: now(),
        }
    }

    fn update(&mut self) {
        self.animation = now();
    }

    fn draw(&self, frame: &mut [u8]) {
        let shift_x = (self.texture_width as f64 * self.animation * 0.5) as u64;
        let shift_y = (self.texture_height as f64 * self.animation * 0.1) as u64;

        let shift_look_x =
            (WIDTH as i32 / 2 + ((WIDTH / 2) as f64 * self.animation.sin()) as i32) as usize;
        let shift_look_y = (HEIGHT as i32 / 2
            + ((HEIGHT / 2) as f64 * (self.animation * 2.0).sin()) as i32)
            as usize;

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = i % WIDTH as usize;
            let y = i / WIDTH as usize;

            let tex_x = ((self.distances[y + shift_look_y][x + shift_look_x] as u64 + shift_x)
                as u64
                % self.texture_width as u64) as usize;
            let tex_y = ((self.angles[y + shift_look_y][x + shift_look_x] as u64 + shift_y) as u64
                % self.texture_height as u64) as usize;

            let color = self.texture[tex_y][tex_x];

            let rgba = [0u8, color as u8, 0u8, 0xff];

            pixel.copy_from_slice(&rgba);
        }
    }
}
