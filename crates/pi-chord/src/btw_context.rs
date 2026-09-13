//! Context summarization for `/btw` ephemeral side questions.

use pi_ai::model::{ContentBlock, Message, UserContent};

const CONTEXT_BUDGET_CHARS: usize = 4_000;

/// Compact the newest part of a conversation for an ephemeral side question.
#[must_use]
pub fn build_context_summary(messages: &[Message]) -> String {
    let mut pieces: Vec<String> = Vec::new();
    let mut used = 0usize;
    for message in messages.iter().rev() {
        let mut message_pieces: Vec<String> = Vec::new();
        match message {
            Message::User(user) => {
                if let UserContent::Text(text) = &user.content {
                    message_pieces.push(format!("user: {}", truncate(text, 400)));
                }
            }
            Message::Assistant(assistant) => {
                for block in &assistant.content {
                    match block {
                        ContentBlock::Text(text) => {
                            message_pieces
                                .push(format!("assistant: {}", truncate(&text.text, 400)));
                        }
                        ContentBlock::ToolCall(call) => {
                            message_pieces.push(format!("assistant ran tool {}", call.name));
                        }
                        _ => {}
                    }
                }
            }
            Message::ToolResult(result) => {
                let first = result.content.iter().find_map(|block| match block {
                    ContentBlock::Text(text) => Some(text.text.clone()),
                    _ => None,
                });
                message_pieces.push(format!(
                    "tool {}: {}",
                    result.tool_name,
                    truncate(first.as_deref().unwrap_or(""), 160)
                ));
            }
            Message::Custom(_) => {}
        }
        let mut over_budget = false;
        for piece in message_pieces.into_iter().rev() {
            if used + piece.len() + 1 > CONTEXT_BUDGET_CHARS {
                over_budget = true;
                break;
            }
            used += piece.len() + 1;
            pieces.push(piece);
        }
        if over_budget {
            break;
        }
    }
    pieces.reverse();
    pieces.join("\n")
}

fn truncate(text: &str, limit: usize) -> &str {
    match text.char_indices().nth(limit) {
        Some((index, _)) => &text[..index],
        None => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pi_ai::model::{AssistantMessage, ToolCall, UserMessage};

    #[test]
    fn summary_keeps_recent_user_and_tool_activity() {
        let messages = vec![
            Message::User(UserMessage {
                content: UserContent::Text("fix the flaky test".into()),
                timestamp: 0,
            }),
            Message::Assistant(
                AssistantMessage {
                    content: vec![ContentBlock::ToolCall(ToolCall {
                        id: "c1".into(),
                        name: "bash".into(),
                        arguments: serde_json::json!({"command": "cargo test"}),
                        thought_signature: None,
                    })],
                    api: "test-api".into(),
                    provider: "test-provider".into(),
                    model: "test-model".into(),
                    ..Default::default()
                }
                .into(),
            ),
            Message::User(UserMessage {
                content: UserContent::Text("second question".into()),
                timestamp: 0,
            }),
        ];
        let summary = build_context_summary(&messages);
        assert!(summary.contains("fix the flaky test"), "{summary}");
        assert!(summary.contains("ran tool bash"), "{summary}");
        assert!(summary.contains("second question"), "{summary}");
    }

    #[test]
    fn summary_obeys_budget_for_large_messages() {
        let messages = vec![Message::User(UserMessage {
            content: UserContent::Text("x".repeat(10_000)),
            timestamp: 0,
        })];
        assert!(build_context_summary(&messages).len() <= CONTEXT_BUDGET_CHARS + 32);
    }
}
