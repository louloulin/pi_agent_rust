use pi_ai::model::{Message, UserContent};

/// Compact context summary from the live agent message list.
///
/// The most recent exchanges, truncated to [`CONTEXT_BUDGET_CHARS`]. Tool
/// noise (calls/results) is summarized as one-liners so the budget buys
/// prose.
#[must_use]
pub fn build_context_summary(messages: &[Message]) -> String {
    // Pieces accumulate newest-first (walking backwards); each message's
    // OWN pieces are appended in reverse so the final flip restores true
    // chronological order within a message too. The budget drops the
    // OLDEST content — the newest exchange is what a side question is
    // usually about.
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
                        pi_ai::model::ContentBlock::Text(t) => {
                            message_pieces.push(format!("assistant: {}", truncate(&t.text, 400)));
                        }
                        pi_ai::model::ContentBlock::ToolCall(call) => {
                            message_pieces.push(format!("assistant ran tool {}", call.name));
                        }
                        _ => {}
                    }
                }
            }
            Message::ToolResult(result) => {
                let first = result.content.iter().find_map(|block| match block {
                    pi_ai::model::ContentBlock::Text(t) => Some(t.text.clone()),
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
            // +1 for the join separator; stop BEFORE exceeding the budget
            // so the newest pieces are never tail-truncated later.
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

    #[test]
    fn context_summary_captures_recent_exchanges_and_tool_noise() {
        let messages = vec![
            Message::User(UserMessage { content: UserContent::Text("fix the flaky test".into()), timestamp: 0 }),
            Message::Assistant(pi_ai::model::AssistantMessage {
                content: vec![pi_ai::model::ContentBlock::ToolCall(pi_ai::model::ToolCall {
                    id: "c1".into(), name: "bash".into(), arguments: serde_json::json!({"command": "cargo test"}), thought_signature: None,
                })], api: "test-api".into(), provider: "test-provider".into(), model: "test-model".into(), ..Default::default()
            }.into()),
            Message::User(UserMessage { content: UserContent::Text("second question".into()), timestamp: 0 }),
        ];
        let summary = build_context_summary(&messages);
        assert!(summary.contains("fix the flaky test"));
        assert!(summary.contains("ran tool bash"));
        assert!(summary.contains("second question"));
    }

    #[test]
    fn context_summary_respects_budget() {
        let summary = build_context_summary(&[Message::User(UserMessage { content: UserContent::Text("x".repeat(10_000)), timestamp: 0 })]);
        assert!(summary.len() <= CONTEXT_BUDGET_CHARS + 32);
    }
}
