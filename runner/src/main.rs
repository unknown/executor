use std::fs::OpenOptions;
use std::io::prelude::*;
use std::os::unix::fs::OpenOptionsExt;
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
        .mode(0o777)
        .create(true)
        .open(file_path)?;
    file.write_all(&buffer)
}

fn execute_binary(binary_path: &std::path::Path) -> std::io::Result<Output> {
    Command::new(binary_path).output()
}

fn main() {
    let binary_path = std::path::Path::new("./binary");

    if let Err(e) = write_stdin_to_file(binary_path) {
        eprintln!("Failed to save stdin binary: {}", e.to_string());
        return;
    }

    match execute_binary(binary_path) {
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout.clone()).unwrap();
            let stderr = String::from_utf8(output.stderr.clone()).unwrap();
            println!("{}", output.status);
            println!("stdout:\n{}", stdout);
            eprintln!("stderr:\n{}", stderr);
        }
        Err(e) => eprintln!("Failed to execute binary: {}", e),
    }
}
