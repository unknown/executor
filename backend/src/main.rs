mod error;
mod jobs;

use std::time::Duration;

use jobs::{Job, RustJob};
use nomad_rs::Nomad;

#[tokio::main]
async fn main() {
    let nomad = Nomad::default();
    let timeout = Duration::from_secs(20);
    let interval = Duration::from_secs(1);

    let code = r#"fn main() {
    println!("Hello, from Rust code!");
}"#;

    let job = RustJob::new(code);

    match job.execute(&nomad, timeout, interval).await {
        Ok(output) => println!("Output:\n{}", output),
        Err(error) => eprintln!("{}", error),
    };
}
