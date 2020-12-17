use crate::fontset::Fontset;
use crate::rand::Rand;

pub struct Chip8 {
    ram: [u8; 4096],     // 4k memory
    vram: [u8; 64 * 32], // Graphics memory
    stack: [u16; 16],    // Stack w/ 16 levels
    sp: usize,           // Stack pointer
    v: [u8; 16],         // General purpose registers V0-VE + flag register VF (8bit)
    i: u16,              // Index register (16bit)
    pc: usize,           // Program counter
    delay_timer: u8,     // Delay Timer
    sound_timer: u8,     // Sound timer
    key: u16,            // Hex Keypad bit array
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            ram: [0; 4096],
            vram: [0; 64 * 32],
            stack: [0; 16],
            sp: 0,
            v: [0; 16],
            i: 0x0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            key: 0,
        }
    }

    pub fn initialize(&mut self) {
        // PC starts at 0x200
        self.pc = 0x200;

        // Load fontset; fontset is stored in memory location 0x50
        let fontset = Fontset::new();
        for i in 0..80 {
            self.ram[i] = fontset.data[i];
        }
    }

    pub fn load_game(&mut self, game: &str) {
        // Load game into ram starting from 0x200
        let buffer_size = 512;
        for i in 0..buffer_size {
            self.ram[i + 0x200] = 0;
        }
    }

    pub fn fetch_opcode(&mut self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | self.ram[self.pc + 1] as u16
    }

    pub fn execute(&mut self, opcode: u16) {
        // Decode opcode + execute
        match opcode & 0xF000 {
            // Opcodes starting with 0x0 are base operations
            0x0000 => match opcode & 0x00FF {
                // 00E0 => Clear screen
                0x00E0 => { },
                // 00EE => Return from subroutine
                0x00EE => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp] as usize;
                },
                // Default => print an error an exit
                _ => {
                    eprintln!("Unknown opcode [0x0000]: {:x}", opcode);
                }
            },

            // 1NNN => Jump to address NNN
            0x1000 => {
                self.pc = (opcode & 0x0FFF) as usize;
            },

            // 2NNN => Execute subroutine at address NNN
            0x2000 => {
                // Push current PC to the stack
                self.stack[self.sp] = self.pc as u16;
                self.sp += 1;
                // Set PC to the address of the subroutine
                self.pc = (opcode & 0x0FFF) as usize;
            },

            // 3XNN => Skip instruction if VX == NN
            0x3000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;

                if self.v[x] == (opcode & 0x00FF) as u8 {
                    self.pc += 2;
                    self.next();
                }
            },

            // 4XNN => Skip instruction if VX != NN
            0x4000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;

                if self.v[x] != (opcode & 0x00FF) as u8 {
                    self.pc += 2;
                    self.next();
                }
            },

            // 5XY0 => Skip instruction if VX == VY
            0x5000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;

                if self.v[x] == self.v[y] {
                    self.pc += 2;
                    self.next();
                }
            },

            // 6XNN => Move NN into VX
            0x6000 => {
                self.v[((opcode & 0x0F00) >> 8) as usize] = (opcode & 0x00FF) as u8;
                self.next();
            },

            // 7XNN => Add NN to VX, do not change carry flag
            0x7000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;

                self.v[x] = self.v[x].wrapping_add((opcode & 0x00FF) as u8);
                self.next();
            },

            // Opcodes starting with 0x8 are math operations
            0x8000 => match opcode & 0x000F {
                // 8XY0 => Move VY into VX
                0x0000 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;

                    self.v[x] = self.v[y];
                    self.next();
                },
                // 8XY1 => Set VX to VX | VY
                0x0001 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;

                    self.v[x] |= self.v[y];
                    self.next();
                },
                // 8XY2 => Set VX to VX & VY
                0x0002 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;

                    self.v[x] &= self.v[y];
                    self.next();
                },
                // 8XY3 => Set VX to VX ^ VY
                0x0003 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;

                    self.v[x] ^= self.v[y];
                    self.next();
                },
                // 8XY4 => Add VY to VX, set VF if there's a carry
                0x0004 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;

                    let (sum, carry) = self.v[x].overflowing_add(self.v[y]);

                    self.v[x] = sum;
                    self.v[0xF] = if carry { 1 } else { 0 };
                    self.next();
                },
                // 8XY5 => Subtract VY from VX, set VF if there's a borrow
                0x0005 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;

                    let (diff, borrow) = self.v[x].overflowing_sub(self.v[y]);

                    self.v[x] = diff;
                    self.v[0xF] = if borrow { 1 } else { 0 };
                    self.next();
                },
                // 8XY6 => Shift VX right, store least significant bit of VX in VF
                0x0006 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;

                    self.v[0xF] = self.v[x] & 0b1;
                    self.v[x] >>= 1;
                    self.next();
                },
                // 8XY7 => Set VX to VY - VX, set VF to 0 if there's a borrow
                0x0007 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;

                    let (diff, borrow) = self.v[y].overflowing_sub(self.v[x]);

                    self.v[x] = diff;
                    self.v[0xF] = if borrow { 1 } else { 0 };
                    self.next();
                },
                // 8XYE => Shift VX left, store most significant bit of VX in VF
                0x000E => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;

                    self.v[0xF] = (self.v[x] & 0b1000_0000) >> 7;
                    self.v[x] <<= 1;
                    self.next();
                },
                // Default => print an error
                _ => {
                    eprintln!("Unknown opcode [0x8000]: {:x}", opcode);
                }
            }

            // 9XY0 => Skip instruction if VX != VY
            0x9000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;

                if self.v[x] != self.v[y] {
                    self.pc += 2;
                    self.next();
                }
            },

            // ANNN => Set i to address NNN
            0xA000 => {
                self.i = opcode & 0x0FFF;
                self.next();
            },

            // BNNN => Jump to address NNN + V0
            0xB000 => {
                let addr = opcode & 0x0FFF;
                self.pc = (addr + self.v[0x0] as u16) as usize;
            },

            // CXNN => Set VX to <random> & NN
            0xC000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;

                self.v[x] = nn.wrapping_add(Rand::random_u8().unwrap());
                self.next();
            },

            // DXYN => Draw a sprite at (VX, VY) with a height of N+1
            //         Set VF if any pixels are flipped from set to unset
            0xD000 => { },

            // Opcodes starting with 0xE are keycode operations
            0xE000 => match opcode & 0x00FF {
                // EX9E => Skip instruction if key stored in VX is pressed
                0x00E9 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;

                    if self.key & (1 << self.v[x]) != 0 {
                        self.pc += 2;
                        self.next();
                    }
                },
                // EXA1 => Skip instruction if key stored in VX isn't pressed
                0x00A1 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;

                    if self.key & (1 << self.v[x]) == 0 {
                        self.pc += 2;
                        self.next();
                    }
                },
                // Default => print an error
                _ => {
                    eprintln!("Unknown opcode [0xE000]: {:x}", opcode);
                }
            }

            // Opcodes starting with 0xF are system operations
            0xF000 => match opcode & 0x00FF {
                // FX07 => Set VX to value of delay timer
                0x0007 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;

                    self.v[x] = self.delay_timer;
                    self.next();
                },
                // FX0A => Wait for input, then store key in VX
                0x000A => {},
                // FX15 => Set delay timer to VX
                0x0015 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;

                    self.delay_timer = self.v[x];
                    self.next();
                },
                // FX18 => Set sound timer to VX
                0x0018 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;

                    self.sound_timer = self.v[x];
                    self.next();
                },
                // FX1E => Add VX to i, ignore carry
                0x001E => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;

                    self.i = self.i.wrapping_add(self.v[x] as u16);
                },
                // FX29 => Set i to location of sprite for value in VX
                0x0029 => {},
                // FX33 => Store 3-digit binary-coded decimal of VX in memory at address i..i+2
                0x0033 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let j = self.i as usize;

                    self.ram[j] = self.v[x] / 100;
                    self.ram[j + 1] = (self.v[x] / 10) % 10;
                    self.ram[j + 2] = (self.v[x] % 100) % 10;

                    self.next();
                },
                // FX55 => Dump registers 0..X to ram, starting at address i
                0x0055 => {
                    for j in 0..=((opcode & 0x0F00) >> 8) as usize {
                        self.ram[self.i as usize + j] = self.v[j];
                    }

                    self.next();
                },
                // FX65 => Fill registers 0..X with data from ram, starting at address i
                0x0065 => {
                    for j in 0..=((opcode & 0x0F00) >> 8) as usize {
                        self.v[j] = self.ram[self.i as usize + j];
                    }

                    self.next();
                },
                // Default => print an error
                _ => {
                    eprintln!("Unknown opcode [0xF000]: {:x}", opcode);
                }
            },

            // Default => print an error
            _ => {
                eprintln!("Unknown opcode: {:x}", opcode);
            }
        }

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP!");
            }
            self.sound_timer -= 1;
        }
    }

    pub fn next(&self) {
        self.pc += 2;
    }

    pub fn set_keys(&self) {
        unimplemented!();
    }
}
