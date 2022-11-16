mod framebuffer;
mod rasterizer;
mod texture;
mod types;

pub mod prelude {
    pub use crate::framebuffer::*;
    pub use crate::rasterizer::*;
    pub use crate::texture::*;
    pub use crate::types::*;
    pub use anyhow::Result;
    pub use glam::{IVec2, Vec2, Vec3};
    pub use image;
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

    let model = load_obj("assets/african_head/african_head.obj")?;

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

            for mesh in &model.meshes {
                let vertices = &mesh.vertices;
                assert!(vertices.len() % 3 == 0);

                for i in (0..vertices.len()).step_by(3) {
                    let mut triangle_coords = [Vec3::ZERO; 3];
                    let mut tex_coords = [Vec2::ZERO; 3];
                    let mut world_coords = [Vec3::ZERO; 3];
                    for j in 0..3 {
                        if wireframe {
                            let v0 = vertices[i + j];
                            let v1 = vertices[i + ((j + 1) % 3)];

                            let p0 = v0.position;
                            let p1 = v1.position;

                            let x0 = (p0.x + 1.0) * (width - 1) as f32 / 2.0;
                            let y0 = (p0.y + 1.0) * (height - 1) as f32 / 2.0;
                            let x1 = (p1.x + 1.0) * (width - 1) as f32 / 2.0;
                            let y1 = (p1.y + 1.0) * (height - 1) as f32 / 2.0;
                            rasterizer.draw_line(
                                (x0 as u16, y0 as u16),
                                (x1 as u16, y1 as u16),
                                (255, 255, 255),
                            );
                        } else {
                            let v = vertices[i + j];
                            let p = v.position;
                            triangle_coords[j] = Vec3::new(
                                (p.x + 1.0) * (width - 1) as f32 / 2.0,
                                (p.y + 1.0) * (height - 1) as f32 / 2.0,
                                p.z,
                            );
                            tex_coords[j] = v.tex_coord;
                            world_coords[j] = v.position;
                        }
                    }
                    if !wireframe {
                        let normal = (world_coords[2] - world_coords[0])
                            .cross(world_coords[1] - world_coords[0])
                            .normalize();
                        let intensity = normal.dot(light_dir);
                        if intensity > 0.0 {
                            let c = (intensity * 255.0) as u8;
                            rasterizer.draw_triangle(
                                triangle_coords,
                                tex_coords,
                                mesh.material.diffuse_texture.as_ref().unwrap(),
                            );
                        } else {
                            rasterizer.draw_triangle(
                                triangle_coords,
                                tex_coords,
                                mesh.material.diffuse_texture.as_ref().unwrap(),
                            );
                        }
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

struct Model {
    meshes: Vec<Mesh>,
}

impl Model {
    pub fn new(meshes: Vec<Mesh>) -> Self {
        Self { meshes }
    }
}

struct Mesh {
    vertices: Vec<Vertex>,
    material: Material,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, material: Material) -> Self {
        Self { vertices, material }
    }
}

struct Material {
    diffuse_color: Color,
    diffuse_texture: Option<Texture>,
}

impl Material {
    pub fn new(diffuse_color: Color, diffuse_texture: Option<Texture>) -> Self {
        Self {
            diffuse_color,
            diffuse_texture,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            diffuse_color: (120, 120, 120),
            diffuse_texture: None,
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
struct Vertex {
    position: Vec3,
    tex_coord: Vec2,
}
impl Vertex {
    pub fn new(position: Vec3, tex_coord: Vec2) -> Self {
        Self {
            position,
            tex_coord,
        }
    }
}

fn load_obj(file_path: &str) -> Result<Model> {
    let options = tobj::LoadOptions {
        single_index: true,
        triangulate: true,
        ..Default::default()
    };

    let (models, materials) = tobj::load_obj(file_path, &options)?;

    let mut meshes = Vec::new();
    for model in models {
        let mesh = model.mesh;

        let material = if let Some(material_id) = mesh.material_id {
            let material = &materials.as_ref().unwrap()[material_id];
            let diffuse_color = (
                (material.diffuse[0] * 255.0) as u8,
                (material.diffuse[1] * 255.0) as u8,
                (material.diffuse[2] * 255.0) as u8,
            );
            let diffuse_tex_name = &material.diffuse_texture;
            let diffuse_texture = if !material.diffuse_texture.is_empty() {
                Some(Texture::from_file(diffuse_tex_name)?)
            } else {
                None
            };

            Material::new(diffuse_color, diffuse_texture)
        } else {
            // FIXME: This is only a temporary workaround beacuse there is no mtl file
            Material::new(
                (255, 255, 255),
                Some(Texture::from_file(
                    "assets/african_head/african_head_diffuse.tga",
                )?),
            )
            // Material::default()
        };

        let mut vertices = Vec::new();
        // Only triangles are supported
        assert!(mesh.indices.len() % 3 == 0);

        for face_idx in 0..(mesh.indices.len() / 3) {
            let fv = 3; // Hardcode triangles
            for i in 0..fv {
                // Index
                let idx = mesh.indices[fv * face_idx + i] as usize;

                // Positions
                let vx = mesh.positions[fv * idx + 0];
                let vy = mesh.positions[fv * idx + 1];
                let vz = mesh.positions[fv * idx + 2];

                // Tex coords
                let tu = mesh.texcoords[2 * idx + 0];
                let tv = mesh.texcoords[2 * idx + 1];

                vertices.push(Vertex::new(
                    Vec3::new(vx, vy, vz),
                    Vec2::new(tu, tv),
                ));
            }
        }

        meshes.push(Mesh::new(vertices, material))
    }

    Ok(Model::new(meshes))
}
