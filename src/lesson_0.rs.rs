extern crate wgpu;

use wgpu::*;
use wgpu::winit::{
    EventsLoop, WindowBuilder, WindowEvent, Event, dpi::LogicalSize,
    KeyboardInput, VirtualKeyCode, ElementState };



fn main() {
    println!("Hello, world!");
    let width = 800.0;
    let height = 640.0;
    let mut events_loop = EventsLoop::new();

    let _window = WindowBuilder::new()
        .with_title("WGPU")
        .with_dimensions(LogicalSize::new(width as f64, height as f64))
        .build(&events_loop)
        .unwrap();

    let mut running = true;
    let mut resized_extent = None;
    while running {
        events_loop.poll_events(|ev| {
            match ev {
                Event::WindowEvent { event: WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            .. 
                        },
                        ..
                    },
                    ..    
                } |
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    running = false
                },               
                Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                    let dpi_factor = _window.get_hidpi_factor();
                    let size = _window
                        .get_inner_size()
                        .unwrap()
                        .to_physical(dpi_factor);
                    resized_extent = Some( wgpu::Extent3d {
                        width: size.width.round() as u32,
                        height: size.height.round() as u32,
                        depth: 1,
                    });
//                    let _sc_desc = graphics_support::create_sc_desc(&resized_extent.unwrap());
//                    let _swap_chain = device.create_swap_chain(&surface, &_sc_desc);
                },
                _ => (),
            }
        });
    };
    println!("Done!");
    

}
