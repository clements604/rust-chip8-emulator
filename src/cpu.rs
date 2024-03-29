
use std::fs::File;
use std::io::prelude::*;
use std::num::Wrapping;
use chrono::{Utc, TimeZone};
use rand::prelude::*;

use crate::constants;
use crate::constants::*;

pub struct Cpu {

    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16, //  generally used to store memory addresses
    pc: u16, // program counter
    stack: [u16; 16],//16
    sp: u8, // stack pointer
    delay_timer: u8,
    sound_timer: u8,
    pub keypad: [u8; 16], //16
    pub video: [u8; 64 * 32],//64 * 32
    opcode: u16,
}

impl Cpu {
    // TODO for the constructor, check if initial values are correct for all 0s
    pub fn new() -> Self {
        // Initialize the memory with 0s
        let mut memory = [0; 4096];
        // Load the fontset into memory
        for i in 0..constants::FONTSET_SIZE {
            memory[FONTSET_START_ADDRESS as usize + i] = FONTSET[i]; // Load the fontset into memory
        }

        // Initialize the CPU
        Cpu {
            registers: [0; 16],
            memory: [0; 4096],
            index: 0,
            pc: ROM_START, // Start of ROM in memory
            stack: [0; 16],
            sp: 0, // stack pointer
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [0; 64 * 32],
            opcode: 0,
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
            println!("{}", self.memory[ROM_START as usize + i]);
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
        for x in 0..self.video.len() {
            self.video[x] = 0;
        }
    }

    /*
    *   00EE: RET
    *   Return from a subroutine.
    */
    fn op_00ee(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
    }

    /*
    *   1nnn: JP addr
    *   Jump to location nnn.
    */
    fn op_1nnn(&mut self) {
        self.pc = self.opcode;
    }

    /*
    *   2nnn - CALL addr
    *   Call subroutine at nnn.
    */
    fn op_2nnn(&mut self) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = self.opcode;
    }

    /*
    *   3xkk - SE Vx, byte
    *   Skip next instruction if Vx = kk.
    */
    fn op_3xkk(&mut self) {
        let vx: u16 = (self.opcode & 0x0F00) >> 8;
        let kk: u16 = self.opcode & 0x00FFu16;
        if vx == kk {
            self.pc += 2;
        }
    }

    /*
    *   4xkk - SNE Vx, byte
    *   Skip next instruction if Vx != kk.
    */
    fn op_4xkk(&mut self) {
        let vx: u16 = (self.opcode & 0x0F00) >> 8;
        let kk: u16 = self.opcode & 0x00FFu16;
        if vx != kk {
            self.pc += 2;
        }
    }

    /*
    *   5xy0 - SE Vx, Vy
    *   Skip next instruction if Vx = Vy.
    */
    fn op_5xy0(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let vy = (self.opcode & 0x00F0u16) >> 4;

        if vx == vy {
            self.pc += 2;
        }
    }

    /*
    *   6xkk - LD Vx, byte
    *   Set Vx = kk.
    */
    fn op_6xkk(&mut self) {
        let vx = (self.opcode & 16) >> 8;
        let kk = self.opcode & 0x00FFu16;

        self.registers[vx as usize] = kk as u8;
    }

    /*
    *   7xkk - ADD Vx, byte
    *   Set Vx = Vx + kk.
    */
    fn op_7xkk(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let kk = self.opcode & 0x00FFu16;

        self.registers[vx as usize] = (Wrapping(vx) + Wrapping(kk)).0 as u8; // TODO test wrapping add is correct
    }

    /*
    *   8xy0 - LD Vx, Vy
    *   Set Vx = Vy.
    */
    fn op_8xy0(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let vy = (self.opcode & 0x0F00) >> 4;

        self.registers[vx as usize] = self.registers[vy as usize];
    }

