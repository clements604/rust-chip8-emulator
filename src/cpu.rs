
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
    memory: Vec<u8>,//4096
    index: u16,
    pc: u16, // program counter
    stack: Vec<u16>,//16
    sp: u8, // stack pointer
    delay_timer: u8,
    sound_timer: u8,
    keypad: Vec<u8>, //16
    video: Vec<u32>,//64 * 32
    opcode: u16,
    // TODO "table" currently is never read
    table: Vec<InstrPtr>, // Function pointer table
    table0: Vec<InstrPtr>, // 
    table8: Vec<InstrPtr>, // 
    tablee: Vec<InstrPtr>, // 
    tablef: Vec<InstrPtr>, // 
}

impl Cpu {
    // TODO for the constructor, check if initial values are correct for all 0s
    pub fn new() -> Self {
        // Initialize the memory with 0s
        let mut memory = vec![0; 4096];
        // Load the fontset into memory
        for i in 0..constants::FONTSET_SIZE {
            memory[FONTSET_START_ADDRESS as usize + i] = FONTSET[i]; // Load the fontset into memory
        }
        // Initialize the function pointer tables
        let mut table: Vec<fn(&mut Cpu)> = Vec::new();
        let mut table0: Vec<fn(&mut Cpu)> = Vec::new();
        let mut table8: Vec<fn(&mut Cpu)> = Vec::new();
        let mut tablee: Vec<fn(&mut Cpu)> = Vec::new();
        let mut tablef: Vec<fn(&mut Cpu)> = Vec::new();

        // Add the sub-tables to the main table and functions to subtables
        table.push(Cpu::table_0);
        table.push(Cpu::op_1nnn);
        table.push(Cpu::op_2nnn);
        table.push(Cpu::op_3xkk);
        table.push(Cpu::op_4xkk);
        table.push(Cpu::op_5xy0);
        table.push(Cpu::op_6xkk);
        table.push(Cpu::op_7xkk);
        table.push(Cpu::table_8);
        table.push(Cpu::op_9xy0);
        table.push(Cpu::op_annn);
        table.push(Cpu::op_bnnn);
        table.push(Cpu::op_cxkk);
        table.push(Cpu::op_dxyn);
        table.push(Cpu::table_e);
        table.push(Cpu::table_f);

        for i in 0..0xF {
            table0.push(Cpu::op_null);
            table8.push(Cpu::op_null);
            tablee.push(Cpu::op_null);
        }

        table0[0x0] = Cpu::op_00e0; // TODO overriding the above loop?
        table0[0xE] = Cpu::op_00ee; // TODO overriding the above loop?

        table8[0x0] = Cpu::op_8xy0;
        table8[0x1] = Cpu::op_8xy1;
        table8[0x2] = Cpu::op_8xy2;
        table8[0x3] = Cpu::op_8xy3;
        table8[0x4] = Cpu::op_8xy4;
        table8[0x5] = Cpu::op_8xy5;
        table8[0x6] = Cpu::op_8xy6;
        table8[0x7] = Cpu::op_8xy7;
        table8[0xE] = Cpu::op_8xye; // TODO overriding the above loop?


        tablee[0x1] = Cpu::op_exa1;
        tablee[0xE] = Cpu::op_ex9e;

        for i in 0..0x66 {
            tablef.push(Cpu::op_null);
        }

        tablef[0x07] = Cpu::op_fx07;
        tablef[0x0A] = Cpu::op_fx0a;
        tablef[0x15] = Cpu::op_fx15;
        tablef[0x18] = Cpu::op_fx18;
        tablef[0x1E] = Cpu::op_fx1e;
        tablef[0x29] = Cpu::op_fx29;
        tablef[0x33] = Cpu::op_fx33;
        tablef[0x55] = Cpu::op_fx55;
        tablef[0x65] = Cpu::op_fx65;

        // Initialize the CPU
        Cpu {
            registers: [0; 16],
            memory: memory,
            index: 0,
            pc: ROM_START, // Start of ROM in memory
            stack: vec![0; 16],
            sp: 0, // stack pointer
            delay_timer: 0,
            sound_timer: 0,
            keypad: vec![0; 16],
            video: vec![0; 64 * 32],
            opcode: 0,
            table: table,
            table0: table0,
            table8: table8,
            tablee: tablee,
            tablef: tablef,
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
    fn rand_gen(&mut self) -> u8 {
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
                let screen_pixel = &mut self.video[((y_pos as usize + row as usize) * constants::VIDEO_WIDTH as usize + (x_pos as usize + column as usize)) as usize];

                if sprite_pixel != 0 {
                    if *screen_pixel == 0xFFFFFFFF { // colision
                        self.registers[0xF] = 1;
                    }
                    // XOR screen and sprite pixels
                    *screen_pixel ^= 0xFFFFFFFF; // TODO update after I convert every array to vector.
                }
            }

        }
    }

