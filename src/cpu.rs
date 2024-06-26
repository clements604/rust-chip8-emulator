
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use rand::prelude::*;
use log::debug;
use crate::constants;
use crate::constants::*;

pub struct Cpu {

    registers: [u8; constants::REGISTER_COUNT],
    memory: [u8; constants::MEMORY_SIZE],
    index: u16,
    pc: u16, // program counter
    stack: [u16; constants::STACK_LEVELS],
    sp: u8, // stack pointer
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub keyboard: [u8; constants::KEY_COUNT],
    pub display: [u8; constants::VIDEO_WIDTH as usize * constants::VIDEO_HEIGHT as usize],
    opcode: u16,
    pub draw_flag: bool,
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cpu {{\n\n\
            registers: {:?},\n\n\
            memory: {:?},\n\n\
            index: {},\n\n\
            pc: {},\n\n\
            stack: {:?},\n\n\
            sp: {},\n\n\
            delay_timer: {},\n\n\
            sound_timer: {},\n\n\
            keyboard: {:?},\n\n\
            display: {:?},\n\n\
            opcode: {},\n\n\
            draw_flag: {}\n\n\
        }}", self.registers,
             self.memory,
             self.index,
             self.pc,
             self.stack,
             self.sp,
             self.delay_timer,
             self.sound_timer,
             self.keyboard,
             self.display,
             self.opcode,
             self.draw_flag)
    }
}

impl Cpu {
    
    pub fn new() -> Self {
        // Initialize the memory with 0s
        let mut memory = [0; constants::MEMORY_SIZE];
        // Load the fontset into memory
        for i in 0..constants::FONTSET_SIZE {
            memory[FONTSET_START_ADDRESS as usize + i] = FONTSET[i];
        }

        // Initialize the CPU
        Cpu {
            registers: [0; constants::REGISTER_COUNT],
            memory: [0; constants::MEMORY_SIZE],
            index: 0,
            pc: ROM_START, // Start of ROM in memory
            stack: [0; constants::STACK_LEVELS],
            sp: 0, // stack pointer
            delay_timer: 0,
            sound_timer: 0,
            keyboard: [0; constants::KEY_COUNT],
            display: [0; constants::VIDEO_WIDTH as usize * constants::VIDEO_HEIGHT as usize],
            opcode: 0,
            draw_flag: false,
        }
    }

    /*
    * Load a ROM into memory
    */
    pub fn load_rom(&mut self, file_name: String) {

        let mut file = File::open(file_name).expect("ROM file not found");
        let mut buffer: Vec<u8> = Vec::new();

        // Read the file into a buffer
        file.read_to_end(&mut buffer).expect("Error reading file");

        // Load the fonts into CPU memory
        for i in 0..FONTSET_SIZE {
            self.memory[FONTSET_START_ADDRESS as usize + i] = FONTSET[i];
        }

        // Load the buffer into CPU memory
        for i in 0..buffer.len() {
            self.memory[ROM_START as usize + i] = buffer[i] as u8;
            debug!("{}", self.memory[ROM_START as usize + i]);
        }
    }

