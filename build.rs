use std::env;
use std::process::Command;
use std::path::Path;
fn main() {
    println!("Generating protos...");
    let proto_dir = "proto";

    // Change current directory to proto
    env::set_current_dir(&Path::new(proto_dir)).expect("Failed to change directory");

    // Run buf generate to ensure .proto files are up-to-date
    Command::new("buf")
        .args(&["generate", "--template", "buf.gen.yaml"])
        .status()
        .expect("failed to run buf generate");

    // Change back to the original directory
    env::set_current_dir(&env::current_dir().unwrap().parent().unwrap()).expect("Failed to change back to original directory");
}
