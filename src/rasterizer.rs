use crate::prelude::*;

/// Holds a framebuffer for drawing and allows drawing lines and triangles on it.
pub struct Rasterizer {
    framebuffer: Framebuffer,
    clear_color: Color,
    depth_value: f32,
    draw_origin: DrawOrigin,
}

/// Origin for drawing operations
pub enum DrawOrigin {
    TopLeft,
    BottomLeft,
}

impl Rasterizer {
    /// Create a new Rasterizer
    pub fn new() -> Self {
        Self {
            framebuffer: Framebuffer::new(0, 0),
            clear_color: (0, 0, 0),
            depth_value: 0.0,
            draw_origin: DrawOrigin::TopLeft,
        }
    }

    /// Set the origin for draw operations
    pub fn set_origin(&mut self, origin: DrawOrigin) {
        self.draw_origin = origin;
    }

    /// Resizes the framebuffer if the width and height does not match
    pub fn resize(&mut self, width: u16, height: u16) {
        self.framebuffer.resize(width as u32, height as u32);
    }

    /// Set the color that is used for the background
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    /// Set the depth value that is used
    pub fn set_depth_value(&mut self, depth: f32) {
        self.depth_value = depth;
    }

    /// Clears the background to the clear color
    pub fn clear(&mut self) {
        self.framebuffer.set_color_rgb_all(self.clear_color);
        self.framebuffer.set_depth_all(self.depth_value);
    }

    /// Draws a line with the given color
    pub fn draw_line(
        &mut self,
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
                self.set_pixel((y as u16, x as u16), color);
            } else {
                self.set_pixel((x as u16, y as u16), color);
            }
            error2 += derror2;
            if error2 > dx {
                y += if y1 > y0 { 1 } else { -1 };
                error2 -= dx * 2;
            }
        }
    }

    /// Draw a triangle between the given points with the given color
    pub fn draw_triangle(
        &mut self,
        positions: [Vec3; 3],
        tex_coords: [Vec2; 3],
        texture: &Texture,
    ) {
        let mut bboxmin = Vec2::new(f32::MAX, f32::MAX);
        let mut bboxmax = Vec2::new(f32::MIN, f32::MIN);
        let clamp = Vec2::new(f32::MAX, f32::MAX);

        // Calculate bounding box for triangle
        for i in 0..3 {
            for j in 0..2 {
                bboxmin[j] = 0.0_f32.max(bboxmin[j].min(positions[i][j]));
                bboxmax[j] = clamp[j].min(bboxmax[j].max(positions[i][j]));
            }
        }

        // Go through every pixel in the bounding box of the triangle and calculate
        // the barycentric coordinates for the pixel. If the barycentric coordinates
        // are non negative, the pixel is on the triangle and will be drawn
        for x in bboxmin.x as u32..=bboxmax.x as u32 {
            for y in bboxmin.y as u32..=bboxmax.y as u32 {
                let bc_screen = barycentric(
                    positions[0],
                    positions[1],
                    positions[2],
                    Vec3::new(x as f32, y as f32, 0.0),
                );
                if bc_screen.x < 0.0 || bc_screen.y < 0.0 || bc_screen.z < 0.0 {
                    continue;
                }
                // Calculate the z value for the depth test
                let mut z = 0.0;
                for i in 0..3 {
                    z += positions[i][2] * bc_screen[i];
                }

                // Depth test
                let pos = (x as u16, y as u16);
                if self.framebuffer.depth(pos) < z {
                    self.framebuffer.set_depth(pos, z);

                    // Calculate the texture coordinate
                    let mut tex_coord = Vec2::ZERO;
                    for i in 0..3 {
                        tex_coord += tex_coords[i] * bc_screen[i];
                    }
                    let color = texture.color(tex_coord);

                    self.set_pixel(
                        pos,
                        color,
                        // (
                        //     (tex_coord.x * 255.0) as u8,
                        //     (tex_coord.y * 255.0) as u8,
                        //     0,
                        // ),
                    );
                }
            }
        }
    }

    /// Sets the pixel at the given position to the specified color
    pub fn set_pixel(&mut self, pos: PixelPosition, color: Color) {
        match self.draw_origin {
            DrawOrigin::TopLeft => {
                self.framebuffer.set_color_rgb(pos, color);
            }
            DrawOrigin::BottomLeft => {
                let height = self.framebuffer.height() as u16;
                let pos = (pos.0, (height - 1) - pos.1);
                self.framebuffer.set_color_rgb(pos, color);
            }
        };
    }

    /// Returns a reference to the framebuffer
    pub fn buffer(&self) -> &[u32] {
        &self.framebuffer.color_buffer()
    }
}

/// Calculates the barycentric coordinates for the given points
fn barycentric(a: Vec3, b: Vec3, c: Vec3, point: Vec3) -> Vec3 {
    let mut s = [Vec3::ZERO; 2];
    for i in (0..2).rev() {
        s[i][0] = c[i] - a[i];
        s[i][1] = b[i] - a[i];
        s[i][2] = a[i] - point[i];
    }

    let u = s[0].cross(s[1]);

    if u.z.abs() > 1e-2 {
        return Vec3::new(1.0 - (u.x + u.y) / u.z, u.y / u.z, u.x / u.z);
    }

    Vec3::new(-1.0, -1.0, -1.0)
}
