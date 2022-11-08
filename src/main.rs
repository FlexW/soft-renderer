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

            let buffer = (0..((width * height) as usize))
                .map(|index| {
                    let y = index / (width as usize);
                    let x = index % (width as usize);
                    let red = x % 255;
                    let green = y % 255;
                    let blue = (x * y) % 255;

                    let color = blue | (green << 8) | (red << 16);

                    color as u32
                })
                .collect::<Vec<_>>();

            graphics_context.set_buffer(&buffer, width as u16, height as u16);
        }
    });
}