    /*
    *   8xy1 - OR Vx, Vy
    *   Set Vx = Vx OR Vy.
    */
    fn op_8xy1(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let vy = (self.opcode & 0x0F00) >> 4;

        self.registers[vx as usize] = self.registers[vx as usize] | self.registers[vy as usize];
    }

    /*
    *   8xy2 - AND Vx, Vy
    *   Set Vx = Vx AND Vy.
    */
    fn op_8xy2(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let vy = (self.opcode & 0x0F00) >> 4;

        self.registers[vx as usize] = self.registers[vx as usize] & self.registers[vy as usize];
    }

    /*
    *   8xy3 - XOR Vx, Vy
    *   Set Vx = Vx XOR Vy.
    */
    fn op_8xy3(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let vy = (self.opcode & 0x0F00) >> 4;

        self.registers[vx as usize] = self.registers[vx as usize] ^ self.registers[vy as usize];
    }

    /*
    *   op_8xy4
    *   Set Vx = Vx + Vy, set VF = carry.
    */
    fn op_8xy4(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let vy = (self.opcode & 0x0F00) >> 4;

        let sum = self.registers[vx as usize] as u16 + self.registers[vy as usize] as u16;

        if sum > 255 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[vx as usize] = sum as u8;
    }

    /*
    *   8xy5 - SUB Vx, Vy
    *   Set Vx = Vx - Vy, set VF = NOT borrow.
    */
    fn op_8xy5(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let vy = (self.opcode & 0x0F00) >> 4;

        if vx > vy {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[vx as usize] -= self.registers[vy as usize];
    }

    /*
    *   8xy6 - SHR Vx (shift right Vx by 1)
    *   Set Vx = Vx SHR 1.
    */
    fn op_8xy6(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let lsb = vx & 0x1;// TODO unsure if this is right way to get the LSB

        self.registers[0xF] = lsb as u8;
        self.registers[vx as usize] >>= 1;
    }

    /*
    *   8xy7 - SUBN Vx, Vy
    *   Set Vx = Vy - Vx, set VF = NOT borrow.
    */
    fn op_8xy7(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let vy = (self.opcode & 0x0F00) >> 4;

        if vy > vx {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[vy as usize] -= self.registers[vx as usize];
    }

    /*
    *   8xyE - SHL Vx {, Vy} (shift left Vx by 1)
    *   Set Vx = Vx SHL 1.
    */
    fn op_8xye(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let msb = vx & 0x16; // TODO unsure if this is right way to get the MSB

        self.registers[0xF] = msb as u8;
        self.registers[vx as usize] <<= 1;
    }

    /*
    *   9xy0 - SNE Vx, Vy
    *   Skip next instruction if Vx != Vy.
    */
    fn op_9xy0(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let vy = (self.opcode & 0x0F00) >> 4;

        if self.registers[vx as usize] != self.registers[vy as usize] {
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
        let vx = (self.opcode & 0x0F00) >> 8;
        let kk = (self.opcode & 0x00FF) as u8;

        self.registers[vx as usize] = self.rand_gen() & kk;
    }

    /*
    *   Dxyn - DRW Vx, Vy, nibble
    *   Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    */
    fn op_dxyn(&mut self) { // TODO Unsure of this implementation
        let vx = (self.opcode & 0x0F00) >> 8;
        let vy = (self.opcode & 0x0F00) >> 4;
        let height = self.opcode & 0x000F;
        let x_pos = self.registers[vx as usize] % constants::VIDEO_WIDTH;
        let y_pos = self.registers[vy as usize] % constants::VIDEO_HEIGHT;

        self.registers[0xF] = 0; // Setting collision flag

        for row in 0..height { // TODO should this start at index 0 or 1?
            let sprite_byte = self.memory[(self.index + row) as usize];
            for column in 0..8 { // TODO should this start at index 0 or 1?
                // check for collision
                if sprite_byte & (0x80 >> column) != 0 {
                    let pixel_x = (self.registers[(vx + column) as usize]) % constants::VIDEO_WIDTH;
                    let pixel_y = (self.registers[(vy + row) as usize]) % constants::VIDEO_HEIGHT;

                    if self.video[(pixel_x + (pixel_y * 64)) as usize] == 1 {
                        self.registers[0xF] = 1;
                    }

                    // XOR the pixel
                    self.video[(pixel_x + (pixel_y * 64)) as usize] ^= 1;
                }
            }
        }

    }

    /*
    *   Ex9E - SKP Vx
    *   Skip next instruction if key with the value of Vx is pressed.
    */
    fn op_ex9e(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let key = self.registers[vx as usize] as usize;
        let key_state = self.keypad[key];

        if key_state != 0 {
            self.pc += 2;
        }
    }

    /*
    *   ExA1 - SKNP Vx
    *   Skip next instruction if key with the value of Vx is not pressed.
    */
    fn op_exa1(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let key = self.registers[vx as usize] as usize;
        let key_state = self.keypad[key];

        if key_state == 0 {
            self.pc += 2;
        }
    }

    /*
    *   Fx07 - LD Vx, DT
    *   Set Vx = delay timer value.
    */
    fn op_fx07(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        self.registers[vx as usize] = self.delay_timer;
    }

    /*
    *   Fx0A - LD Vx, K
    *   Wait for a key press, store the value of the key in Vx.
    *   The easiest way to “wait” is to decrement the PC by 2 whenever a keypad value is not detected.
    *   This has the effect of running the same instruction repeatedly.
    */
    fn op_fx0a(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;

        for key in 0..self.keypad.len() {
            if self.keypad[key] != 0 {
                self.registers[vx as usize] = key as u8;
                return;
            }
        }
        self.pc -= 2; // No key press
    }

    /*
    *   Fx15 - LD DT, Vx
    *   Set delay timer = Vx.
    */
    fn op_fx15(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        self.delay_timer = self.registers[vx as usize];
    }

    /*
    *   Fx18 - LD ST, Vx
    *   Set sound timer = Vx.
    */
    fn op_fx18(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        self.sound_timer = self.registers[vx as usize];
    }

    /*
    *   Fx1E - ADD I, Vx
    *   Set I = I + Vx.
    */
    fn op_fx1e(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        self.index = self.index.wrapping_add(self.registers[vx as usize] as u16);
    }

    /*
    *   Fx29 - LD F, Vx
    *   Set I = location of sprite for digit Vx.
    */
    fn op_fx29(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let digit = self.registers[vx as usize];

        self.index = constants::FONTSET_START_ADDRESS.wrapping_add(5 * digit) as u16;
    }

    /*
    *   Fx33 - LD B, Vx
    *   Store BCD representation of Vx in memory locations I, I+1, and I+2.
    *   The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I,
    *   the tens digit at location I+1, and the ones digit at location I+2.
    *
    *   We can use the modulus operator to get the right-most digit of a number, and then do a division to remove that digit.
    *   A division by ten will either completely remove the digit (340 / 10 = 34),
    *   or result in a float which will be truncated (345 / 10 = 34.5 = 34).
    */
    fn op_fx33(&mut self) {
        let vx = (self.opcode & 0x0F00) >> 8;
        let value = self.registers[vx as usize];

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
    *   Table 0
    */
    fn table_0(&mut self) {
        
    }

    /*
    *   Table 8
    */
    fn table_8(&mut self) {
       
    }

    /*
    *   Table E
    */
    fn table_e(&mut self) {
        
    }

    /*
    *   Table F
    */
    fn table_f(&mut self) {
        
    }

    /*
    *   OP_NULL
    *   Do nothing.
    */
    fn op_null(&mut self) {}

    /*
    *   Cycle
    *   Fetch, decode, and execute an instruction.
    */
    pub fn cycle(&mut self) {
        
    }

}

type InstrPtr = fn(&mut Cpu);
