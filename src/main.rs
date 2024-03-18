mod cpu;
mod constants;
mod platform;

fn main() {
    let cpu: cpu::Cpu = cpu::Cpu::new();
    println!("Hello, world!");
    let platform: platform::Platform = platform::Platform::new("Hello".to_string(), 800, 600, 64, 32);
}
