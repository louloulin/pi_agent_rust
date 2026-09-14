//! Round 19 build script: forwards Cargo's authoritative build metadata
//! (profile family, opt level, debug-assertions switch, and the sorted
//! enabled feature list) into compile-time environment variables. The
//! `perf_build.rs` module consumes these via `env!()` so the perf evidence
//! layer can fingerprint the binary without re-implementing Cargo's profile
//! inference.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let opt_level = env::var("OPT_LEVEL").unwrap_or_else(|_| "0".to_string());
    let debug = env::var("DEBUG").unwrap_or_else(|_| "true".to_string());
    let cargo_features = env::var("CARGO_FEATURES").unwrap_or_default();

    let mut sorted_features: Vec<&str> =
        cargo_features.split(',').filter(|s| !s.is_empty()).collect();
    sorted_features.sort();
    sorted_features.dedup();
    let features_csv = sorted_features.join(",");

    // The upstream perf fingerprint contract treats release-inheriting
    // custom profiles as `release`; `PROFILE` already reflects that, so
    // passing it through verbatim keeps the contract intact.
    write_env(&[
        ("PI_BUILD_PROFILE_FAMILY", profile.as_str()),
        ("PI_BUILD_OPT_LEVEL", opt_level.as_str()),
        ("PI_BUILD_DEBUG", debug.as_str()),
        ("PI_BUILD_FEATURES", features_csv.as_str()),
    ]);

    // Generate a small fingerprint file so evidence tools can verify the
    // build identity without re-running the linker.
    if let Ok(out_dir) = env::var("OUT_DIR") {
        let fp_path = Path::new(&out_dir).join("pi-build-fingerprint.txt");
        let body = format!(
            "profile={profile}\nopt_level={opt_level}\ndebug={debug}\nfeatures={features_csv}\n",
        );
        let _ = fs::write(fp_path, body);
    }
}

fn write_env(pairs: &[(&str, &str)]) {
    for (name, value) in pairs {
        println!("cargo:rustc-env={}={}", name, value);
    }
}