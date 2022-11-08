use softbuffer::GraphicsContext;
use winit::event::VirtualKeyCode;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

type Color = (u8, u8, u8);
type PixelPosition = (u16, u16);

fn main() {
    let mut input = WinitInputHelper::new();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut graphics_context = unsafe { GraphicsContext::new(window).unwrap() };
    let mut draw_state = DrawState::new();

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

            draw_line(
                &mut draw_state,
                (0, 0),
                ((width - 1) as u16, (height - 1) as u16),
                (255, 255, 255),
            );

            draw_line(
                &mut draw_state,
                (0, (height - 1) as u16),
                ((width - 1) as u16, 0),
                (255, 255, 255),
            );

            draw_line(
                &mut draw_state,
                (0, (height - 1) as u16 / 2),
                ((width - 1) as u16, (height - 1) as u16 / 2),
                (255, 255, 255),
            );

            draw_line(
                &mut draw_state,
                ((width - 1) as u16 / 2, 0),
                ((width - 1) as u16 / 2, (height - 1) as u16),
                (255, 255, 255),
            );

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

/// Holds a framebuffer for drawing
struct DrawState {
    framebuffer: Vec<u32>,
    width: usize,
    height: usize,
    clear_color: Color,
}

impl DrawState {
    /// Create a new DrawState
    pub fn new() -> Self {
        Self {
            framebuffer: Vec::new(),
            width: 0,
            height: 0,
            clear_color: (0, 0, 0),
        }
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

        self.framebuffer[(y * self.width) + x] = color;
    }

    /// Returns a reference to the framebuffer
    pub fn buffer(&self) -> &[u32] {
        &self.framebuffer
    }
}
