//! Provider usage/quota readers and the `/usage` surface (bd-cv653.7.4).
//!
//! Read-only GETs against the quota/credit endpoints providers actually
//! expose (OpenRouter credits, Moonshot balance, GitHub Copilot entitlement),
//! normalized into [`ProviderUsage`] rows so users see quota walls before a
//! 429 does. Providers without a public endpoint (Anthropic, OpenAI expose
//! rate-limit state only in response headers) report `Unavailable` with the
//! reason instead of failing. A failed or slow read never blocks anything:
//! every fetch is timeout-bounded and falls back to the last cached row with
//! an age label.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};


use crate::auth::AuthStorage;
use pi_error::{Error, Result};
use crate::http::client::Client;

use pi_chord::usage::{ProviderUsage, UsageStatus, render_usage_json, render_usage_text, USAGE_CACHE_TTL, USAGE_FETCH_TIMEOUT, USAGE_SCHEMA};

/// Credential lookup seam used by usage assembly.
pub trait AuthProvider: Send + Sync {
    fn resolve_api_key(&self, provider: &str) -> Option<String>;
}

impl AuthProvider for AuthStorage {
    fn resolve_api_key(&self, provider: &str) -> Option<String> {
        AuthStorage::resolve_api_key(self, provider, None)
    }
}

/// Minimal JSON HTTP seam used by quota readers.
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    async fn get_json(&self, url: &str, headers: &[(&str, &str)]) -> Result<serde_json::Value>;
}

#[async_trait::async_trait]
impl HttpClient for Client {
    async fn get_json(&self, url: &str, headers: &[(&str, &str)]) -> Result<serde_json::Value> {
        client_get_json(self, url, headers).await
    }
}

/// A single provider's quota endpoint reader.
#[async_trait::async_trait]
pub trait UsageReader: Send + Sync {
    fn provider(&self) -> &'static str;
    async fn fetch(&self, client: &dyn HttpClient) -> Result<ProviderUsage>;
}

// ── OpenRouter ──────────────────────────────────────────────────────

/// Reads `GET /api/v1/credits`: purchased vs consumed credits (USD).
pub struct OpenRouterUsageReader {
    api_key: String,
    base_url: String,
}

impl OpenRouterUsageReader {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self::with_base_url(api_key, "https://openrouter.ai".to_string())
    }

    #[must_use]
    pub const fn with_base_url(api_key: String, base_url: String) -> Self {
        Self { api_key, base_url }
    }
}

#[async_trait::async_trait]
impl UsageReader for OpenRouterUsageReader {
    fn provider(&self) -> &'static str {
        "openrouter"
    }

    async fn fetch(&self, client: &dyn HttpClient) -> Result<ProviderUsage> {
        let url = format!("{}/api/v1/credits", self.base_url.trim_end_matches('/'));
        let body = client.get_json(
            &url,
            &[("Authorization", &format!("Bearer {}", self.api_key))],
        )
        .await?;
        let data = &body["data"]; // ubs:ignore serde_json Value index returns Null, never panics
        let total = data["total_credits"].as_f64(); // ubs:ignore serde_json Value index returns Null, never panics
        let used = data["total_usage"].as_f64(); // ubs:ignore serde_json Value index returns Null, never panics
        Ok(ProviderUsage {
            provider: "openrouter".to_string(),
            plan: None,
            used,
            limit: total,
            remaining: match (total, used) {
                (Some(total), Some(used)) => Some(total - used),
                _ => None,
            },
            unit: Some("USD credits".to_string()),
            resets_at: None,
            detail: None,
            source: url,
            fetched_at_ms: now_ms(),
            cache_age_secs: None,
        })
    }
}

// ── Moonshot / Kimi ─────────────────────────────────────────────────

/// Reads `GET /v1/users/me/balance`: available/voucher/cash balance.
pub struct MoonshotUsageReader {
    api_key: String,
    base_url: String,
}

impl MoonshotUsageReader {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self::with_base_url(api_key, "https://api.moonshot.ai".to_string())
    }

    #[must_use]
    pub const fn with_base_url(api_key: String, base_url: String) -> Self {
        Self { api_key, base_url }
    }
}

