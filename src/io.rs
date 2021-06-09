use std::time::Duration;
use std::thread::sleep;
use sdl2::audio::AudioDevice;
use sdl2::EventPump;

use display::{Display, WindowDisplay};
use audio::{setup_square_audio,SquareWave};
use constants::{W, H, N, PIXEL_SIZE};
use command::{CommandInterface, CommandInterpreter, Command, DisplayCommand};

const SCREEN_FPS: u32 = 10;
const FRAME_CYCLE: u32 = 120;
const NANO_UNIT: u32 = i64::pow(10,9) as u32;

pub struct IO {
    pub display: Box<dyn Display<bool>>,
    pub event_pump: EventPump, 
    pub audio_device: AudioDevice<SquareWave>,
    pub commands: CommandInterface
}

impl IO {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let display = WindowDisplay::<W,H,N,PIXEL_SIZE>::new(&sdl_context);
        IO {
            display: Box::new(display),
            event_pump: sdl_context.event_pump().unwrap(),
            audio_device: setup_square_audio(&sdl_context),
            commands: CommandInterface::new()
        }
    }

    pub fn sleep_frame() {
        sleep(Duration::new(0, NANO_UNIT / SCREEN_FPS / FRAME_CYCLE));
    }
}

impl CommandInterpreter for IO {
    fn read_commands(&mut self) {
        self.commands.input_stack.pop_all().iter().for_each(|c| 
            match c {
                Command::Display(c) => match c {
                    DisplayCommand::SendClearDisplay => self.display.reset_screen(),
                    DisplayCommand::SendDraw => self.display.draw_pixels(),
                    DisplayCommand::SendPixels(p) => self.display.update_pixels(p)
                }
                _ => {}
            })
    }
}
