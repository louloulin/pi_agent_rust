//! Pure provider routing primitives.
//!
//! This crate deliberately contains no HTTP client, authentication, or runtime
//! integration. It is the dependency-light boundary used by the coding-agent
//! transport layer.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRegistration {
    pub id: String,
    pub api: String,
    pub aliases: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRoute {
    pub provider_id: String,
    pub api: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderHealth {
    Healthy,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone)]
pub struct ProviderRegistry {
    providers: HashMap<String, ProviderRegistration>,
    aliases: HashMap<String, String>,
}
impl ProviderRegistry {
    pub fn new(registrations: impl IntoIterator<Item = ProviderRegistration>) -> Self {
        let mut registry = Self {
            providers: HashMap::new(),
            aliases: HashMap::new(),
        };
        for registration in registrations {
            registry.register(registration);
        }
        registry
    }
    pub fn register(&mut self, registration: ProviderRegistration) {
        let id = normalize(&registration.id);
        if id.is_empty() {
            return;
        }
        self.aliases.insert(id.clone(), id.clone());
        for alias in &registration.aliases {
            let alias = normalize(alias);
            if !alias.is_empty() {
                self.aliases.insert(alias, id.clone());
            }
        }
        self.providers.insert(id, registration);
    }
    pub fn resolve(&self, provider: &str, api_override: Option<&str>) -> Option<ProviderRoute> {
        let canonical = self.aliases.get(&normalize(provider))?;
        let registration = self.providers.get(canonical)?;
        Some(ProviderRoute {
            provider_id: registration.id.clone(),
            api: api_override
                .filter(|api| !api.trim().is_empty())
                .unwrap_or(&registration.api)
                .to_string(),
        })
    }
    pub fn fallback_chain(&self, requested: &str, fallbacks: &[&str]) -> Vec<ProviderRoute> {
        std::iter::once(requested)
            .chain(fallbacks.iter().copied())
            .filter_map(|p| self.resolve(p, None))
            .fold(Vec::new(), |mut routes, route| {
                if !routes.iter().any(|existing: &ProviderRoute| {
                    existing
                        .provider_id
                        .eq_ignore_ascii_case(&route.provider_id)
                }) {
                    routes.push(route);
                }
                routes
            })
    }
}

#[derive(Debug, Clone)]
pub struct HealthTracker {
    states: Arc<RwLock<HashMap<String, ProviderHealth>>>,
}
impl Default for HealthTracker {
    fn default() -> Self {
        Self::new()
    }
}
impl HealthTracker {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn set(&self, provider: &str, health: ProviderHealth) {
        self.states
            .write()
            .expect("health lock poisoned")
            .insert(normalize(provider), health);
    }
    pub fn get(&self, provider: &str) -> ProviderHealth {
        self.states
            .read()
            .expect("health lock poisoned")
            .get(&normalize(provider))
            .copied()
            .unwrap_or(ProviderHealth::Healthy)
    }
    pub fn available_chain(&self, chain: &[ProviderRoute]) -> Vec<ProviderRoute> {
        chain
            .iter()
            .filter(|route| self.get(&route.provider_id) != ProviderHealth::Unavailable)
            .cloned()
            .collect()
    }
}
fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn registry() -> ProviderRegistry {
        ProviderRegistry::new([
            ProviderRegistration {
                id: "openai".into(),
                api: "openai-responses".into(),
                aliases: vec!["oai".into()],
            },
            ProviderRegistration {
                id: "anthropic".into(),
                api: "anthropic-messages".into(),
                aliases: vec![],
            },
        ])
    }
    #[test]
    fn resolves_alias_and_explicit_api() {
        assert_eq!(
            registry().resolve(" OAI ", Some("openai-completions")),
            Some(ProviderRoute {
                provider_id: "openai".into(),
                api: "openai-completions".into()
            })
        );
    }
    #[test]
    fn fallback_chain_deduplicates_aliases_and_skips_unknowns() {
        assert_eq!(
            registry().fallback_chain("oai", &["missing", "openai", "anthropic"]),
            vec![
                ProviderRoute {
                    provider_id: "openai".into(),
                    api: "openai-responses".into()
                },
                ProviderRoute {
                    provider_id: "anthropic".into(),
                    api: "anthropic-messages".into()
                }
            ]
        );
    }
    #[test]
    fn health_filters_unavailable_routes() {
        let tracker = HealthTracker::new();
        let chain = registry().fallback_chain("openai", &["anthropic"]);
        tracker.set("openai", ProviderHealth::Unavailable);
        assert_eq!(tracker.available_chain(&chain), vec![chain[1].clone()]);
    }
}
