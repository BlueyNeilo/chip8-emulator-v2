# Chip8 Emulator in Rust
## Rust Setup
[Setup Guide](https://gist.github.com/BlueyNeilo/88b1a0ef1276b974bb659dc5268ad160)

## Windows Setup

- `cargo build`

- Copy `ROMs/` and `SDL2.dll` to the `target/debug/` folder

- Run the executable file `/target/debug/chip_8.exe`

## MacOS Setup

Not Supported

## How to play

- A terminal will appear (main screen). Enter the name for one of the listed ROMs

- A window will open up with the emulated chip8 ROM.

## Keyboard

See [EmulatorSpecs.docx](https://github.com/BlueyNeilo/Chip8Emulator/blob/master/EmulatorSpecs.docx) for all possible keys to press

### Pong controls

- 1 - left paddle up

- Q - left paddle down

- 4 - right paddle up

- R - right paddle down

## ROM Copyright

Chip8 ROMs are in [public domain](https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html)

## Technical references

[Laurence Muller's Chip8 Emulator Guide](http://www.multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/)

[Cowgod's Chip-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)

[Bindings for SDL2 in Rust](https://github.com/Rust-SDL2/rust-sdl2)
