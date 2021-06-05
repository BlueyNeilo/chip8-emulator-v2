use std::env;
use std::path::PathBuf;
use std::fs;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.contains("pc-windows") {
        load_sdl2_lib(&target);
        load_sdl2_dll(&target);
    }
}


fn load_sdl2_lib(target: &str) {
    let lib_dir = get_sdl2_library_dir(&target, "lib");
    println!("cargo:rustc-link-search=all={}", lib_dir.display())
}

fn load_sdl2_dll(target: &str) {
    let dll_dir = get_sdl2_library_dir(&target, "dll");

    for entry in fs::read_dir(dll_dir).expect("Can't read DLL dir")  {
        let entry_path = entry.expect("Invalid fs entry").path();

        // 'if let = [expr] && [expr]' is experimental, waiting for release
        if let Some(extension) = entry_path.extension() { 
            if extension == "dll" {
            let destination_file_path = get_manifest_dir()
                .join(entry_path.file_name()
                    .expect("Invalid DLL file name"));
            fs::copy(&entry_path, destination_file_path.as_path())
                .expect("Can't copy from DLL dir");
            }
        }
    }
}

fn get_sdl2_library_dir(target: &str, dir_type: &str) -> PathBuf {
    let win_compiler_dir = if target.contains("msvc") { "msvc" } else { "gnu-mingw" };
    let cpu_architecture_dir = if target.contains("x86_64") { "64" } else { "32" };

    let path_list = vec![
        "sdl2-libs", 
        win_compiler_dir, 
        dir_type, 
        cpu_architecture_dir
    ];

    get_manifest_dir().join(path_list.iter().collect::<PathBuf>())
}

fn get_manifest_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
}
