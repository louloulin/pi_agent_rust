//! Smoke test for the `pi-cli` Stage 2 leaf crate.
//!
//! Verifies that the clap argument surface parses correctly. Exercises both
//! the help-flag short-circuit and the default-args path so that any
//! regression in `Cli::derive` is caught immediately.

use clap::Parser;
use pi_cli::Cli;

#[test]
fn parses_help_flag() {
    // clap rejects --help with an error kind = DisplayHelp; we don't surface
    // the rendered help text, just assert the parse error has the right kind.
    let err = Cli::try_parse_from(["pi", "--help"]).unwrap_err();
    assert!(
        matches!(err.kind(), clap::error::ErrorKind::DisplayHelp),
        "expected DisplayHelp, got {:?}",
        err.kind()
    );
}

#[test]
fn parses_no_args() {
    let cli = Cli::try_parse_from(["pi"]).expect("default args parse");
    // Smoke check that the structure is intact and Debug-printable.
    let _ = format!("{cli:?}");
}

#[test]
fn parses_list_models_flag() {
    let cli = Cli::try_parse_from(["pi", "--list-models"]).expect("list-models parses");
    assert_eq!(cli.list_models, Some(None));
}