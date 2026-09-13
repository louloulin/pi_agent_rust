//! Phase-2 aggregator mirroring `@earendil-works/pi-chord`: the hostcall /
//! buffer / file-lock runtime. After Round 17, every Phase-1 leaf has been
//! inlined directly here.

#![forbid(unsafe_code)]

pub mod buffer_shim;
pub mod file_lock;
pub mod hostcall_amac;
pub mod hostcall_egraph;
pub mod hostcall_io_uring_lane;
pub mod hostcall_queue;
pub mod hostcall_rewrite;
pub mod hostcall_s3_fifo;
pub mod hostcall_superinstructions;
pub mod hostcall_trace_jit;
pub mod http_shim;
pub mod swarm_activity_ledger;
pub mod swarm_flight_recorder;
pub mod swarm_progress_slo;
pub mod swarm_replay;