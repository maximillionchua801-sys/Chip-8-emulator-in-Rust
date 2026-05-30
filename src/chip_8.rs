use std::path::Component::ParentDir;

use rand::random;
pub const VIDEO_WIDTH: usize = 64;
pub const VIDEO_HEIGHT: usize = 32;
const FONTSET_START_ADDRESS: usize = 0x50;
#[derive(Debug)]
pub struct Memory {
    //The CHIP-8 has 4096 bytes of memory, meaning the address space is from 0x000 to 0xFFF.
    // 0x000-0x1FF: Originally reserved for the CHIP-8 interpreter, but in our modern emulator we will just never write to or read from that area. Except for…
    // 0x050-0x0A0: Storage space for the 16 built-in characters (0 through F), which we will need to manually put into our memory because ROMs will be looking for those characters.
    // 0x200-0xFFF: Instructions from the ROM will be stored starting at 0x200, and anything left after the ROM’s space is free to use.
    pub mem: [u8; 4096],
    // register V0 to VF contains values from 0x00 - 0xFF or 8 byte values
    pub register: [u8; 16],
    // The Index Register is a special register used to store memory addresses for use in operations. It’s a 16-bit register because the maximum memory address (0xFFF) is too big for an 8-bit register
    pub index_reg: u16,
    //The Program Counter (PC) is a special register that holds the address of the next instruction to execute. Again, it’s 16 bits because it has to be able to hold the maximum memory address (0xFFF).
    pub program_counter: u16,
    //16 level stack
    //the stack stores return addresses so the program knows where to continue after a function call.
    pub stack: Vec<u16>,
    //Stack pointer basically just keeps track on which register we are in on the top of the stack.
    pub stack_pointer: u8,
    //The CHIP-8 has a simple timer used for timing. If the timer value is zero, it stays zero. If it is loaded with a value, it will decrement at a rate of 60Hz.
    pub delay_timer: u8,
    //The CHIP-8 also has another simple timer used for sound. Its behavior is the same (decrementing at 60Hz if non-zero), but a single tone will buzz when it’s non-zero. Programmers used this for simple sound emission.
    pub sound_timer: u8,
    //basically just stores the current state of if the related key is currently pressed.
    pub input_key: [bool; 16],
    //display essentially a array that represents each pixel in a 64x32 pixel screen each bool tells if the display for this pixel is on or off
    //note to index through graphics array let index = y * 64 + x; 64 being its row
    pub graphics: [bool; 64 * 32],
    //opcode
    pub opcode: u16,
    pub rng_state: u32,
}
impl Memory {
    pub fn new() -> Self {
        Self {
            mem: [0; 4096],
            register: [0; 16],
            index_reg: 0,
            program_counter: 0,
            stack: Vec::new(),
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            input_key: [false; 16],
            graphics: [false; 64 * 32],
            opcode: 0,
            rng_state: 0,
        }
    }
    pub fn execute_opcode(&mut self) {
        match self.opcode & 0xF000 {
            0x0000 => match self.opcode {
                0x00E0 => self.op_00E0(),
                0x00EE => self.op_00EE(),
                _ => {}
            },

            0x1000 => self.op_1nnn(),
            0x2000 => self.op_2nnn(),
            0x3000 => self.op_3xkk(),
            0x4000 => self.op_4xkk(),
            0x5000 => self.op_5xy0(),
            0x6000 => self.op_6xkk(),
            0x7000 => self.op_7xkk(),
            0x8000 => match self.opcode & 0x000F {
                0x0 => self.op_8xy0(),
                0x1 => self.op_8xy1(),
                0x2 => self.op_8xy2(),
                0x3 => self.op_8xy3(),
                0x4 => self.op_8xy4(),
                0x5 => self.op_8xy5(),
                0x6 => self.op_8xy6(),
                0x7 => self.op_8xy7(),
                0xE => self.op_8xyE(),
                _ => {}
            },
            0x9000 => self.op_9xy0(),
            0xA000 => self.op_Annn(),
            0xB000 => self.op_Bnnn(),
            0xC000 => self.op_Cxkk(),
            0xD000 => self.op_Dxyn(),
            0xE000 => match self.opcode & 0x00FF {
                0x009E => self.op_Ex9E(),
                0x00A1 => self.op_ExA1(),
                _ => {}
            },
            0xF000 => match self.opcode & 0x00FF {
                0x0007 => self.op_Fx07(),
                0x000A => self.op_Fx0A(),
                0x0015 => self.op_Fx15(),
                0x0018 => self.op_Fx18(),
                0x001E => self.op_Fx1E(),
                0x0029 => self.op_Fx29(),
                0x0033 => self.op_Fx33(),
                0x0055 => self.op_Fx55(),
                0x0065 => self.op_Fx65(),
                _ => {}
            },

            _ => {}
        }
    }
    //fetch,decode, execute
    pub fn cycle(&mut self) {
        //fetch
        self.opcode = ((self.mem[self.program_counter as usize] as u16) << 8)
            | (self.mem[(self.program_counter + 1) as usize] as u16);

        //update our program counter
        self.program_counter += 2;

        //decode and execute
        self.execute_opcode();

        if (self.delay_timer > 0) {
            self.delay_timer -= 1;
        }

        if (self.sound_timer > 0) {
            self.sound_timer -= 1;
        }
    }
    //CLS clear graphics
    fn op_00E0(&mut self) {
        self.graphics.fill(false)
    }
    //RET return from a subroutine
    fn op_00EE(&mut self) {
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
    }
    //JP addr Jump to location nnn. The interpreter sets the program counter to nnn.
    fn op_1nnn(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;
        self.program_counter = address;
    }
    //CALL addr - Call subroutine at nnn.
    fn op_2nnn(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = address;
    }
    // SE Vx, byte - Skip next instruction if Vx = kk.
    fn op_3xkk(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.program_counter = address;
    }
    // SNE Vx, byte - Skip next instruction if Vx != KK.
    fn op_4xkk(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte: u8 = (self.opcode & 0x00FF) as u8;
        if (self.register[vx as usize] != byte) {
            self.program_counter += 2;
        }
    }
    // SE Vx,Vy - Skip next instruction if Vx = Vy
    fn op_5xy0(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

        if (self.register[vx as usize] == self.register[vy as usize]) {
            self.program_counter += 2;
        }
    }
    // LD Vx, byte - Set VX = kk
    fn op_6xkk(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte: u8 = (self.opcode & 0x00FF) as u8;
        self.register[vx as usize] = byte;
    }
    // ADD Vx,byte - Set Vx = Vx + kk.
    fn op_7xkk(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte: u8 = (self.opcode & 0x00FF) as u8;
        self.register[vx as usize] += byte;
    }
    // LD Vx,Vy -Set Vx = Vy.
    fn op_8xy0(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;
        self.register[vx as usize] = self.register[vy as usize];
    }
    // OR Vx,Vy  - Set Vx = Vx OR Vy
    fn op_8xy1(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;
        self.register[vx as usize] |= self.register[vy as usize];
    }
    //AND Vx, Vy - Set Vx = Vx AND Vy
    fn op_8xy2(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;
        self.register[vx as usize] &= self.register[vy as usize];
    }
    //XOR Vx, VY - Set Vx = Vx XOR Vy
    fn op_8xy3(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;
        self.register[vx as usize] ^= self.register[vy as usize];
    }
    // ADD Vx, Vy - Set Vx = Vx + Vy, set VF = carry.
    // The values of Vx and Vy are added together.
    // If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0.
    //  Only the lowest 8 bits of the result are kept, and stored in Vx.
    fn op_8xy4(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;
        let sum = self.register[vx as usize] as u16 + self.register[vy as usize] as u16;

        self.register[0xF] = if sum > 0xFF { 1 } else { 0 };
        self.register[vx as usize] = sum as u8;
    }
    //SUB Vx, Vy - Set Vx = Vx - Vy, set VF = NOT borrow. If Vx > Vy, then VF is set to 1,
    // otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx
    fn op_8xy5(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;
        self.register[0xF] = if self.register[vx as usize] > self.register[vy as usize] {
            1
        } else {
            0
        };
        self.register[vx as usize] -= self.register[vy as usize]
    }
    // Set Vx = Vx SHR 1
    // If the least-significant bit of Vx is 1,
    // then VF is set to 1, otherwise 0.
    //  Then Vx is divided by 2.
    fn op_8xy6(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        self.register[0xF] = (self.register[vx as usize] & 0x1);
        self.register[vx as usize] >>= 1;
    }
    //SUBN Vx, Vy
    //Set Vx = Vy - Vx, set VF = NOT borrow.
    //If Vy > Vx, then VF is set to 1, otherwise 0.
    // Then Vx is subtracted from Vy, and the results stored in Vx.
    fn op_8xy7(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;
        if (self.register[vy as usize] > self.register[vx as usize]) {
            self.register[0xF] = 1;
        } else {
            self.register[0xF] = 0;
        }
        self.register[vx as usize] = self.register[vy as usize] - self.register[vx as usize];
    }
    //SHL Vx {, Vy}
    //Set Vx = Vx SHL 1.
    // If the most-significant bit of Vx is 1,
    // then VF is set to 1, otherwise to 0.
    // Then Vx is multiplied by 2.
    fn op_8xyE(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        self.register[0xF] = (self.register[vx as usize] & 0x80) >> 7;
        self.register[vx as usize] <<= 1;
    }
    //SNE Vx, Vy
    //Skip next instruction if Vx != Vy.
    fn op_9xy0(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;
        if (self.register[vx as usize] != self.register[vy as usize]) {
            self.program_counter += 2;
        }
    }
    //LD I, addr
    //Set I = nnn.
    fn op_Annn(&mut self) {
        let address = self.opcode & 0x0FFF;
        self.index_reg = address;
    }
    //JP V0, addr
    //Jump to location nnn + V0.
    fn op_Bnnn(&mut self) {
        let address = self.opcode & 0x0FFF;
        self.program_counter = self.register[0] as u16 + address;
    }
    //RND Vx, byte
    //Set Vx = random byte AND kk
    fn op_Cxkk(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte = self.opcode & 0x00FF;
        let rand: u8 = random();
        self.register[vx as usize] = rand & byte as u8;
    }
    //DRW Vx, Vy, nibble
    //Display n-byte sprite starting at memory location I at (Vx, Vy),
    // set VF = collision.
    fn op_Dxyn(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;
        let height = self.opcode & 0x000F;

        let pos_x = self.register[vx as usize] % VIDEO_WIDTH as u8;
        let pos_y = self.register[vy as usize] % VIDEO_HEIGHT as u8;

        self.register[0xF] = 0;

        for row in 0..height {
            let sprite_byte = self.mem[(self.index_reg + row as u16) as usize];
            for col in 0..8 {
                let _sprite_pixel = sprite_byte & (0x80 >> col);
                let _index =
                    (pos_y as usize + row as usize) * VIDEO_WIDTH + (pos_x as usize + col as usize);
                if (_sprite_pixel != 0) {
                    if self.graphics[_index] {
                        self.register[0xF] = 1;
                    }
                    self.graphics[_index] ^= true;
                }
            }
        }
    }
    //SKP Vx
    //Skip next instruction if key with the value of Vx is pressed.
    fn op_Ex9E(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let key = self.register[vx as usize];
        if (self.input_key[key as usize]) {
            self.program_counter += 2;
        }
    }
    //SKNP Vx
    //Skip next instruction if key with the value of Vx is not pressed.
    fn op_ExA1(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let key = self.register[vx as usize];
        if (!self.input_key[key as usize]) {
            self.program_counter += 2;
        }
    }
    //LD Vx, DT
    //Set Vx = delay timer value
    fn op_Fx07(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        self.register[vx as usize] = self.delay_timer;
    }
    //LD Vx,K
    //Wait for a key press, store the value of the key in Vx.
    fn op_Fx0A(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        if (self.input_key[0]) {
            self.register[vx as usize] = 0;
        } else if (self.input_key[1]) {
            self.register[vx as usize] = 1;
        } else if (self.input_key[2]) {
            self.register[vx as usize] = 2;
        } else if (self.input_key[3]) {
            self.register[vx as usize] = 3;
        } else if (self.input_key[4]) {
            self.register[vx as usize] = 4;
        } else if (self.input_key[5]) {
            self.register[vx as usize] = 5;
        } else if (self.input_key[6]) {
            self.register[vx as usize] = 6;
        } else if (self.input_key[7]) {
            self.register[vx as usize] = 7;
        } else if (self.input_key[8]) {
            self.register[vx as usize] = 8;
        } else if (self.input_key[9]) {
            self.register[vx as usize] = 9;
        } else if (self.input_key[10]) {
            self.register[vx as usize] = 10;
        } else if (self.input_key[11]) {
            self.register[vx as usize] = 11;
        } else if (self.input_key[12]) {
            self.register[vx as usize] = 12;
        } else if (self.input_key[13]) {
            self.register[vx as usize] = 13;
        } else if (self.input_key[14]) {
            self.register[vx as usize] = 14;
        } else if (self.input_key[15]) {
            self.register[vx as usize] = 15;
        } else {
            self.program_counter -= 2;
        }
    }
    //LD DT, Vx
    //Set delay timer = Vx.
    fn op_Fx15(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        self.delay_timer = self.register[vx as usize];
    }
    // LD ST, Vx
    //Set Sound Timer = Vx.
    fn op_Fx18(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        self.sound_timer = self.register[vx as usize];
    }
    //ADD I, Vx
    // Set I = I + Vx
    fn op_Fx1E(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        self.index_reg += self.register[vx as usize] as u16;
    }
    // LD F, Vx
    //Set I = location of sprite for digit Vx
    fn op_Fx29(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let digit = self.register[vx as usize];
        self.index_reg = (FONTSET_START_ADDRESS as u16 + (5 * digit as u16));
    }
    // LD B, Vx
    //Store BCD representation of Vx in memory locations I, I+1, and I+2
    //The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I,
    // the tens digit at location I+1, and the ones digit at location I+2.
    fn op_Fx33(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        let mut value = self.register[vx as usize];

        self.mem[(self.index_reg + 2 as u16) as usize] = value % 10;
        value /= 10;

        self.mem[(self.index_reg + 1 as u16) as usize] = value % 10;
        value /= 10;

        self.mem[self.index_reg as usize] = value % 10;
    }
    //LD [I], Vx
    //Store registers V0 through Vx in memory starting at location I
    fn op_Fx55(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        for i in 0..vx {
            self.mem[(self.index_reg + i as u16) as usize] = self.register[i as usize];
        }
    }
    //LD Vx, [I]
    // Read registers V0 through Vx from memory starting at location I.
    fn op_Fx65(&mut self) {
        let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
        for i in 0..vx {
            self.register[i as usize] = self.mem[(self.index_reg + i as u16) as usize]
        }
    }
}
