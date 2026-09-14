# pi.rs crate modularization

## Round 82: `pi-conformance-matrix-core`

`crates/pi-conformance-matrix-core` owns the pure extension conformance matrix and assertion algorithms: host capability taxonomy, expected behaviors, matrix cells, fixture assignments, category criteria, coverage summaries, API matrix interpretation, and deterministic test-plan construction. It has no runtime, filesystem, network, or UI dependencies.

`pi_agent_rust::extension_conformance_matrix` remains the coding-agent compatibility surface while the extracted core crate is validated independently. Test runners, report generation, and runtime integration stay in the root crate.
