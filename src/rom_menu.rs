use std::fs::read_dir;
use std::path::Path;
use std::io;

pub fn choose_rom() -> String {
    let dir = Path::new("./ROMs/");
    let mut rom = String::new();
    println!("\nAvailable ROMS:\n");
    if dir.is_dir() {
        for entry in read_dir(&dir).unwrap() {
            println!("{}", entry.unwrap().path().file_name().unwrap().to_str().unwrap())
        }
        'getrom: loop {
            println!("\nPlease chose a ROM to play:");
            io::stdin().read_line(&mut rom)
                .expect("Failed to read line");
            let tlen: usize = rom.len()-2;
            rom.truncate(tlen); //Remove '\n\r' at the end.
            if dir.join(&rom).as_path().exists() {
                break 'getrom
            }
            else
            {
                println!("Sorry that ROM does not exist.");
                rom=String::new() //clear string for next read attempt
            }
        }
    }
    else
    {
        panic!("ROM directory doesn't exist.")
    }
    dir.join(rom).as_path().to_str().unwrap().to_string()
}