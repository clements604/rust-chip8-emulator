extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

pub struct Platform {

    aa: u32,
    
}

impl Platform {
    pub fn new(title: String, window_width: u32, window_height: u32, texture_width: u32, texture_height: u32) -> Self { 

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        // Create a window
        let window = video_subsystem.window("SDL2 Window", window_width, window_height)
        .position_centered()
        .build()
        .unwrap();

        // Create a renderer
        let mut renderer = window.into_canvas().build().unwrap();

        // Set render draw color to white
        renderer.set_draw_color(Color::RGB(255, 255, 255));

        // Clear the window
        renderer.clear();

        // Set render draw color to black
        renderer.set_draw_color(Color::RGB(0, 0, 0));

        Platform{
            aa: 0,
        }

    }
}