use std::fs::OpenOptions;
use std::io::prelude::*;
use std::process::{Command, Output};

fn write_stdin_to_file(file_path: &std::path::Path) -> std::io::Result<()> {
    // read stdin to buffer
    let mut buffer = Vec::new();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_end(&mut buffer)?;

    // save buffer to file
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(file_path)?;
    file.write_all(&buffer)
}

fn compile_program() -> std::io::Result<Output> {
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir("./templates/rust/")
        .output()
}

fn main() {
    let main_file_path = std::path::Path::new("./templates/rust/src/main.rs");

    if let Err(e) = write_stdin_to_file(main_file_path) {
        eprintln!("Failed to save stdin: {}", e.to_string());
        return;
    }

    match compile_program() {
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout.clone()).unwrap();
            let stderr = String::from_utf8(output.stderr.clone()).unwrap();
            println!("{}", output.status);
            println!("stdout:\n{}", stdout);
            eprintln!("stderr:\n{}", stderr);
        }
        Err(e) => eprintln!("Failed to compile: {}", e),
    }
}
