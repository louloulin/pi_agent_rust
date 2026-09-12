use std::process::Command;

fn emit_git_sha() {
    let sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty());

    if let Some(sha) = sha {
        println!("cargo:rustc-env=PI_CRASH_GIT_SHA={sha}");
    }
}

fn emit_build_timestamp() {
    let stamp = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty());

    if let Some(stamp) = stamp {
        println!("cargo:rustc-env=PI_CRASH_BUILD_TIMESTAMP={stamp}");
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    emit_git_sha();
    emit_build_timestamp();
}