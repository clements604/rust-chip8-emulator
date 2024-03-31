# Rust Chip-8 Emulator (interpreter)

A simple Chip-8 emulator written in Rust.

![test rom](https://github.com/clements604/rust-chip8-emulator/blob/master/test_rom.png?raw=true)

## Table of Contents

- [Build](#build)
- [Usage](#usage)
- [Resources](#resources)

## Build

`cargo build` is sufficient to build the binary unoptimized.

`cargo build --release` will compile the binary optimized.

## Usage

```
./chip8 <ROM>
```

### Example

```
./chip8 test_opcode.ch8
```

### Key Bindings

The supported keys are:

- 1
- 2
- 3
- 4
- Q
- W
- E
- R
- A
- S
- D
- F
- Z
- X
- C
- V
- ESC -- Terminates application.

## Resources

- http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#ExA1

  - Chip-8 technical reference.

- https://austinmorlan.com/posts/chip8_emulator/

  - C++ Chip-8 guide.

- https://tobiasvl.github.io/blog/write-a-chip-8-emulator/

  - Chip-8 high-level emulation guide.

- http://www.emulator101.com/6502-emulator.html

  - Emulator 101.

- https://chip-8.github.io/links/

  - Awesome Chip-8.

- https://github.com/mattmikolay/chip-8/wiki/Mastering-CHIP%E2%80%908

  - Mastering Chip-8.

- https://tonisagrista.com/blog/2021/chip8-implementation/

  - A similar Rust implementation.

- https://github.com/tikijian/CRAB8/tree/master

  - A similar Rust implementation.

- https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/

  - This is for the keyboard input mapping.
