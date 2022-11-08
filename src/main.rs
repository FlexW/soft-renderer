use softbuffer::GraphicsContext;
use winit::event::VirtualKeyCode;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

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

            for index in 0..((width * height) as usize) {
                let y = index / (width as usize);
                let x = index % (width as usize);
                let red = x % 255;
                let green = y % 255;
                let blue = (x * y) % 255;
                draw_state.set_pixel_rgb(
                    (x as u16, y as u16),
                    (red as u8, green as u8, blue as u8),
                );
            }

            graphics_context.set_buffer(
                draw_state.buffer(),
                width as u16,
                height as u16,
            );
        }
    });
}

/// Holds a framebuffer for drawing
struct DrawState {
    framebuffer: Vec<u32>,
    width: u16,
    height: u16,
}

impl DrawState {
    /// Create a new DrawState
    pub fn new() -> Self {
        Self {
            framebuffer: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    /// Resizes the framebuffer if the width and height does not match
    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;

            self.framebuffer
                .resize(self.width as usize * self.height as usize, 0);
        }
    }

    /// Sets the pixel at the given position to the specified RGB color
    pub fn set_pixel_rgb(&mut self, pos: (u16, u16), rgb: (u8, u8, u8)) {
        let x = pos.0 as usize;
        let y = pos.1 as usize;

        let red = rgb.0 as u32;
        let green = rgb.1 as u32;
        let blue = rgb.2 as u32;

        let color = blue | (green << 8) | (red << 16);

        self.framebuffer[(y * self.width as usize) + x] = color;
    }

    /// Returns a reference to the framebuffer
    pub fn buffer(&self) -> &[u32] {
        &self.framebuffer
    }
}