#[async_trait::async_trait]
impl UsageReader for MoonshotUsageReader {
    fn provider(&self) -> &'static str {
        "moonshotai"
    }

    async fn fetch(&self, client: &dyn HttpClient) -> Result<ProviderUsage> {
        let url = format!(
            "{}/v1/users/me/balance",
            self.base_url.trim_end_matches('/')
        );
        let body = client.get_json(
            &url,
            &[("Authorization", &format!("Bearer {}", self.api_key))],
        )
        .await?;
        let data = &body["data"]; // ubs:ignore serde_json Value index returns Null, never panics
        let available = data["available_balance"].as_f64(); // ubs:ignore serde_json Value index returns Null, never panics
        let voucher = data["voucher_balance"].as_f64(); // ubs:ignore serde_json Value index returns Null, never panics
        let cash = data["cash_balance"].as_f64();
        Ok(ProviderUsage {
            provider: "moonshotai".to_string(),
            plan: None,
            used: None,
            limit: None,
            remaining: available,
            unit: Some("balance".to_string()),
            resets_at: None,
            detail: match (voucher, cash) {
                (Some(voucher), Some(cash)) => {
                    Some(format!("voucher {voucher:.2}, cash {cash:.2}"))
                }
                _ => None,
            },
            source: url,
            fetched_at_ms: now_ms(),
            cache_age_secs: None,
        })
    }
}

// ── GitHub Copilot ──────────────────────────────────────────────────

/// Reads the Copilot token endpoint: plan SKU plus limited-user quota
/// counters when present (free-tier accounts).
pub struct CopilotUsageReader {
    github_token: String,
    base_url: String,
}

impl CopilotUsageReader {
    #[must_use]
    pub fn new(github_token: String) -> Self {
        // Same resolution as the Copilot provider's token exchange:
        // api.github.com unless `PI_COPILOT_GITHUB_API_BASE` moves it
        // (GHE / data residency, gh #191).
        Self::with_base_url(github_token, crate::providers::copilot::github_api_base())
    }

    #[must_use]
    pub const fn with_base_url(github_token: String, base_url: String) -> Self {
        Self {
            github_token,
            base_url,
        }
    }
}

#[async_trait::async_trait]
impl UsageReader for CopilotUsageReader {
    fn provider(&self) -> &'static str {
        "github-copilot"
    }

    async fn fetch(&self, client: &dyn HttpClient) -> Result<ProviderUsage> {
        let url = format!(
            "{}/copilot_internal/v2/token",
            self.base_url.trim_end_matches('/')
        );
        let body = client.get_json(
            &url,
            &[
                ("Authorization", &format!("token {}", self.github_token)), // ubs:ignore outbound auth header, local credential
                ("User-Agent", "pi-agent-rust"),
            ],
        )
        .await?;
        let plan = body["sku"].as_str().map(str::to_string); // ubs:ignore serde_json Value index returns Null, never panics
        let chat_quota = body["limited_user_quotas"]["chat"].as_f64(); // ubs:ignore serde_json Value index returns Null, never panics
        let reset = body["limited_user_reset_date"].as_str().map(str::to_string);
        Ok(ProviderUsage {
            provider: "github-copilot".to_string(),
            plan,
            used: None,
            limit: None,
            remaining: chat_quota,
            unit: chat_quota.is_some().then(|| "chat requests".to_string()),
            resets_at: reset,
            detail: body["limited_user_quotas"]["completions"] // ubs:ignore serde_json Value index returns Null, never panics
                .as_f64()
                .map(|c| format!("completions quota {c}")),
            source: url,
            fetched_at_ms: now_ms(),
            cache_age_secs: None,
        })
    }
}

async fn client_get_json(
    client: &Client,
    url: &str,
    headers: &[(&str, &str)],
) -> Result<serde_json::Value> {
    let mut request = client.get(url);
    for (name, value) in headers {
        request = request.header(*name, *value); // ubs:ignore outbound request header from local auth storage, not request-controlled
    }
    let response = Box::pin(request.send()).await?;
    let status = response.status();
    let text = response
        .text()
        .await
        .unwrap_or_else(|e| format!("<failed to read body: {e}>"));
    if !(200..300).contains(&status) {
        return Err(Error::api(format!("HTTP {status} from {url}: {text}")));
    }
    serde_json::from_str(&text).map_err(|e| Error::api(format!("Invalid JSON from {url}: {e}")))
}

// ── Assembly, cache, rendering ──────────────────────────────────────

/// Providers that hold credentials but expose no public quota endpoint.
const NO_ENDPOINT_REASON: &[(&str, &str)] = &[
    (
        "anthropic",
        "no public quota endpoint (rate-limit state arrives only in response headers)",
    ),
    (
        "openai",
        "no public quota endpoint (usage dashboard requires a browser session)",
    ),
];

