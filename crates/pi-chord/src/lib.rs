//! Phase-2 aggregator mirroring `@earendil-works/chord`: the
//! application composition runtime (services, replicated state, RPC,
//! plugins, hostcall lanes, file-locking primitives).

#![forbid(unsafe_code)]

pub use pi_buffer_shim::*;
pub use pi_file_lock::*;
pub use pi_hostcall_io_uring_lane::*;
pub use pi_hostcall_s3_fifo::*;
pub use pi_hostcall_superinstructions::*;
pub use pi_http_shim::*;
