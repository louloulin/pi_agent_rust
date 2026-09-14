//! Pure protocol core for the PiJS bridge.
//!
//! This crate deliberately contains no QuickJS, scheduler, or host authority
//! dependencies. It owns the data and ordering contracts shared by the JS
//! runtime facade and hostcall dispatchers.

#![forbid(unsafe_code)]

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Type of hostcall requested by JavaScript.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HostcallKind {
    Tool { name: String },
    Exec { cmd: String },
    Http,
    Session { op: String },
    Ui { op: String },
    Events { op: String },
    Log,
}

/// Hostcall envelope shared across the JS/runtime boundary.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HostcallRequest {
    pub call_id: String,
    pub kind: HostcallKind,
    pub payload: serde_json::Value,
    pub trace_id: u64,
    pub extension_id: Option<String>,
}

/// Tool metadata registered by an extension.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct ExtensionToolDef {
    pub name: String,
    #[serde(default)]
    pub label: Option<String>,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Clock used by the deterministic event loop.
pub trait Clock: Send + Sync {
    fn now_ms(&self) -> u64;
}

#[derive(Clone)]
pub struct ClockHandle(Arc<dyn Clock>);

impl ClockHandle {
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self(clock)
    }
}

impl Clock for ClockHandle {
    fn now_ms(&self) -> u64 {
        self.0.now_ms()
    }
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        u64::try_from(now.as_millis()).unwrap_or(u64::MAX)
    }
}

#[derive(Debug)]
pub struct ManualClock {
    now_ms: AtomicU64,
}

