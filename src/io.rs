use std::time::Duration;
use std::thread::sleep;
use sdl2::audio::{AudioDevice, AudioStatus};
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::convert::TryInto;

use display::{Display, WindowDisplay};
use audio::{setup_square_audio, SquareWave};
use constants::{W, H, N, PIXEL_SIZE, KEY_VALUES};
use command::{CommandEmulator, Command, 
    DisplayCommand::{*, self}, AudioCommand, KeyCommand::*, GameCommand::Exit};
use router::Router;

const SCREEN_FPS: u32 = 10;
const FRAME_CYCLE: u32 = 120;
const NANO_UNIT: u32 = i64::pow(10,9) as u32;

pub struct IO {
    display: Box<dyn Display<bool>>,
    event_pump: EventPump, 
    audio_device: AudioDevice<SquareWave>,
    commands: Router<Command>
}

impl IO {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let display = WindowDisplay::<W,H,N,PIXEL_SIZE>::new(&sdl_context);
        IO {
            display: Box::new(display),
            event_pump: sdl_context.event_pump().unwrap(),
            audio_device: setup_square_audio(&sdl_context),
            commands: Router::<Command>::new()
        }
    }

    pub fn poll_event_pump(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit {..} 
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    self.commands.send_outbound(Command::GameState(Exit))
                },
                Event::KeyDown { keycode: Some(key), repeat: false, .. } => {
                    if let Some(key_i) = IO::get_key_index(key) {
                        self.commands.send_outbound(
                            Command::Key(KeyDownUp(key_i, true)))
                    }
                },
                Event::KeyUp { keycode: Some(key), repeat: false, .. } => {
                    if let Some(key_i) = IO::get_key_index(key) {
                        self.commands.send_outbound(
                            Command::Key(KeyDownUp(key_i, false)))
                    }
                },
                _ => {}
            }
        };
    }

    fn sleep_frame() {
        sleep(Duration::new(0, NANO_UNIT / SCREEN_FPS / FRAME_CYCLE));
    }

    fn get_key_index(key: Keycode) -> Option<usize> {
        for (i, key_lookup) in KEY_VALUES.iter().enumerate() {
            if key==*key_lookup {
                return Some(i)
            }
        }
        None
    }
}

impl CommandEmulator for IO {
    fn get_commands(&mut self) -> &mut Router<Command> {
        &mut self.commands
    }

    fn process_inbound_command(&mut self, command: &Command) {
        match command {
            Command::Display(c) => match c {
                DisplayCommand::SendClearDisplay => self.display.reset_screen(),
                DisplayCommand::SendDraw => self.display.draw_pixels(),
                DisplayCommand::SendPixels(p) => self.display.update_pixels(p)
            },
            Command::Audio(c) => match c {
                AudioCommand::Play => {
                    if self.audio_device.status()==AudioStatus::Paused {
                        self.audio_device.resume();
                    }
                },
                AudioCommand::Pause => {
                    if self.audio_device.status()==AudioStatus::Playing {
                        self.audio_device.pause();
                    }
                }
            }
            _ => {}
        }
    }

    fn emulate_cycle(&mut self) {
        self.commands.send_outbound(Command::Display(
            SendPixels(self.display.get_pixels().try_into().unwrap())));
    
        self.poll_event_pump();

        IO::sleep_frame();
    }
}
