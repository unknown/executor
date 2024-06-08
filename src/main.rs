use std::fs::OpenOptions;
use std::io::prelude::*;
use std::os::unix::fs::OpenOptionsExt;
use std::process::{Command, Output};

fn write_stdin_to_file() -> std::io::Result<()> {
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
        .open("./binary")?;
    file.write_all(&buffer)
}

fn execute_binary() -> std::io::Result<Output> {
    Command::new("./binary").output()
}

fn main() {
    if let Err(e) = write_stdin_to_file() {
        eprintln!("Failed to save stdin binary: {}", e.to_string());
        return;
    }

    match execute_binary() {
        Ok(output) => {
            println!("{}", output.status);
            println!(
                "stdout:\n{}",
                String::from_utf8(output.stdout.clone()).unwrap()
            );
            eprintln!(
                "stderr:\n{}",
                String::from_utf8(output.stderr.clone()).unwrap()
            );
        }
        Err(e) => eprintln!("Failed to execute binary: {}", e),
    }
}
