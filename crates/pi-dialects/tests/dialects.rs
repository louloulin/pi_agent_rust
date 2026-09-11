//! Round-trip and behavior tests for the per-model tool-call dialect layer.

use pi_dialects::{
    dialect_for_model, extract_text_tool_calls, strip_candidates, Dialect, RepairCandidate,
    RepairLedger,
};
use serde_json::json;

#[test]
fn dialect_for_anthropic_is_native() {
    assert_eq!(dialect_for_model("anthropic", "claude-3-5-sonnet"), Dialect::Native);
}

#[test]
fn dialect_for_openai_is_native_by_default() {
    assert_eq!(dialect_for_model("openai", "gpt-4o"), Dialect::Native);
}

#[test]
fn dialect_round_trips_through_serde() {
    for d in [Dialect::Native, Dialect::Harmony, Dialect::Xmlish] {
        let s = serde_json::to_string(&d).expect("serialize");
        let back: Dialect = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(back, d);
    }
}

#[test]
fn strip_candidates_removes_known_fences() {
    let text = "before <tool name=\"foo\"/> after";
    let cands = vec![RepairCandidate {
        name: "foo".to_string(),
        arguments: json!({}),
        start: 10,
        end: 25,
    }];
    let s = strip_candidates(text, &cands);
    assert!(!s.contains("foo"));
    assert!(s.contains("before"));
    assert!(s.contains("after"));
}

#[test]
fn extract_text_tool_calls_rejects_unknown_tool_name() {
    let text = r#"<tool_call>
{"name": "unknown_tool", "arguments": {}}
</tool_call>"#;
    let entries = extract_text_tool_calls(text, &|name| matches!(name, "bash" | "read"));
    // The candidate is rejected because "unknown_tool" is not registered.
    assert!(entries.is_empty(), "expected no candidates, got {entries:?}");
}

#[test]
fn extract_text_tool_calls_accepts_known_tool_name() {
    let text = r#"<tool_call>
{"name": "bash", "arguments": {"cmd": "ls"}}
</tool_call>"#;
    let entries = extract_text_tool_calls(text, &|name| matches!(name, "bash" | "read"));
    assert!(!entries.is_empty(), "expected at least one candidate");
    assert_eq!(entries[0].name, "bash");
    assert_eq!(entries[0].arguments["cmd"], "ls");
}

#[test]
fn repair_ledger_records_three_calls() {
    let mut ledger = RepairLedger::default();
    ledger.record("bash", 16, 100);
    ledger.record("read", 8, 92);
    ledger.record("grep", 12, 80);
    assert_eq!(ledger.entries.len(), 3);
    assert_eq!(ledger.entries[0].tool, "bash");
    assert_eq!(ledger.entries[1].stripped_bytes, 8);
    assert_eq!(ledger.entries[2].remaining_text_bytes, 80);
}
