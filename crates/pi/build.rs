use vergen_gix::{BuildBuilder, CargoBuilder, Emitter, GixBuilder, RustcBuilder};

fn emit_benchmark_build_fingerprint() -> Result<(), Box<dyn std::error::Error>> {
    fn required_env(name: &str) -> Result<String, std::io::Error> {
        std::env::var(name).map_err(|_| {
            std::io::Error::other(format!(
                "Cargo did not provide required build variable {name}"
            ))
        })
    }

    let profile_family = required_env("PROFILE")?;
    let opt_level = required_env("OPT_LEVEL")?;
    let debug = required_env("DEBUG")?;
    let mut features = std::env::vars()
        .filter_map(|(name, _)| {
            name.strip_prefix("CARGO_FEATURE_")
                .map(|feature| feature.to_ascii_lowercase().replace('_', "-"))
        })
        .collect::<Vec<_>>();
    features.sort_unstable();
    features.dedup();

    println!("cargo:rustc-env=PI_BUILD_PROFILE_FAMILY={profile_family}");
    println!("cargo:rustc-env=PI_BUILD_OPT_LEVEL={opt_level}");
    println!("cargo:rustc-env=PI_BUILD_DEBUG={debug}");
    println!("cargo:rustc-env=PI_BUILD_FEATURES={}", features.join(","));
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    emit_benchmark_build_fingerprint()?;

    let build = BuildBuilder::default().build_timestamp(true).build()?;
    let cargo = CargoBuilder::default().target_triple(true).build()?;
    let gix = GixBuilder::default().sha(true).dirty(true).build()?;
    let rustc = RustcBuilder::default().semver(true).build()?;

    let mut emitter = Emitter::default();
    // Offloaded builds can temporarily miss git objects and trigger fallback warnings.
    // Keep default env fallbacks, but suppress warning noise in build output.
    emitter
        .quiet()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&gix)?
        .add_instructions(&rustc)?
        .emit()?;

    Ok(())
}
