use std::env;
use std::time::{Duration, Instant};

use crate::constants::VIDEO_WIDTH;

mod cpu;
mod constants;
mod display;

/*fn main() {
    let cpu: cpu::Cpu = cpu::Cpu::new();
    println!("Hello, world!");
    let platform: platform::Platform = platform::Platform::new("Hello".to_string(), 800, 600, 64, 32);
    // sleep for 60 seconds
    std::thread::sleep(std::time::Duration::from_secs(60));
}*/

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

    while ! quit {
        //quit = platform.process_input(&mut cpu.keypad);

        if cpu.draw_flag {
            display.redraw(&cpu.display);
            cpu.draw_flag = false;
        }

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
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 2564));
    }

}
