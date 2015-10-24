extern crate rand;

use std::io::*;
use std::fs::File;
use std::num::wrapping::OverflowingOps;
use std::thread::sleep_ms;

#[cfg(test)]
mod tests;

pub trait ByteManip {
    fn high_byte(&self) -> u8;
    fn low_byte(&self) -> u8;
    fn nibble(&self, position: u8) -> u8;

    fn x(&self) -> usize; //shorthand for the positions of X and Y register indices in Chip8 opcodes
    fn y(&self) -> usize;
    fn nnn(&self) -> u16; //shorthand for NNN memory location pattern in opcodes
}

impl ByteManip for u16 {
    fn high_byte(&self) -> u8 {
        (self >> 8) as u8
    }

    fn low_byte(&self) -> u8 {
        (self & 0x00FF) as u8
    }

    fn nibble(&self, position: u8) -> u8 {
        match position {
            1 => (self >> 12) as u8,
            2 => ((self >> 8) & 0x000F as u16) as u8,
            3 => ((self >> 4) & 0x000F as u16) as u8,
            4 => (self & 0x000F as u16) as u8,
            _ => panic!("Out of range nibble position {}", position),
        }
    }

    fn x(&self) -> usize {
        self.nibble(2) as usize
    }

    fn y(&self) -> usize {
        self.nibble(3) as usize
    }

    fn nnn(&self) -> u16 {
        self & 0x0FFF as u16
    }
}

const FONTSET: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0 /* 0 */, 0x20, 0x60, 0x20, 0x20,
                           0x70 /* 1 */, 0xF0, 0x10, 0xF0, 0x80, 0xF0 /* 2 */, 0xF0,
                           0x10, 0xF0, 0x10, 0xF0 /* 3 */, 0x90, 0x90, 0xF0, 0x10,
                           0x10 /* 4 */, 0xF0, 0x80, 0xF0, 0x10, 0xF0 /* 5 */, 0xF0,
                           0x80, 0xF0, 0x90, 0xF0 /* 6 */, 0xF0, 0x10, 0x20, 0x40,
                           0x40 /* 7 */, 0xF0, 0x90, 0xF0, 0x90, 0xF0 /* 8 */, 0xF0,
                           0x90, 0xF0, 0x10, 0xF0 /* 9 */, 0xF0, 0x90, 0xF0, 0x90,
                           0x90 /* A */, 0xE0, 0x90, 0xE0, 0x90, 0xE0 /* B */, 0xF0,
                           0x80, 0x80, 0x80, 0xF0 /* C */, 0xE0, 0x90, 0x90, 0x90,
                           0xE0 /* D */, 0xF0, 0x80, 0xF0, 0x80, 0xF0 /* E */, 0xF0,
                           0x80, 0xF0, 0x80, 0x80 /* F */];

pub struct Chip8 {
    // 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
    // 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
    // 0x200-0xFFF - Program ROM and work RAM
    memory: [u8; 4096],

    // graphics memory
    pub gfx: [bool; 2048],
    /* when true, update screen. Set by instructions 0x00E0 (clear screen) and
     * 0xDXYN (draw sprite) */
    pub draw_flag: bool, 

    // All registers GP
    // V[0xF] is a carry flag
    V: [u8; 16], // registers
    I: u16, // indexing register

    pc: u16, // program counter

    // counts down at 60Hz
    delay_timer: u8,
    sound_timer: u8,

    stack: [u16; 16], // 16 level stack
    sp: u16, // stack pointer

    // Original mapping
    // 1 2 3 C
    // 4 5 6 D
    // 7 8 9 E
    // A 0 B F
    // Map keys to
    // 1 2 3 4
    // q w e r
    // a s d f
    // z x c v
    key: [bool; 16], // keypad
}

impl Chip8 {
    // add code here
    pub fn init() -> Chip8 {
        let mut temp = Chip8 {
            memory: [0; 4096],
            gfx: [false; 2048],
            draw_flag: true,
            V: [0; 16],
            I: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 16,
            key: [false; 16],
        };
        for i in 0..80 {
            temp.memory[i] = FONTSET[i];
        }
        temp
    }

    pub fn load_program(&mut self, filename: String) {
        let mut f = File::open(filename).unwrap();
        let mut s = Vec::new();
        f.read_to_end(&mut s).unwrap();
        for (index, byte) in s.into_iter().enumerate() {
            self.memory[0x200 + index] = byte;
        }
    }

    #[allow(dead_code)]
    pub fn reginfo(&self) {
        print!("PC = {:04X}\nINSTRUCTION = {:04X}\nI = {:04X}\nSP = {}\n",
               self.pc,
               (self.memory[self.pc as usize] as u16) << 8 |
               self.memory[(self.pc + 1) as usize] as u16,
               self.I,
               self.sp);
        for i in 0..16 {
            print!("V[0x{:02X}] = 0x{:02X}{}",
                   i,
                   self.V[i],
                   if (i + 1) % 4 == 0 {
                       "\n"
                   } else {
                       "\t"
                   });
        }
        print!("\n");
    }

