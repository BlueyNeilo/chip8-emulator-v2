use std::fs::read_dir;
use std::path::Path;
use std::io::stdin;

const MENU_ROWS: usize = 4;
const MENU_COL_LEN: usize = 10;

pub fn choose_rom() -> String {
    let roms_dir = Path::new("./ROMs/");
    let rom_name: String;
    print_roms(&get_rom_names(&roms_dir));

    'getrom: loop {
        match get_valid_rom(&roms_dir) {
            Ok(good_name) => { rom_name = good_name; break 'getrom },
            Err(bad_name) => println!("Sorry the ROM '{}' does not exist.", bad_name)
        }
    }

    roms_dir.join(rom_name).as_path().to_str().unwrap().to_string()
}

fn get_rom_names(roms_dir: &Path) -> Vec<String> {
    read_dir(&roms_dir).expect("ROM directory doesn't exist.")
        .into_iter()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<String>>()
}

fn print_roms(rom_names: &Vec<String>) {
    let mut rom_names_iter = rom_names.clone().into_iter().peekable();
    let mut rom_names_grid: Vec<Vec<String>> = Vec::new();

    while let Some(_) = (&mut rom_names_iter).peek() { 
        rom_names_grid.push((&mut rom_names_iter)
            .take(MENU_ROWS)
            .map(|name| format!("{:padding$}", name, padding = MENU_COL_LEN))
            .collect::<Vec<String>>()
        ); 
    };

    println!("\nAvailable ROMS:\n");
    rom_names_grid.iter()
        .for_each(|v| println!("{}", v.concat()))
}

fn get_valid_rom(roms_dir: &Path) -> Result<String, String> {
    let mut rom_name = String::new();

    println!("\nPlease choose a ROM to play:");
    stdin().read_line(&mut rom_name).expect("Failed to read line");
    rom_name = rom_name.trim_end_matches(char::is_control).to_string();

    if roms_dir.join(&rom_name).exists() {
        Ok(rom_name)
    } else {
        Err(rom_name)
    }
}