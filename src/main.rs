
use std::process::Command;

mod path;

fn main() {
    let jar = path::openapi_generator_jar_path();
    let java = path::java_path();

    let args: Vec<_> = std::env::args_os().skip(1).collect();

    let status = Command::new(java)
        .arg("-jar")
        .arg(jar)
        .args(&args)
        .status()
        .expect("Failed to launch Java — is it installed?");

    std::process::exit(status.code().unwrap_or(1));
}