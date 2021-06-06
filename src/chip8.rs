//ROM file access
use std::fs::File;
use std::io::Read;
use sdl2::audio::{AudioDevice, AudioStatus, AudioCallback};

//RNG for rand_byte() instruction
use rand::Rng;

pub const W: usize = 64;
pub const H: usize = 32;
pub const PIXEL_SIZE: u32 = 20;

#[allow(non_snake_case)]
pub struct Chip8 {
    pub draw_flag: bool, //Only draw if screen changes: 0x00E0 – Clear screen, 0xDXYN – Draw sprite
    pub key_wait: bool, //set to true if the program is waiting for a key to be entered
    pub reg_wait: usize, //the index of the V register the waited key value will be stored in
    pc: u16, //Program counter
    I: u16, //Index register
    sp: u8, //Stack pointer
    stack: [u16; 0x10], //The stack for return calls
    pub reg_v: [u8; 0x10], //16 general purpose registers V0..V15. V15 (VF) is used for the carry flag
    delay_timer: u8, //counts down to 0 (60Hz)
    sound_timer: u8 //counts down to 0 (60Hz). system's buzzer sounds whenever the timer reaches 0.
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            draw_flag: false,
            key_wait: false,
            reg_wait: 0,
            pc: 0x200, //program starts at 0x200
            I: 0,
            sp: 0,
            stack: [0; 0x10],
            reg_v: [0; 0x10],
            delay_timer: 0,
            sound_timer: 0
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

    fn update_pixel(&mut self, pixels: &mut [bool; 0x800], x: usize, y: usize, val: bool) {
        if pixels[y*W+x]==val {self.reg_v[0xF]=1};
        pixels[y*W+x] ^= val
    }

    //Clears the display. All pixels set to black (off)
    fn clear_display(&mut self, pixels: &mut [bool; 0x800]) {
        for y in 0..H {
            for x in 0..W {
                reset_pixel(pixels, x as usize, y as usize, false)
            }
        }
    }

