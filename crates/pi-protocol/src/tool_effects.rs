//! Protocol-level coarse tool side-effect declarations.
//!
//! Kept independent from tool implementations so plan gates, schedulers,
//! extensions, and protocol clients can share the same compatibility rules.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolEffects {
    bits: u8,
}

impl ToolEffects {
    const READ: u8 = 1 << 0;
    const WRITE: u8 = 1 << 1;
    const APPEND: u8 = 1 << 2;
    const NETWORK: u8 = 1 << 3;
    const PROCESS: u8 = 1 << 4;
    const BARRIER: u8 = Self::WRITE | Self::APPEND | Self::PROCESS;

    #[must_use] pub const fn read() -> Self { Self { bits: Self::READ } }
    #[must_use] pub const fn write() -> Self { Self { bits: Self::WRITE } }
    #[must_use] pub const fn append() -> Self { Self { bits: Self::APPEND } }
    #[must_use] pub const fn network() -> Self { Self { bits: Self::NETWORK } }
    #[must_use] pub const fn process() -> Self { Self { bits: Self::PROCESS } }
    #[must_use] pub const fn union(self, other: Self) -> Self { Self { bits: self.bits | other.bits } }
    #[must_use] pub const fn reads(self) -> bool { self.bits & Self::READ != 0 }
    #[must_use] pub const fn writes(self) -> bool { self.bits & Self::WRITE != 0 }
    #[must_use] pub const fn appends(self) -> bool { self.bits & Self::APPEND != 0 }
    #[must_use] pub const fn networks(self) -> bool { self.bits & Self::NETWORK != 0 }
    #[must_use] pub const fn processes(self) -> bool { self.bits & Self::PROCESS != 0 }

    #[must_use]
    pub fn labels(self) -> Vec<&'static str> {
        let mut labels = Vec::with_capacity(5);
        if self.reads() { labels.push("read"); }
        if self.writes() { labels.push("write"); }
        if self.appends() { labels.push("append"); }
        if self.networks() { labels.push("network"); }
        if self.processes() { labels.push("process"); }
        labels
    }

    #[must_use] pub const fn parallel_safe(self) -> bool { self.bits != 0 && self.bits & Self::BARRIER == 0 }
    #[must_use] pub const fn compatible_with(self, other: Self) -> bool { self.parallel_safe() && other.parallel_safe() }
}

#[cfg(test)]
mod tests {
    use super::ToolEffects;
    #[test]
    fn barrier_and_labels_match_tool_scheduler_contract() {
        let effects = ToolEffects::read().union(ToolEffects::network());
        assert_eq!(effects.labels(), vec!["read", "network"]);
        assert!(effects.parallel_safe());
        assert!(!effects.compatible_with(ToolEffects::write()));
    }
}
