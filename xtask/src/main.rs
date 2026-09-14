//! `xtask` — workspace plumbing for the `pi.rs` modularization.
//!
//! Subcommands:
//!
//! - `cargo xtask check`   — `cargo check --workspace --locked --all-targets`
//! - `cargo xtask test`    — `cargo test  --workspace --locked --no-run`
//! - `cargo xtask clippy`  — `cargo clippy --workspace --locked --all-targets -- -D warnings`
//! - `cargo xtask fmt`     — `cargo fmt --all -- --check`
//! - `cargo xtask doc`     — `cargo doc  --workspace --no-deps --locked`
//! - `cargo xtask ci`      — runs check + test + clippy sequentially
//!
//! Phase-1 keeps `xtask` minimal. Later stages may add generation helpers for
//! stub crates, dependency-graph audits, and migration rollouts.

use anyhow::{Context, Result};
use cargo_metadata::{Metadata, MetadataCommand};
use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "xtask", about = "Workspace plumbing for pi.rs")]
struct Xtask {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Print the resolved workspace member crates and exit.
    Members,
    /// `cargo check --workspace --locked --all-targets`
    Check,
    /// `cargo test --workspace --locked --no-run`
    Test,
    /// `cargo clippy --workspace --locked --all-targets -- -D warnings`
    Clippy,
    /// `cargo fmt --all -- --check`
    Fmt,
    /// `cargo doc --workspace --no-deps --locked`
    Doc,
    /// Run check + test + clippy sequentially.
    Ci,
}

fn metadata() -> Result<Metadata> {
    MetadataCommand::new()
        .manifest_path("Cargo.toml")
        .exec()
        .context("resolving workspace metadata")
}

fn run(cmd: &mut Command) -> Result<()> {
    let status = cmd.status().with_context(|| format!("spawning {cmd:?}"))?;
    if !status.success() {
        anyhow::bail!("command failed (exit {:?}): {:?}", status.code(), cmd);
    }
    Ok(())
}

fn cmd_check() -> Result<()> {
    let mut c = Command::new("cargo");
    c.args(["check", "--workspace", "--locked", "--all-targets"]);
    run(&mut c)
}

fn cmd_test() -> Result<()> {
    let mut c = Command::new("cargo");
    c.args(["test", "--workspace", "--locked", "--no-run"]);
    run(&mut c)
}

fn cmd_clippy() -> Result<()> {
    let mut c = Command::new("cargo");
    c.args([
        "clippy",
        "--workspace",
        "--locked",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ]);
    run(&mut c)
}

fn cmd_fmt() -> Result<()> {
    let mut c = Command::new("cargo");
    c.args(["fmt", "--all", "--", "--check"]);
    run(&mut c)
}

fn cmd_doc() -> Result<()> {
    let mut c = Command::new("cargo");
    c.args(["doc", "--workspace", "--no-deps", "--locked"]);
    run(&mut c)
}

fn cmd_members() -> Result<()> {
    let meta = metadata()?;
    let names: Vec<String> = meta
        .workspace_members
        .iter()
        .filter_map(|id| meta.packages.iter().find(|p| &p.id == id))
        .map(|p| p.name.clone())
        .collect();
    for name in names {
        println!("{name}");
    }
    Ok(())
}

fn cmd_ci() -> Result<()> {
    cmd_check()?;
    cmd_test()?;
    cmd_clippy()?;
    println!("xtask ci: check + test + clippy all green");
    Ok(())
}

fn main() -> Result<()> {
    let args = Xtask::parse();
    match args.cmd {
        Cmd::Members => cmd_members(),
        Cmd::Check => cmd_check(),
        Cmd::Test => cmd_test(),
        Cmd::Clippy => cmd_clippy(),
        Cmd::Fmt => cmd_fmt(),
        Cmd::Doc => cmd_doc(),
        Cmd::Ci => cmd_ci(),
    }
}