impl ManualClock {
    pub const fn new(start_ms: u64) -> Self {
        Self {
            now_ms: AtomicU64::new(start_ms),
        }
    }
    pub fn set(&self, ms: u64) {
        self.now_ms.store(ms, AtomicOrdering::SeqCst);
    }
    pub fn advance(&self, delta_ms: u64) {
        self.now_ms.fetch_add(delta_ms, AtomicOrdering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> u64 {
        self.now_ms.load(AtomicOrdering::SeqCst)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacrotaskKind {
    TimerFired { timer_id: u64 },
    HostcallComplete { call_id: String },
    InboundEvent { event_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Macrotask {
    pub seq: u64,
    pub trace_id: u64,
    pub kind: MacrotaskKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MacrotaskEntry {
    seq: u64,
    trace_id: u64,
    kind: MacrotaskKind,
}
impl Ord for MacrotaskEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.seq.cmp(&other.seq)
    }
}
impl PartialOrd for MacrotaskEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TimerEntry {
    deadline_ms: u64,
    order_seq: u64,
    timer_id: u64,
    trace_id: u64,
}
impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.deadline_ms, self.order_seq, self.timer_id).cmp(&(
            other.deadline_ms,
            other.order_seq,
            other.timer_id,
        ))
    }
}
impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickResult {
    pub ran_macrotask: bool,
    pub microtasks_drained: usize,
}

/// Deterministic one-macrotask-per-tick event loop protocol.
pub struct PiEventLoop {
    clock: ClockHandle,
    seq: u64,
    next_timer_id: u64,
    pending: VecDeque<(u64, MacrotaskKind)>,
    macro_queue: BinaryHeap<std::cmp::Reverse<MacrotaskEntry>>,
    timers: BinaryHeap<std::cmp::Reverse<TimerEntry>>,
    cancelled_timers: HashSet<u64>,
}

impl PiEventLoop {
    pub fn new(clock: ClockHandle) -> Self {
        Self {
            clock,
            seq: 0,
            next_timer_id: 1,
            pending: VecDeque::new(),
            macro_queue: BinaryHeap::new(),
            timers: BinaryHeap::new(),
            cancelled_timers: HashSet::new(),
        }
    }
    pub fn enqueue_hostcall_completion(&mut self, call_id: impl Into<String>) {
        let trace = self.next_seq();
        self.pending.push_back((
            trace,
            MacrotaskKind::HostcallComplete {
                call_id: call_id.into(),
            },
        ));
    }
    pub fn enqueue_inbound_event(&mut self, event_id: impl Into<String>) {
        let trace = self.next_seq();
        self.pending.push_back((
            trace,
            MacrotaskKind::InboundEvent {
                event_id: event_id.into(),
            },
        ));
    }
    pub fn set_timeout(&mut self, delay_ms: u64) -> u64 {
        let timer_id = self.next_timer_id;
        self.next_timer_id = self.next_timer_id.saturating_add(1);
        let order_seq = self.next_seq();
        self.timers.push(std::cmp::Reverse(TimerEntry {
            deadline_ms: self.clock.now_ms().saturating_add(delay_ms),
            order_seq,
            timer_id,
            trace_id: order_seq,
        }));
        timer_id
    }
    pub fn clear_timeout(&mut self, timer_id: u64) -> bool {
        if self.timers.iter().any(|entry| entry.0.timer_id == timer_id)
            && self.cancelled_timers.insert(timer_id)
        {
            true
        } else {
            false
        }
    }
    pub fn tick(
        &mut self,
        mut on_macrotask: impl FnMut(Macrotask),
        mut drain_microtasks: impl FnMut() -> bool,
    ) -> TickResult {
        while let Some((trace, kind)) = self.pending.pop_front() {
            self.enqueue_macrotask(trace, kind);
        }
        let now = self.clock.now_ms();
        while let Some(std::cmp::Reverse(entry)) = self.timers.peek().cloned() {
            if entry.deadline_ms > now {
                break;
            }
            let _ = self.timers.pop();
            if !self.cancelled_timers.remove(&entry.timer_id) {
                self.enqueue_macrotask(
                    entry.trace_id,
                    MacrotaskKind::TimerFired {
                        timer_id: entry.timer_id,
                    },
                );
            }
        }
        let Some(entry) = self.macro_queue.pop() else {
            return TickResult {
                ran_macrotask: false,
                microtasks_drained: 0,
            };
        };
        let entry = entry.0;
        on_macrotask(Macrotask {
            seq: entry.seq,
            trace_id: entry.trace_id,
            kind: entry.kind,
        });
        let mut drained = 0;
        while drain_microtasks() {
            drained += 1;
        }
        TickResult {
            ran_macrotask: true,
            microtasks_drained: drained,
        }
    }
    fn enqueue_macrotask(&mut self, trace_id: u64, kind: MacrotaskKind) {
        let seq = self.next_seq();
        self.macro_queue.push(std::cmp::Reverse(MacrotaskEntry {
            seq,
            trace_id,
            kind,
        }));
    }
    fn next_seq(&mut self) -> u64 {
        let current = self.seq;
        self.seq = self.seq.saturating_add(1);
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_precedes_due_timer_and_drains_microtasks() {
        let clock = Arc::new(ManualClock::new(0));
        let mut event_loop = PiEventLoop::new(ClockHandle::new(clock));
        let _timer = event_loop.set_timeout(0);
        event_loop.enqueue_hostcall_completion("call-1");
        let mut seen = Vec::new();
        let mut drain_calls = 0;
        let result = event_loop.tick(
            |task| seen.push(task.kind),
            || {
                drain_calls += 1;
                drain_calls <= 2
            },
        );
        assert!(result.ran_macrotask);
        assert_eq!(result.microtasks_drained, 2);
        assert_eq!(
            seen[0],
            MacrotaskKind::HostcallComplete {
                call_id: "call-1".into()
            }
        );
    }
    #[test]
    fn timers_order_by_deadline_then_schedule_order() {
        let clock = Arc::new(ManualClock::new(0));
        let mut event_loop = PiEventLoop::new(ClockHandle::new(clock.clone()));
        let first = event_loop.set_timeout(10);
        let second = event_loop.set_timeout(10);
        let early = event_loop.set_timeout(5);
        clock.set(10);
        let mut fired = Vec::new();
        for _ in 0..3 {
            event_loop.tick(
                |task| {
                    if let MacrotaskKind::TimerFired { timer_id } = task.kind {
                        fired.push(timer_id)
                    }
                },
                || false,
            );
        }
        assert_eq!(fired, vec![early, first, second]);
    }
    #[test]
    fn clear_timeout_is_idempotent_and_unknown_ids_do_not_accumulate() {
        let clock = Arc::new(ManualClock::new(0));
        let mut event_loop = PiEventLoop::new(ClockHandle::new(clock));
        assert!(!event_loop.clear_timeout(42));
        let timer = event_loop.set_timeout(1);
        assert!(event_loop.clear_timeout(timer));
        assert!(!event_loop.clear_timeout(timer));
    }
}
