use rand::{rngs::ThreadRng, Rng};

pub struct Chip8 {
    opcode: u16,
    memory: [u8; 4096],
    graphics: [u8; 64 * 32],
    registers: [u8; 16],
    index: u16,
    program_counter: u16,

    delay_timer: u8,
    sound_timer: u8,

    stack: [u16; 16],
    sp: u8,

    keys: [u8; 16],
    rng: ThreadRng,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut memory: [u8; 4096] = [0; 4096];

        let fontset = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        fontset
            .iter()
            .enumerate()
            .for_each(|(i, val)| memory[i] = *val);

        Chip8 {
            rng: rand::thread_rng(),
            opcode: 0,
            memory: memory,
            graphics: [0; 64 * 32],
            registers: [0; 16],
            index: 0,
            program_counter: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            keys: [0; 16],
        }
    }

    pub fn run(&mut self) {
        self.opcode = (self.memory[self.program_counter as usize] as u16) << 8
            | (self.memory[(self.program_counter + 1) as usize] as u16);

        let x = self.opcode >> 12;

        match x {
            0x0 => {
                if self.opcode == 0x00E0 {
                    self.graphics.iter_mut().for_each(|p| *p = 0);
                } else if self.opcode == 0x00EE {
                    self.sp -= 1;
                    self.program_counter = self.stack[self.sp as usize];
                }
                self.program_counter += 2
            }
            0x1 => self.program_counter = self.opcode & 0x0FFF,
            0x2 => {
                self.stack[self.sp as usize] = self.program_counter;
                self.sp += 1;
                self.program_counter = self.opcode & 0x0FFF;
            }
            0x3 => {
                let x = (self.opcode & 0x0F00) >> 8;

                if (self.registers[x as usize] as u16) == self.opcode & 0x00FF {
                    self.program_counter += 2
                }
                self.program_counter += 2
            }

            0x4 => {
                let x = (self.opcode & 0x0F00) >> 8;

                if (self.registers[x as usize] as u16) != self.opcode & 0x00FF {
                    self.program_counter += 2
                }
                self.program_counter += 2
            }

            0x5 => {
                let x = (self.opcode & 0x0F00) >> 8;
                let y = (self.opcode & 0x00F0) >> 4;

                if self.registers[x as usize] == self.registers[y as usize] {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }

            0x6 => {
                let x = (self.opcode & 0x0F00) >> 8;
                self.registers[x as usize] = (self.opcode & 0x00FF) as u8;
                self.program_counter += 2;
            }
            0x7 => {
                let x = (self.opcode & 0x0F00) >> 8;
                self.registers[x as usize] += (self.opcode & 0x0FF) as u8;
                self.program_counter += 2;
            }

            0x8 => {
                let x = (self.opcode & 0x0F00) >> 8;
                let y = (self.opcode & 0x00F0) >> 4;
                let mode = self.opcode & 0x000F;

                match mode {
                    0 => self.registers[x as usize] = self.registers[y as usize],
                    1 => self.registers[x as usize] |= self.registers[y as usize],
                    2 => self.registers[x as usize] &= self.registers[y as usize],
                    3 => self.registers[x as usize] ^= self.registers[y as usize],
                    4 => {
                        let sum = (self.registers[x as usize] as u16)
                            + (self.registers[y as usize] as u16);

                        self.registers[0xF] = if sum > 255 { 1 } else { 0 };
                        self.registers[x as usize] = sum as u8;
                    }
                    5 => {
                        self.registers[0xF] =
                            if self.registers[x as usize] > self.registers[y as usize] {
                                1
                            } else {
                                0
                            };
                        self.registers[x as usize] -= self.registers[y as usize]
                    }
                    6 => {
                        self.registers[0xF] = self.registers[x as usize] & 0b00000001;
                        self.registers[x as usize] >>= 1;
                    }
                    7 => {
                        self.registers[0xF] =
                            if self.registers[y as usize] > self.registers[x as usize] {
                                1
                            } else {
                                0
                            };
                        self.registers[x as usize] =
                            self.registers[y as usize] - self.registers[x as usize]
                    }
                    0xE => {
                        self.registers[0xF] = if self.registers[x as usize] & 0x80 != 0 {
                            1
                        } else {
                            0
                        };
                        self.registers[x as usize] <<= 1;
                    }
                    _ => {}
                }

                self.program_counter += 2;
            }

            0x9 => {
                let x = self.opcode & 0x0F00 >> 8;
                let y = self.opcode & 0x00F0 >> 4;
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.program_counter += 2
                }
                self.program_counter += 2
            }

            0xA => {
                self.index = self.opcode & 0x0FFF;
                self.program_counter += 2
            }

            0xB => self.program_counter = self.opcode & 0x0FFF + self.registers[0] as u16,
            0xC => {
                let x = self.opcode & 0x0F00 >> 8;
                let random_byte: u8 = self.rng.gen();
                self.registers[x as usize] = random_byte & (self.opcode & 0x00FF) as u8;
            }
            0xD => {
                self.registers[0xF] = 0;

                let x = self.opcode & 0x0F00 >> 8;
                let y = self.opcode & 0x00F0 >> 4;
                let n = self.opcode & 0x000F;

                let register_x = self.registers[x as usize];
                let register_y = self.registers[y as usize];

                let mut height = 0;
                while height < n {
                    let pixel = self.memory[(self.index + 1) as usize];

                    let mut width = 0;
                    while width < 8 {
                        let a420: u8 = 0x80;

                        if pixel & (a420 >> width) != 0 {
                            let t_x = (x + width) % 64;
                            let t_y = (y + height) % 32;

                            let index = t_x + t_y * 64;

                            self.graphics[index as usize] ^= 1;

                            if self.graphics[index as usize] == 0 {
                                self.registers[0xF] = 1;
                            }
                        }

                        width += 1;
                    }
                    height += 1;
                }

                self.program_counter += 2;
            }

            0xE => {
                let x = self.opcode & 0x0F00 >> 8;
                let mode = self.opcode & 0x00FF;

                if mode == 0x9E {
                    if self.keys[self.registers[x as usize] as usize] == 1 {
                        self.program_counter += 2
                    }
                } else if mode == 0xA1 {
                    if self.keys[self.registers[x as usize] as usize] != 1 {
                        self.program_counter += 2
                    }
                }

                self.program_counter += 2;
            }

            0xF => {
                let x = self.opcode & 0x0F00 >> 8;
                let mode = self.opcode & 0x00FF;

                match mode {
                    0x07 => {
                        self.registers[x as usize] = self.delay_timer;
                    }
                    0x0A => {
                        let key_pressed = false;

                        for (i, &key) in self.keys.iter().enumerate() {
                            if key != 0 {
                                self.registers[x as usize] = i as u8;
                                break;
                            }
                        }

                        if !key_pressed {
                            return; // TODO: will not work?
                        }
                    }
                    0x15 => self.delay_timer = self.registers[x as usize],
                    0x18 => self.sound_timer = self.registers[x as usize],
                    0x1E => self.index += self.registers[x as usize] as u16,
                    0x29 => {
                        if self.registers[x as usize] > 16 {
                            self.index = (self.registers[x as usize] * 0x5) as u16;
                        }
                    }
                    0x33 => {
                        self.memory[self.index as usize] = self.registers[x as usize] / 100;
                        self.memory[(self.index + 1) as usize] =
                            (self.registers[x as usize] / 10) % 10;
                        self.memory[(self.index + 2) as usize] = self.registers[x as usize] % 10;
                    }
                    0x55 => {
                        for i in 0..x {
                            self.memory[(self.index + i) as usize] = self.registers[i as usize]
                        }
                    }
                    0x65 => {
                        for i in 0..x {
                            self.registers[(self.index + i) as usize] = self.memory[i as usize]
                        }
                    }
                    _ => {}
                }

                self.program_counter += 2;
            }

            _ => {}
        }
    }
}
