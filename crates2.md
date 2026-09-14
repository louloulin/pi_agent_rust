# pi.rs crate modularization

## Round 80: `pi-swarm-ledger-core`

`crates/pi-swarm-ledger-core` owns the pure swarm activity ledger algorithms:
ledger entry types and validation, monotonic append, JSONL encode/decode and
timeline queries, bounded summaries, latency sketches, and transcript digest /
aggregation logic. It has no filesystem, network, runtime, or UI dependencies.

`pi_agent_rust::swarm_activity_ledger` remains as a compatibility re-export.
Agent runtime integration and UI concerns stay in the root coding-agent crate;
future work can move those adapters without changing the core data model.
