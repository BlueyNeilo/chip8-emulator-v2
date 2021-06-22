
use byteorder::{ByteOrder, BigEndian};

use std::fs::File;
use std::io::Read;

use memory::Memory;
use chip8::Chip8;
use io::IO;
use constants::{ROM_ADDR};
use opcode::OpcodeDisassembler;
use command::{Command::{self}, CommandEmulator, GameCommand::*};
use router::Router;

pub struct Chip8Emulator {
    io: IO,
    memory: Memory,
    chip8: Chip8,
    running_flag: bool,
    router_bridge: Router<Command>
}

impl Chip8Emulator {
    pub fn new() -> Self {
        Chip8Emulator {
            io: IO::new(),
            memory: Memory::new(),
            chip8: Chip8::new(),
            running_flag: true,
            router_bridge: Router::new(),
        }
    }

    pub fn start_game(&mut self, rom_path: &str) {
        let rom_bytes = self.get_rom_bytes(rom_path);
        self.memory.load_font_sprites();
        self.memory.load_rom(&rom_bytes);
        self.disassemble_code(&rom_bytes);

        while self.running_flag {            
            {
                let Chip8Emulator {memory, router_bridge, ..} = self;
                Chip8Emulator::simulate_component(
                    Box::new(memory), 
                    Box::new(router_bridge), 
                    false);
            }
            self.route_to_components();

            {
                let Chip8Emulator {io, router_bridge, ..} = self;
                Chip8Emulator::simulate_component(
                    Box::new(io), 
                    Box::new(router_bridge), 
                    false);
            }
            self.route_to_components();

            {
                let Chip8Emulator {chip8, router_bridge, ..} = self;
                Chip8Emulator::simulate_component(
                    Box::new(chip8), 
                    Box::new(router_bridge), 
                    true);
            }
            self.route_to_components();
        }
    }

    fn simulate_component<'a>(
        component: Box<&'a mut dyn CommandEmulator>, 
        router_bridge: Box<&'a mut Router<Command>>, 
        is_chip8_routing: bool) 
    {
        component.process_inbound_commands();
        component.emulate_cycle();

        if is_chip8_routing {
            (*component).get_commands().forward_outbound(*router_bridge)
        } else {        
            (*component).get_commands().forward_inbound(*router_bridge)
        }
    }

    fn route_to_components(&mut self) {
        self.router_bridge.consume_all_inbound().into_iter().for_each(|c| {
            match c {
                Command::GameState(Exit) => self.exit_game(),
                _ => self.chip8.get_commands().send_inbound(c)
            }
        });

        self.router_bridge.consume_all_outbound().into_iter().for_each(|c| {
            match c {
                Command::GameState(Exit) => self.exit_game(),
                Command::Display(_)
                | Command::Audio(_)
                | Command::Key(_) => self.io.get_commands().send_inbound(c),
                Command::Memory(_) => self.memory.get_commands().send_inbound(c)
            }
        });
    }

    fn exit_game(&mut self) {
        self.running_flag = false
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
