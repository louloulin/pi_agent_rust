# pi.rs crate modularization inventory

## Round 92

- `pi-workspace-trust-core`: pure workspace trust policy model and precedence
  evaluation (`TrustLevel`, `TrustRule`, `TrustPolicy`, `TrustOutcome`).
- Remaining in `src/workspace_trust.rs`: filesystem surface scanning, digesting,
  persistent store, environment parsing, prompt integration, and application
  error/config/UI adapters.

The core crate intentionally has no runtime, filesystem, UI, or application
crate dependencies.
