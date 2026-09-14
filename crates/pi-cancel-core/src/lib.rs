//! Runtime-independent cancellation and deadline primitives.
//!
//! This crate deliberately contains no agent, provider, or UI code. It is the
//! shared seam for request cancellation: a cheap cloneable signal, an owning
//! handle, and a deadline-aware token that can be passed across runtime layers.

#![forbid(unsafe_code)]

use asupersync::sync::Notify;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct AbortHandle {
    inner: Arc<AbortState>,
}

#[derive(Debug, Clone)]
pub struct AbortSignal {
    inner: Arc<AbortState>,
}

#[derive(Debug)]
struct AbortState {
    aborted: AtomicBool,
    notify: Notify,
}

impl AbortHandle {
    #[must_use]
    pub fn new() -> (Self, AbortSignal) {
        let inner = Arc::new(AbortState {
            aborted: AtomicBool::new(false),
            notify: Notify::new(),
        });
        (Self { inner: Arc::clone(&inner) }, AbortSignal { inner })
    }

    pub fn abort(&self) {
        if !self.inner.aborted.swap(true, Ordering::SeqCst) {
            self.inner.notify.notify_waiters();
        }
    }
}

impl AbortSignal {
    #[must_use]
    pub fn is_aborted(&self) -> bool {
        self.inner.aborted.load(Ordering::SeqCst)
    }

    pub async fn wait(&self) {
        if self.is_aborted() {
            return;
        }
        loop {
            self.inner.notify.notified().await;
            if self.is_aborted() {
                return;
            }
        }
    }
}

/// A cloneable cancellation token with an optional monotonic deadline.
#[derive(Debug, Clone)]
pub struct CancellationToken {
    signal: AbortSignal,
    deadline: Option<Instant>,
}

impl CancellationToken {
    #[must_use]
    pub fn new() -> (AbortHandle, Self) {
        let (handle, signal) = AbortHandle::new();
        (handle, Self { signal, deadline: None })
    }

    #[must_use]
    pub fn from_signal(signal: AbortSignal) -> Self {
        Self { signal, deadline: None }
    }

    #[must_use]
    pub fn with_deadline(self, deadline: Instant) -> Self {
        Self { deadline: Some(deadline), ..self }
    }

    #[must_use]
    pub fn with_timeout(self, timeout: Duration) -> Self {
        self.with_deadline(Instant::now() + timeout)
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.signal.is_aborted() || self.deadline.is_some_and(|deadline| Instant::now() >= deadline)
    }

    #[must_use]
    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    #[must_use]
    pub fn signal(&self) -> &AbortSignal {
        &self.signal
    }

    pub async fn cancelled(&self) {
        self.signal.wait().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abort_signal_propagates() {
        let (handle, signal) = AbortHandle::new();
        assert!(!signal.is_aborted());
        handle.abort();
        handle.abort();
        assert!(signal.is_aborted());
    }

    #[test]
    fn timeout_deadline_is_observable() {
        let (_handle, token) = CancellationToken::new();
        assert!(!token.is_cancelled());
        let expired = token.with_timeout(Duration::ZERO);
        assert!(expired.is_cancelled());
        assert!(expired.deadline().is_some());
    }
}
