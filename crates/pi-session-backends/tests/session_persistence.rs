//! Round 26.2: integration test for the JSONL session persistence layer.
//!
//! Round 18 moved this test from `pi-coding-agent/src/session_test.rs`
//! alongside the `session.rs` it exercises. Round 26.2 re-houses it in
//! `pi-session-backends/tests/` (the upstream home of
//! `@earendil-works/pi-session-backends`). It lives under `tests/` rather
//! than `src/` because compiling it inside the library crate would force a
//! `pi-session-backends → pi-coding-agent` dependency edge and reintroduce
//! the cycle Round 18 had to break. As an external integration test it
//! only pulls in `pi-coding-agent` for the test binary.
//!
//! The test itself is unchanged: it covers the round-trip of a single
//! `User` message through `Session::create_with_dir` → `save` →
//! `open_with_diagnostics`.

use pi_coding_agent::session::{Session, SessionEntry, SessionMessage};

#[test]
fn test_session_save_persistence() {
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .expect("runtime build");

    runtime.block_on(async move {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let session_path = temp_dir.path().join("test_session.jsonl");

        // Create a new session
        let mut session = Session::create_with_dir(Some(temp_dir.path().to_path_buf()));
        session.path = Some(session_path.clone());

        // Add some messages
        session.append_message(SessionMessage::User {
            content: pi_ai::model::UserContent::Text("Hello".to_string()),
            timestamp: Some(0),
        });

        // Save the session
        session.save().await.expect("save session");

        // Check if file exists
        assert!(session_path.exists());

        // Re-open the session
        let (loaded_session, diagnostics) = Session::open_with_diagnostics(
            session_path.to_str().unwrap(),
        )
        .await
        .expect("load session");

        assert!(diagnostics.skipped_entries.is_empty());
        assert_eq!(loaded_session.entries.len(), 1);

        if let SessionEntry::Message(msg) = &loaded_session.entries[0] {
            if let SessionMessage::User { content, .. } = &msg.message {
                if let pi_ai::model::UserContent::Text(text) = content {
                    assert_eq!(text, "Hello");
                } else {
                    unreachable!("Unexpected content type");
                }
            } else {
                unreachable!("Unexpected message type");
            }
        } else {
            unreachable!("Unexpected entry type");
        }
    });
}
