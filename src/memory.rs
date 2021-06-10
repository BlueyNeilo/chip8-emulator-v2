/*
Memory map:
0x000-0x1FF - Chip 8 interpreter (contains font set in emulator)
0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
0x200-0xFFF - Program ROM and work RAM
*/

use constants::*;
use command::{CommandInterface, CommandInterpreter, Command, 
    MemoryCommand::SendRAM};

pub struct Memory {
    pub ram: [u8; RAM_BYTES],
    pub commands: CommandInterface
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            ram: [0; RAM_BYTES],
            commands: CommandInterface::new()
        }
    }

    pub fn load_font_sprites(&mut self) {
        self.load_bytes_from(FONT_ADDR, &FONT_SPRITES)
    }

    pub fn load_rom(&mut self, rom_bytes: &[u8]) {
        self.load_bytes_from(ROM_ADDR, rom_bytes);
    }

    fn load_bytes_from(&mut self, start_addr: usize, bytes: &[u8]) {
        let mut addr = start_addr;
        for byte in bytes {
            self.ram[addr] = *byte;
            addr += 1
        }
    }

    pub fn emulate_cycle(&mut self) {
        self.commands.output_stack.push(Command::Memory(
            SendRAM(self.ram.clone())));
    }

}

impl CommandInterpreter for Memory {
    fn read_commands(&mut self) {
        self.commands.input_stack.pop_all().iter().for_each(|c| 
            match c {
                Command::Memory(c) => match c {
                    SendRAM(bytes) => self.ram.copy_from_slice(bytes)
                },
                _ => {}
            })
    }
}
