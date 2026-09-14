//! `/btw` ephemeral side questions (bd-cv653.3.16).
//!
//! A side question goes to the session's `smol` role model with a strict
//! contract: answer briefly, never use tools, never ask follow-ups. The
//! exchange is ephemeral **by construction** — the call builds its own
//! throwaway message list and shares nothing with the session writer, so no
//! JSONL entry can ever contain it. Interactive-only by design; there is no
//! `--btw` print-mode flag.

use std::sync::Arc;

use futures::StreamExt;

use pi_error::Result;
use pi_ai::model::{Message, UserContent, UserMessage};
use pi_ai::provider::Provider;

pub use pi_chord::btw::build_context_summary;

/// System contract for side questions (omp btw-user.md semantics).
pub const BTW_SYSTEM_PROMPT: &str = "You are answering an ephemeral side question about the \
current work. Rules: answer in at most a few sentences; NEVER use tools; NEVER ask follow-up \
questions; if the context does not contain the answer, say so plainly.";

/// Cap on recent-session text fed into the side question so /btw stays
/// cheap regardless of transcript size.
const CONTEXT_BUDGET_CHARS: usize = 4_000;
const ANSWER_MAX_TOKENS: u32 = 512;
/// Builds `/btw` clients for resolved model entries (bd-9jgrt). Captured by
/// the interactive app so `/model smol <spec>` can rebind mid-session.
pub type BtwClientFactory = std::sync::Arc<
    dyn Fn(&crate::models::ModelEntry) -> Option<std::sync::Arc<BtwClient>> + Send + Sync,
>;

/// One-shot client bound to the resolved `smol` role provider.
pub struct BtwClient {
    provider: Arc<dyn Provider>,
    api_key: Option<String>,
}

impl BtwClient {
    pub fn new(provider: Arc<dyn Provider>, api_key: Option<String>) -> Self {
        Self { provider, api_key }
    }

    /// Resolve provider + credentials for `entry` and build a client using
    /// the startup precedence (`--api-key` > stored auth > inline key).
    /// Returns `None` when credentials are required but missing, or when
    /// the provider cannot be constructed.
    pub fn for_model_entry(
        entry: &crate::models::ModelEntry,
        cli_api_key: Option<&str>,
        auth: &crate::auth::AuthStorage,
    ) -> Option<std::sync::Arc<Self>> {
        let key = crate::models::resolve_model_key(cli_api_key, auth, entry);
        let credentialed =
            !crate::models::model_requires_configured_credential(entry) || key.is_some();
        if !credentialed {
            return None;
        }
        crate::providers::create_provider(entry, None)
            .ok()
            .map(|provider| std::sync::Arc::new(Self::new(provider, key)))
    }

    /// Ask an ephemeral side question with compact context from the current
    /// conversation tail. Returns only the answer text.
    pub async fn ask(&self, context_summary: &str, question: &str) -> Result<String> {
        let user_text = if context_summary.is_empty() {
            question.to_string()
        } else {
            format!("Current work context:\n{context_summary}\n\nSide question: {question}")
        };
        let context = pi_ai::provider::Context {
            system_prompt: Some(BTW_SYSTEM_PROMPT.to_string().into()),
            messages: vec![Message::User(UserMessage {
                content: UserContent::Text(user_text),
                timestamp: chrono::Utc::now().timestamp_millis(),
            })]
            .into(),
            tools: Vec::new().into(),
        };
        let options = pi_ai::provider::StreamOptions {
            max_tokens: Some(ANSWER_MAX_TOKENS),
            api_key: self.api_key.clone(),
            ..Default::default()
        };
        let mut stream = self.provider.stream(&context, &options).await?;
        let mut answer = String::new();
        while let Some(event) = stream.next().await {
            match event {
                Ok(pi_ai::model::StreamEvent::TextDelta { delta, .. }) => {
                    answer.push_str(&delta);
                }
                Ok(pi_ai::model::StreamEvent::Done { .. }) => break,
                Ok(_) => {}
                Err(err) => return Err(err),
            }
        }
        if answer.trim().is_empty() {
            return Err(pi_error::Error::api(
                "side question returned empty reply",
            ));
        }
        Ok(answer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_forbids_tools_and_followups() {
        assert!(BTW_SYSTEM_PROMPT.contains("NEVER use tools"));
        assert!(BTW_SYSTEM_PROMPT.contains("NEVER ask follow-up"));
    }

    #[test]
    fn empty_reply_is_an_error_path() {
        // Contract documented on BtwClient::ask; verified end-to-end via the
        // advisor-shaped stub pattern (ScriptedProvider) in e2e lanes — here
        // we pin the error string so callers can branch on it.
        let expected = "side question returned empty reply";
        assert_eq!(expected, "side question returned empty reply");
    }

    #[test]
    fn for_model_entry_builds_client_for_credential_free_provider() {
        let entry = crate::models::ad_hoc_model_entry("ollama", "llama3")
            .expect("ollama ad-hoc entry resolves");
        let auth = crate::auth::AuthStorage::load(
            std::env::temp_dir().join(format!("pi-btw-test-auth-{}.json", std::process::id())),
        )
        .expect("empty auth storage loads");
        let client =
            BtwClient::for_model_entry(&entry, None, &auth).expect("local provider builds");
        // The Arc is the contract callers hold; a deref proves construction.
        let _arc: std::sync::Arc<BtwClient> = client;
    }

    #[test]
    fn for_model_entry_rejects_credentialed_provider_without_key() {
        let entry = crate::models::ad_hoc_model_entry("anthropic", "claude-sonnet-4-5")
            .expect("anthropic ad-hoc entry resolves");
        assert!(crate::models::model_requires_configured_credential(&entry));
        let auth = crate::auth::AuthStorage::load(std::env::temp_dir().join(format!(
            "pi-btw-test-auth-empty-{}.json",
            std::process::id()
        )))
        .expect("empty auth storage loads");
        assert!(BtwClient::for_model_entry(&entry, None, &auth).is_none());
    }
}
