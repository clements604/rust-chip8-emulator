use std::env;
use std::time::Instant;

use crate::constants::VIDEO_WIDTH;

mod cpu;
mod constants;
mod platform;

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

    let video_scale = args[1].parse::<u32>().unwrap();
    let cycle_delay = args[2].parse::<u32>().unwrap();
    let rom_path: String = args[3].clone();

    let mut platform = platform::Platform::new(&"Chip-8 Emulator".to_string(), 64 * video_scale, 32 * video_scale, 64, 32);

    let mut cpu = cpu::Cpu::new(); // CHIP8 CPU
    cpu.load_rom(rom_path);

    let video_pitch = (std::mem::size_of_val(&cpu.display[0]) * VIDEO_WIDTH as usize) as usize; // TODO likely incorrect

    let mut last_cycle_time =Instant::now();

    let mut quit: bool = false;

    while ! quit {
        //quit = platform.process_input(&mut cpu.keypad);

        let current_time = Instant::now();
        let dt = current_time.duration_since(last_cycle_time);

        if dt.as_millis() > cycle_delay as u128 {
            last_cycle_time = current_time;
            cpu.cycle();
            //platform.update_with_buffer(&cpu.video, video_pitch);
        }
    }

}
