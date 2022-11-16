mod framebuffer;
mod rasterizer;
mod types;

pub mod prelude {
    pub use crate::framebuffer::*;
    pub use crate::rasterizer::*;
    pub use crate::types::*;
    pub use anyhow::Result;
    pub use glam::{IVec2, Vec2, Vec3};
}

use crate::prelude::*;

use softbuffer::GraphicsContext;
use std::time::SystemTime;
use tobj;
use winit::event::VirtualKeyCode;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

fn main() -> Result<()> {
    let mut input = WinitInputHelper::new();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut graphics_context = unsafe { GraphicsContext::new(window).unwrap() };
    let mut rasterizer = Rasterizer::new();
    rasterizer.set_origin(DrawOrigin::BottomLeft);

    let obj = load_obj("assets/african_head/african_head.obj")?;

    let mut wireframe = false;
    let mut light_dir = Vec3::new(0.0, 0.0, -1.0);

    rasterizer.set_clear_color((81, 141, 237));
    rasterizer.set_depth_value(f32::MIN);

    let start_time = SystemTime::now();

    event_loop.run(move |event, _, control_flow| {
        let current_time = SystemTime::now();
        let time_passed = current_time.duration_since(start_time).unwrap();
        let light_dir_x =
            f32::sin(time_passed.as_millis() as f32 * 0.002).abs();
        light_dir.x = light_dir_x;

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
            rasterizer.resize(width as u16, height as u16);
            rasterizer.clear();

            assert!(obj.len() % 3 == 0);
            for i in (0..obj.len()).step_by(3) {
                let mut triangle_coords = [Vec3::ZERO; 3];
                let mut world_coords = [Vec3::ZERO; 3];
                for j in 0..3 {
                    if wireframe {
                        let v0 = obj[i + j];
                        let v1 = obj[i + ((j + 1) % 3)];
                        let x0 = (v0.x + 1.0) * (width - 1) as f32 / 2.0;
                        let y0 = (v0.y + 1.0) * (height - 1) as f32 / 2.0;
                        let x1 = (v1.x + 1.0) * (width - 1) as f32 / 2.0;
                        let y1 = (v1.y + 1.0) * (height - 1) as f32 / 2.0;
                        rasterizer.draw_line(
                            (x0 as u16, y0 as u16),
                            (x1 as u16, y1 as u16),
                            (255, 255, 255),
                        );
                    } else {
                        let v = obj[i + j];
                        triangle_coords[j] = Vec3::new(
                            (v.x + 1.0) * (width - 1) as f32 / 2.0,
                            (v.y + 1.0) * (height - 1) as f32 / 2.0,
                            v.z,
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
                        rasterizer.draw_triangle(triangle_coords, (c, c, c));
                    } else {
                        rasterizer.draw_triangle(triangle_coords, (0, 0, 0));
                    }
                }
            }

            graphics_context.set_buffer(
                rasterizer.buffer(),
                width as u16,
                height as u16,
            );
        }
    });
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
