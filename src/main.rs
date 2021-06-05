/*
Chip8 Emulator in Rust - Patrick Neilson 2018

//http://www.multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/
//http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
*/
/*
Memory map:
0x000-0x1FF - Chip 8 interpreter (contains font set in emulator)
0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
0x200-0xFFF - Program ROM and work RAM
*/
/*
The chip 8 has one instruction that draws sprite to the screen.
Drawing is done in XOR mode and
if a pixel is turned off as a result of drawing, the VF register is set.
This is used for collision detection.
*/
/*
Graphics
64x32 pixels
monochrome colour
graphics are drawn only with sprites
(sprites are 8 pixels wide, may be from 1 to 15 pixels in height)
*/
/*
drawFlag
Because the system does not draw every cycle,
we should set a draw flag when we need to update our screen.
Only two opcodes should set this flag:
    0x00E0 – Clears the screen
    0xDXYN – Draws a sprite on the screen
*/

//SDL
extern crate sdl2;
use sdl2::EventPump;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::{Window, WindowContext};
use sdl2::render::{Canvas, WindowCanvas};
use sdl2::audio::{AudioDevice, AudioCallback, AudioSpecDesired, AudioStatus};
//RNG for rand_byte() instruction
extern crate rand;
use rand::Rng;
//CPU cycling
use std::time::Duration;
use std::thread::sleep;
//ROM file access
use std::fs::{File, read_dir};
use std::ffi::OsStr;
use std::path::Path;
use std::io::{self, Read, Write, BufWriter};

//Pixel dimensions
const W: usize = 64;
const H: usize = 32;
const pixel_size: u32 = 20;

fn chooseroms() -> String {
    let dir = Path::new("./ROMs/");
    let mut rom = String::new();
    println!("\nAvailable ROMS:\n");
    if dir.is_dir() {
        for entry in read_dir(&dir).unwrap() {
            println!("{}", entry.unwrap().path().file_name().unwrap().to_str().unwrap())
        }
        'getrom: loop {
            println!("\nPlease chose a ROM to play:");
            io::stdin().read_line(&mut rom)
                .expect("Failed to read line");
            let tlen: usize = rom.len()-2;
            rom.truncate(tlen); //Remove '\n\r' at the end.
            if dir.join(&rom).as_path().exists() {
                break 'getrom
            }
            else
            {
                println!("Sorry that ROM does not exist.");
                rom=String::new() //clear string for next read attempt
            }
        }
    }
    else
    {
        panic!("ROM directory doesn't exist.")
    }
    dir.join(rom).as_path().to_str().unwrap().to_string()
}

//Borrowed from API reference http://angrylawyer.github.io/rust-sdl2/sdl2/audio/index.html
struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}
impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = match self.phase {
                0.0...0.5 => self.volume,
                _ => -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

fn setupIO() -> (WindowCanvas, EventPump, AudioDevice<SquareWave>) {
    //window size
    //display mode
    //audio settings
    //etc
    let sdl_context = sdl2::init().unwrap(); //Sdl type
    let video_subsystem = sdl_context.video().unwrap(); //VideoSubsystem type
    let window = video_subsystem
        .window("Chip8 Emulator",
                pixel_size*(W as u32),
                pixel_size*(H as u32))
        .position_centered()
        .build().unwrap(); //Window type
    let mut canvas = window.into_canvas()
        .target_texture()
        .present_vsync()
        .build().unwrap(); //WindowCanvas
    //println!("Using SDL_Renderer \"{}\"", canvas.info().name);
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap(); //EventPump
    //261.63Hz Middle C
    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),  // mono
        samples: None       // default sample size
    };
    let device = audio_subsystem.open_playback(None, &desired_spec,
        |spec| {
            SquareWave {
                phase_inc: 261.63 / spec.freq as f32, //261.63Hz Middle C (440Hz A)
                phase: 0.0,
                volume: 0.15
            }
        }).unwrap();
    device.pause();

    (canvas, event_pump, device)
}

fn drawGraphics(pixels: &[bool; 0x800], canvas: &mut WindowCanvas) {
    canvas.set_draw_color(Color::RGB(0,0,0)); //black
    canvas.clear();
    canvas.set_draw_color(Color::RGB(255,255,255)); //white
    for (i,v) in pixels.into_iter().enumerate() {
        if *v {
            canvas.fill_rect(Rect::new(((i as i32)%(W as i32))*(pixel_size as i32), (i as i32)/(W as i32)*(pixel_size as i32), pixel_size, pixel_size)).unwrap()
        }
    }
    canvas.present()
}

