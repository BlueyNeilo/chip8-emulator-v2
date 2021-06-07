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

//SDL
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

//RNG
extern crate rand;

use std::convert::TryInto;

//Use internal modules
mod memory;
use memory::init_memory;

mod rom_menu;
use rom_menu::choose_rom;

mod audio;

mod display;

mod chip8;
use chip8::Chip8;

mod io;
use io::IO;

mod constants;
use constants::N;

mod rng;

mod opcode;

#[cfg(test)]
mod tests;

fn main() {
    let rom = choose_rom(); //String::from("./ROMs/PONG"); //let rom: String = "./pong.bin".to_string();
    let io = IO::new();
    let mut memory: [u8; 0x1000] = [0; 0x1000]; //4kB program ROM and work RAM
    let mut key: [bool; 0x10] = [false; 0x10]; //0x0-0xF
    let mut kvals: [Keycode; 0x10] = [Keycode::A; 0x10]; //Keyboard input configuration
    init_memory(&mut memory, &mut kvals);
    let kvals = kvals; //remove mutablity
    let (mut display, mut event_pump, device) = (io.display, io.event_pump, io.audio_device);
    let mut chip8 = Chip8::new();
    chip8.load_game(&mut memory, rom);
    chip8.disassemble_code(&memory);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(k), repeat: false, .. } => {
                    for (i, kd) in kvals.iter().enumerate() {
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
                    for (i, kd) in kvals.iter().enumerate() {
                        if k==*kd {
                            key[i as usize] = false;
                            //println!("Key 0x{:X} up",i)
                        }
                    }
                },
                _ => {}
            }
        };

        IO::sleep_frame();

        if !chip8.key_wait {
            let mut pixels: [bool; N] = (*display).get_pixels().as_slice().try_into().unwrap();
            chip8.emulate_cycle(&mut memory, &mut pixels, &key, &device);
            (*display).update_pixels(&pixels);
        };

        if chip8.draw_flag {
            (*display).draw_pixels();
            chip8.draw_flag = false;
            if chip8.clear_display_flag {
                (*display).reset_screen();
                chip8.clear_display_flag = false
            }
        }
    }
}
