use std::{
    io,
    path::Path,
    process::{self, Command, Output},
};

fn compile_program() -> io::Result<Output> {
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir("./templates/rust/")
        .output()
}

fn execute_binary(binary_path: &Path) -> io::Result<Output> {
    Command::new(binary_path).output()
}

fn run() -> io::Result<()> {
    let compile_output = compile_program()?;
    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        eprint!("{}", stderr);
        process::exit(1);
    }

    let binary_path = Path::new("./templates/rust/target/release/rust");
    let exec_output = execute_binary(binary_path)?;
    let stdout = String::from_utf8_lossy(&exec_output.stdout);
    let stderr = String::from_utf8_lossy(&exec_output.stderr);
    print!("{}", stdout);
    eprint!("{}", stderr);

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
