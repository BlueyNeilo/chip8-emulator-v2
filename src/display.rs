use sdl2::Sdl;
use sdl2::render::WindowCanvas;
use sdl2::rect::Rect;
use sdl2::pixels::Color;

use chip8::{W,H,PIXEL_SIZE};

pub fn setup_display(sdl_context: &Sdl) -> WindowCanvas {
    let video_subsystem = sdl_context.video().unwrap();

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

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    
    canvas
}

pub fn draw_graphics(pixels: &[bool; 0x800], canvas: &mut WindowCanvas) {
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