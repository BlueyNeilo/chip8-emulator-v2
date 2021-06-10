
use byteorder::{ByteOrder, BigEndian};

use std::fs::File;
use std::io::Read;

use memory::Memory;
use chip8::Chip8;
use io::IO;
use constants::{ROM_ADDR};
use opcode::OpcodeDisassembler;
use command::{Command::{self}, CommandInterpreter, GameCommand::*};

pub struct Chip8Emulator {
    io: IO,
    memory: Memory,
    chip8: Chip8,
    running_flag: bool,
}

impl Chip8Emulator {
    pub fn new() -> Self {
        Chip8Emulator {
            io: IO::new(),
            memory: Memory::new(),
            chip8: Chip8::new(),
            running_flag: true,
        }
    }

    pub fn start_game(&mut self, rom_path: &str) {
        let rom_bytes = self.get_rom_bytes(rom_path);
        self.memory.load_font_sprites();
        self.memory.load_rom(&rom_bytes);
        self.disassemble_code(&rom_bytes);

        while self.running_flag {
            self.memory.read_commands();
            self.memory.emulate_cycle();
            self.memory.commands.outbound_queue.remove_all().into_iter().for_each(|c| 
                match c {
                    Command::Memory(_) => self.chip8.commands.inbound_queue.push(c),
                    _ => {}
                });

            self.io.read_commands();
            self.io.emulate_cycle();
            self.io.commands.outbound_queue.remove_all().into_iter().for_each(|c|
                match c {
                    Command::Display(_)
                    | Command::Audio(_)
                    | Command::Key(_) => self.chip8.commands.inbound_queue.push(c),
                    Command::GameState(Exit) => self.running_flag = false,
                    _ => {}
                });
            
            self.chip8.read_commands();
            self.chip8.emulate_cycle();
            self.chip8.commands.outbound_queue.remove_all().into_iter().for_each(|c|
                match c {
                    Command::Display(_)
                    | Command::Audio(_) => self.io.commands.inbound_queue.push(c),
                    Command::Memory(_) => self.memory.commands.inbound_queue.push(c),
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
