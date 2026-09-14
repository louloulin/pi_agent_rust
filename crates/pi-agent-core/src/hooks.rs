//! Lightweight, deterministic lifecycle hooks for the agent core.
//!
//! Hooks intentionally operate on a small mutable event rather than importing
//! session, provider, or CLI types. This keeps the core reusable by alternate
//! frontends and makes extension adapters responsible for their own translation.

use std::collections::HashMap;
use std::fmt;

/// Lifecycle points exposed by the agent core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookKind {
    BeforeTurn,
    AfterTurn,
    BeforeTool,
    AfterTool,
}

/// Mutable event passed to registered handlers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookEvent {
    pub kind: HookKind,
    pub payload: String,
}

impl HookEvent {
    #[must_use]
    pub fn new(kind: HookKind, payload: impl Into<String>) -> Self {
        Self {
            kind,
            payload: payload.into(),
        }
    }
}

/// A hook rejection or execution failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookError {
    pub message: String,
}

impl HookError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for HookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for HookError {}

type Handler = Box<dyn FnMut(&mut HookEvent) -> Result<(), HookError> + Send + 'static>;

/// Ordered, in-process hook registry.
#[derive(Default)]
pub struct HookRegistry {
    handlers: HashMap<HookKind, Vec<Handler>>,
}

impl HookRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<F>(&mut self, kind: HookKind, handler: F)
    where
        F: FnMut(&mut HookEvent) -> Result<(), HookError> + Send + 'static,
    {
        self.handlers
            .entry(kind)
            .or_default()
            .push(Box::new(handler));
    }

    pub fn dispatch(&mut self, event: &mut HookEvent) -> Result<(), HookError> {
        if let Some(handlers) = self.handlers.get_mut(&event.kind) {
            for handler in handlers {
                handler(event)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatches_handlers_in_registration_order() {
        let mut hooks = HookRegistry::new();
        hooks.register(HookKind::BeforeTurn, |event| {
            event.payload.push_str("-first");
            Ok(())
        });
        hooks.register(HookKind::BeforeTurn, |event| {
            event.payload.push_str("-second");
            Ok(())
        });
        let mut event = HookEvent::new(HookKind::BeforeTurn, "start");
        hooks.dispatch(&mut event).unwrap();
        assert_eq!(event.payload, "start-first-second");
    }

    #[test]
    fn only_handlers_for_the_event_kind_are_called() {
        let mut hooks = HookRegistry::new();
        hooks.register(HookKind::BeforeTurn, |event| {
            event.payload.push_str("-before");
            Ok(())
        });
        hooks.register(HookKind::AfterTurn, |event| {
            event.payload.push_str("-after");
            Ok(())
        });
        let mut event = HookEvent::new(HookKind::AfterTurn, "start");
        hooks.dispatch(&mut event).unwrap();
        assert_eq!(event.payload, "start-after");
    }

    #[test]
    fn handler_errors_stop_dispatch_and_are_reported() {
        let mut hooks = HookRegistry::new();
        hooks.register(HookKind::BeforeTool, |_| Err(HookError::new("denied")));
        hooks.register(HookKind::BeforeTool, |event| {
            event.payload.push_str("-unreachable");
            Ok(())
        });
        let mut event = HookEvent::new(HookKind::BeforeTool, "start");
        let error = hooks.dispatch(&mut event).expect_err("hook should fail");
        assert_eq!(error.message, "denied");
        assert_eq!(event.payload, "start");
    }
}