    pub fn disassemble_code(&mut self, memory: &[u8; 0x1000]) {
        println!("Disassembling code: \n");

        //let mut dfilebuf = BufWriter::new(File::create("hex.txt").unwrap());

        let mut addr: usize = 0x200;
        let mut opcode: u16 = (memory[addr] as u16) << 8 | (memory[addr + 1] as u16);
        while opcode!=0 {
            //Log to the assembly file
            //let opform = format!("{:04X}\n",opcode);
            //let opbuf = opform.as_bytes();
            //dfilebuf.write(&opbuf);

            //Decode Opcode
            let u = opcode >> 12; //the first nibble in the instruction
            let nnn = opcode & 0xFFF; //address
            let nn = opcode & 0xFF; //8 bit constant (byte)
            let n = opcode & 0xF; //4 bit constant (last nibble)
            let x = nnn >> 8; //the second nibble in the instruction (when applicable)
            let y = nn >> 4; // the third nibble in the instruction (when applicable)

            //Execute Opcode
            match opcode {
                0x00E0 => println!("{:03x}: {:04X} 'CLS'",addr,opcode), //CLS; clears the display
                0x00EE => println!("{:03x}: {:04X} 'RET'",addr,opcode), //RET; return from a subroutine
                _ => match u {
                    //0x0 => {}, //SYS addr; 0NNN, Calls RCA 1802 program at address NNN. not used for this emulator
                    0x1 => println!("{:03x}: {:04X} 'JP ${:03x}'",addr,opcode,nnn), //JP addr; 1NNN, Goto NNN
                    0x2 => println!("{:03x}: {:04X} 'CALL ${:03x}'",addr,opcode,nnn), //CALL addr; 2NNN, Call subroutine at NNN
                    0x3 => println!("{:03x}: {:04X} 'SE V{:X}, {}'",addr,opcode,x,nn), //SE Vx, byte; 3XNN, skip if Vx==NN
                    0x4 => println!("{:03x}: {:04X} 'SNE V{:X}, {}'",addr,opcode,x,nn), //SNE Vx, byte; 4XNN, skip if Vx!=NN
                    0x5 => if n==0 {println!("{:03x}: {:04X} 'SE V{:X}, V{:X}'",addr,opcode,x,y) //SE Vx, Vy; 5XY0, skip if Vx==Vy
                        } else {println!("Invalid opcode: {:04X}",opcode)},
                    0x6 => println!("{:03x}: {:04X} 'LD V{:X}, {}'",addr,opcode,x,nn), //LD Vx, byte; 6XNN, Vx = NN (assignment)
                    0x7 => println!("{:03x}: {:04X} 'ADD V{:X}, {}'",addr,opcode,x,nn), //ADD Vx, byte; 7XNN, Vx += NN (addition)
                    0x8 => match n { //8XYn
                        0x0 => println!("{:03x}: {:04X} 'LD V{:X}, V{:X}'",addr,opcode,x,y), //LD Vx, Vy; 8XY0, Vx = Vy
                        0x1 => println!("{:03x}: {:04X} 'OR V{:X}, V{:X}'",addr,opcode,x,y), //OR Vx, Vy; 8XY1, Vx = Vx|Vy (OR)
                        0x2 => println!("{:03x}: {:04X} 'AND V{:X}, V{:X}'",addr,opcode,x,y), //AND Vx, Vy; 8XY2, Vx = Vx&Vy (AND)
                        0x3 => println!("{:03x}: {:04X} 'XOR V{:X}, V{:X}'",addr,opcode,x,y), //XOR Vx, Vy; 8XY3, Vx = Vx^Vy (XOR)
                        0x4 => println!("{:03x}: {:04X} 'ADD V{:X}, V{:X}'",addr,opcode,x,y), //ADD Vx, Vy; 8XY4, Vx += Vy
                        0x5 => println!("{:03x}: {:04X} 'SUB V{:X}, V{:X}'",addr,opcode,x,y), //SUB Vx, Vy; 8XY5, Vx -= Vy
                        0x6 => println!("{:03x}: {:04X} 'SHR V{:X}'",addr,opcode,x), //SHR Vx; 8XY6, Vx >> 1
                        0x7 => println!("{:03x}: {:04X} 'SUBN V{:X}, V{:X}'",addr,opcode,x,y), //SUBN Vx, Vy; 8XY7, Vx=Vy-Vx
                        0xE => println!("{:03x}: {:04X} 'SHL V{:X}'",addr,opcode,x), //SHL Vx; 8XYE, Vx << 1
                        _ => println!("Invalid opcode: {:04X}",opcode)
                    },
                    0x9 => println!("{:03x}: {:04X} 'SNE V{:X}, V{:X}'",addr,opcode,x,y), //SNE Vx, Vy; 9XY0, skip if Vx!=Vy
                    0xA => println!("{:03x}: {:04X} 'LD I, {:X}'",addr,opcode,nnn), //LD I, addr; ANNN, I=NNN
                    0xB => println!("{:03x}: {:04X} 'JP V0, {:X}'",addr,opcode,nnn), //JP V0, addr; BNNN, PC = V0 + NNN
                    0xC => println!("{:03x}: {:04X} 'RND V{:X}, {:b}'",addr,opcode,x,nn), //RND Vx, byte; CXNN, Vx = rand() & NN
                    0xD => println!("{:03x}: {:04X} 'DRW V{:X}, V{:X}, {}'",addr,opcode,x,y,n), //DRW Vx, Vy, nibble; DXYN, Display n-byte sprite starting at memory location I at (Vx, Vy)
                    0xE => match nn {
                        0x9E => println!("{:03x}: {:04X} 'SKP V{:X}'",addr,opcode,x), //SKP Vx; EX9E, Skip next instruction if key with the value of Vx is pressed.
                        0xA1 => println!("{:03x}: {:04X} 'SKNP V{:X}'",addr,opcode,x), //SKNP Vx; EXA1, Skip next instruction if key with the value of Vx is not pressed.
                        _ => println!("Invalid opcode: {:04X}", opcode)
                    },
                    0xF => match nn { //FXzz
                        0x07 => println!("{:03x}: {:04X} 'LD V{:X}, DT'",addr,opcode,x), //LD Vx, DT; FX07 Set Vx = delay timer value.
                        0x0A => println!("{:03x}: {:04X} 'LD V{:X}, K'",addr,opcode,x), //LD Vx, K; FX0A, Wait for a key press, store the value of the key in Vx.
                        0x15 => println!("{:03x}: {:04X} 'LD DT, V{:X}'",addr,opcode,x), //LD DT, Vx; FX15, Set delay timer = Vx.
                        0x18 => println!("{:03x}: {:04X} 'LD ST, V{:X}'",addr,opcode,x), //LD ST, Vx; FX18, Set sound timer = Vx.
                        0x1E => println!("{:03x}: {:04X} 'ADD I, V{:X}'",addr,opcode,x), //ADD I, Vx; FX1E I += Vx
                        0x29 => println!("{:03x}: {:04X} 'LD F, V{:X}'",addr,opcode,x), //LD F, Vx; FX29 I=sprite_addr[Vx]
                        0x33 => println!("{:03x}: {:04X} 'LD B, V{:X}'",addr,opcode,x), //LD B, Vx; FX33 set_BCD(Vx)
                        0x55 => println!("{:03x}: {:04X} 'LD [I], V{:X}'",addr,opcode,x), //LD [I], Vx; LD [I], Vx; FX55 reg_dump(Vx, &I)
                        0x65 => println!("{:03x}: {:04X} 'LD V{:X}, [I]'",addr,opcode,x), //LD Vx, [I]; FX65 reg_load(Vx,&I)
                        _ => println!("Invalid opcode: {:04X}", opcode)
                    },
                    _ => println!("Invalid opcode: {:04X}", opcode)
                }
            };
            addr+=2;
            opcode = (memory[addr] as u16) << 8 | (memory[addr + 1] as u16)
        }
        //dfilebuf.flush();
    }

