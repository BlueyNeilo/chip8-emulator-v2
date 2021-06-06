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
use sdl2::EventPump;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{WindowCanvas};
use sdl2::audio::{AudioDevice, AudioCallback};

//RNG
extern crate rand;

//CPU cycling
use std::time::Duration;
use std::thread::sleep;

//Use internal modules
mod memory;
use memory::init_memory;

mod rom_menu;
use rom_menu::choose_rom;

mod audio;
use audio::setup_audio;

mod chip8;
use chip8::{W,H,PIXEL_SIZE,Chip8};

fn setup_io() -> (WindowCanvas, EventPump, AudioDevice<impl AudioCallback>) {
    //window size
    //display mode
    //audio settings
    //etc
    let sdl_context = sdl2::init().unwrap(); //Sdl type
    let video_subsystem = sdl_context.video().unwrap(); //VideoSubsystem type
    let window = video_subsystem
        .window("Chip8 Emulator",
            PIXEL_SIZE*(W as u32),
            PIXEL_SIZE*(H as u32))
        .position_centered()
        .build().unwrap(); //Window type
    let mut canvas = window.into_canvas()
        .target_texture()
        .present_vsync()
        .build().unwrap(); //WindowCanvas
    //println!("Using SDL_Renderer \"{}\"", canvas.info().name);
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let event_pump = sdl_context.event_pump().unwrap(); //EventPump
    let audio_device = setup_audio(&sdl_context);

    (canvas, event_pump, audio_device)
}

fn draw_graphics(pixels: &[bool; 0x800], canvas: &mut WindowCanvas) {
    canvas.set_draw_color(Color::RGB(0,0,0)); //black
    canvas.clear();
    canvas.set_draw_color(Color::RGB(255,255,255)); //white
    for (i,v) in pixels.into_iter().enumerate() {
        if *v {
            canvas.fill_rect(Rect::new(((i as i32)%(W as i32))*(PIXEL_SIZE as i32), (i as i32)/(W as i32)*(PIXEL_SIZE as i32), PIXEL_SIZE, PIXEL_SIZE)).unwrap()
        }
    }
    canvas.present()
}

fn main() {
    let rom = choose_rom(); //String::from("./ROMs/PONG"); //let rom: String = "./pong.bin".to_string();
    let mut memory: [u8; 0x1000] = [0; 0x1000]; //4kB program ROM and work RAM
    let mut pixels: [bool; 0x800] = [false; 0x800]; //64x32 pixels
    let mut key: [bool; 0x10] = [false; 0x10]; //0x0-0xF
    let mut kvals: [Keycode; 0x10] = [Keycode::A; 0x10]; //Keyboard input configuration
    init_memory(&mut memory, &mut kvals);
    let kvals = kvals; //remove mutablity
    let (mut canvas, mut event_pump, device) = setup_io();
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

        sleep(Duration::new(0, 1666667)); //16666667ns period for a 60Hz cycle
        if !chip8.key_wait {
            chip8.emulate_cycle(&mut memory, &mut pixels, &key, &device)
        };
        if chip8.draw_flag {
            draw_graphics(&pixels, &mut canvas);
            chip8.draw_flag = false
        }
    }
}
