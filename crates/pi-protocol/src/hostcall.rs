//! Protocol data types for JavaScript-originated hostcalls.
//! Pure request metadata shared by hostcall planners and runtime adapters.

/// Type of hostcall being requested from JavaScript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostcallKind {
    Tool { name: String },
    Exec { cmd: String },
    Http,
    Session { op: String },
    Ui { op: String },
    Events { op: String },
    Log,
}

/// A hostcall request enqueued from JavaScript.
#[derive(Debug, Clone)]
pub struct HostcallRequest {
    pub call_id: String,
    pub kind: HostcallKind,
    pub payload: serde_json::Value,
    pub trace_id: u64,
    pub extension_id: Option<String>,
}
