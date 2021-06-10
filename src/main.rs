/*
Chip8 Emulator in Rust 

Patrick Neilson 2021
*/

extern crate sdl2;
extern crate rand;
extern crate byteorder;

mod memory;
mod rom_menu;
mod audio;
mod display;
mod chip8;
mod io;
mod constants;
mod rng;
mod opcode;
mod emulator;
mod command;

use rom_menu::choose_rom;
use emulator::Chip8Emulator;

#[cfg(test)]
mod tests;

fn main() {
    let rom = choose_rom(); //"./ROMs/PONG";
    let mut emulator = Chip8Emulator::new();
    emulator.start_game(&rom)
}
