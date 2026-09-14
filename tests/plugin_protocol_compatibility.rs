#![forbid(unsafe_code)]

//! Protocol-only compatibility checks for the pi-mono/pi.dev plugin boundary.
//!
//! These tests intentionally stop at JSON serialization and schema validation:
//! they do not load JavaScript, call providers, or exercise crypto/hostcall
//! implementation details. The fixture is the reviewed wire contract; the
//! assertions below connect it to the public Rust protocol types.

use std::sync::Arc;

use jsonschema::Validator;
use pi::agent::AgentEvent;
use pi::extensions::{ExtensionUiRequest, extension_event_from_agent};
use pi::model::{ContentBlock, TextContent};
use pi::tools::ToolOutput;
use serde_json::{Value, json};

const FIXTURE: &str = include_str!("fixtures/plugin_protocol_compatibility.json");
const PROTOCOL_SCHEMA: &str = include_str!("../docs/schema/extension_protocol.json");

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).expect("plugin compatibility fixture must be valid JSON")
}

fn validate_against_protocol_schema(message: &Value) {
    let schema: Value = serde_json::from_str(PROTOCOL_SCHEMA).expect("protocol schema JSON");
    let validator: Validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("protocol schema must compile");
    let errors: Vec<String> = validator
        .iter_errors(message)
        .map(|error| error.to_string())
        .collect();
    assert!(errors.is_empty(), "protocol message is invalid: {errors:?}");
}

#[test]
fn fixture_declares_the_pi_mono_wire_conventions() {
    let fixture = fixture();
    assert_eq!(fixture["schema"], "pi.plugin_protocol_compatibility.v1");
    assert_eq!(fixture["upstream"], "pi-mono/pi.dev");
    assert_eq!(fixture["wire"]["encoding"], "json-lines");
    assert_eq!(fixture["wire"]["event_type_style"], "snake_case");
    assert_eq!(fixture["wire"]["field_style"], "camelCase");
}

#[test]
fn agent_events_match_fixture_and_extension_dispatch_names() {
    let fixture = fixture();
    let session_id: Arc<str> = "session-1".into();
    let events = vec![
        AgentEvent::AgentStart {
            session_id: Arc::clone(&session_id),
        },
        AgentEvent::TurnStart {
            session_id,
            turn_index: 2,
            timestamp: 1_700_000_000,
        },
        AgentEvent::ToolExecutionStart {
            tool_call_id: "call-1".to_string(),
            tool_name: "read".to_string(),
            args: json!({"path": "README.md"}),
        },
        AgentEvent::ToolExecutionEnd {
            tool_call_id: "call-1".to_string(),
            tool_name: "read".to_string(),
            result: ToolOutput {
                content: vec![ContentBlock::Text(TextContent::new("ok"))],
                details: None,
                is_error: false,
            },
            is_error: false,
        },
    ];

    let expected = fixture["agent_events"]
        .as_array()
        .expect("agent_events must be an array");
    assert_eq!(events.len(), expected.len());

    for (event, expected_case) in events.iter().zip(expected) {
        let serialized = serde_json::to_value(event).expect("AgentEvent serialization");
        assert_eq!(serialized, expected_case["expected"]);

        let (name, payload) = extension_event_from_agent(event).expect("event is dispatchable");
        assert_eq!(name.to_string(), expected_case["event"]);
        assert_eq!(payload.expect("dispatch payload"), serialized);
    }
}

#[test]
fn extension_ui_requests_preserve_pi_mono_fields_and_authoritative_envelope() {
    let fixture = fixture();
    let expected = fixture["extension_ui"]
        .as_array()
        .expect("extension_ui must be an array");
    let requests = [
        ExtensionUiRequest::new(
            "ui-1",
            "confirm",
            json!({"title": "Continue?", "message": "Use the plugin?"}),
        ),
        ExtensionUiRequest::new(
            "ui-2",
            "select",
            json!({"title": "Pick a provider", "options": ["anthropic", "openai"]}),
        ),
    ];

    for (request, expected_case) in requests.iter().zip(expected) {
        assert_eq!(request.to_rpc_event(), expected_case["expected"]);
    }

    let collision = ExtensionUiRequest::new(
        "authoritative-id",
        "confirm",
        json!({"type": "spoofed", "id": "spoofed", "method": "spoofed", "capabilityPrompt": true}),
    )
    .to_rpc_event();
    assert_eq!(collision["type"], "extension_ui_request");
    assert_eq!(collision["id"], "authoritative-id");
    assert_eq!(collision["method"], "confirm");
    assert_eq!(collision["capabilityPrompt"], false);
}

#[test]
fn protocol_fixture_messages_validate_against_extension_schema() {
    let fixture = fixture();
    let messages = fixture["protocol_messages"]
        .as_array()
        .expect("protocol_messages must be an array");
    assert_eq!(messages.len(), 3);
    for case in messages {
        validate_against_protocol_schema(&case["message"]);
    }
}
