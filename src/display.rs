use sdl2::Sdl;
use sdl2::render::WindowCanvas;
use sdl2::rect::Rect;

use constants::{BLACK, WHITE};

pub trait Display<T> {
    fn draw_pixels(&mut self);
    fn reset_screen(&mut self);
    fn update_pixels(&mut self, &[T]);
    fn get_pixels(&self) -> &[T];
}

pub struct WindowDisplay<const W: usize, 
    const H: usize,
    const N: usize, 
    const PIXEL_SIZE: u32> 
{
    canvas: WindowCanvas,
    pub pixels: [bool; N]
}

impl <const W: usize, const H: usize, const N: usize, const PIXEL_SIZE: u32> 
WindowDisplay<W, H, N, PIXEL_SIZE> 
{
    pub fn new(sdl_context: &Sdl) -> Self {
        assert_eq!(W * H, N);

        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("Chip8 Emulator",
                PIXEL_SIZE*(W as u32),
                PIXEL_SIZE*(H as u32))
            .position_centered()
            .build().unwrap();
        
        let mut canvas = window.into_canvas()
            .target_texture()
            .present_vsync()
            .build().unwrap(); //WindowCanvas
    
        canvas.set_draw_color(BLACK);
        canvas.clear();
        canvas.present();
        
        WindowDisplay {
            canvas: canvas, 
            pixels: [false; N]
        }
    }
}

impl <const W: usize, const H: usize, const N: usize, const PIXEL_SIZE: u32> 
Display<bool> for WindowDisplay<W, H, N, PIXEL_SIZE> {
    fn draw_pixels(&mut self) {
        self.canvas.set_draw_color(BLACK);
        self.canvas.clear();
        self.canvas.set_draw_color(WHITE);
        let width: i32 = W as i32;
        
        for (i,v) in self.pixels.iter().enumerate() {
            if *v {
                let i = i as i32;
                self.canvas.fill_rect(Rect::new(
                    (i % width)*(PIXEL_SIZE as i32), 
                    (i / width)*(PIXEL_SIZE as i32), 
                    PIXEL_SIZE, 
                    PIXEL_SIZE)
                ).unwrap()
            }
        }
    
        self.canvas.present()
    }

    fn reset_screen(&mut self) {
        self.pixels = [false; N];
    }
    
    fn update_pixels(&mut self, pixels: &[bool]) {
        assert_eq!(pixels.len(), N);
        self.pixels.copy_from_slice(pixels)
    }

    fn get_pixels(&self) -> &[bool] {
        &self.pixels
    }
}