    pub fn emulate_cycle(&mut self, memory: &mut [u8; 0x1000], pixels: &mut [bool; 0x800], key: &[bool; 0x10], device: &AudioDevice<impl AudioCallback>) {
        //Fetch Opcode
        let opcode: u16 = (memory[self.pc as usize] as u16) << 8 | (memory[(self.pc as usize) + 1] as u16);
        self.pc += 2;

        //Decode Opcode
        let u = opcode >> 12; //the first nibble in the instruction
        let nnn = opcode & 0xFFF; //address
        let nn: u8 = (opcode & 0xFF) as u8; //8 bit constant (byte)
        let n = opcode & 0xF; //4 bit constant (last nibble)
        let x: usize = (nnn as usize) >> 8; //the second nibble in the instruction (when applicable)
        let y: usize = (nn as usize) >> 4; // the third nibble in the instruction (when applicable)
        #[allow(non_snake_case)]
        let I: usize = self.I as usize;
        //Execute Opcode
        match opcode {
            0x00E0 => {self.clear_display(pixels); self.draw_flag = true}, //CLS; clears the display
            0x00EE => {self.pc = self.stack[self.sp as usize]; self.sp-=1}, //RET; return from a subroutine
            _ => match u {
                //0x0 => {}, //SYS addr; 0NNN, Calls RCA 1802 program at address NNN. not used for this emulator
                0x1 => {self.pc = nnn}, //JP addr; 1NNN, Goto NNN
                0x2 => {self.sp+=1; self.stack[self.sp as usize]=self.pc; self.pc = nnn}, //CALL addr; 2NNN, Call subroutine at NNN
                0x3 => {if self.reg_v[x]==nn {self.pc+=2}}, //SE Vx, byte; 3XNN, skip if Vx==NN
                0x4 => {if self.reg_v[x]!=nn {self.pc+=2}}, //SNE Vx, byte; 4XNN, skip if Vx!=NN
                0x5 => if n==0 {
                        if self.reg_v[x]==self.reg_v[y] {self.pc+=2} //SE Vx, Vy; 5XY0, skip if Vx==Vy
                    } else {panic!("Invalid opcode: {:04X} at {:04x}", opcode, self.pc-2)},
                0x6 => {self.reg_v[x]=nn}, //LD Vx, byte; 6XNN, Vx = NN (assignment)
                0x7 => {self.reg_v[x]=((self.reg_v[x] as u16)+(nn as u16)) as u8}, //ADD Vx, byte; 7XNN, Vx += NN (addition)
                0x8 => match n { //8XYn
                    0x0 => {self.reg_v[x]=self.reg_v[y]}, //LD Vx, Vy; 8XY0, Vx = Vy
                    0x1 => {self.reg_v[x]|=self.reg_v[y]}, //OR Vx, Vy; 8XY1, Vx = Vx|Vy (OR)
                    0x2 => {self.reg_v[x]&=self.reg_v[y]}, //AND Vx, Vy; 8XY2, Vx = Vx&Vy (AND)
                    0x3 => {self.reg_v[x]^=self.reg_v[y]}, //XOR Vx, Vy; 8XY3, Vx = Vx^Vy (XOR)
                    0x4 => {self.reg_v[0xF]=(((self.reg_v[x] as u16)+(self.reg_v[y] as u16))>>8) as u8; self.reg_v[x]=(((self.reg_v[x] as u16)+(self.reg_v[y] as u16))&0xFF) as u8}, //ADD Vx, Vy; 8XY4, Vx += Vy
                    0x5 => {self.reg_v[0xF]=(self.reg_v[x]>self.reg_v[y]) as u8; self.reg_v[x]=((self.reg_v[x] as i16)-(self.reg_v[y] as i16)) as u8}, //SUB Vx, Vy; 8XY5, Vx -= Vy
                    0x6 => {self.reg_v[0xF]=self.reg_v[x]&0x1; self.reg_v[x]=self.reg_v[x] >> 1}, //SHR Vx; 8XY6, Vx >> 1
                    0x7 => {self.reg_v[0xF]=(self.reg_v[y]>self.reg_v[x]) as u8; self.reg_v[x]=((self.reg_v[y] as i16)-(self.reg_v[x] as i16)) as u8}, //SUBN Vx, Vy; 8XY7, Vx=Vy-Vx
                    0xE => {self.reg_v[0xF]=self.reg_v[x]>>7; self.reg_v[x]=self.reg_v[x] << 1}, //SHL Vx; 8XYE, Vx << 1
                    _ => panic!("Invalid opcode: {:04X} at {:04x}", opcode, self.pc-2)
                },
                0x9 => {if self.reg_v[x]!=self.reg_v[y] {self.pc+=2}}, //SNE Vx, Vy; 9XY0, skip if Vx!=Vy
                0xA => {self.I=nnn}, //LD I, addr; ANNN, I=NNN
                0xB => {self.pc=(self.reg_v[0] as u16)+nnn}, //JP V0, addr; BNNN, PC = V0 + NNN (Error handling could be added)
                0xC => {self.reg_v[x]=rng_byte()&nn}, //RND Vx, byte; CXNN, Vx = rand() & NN
                0xD => {
                    /*The interpreter reads n bytes from memory, starting at the address stored in I.
                    These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
                    Sprites are XORed onto the existing screen.
                    If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
                    If the sprite is positioned so part of it is outside the coordinates of the display,
                    it wraps around to the opposite side of the screen. */
                    self.reg_v[0xF]=0;
                    let mut px: usize = self.reg_v[x] as usize;
                    let mut py: usize = self.reg_v[y] as usize;
                    for i in 0..n {
                        while py>=H {py-=H};
                        for ii in 0..8 {
                            while px>=W {px-=W};
                            self.update_pixel(pixels, px, py, (((memory[I+(i as usize)]>>(7-ii))&1)==1) as bool);
                            px+=1
                        };
                        px=self.reg_v[x] as usize;
                        py+=1
                    };
                    self.draw_flag = true
                }, //DRW Vx, Vy, nibble; DXYN, Display n-byte sprite starting at memory location I at (Vx, Vy)
                0xE => match nn {
                    0x9E => {if key[(self.reg_v[x]&0xF) as usize] {self.pc+=2}}, //SKP Vx; EX9E, Skip next instruction if key with the value of Vx is pressed.
                    0xA1 => {if !key[(self.reg_v[x]&0xF) as usize] {self.pc+=2}}, //SKNP Vx; EXA1, Skip next instruction if key with the value of Vx is not pressed.
                    _ => panic!("Invalid opcode: {:04X} at {:04x}", opcode, self.pc-2)
                },
                0xF => match nn { //FXzz
                    0x07 => {self.reg_v[x]=self.delay_timer}, //LD Vx, DT; FX07 Set Vx = delay timer value.
                    0x0A => {self.key_wait=true; self.reg_wait=x as usize}, //LD Vx, K; FX0A, Wait for a key press, store the value of the key in Vx.
                    0x15 => {self.delay_timer=self.reg_v[x]}, //LD DT, Vx; FX15, Set delay timer = Vx.
                    0x18 => {self.sound_timer=self.reg_v[x]}, //LD ST, Vx; FX18, Set sound timer = Vx.
                    0x1E => {self.reg_v[0xF]=((self.I+(self.reg_v[x] as u16))>>12) as u8; self.I=(self.I+(self.reg_v[x] as u16))&0xFFF}, //ADD I, Vx; FX1E I += Vx
                    0x29 => {self.I=(0x50+(0x5*self.reg_v[x])) as u16}, //LD F, Vx; FX29 I=sprite_addr[Vx]
                    0x33 => {
                        /*The interpreter takes the decimal value of Vx,
                        and places the hundreds digit in memory at location in I,
                        the tens digit at location I+1,
                        and the ones digit at location I+2.*/
                        memory[I] = (self.reg_v[x]/10)/10; //hundreds
                        memory[I+1] = (self.reg_v[x]/10)%10; //tens
                        memory[I+2] = self.reg_v[x]%10 //ones
                    }, //LD B, Vx; FX33 set_BCD(Vx)
                    0x55 => {
                        for i in 0..(x+1) {
                            memory[I+i] = self.reg_v[i]
                        }
                    }, //LD [I], Vx; LD [I], Vx; FX55 reg_dump(Vx, &I)
                    0x65 => {
                        for i in 0..(x+1) {
                            self.reg_v[i] = memory[I+i]
                        }
                    }, //LD Vx, [I]; FX65 reg_load(Vx,&I)
                    _ => panic!("Invalid opcode: {:04X} at {:04x}", opcode, self.pc-2)
                },
                _ => panic!("Invalid opcode: {:04X} at {:04x}", opcode, self.pc-2)
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

fn reset_pixel(pixels: &mut [bool; 0x800], x: usize, y: usize, val: bool) {
    pixels[y*W+x] = val
}

fn rng_byte() -> u8 {
    rand::thread_rng().gen_range(0,256) as u8
}