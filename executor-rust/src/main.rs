use std::process::{Command, Output};

fn compile_program() -> std::io::Result<Output> {
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir("./templates/rust/")
        .output()
}

fn execute_binary(binary_path: &std::path::Path) -> std::io::Result<Output> {
    Command::new(binary_path).output()
}

fn main() {
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

    let binary_path = std::path::Path::new("./templates/rust/target/release/rust");

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
