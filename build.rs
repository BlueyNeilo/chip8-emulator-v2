use std::env;
use std::path::PathBuf;
use std::fs;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.contains("pc-windows") {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let lib_dir = get_sdl2_library_dir(&target, "lib");
        let dll_dir = get_sdl2_library_dir(&target, "dll");

        println!("cargo:rustc-link-search=all={}", lib_dir.display());
        for entry in fs::read_dir(dll_dir).expect("Can't read DLL dir")  {
            let entry_path = entry.expect("Invalid fs entry").path();
            let file_name_result = entry_path.file_name();
            let mut new_file_path = manifest_dir.clone();
            if let Some(file_name) = file_name_result {
                let file_name = file_name.to_str().unwrap();
                if file_name.ends_with(".dll") {
                    new_file_path.push(file_name);
                    fs::copy(&entry_path, new_file_path.as_path()).expect("Can't copy from DLL dir");
                }
            }
        }
    }
}

fn get_sdl2_library_dir(target: &str, dir_type: &str) -> PathBuf {
    let mut dir_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    
    let windows_compiler_dir = if target.contains("msvc") { "msvc" } else { "gnu-mingw" };
    let cpu_architecture_dir = if target.contains("x86_64") { "64" } else { "32" };
    let path_list = vec![windows_compiler_dir, dir_type, cpu_architecture_dir];

    for path in path_list.into_iter() {
        dir_path.push(path);
    }

    dir_path
}
