use std::process::Command;

fn main() {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()
        .expect("failed to execute process");
    let git_hash = String::from_utf8_lossy(&output.stdout);
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}