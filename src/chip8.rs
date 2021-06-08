//ROM file access
use std::fs::File;
use std::io::Read;
use sdl2::audio::{AudioDevice, AudioStatus, AudioCallback};

use rng::rng_byte;
use constants::{W, H, N};
use opcode::{Opcode, Operation::*, OpcodeType::*, OpcodeDisassembler};

#[allow(non_snake_case)]
pub struct Chip8 {
    pub draw_flag: bool, //Only draw if screen changes: 0x00E0 – Clear screen, 0xDXYN – Draw sprite
    pub key_wait: bool, //set to true if the program is waiting for a key to be entered
    pub reg_wait: usize, //the index of the V register the waited key value will be stored in
    pc: u16, //Program counter
    I: u16, //Index register
    sp: u8, //Stack pointer
    stack: [u16; 0x10], //The stack for return calls
    pub V: [u8; 0x10], //16 general purpose registers V0..V15. V15 (VF) is used for the carry flag
    delay_timer: u8, //counts down to 0 (60Hz)
    sound_timer: u8, //counts down to 0 (60Hz). system's buzzer sounds whenever the timer reaches 0.
    pub clear_display_flag: bool
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            draw_flag: false,
            key_wait: false,
            reg_wait: 0,
            pc: 0x200, //program starts at 0x200
            I: 0,
            sp: 0,
            stack: [0; 0x10],
            V: [0; 0x10],
            delay_timer: 0,
            sound_timer: 0,
            clear_display_flag: false
        }
    }

    pub fn load_game(&mut self, memory:  &mut [u8; 0x1000], game_name: String) {
        let mut rom_buf: Vec<u8> = Vec::new();
        let mut file = File::open(&game_name).unwrap();
        file.read_to_end(&mut rom_buf).unwrap();
        let mut addr = 0x200;
        for byte in &rom_buf {
            memory[addr]=*byte;
            addr+=1
        }
    }

    fn update_pixel(&mut self, pixels: &mut [bool; N], x: usize, y: usize, val: bool) {
        if pixels[y*W+x] && val {self.V[0xF]=1};
        pixels[y*W+x] ^= val;
        self.draw_flag = true
    }

    //Clears the display. All pixels set to black (off)
    fn clear_display(&mut self) {
        self.clear_display_flag = true; 
        self.draw_flag = true
    }

    pub fn disassemble_code(&mut self, memory: &[u8; 0x1000]) {
        println!("Disassembling code: \n");

        let mut addr: usize = 0x200;
        let mut opcode: u16 = (memory[addr] as u16) << 8 | (memory[addr + 1] as u16);
        while opcode!=0 {
            let disassembled = OpcodeDisassembler::disassemble(opcode);
            println!("{:03x}: {:04X} '{}'", addr, opcode, disassembled);
            addr+=2;
            opcode = (memory[addr] as u16) << 8 | (memory[addr + 1] as u16)
        }
    }
    
    fn fetch_instruction(&mut self, bytes: &[u8]) -> u16 {
        (bytes[0] as u16) << 8 | bytes[1] as u16
    }

    pub fn emulate_cycle(&mut self, memory: &mut [u8; 0x1000], pixels: &mut [bool; N], key: &[bool; 0x10], device: &AudioDevice<impl AudioCallback>) {
        //Fetch Opcode
        let pc = self.pc as usize;
        let instruction: u16 = self.fetch_instruction(&memory[pc..pc+2]);
        self.pc += 2;

        //Decode Opcode
        let opcode: Opcode = OpcodeDisassembler::disassemble(instruction);
        let u = instruction >> 12; //the first nibble in the instruction
        let nnn = instruction & 0xFFF; //address
        let nn: u8 = (instruction & 0xFF) as u8; //8 bit constant (byte)
        let n = instruction & 0xF; //4 bit constant (last nibble)
        let x: usize = (nnn as usize) >> 8; //the second nibble in the instruction (when applicable)
        let y: usize = (nn as usize) >> 4; // the third nibble in the instruction (when applicable)
        #[allow(non_snake_case)]
        let I: usize = self.I as usize;

        match opcode {
            Opcode(CLS, NONE) => self.clear_display(),
            _ => match instruction {
                0x00EE => {self.pc = self.stack[self.sp as usize]; self.sp-=1}, //RET; return from a subroutine
                _ => match u {
                    //0x0 => {}, //SYS addr; 0NNN, Calls RCA 1802 program at address NNN. not used for this emulator
                    0x1 => {self.pc = nnn}, //JP addr; 1NNN, Goto NNN
                    0x2 => {self.sp+=1; self.stack[self.sp as usize]=self.pc; self.pc = nnn}, //CALL addr; 2NNN, Call subroutine at NNN
                    0x3 => {if self.V[x]==nn {self.pc+=2}}, //SE Vx, byte; 3XNN, skip if Vx==NN
                    0x4 => {if self.V[x]!=nn {self.pc+=2}}, //SNE Vx, byte; 4XNN, skip if Vx!=NN
                    0x5 => if n==0 {
                            if self.V[x]==self.V[y] {self.pc+=2} //SE Vx, Vy; 5XY0, skip if Vx==Vy
                        } else {panic!("Invalid opcode: {:04X} at {:04x}", instruction, self.pc-2)},
                    0x6 => {self.V[x]=nn}, //LD Vx, byte; 6XNN, Vx = NN (assignment)
                    0x7 => {self.V[x]=((self.V[x] as u16)+(nn as u16)) as u8}, //ADD Vx, byte; 7XNN, Vx += NN (addition)
                    0x8 => match n { //8XYn
                        0x0 => {self.V[x]=self.V[y]}, //LD Vx, Vy; 8XY0, Vx = Vy
                        0x1 => {self.V[x]|=self.V[y]}, //OR Vx, Vy; 8XY1, Vx = Vx|Vy (OR)
                        0x2 => {self.V[x]&=self.V[y]}, //AND Vx, Vy; 8XY2, Vx = Vx&Vy (AND)
                        0x3 => {self.V[x]^=self.V[y]}, //XOR Vx, Vy; 8XY3, Vx = Vx^Vy (XOR)
                        0x4 => {self.V[0xF]=(((self.V[x] as u16)+(self.V[y] as u16))>>8) as u8; self.V[x]=(((self.V[x] as u16)+(self.V[y] as u16))&0xFF) as u8}, //ADD Vx, Vy; 8XY4, Vx += Vy
                        0x5 => {self.V[0xF]=(self.V[x]>self.V[y]) as u8; self.V[x]=((self.V[x] as i16)-(self.V[y] as i16)) as u8}, //SUB Vx, Vy; 8XY5, Vx -= Vy
                        0x6 => {self.V[0xF]=self.V[x]&0x1; self.V[x]=self.V[x] >> 1}, //SHR Vx; 8XY6, Vx >> 1
                        0x7 => {self.V[0xF]=(self.V[y]>self.V[x]) as u8; self.V[x]=((self.V[y] as i16)-(self.V[x] as i16)) as u8}, //SUBN Vx, Vy; 8XY7, Vx=Vy-Vx
                        0xE => {self.V[0xF]=self.V[x]>>7; self.V[x]=self.V[x] << 1}, //SHL Vx; 8XYE, Vx << 1
                        _ => panic!("Invalid opcode: {:04X} at {:04x}", instruction, self.pc-2)
                    },
                    0x9 => {if self.V[x]!=self.V[y] {self.pc+=2}}, //SNE Vx, Vy; 9XY0, skip if Vx!=Vy
                    0xA => {self.I=nnn}, //LD I, addr; ANNN, I=NNN
                    0xB => {self.pc=(self.V[0] as u16)+nnn}, //JP V0, addr; BNNN, PC = V0 + NNN (Error handling could be added)
                    0xC => {self.V[x]=rng_byte()&nn}, //RND Vx, byte; CXNN, Vx = rand() & NN
                    0xD => {
                        /*The interpreter reads n bytes from memory, starting at the address stored in I.
                        These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
                        Sprites are XORed onto the existing screen.
                        If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
                        If the sprite is positioned so part of it is outside the coordinates of the display,
                        it wraps around to the opposite side of the screen. */
                        self.V[0xF]=0;
                        let mut px: usize = self.V[x] as usize;
                        let mut py: usize = self.V[y] as usize;
                        for i in 0..n {
                            while py>=H {py-=H};
                            for ii in 0..8 {
                                while px>=W {px-=W};
                                self.update_pixel(pixels, px, py, (((memory[I+(i as usize)]>>(7-ii))&1)==1) as bool);
                                px+=1
                            };
                            px=self.V[x] as usize;
                            py+=1
                        };
                    }, //DRW Vx, Vy, nibble; DXYN, Display n-byte sprite starting at memory location I at (Vx, Vy)
                    0xE => match nn {
                        0x9E => {if key[(self.V[x]&0xF) as usize] {self.pc+=2}}, //SKP Vx; EX9E, Skip next instruction if key with the value of Vx is pressed.
                        0xA1 => {if !key[(self.V[x]&0xF) as usize] {self.pc+=2}}, //SKNP Vx; EXA1, Skip next instruction if key with the value of Vx is not pressed.
                        _ => panic!("Invalid opcode: {:04X} at {:04x}", instruction, self.pc-2)
                    },
                    0xF => match nn { //FXzz
                        0x07 => {self.V[x]=self.delay_timer}, //LD Vx, DT; FX07 Set Vx = delay timer value.
                        0x0A => {self.key_wait=true; self.reg_wait=x as usize}, //LD Vx, K; FX0A, Wait for a key press, store the value of the key in Vx.
                        0x15 => {self.delay_timer=self.V[x]}, //LD DT, Vx; FX15, Set delay timer = Vx.
                        0x18 => {self.sound_timer=self.V[x]}, //LD ST, Vx; FX18, Set sound timer = Vx.
                        0x1E => {self.V[0xF]=((self.I+(self.V[x] as u16))>>12) as u8; self.I=(self.I+(self.V[x] as u16))&0xFFF}, //ADD I, Vx; FX1E I += Vx
                        0x29 => {self.I=(0x50+(0x5*self.V[x])) as u16}, //LD F, Vx; FX29 I=sprite_addr[Vx]
                        0x33 => {
                            /*The interpreter takes the decimal value of Vx,
                            and places the hundreds digit in memory at location in I,
                            the tens digit at location I+1,
                            and the ones digit at location I+2.*/
                            memory[I] = (self.V[x]/10)/10; //hundreds
                            memory[I+1] = (self.V[x]/10)%10; //tens
                            memory[I+2] = self.V[x]%10 //ones
                        }, //LD B, Vx; FX33 set_BCD(Vx)
                        0x55 => {
                            for i in 0..(x+1) {
                                memory[I+i] = self.V[i]
                            }
                        }, //LD [I], Vx; LD [I], Vx; FX55 reg_dump(Vx, &I)
                        0x65 => {
                            for i in 0..(x+1) {
                                self.V[i] = memory[I+i]
                            }
                        }, //LD Vx, [I]; FX65 reg_load(Vx,&I)
                        _ => panic!("Invalid opcode: {:04X} at {:04x}", instruction, self.pc-2)
                    },
                    _ => panic!("Invalid opcode: {:04X} at {:04x}", instruction, self.pc-2)
                }
            }
        };

        //Update timers and control audio beep
        if self.delay_timer>0 { self.delay_timer-=1 };
        if self.sound_timer>0 {
            self.sound_timer-=1;
            if device.status()==AudioStatus::Paused {
                device.resume();
            }
        }
        else
        {
            if device.status()==AudioStatus::Playing {
                device.pause();
            }
        }
    }
}
