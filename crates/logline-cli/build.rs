#![forbid(unsafe_code)]

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=LOGLINE_GIT_COMMIT");
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/logs/HEAD");

    let commit = match commit_from_env().or_else(commit_from_git) {
        Some(commit) => commit,
        None => "unknown".to_string(),
    };

    println!("cargo:rustc-env=LOGLINE_GIT_COMMIT={commit}");
}

fn commit_from_env() -> Option<String> {
    std::env::var("LOGLINE_GIT_COMMIT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn commit_from_git() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
