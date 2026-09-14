use pi_ai::model::{Message, UserContent, UserMessage};
use pi_chord::btw::build_context_summary;

#[test]
fn context_summary_keeps_recent_user_exchange() {
    let messages = vec![Message::User(UserMessage {
        content: UserContent::Text("older".into()),
        timestamp: 0,
    }), Message::User(UserMessage {
        content: UserContent::Text("latest question".into()),
        timestamp: 0,
    })];

    let summary = build_context_summary(&messages);

    assert!(summary.contains("user: latest question"));
}
