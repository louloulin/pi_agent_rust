//! Core producer/consumer queues shared by agent runtimes.

#![forbid(unsafe_code)]

use pi_ai::model::{ContentBlock, Message, UserContent, UserMessage};
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, OnceLock};

pub const MAX_STEERING_QUEUE_SIZE: usize = 100;
pub const MAX_FOLLOW_UP_QUEUE_SIZE: usize = 100;

#[derive(Debug, Clone)]
pub struct QueuedAgentMessage {
    message: Message,
    keyword_scan_source: Option<String>,
    persistence_identity: Arc<OnceLock<QueuedPersistenceIdentity>>,
}

#[derive(Debug, Clone)]
struct QueuedPersistenceIdentity {
    entry_id: String,
    timestamp: String,
    parent_id: Option<String>,
}

impl QueuedAgentMessage {
    #[must_use]
    pub fn authored(message: Message, keyword_scan_source: impl Into<String>) -> Self {
        let keyword_scan_source =
            matches!(&message, Message::User(_)).then(|| keyword_scan_source.into());
        Self {
            message,
            keyword_scan_source,
            persistence_identity: Arc::new(OnceLock::new()),
        }
    }

    #[must_use]
    pub fn from_authored_message(message: Message) -> Self {
        let keyword_scan_source = match &message {
            Message::User(UserMessage { content, .. }) => Some(match content {
                UserContent::Text(text) => text.clone(),
                UserContent::Blocks(blocks) => blocks
                    .iter()
                    .filter_map(|block| match block {
                        ContentBlock::Text(text) => Some(text.text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            }),
            _ => None,
        };
        Self {
            message,
            keyword_scan_source,
            persistence_identity: Arc::new(OnceLock::new()),
        }
    }

    #[must_use]
    pub fn generated(message: Message) -> Self {
        Self {
            message,
            keyword_scan_source: None,
            persistence_identity: Arc::new(OnceLock::new()),
        }
    }

    #[must_use]
    pub const fn message(&self) -> &Message {
        &self.message
    }

    #[must_use]
    pub fn bind_persistence_identity(
        &self,
        parent_id: Option<String>,
    ) -> (String, String, Option<String>) {
        let identity = self
            .persistence_identity
            .get_or_init(|| QueuedPersistenceIdentity {
                entry_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                parent_id,
            });
        (
            identity.entry_id.clone(),
            identity.timestamp.clone(),
            identity.parent_id.clone(),
        )
    }

    #[must_use]
    pub fn persistence_entry_id(&self) -> Option<&str> {
        self.persistence_identity
            .get()
            .map(|identity| identity.entry_id.as_str())
    }

    #[must_use]
    pub fn keyword_scan_source(&self) -> Option<&str> {
        self.keyword_scan_source.as_deref()
    }

    #[must_use]
    pub fn text_for_display(&self) -> Option<&str> {
        self.keyword_scan_source().or_else(|| match &self.message {
            Message::User(user) => match &user.content {
                UserContent::Text(text) => Some(text.as_str()),
                UserContent::Blocks(blocks) => blocks.iter().find_map(|block| match block {
                    ContentBlock::Text(text) => Some(text.text.as_str()),
                    _ => None,
                }),
            },
            _ => None,
        })
    }

    #[must_use]
    pub fn into_message(self) -> Message {
        self.message
    }

    fn shares_persistence_identity(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.persistence_identity, &other.persistence_identity)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueMode {
    All,
    OneAtATime,
}

impl QueueMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::OneAtATime => "one-at-a-time",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum QueueKind {
    Steering,
    FollowUp,
}

#[derive(Debug, Clone)]
struct SequencedQueuedMessage {
    delivery: QueuedAgentMessage,
    job_owner_session_id: Option<String>,
}

#[derive(Debug)]
pub struct MessageQueue {
    steering: VecDeque<SequencedQueuedMessage>,
    follow_up: VecDeque<SequencedQueuedMessage>,
    pub steering_mode: QueueMode,
    pub follow_up_mode: QueueMode,
    next_seq: u64,
}

impl MessageQueue {
    pub const fn new(steering_mode: QueueMode, follow_up_mode: QueueMode) -> Self {
        Self {
            steering: VecDeque::new(),
            follow_up: VecDeque::new(),
            steering_mode,
            follow_up_mode,
            next_seq: 0,
        }
    }
    pub const fn set_modes(&mut self, steering_mode: QueueMode, follow_up_mode: QueueMode) {
        self.steering_mode = steering_mode;
        self.follow_up_mode = follow_up_mode;
    }
    pub fn pending_count(&self) -> usize {
        self.steering.len() + self.follow_up.len()
    }
    fn next_entry(
        &mut self,
        delivery: QueuedAgentMessage,
        owner: Option<String>,
    ) -> SequencedQueuedMessage {
        self.next_seq = self.next_seq.saturating_add(1);
        SequencedQueuedMessage {
            delivery,
            job_owner_session_id: owner,
        }
    }
    fn push(&mut self, kind: QueueKind, delivery: QueuedAgentMessage) -> u64 {
        let entry = self.next_entry(delivery, None);
        let seq = self.next_seq.saturating_sub(1);
        match kind {
            QueueKind::Steering => {
                if self.steering.len() >= MAX_STEERING_QUEUE_SIZE {
                    self.steering.pop_front();
                }
                self.steering.push_back(entry);
            }
            QueueKind::FollowUp => {
                if self
                    .follow_up
                    .iter()
                    .filter(|entry| entry.job_owner_session_id.is_none())
                    .count()
                    >= MAX_FOLLOW_UP_QUEUE_SIZE
                {
                    if let Some(index) = self
                        .follow_up
                        .iter()
                        .position(|entry| entry.job_owner_session_id.is_none())
                    {
                        self.follow_up.remove(index);
                    }
                }
                self.follow_up.push_back(entry);
            }
        }
        seq
    }
    pub fn push_steering(&mut self, delivery: QueuedAgentMessage) -> u64 {
        self.push(QueueKind::Steering, delivery)
    }
    pub fn push_steering_lossless(&mut self, delivery: QueuedAgentMessage) -> u64 {
        let entry = self.next_entry(delivery, None);
        let seq = self.next_seq - 1;
        self.steering.push_back(entry);
        seq
    }
    pub fn restore_steering_front_lossless(&mut self, deliveries: Vec<QueuedAgentMessage>) {
        let mut restored = deliveries
            .into_iter()
            .map(|delivery| self.next_entry(delivery, None))
            .collect::<VecDeque<_>>();
        restored.append(&mut self.steering);
        self.steering = restored;
    }
    pub fn push_follow_up(&mut self, delivery: QueuedAgentMessage) -> u64 {
        self.push(QueueKind::FollowUp, delivery)
    }
    pub fn push_follow_up_lossless(&mut self, delivery: QueuedAgentMessage) -> u64 {
        let entry = self.next_entry(delivery, None);
        let seq = self.next_seq - 1;
        self.follow_up.push_back(entry);
        seq
    }
    pub fn push_job_follow_up_lossless(
        &mut self,
        owner: String,
        delivery: QueuedAgentMessage,
    ) -> u64 {
        let entry = self.next_entry(delivery, Some(owner));
        let seq = self.next_seq - 1;
        self.follow_up.push_back(entry);
        seq
    }
    pub fn has_job_follow_up(&self) -> bool {
        self.follow_up
            .iter()
            .any(|entry| entry.job_owner_session_id.is_some())
    }
    pub fn take_job_follow_ups_except(
        &mut self,
        owner: Option<&str>,
    ) -> Vec<(String, QueuedAgentMessage)> {
        let mut retained = VecDeque::new();
        let mut released = Vec::new();
        while let Some(entry) = self.follow_up.pop_front() {
            if entry
                .job_owner_session_id
                .as_deref()
                .is_some_and(|id| Some(id) != owner)
            {
                released.push((entry.job_owner_session_id.expect("checked"), entry.delivery));
            } else {
                retained.push_back(entry);
            }
        }
        self.follow_up = retained;
        released
    }
    pub fn pop_steering(&mut self) -> Vec<QueuedAgentMessage> {
        self.pop_kind(QueueKind::Steering)
    }
    pub fn pop_follow_up(&mut self) -> Vec<QueuedAgentMessage> {
        self.pop_kind(QueueKind::FollowUp)
    }
    pub fn follow_up_batch_len(&self) -> usize {
        match self.follow_up_mode {
            QueueMode::All => self.follow_up.len(),
            QueueMode::OneAtATime => usize::from(!self.follow_up.is_empty()),
        }
    }
    fn pop_kind(&mut self, kind: QueueKind) -> Vec<QueuedAgentMessage> {
        let (queue, mode) = match kind {
            QueueKind::Steering => (&mut self.steering, self.steering_mode),
            QueueKind::FollowUp => (&mut self.follow_up, self.follow_up_mode),
        };
        match mode {
            QueueMode::All => queue.drain(..).map(|entry| entry.delivery).collect(),
            QueueMode::OneAtATime => queue
                .pop_front()
                .into_iter()
                .map(|entry| entry.delivery)
                .collect(),
        }
    }
    pub fn discard_persistence_ids(&mut self, ids: &HashSet<String>) -> usize {
        let before = self.pending_count();
        self.steering.retain(|entry| {
            !entry
                .delivery
                .persistence_entry_id()
                .is_some_and(|id| ids.contains(id))
        });
        self.follow_up.retain(|entry| {
            !entry
                .delivery
                .persistence_entry_id()
                .is_some_and(|id| ids.contains(id))
        });
        before - self.pending_count()
    }
    pub fn contains_delivery(&self, delivery: &QueuedAgentMessage) -> bool {
        self.steering
            .iter()
            .chain(&self.follow_up)
            .any(|entry| entry.delivery.shares_persistence_identity(delivery))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pi_ai::model::{Message, UserContent, UserMessage};
    fn message(text: &str) -> QueuedAgentMessage {
        QueuedAgentMessage::from_authored_message(Message::User(UserMessage {
            content: UserContent::Text(text.to_string()),
            timestamp: 0,
        }))
    }
    #[test]
    fn one_at_a_time_preserves_order() {
        let mut queue = MessageQueue::new(QueueMode::OneAtATime, QueueMode::OneAtATime);
        queue.push_steering(message("a"));
        queue.push_steering(message("b"));
        assert_eq!(queue.pop_steering().len(), 1);
        assert_eq!(queue.pop_steering().len(), 1);
    }
    #[test]
    fn all_mode_drains_queue() {
        let mut queue = MessageQueue::new(QueueMode::All, QueueMode::OneAtATime);
        queue.push_steering(message("a"));
        queue.push_steering(message("b"));
        assert_eq!(queue.pop_steering().len(), 2);
    }
}
