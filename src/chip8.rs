//ROM file access
use std::fs::File;
use std::io::Read;
use sdl2::audio::{AudioDevice, AudioStatus, AudioCallback};

use rng::rng_byte;
use constants::{W, H, N};
use opcode::{Opcode, Operation::*, OpcodeType::{self,*}, OpcodeDisassembler};

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
        //Fetch Instruction
        let pc = self.pc as usize;
        let instruction: u16 = self.fetch_instruction(&memory[pc..pc+2]);
        self.pc += 2;

        //Decode Opcode
        let opcode: Opcode = OpcodeDisassembler::disassemble(instruction);

        //Execute Opcode
        self.execute_opcode(opcode, memory, pixels, key);

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

    fn execute_opcode(&mut self, opcode: Opcode, memory: &mut [u8; 0x1000], pixels: &mut [bool; N], key: &[bool; 0x10]) {
        match opcode {
            Opcode(CLS, NONE) => self.clear_display(),
            Opcode(RET, NONE) => self.subroutine_return(),
            Opcode(JP, NNN(nnn)) => self.jump(nnn),
            Opcode(JP, V0_NNN(nnn)) => self.jump(self.V[0] as u16 + nnn),
            Opcode(CALL, NNN(nnn)) => self.subroutine_call(nnn),
            Opcode(SE, op_type) => self.skip_equal(op_type),
            Opcode(SNE, op_type) => self.skip_not_equal(op_type),
            Opcode(LD, XNN(x, nn)) => self.V[x as usize] = (nn & 0xFF) as u8,
            Opcode(LD, XY(x, y)) => self.V[x as usize] = self.V[y as usize],
            Opcode(LD, I_NNN(nnn)) => self.I = nnn,
            Opcode(LD, X_DT(x)) => self.V[x as usize] = self.delay_timer,
            Opcode(LD, X_K(x)) => { 
                self.key_wait = true; 
                self.reg_wait = (x & 0xF) as usize 
            },
            Opcode(LD, DT_X(x)) => self.delay_timer = self.V[x as usize],
            Opcode(LD, ST_X(x)) => self.sound_timer = self.V[x as usize],
            Opcode(LD, F_X(x)) => {
                self.I = (0x50 + (0x5 * self.V[x as usize])) as u16
            },
            Opcode(LD, B_X(x)) => { 
                memory[self.I as usize] = self.V[x as usize] / 100;
                memory[(self.I+1) as usize] = (self.V[x as usize] / 10) % 10;
                memory[(self.I+2) as usize] = self.V[x as usize] % 10
            },
            Opcode(LD, RI_X(x)) => (0..x+1).for_each(|i| 
                memory[(self.I + i) as usize] = self.V[i as usize]),
            Opcode(LD, X_RI(x)) => (0..x+1).for_each(|i| 
                self.V[i as usize] = memory[(self.I + i) as usize]),
            Opcode(ADD, XNN(x, nn)) => {
                //self.V[0xF] = ((((self.V[x as usize] as u16) + nn) & 0xFF) >> 8) as u8; 
                self.V[x as usize] = (((self.V[x as usize] as u16) + nn) & 0xFF) as u8
            },
            Opcode(ADD, I_X(x)) => {
                //self.V[0xF] = ((self.I + (self.V[x as usize] as u16)) >> 12) as u8; 
                self.I += self.V[x as usize] as u16;
                self.I &= 0xFFF
            },
            Opcode(ADD, XY(x, y)) => {
                self.V[0xF] = (((self.V[x as usize] as u16) + (self.V[y as usize] as u16)) >> 8) as u8; 
                self.V[x as usize] = (((self.V[x as usize] as u16)+(self.V[y as usize] as u16)) & 0xFF) as u8
            },
            Opcode(OR, XY(x, y)) => self.V[x as usize] |= self.V[y as usize],
            Opcode(AND, XY(x, y)) => self.V[x as usize] &= self.V[y as usize],
            Opcode(XOR, XY(x, y)) => self.V[x as usize] ^= self.V[y as usize],
            Opcode(SUB, XY(x, y)) => {
                self.V[0xF] = (self.V[x as usize] > self.V[y as usize]) as u8; 
                self.V[x as usize] = ((self.V[x as usize] as i16) - (self.V[y as usize] as i16)) as u8
            },
            Opcode(SUBN, XY(x, y)) => {
                self.V[0xF] = (self.V[y as usize] > self.V[x as usize]) as u8; 
                self.V[x as usize] = ((self.V[y as usize] as i16) - (self.V[x as usize] as i16)) as u8
            },
            Opcode(SHR, X(x)) => {
                self.V[0xF] = self.V[x as usize] & 0x1;
                self.V[x as usize] >>= 1
            },
            Opcode(SHL, X(x)) => {
                self.V[0xF] = self.V[x as usize] >> 7;
                self.V[x as usize] <<= 1
            },
            Opcode(RND, XNN(x, nn)) => self.V[x as usize] = rng_byte() & nn as u8,
            Opcode(DRW, XYN(x, y, n)) => {
                self.V[0xF] = 0;
                let mut py: usize = self.V[y as usize] as usize;
                for i in 0..n {
                    while py >= H {py -= H};
                    let mut px: usize = self.V[x as usize] as usize;
                    for ii in 0..8 {
                        while px >= W {px -= W};
                        self.update_pixel(pixels, px, py, 
                            (((memory[(self.I + i) as usize] >> (7 - ii)) & 1) == 1) as bool);
                        px+=1
                    };
                    py+=1
                };
            },
            Opcode(SKP, X(x)) => self.skip(key[(self.V[x as usize] & 0xF) as usize]),
            Opcode(SKNP, X(x)) => self.skip(!key[(self.V[x as usize] & 0xF) as usize]),
            Opcode(UNDEFINED, ..) => unimplemented!("Undefined opcode behaviour encountered in program."),
            _ => unimplemented!("This opcode is not implemented in this chip8 emulator. Opcode: {:?}", opcode)
        }
    }

    fn get_tuple_from_type(&self, op_type: OpcodeType) -> Option<(u16, u16)> {
        match op_type {
            XNN(x, nn) => Some((self.V[x as usize] as u16, nn)),
            XY(x, y) => Some((self.V[x as usize] as u16, self.V[y as usize] as u16)),
            _ => None
        }
    }

    fn clear_display(&mut self) {
        self.clear_display_flag = true; 
        self.draw_flag = true
    }

    fn subroutine_return(&mut self) {
        self.pc = self.stack[self.sp as usize]; 
        self.sp -= 1 
    }

    fn jump(&mut self, location: u16) {
        self.pc = location
    }

    fn subroutine_call(&mut self, location: u16) {
        self.sp += 1; 
        self.stack[self.sp as usize] = self.pc;
        self.jump(location)
    }

    fn skip(&mut self, condition: bool) {
        if condition { self.pc += 2 }
    }

    fn skip_equal(&mut self, op_type: OpcodeType) {
        if let Some((left, right)) = self.get_tuple_from_type(op_type) {
            self.skip(left==right)
        }
    }

    fn skip_not_equal(&mut self, op_type: OpcodeType) {
        if let Some((left, right)) = self.get_tuple_from_type(op_type) {
            self.skip(left!=right)
        }
    }
}
