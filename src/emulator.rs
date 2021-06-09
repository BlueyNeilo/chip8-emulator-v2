use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use byteorder::{ByteOrder, BigEndian};

use std::convert::TryInto;
use std::fs::File;
use std::io::Read;

use memory::Memory;
use chip8::Chip8;
use io::IO;
use constants::{N, KEY_VALUES, ROM_ADDR};
use opcode::OpcodeDisassembler;
use command::{Command::{self}, CommandInterpreter};

pub struct Emulator {
    io: IO,
    memory: Memory,
    chip8: Chip8
}

impl Emulator {
    pub fn new() -> Self {
        Emulator {
            io: IO::new(),
            memory: Memory::new(),
            chip8: Chip8::new()
        }
    }

    pub fn start_game(&mut self, rom_path: &str) {
        let rom_bytes = self.get_rom_bytes(rom_path);
        self.load_rom(&rom_bytes);
        self.disassemble_code(&rom_bytes);

        'running: loop {
            for event in self.io.event_pump.poll_iter() {
                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    Event::KeyDown { keycode: Some(k), repeat: false, .. } => {
                        for (i, kd) in KEY_VALUES.iter().enumerate() {
                            if k==*kd {
                                self.memory.key_state[i as usize] = true;
                                if self.chip8.key_wait {
                                    self.chip8.key_wait=false;
                                    self.chip8.V[self.chip8.reg_wait]=i as u8
                                }
                            }
                        }
                    },
                    Event::KeyUp { keycode: Some(k), repeat: false, .. } => {
                        for (i, kd) in KEY_VALUES.iter().enumerate() {
                            if k==*kd {
                                self.memory.key_state[i as usize] = false;
                            }
                        }
                    },
                    _ => {}
                }
            };

            IO::sleep_frame();

            self.io.read_commands();
            self.io.emulate_cycle();
            self.io.commands.output_stack.pop_all().into_iter().for_each(|c|
                match c {
                    Command::Display(_)
                    | Command::Audio(_) => self.chip8.commands.input_stack.push(c),
                    _ => {}
                });
            
            self.chip8.read_commands();
            self.chip8.emulate_cycle(&mut self.memory.ram, &self.memory.key_state);
            self.chip8.commands.output_stack.pop_all().into_iter().for_each(|c|
                match c {
                    Command::Display(_)
                    | Command::Audio(_) => self.io.commands.input_stack.push(c),
                    _ => {}
                });
        }
    }

    pub fn get_rom_bytes(&mut self, rom_path: &str) -> Vec<u8> {
        let mut rom_buf: Vec<u8> = Vec::new();
        let mut file = File::open(&rom_path).unwrap();
        file.read_to_end(&mut rom_buf).unwrap();
        
        rom_buf
    }

    pub fn load_rom(&mut self, rom_bytes: &Vec<u8>) {
        let mut addr = ROM_ADDR;
        for byte in rom_bytes {
            self.memory.ram[addr]=*byte;
            addr+=1
        }
    }

    pub fn disassemble_code(&mut self, rom_bytes: &Vec<u8>) {
        println!("Disassembling code: \n");

        (0..rom_bytes.len()/2)
            .map(|i| (i, BigEndian::read_u16(&rom_bytes[i*2..i*2+2])))
            .for_each(|(i, instruction)| {
                println!("{:03x}: {:04X} '{}'", 
                    i + ROM_ADDR, 
                    instruction,
                    OpcodeDisassembler::disassemble(instruction))
            })
    }
}