    /*
    * Generate a random u8 number
    */
    fn rand_gen(&mut self) -> u8 {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..=255)
    }

    /*
    *   00E0: CLS
    *   Clear the display.
    */
    fn op_00e0(&mut self) {
        for x in 0..self.display.len() {
            self.display[x] = 0;
        }
    }

    /*
    *   00EE: RET
    *   Return from a subroutine.
    */
    fn op_00ee(&mut self) {
        if self.sp == 0 {
            panic!("Stack underflow in op_00ee");
        }
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    /*
    *   1nnn: JP addr
    *   Jump to location nnn.
    */
    fn op_1nnn(&mut self) {
        self.pc = self.opcode & 0x0FFF;
    }

    /*
    *   2nnn - CALL addr
    *   Call subroutine at nnn.
    */
    fn op_2nnn(&mut self) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = self.opcode & 0x0FFF;
    }

    /*
    *   3xkk - SE Vx, byte
    *   Skip next instruction if Vx = kk.
    */
    fn op_3xkk(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let kk = (self.opcode & 0x00FF) as u8;
        if self.registers[vx] == kk {
            self.pc += 2;
        }
    }

    /*
    *   4xkk - SNE Vx, byte
    *   Skip next instruction if Vx != kk.
    */
    fn op_4xkk(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let kk = (self.opcode & 0x00FF) as u8;
        if self.registers[vx] != kk {
            self.pc += 2;
        }
    }

    /*
    *   5xy0 - SE Vx, Vy
    *   Skip next instruction if Vx = Vy.
    */
    fn op_5xy0(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        if self.registers[vx] == self.registers[vy] {
            self.pc += 2;
        }
    }

    /*
    *   6xkk - LD Vx, byte
    *   Set Vx = kk.
    */
    fn op_6xkk(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let kk = (self.opcode & 0x00FF) as u8;

        self.registers[vx] = kk;
    }

    /*
    *   7xkk - ADD Vx, byte
    *   Set Vx = Vx + kk.
    */
    fn op_7xkk(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let kk = (self.opcode & 0x00FF) as u8;

        self.registers[vx] = self.registers[vx].wrapping_add(kk);
    }

    /*
    *   8xy0 - LD Vx, Vy
    *   Set Vx = Vy.
    */
    fn op_8xy0(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.registers[vx] = self.registers[vy];
    }

    /*
    *   8xy1 - OR Vx, Vy
    *   Set Vx = Vx OR Vy.
    */
    fn op_8xy1(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.registers[vx] = self.registers[vx] | self.registers[vy];
    }

    /*
    *   8xy2 - AND Vx, Vy
    *   Set Vx = Vx AND Vy.
    */
    fn op_8xy2(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.registers[vx] = self.registers[vx] & self.registers[vy];
    }

    /*
    *   8xy3 - XOR Vx, Vy
    *   Set Vx = Vx XOR Vy.
    */
    fn op_8xy3(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.registers[vx] = self.registers[vx] ^ self.registers[vy];
    }

    /*
    *   op_8xy4
    *   Set Vx = Vx + Vy, set VF = carry.
    */
    fn op_8xy4(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        let (sum, overflow) = (self.registers[vx]).overflowing_add(self.registers[vy]);

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[vx] = sum;
    }

    /*
    *   8xy5 - SUB Vx, Vy
    *   Set Vx = Vx - Vy, set VF = NOT borrow.
    */
    fn op_8xy5(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.registers[0xF] = if self.registers[vx] > self.registers[vy] { 1 } else { 0 };
        self.registers[vx] = self.registers[vx].wrapping_sub(self.registers[vy]);
    }

    /*
    *   8xy6 - SHR Vx (shift right Vx by 1)
    *   Set Vx = Vx SHR 1.
    */
    fn op_8xy6(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let lsb = self.registers[vx] >> 0x1;

        if lsb == 1 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[vx] >>= 1;
    }

    /*
    *   8xy7 - SUBN Vx, Vy
    *   Set Vx = Vy - Vx, set VF = NOT borrow.
    */
    fn op_8xy7(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        let (sum, overflow) = (self.registers[vx]).overflowing_add(self.registers[vy]);

        if overflow {
            self.registers[0xF] = 0;
        } else {
            self.registers[0xF] = 1;
        }

        self.registers[vy] = sum;

    }

    /*
    *   8xyE - SHL Vx {, Vy} (shift left Vx by 1)
    *   Set Vx = Vx SHL 1.
    */
    fn op_8xye(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        self.registers[0xF] = (self.registers[vx] & 0x80) >> 7;
        self.registers[vx] <<= 1;

    }

    /*
    *   9xy0 - SNE Vx, Vy
    *   Skip next instruction if Vx != Vy.
    */
    fn op_9xy0(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        if self.registers[vx] != self.registers[vy] {
            self.pc += 2;
        }
    }

    /*
    *   Annn - LD I, addr
    *   Set I = nnn.
    */
    fn op_annn(&mut self) {
        self.index = self.opcode & 0x0FFF;
    }

    /*
    *   Bnnn - JP V0, addr
    *   Jump to location nnn + V0.
    */
    fn op_bnnn(&mut self) {
        self.pc = (self.registers[0] as u16) + (self.opcode & 0x0FFF);
    }

    /*
    *   Cxkk - RND Vx, byte
    *   Set Vx = random byte AND kk.
    */
    fn op_cxkk(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let kk = (self.opcode & 0x00FF) as u8;

        self.registers[vx] = self.rand_gen() & kk;
    }

    /*
    *   Dxyn - DRW Vx, Vy, nibble
    *   Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    *   TODO maybe incorrect
    */
    fn op_dxyn(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as usize;
        let y = ((self.opcode & 0x00F0) >> 4) as usize;
        let height = (self.opcode & 0x000F) as usize;

        // Reset collision flag
        self.registers[0xF] = 0;

        for row in 0..height {
            let sprite_byte = self.memory[(self.index + row as u16) as usize];
            let screen_y = (self.registers[y] as usize + row) % VIDEO_HEIGHT as usize;
            
            // Iterate over each pixel in the sprite byte
            for col in 0..8 {
                let screen_x = (self.registers[x] as usize + col) % VIDEO_WIDTH as usize;
                let sprite_pixel = (sprite_byte >> (7 - col)) & 0x1;

                // Check for collision
                if sprite_pixel == 1 && self.display[screen_y * VIDEO_WIDTH as usize + screen_x ] == 1 {
                    self.registers[0xF] = 1; // Set collision flag
                }
                
                // XOR the sprite pixel onto the display
                let display_index = screen_y * VIDEO_WIDTH as usize + screen_x;
                self.display[display_index] ^= sprite_pixel;
            }

        }
        self.draw_flag = true;
    }

    /*
    *   Ex9E - SKP Vx
    *   Skip next instruction if key with the value of Vx is pressed.
    */
    fn op_ex9e(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let key = self.registers[vx] as usize;
        let key_state = self.keyboard[key];

        if key_state != 0 {
            self.pc += 2;
        }

        self.keyboard[key] = 0;

    }

    /*
    *   ExA1 - SKNP Vx
    *   Skip next instruction if key with the value of Vx is not pressed.
    */
    fn op_exa1(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let key = self.registers[vx] as usize;
        let key_state = self.keyboard[key];

        if key_state == 0 {
            self.pc += 2;
        }

        self.keyboard[key] = 0;
    }

    /*
    *   Fx07 - LD Vx, DT
    *   Set Vx = delay timer value.
    */
    fn op_fx07(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        self.registers[vx] = self.delay_timer;
    }

    /*
    *   Fx0A - LD Vx, K
    *   Wait for a key press, store the value of the key in Vx.
    *   The easiest way to “wait” is to decrement the PC by 2 whenever a keypad value is not detected.
    *   This has the effect of running the same instruction repeatedly.
    */
    fn op_fx0a(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        if let Some (index) = self.keyboard.iter().position(|&key| key != 0) {
            self.registers[vx] = index as u8;
        } else {
            // No key press, decrement the PC to repeat the instruction to make instuction loop until key press
            self.pc -= 2;
        }
    }

    /*
    *   Fx15 - LD DT, Vx
    *   Set delay timer = Vx.
    */
    fn op_fx15(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        self.delay_timer = self.registers[vx];
    }

    /*
    *   Fx18 - LD ST, Vx
    *   Set sound timer = Vx.
    */
    fn op_fx18(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        self.sound_timer = self.registers[vx];
    }

    /*
    *   Fx1E - ADD I, Vx
    *   Set I = I + Vx.
    */
    fn op_fx1e(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        self.index = self.index.wrapping_add(self.registers[vx] as u16);
    }

    /*
    *   Fx29 - LD F, Vx
    *   Set I = location of sprite for digit Vx.
    */
    fn op_fx29(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let digit = self.registers[vx];

        self.index = constants::FONTSET_START_ADDRESS.wrapping_add(5 * digit) as u16;
    }

    /*
    *   Fx33 - LD B, Vx
    *   Store BCD representation of Vx in memory locations I, I+1, and I+2.
    */
    fn op_fx33(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let value = self.registers[vx];

        self.memory[self.index as usize] = value / 100;
        self.memory[(self.index + 1) as usize] = (value / 10) % 10;
        self.memory[(self.index + 2) as usize] = value % 10;
    }

    /*
    *   Fx55 - LD [I], Vx
    *   Store registers V0 through Vx in memory starting at location I.
    */
    fn op_fx55(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        for register in 0..=vx {
            self.memory[(self.index + register) as usize] = self.registers[register as usize];
        }
    }

    /*
    *   Fx65 - LD Vx, [I]
    *   Read registers V0 through Vx from memory starting at location I.
    */
    fn op_fx65(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        for register in 0..=vx {
            self.registers[register as usize] = self.memory[(self.index + register) as usize];
        }
    }

    /*
    *   Cycle
    *   Fetch, decode, and execute an instruction.
    */
    pub fn cycle(&mut self) {
        // Fetch
        self.opcode = (self.memory[self.pc as usize] as u16) << 8 | self.memory[(self.pc + 1) as usize] as u16;

        // Increment the program counter
        self.pc += 2;

        // Decode and execute.
        match self.opcode & 0xF000 {
            0x0000 => {
                match self.opcode & 0x00FF {
                    0xE0 => self.op_00e0(),
                    0xEE => self.op_00ee(),
                    _ => {
                        panic!("Unsupported 0x0 opcode: {}", (self.opcode & 0x00FF));
                    },
                }
            }
            0x1000 => self.op_1nnn(),
            0x2000 => self.op_2nnn(),
            0x3000 => self.op_3xkk(),
            0x4000 => self.op_4xkk(),
            0x5000 => self.op_5xy0(),
            0x6000 => self.op_6xkk(),
            0x7000 => self.op_7xkk(),
            0x8000 => {
                match self.opcode & 0x000F {
                    0x0 => self.op_8xy0(),
                    0x1 => self.op_8xy1(),
                    0x2 => self.op_8xy2(),
                    0x3 => self.op_8xy3(),
                    0x4 => self.op_8xy4(),
                    0x5 => self.op_8xy5(),
                    0x6 => self.op_8xy6(),
                    0x7 => self.op_8xy7(),
                    0xE => self.op_8xye(),
                    _ => {
                        panic!("Unsupported 0x8 opcode: {}", (self.opcode & 0x000F));
                    },
                }
            },
            0x9000 => self.op_9xy0(),
            0xA000 => self.op_annn(),
            0xB000 => self.op_bnnn(),
            0xC000 => self.op_cxkk(),
            0xD000 => self.op_dxyn(),
            0xE000 => {
                match self.opcode & 0x00FF {
                    0x9E => self.op_ex9e(),
                    0xA1 => self.op_exa1(),
                    _ => {
                        panic!("Unsupported 0xE opcode: {}", (self.opcode & 0x00FF));
                    },
                }
            },
            0xF000 => {
                match self.opcode & 0x00FF {
                    0x7 => self.op_fx07(),
                    0xA => self.op_fx0a(),
                    0x15 => self.op_fx15(),
                    0x18 => self.op_fx18(),
                    0x1E => self.op_fx1e(),
                    0x29 => self.op_fx29(),
                    0x33 => self.op_fx33(),
                    0x55 => self.op_fx55(),
                    0x65 => self.op_fx65(),
                    _ => {
                        panic!("Unsupported 0xF opcode: {}", (self.opcode & 0x00FF));
                    },
                }
            }
            _ => {
                panic!("Unsupported opcode: {}", (self.opcode & 0x00FF));
            },
        }

        // Decrement delay and sound timers if required.
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

    }

}