/// Readers for configured providers plus (provider, reason) rows for the
/// known no-endpoint set.
pub type ConfiguredReaders = (Vec<Box<dyn UsageReader>>, Vec<(String, String)>);

/// Build readers for every provider with resolvable credentials, plus
/// documented-unavailable rows for known no-endpoint providers.
#[must_use]
pub fn readers_from_auth(auth: &impl AuthProvider) -> ConfiguredReaders {
    let mut readers: Vec<Box<dyn UsageReader>> = Vec::new();
    let mut unavailable: Vec<(String, String)> = Vec::new();

    if let Some(key) = auth.resolve_api_key("openrouter") {
        readers.push(Box::new(OpenRouterUsageReader::new(key)));
    }
    if let Some(key) = auth.resolve_api_key("moonshotai") {
        readers.push(Box::new(MoonshotUsageReader::new(key)));
    }
    if let Some(token) = auth.resolve_api_key("github-copilot") {
        readers.push(Box::new(CopilotUsageReader::new(token)));
    }
    for (provider, reason) in NO_ENDPOINT_REASON {
        if auth.resolve_api_key(provider).is_some() {
            unavailable.push(((*provider).to_string(), (*reason).to_string()));
        }
    }

    (readers, unavailable)
}

static USAGE_CACHE: Mutex<Option<HashMap<String, (Instant, ProviderUsage)>>> = Mutex::new(None);

#[allow(clippy::significant_drop_tightening)] // the guard is the whole body
fn cache_get(provider: &str, max_age: Duration) -> Option<(Duration, ProviderUsage)> {
    let guard = USAGE_CACHE.lock().expect("usage cache lock"); // ubs:ignore poisoned lock means a prior fetch panicked; propagating cannot help
    let cache = guard.as_ref()?;
    let (at, usage) = cache.get(provider)?;
    let age = at.elapsed();
    (age <= max_age).then(|| (age, usage.clone()))
}

fn cache_put(provider: &str, usage: &ProviderUsage) {
    let mut guard = USAGE_CACHE.lock().expect("usage cache lock"); // ubs:ignore poisoned lock means a prior fetch panicked; propagating cannot help
    guard
        .get_or_insert_with(HashMap::new)
        .insert(provider.to_string(), (Instant::now(), usage.clone()));
}

/// Fetch usage for every configured provider.
///
/// Fresh cache hits (younger than [`USAGE_CACHE_TTL`]) short-circuit unless
/// `refresh` forces a live read; on a failed live read, any cached row of ANY
/// age is returned with its age labeled before the error row is considered.
pub async fn gather_usage(auth: &AuthStorage, refresh: bool) -> Vec<UsageStatus> {
    let (readers, unavailable) = readers_from_auth(auth);
    let client = Client::new();
    let mut rows = Vec::new();

    for reader in readers {
        let provider = reader.provider().to_string();
        if !refresh && let Some((age, mut usage)) = cache_get(&provider, USAGE_CACHE_TTL) {
            usage.cache_age_secs = Some(age.as_secs());
            rows.push(UsageStatus::Ready(usage));
            continue;
        }
        let fetched = asupersync::time::timeout(
            asupersync::time::wall_now(),
            USAGE_FETCH_TIMEOUT,
            reader.fetch(&client),
        )
        .await;
        match fetched {
            Ok(Ok(usage)) => {
                cache_put(&provider, &usage);
                rows.push(UsageStatus::Ready(usage));
            }
            Ok(Err(err)) => rows.push(stale_or_error(&provider, &err.to_string())),
            Err(_) => rows.push(stale_or_error(
                &provider,
                &format!("timed out after {}s", USAGE_FETCH_TIMEOUT.as_secs()),
            )),
        }
    }

    for (provider, reason) in unavailable {
        rows.push(UsageStatus::Unavailable { provider, reason });
    }

    rows
}

fn stale_or_error(provider: &str, error: &str) -> UsageStatus {
    cache_get(provider, Duration::MAX).map_or_else(
        || UsageStatus::Error {
            provider: provider.to_string(),
            error: error.to_string(),
        },
        |(age, mut usage)| {
            usage.cache_age_secs = Some(age.as_secs());
            usage.detail = Some(usage.detail.take().map_or_else(
                || format!("live read failed: {error}"),
                |detail| format!("{detail}; live read failed: {error}"),
            ));
            UsageStatus::Ready(usage)
        },
    )
}
