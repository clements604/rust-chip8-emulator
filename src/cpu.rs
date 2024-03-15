use std::fs;
use std::io::prelude::*;

use constants::*;

struct Cpu {

    

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

    fn new() -> Self {
        let mut memory: [u8; 4096] = [0; 4096];
        for i in 0..FONTSET_SIZE {
            memory[FONTSET_START_ADDRESS + i] = FONTSET[i]; // Load the fontset into memory
        }
        Cpu {
            pc: ROM_START, // Start of ROM in memory
            memory: memory,
        }
    }

    /*
    * Load a ROM into memory
    */
    fn load_rom(&mut self, file_name: Sring) {
        let mut file = File::open(file_name).expect("ROM file not found");
        let file_size = file.metadata().unwrap().len() as usize;
        let mut buffer: [char; file_size];

        // Read the file into a buffer
        file.read(&mut buffer).expect("Error reading file");

        // Load the buffer into CPU memory
        for i in 0..file_size {
            self.memory[ROM_START + 1] = buffer[i] as u8;
        }
    }

}
