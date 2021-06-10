use sdl2::audio::{AudioDevice, AudioSpecDesired, AudioSpec, AudioCallback};
use sdl2::Sdl;

const DSP_FREQ: i32 = 44100;
const MIDDLE_C_FREQ: f32 = 261.63;
const VOLUME: f32 = 0.02;
const MONO_CHANNEL: u8 = 1;

pub struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase < 0.5 { self.volume } else { -self.volume };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub fn setup_square_audio(sdl_context: &Sdl) -> AudioDevice<SquareWave> {
    setup_audio(sdl_context, 
        |spec| SquareWave {
            phase_inc: MIDDLE_C_FREQ / spec.freq as f32,
            phase: 0.0,
            volume: VOLUME
        }
    )
}

fn setup_audio<T: AudioCallback, F: FnOnce(AudioSpec) -> T>(
        sdl_context: &Sdl, callback: F) -> AudioDevice<T> {
    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(DSP_FREQ),
        channels: Some(MONO_CHANNEL),
        samples: None
    };
    
    let audio_device = audio_subsystem.open_playback(
        None, 
        &desired_spec,
        callback
    ).unwrap();
    audio_device.pause();

    audio_device
}
