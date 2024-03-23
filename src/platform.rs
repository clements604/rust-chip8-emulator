extern crate sdl2;

use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::video::{Window, WindowContext};
use sdl2::render::{Canvas, TextureCreator};

pub struct Platform {
    window: Window,
    renderer: sdl2::render::Canvas<Window>,
    texture: Texture<'static>,
    texture_creator: TextureCreator<WindowContext>,
}

impl Platform {
    pub fn new(title: &String, window_width: u32, window_height: u32, texture_width: u32, texture_height: u32) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window(title, window_width, window_height)
            .position_centered()
            .build()
            .unwrap();

            let renderer: sdl2::render::Canvas<Window> = window.into_canvas()
            .accelerated()
            .build()
            .unwrap();

            let texture_creator: TextureCreator<_> = renderer.texture_creator();
            let texture = &texture_creator.create_texture_streaming(PixelFormatEnum::RGBA8888, texture_width, texture_height).unwrap();

        Platform {
            window: window,
            renderer: renderer,
            texture: texture,
            texture_creator: texture_creator,
        }
    }

}