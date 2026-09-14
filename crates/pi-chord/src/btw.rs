//! Ephemeral side-question context shaping shared by the TUI and agent layers.

use pi_ai::model::{ContentBlock, Message};

/// Maximum number of characters included in a side-question context.
pub const CONTEXT_BUDGET_CHARS: usize = 4_000;

/// Build a compact chronological summary from the most recent messages.
#[must_use]
pub fn build_context_summary(messages: &[Message]) -> String {
    let mut pieces = Vec::new();
    let mut used = 0usize;

    for message in messages.iter().rev() {
        let mut message_pieces = Vec::new();
        match message {
            Message::User(user) => {
                if let pi_ai::model::UserContent::Text(text) = &user.content {
                    message_pieces.push(format!("user: {}", truncate(text, 400)));
                }
            }
            Message::Assistant(assistant) => {
                for block in &assistant.content {
                    match block {
                        ContentBlock::Text(text) => {
                            message_pieces.push(format!("assistant: {}", truncate(&text.text, 400)));
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
                    ContentBlock::Text(text) => Some(text.text.as_str()),
                    _ => None,
                });
                message_pieces.push(format!(
                    "tool {}: {}",
                    result.tool_name,
                    truncate(first.unwrap_or(""), 160)
                ));
            }
            Message::Custom(_) => {}
        }

        for piece in message_pieces.into_iter().rev() {
            if used + piece.len() + 1 > CONTEXT_BUDGET_CHARS {
                pieces.reverse();
                return pieces.join("\n");
            }
            used += piece.len() + 1;
            pieces.push(piece);
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
    use pi_ai::model::{Message, UserContent, UserMessage};

    #[test]
    fn summary_is_bounded_and_keeps_latest_message() {
        let messages = vec![
            Message::User(UserMessage { content: UserContent::Text("old".into()), timestamp: 0 }),
            Message::User(UserMessage { content: UserContent::Text("latest".into()), timestamp: 0 }),
        ];
        let summary = build_context_summary(&messages);
        assert!(summary.contains("user: latest"));
        assert!(summary.len() <= CONTEXT_BUDGET_CHARS);
    }
}
