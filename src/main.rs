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

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 900;

struct World {
    tex_width: usize,
    tex_height: usize,
    texture: Vec<u32>,
    distances: Vec<Vec<u32>>,
    angles: Vec<Vec<u32>>,
    clock: f64,
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

fn generate_texture(width: usize, height: usize) -> Vec<u32> {
    let size = width * height;
    let mut texture = vec![0u32; size];
    for i in 0..size {
        let x = i % width as usize;
        let y = i / width as usize;
        texture[i] = ((x * 256 / width) ^ (y * 256 / height)) as u32;
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
        let tex_width = 256usize;
        let tex_height = 256usize;

        let mut distances = vec![vec![0u32; (WIDTH * 2) as usize]; (HEIGHT * 2) as usize];
        let mut angles = vec![vec![0u32; (WIDTH * 2) as usize]; (HEIGHT * 2) as usize];

        let w = WIDTH as f64;
        let h = HEIGHT as f64;
        let tw = tex_width as f64;
        let th = tex_height as f64;

        let ratio = 64.0;

        for y in 0..HEIGHT * 2 {
            for x in 0..WIDTH * 2 {
                let xf = x as f64;
                let yf = y as f64;
                let sq_sum = (xf - w) * (xf - w) + (yf - h) * (yf - h);
                let distance = (ratio * th / sq_sum.sqrt()) as u32 % tex_height as u32;
                let angle = ((0.5 * tw * (yf - h).atan2(xf - w) / PI) as i32) as u32;
                distances[y as usize][x as usize] = distance;
                angles[y as usize][x as usize] = angle;
            }
        }

        Self {
            tex_width,
            tex_height,
            texture: generate_texture(tex_width, tex_height),
            distances,
            angles,
            clock: now(),
        }
    }

    fn update(&mut self) {
        self.clock = now();
    }

    fn draw(&self, frame: &mut [u8]) {
        let shift_x = (self.tex_width as f64 * self.clock * 0.5) as u64;
        let shift_y = (self.tex_height as f64 * self.clock * 0.1) as u64;

        let look_x_dist = (WIDTH / 2) as f64 * self.clock.sin();
        let look_y_dist = (HEIGHT / 2) as f64 * (self.clock * 2.0).sin();

        let shift_look_x = (WIDTH as i32 / 2 + look_x_dist as i32) as usize;
        let shift_look_y = (HEIGHT as i32 / 2 + look_y_dist as i32) as usize;

        let threads = 20;
        let rows_per_band = (HEIGHT / threads + 1) as usize;

        let band_size = rows_per_band * WIDTH as usize * 4;
        let bands: Vec<&mut [u8]> = frame.chunks_mut(band_size).collect();

        fn render_band(
            band: &mut [u8],
            offset: usize,
            shift: (u64, u64),
            shift_look: (usize, usize),
            world: &World,
        ) {
            for (i, pixel) in band.chunks_exact_mut(4).enumerate() {
                let j = i + offset;
                let x = j % WIDTH as usize;
                let y = j / WIDTH as usize;
                let dist = world.distances[y + shift_look.1][x + shift_look.0];
                let tex_x = (dist as u64 + shift.0) % world.tex_width as u64;
                let angle = world.angles[y + shift_look.1][x + shift_look.0];
                let tex_y = (angle as u64 + shift.1) % world.tex_height as u64;
                let tex_i = tex_y as usize * world.tex_width + tex_x as usize;
                let color = world.texture[tex_i];
                let rgba = [0u8, color as u8, 0u8, 0xff];
                pixel.copy_from_slice(&rgba);
            }
        }

        crossbeam::scope(|spawner| {
            for (i, band) in bands.into_iter().enumerate() {
                let offset = i * rows_per_band * WIDTH as usize;

                spawner.spawn(move |_| {
                    render_band(
                        band,
                        offset,
                        (shift_x, shift_y),
                        (shift_look_x, shift_look_y),
                        self,
                    );
                });
            }
        })
        .unwrap();
    }
}
