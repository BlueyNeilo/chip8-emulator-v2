use std::time::Duration;
use std::thread::sleep;
use sdl2::render::WindowCanvas;
use sdl2::audio::AudioDevice;
use sdl2::EventPump;

use display::{setup_display,draw_graphics};
use audio::{setup_square_audio,SquareWave};


const SCREEN_FPS: u32 = 10;
const FRAME_CYCLE: u32 = 120;
const NANO_UNIT: u32 = i64::pow(10,9) as u32;

pub struct IO {
    pub canvas: WindowCanvas,
    pub event_pump: EventPump, 
    pub audio_device: AudioDevice<SquareWave>
}

impl IO {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        IO {
            canvas: setup_display(&sdl_context),
            event_pump: sdl_context.event_pump().unwrap(),
            audio_device: setup_square_audio(&sdl_context)
        }
    }

    pub fn sleep_frame() {
        sleep(Duration::new(0, NANO_UNIT / SCREEN_FPS / FRAME_CYCLE));
    }
}
