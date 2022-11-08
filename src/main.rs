use anyhow::Result;
use glam::Vec3;
use softbuffer::GraphicsContext;
use tobj;
use winit::event::VirtualKeyCode;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

type Color = (u8, u8, u8);
type PixelPosition = (u16, u16);

fn main() -> Result<()> {
    let mut input = WinitInputHelper::new();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut graphics_context = unsafe { GraphicsContext::new(window).unwrap() };
    let mut draw_state = DrawState::new();
    draw_state.set_origin(DrawOrigin::BottomLeft);

    let obj = load_obj("assets/african_head/african_head.obj")?;

    event_loop.run(move |event, _, control_flow| {
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) {
                control_flow.set_exit();
            }

            // Draw
            let (width, height) = {
                let size = graphics_context.window().inner_size();
                (size.width, size.height)
            };
            draw_state.resize(width as u16, height as u16);
            draw_state.clear();

            assert!(obj.len() % 3 == 0);
            for i in (0..obj.len()).step_by(3) {
                for j in 0..3 {
                    let v0 = obj[i + j];
                    let v1 = obj[i + ((j + 1) % 3)];
                    let x0 = (v0.x + 1.0) * (width - 1) as f32 / 2.0;
                    let y0 = (v0.y + 1.0) * (height - 1) as f32 / 2.0;
                    let x1 = (v1.x + 1.0) * (width - 1) as f32 / 2.0;
                    let y1 = (v1.y + 1.0) * (height - 1) as f32 / 2.0;
                    draw_line(
                        &mut draw_state,
                        (x0 as u16, y0 as u16),
                        (x1 as u16, y1 as u16),
                        (255, 255, 255),
                    );
                }
            }

            graphics_context.set_buffer(
                draw_state.buffer(),
                width as u16,
                height as u16,
            );
        }
    });
}

fn draw_line(
    draw_state: &mut DrawState,
    start: PixelPosition,
    end: PixelPosition,
    color: Color,
) {
    use std::mem::swap;

    let mut x0 = start.0 as i32;
    let mut y0 = start.1 as i32;

    let mut x1 = end.0 as i32;
    let mut y1 = end.1 as i32;

    let steep = if (x0 - x1).abs() < (y0 - y1).abs() {
        swap(&mut x0, &mut y0);
        swap(&mut x1, &mut y1);
        true
    } else {
        false
    };

    if x0 > x1 {
        swap(&mut x0, &mut x1);
        swap(&mut y0, &mut y1);
    }

    let dx = x1 - x0;
    let dy = y1 - y0;
    let derror2 = dy.abs() * 2;
    let mut error2 = 0;
    let mut y = y0;
    for x in x0..x1 {
        if steep {
            draw_state.set_pixel_rgb((y as u16, x as u16), color);
        } else {
            draw_state.set_pixel_rgb((x as u16, y as u16), color);
        }
        error2 += derror2;
        if error2 > dx {
            y += if y1 > y0 { 1 } else { -1 };
            error2 -= dx * 2;
        }
    }
}

fn load_obj(file_path: &str) -> Result<Vec<Vec3>> {
    let options = tobj::LoadOptions {
        single_index: true,
        triangulate: true,
        ..Default::default()
    };

    let (models, _materials) = tobj::load_obj(file_path, &options)?;

    let mut vertices = Vec::new();
    for model in models {
        let mesh = model.mesh;
        for face_idx in 0..(mesh.indices.len() / 3) {
            let fv = 3; // Hardcode triangles
            for i in 0..fv {
                // Index
                let idx = mesh.indices[fv * face_idx + i] as usize;

                // Positions
                let vx = mesh.positions[fv * idx + 0];
                let vy = mesh.positions[fv * idx + 1];
                let vz = mesh.positions[fv * idx + 2];
                vertices.push(Vec3::new(vx, vy, vz));
            }
        }
    }

    Ok(vertices)
}

/// Holds a framebuffer for drawing
struct DrawState {
    framebuffer: Vec<u32>,
    width: usize,
    height: usize,
    clear_color: Color,
    draw_origin: DrawOrigin,
}

/// Origin for drawing operations
enum DrawOrigin {
    TopLeft,
    BottomLeft,
}

impl DrawState {
    /// Create a new DrawState
    pub fn new() -> Self {
        Self {
            framebuffer: Vec::new(),
            width: 0,
            height: 0,
            clear_color: (0, 0, 0),
            draw_origin: DrawOrigin::TopLeft,
        }
    }

    /// Set the origin for draw operations
    pub fn set_origin(&mut self, origin: DrawOrigin) {
        self.draw_origin = origin;
    }

    /// Resizes the framebuffer if the width and height does not match
    pub fn resize(&mut self, width: u16, height: u16) {
        let width = width as usize;
        let height = height as usize;

        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;

            self.framebuffer.resize(self.width * self.height, 0);
        }
    }

    /// Clears the background to the clear color
    pub fn clear(&mut self) {
        let red = self.clear_color.0 as u32;
        let green = self.clear_color.1 as u32;
        let blue = self.clear_color.2 as u32;
        let color = blue | (green << 8) | (red << 16);

        for i in 0..self.width * self.height {
            self.framebuffer[i] = color;
        }
    }

    /// Sets the pixel at the given position to the specified RGB color
    pub fn set_pixel_rgb(&mut self, pos: PixelPosition, rgb: Color) {
        let x = pos.0 as usize;
        let y = pos.1 as usize;

        let red = rgb.0 as u32;
        let green = rgb.1 as u32;
        let blue = rgb.2 as u32;

        let color = blue | (green << 8) | (red << 16);

        match self.draw_origin {
            DrawOrigin::TopLeft => {
                self.framebuffer[(y * self.width) + x] = color
            }
            DrawOrigin::BottomLeft => {
                self.framebuffer[((self.height - 1) - y) * self.width + x] =
                    color
            }
        };
    }

    /// Returns a reference to the framebuffer
    pub fn buffer(&self) -> &[u32] {
        &self.framebuffer
    }
}