    #[allow(dead_code)]
    pub fn dumpgfx(&self) {
        println!("▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁");
        for y in 0..32 {
            print!("|");
            for x in 0..64 {
                print!("{}",
                       if self.gfx[x + (y * 64)] {
                           "█"
                       } else {
                           "░"
                       });
            }
            println!("|");
        }
        println!("▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔");
    }

    pub fn step(&mut self) {
        // fetch
        let op = (self.memory[self.pc as usize] as u16) << 8 |
                 self.memory[(self.pc + 1) as usize] as u16;

        // decode and execute
        match op & 0xF000 {
            0x0000 => {
                match op {
                    0x00E0 => {
                        // clear the screen
                        self.gfx = [false; 2048];
                        self.draw_flag = true;
                        self.pc += 2;
                    }
                    0x00EE => {
                        // return from function
                        self.pc = self.stack[self.sp as usize] + 2;
                        self.sp += 1;
                    }
                    _ => self.unknown_opcode_panic(),
                }
            }
            0x1000 => {
                // jump to address NNN
                self.pc = op.nnn();
            }
            0x2000 => {
                // call function at NNN
                self.sp -= 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = op.nnn();
            }
            0x3000 => {
                // skip instruction if VX == NN
                if self.V[op.x()] == op.low_byte() {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x4000 => {
                // skip instruction if VX != NN
                if self.V[op.x()] != op.low_byte() {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x5000 => {
                // skip instruction if VX == VY
                if self.V[op.x()] == self.V[op.y()] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x6000 => {
                // set VX to NN
                self.V[op.x()] = op.low_byte();
                self.pc += 2;
            }
            0x7000 => {
                // adds NN to VX
                let reg = op.x();
                let (x, carry) = self.V[reg].overflowing_add(op.low_byte());
                self.V[reg] = x;
                self.V[15] = if carry {
                    1
                } else {
                    0
                };
                self.pc += 2;
            }
            0x8000 => {
                // 0x8000 series opcodes
                match op & 0x000F {
                    0x0000 => {
                        // sets VX to VY
                        self.V[op.x()] = self.V[op.y()];
                    }
                    0x0001 => {
                        // sets VX to VX | VY
                        self.V[op.x()] = self.V[op.x()] | self.V[op.y()];
                    }
                    0x0002 => {
                        // sets VX to VX & VY
                        self.V[op.x()] = self.V[op.x()] & self.V[op.y()];
                    }
                    0x0003 => {
                        // sets VX to VX ^ VY
                        self.V[op.x()] = self.V[op.x()] ^ self.V[op.y()];
                    }
                    0x0004 => {
                        // sets VX to VX + VY. VF set if carry
                        let (x, carry) = self.V[op.x()].overflowing_add(self.V[op.y()]);
                        self.V[op.x()] = x;
                        self.V[15] = if carry {
                            1
                        } else {
                            0
                        };
                    }
                    0x0005 => {
                        // sets VX to VX - VY. VF set if no borrow
                        let (x, borrow) = self.V[op.x()].overflowing_sub(self.V[op.y()]);
                        self.V[op.x()] = x;
                        self.V[15] = if borrow {
                            0
                        } else {
                            1
                        };
                    }
                    0x0006 => {
                        // sets VX to VX >> 1. VF set to least significant bit before shift
                        self.V[15] = self.V[op.x()] & 0x000F;
                        self.V[op.x()] = self.V[op.x()].wrapping_shr(1);
                    }
                    0x0007 => {
                        // sets VX to VY - VX. VF set if no borrow
                        let (x, borrow) = self.V[op.y()].overflowing_sub(self.V[op.x()]);
                        self.V[op.x()] = x;
                        self.V[15] = if borrow {
                            0
                        } else {
                            1
                        };
                    }
                    0x000E => {
                        // sets VX to VX << 1. VF set to value of most significant bit before shift
                        self.V[15] = (self.V[op.x()] >> 4) as u8;
                        self.V[op.x()] = self.V[op.x()].wrapping_shl(1);
                    }
                    _ => self.unknown_opcode_panic(),
                }
                self.pc += 2; //all 0x8000 series opcodes are two bytes
            }
            0x9000 => {
                // skips next instruction if VX doesn't equal VY
                if self.V[op.x()] != self.V[op.y()] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0xA000 => {
                // sets I to address NNN
                self.I = op.nnn();
                self.pc += 2;
            }
            0xB000 => {
                // jumps to address NNN + V0
                self.pc = op.nnn() + self.V[0] as u16;
            }
            0xC000 => {
                // sets VX to result of bitwise AND on a random number and NN
                self.V[op.x()] = op.low_byte() & rand::random::<u8>();
                self.pc += 2;
            }
            0xD000 => {
                // XOR-draws sprite at memory location I
                // Sprites are 8 bits wide. Wraps around the screen. If drawing clears a pixel,
                // VF is set to TRUE.
                // Draws at position VX, VY, N rows high
                let x = self.V[op.x()] as u32;
                let y = self.V[op.y()] as u32;
                let h = (op & 0x000F) as u32;
                let mut p: u8;
                let mut flag = false;

                self.V[15] = 0;
                for yline in 0..h {
                    p = self.memory[(self.I as u32 + yline) as usize];
                    for xline in 0..8 {
                        let pos = ((x + xline + ((y + yline) * 64)) % 2048) as usize;
                        if (p & (0x80 >> xline)) != 0 {
                            if !flag && self.gfx[pos] {
                                self.V[15] = 1;
                                flag = true;
                            }
                            self.gfx[pos] ^= true;

                        }
                    }
                }
                self.draw_flag = true;
                self.pc += 2;
            }
            0xE000 => {
                // 0xE000 series opcodes
                match op & 0xF0FF {
                    0xE09E => {
                        // skips next instruction if key stored in VX is pressed
                        if self.key[self.V[op.x()] as usize] {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }
                    0xE0A1 => {
                        // skips next instruction if key stores in VX is not pressed
                        if !self.key[self.V[op.x()] as usize] {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }
                    _ => self.unknown_opcode_panic(),
                }
            }
            0xF000 => {
                // 0xF000 series opcodes
                match op & 0xF0FF {
                    0xF007 => {
                        // sets VX to value of delay timer
                        self.V[op.x()] = self.delay_timer;
                        self.pc += 2;
                    }
                    0xF00A => {
                        // loop on this instruction until a key is pressed, store in VX
                        for i in 0..16 {
                            if self.key[i] {
                                self.V[op.x()] = i as u8;
                                self.pc += 2; //pc will only move on once a key has been pressed
                            }
                        }
                    }
                    0xF015 => {
                        // set delay timer to VX
                        self.delay_timer = self.V[op.x()];
                        self.pc += 2;
                    }
                    0xF018 => {
                        // set sound timer to VX
                        self.sound_timer = self.V[op.x()];
                        self.pc += 2;
                    }
                    0xF01E => {
                        // adds VX to I
                        let (x, carry) = self.I.overflowing_add(self.V[op.x()] as u16);
                        self.I = x;
                        self.V[15] = if carry {
                            1
                        } else {
                            0
                        };
                        self.pc += 2;
                    }
                    0xF029 => {
                        // sets I to the location of the sprite for the character in VX
                        self.I = self.V[op.x()] as u16 * 0x5;
                        self.pc += 2;
                    }
                    0xF033 => {
                        // create decimal representation of VX, place hundreds at memory location
                        // I, tens at I+1, and ones at I+2
                        let d = self.V[op.x()];
                        self.memory[self.I as usize] = d / 100;
                        self.memory[(self.I + 1) as usize] = (d / 10) % 10;
                        self.memory[(self.I + 2) as usize] = (d % 100) % 10;
                        self.pc += 2;
                    }
                    0xF055 => {
                        // stores V0 to VX in memory starting at addresss I
                        for i in 0..op.x() + 1 {
                            self.memory[self.I as usize + i] = self.V[i];
                        }
                        self.I += op.x() as u16 + 1; //happens on the original emulator, on current ones supposedly unchanged
                        self.pc += 2;
                    }
                    0xF065 => {
                        // fills V0 to VX with values from memory starting at address I
                        for i in 0..op.x() + 1 {
                            self.V[i] = self.memory[self.I as usize + i];
                        }
                        self.I += op.x() as u16 + 1; //happens on the original emulator, on current ones supposedly unchanged
                        self.pc += 2;
                    }
                    _ => self.unknown_opcode_panic(),
                }
            }
            _ => self.unknown_opcode_panic(),
        }
    }

    pub fn tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn update_keys(&mut self, key: u8, pressed: bool) {
        self.key[key as usize] = pressed;
    }

    fn unknown_opcode_panic(&mut self) {
        println!("!!PANIC!!\n!!UNKNOWN OPCODE {}!!",
                 ((self.memory[self.pc as usize] as u16) << 8) +
                 self.memory[self.pc as usize + 1] as u16);
        self.reginfo();
        self.dumpgfx();
        println!("Dumping memory in 3 \
                  seconds...\
                  \n-----------------------------------------------------------------------------\
                  ---");
        sleep_ms(3000);
        for i in (0..4096).step_by(2) {
            let op = ((self.memory[i] as u16) << 8) + self.memory[i + 1] as u16;
            println!("{:04X}: {:04X}", i, op);
        }
        panic!("");
    }
 }
