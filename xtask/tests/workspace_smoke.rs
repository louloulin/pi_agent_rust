//! Workspace smoke test for the `pi.rs` Phase-1 modularization.
//!
//! Verifies the workspace root and every declared member crate resolve via
//! `cargo metadata`. This is a minimal sentinel — the deeper invariants
//! (public API continuity, plugin ecosystem compatibility, clippy parity) are
//! enforced by the per-member crate test suites and by `cargo xtask ci`.
//!
//! Accepts A3 from `docs/comet/changes/pi-rs-modularization-phase1/brief.md`.

use cargo_metadata::{MetadataCommand, TargetKind};
use std::collections::BTreeSet;

fn workspace_meta() -> cargo_metadata::Metadata {
    MetadataCommand::new()
        .manifest_path("Cargo.toml")
        .exec()
        .expect("workspace metadata must resolve")
}

#[test]
fn workspace_root_resolves_and_lists_expected_members() {
    let meta = workspace_meta();

    // The workspace root itself must be present and unique.
    assert_eq!(meta.workspace_root.as_os_str().len(), meta.workspace_root.as_os_str().len(), "workspace_root must be a single path");
    assert!(meta.workspace_root.is_absolute() || meta.workspace_root.starts_with("."),
        "workspace_root should resolve to a real path; got {:?}", meta.workspace_root);

    let declared: BTreeSet<String> = meta
        .workspace_members
        .iter()
        .filter_map(|id| meta.packages.iter().find(|p| &p.id == id))
        .map(|p| p.name.clone())
        .collect();

    for expected in ["pi", "pi-mono", "xtask"] {
        assert!(
            declared.contains(expected),
            "workspace member `{expected}` missing; declared members: {declared:?}"
        );
    }
}

#[test]
fn pi_package_exposes_public_lib_target() {
    let meta = workspace_meta();

    let pi = meta
        .packages
        .iter()
        .find(|p| p.name == "pi")
        .expect("workspace must contain a `pi` package");

    let lib_targets: Vec<&str> = pi
        .targets
        .iter()
        .filter(|t| matches!(t.kind.first(), Some(TargetKind::Lib)))
        .map(|t| t.name.as_str())
        .collect();

    assert!(
        lib_targets.contains(&"pi"),
        "`pi` package must expose a lib target named `pi` (A6: cargo build -p pi); \
         got lib target names: {lib_targets:?}"
    );
}

#[test]
fn pi_mono_depends_on_pi_via_path() {
    let meta = workspace_meta();

    let pi_mono = meta
        .packages
        .iter()
        .find(|p| p.name == "pi-mono")
        .expect("workspace must contain `pi-mono`");

    let pi_dep = pi_mono
        .dependencies
        .iter()
        .find(|d| d.name == "pi")
        .unwrap_or_else(|| panic!("`pi-mono` must depend on `pi`; got: {:?}", pi_mono.dependencies));

    let expected_path = meta.workspace_root.join("crates/pi");
    let dep_path = pi_dep.path.as_deref();
    assert_eq!(
        dep_path,
        Some(expected_path.as_path()),
        "`pi-mono` must depend on `pi` via path ../pi"
    );
}

#[test]
fn xtask_is_publish_false_and_exposes_binary() {
    let meta = workspace_meta();

    let xtask = meta
        .packages
        .iter()
        .find(|p| p.name == "xtask")
        .expect("workspace must contain `xtask`");

    assert_eq!(
        xtask.publish.as_deref(),
        Some(&[] as &[String]),
        "`xtask` must not be publishable"
    );

    let has_bin = xtask
        .targets
        .iter()
        .any(|t| t.name == "xtask" && matches!(t.kind.first(), Some(TargetKind::Bin)));
    assert!(
        has_bin,
        "`xtask` package must expose a binary target named `xtask`; \
         got targets: {:?}",
        xtask.targets.iter().map(|t| (&t.name, &t.kind)).collect::<Vec<_>>()
    );
}