//! Dependency-light event primitives shared by Pi runtimes.
//!
//! The core deliberately knows nothing about the agent, UI, or async runtime. It
//! owns subscription identity, filtering, and snapshot-before-dispatch semantics.
use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Stable event type name used by [`EventFilter`].
pub trait EventType {
    fn event_type(&self) -> &'static str;
}

/// A composable name filter. An empty filter matches every event.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EventFilter {
    exact: BTreeSet<String>,
    prefixes: Vec<String>,
}

impl EventFilter {
    #[must_use]
    pub fn exact(name: impl Into<String>) -> Self {
        Self { exact: [name.into()].into_iter().collect(), prefixes: Vec::new() }
    }

    #[must_use]
    pub fn any<I, S>(names: I) -> Self
    where I: IntoIterator<Item = S>, S: Into<String> {
        Self { exact: names.into_iter().map(Into::into).collect(), prefixes: Vec::new() }
    }

    #[must_use]
    pub fn prefix(prefix: impl Into<String>) -> Self {
        Self { exact: BTreeSet::new(), prefixes: vec![prefix.into()] }
    }

    #[must_use]
    pub fn or_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefixes.push(prefix.into()); self
    }

    #[must_use]
    pub fn matches<T: EventType>(&self, event: &T) -> bool {
        self.exact.is_empty() && self.prefixes.is_empty()
            || self.exact.contains(event.event_type())
            || self.prefixes.iter().any(|p| event.event_type().starts_with(p))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    #[must_use]
    pub const fn get(self) -> u64 { self.0 }
}

type Listener<T> = Arc<dyn Fn(&T) + Send + Sync + 'static>;
struct Entry<T> { filter: EventFilter, listener: Listener<T> }

/// Thread-safe event bus. Listeners are cloned before callbacks run, so a
/// callback may subscribe or unsubscribe without holding the bus lock.
#[derive(Clone)]
pub struct EventBus<T> {
    next_id: Arc<AtomicU64>,
    listeners: Arc<Mutex<HashMap<SubscriptionId, Entry<T>>>>,
}

impl<T> Default for EventBus<T> { fn default() -> Self { Self::new() } }

impl<T> EventBus<T> {
    #[must_use]
    pub fn new() -> Self {
        Self { next_id: Arc::new(AtomicU64::new(1)), listeners: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn subscribe(&self, listener: impl Fn(&T) + Send + Sync + 'static) -> SubscriptionId
    where T: EventType {
        self.subscribe_filtered(EventFilter::default(), listener)
    }

    pub fn subscribe_filtered(&self, filter: EventFilter, listener: impl Fn(&T) + Send + Sync + 'static) -> SubscriptionId {
        let id = SubscriptionId(self.next_id.fetch_add(1, Ordering::Relaxed));
        self.listeners.lock().unwrap_or_else(|e| e.into_inner()).insert(id, Entry { filter, listener: Arc::new(listener) });
        id
    }

    pub fn unsubscribe(&self, id: SubscriptionId) -> bool {
        self.listeners.lock().unwrap_or_else(|e| e.into_inner()).remove(&id).is_some()
    }

    pub fn publish(&self, event: &T)
    where T: EventType {
        let listeners: Vec<_> = self.listeners.lock().unwrap_or_else(|e| e.into_inner()).values()
            .filter(|entry| entry.filter.matches(event)).map(|entry| Arc::clone(&entry.listener)).collect();
        for listener in listeners { listener(event); }
    }

    #[must_use]
    pub fn len(&self) -> usize { self.listeners.lock().unwrap_or_else(|e| e.into_inner()).len() }
    #[must_use]
    pub fn is_empty(&self) -> bool { self.len() == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Clone)] struct E(&'static str);
    impl EventType for E { fn event_type(&self) -> &'static str { self.0 } }
    #[test]
    fn filtered_dispatch_and_unsubscribe() {
        let bus = EventBus::new(); let seen = Arc::new(Mutex::new(Vec::new()));
        let copy = Arc::clone(&seen);
        let id = bus.subscribe_filtered(EventFilter::prefix("tool."), move |e: &E| copy.lock().unwrap().push(e.0));
        bus.publish(&E("agent.start")); bus.publish(&E("tool.start"));
        assert_eq!(*seen.lock().unwrap(), ["tool.start"]); assert!(bus.unsubscribe(id)); assert!(bus.is_empty());
    }
}
