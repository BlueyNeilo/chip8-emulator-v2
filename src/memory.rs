/*
Memory map:
0x000-0x1FF - Chip 8 interpreter (contains font set in emulator)
0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
0x200-0xFFF - Program ROM and work RAM
*/

use constants::*;

#[derive(Clone)]
pub struct Memory {
    pub ram: [u8; RAM_BYTES]
}

impl Memory {
    pub fn new() -> Self {
        let mut ram: [u8; RAM_BYTES] = [0; RAM_BYTES];
        let mut addr = FONT_ADDR;
        
        for byte in FONT_SPRITES.iter() {
            ram[addr] = *byte;
            addr+=1
        }

        Memory {
            ram: ram
        }
    }
}
