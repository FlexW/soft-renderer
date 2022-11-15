use crate::prelude::*;

/// A framebuffer holds a color and depth buffer
pub struct Framebuffer {
    color_buffer: Vec<u32>,
    depth_buffer: Vec<f32>,

    width: u32,
    height: u32,
}

impl Framebuffer {
    /// Construct a new framebuffer with the given width and height
    pub fn new(width: u32, height: u32) -> Self {
        let buffer_size = width * height;
        Self {
            color_buffer: vec![0; buffer_size as usize],
            depth_buffer: vec![0.0; buffer_size as usize],
            width,
            height,
        }
    }

    /// Resizes the framebuffer if the width and height does not match
    pub fn resize(&mut self, width: u32, height: u32) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;

            let buffer_size = (self.width * self.height) as usize;
            self.color_buffer.resize(buffer_size, 0);
            self.depth_buffer.resize(buffer_size, 0.0);
        }
    }

    /// Sets the color at the given position to the specified RGB color
    pub fn set_color_rgb(&mut self, pos: PixelPosition, color: Color) {
        let color = color_to_u32(color);
        self.set_color(pos, color);
    }

    /// Clears the color buffer to the given color
    pub fn set_color_rgb_all(&mut self, color: Color) {
        let color = color_to_u32(color);
        self.set_color_all(color);
    }

    /// Clears the color buffer to the given color
    pub fn set_color_all(&mut self, color: u32) {
        for c in &mut self.color_buffer {
            *c = color;
        }
    }

    /// Clears the depth buffer to the given value
    pub fn set_depth_all(&mut self, depth: f32) {
        for d in &mut self.depth_buffer {
            *d = depth;
        }
    }

    /// Sets the color at the given position to the specified color
    pub fn set_color(&mut self, pos: PixelPosition, color: u32) {
        let idx = self.pos_to_idx(pos);
        self.color_buffer[idx] = color;
    }

    /// Sets the depth at the given position to the specified depth
    pub fn set_depth(&mut self, pos: PixelPosition, depth: f32) {
        let idx = self.pos_to_idx(pos);
        self.depth_buffer[idx] = depth;
    }

    /// Return the depth on the given position
    pub fn depth(&self, pos: PixelPosition) -> f32 {
        self.depth_buffer[self.pos_to_idx(pos)]
    }

    /// Returns a reference to the color buffer
    pub fn color_buffer(&self) -> &[u32] {
        &self.color_buffer
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn pos_to_idx(&self, pos: PixelPosition) -> usize {
        let x = pos.0 as u32;
        let y = pos.1 as u32;
        (x + y * self.width) as usize
    }

    fn _idx_to_pos(&self, idx: usize) -> (u32, u32) {
        let idx = idx as u32;
        (idx % self.width, idx / self.width)
    }
}

fn color_to_u32(color: Color) -> u32 {
    let red = color.0 as u32;
    let green = color.1 as u32;
    let blue = color.2 as u32;
    let color = blue | (green << 8) | (red << 16);
    color
}
