
use std::fs::File;
use std::io::prelude::*;
use chrono::{Utc, TimeZone};
use rand::prelude::*;

use crate::constants;
use crate::constants::*;

pub struct Cpu {

    

    /*
    The CHIP-8 has sixteen 8-bit registers, labeled V0 to VF. Each register is able to hold any value from 0x00 to 0xFF.
    Register VF is a bit special. It’s used as a flag to hold information about the result of operations.
    */
    registers: [u8; 16],

    /*
    The CHIP-8 has 4096 bytes of memory, meaning the address space is from 0x000 to 0xFFF.
    Never write to or read from that area. Except for:

    0x050-0x0A0: Storage space for the 16 built-in characters (0 through F),
    which we will need to manually put into our memory because ROMs will be looking for those characters.

    0x200-0xFFF: Instructions from the ROM will be stored starting at 0x200,
    and anything left after the ROM’s space is free to use.
    */
    memory: [u8; 4096],
    index: u16,
    pc: u16, // program counter
    stack: [u16; 16],
    sp: u8, // stack pointer
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    video: [u32; 64 * 32],
    opcode: u16,
}

impl Cpu {
    // TODO for the constructor, check if initial values are correct for all 0s
    pub fn new() -> Self {
        let mut memory: [u8; 4096] = [0; 4096];
        
        for i in 0..constants::FONTSET_SIZE {
            memory[FONTSET_START_ADDRESS as usize + i] = FONTSET[i]; // Load the fontset into memory
        }
        Cpu {
            registers: [0; 16],
            memory: memory,
            index: 0,
            pc: ROM_START as u16, // Start of ROM in memory
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
    fn load_rom(&mut self, file_name: String) {
        let mut file = File::open(file_name).expect("ROM file not found");
        let file_size: usize = file.metadata().unwrap().len() as usize;
        let mut buffer: Vec<u8> = vec![0; file_size];

        // Read the file into a buffer
        file.read(&mut buffer).expect("Error reading file");

        // Load the buffer into CPU memory
        for i in 0..file_size {
            self.memory[ROM_START as usize + 1] = buffer[i] as u8;
        }
    }

    /*
    * Generate a random u8 number
    */
    fn rand_gen(&self) -> u8 {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..=255)
    }

    /*
    *   00E0: CLS
    *   Clear the display.
    */
    fn op_00e0(&mut self) {
        for i in 0..64 * 32 {
            self.video[i] = 0;
        }
    }

    /*
    *   00EE: RET
    *   Return from a subroutine.
    */
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize]; // TODO possible needs to add cast to usize for array index
    }

    /*
    *   1nnn: JP addr
    *   Jump to location nnn.
    */
    fn op_1nnn(&mut self) {
        self.pc = self.opcode & 0x0FFFu16;
    }

    /*
    *   2nnn - CALL addr
    *   Call subroutine at nnn.
    */
    fn op_2nnn(&mut self) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = self.opcode & 0x0FFFu16;
    }

    /*
    *   3xkk - SE Vx, byte
    *   Skip next instruction if Vx = kk.
    */
    fn op_3xkk(&mut self) {
        let v_x: u16 = (self.opcode & 0x0F00u16) >> 8;
        let byte = self.opcode & 0x00Fu16;

        if self.registers[v_x as usize] == byte as u8 { // TODO possible needs to add cast to usize for array index
            self.pc += 2;
        } 
    }

    /*
    *   4xkk - SNE Vx, byte
    *   Skip next instruction if Vx != kk.
    */
    fn op_4xkk(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8;
        let byte = self.opcode & 0x00Fu16;

        if self.registers[v_x as usize] != byte as u8 { // TODO possible needs to add cast to usize for array index
            self.pc += 2;
        } 
    }

    /*
    *   5xy0 - SE Vx, Vy
    *   Skip next instruction if Vx = Vy.
    */
    fn op_5xy0(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8;
        let v_y = (self.opcode & 0x00F0u16) >> 4;

        if self.registers[v_x as usize] == self.registers[v_y as usize] { // TODO possible needs to add cast to usize for array index
            self.pc += 2;
        }
    }

    /*
    *   6xkk - LD Vx, byte
    *   Set Vx = kk.
    */
    fn op_6xkk(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        self.registers[v_x as usize] = (self.opcode & 0x00FFu16) as u8;
    }

    /*
    *   7xkk - ADD Vx, byte
    *   Set Vx = Vx + kk.
    */
    fn op_7xkk(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        //TODO possible wrapping add?
        self.registers[v_x as usize] += (self.opcode & 0x00FFu16) as u8;
    }

    /*
    *   8xy0 - LD Vx, Vy
    *   Set Vx = Vy.
    */
    fn op_8xy0(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        let v_y = (self.opcode & 0x00F0u16) >> 4u16;

        self.registers[v_x as usize] = self.registers[v_y as usize];
    }

    /*
    *   8xy1 - OR Vx, Vy
    *   Set Vx = Vx OR Vy.
    */
    fn op_8xy1(&mut self) {
        let v_x: u16 = (self.opcode & 0x0F00u16) >> 8u16;
        let v_y = (self.opcode & 0x00F0u16) >> 4u16;
        self.registers[v_x as usize] |= self.registers[v_y as usize];
    }

    /*
    *   8xy2 - AND Vx, Vy
    *   Set Vx = Vx AND Vy.
    */
    fn op_8xy2(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        let v_y = (self.opcode & 0x00F0u16) >> 4u16;
        self.registers[v_x as usize] &= self.registers[v_y as usize];
    }

    /*
    *   8xy3 - XOR Vx, Vy
    *   Set Vx = Vx XOR Vy.
    */
    fn op_8xy3(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        let v_y = (self.opcode & 0x00F0u16) >> 4u16;
        self.registers[v_x as usize] ^= self.registers[v_y as usize];
    }

    /*
    *   op_8xy4
    *   Set Vx = Vx + Vy, set VF = carry.
    */
    fn op_8xy4(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        let v_y = (self.opcode & 0x00F0u16) >> 4u16;
        //TODO possible wrapping add?
        let sum:u16 = (self.registers[v_x as usize] + self.registers[v_y as usize]) as u16;

        if sum > 255 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[v_x as usize] = (sum & 0xFFu16) as u8;
    }

    /*
    *   8xy5 - SUB Vx, Vy
    *   Set Vx = Vx - Vy, set VF = NOT borrow.
    */
    fn op_8xy5(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        let v_y = (self.opcode & 0x00F0u16) >> 4u16;

        if self.registers[v_x as usize] > self.registers[v_y as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        // TODO possible wrapping subtract?
        self.registers[v_x as usize] -= self.registers[v_y as usize];
    }

    /*
    *   8xy6 - SHR Vx (shift right Vx by 1)
    *   Set Vx = Vx SHR 1.
    */
    fn op_8xy6(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;

        // Save LSB (least significant bit) in VF
        self.registers[0xF] = self.registers[v_x as usize] & 0x1u8;

        self.registers[v_x as usize] >>= 1;
    }

    /*
    *   8xy7 - SUBN Vx, Vy
    *   Set Vx = Vy - Vx, set VF = NOT borrow.
    */
    fn op_8xy7(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        let v_y = (self.opcode & 0x00F0u16) >> 4u16;

        if self.registers[v_y as usize] > self.registers[v_x as usize] {
            self.registers[0xF] = 1;
        }
        else {
            self.registers[0xF] = 0;
        }
        // TODO possible wrapping subtract?
        self.registers[v_x as usize] = self.registers[v_y as usize] - self.registers[v_x as usize];
    }

    /*
    *   8xyE - SHL Vx {, Vy} (shift left Vx by 1)
    *   Set Vx = Vx SHL 1.
    */
    fn op_8xye(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;

        // Save MSB (most significant bit) in VF
        self.registers[0xF] = (self.registers[v_x as usize] & 0x80u8) >> 7u8;

        self.registers[v_x as usize] <<= 1;
    }

    /*
    *   9xy0 - SNE Vx, Vy
    *   Skip next instruction if Vx != Vy.
    */
    fn op_9xy0(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        let v_y = (self.opcode & 0x00F0u16) >> 4u16;

        if self.registers[v_x as usize] != self.registers[v_y as usize] {
            self.pc += 2;
        }
    }

    /*
    *   Annn - LD I, addr
    *   Set I = nnn.
    */
    fn op_annn(&mut self) {
        self.index = self.opcode & 0x0FFFu16;
    }

    /*
    *   Bnnn - JP V0, addr
    *   Jump to location nnn + V0.
    */
    fn op_bnnn(&mut self) {
        let address: u16 = self.opcode & 0x0FFFu16;
        self.pc = (self.registers[0] as u16).wrapping_add(address);
    }

    /*
    *   Cxkk - RND Vx, byte
    *   Set Vx = random byte AND kk.
    */
    fn op_cxkk(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        let byte = self.opcode & 0x00FFu16;

        self.registers[v_x as usize] = self.rand_gen() & byte as u8; // TODO check random generation works as expected
    }

    /*
    *   Dxyn - DRW Vx, Vy, nibble
    *   Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    */
    fn op_dxyn(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        let v_y = (self.opcode & 0x00F0u16) >> 4u16;
        let height = self.opcode & 0x000Fu16;

        // Wrap the coordinates if they go out of bounds
        let x_pos = self.registers[v_x as usize] % constants::VIDEO_WIDTH;
        let y_pos = self.registers[v_y as usize] % constants::VIDEO_HEIGHT;

        self.registers[0xF] = 0;

        for row in 1..height {
            
            let sprite_byte = self.memory[(self.index + row) as usize];

            for column in 1..8 {
                let sprite_pixel = sprite_byte & (0x80u8 >> column);
                let screen_pixel = &self.video[((y_pos as usize + row as usize) * constants::VIDEO_WIDTH as usize + (x_pos as usize + column as usize)) as usize];
                //let screen_pixel: u32 = self.video[((y_pos as usize + row as usize) * constants::VIDEO_WIDTH as usize + (x_pos as usize + column as usize)) as usize]; // TODO potentially incorrect array index

                if sprite_pixel != 0 {
                    if *screen_pixel == 0xFFFFFFFF { // colision
                        self.registers[0xF] = 1;
                    }
                    // XOR screen and sprite pixels
                    &screen_pixel ^= 0xFFFFFFFF; // TODO update after I convert every array to vector.
                }
            }

        }
    }

}