fn resetPixel(pixels: &mut [bool; 0x800], x: usize, y: usize, val: bool) {
    pixels[y*W+x] = val
}

fn RNG_byte() -> u8 {
    rand::thread_rng().gen_range(0,256) as u8
}

struct Chip8 {
    drawFlag: bool,
    key_wait: bool, //set to true if the program is waiting for a key to be entered
    reg_wait: usize, //the index of the V register the waited key value will be stored in
    pc: u16, //Program counter
    I: u16, //Index register
    sp: u8, //Stack pointer
    stack: [u16; 0x10], //The stack for return calls
    reg_v: [u8; 0x10], //16 general purpose registers V0..V15. V15 (VF) is used for the carry flag
    delay_timer: u8, //counts down to 0 (60Hz)
    sound_timer: u8 //counts down to 0 (60Hz). system's buzzer sounds whenever the timer reaches 0.
}
impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {drawFlag: false,
        key_wait: false,
        reg_wait: 0,
        pc: 0x200, //program starts at 0x200
        I: 0,
        sp: 0,
        stack: [0; 0x10],
        reg_v: [0; 0x10],
        delay_timer: 0,
        sound_timer: 0}
    }

    fn loadGame(&mut self, memory:  &mut [u8; 0x1000], gameName: String) {
        let mut rom_buf: Vec<u8> = Vec::new();
        let mut file = File::open(&gameName).unwrap();
        file.read_to_end(&mut rom_buf).unwrap();
        let mut addr = 0x200;
        for byte in &rom_buf {
            memory[addr]=*byte;
            addr+=1
        }
    }

    fn updatePixel(&mut self, pixels: &mut [bool; 0x800], x: usize, y: usize, val: bool) {
        if pixels[y*W+x]==val {self.reg_v[0xF]=1};
        pixels[y*W+x] ^= val
    }

    //Clears the display. All pixels set to black (off)
    fn clearDisplay(&mut self, pixels: &mut [bool; 0x800]) {
        for y in 0..H {
            for x in 0..W {
                resetPixel(pixels, x as usize, y as usize, false)
            }
        }
    }

    fn disassembleCode(&mut self, memory: &[u8; 0x1000]) {
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

    fn emulateCycle(&mut self, memory: &mut [u8; 0x1000], pixels: &mut [bool; 0x800], key: &[bool; 0x10], device: &AudioDevice<SquareWave>) {
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
        let I: usize = self.I as usize;
        //Execute Opcode
        match opcode {
            0x00E0 => {self.clearDisplay(pixels); self.drawFlag = true}, //CLS; clears the display
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
                0xC => {self.reg_v[x]=RNG_byte()&nn}, //RND Vx, byte; CXNN, Vx = rand() & NN
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
                            self.updatePixel(pixels, px, py, (((memory[I+(i as usize)]>>(7-ii))&1)==1) as bool);
                            px+=1
                        };
                        px=self.reg_v[x] as usize;
                        py+=1
                    };
                    self.drawFlag = true
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

fn initMemory(memory: &mut [u8; 0x1000], kvals: &mut [Keycode; 0x10]) {
    let font: [u8; 16*5] = [0xF0,0x90,0x90,0x90,0xF0, //0
    0x20,0x60,0x20,0x20,0x70, //1
    0xF0,0x10,0xF0,0x80,0xF0, //2
    0xF0,0x10,0xF0,0x10,0xF0, //3
    0x90,0x90,0xF0,0x10,0x10, //4
    0xF0,0x80,0xF0,0x10,0xF0, //5
    0xF0,0x80,0xF0,0x90,0xF0, //6
    0xF0,0x10,0x20,0x40,0x40, //7
    0xF0,0x90,0xF0,0x90,0xF0, //8
    0xF0,0x90,0xF0,0x10,0xF0, //9
    0xF0,0x90,0xF0,0x90,0x90, //A
    0xE0,0x90,0xE0,0x90,0xE0, //B
    0xF0,0x80,0x80,0x80,0xF0, //C
    0xE0,0x90,0x90,0x90,0xE0, //D
    0xF0,0x80,0xF0,0x80,0xF0, //E
    0xF0,0x80,0xF0,0x80,0x80]; //F
    *kvals = [Keycode::X,
        Keycode::Num1,
        Keycode::Num2,
        Keycode::Num3,
        Keycode::Q,
        Keycode::W,
        Keycode::E,
        Keycode::A,
        Keycode::S,
        Keycode::D,
        Keycode::Z,
        Keycode::C,
        Keycode::Num4,
        Keycode::R,
        Keycode::F,
        Keycode::V]; //1234QWERASDFZXCV hex keypad
    let mut addr = 0x50;
    for byte in font.iter() {
        memory[addr] = *byte;
        addr+=1
    }
}

fn main() {
    let rom = chooseroms(); //String::from("./ROMs/PONG"); //let rom: String = "./pong.bin".to_string();
    let mut memory: [u8; 0x1000] = [0; 0x1000]; //4kB program ROM and work RAM
    let mut pixels: [bool; 0x800] = [false; 0x800]; //64x32 pixels
    let mut key: [bool; 0x10] = [false; 0x10]; //0x0-0xF
    let mut kvals: [Keycode; 0x10] = [Keycode::A; 0x10]; //Keyboard input configuration
    initMemory(&mut memory, &mut kvals);
    let kvals = kvals; //remove mutablity
    let (mut canvas, mut event_pump, device) = setupIO();
    let mut chip8 = Chip8::new();
    chip8.loadGame(&mut memory, rom);
    chip8.disassembleCode(&memory);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(k), repeat: false, .. } => {
                    for (i, kd) in kvals.into_iter().enumerate() {
                        if k==*kd {
                            key[i as usize] = true;
                            if chip8.key_wait {
                                chip8.key_wait=false;
                                chip8.reg_v[chip8.reg_wait]=i as u8
                            }
                            //println!("Key 0x{:X} down",i)
                        }
                    }
                },
                Event::KeyUp { keycode: Some(k), repeat: false, .. } => {
                    for (i, kd) in kvals.into_iter().enumerate() {
                        if k==*kd {
                            key[i as usize] = false;
                            //println!("Key 0x{:X} up",i)
                        }
                    }
                },
                _ => {}
            }
        };

        sleep(Duration::new(0, 1666667)); //16666667ns period for a 60Hz cycle
        if !chip8.key_wait {
            chip8.emulateCycle(&mut memory, &mut pixels, &key, &device)
        };
        if chip8.drawFlag {
            drawGraphics(&pixels, &mut canvas);
            chip8.drawFlag = false
        }
    }
}

