/*
Memory map:
0x000-0x1FF - Chip 8 interpreter (contains font set in emulator)
0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
0x200-0xFFF - Program ROM and work RAM
*/

use constants::*;
use command::{CommandEmulator, Command, 
    MemoryCommand::SendRAM};
use router::Router;

pub struct Memory {
    ram: [u8; RAM_BYTES],
    commands: Router<Command>
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            ram: [0; RAM_BYTES],
            commands: Router::<Command>::new()
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
}

impl CommandEmulator for Memory {
    fn get_commands(&mut self) -> &mut Router<Command> {
        &mut self.commands
    }

    fn process_inbound_command(&mut self, command: &Command) {
        match command {
            Command::Memory(c) => match c {
                SendRAM(bytes) => self.ram.copy_from_slice(bytes)
            },
            _ => {}
        }
    }

    fn emulate_cycle(&mut self) {
        self.commands.send_outbound(Command::Memory(
            SendRAM(self.ram.clone())));
    }
}
