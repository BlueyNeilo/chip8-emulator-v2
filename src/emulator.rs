use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::convert::TryInto;

use memory::Memory;
use chip8::Chip8;
use io::IO;


use constants::{N, KEY_VALUES};


pub struct Emulator;

impl Emulator {
    pub fn new() -> Self {
        Emulator {}
    }

    pub fn start_game(&self, rom: String) {
        let io = IO::new();
        let memory = Memory::new();
        let Memory {mut ram, mut key_state} = memory;
        let IO {mut display, mut event_pump, audio_device } = io;
        let mut chip8 = Chip8::new();
        chip8.load_game(&mut ram, rom);
        chip8.disassemble_code(&ram);

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    Event::KeyDown { keycode: Some(k), repeat: false, .. } => {
                        for (i, kd) in KEY_VALUES.iter().enumerate() {
                            if k==*kd {
                                key_state[i as usize] = true;
                                if chip8.key_wait {
                                    chip8.key_wait=false;
                                    chip8.V[chip8.reg_wait]=i as u8
                                }
                                //println!("Key 0x{:X} down",i)
                            }
                        }
                    },
                    Event::KeyUp { keycode: Some(k), repeat: false, .. } => {
                        for (i, kd) in KEY_VALUES.iter().enumerate() {
                            if k==*kd {
                                key_state[i as usize] = false;
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
                chip8.emulate_cycle(&mut ram, &mut pixels, &key_state, &audio_device);
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
}