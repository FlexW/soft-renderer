use anyhow::Result;
use glam::{IVec2, Vec3};
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

    let mut wireframe = false;
    let light_dir = Vec3::new(0.0, 0.0, -1.0);

    draw_state.set_clear_color((81, 141, 237));

    event_loop.run(move |event, _, control_flow| {
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) {
                control_flow.set_exit();
            }

            if input.key_pressed(VirtualKeyCode::F1) {
                wireframe = !wireframe;
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
                let mut triangle_coords = [IVec2::ZERO; 3];
                let mut world_coords = [Vec3::ZERO; 3];
                for j in 0..3 {
                    if wireframe {
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
                    } else {
                        let v = obj[i + j];
                        triangle_coords[j] = IVec2::new(
                            ((v.x + 1.0) * (width - 1) as f32 / 2.0) as i32,
                            ((v.y + 1.0) * (height - 1) as f32 / 2.0) as i32,
                        );
                        world_coords[j] = v;
                    }
                }
                if !wireframe {
                    let normal = (world_coords[2] - world_coords[0])
                        .cross(world_coords[1] - world_coords[0])
                        .normalize();
                    let intensity = normal.dot(light_dir);
                    if intensity > 0.0 {
                        let c = (intensity * 255.0) as u8;
                        draw_triangle(
                            triangle_coords,
                            &mut draw_state,
                            (c, c, c),
                        );
                    }
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
fn draw_triangle(pts: [IVec2; 3], draw_state: &mut DrawState, color: Color) {
    let width = std::i32::MAX;
    let height = std::i32::MAX;
    let mut bboxmin = IVec2::new(width - 1, height - 1);
    let mut bboxmax = IVec2::new(0, 0);

    let clamp = IVec2::new(width - 1, height - 1);
    for i in 0..3 {
        bboxmin.x = 0.max(bboxmin.x.min(pts[i].x));
        bboxmin.y = 0.max(bboxmin.y.min(pts[i].y));

        bboxmax.x = clamp.x.min(bboxmax.x.max(pts[i].x));
        bboxmax.y = clamp.y.min(bboxmax.y.max(pts[i].y));
    }

    for x in bboxmin.x..=bboxmax.x {
        for y in bboxmin.y..=bboxmax.y {
            let bc_screen = barycentric(pts, IVec2::new(x, y));
            if bc_screen.x < 0.0 || bc_screen.y < 0.0 || bc_screen.z < 0.0 {
                continue;
            }
            draw_state.set_pixel_rgb((x as u16, y as u16), color);
        }
    }
}

fn barycentric(pts: [IVec2; 3], point: IVec2) -> Vec3 {
    let u = Vec3::new(
        (pts[2][0] - pts[0][0]) as f32,
        (pts[1][0] - pts[0][0]) as f32,
        (pts[0][0] - point[0]) as f32,
    )
    .cross(Vec3::new(
        (pts[2][1] - pts[0][1]) as f32,
        (pts[1][1] - pts[0][1]) as f32,
        (pts[0][1] - point[1]) as f32,
    ));

    if u.z.abs() < 1.0 {
        return Vec3::new(-1.0, -1.0, -1.0);
    }

    Vec3::new(1.0 - (u.x + u.y) / u.z, u.y / u.z, u.x / u.z)
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

    /// Set the color that is used for the background
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
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
