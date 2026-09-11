# Outcome

Phase-1 modularization of `pi.rs` into a multi-crate Cargo workspace whose
boundary seams mirror the TypeScript `pi`/`@earendil-works/*` packages, while
keeping the existing `pi_agent_rust` single-crate build green and the plugin
ecosystem (PIJS / QuickJS / WASM / WIT) compatible.

# Scope

This change covers Phase-1 only (foundation + first extraction slice):

- Workspace root bootstrap (root `Cargo.toml`, `[workspace.dependencies]`,
  `rust-toolchain.toml` preservation, `clippy.toml`/`rustfmt.toml` carry-over).
- `xtask` for workspace plumbing (`cargo xtask check`, `cargo xtask fmt`,
  `cargo xtask doc`, `cargo xtask ci`).
- `crates/pi` shim crate re-exporting the current `pi` public API.
- `crates/pi-mono` aggregator (mirror of `@earendil-works/mono` if present in
  upstream, else aggregate workspace convenience crate).
- `tests/workspace_smoke.rs` integration test.
- Toolchain verification: `cargo check`, `cargo test --no-run`, and the
  smoke test all pass on `nightly-2026-08-31`.

Out of scope (covered in later stages of the same plan, separate changes):

- Full extraction of `@earendil-works/chord`, `tui`, `telemetry`, `ai`,
  `agent`, `sqlite-node`, `protocol`, `client`, `server`, `coding-agent`,
  `evals` into standalone crates (Stages 2-9).
- PIJS / WASM hostcall re-homing (Stages 10-11).
- Documentation, examples, and packaging changes (Stage 12).

# Non-goals

- Breaking the existing `pi` crate's public API surface. Downstream callers
  that depend on `pi = "..."` continue to compile.
- Re-licensing or re-publishing.
- Performance optimization unrelated to the refactor.
- Migrating away from `asupersync`, `rquickjs`, `wasmtime`, `fsqlite`,
  `charmed-*`, or `ftui`.
- Touching the TypeScript `pi` repo at https://github.com/louloulin/pi.git.

# Acceptance examples

- `cargo check --workspace --locked` exits 0 on the pinned nightly.
- `cargo test --workspace --locked --no-run` exits 0.
- `cargo test --workspace --locked --test workspace_smoke` exits 0 and the
  test asserts every declared member crate resolves and re-exports its
  declared surface.
- `cargo clippy --workspace --locked --all-targets -- -D warnings` exits 0
  (parity with the pre-refactor clippy clean state documented in
  `rust-toolchain.toml`).
- `cargo run -p xtask -- ci` exits 0 (runs check + fmt + clippy + test).
- `cargo build -p pi --locked` exits 0, confirming the `crates/pi` shim
  re-exports the existing public API surface.

# Constraints and invariants

- `rust-toolchain.toml` pins `nightly-2026-08-31`; the pin is not changed by
  this change.
- The 175+ modules currently under `src/` are not deleted; they are moved
  into `crates/pi/src/` and the original root `Cargo.toml` is preserved as
  `crates/pi/Cargo.toml.legacy` for reference until Stage 12 cleanup.
- `xtask` must be a member of the workspace (not a separate workspace).
- Public re-exports in `crates/pi/src/lib.rs` preserve the current names so
  `use pi::...` paths keep working.
- Git: all work happens on branch `agent/devbox2/f963f7e70515`. No force
  push, no reset --hard, no `branch -D` without explicit confirmation.
- No secrets, tokens, or `.env` content in any commit.

# Decisions

- Workspace root manifest lives at the repo root. The legacy `Cargo.toml`
  is preserved (not deleted) at `crates/pi/Cargo.toml.legacy` for diff
  reviewers.
- Crate naming: `pi` (shim, public API), `pi-mono` (workspace aggregator),
  `xtask` (workspace plumbing). Mirrors `@earendil-works/mono` semantics
  from upstream.
- `[workspace.dependencies]` is the single source of truth for versions.
  Member crates reference workspace keys (e.g.
  `asupersync.workspace = true`).
- Smoke test uses `cargo_metadata` to enumerate members and assert every
  declared member crate resolves. Locked to a specific `cargo_metadata`
  minor via the workspace.

# Open questions

- Should `crates/pi/Cargo.toml.legacy` be kept indefinitely (diff aid) or
  scheduled for removal at the end of Stage 12? Default: keep until Stage
  12 review, then remove.
- Should `xtask` enforce `cargo fmt --check` or rely on CI? Default: CI
  only, `xtask ci` runs clippy + test.

# Verification expectations

- `cargo check --workspace --locked` green on `nightly-2026-08-31`.
- `cargo test --workspace --locked --no-run` green.
- `cargo test --workspace --locked --test workspace_smoke` green and the
  test body is committed in full.
- `cargo clippy --workspace --locked --all-targets -- -D warnings` green.
- `cargo run -p xtask -- ci` green.
- All commits land on branch `agent/devbox2/f963f7e70515`. No commits to
  `main` or `master`.