/*
PONG:
6A 02 6B 0C 6C 3F 6D 0C C2 A2 C3 AA C3 9A C2 B6 C3 9C C3 96 6E 20 22 C3 94 66 03 68 02 60 60 C3 B0 15 C3 B0 07 30 20 12 1A C3 87 17 77 08 69 C3 BF C2 A2 C3 B0 C3 96 71 C2 A2 C3 AA C3 9A C2 B6 C3 9C C3 96 60 01 C3 A0 C2 A1 7B C3 BE 60 04 C3 A0 C2 A1 7B 02 60 1F E2 80 B9 02 C3 9A C2 B6 60 0C C3 A0 C2 A1 7D C3 BE 60 0A C3 A0 C2 A1 7D 02 60 1F C2 8D 02 C3 9C C3 96 C2 A2 C3 B0 C3 96 71 E2 80 A0 E2 80 9E E2 80 A1 E2 80 9D 60 3F E2 80 A0 02 61 1F E2 80 A1 12 46 02 12 78 46 3F 12 E2 80 9A 47 1F 69 C3 BF 47 20 69 01 C3 96 71 12 2A 68 02 63 01 E2 82 AC 70 E2 82 AC C2 B5 12 C5 A0 68 C3 BE 63 0A E2 82 AC 70 E2 82 AC C3 95 3F 01 12 C2 A2 61 02 E2 82 AC 15 3F 01 12 C2 BA E2 82 AC 15 3F 01 12 C3 88 E2 82 AC 15 3F 01 12 C3 82 60 20 C3 B0 18 22 C3 94 C5 BD 34 22 C3 94 66 3E 33 01 66 03 68 C3 BE 33 01 68 02 12 16 79 C3 BF 49 C3 BE 69 C3 BF 12 C3 88 79 01 49 02 69 01 60 04 C3 B0 18 76 01 46 40 76 C3 BE 12 6C C2 A2 C3 B2 C3 BE 33 C3 B2 65 C3 B1 29 64 14 65 20 C3 94 55 74 15 C3 B2 29 C3 94 55 20 C3 AE E2 82 AC E2 82 AC E2 82 AC E2 82 AC E2 82 AC E2 82 AC E2 82 AC 20 20 20 20 20
*/
