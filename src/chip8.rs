use byteorder::{ByteOrder, BigEndian};

use rng::rng_byte;
use constants::{W, H, N, ROM_ADDR, RAM_BYTES};
use opcode::{Opcode, Operation::*, OpcodeType::{self,*}, OpcodeDisassembler};
use command::{CommandRouter, CommandInterpreter, Command, 
    DisplayCommand::*, AudioCommand::*, KeyCommand::KeyDownUp, 
    MemoryCommand::SendRAM};

#[allow(non_snake_case)]
pub struct Chip8 {
    draw_flag: bool,
    key_wait: bool,
    reg_wait: usize,
    pc: u16,
    I: u16,
    sp: u8,
    stack: [u16; 0x10],
    V: [u8; 0x10],
    delay_timer: u8,
    sound_timer: u8,

    pub commands: CommandRouter,
    key_buf: [bool; 0x10],
    pixel_buf: [bool; N],
    memory_buf: [u8; RAM_BYTES]
}

impl CommandInterpreter for Chip8 {
    fn read_commands(&mut self) {
        self.commands.inbound_queue.remove_all().iter().for_each(|c| 
            match c {
                Command::Display(c) => match c {
                    SendPixels(p) => self.pixel_buf.copy_from_slice(p),
                    _ => {}
                },
                Command::Key(c) => match *c {
                    KeyDownUp(key_i, key_is_down) => {
                        self.key_buf[key_i] = key_is_down;
                        if self.key_wait && key_is_down {
                            self.key_wait=false;
                            self.V[self.reg_wait]=key_i as u8
                        }
                    }
                },
                Command::Memory(c) => match c {
                    SendRAM(bytes) => self.memory_buf.copy_from_slice(bytes)
                }
                _ => {}
            })
    }
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            draw_flag: false,
            key_wait: false,
            reg_wait: 0,
            pc: ROM_ADDR as u16,
            I: 0,
            sp: 0,
            stack: [0; 0x10],
            V: [0; 0x10],
            delay_timer: 0,
            sound_timer: 0,

            commands: CommandRouter::new(),
            key_buf: [false; 0x10],
            pixel_buf: [false; N],
            memory_buf: [0; RAM_BYTES],
        }
    }

    fn update_pixel(&mut self, x: usize, y: usize, val: bool) {
        if self.pixel_buf[y*W+x] && val {self.V[0xF]=1};
        self.pixel_buf[y*W+x] ^= val;
        self.draw_flag = true
    }

    pub fn emulate_cycle(&mut self) {
        if !self.key_wait {
            //Fetch Instruction
            let pc = self.pc as usize;
            let instruction: u16 = BigEndian::read_u16(&self.memory_buf[pc..pc+2]);
            self.pc += 2;

            //Decode Opcode
            let opcode: Opcode = OpcodeDisassembler::disassemble(instruction);

            //Execute Opcode
            self.execute_opcode(opcode);

            //Update timers and control audio beep
            if self.delay_timer > 0 { self.delay_timer -= 1 };
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
                self.commands.outbound_queue.push(Command::Audio(Play));
            } else {
                self.commands.outbound_queue.push(Command::Audio(Pause));
            }

            self.commands.outbound_queue.push(
                Command::Display(SendPixels(self.pixel_buf.clone())));
            
            self.commands.outbound_queue.push(
                Command::Memory(SendRAM(self.memory_buf.clone())));

            if self.draw_flag {
                self.commands.outbound_queue.push(Command::Display(SendDraw));
                self.draw_flag = false;
            }
        }
    }

    fn execute_opcode(&mut self, opcode: Opcode) {
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
                self.memory_buf[self.I as usize] = self.V[x as usize] / 100;
                self.memory_buf[(self.I+1) as usize] = (self.V[x as usize] / 10) % 10;
                self.memory_buf[(self.I+2) as usize] = self.V[x as usize] % 10
            },
            Opcode(LD, RI_X(x)) => (0..x+1).for_each(|i| 
                self.memory_buf[(self.I + i) as usize] = self.V[i as usize]),
            Opcode(LD, X_RI(x)) => (0..x+1).for_each(|i| 
                self.V[i as usize] = self.memory_buf[(self.I + i) as usize]),
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
                        self.update_pixel(px, py, 
                            (((self.memory_buf[(self.I + i) as usize] >> (7 - ii)) & 1) == 1) as bool);
                        px+=1
                    };
                    py+=1
                };
            },
            Opcode(SKP, X(x)) => self.skip(self.key_buf[(self.V[x as usize] & 0xF) as usize]),
            Opcode(SKNP, X(x)) => self.skip(!self.key_buf[(self.V[x as usize] & 0xF) as usize]),
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
        self.commands.outbound_queue.push(Command::Display(SendClearDisplay));
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
