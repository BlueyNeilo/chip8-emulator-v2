use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::convert::TryInto;

use memory::Memory;
use chip8::Chip8;
use io::IO;


use constants::{N, KEY_VALUES};


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

    pub fn start_game(&mut self, rom: &str) {
        self.chip8.load_game(&mut self.memory.ram, &rom);
        self.chip8.disassemble_code(&mut self.memory.ram);

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

            if !self.chip8.key_wait {
                let mut pixels: [bool; N] = self.io.display.get_pixels().as_slice().try_into().unwrap();
                self.chip8.emulate_cycle(&mut self.memory.ram, &mut pixels, &self.memory.key_state, &self.io.audio_device);
                (self.io.display).update_pixels(&pixels);
            };

            if self.chip8.draw_flag {
                self.io.display.draw_pixels();
                self.chip8.draw_flag = false;
                if self.chip8.clear_display_flag {
                    self.io.display.reset_screen();
                    self.chip8.clear_display_flag = false
                }
            }
        }
    }
}