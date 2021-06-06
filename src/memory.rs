/*
Memory map:
0x000-0x1FF - Chip 8 interpreter (contains font set in emulator)
0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
0x200-0xFFF - Program ROM and work RAM
*/
use sdl2::keyboard::Keycode;

pub fn init_memory(memory: &mut [u8; 0x1000], kvals: &mut [Keycode; 0x10]) {
    let font: [u8; 16*5] = [0xF0,0x90,0x90,0x90,0xF0, //0
    0x20,0x60,0x20,0x20,0x70, //1
    0xF0,0x10,0xF0,0x80,0xF0, //2
    0xF0,0x10,0xF0,0x10,0xF0, //3
    0x90,0x90,0xF0,0x10,0x10, //4
    0xF0,0x80,0xF0,0x10,0xF0, //5
    0xF0,0x80,0xF0,0x90,0xF0, //6
    0xF0,0x10,0x20,0x40,0x40, //7
    0xF0,0x90,0xF0,0x90,0xF0, //8
    0xF0,0x90,0xF0,0x10,0xF0, //9
    0xF0,0x90,0xF0,0x90,0x90, //A
    0xE0,0x90,0xE0,0x90,0xE0, //B
    0xF0,0x80,0x80,0x80,0xF0, //C
    0xE0,0x90,0x90,0x90,0xE0, //D
    0xF0,0x80,0xF0,0x80,0xF0, //E
    0xF0,0x80,0xF0,0x80,0x80]; //F
    *kvals = [Keycode::X,
        Keycode::Num1,
        Keycode::Num2,
        Keycode::Num3,
        Keycode::Q,
        Keycode::W,
        Keycode::E,
        Keycode::A,
        Keycode::S,
        Keycode::D,
        Keycode::Z,
        Keycode::C,
        Keycode::Num4,
        Keycode::R,
        Keycode::F,
        Keycode::V]; //1234QWERASDFZXCV hex keypad
    let mut addr = 0x50;
    for byte in font.iter() {
        memory[addr] = *byte;
        addr+=1
    }
}