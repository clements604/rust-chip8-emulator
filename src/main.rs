use std::env;
use std::time::{Duration, Instant};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use crate::constants::VIDEO_WIDTH;

mod cpu;
mod constants;
mod display;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    if args.len() != 4 {
        panic!("Usage: {} <Scale> <Delay> <ROM>\n", args[0]);
    }
    let rom_path: String = args[3].clone();

    let mut display = display::Display::new(&"Chip-8 Emulator".to_string(), 64, 32, 64, 32);

    let mut cpu = cpu::Cpu::new(); // CHIP8 CPU
    cpu.load_rom(rom_path);

    let mut quit: bool = false;

    let mut cycle_time = Instant::now();
    let mut event_pump = display.sdl_context.event_pump().unwrap();

    while ! quit {

        //std::thread::sleep(Duration::from_millis(100));

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    quit = true;
                    break;
                },
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    println!("{:?}", keycode);
                    // Reset keyboard
                    for i in 0..16 {
                        cpu.keyboard[i] = 0;
                    }
                    match keycode {
                        Keycode::Num1 => cpu.keyboard[0x1] = 1,
                        Keycode::Num2 => cpu.keyboard[0x2] = 1,
                        Keycode::Num3 => cpu.keyboard[0x3] = 1,
                        Keycode::Num4 => cpu.keyboard[0xC] = 1,
                        Keycode::Q => cpu.keyboard[0x4] = 1,
                        Keycode::W => cpu.keyboard[0x5] = 1,
                        Keycode::E => cpu.keyboard[0x6] = 1,
                        Keycode::R => cpu.keyboard[0xD] = 1,
                        Keycode::A => cpu.keyboard[0x7] = 1,
                        Keycode::S => cpu.keyboard[0x8] = 1,
                        Keycode::D => cpu.keyboard[0x9] = 1,
                        Keycode::F => cpu.keyboard[0xE] = 1,
                        Keycode::Z => cpu.keyboard[0xA] = 1,
                        Keycode::X => cpu.keyboard[0x0] = 1,
                        Keycode::C => cpu.keyboard[0xB] = 1,
                        Keycode::V => cpu.keyboard[0xF] = 1,
                        _ => {}
                    }
                },
                _ => println!("Unhandled event"),
            }
        }

        if cpu.draw_flag {
            display.redraw(&cpu.display);
            cpu.draw_flag = false;
        }

        println!("CPU {}", cpu);

        cpu.cycle();

        if cycle_time.elapsed() >= Duration::from_millis(1000 / 60) {
            if cpu.delay_timer > 0 {
                cpu.delay_timer -= 1;
            }
            if cpu.sound_timer > 0 {
                cpu.sound_timer -= 1;
            }
            cycle_time = Instant::now();
        }
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 2564));

        ::std::thread::sleep(Duration::from_micros(1000)); // Sleep for 1 millisecond
    }

}
