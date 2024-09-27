use std::path::{Path, PathBuf};
use std::{env, fs, io};
use winresource::WindowsResource;

fn main() -> io::Result<()> {
    let src = Path::new("resources");
    let dst = PathBuf::from(env::var("OUT_DIR").unwrap()).join("resources");

    fs::create_dir_all(&dst).unwrap();

    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name().unwrap();
            let destination = dst.join(file_name);
            fs::copy(&path, &destination).unwrap();
        }
    }

    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        WindowsResource::new()
            .set_icon("resources/icon.ico")
            .compile()?;
    }
    Ok(())
}
