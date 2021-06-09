/*
Chip8 Emulator in Rust - Patrick Neilson 2018
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
