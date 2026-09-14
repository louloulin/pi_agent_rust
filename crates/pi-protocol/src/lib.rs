//! Phase-2 aggregator mirroring `@earendil-works/pi-protocol`:
//! JSON-RPC / framing / tail-buffer primitives shared by LSP, DAP, and
//! MCP transports.
//!
//! Round 30.1 began re-housing the framing + tail + jsonrpc modules
//! here. The transport-specific surface (`JsonRpcClient`, `await_completion`,
//! `apply_env_policy`, `reader_loop`, `PendingMap`, `SharedWriter`,
//! `ServerRequestHandler`, `lock`) stays in `pi-coding-agent/src/lsp/jsonrpc.rs`
//! because it threads through `crate::tools::ProcessGuard`,
//! `pi_error::Error`, and `crate::agent_cx::AgentCx` that this leaf crate
//! intentionally avoids.

#![forbid(unsafe_code)]

pub mod framing;
pub mod jsonrpc;
pub mod mcp;
pub mod tail;
pub mod tool_effects;

/// Strict JSON parser rejecting duplicate object keys.
pub mod duplicate_json {
    use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
    use serde::Deserialize;
    use serde_json::Value;

    struct DuplicateValue(Value);

    impl<'de> Deserialize<'de> for DuplicateValue {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> {
            deserializer.deserialize_any(DuplicateVisitor)
        }
    }

    struct DuplicateVisitor;

    impl<'de> Visitor<'de> for DuplicateVisitor {
        type Value = DuplicateValue;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a JSON value without duplicate object keys")
        }
        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> { Ok(DuplicateValue(Value::Bool(v))) }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> { Ok(DuplicateValue(Value::Number(v.into()))) }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> { Ok(DuplicateValue(Value::Number(v.into()))) }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
            serde_json::Number::from_f64(v).map(Value::Number).map(DuplicateValue).ok_or_else(|| E::custom("JSON number must be finite"))
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> { Ok(DuplicateValue(Value::String(v.to_owned()))) }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> { Ok(DuplicateValue(Value::String(v))) }
        fn visit_none<E>(self) -> Result<Self::Value, E> { Ok(DuplicateValue(Value::Null)) }
        fn visit_unit<E>(self) -> Result<Self::Value, E> { Ok(DuplicateValue(Value::Null)) }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut values = Vec::new(); while let Some(v) = seq.next_element::<DuplicateValue>()? { values.push(v.0); }
            Ok(DuplicateValue(Value::Array(values)))
        }
        fn visit_map<A: MapAccess<'de>>(self, mut entries: A) -> Result<Self::Value, A::Error> {
            let mut object = serde_json::Map::new();
            while let Some(key) = entries.next_key::<String>()? {
                let value = entries.next_value::<DuplicateValue>()?;
                if object.insert(key, value.0).is_some() { return Err(<A::Error as serde::de::Error>::custom("duplicate JSON object key")); }
            }
            Ok(DuplicateValue(Value::Object(object)))
        }
    }

    pub fn parse_json_rejecting_duplicate_keys(content: &str) -> serde_json::Result<Value> {
        let mut deserializer = serde_json::Deserializer::from_str(content);
        let value = DuplicateValue::deserialize(&mut deserializer)?.0;
        deserializer.end()?;
        Ok(value)
    }
}
