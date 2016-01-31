extern crate rand;

use std::io::*;
use std::fs::File;
use std::time::Duration;
use std::thread::sleep;
use std::ptr;

#[cfg(test)]
mod tests;

const FONTSET: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10,
                           0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10,
                           0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0,
                           0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
                           0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
                           0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80,
                           0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80];

const SFONTSET: [u8; 160] = [0xF0, 0xF0, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0xF0, 0xF0, 0x20,
                             0x20, 0x60, 0x60, 0x20, 0x20, 0x20, 0x20, 0x70, 0x70, 0xF0, 0xF0,
                             0x10, 0x10, 0xF0, 0xF0, 0x80, 0x80, 0xF0, 0xF0, 0xF0, 0xF0, 0x10,
                             0x10, 0xF0, 0xF0, 0x10, 0x10, 0xF0, 0xF0, 0x90, 0x90, 0x90, 0x90,
                             0xF0, 0xF0, 0x10, 0x10, 0x10, 0x10, 0xF0, 0xF0, 0x80, 0x80, 0xF0,
                             0xF0, 0x10, 0x10, 0xF0, 0xF0, 0xF0, 0xF0, 0x80, 0x80, 0xF0, 0xF0,
                             0x90, 0x90, 0xF0, 0xF0, 0xF0, 0xF0, 0x10, 0x10, 0x20, 0x20, 0x40,
                             0x40, 0x40, 0x40, 0xF0, 0xF0, 0x90, 0x90, 0xF0, 0xF0, 0x90, 0x90,
                             0xF0, 0xF0, 0xF0, 0xF0, 0x90, 0x90, 0xF0, 0xF0, 0x10, 0x10, 0xF0,
                             0xF0, 0xF0, 0xF0, 0x90, 0x90, 0xF0, 0xF0, 0x90, 0x90, 0x90, 0x90,
                             0xE0, 0xE0, 0x90, 0x90, 0xE0, 0xE0, 0x90, 0x90, 0xE0, 0xE0, 0xF0,
                             0xF0, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0xF0, 0xF0, 0xE0, 0xE0,
                             0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0xE0, 0xE0, 0xF0, 0xF0, 0x80,
                             0x80, 0xF0, 0xF0, 0x80, 0x80, 0xF0, 0xF0, 0xF0, 0xF0, 0x80, 0x80,
                             0xF0, 0xF0, 0x80, 0x80, 0x80, 0x80];

pub struct Chip8 {
    // 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
    // 0x200-0xFFF - Program ROM and work RAM
    memory: [u8; 4096],

    // graphics memory
    pub gfx: [u8; 8192],
    // when true, update screen. Set by instructions 0x00E0 (clear screen) and
    // 0xDXYN (draw sprite)
    pub draw_flag: bool,
    pub no_overdraw: bool,

    // All registers GP
    // V[0xF] is a carry flag
    V: [u8; 16], // registers
    I: u16, // indexing register

    pc: u16, // program counter

    // counts down at 60Hz
    delay_timer: u8,
    pub sound_timer: u8,

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

    // SCHIP8 extended mode
    pub extended_mode: bool,
    user_flags: [u8; 8],
}