    /*
    *   Ex9E - SKP Vx
    *   Skip next instruction if key with the value of Vx is pressed.
    */
    fn op_ex9e(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        let key = self.registers[v_x as usize];

        if self.keypad[key as usize] != 0 {
            self.pc += 2;
        }
    }

    /*
    *   ExA1 - SKNP Vx
    *   Skip next instruction if key with the value of Vx is not pressed.
    */
    fn op_exa1(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        let key = self.registers[v_x as usize];

        if self.keypad[key as usize] == 0 {
            self.pc += 2;
        }
    }

    /*
    *   Fx07 - LD Vx, DT
    *   Set Vx = delay timer value.
    */
    fn op_fx07(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        self.registers[v_x as usize] = self.delay_timer;
    }

    /*
    *   Fx0A - LD Vx, K
    *   Wait for a key press, store the value of the key in Vx.
    *   The easiest way to “wait” is to decrement the PC by 2 whenever a keypad value is not detected.
    *   This has the effect of running the same instruction repeatedly.
    */
    fn op_fx0a(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;

        for i in 0..15 {
            if self.keypad[i] != 0 {
                self.registers[v_x as usize] = i as u8;
                return;
            }
        }

        self.pc -= 2;
    }

    /*
    *   Fx15 - LD DT, Vx
    *   Set delay timer = Vx.
    */
    fn op_fx15(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        self.delay_timer = self.registers[v_x as usize];
    }

    /*
    *   Fx18 - LD ST, Vx
    *   Set sound timer = Vx.
    */
    fn op_fx18(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        self.sound_timer = self.registers[v_x as usize];
    }

    /*
    *   Fx1E - ADD I, Vx
    *   Set I = I + Vx.
    */
    fn op_fx1e(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        self.index += self.registers[v_x as usize] as u16;
    }

    /*
    *   Fx29 - LD F, Vx
    *   Set I = location of sprite for digit Vx.
    */
    fn op_fx29(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        let digit = self.registers[v_x as usize];

        self.index = FONTSET_START_ADDRESS as u16 + (5 * digit as u16);
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
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;
        let mut value = self.registers[v_x as usize];

        // Ones-place
        self.memory[self.index as usize + 2] = value % 10;
        value /= 10;
        // Tens-place
        self.memory[self.index as usize + 1] = value % 10;
        value /= 10;
        // Hundreds-place
        self.memory[self.index as usize] = value % 10;
    }

    /*
    *   Fx55 - LD [I], Vx
    *   Store registers V0 through Vx in memory starting at location I.
    */
    fn op_fx55(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;

        for i in 0..v_x {
            self.memory[self.index as usize + i as usize] = self.registers[i as usize];
        }
    }

    /*
    *   Fx65 - LD Vx, [I]
    *   Read registers V0 through Vx from memory starting at location I.
    */
    fn op_fx65(&mut self) {
        let v_x = (self.opcode & 0x0F00u16) >> 8u16;

        for i in 0..v_x {
            self.registers[i as usize] = self.memory[self.index as usize + i as usize];
        }
    }

    /*
    *   Table 0
    */
    fn table_0(&mut self) {
        self.table0[(self.opcode & 0x000Fu16) as usize];
    }

    /*
    *   Table 8
    */
    fn table_8(&mut self) {
        self.table8[(self.opcode & 0x000Fu16) as usize];
    }

    /*
    *   Table E
    */
    fn table_e(&mut self) {
        self.tablee[(self.opcode & 0x000Fu16) as usize];
    }

    /*
    *   Table F
    */
    fn table_f(&mut self) {
        self.tablef[(self.opcode & 0x000Fu16) as usize];
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
    fn cycle(&mut self) {
        // Fetch the opcode
        // TODO possibly needs to be u8, not U16
        let opcode: u16 = ((self.memory[self.pc as usize] as u16) << 8u16) | self.memory[self.pc as usize + 1] as u16;
        // Increnemnt program counter before executing anyting
        self.pc += 2;
        // Decode and execute the opcode
        (self.table[(opcode & 0xF000u16) as usize >> 12u16])(self);
        // Decrement the delat timer if it has been set
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        // Decrement the sound timer if it has been set
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

}

type InstrPtr = fn(&mut Cpu);
