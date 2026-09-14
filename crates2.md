# Crate modularization

## pi-semantic-graph-core

The pure semantic workspace graph algorithms are isolated in
`crates/pi-semantic-graph-core`: graph/node/edge types, deterministic builder,
query/planning, graph walking, evidence classification, and path/fingerprint
helpers. It has no dependency on the Pi runtime, UI, or agent orchestration.

`pi_agent_rust::semantic_workspace_graph` remains a compatibility facade that
re-exports the core crate while callers migrate to the standalone API.