impl Chip8 {
    pub fn init() -> Chip8 {
        let mut temp = Chip8 {
            memory: [0; 4096],
            gfx: [0; 8192],
            draw_flag: true,
            no_overdraw: false,
            V: [0; 16],
            I: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 16,
            key: [false; 16],
            extended_mode: false,
            user_flags: [0; 8],
        };
        for i in 0..240 {
            temp.memory[i] = if i < 80 {
                FONTSET[i]
            } else {
                SFONTSET[i - 80]
            }
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

    pub fn step(&mut self) {
        // fetch
        let op = (self.memory[self.pc as usize] as u16) << 8 |
                 self.memory[(self.pc + 1) as usize] as u16;
        // println!("{:04X}: 0x{:04X}", self.pc, op);

        // decode and execute
        match op & 0xF000 {
            0x0000 => {
                match op {
                    0x00C0...0x00CF => {
                        // scroll down N lines
                        let lines = (op & 0x000F) as usize;
                        let (width, _) = self.screen_dimens();
                        unsafe {
                            // copy memory lines down
                            ptr::copy(self.gfx.as_ptr(),
                                      self.gfx.as_mut_ptr().offset((width * lines) as isize),
                                      8192 - (width * lines));
                            // zero out old memory
                            ptr::write_bytes::<u8>(self.gfx.as_mut_ptr(), 0, width * lines);
                        }
                        self.draw_flag = true;
                        self.pc += 2;
                    }
                    0x00E0 => {
                        // clear the screen
                        self.gfx = [0; 8192];
                        self.draw_flag = true;
                        self.pc += 2;
                    }
                    0x00EE => {
                        // return from function
                        self.pc = self.stack[self.sp as usize] + 2;
                        self.sp += 1;
                    }
                    0x00FB => {
                        // scroll 4 pixels right
                        let (width, height) = self.screen_dimens();
                        let scroll = if self.extended_mode {
                            4
                        } else {
                            2
                        };
                        unsafe {
                            for i in (0..height).rev() {
                                ptr::copy(&self.gfx[width * i],
                                          &mut self.gfx[(width * i) + scroll],
                                          width - scroll);
                                ptr::write_bytes::<u8>(&mut self.gfx[width * i], 0, scroll)
                            }
                        }
                        self.draw_flag = true;
                        self.pc += 2;
                    }
                    0x00FC => {
                        // scroll 4 pixels left
                        let (width, height) = self.screen_dimens();
                        let scroll = if self.extended_mode {
                            4
                        } else {
                            2
                        };
                        unsafe {
                            for i in 0..height {
                                ptr::copy(&self.gfx[(width * i) + scroll],
                                          &mut self.gfx[width * i],
                                          width - scroll);
                                ptr::write_bytes::<u8>(&mut self.gfx[(width * i) + width - scroll],
                                                       0,
                                                       scroll)
                            }
                        }
                        self.draw_flag = true;
                        self.pc += 2;
                    }
                    0x00FD => {
                        // exit interpreter
                        // just hang
                    }
                    0x00FE => {
                        // disable extended screen mode
                        self.extended_mode = false;
                        self.pc += 2;
                    }
                    0x00FF => {
                        // enable extended screen mode
                        self.extended_mode = true;
                        self.pc += 2;
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
                self.V[op.x()] = self.V[op.x()].wrapping_add(op.low_byte());
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
                let (width, height) = self.screen_dimens();
                let x = self.V[op.x()] as usize;
                let y = self.V[op.y()] as usize;
                let h = (op & 0x000F) as usize;
                let mut all_overdraw = true;

                self.V[15] = 0;
                for yline in 0..(if h == 0 {
                    16
                } else {
                    h
                }) {
                    let widex = h == 0 && self.extended_mode;
                    let (p, shift) = if widex {
                        (((self.memory[self.I as usize + (yline * 2)] as u16) << 8) +
                         self.memory[self.I as usize + (yline * 2) + 1] as u16,
                         32768)
                    } else {
                        (self.memory[self.I as usize + yline] as u16, 0b1000_0000)
                    };
                    for xline in 0..(if widex {
                        16
                    } else {
                        8
                    }) {
                        let pos = ((x + xline) % width) + (((y + yline) % height) * width);
                        if (p & (shift >> xline)) != 0 {
                            if self.gfx[pos] == 255 {
                                self.V[15] = 1;
                            } else {
                                all_overdraw = false;
                            }
                            self.gfx[pos] ^= 255;

                        }
                    }
                }
                self.draw_flag = !all_overdraw || self.no_overdraw;
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
                        self.I = self.V[op.x()] as u16 * 5;
                        self.pc += 2;
                    }
                    0xF030 => {
                        // sets I to the location of the SCHIP8 sprite for the character in VX
                        self.I = (self.V[op.x()] as u16 * 10) + 80;
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
                        // Next line breaks SCStars, but I don't know what not having it breaks.
                        // self.I += op.x() as u16 + 1; //happens on the original emulator, on
                        // current ones supposedly unchanged
                        self.pc += 2;
                    }
                    0xF065 => {
                        // fills V0 to VX with values from memory starting at address I
                        for i in 0..op.x() + 1 {
                            self.V[i] = self.memory[self.I as usize + i];
                        }
                        // Next line breaks SCStars, but I don't know what not having it breaks.
                        // self.I += op.x() as u16 + 1; //happens on the original emulator, on
                        // current ones supposedly unchanged
                        self.pc += 2;
                    }
                    0xF075 => {
                        // store V0 to VX in user flags
                        for i in 0..op.x() + 1 {
                            self.user_flags[i] = self.V[i];
                        }
                        self.pc += 2;
                    }
                    0xF085 => {
                        // fill V0 to VX from user flags
                        for i in 0..op.x() + 1 {
                            self.V[i] = self.user_flags[i];
                        }
                        self.pc += 2;
                    }
                    _ => self.unknown_opcode_panic(),
                }
            }
            _ => self.unknown_opcode_panic(),
        }
    }

    #[inline]
    pub fn tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    #[inline]
    pub fn update_keys(&mut self, key: u8, pressed: bool) {
        self.key[key as usize] = pressed;
    }


    // Debugging methods

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
                       if self.gfx[x + (y * 64)] == 255 {
                           "█"
                       } else {
                           "░"
                       });
            }
            println!("|");
        }
        println!("▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔");
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
        sleep(Duration::new(3, 0));
        for i in (0..4096).step_by(2) {
            let op = ((self.memory[i] as u16) << 8) + self.memory[i + 1] as u16;
            println!("{:04X}: {:04X}", i, op);
        }
        panic!("");
    }

    // Utils
    #[inline]
    pub fn screen_dimens(&self) -> (usize, usize) {
        (if self.extended_mode {
            (128, 64)
        } else {
            (64, 32)
        })
    }
}

pub trait ByteManip {
    fn high_byte(&self) -> u8;
    fn low_byte(&self) -> u8;
    fn nibble(&self, position: u8) -> u8;

    fn x(&self) -> usize; //shorthand for the positions of X and Y register indices in Chip8 opcodes
    fn y(&self) -> usize;
    fn nnn(&self) -> u16; //shorthand for NNN memory location pattern in opcodes
}

impl ByteManip for u16 {
    #[inline]
    fn high_byte(&self) -> u8 {
        (self >> 8) as u8
    }

    #[inline]
    fn low_byte(&self) -> u8 {
        (self & 0x00FF) as u8
    }

    #[inline]
    fn nibble(&self, position: u8) -> u8 {
        match position {
            1 => (self >> 12) as u8,
            2 => ((self >> 8) & 0x000F as u16) as u8,
            3 => ((self >> 4) & 0x000F as u16) as u8,
            4 => (self & 0x000F as u16) as u8,
            _ => panic!("Out of range nibble position {}", position),
        }
    }

    #[inline]
    fn x(&self) -> usize {
        self.nibble(2) as usize
    }

    #[inline]
    fn y(&self) -> usize {
        self.nibble(3) as usize
    }

    #[inline]
    fn nnn(&self) -> u16 {
        self & 0x0FFF as u16
    }
}
