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
//!
//! The HTTP transport is abstracted through [`HttpFetch`] so this crate
//! does not have to depend on the in-`pi` HTTP client. The `pi` crate
//! provides the production implementation that delegates to
//! `http::client::Client`; the bundled tests use a tiny raw-TCP mock server.
//!
//! Credential lookup is abstracted through [`CredentialLookup`] so this crate
//! also avoids depending on the in-`pi` `AuthStorage`. The `pi` crate
//! provides an implementation that delegates to the real `AuthStorage`.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;

use pi_error::{Error, Result};

/// Schema tag for usage rows in JSON output and RPC events.
pub const USAGE_SCHEMA: &str = "pi.usage.v1";

/// How long a fetched row stays fresh before `/usage` re-reads the endpoint.
pub const USAGE_CACHE_TTL: Duration = Duration::from_secs(60);

/// Per-provider fetch budget; a slow endpoint degrades to cache/unavailable.
pub const USAGE_FETCH_TIMEOUT: Duration = Duration::from_secs(8);

/// Minimal HTTP abstraction used to fetch quota endpoint responses.
///
/// `pi-usage` deliberately avoids depending on the in-`pi` HTTP client; the
/// `pi` crate provides an implementation that delegates to
/// `http::client::Client` so production wiring keeps the existing
/// capability-gated transport.
pub trait HttpFetch {
    /// Issue a GET request with the given headers and return the raw
    /// response body as text plus the HTTP status code.
    ///
    /// # Errors
    /// Surfaces transport-level errors as `Error::api` so callers can
    /// collapse any failure into a cache-or-error fallback row.
    fn get_text<'a>(
        &'a self,
        url: &'a str,
        headers: &'a [(&'a str, &'a str)],
        timeout: Duration,
    ) -> impl std::future::Future<Output = Result<(u16, String)>> + Send + 'a;
}

/// Minimal credential lookup abstraction.
///
/// `pi-usage` deliberately avoids depending on the in-`pi` `AuthStorage`.
/// The `pi` crate provides an implementation that delegates to the real
/// `AuthStorage::resolve_api_key`.
pub trait CredentialLookup {
    /// Resolve an API key for the given provider, returning `None` when no
    /// credential is configured.
    fn resolve_api_key(&self, provider: &str) -> Option<String>;
}

/// Normalized usage/quota snapshot for one provider.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderUsage {
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resets_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    pub source: String,
    pub fetched_at_ms: i64,
    /// Set when this row was served from cache instead of a live read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_age_secs: Option<u64>,
}

/// One `/usage` row: a reading, a documented absence, or a failed read.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum UsageStatus {
    Ready(ProviderUsage),
    Unavailable { provider: String, reason: String },
    Error { provider: String, error: String },
}

impl UsageStatus {
    #[must_use]
    pub fn provider(&self) -> &str {
        match self {
            Self::Ready(usage) => &usage.provider,
            Self::Unavailable { provider, .. } | Self::Error { provider, .. } => provider,
        }
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

/// A single provider's quota endpoint reader.
///
/// Concrete reader types live in this crate; callers should use
/// [`AnyUsageReader`] when they need to hold a heterogeneous collection
/// (the enum preserves the closed set of currently supported providers).
#[async_trait::async_trait]
pub trait UsageReader: Send + Sync {
    fn provider(&self) -> &'static str;
    async fn fetch<H: HttpFetch + Sync + ?Sized>(&self, http: &H) -> Result<ProviderUsage>;
}

/// Closed-set enum mirroring every concrete [`UsageReader`] in this crate.
///
/// The enum exists so callers can store a heterogeneous list of readers
/// without resorting to `Box<dyn UsageReader>` (which would require the
/// trait to be object-safe, but `fetch` takes a generic `HttpFetch` so the
/// trait is intentionally not object-safe — the production HTTP client is
/// chosen by the caller, not the reader).
#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // all variants stay small (single string fields each)
#[non_exhaustive]
pub enum AnyUsageReader {
    OpenRouter(OpenRouterUsageReader),
    Moonshot(MoonshotUsageReader),
    Copilot(CopilotUsageReader),
}

impl AnyUsageReader {
    /// Provider identifier used for cache keys and rendering.
    #[must_use]
    pub fn provider(&self) -> &'static str {
        match self {
            Self::OpenRouter(r) => r.provider(),
            Self::Moonshot(r) => r.provider(),
            Self::Copilot(r) => r.provider(),
        }
    }

    /// Fetch a fresh quota row using the caller's HTTP transport.
    pub async fn fetch<H: HttpFetch + Sync + ?Sized>(&self, http: &H) -> Result<ProviderUsage> {
        match self {
            Self::OpenRouter(r) => r.fetch(http).await,
            Self::Moonshot(r) => r.fetch(http).await,
            Self::Copilot(r) => r.fetch(http).await,
        }
    }
}

// ── OpenRouter ──────────────────────────────────────────────────────

/// Reads `GET /api/v1/credits`: purchased vs consumed credits (USD).
#[derive(Debug)]
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

    async fn fetch<H: HttpFetch + Sync + ?Sized>(&self, http: &H) -> Result<ProviderUsage> {
        let url = format!("{}/api/v1/credits", self.base_url.trim_end_matches('/'));
        let body = get_json(
            http,
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
#[derive(Debug)]
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

    async fn fetch<H: HttpFetch + Sync + ?Sized>(&self, http: &H) -> Result<ProviderUsage> {
        let url = format!(
            "{}/v1/users/me/balance",
            self.base_url.trim_end_matches('/')
        );
        let body = get_json(
            http,
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
#[derive(Debug)]
pub struct CopilotUsageReader {
    github_token: String,
    base_url: String,
}

impl CopilotUsageReader {
    #[must_use]
    pub fn new(github_token: String) -> Self {
        // Mirror `crate::providers::copilot::github_api_base()` —
        // `PI_COPILOT_GITHUB_API_BASE` or the `api.github.com` default.
        Self::with_base_url(github_token, github_api_base())
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

    async fn fetch<H: HttpFetch + Sync + ?Sized>(&self, http: &H) -> Result<ProviderUsage> {
        let url = format!(
            "{}/copilot_internal/v2/token",
            self.base_url.trim_end_matches('/')
        );
        let body = get_json(
            http,
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

const GITHUB_API_BASE_DEFAULT: &str = "https://api.github.com";

fn github_api_base_from_env(value: Option<&str>) -> String {
    match value.map(str::trim) {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => GITHUB_API_BASE_DEFAULT.to_string(),
    }
}

/// Resolve the GitHub REST API base used by the Copilot usage reader.
///
/// Mirrors the in-`pi` `providers::copilot::github_api_base` helper:
/// `PI_COPILOT_GITHUB_API_BASE` or `api.github.com` by default.
#[must_use]
pub fn github_api_base() -> String {
    github_api_base_from_env(std::env::var("PI_COPILOT_GITHUB_API_BASE").ok().as_deref())
}

async fn get_json<H: HttpFetch + Sync + ?Sized>(
    http: &H,
    url: &str,
    headers: &[(&str, &str)],
) -> Result<serde_json::Value> {
    let (status, text) = http.get_text(url, headers, USAGE_FETCH_TIMEOUT).await?;
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
pub type ConfiguredReaders = (Vec<AnyUsageReader>, Vec<(String, String)>);

/// Build readers for every provider with resolvable credentials, plus
/// documented-unavailable rows for known no-endpoint providers.
#[must_use]
pub fn readers_from_auth<L: CredentialLookup>(auth: &L) -> ConfiguredReaders {
    let mut readers: Vec<AnyUsageReader> = Vec::new();
    let mut unavailable: Vec<(String, String)> = Vec::new();

    if let Some(key) = auth.resolve_api_key("openrouter") {
        readers.push(AnyUsageReader::OpenRouter(OpenRouterUsageReader::new(key)));
    }
    if let Some(key) = auth.resolve_api_key("moonshotai") {
        readers.push(AnyUsageReader::Moonshot(MoonshotUsageReader::new(key)));
    }
    if let Some(token) = auth.resolve_api_key("github-copilot") {
        readers.push(AnyUsageReader::Copilot(CopilotUsageReader::new(token)));
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
pub async fn gather_usage<L: CredentialLookup, H: HttpFetch + Sync>(
    auth: &L,
    http: &H,
    refresh: bool,
) -> Vec<UsageStatus> {
    let (readers, unavailable) = readers_from_auth(auth);
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
            reader.fetch(http),
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

/// Render rows as the human-readable `/usage` table.
#[must_use]
pub fn render_usage_text(rows: &[UsageStatus]) -> String {
    if rows.is_empty() {
        return "No providers with credentials configured. Run /login <provider> first."
            .to_string();
    }
    let mut lines = vec!["Provider usage:".to_string()];
    for row in rows {
        match row {
            UsageStatus::Ready(usage) => {
                let mut parts: Vec<String> = Vec::new();
                if let Some(plan) = &usage.plan {
                    parts.push(format!("plan {plan}"));
                }
                match (usage.used, usage.limit) {
                    (Some(used), Some(limit)) => {
                        parts.push(format!("{used:.2} of {limit:.2} used"));
                    }
                    (Some(used), None) => parts.push(format!("{used:.2} used")),
                    _ => {}
                }
                if let Some(remaining) = usage.remaining {
                    parts.push(format!("{remaining:.2} remaining"));
                }
                if let Some(unit) = &usage.unit {
                    parts.push(format!("({unit})"));
                }
                if let Some(resets) = &usage.resets_at {
                    parts.push(format!("resets {resets}"));
                }
                if let Some(detail) = &usage.detail {
                    parts.push(format!("— {detail}"));
                }
                if let Some(age) = usage.cache_age_secs {
                    parts.push(format!("[cached {age}s ago]"));
                }
                if parts.is_empty() {
                    parts.push("no quota data in response".to_string());
                }
                lines.push(format!("  {}: {}", usage.provider, parts.join(" ")));
            }
            UsageStatus::Unavailable { provider, reason } => {
                lines.push(format!("  {provider}: unavailable — {reason}"));
            }
            UsageStatus::Error { provider, error } => {
                lines.push(format!("  {provider}: read failed — {error}"));
            }
        }
    }
    lines.join("\n")
}

/// Render rows as JSON (`pi usage --format json`).
#[must_use]
pub fn render_usage_json(rows: &[UsageStatus]) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": USAGE_SCHEMA,
        "providers": rows,
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Per-instance HTTP fetcher backed by a raw TCP server. Each
    /// `MockHttp` owns its own response slot so parallel tests do not
    /// clobber each other.
    pub struct MockHttp {
        slot: Arc<Mutex<Option<(u16, String)>>>,
    }

    impl MockHttp {
        pub fn new() -> Self {
            Self {
                slot: Arc::new(Mutex::new(None)),
            }
        }

        pub fn respond_with(&self, status: u16, body: &str) {
            let mut slot = self.slot.lock().expect("mock slot");
            *slot = Some((status, body.to_string()));
        }
    }

    impl Default for MockHttp {
        fn default() -> Self {
            Self::new()
        }
    }

    impl HttpFetch for MockHttp {
        async fn get_text(
            &self,
            url: &str,
            _headers: &[(&str, &str)],
            _timeout: Duration,
        ) -> Result<(u16, String)> {
            // The reader is the one calling `get_text`; we know it only does
            // so with URLs it constructed itself, so the body returned here
            // is what the production HTTP path would have decoded.
            let (status, body) = self
                .slot
                .lock()
                .expect("mock slot")
                .clone()
                .expect("mock slot unset");
            let _ = url;
            Ok((status, body))
        }
    }

    fn run_async<F: std::future::Future>(future: F) -> F::Output {
        asupersync::runtime::RuntimeBuilder::current_thread() // ubs:ignore test harness runtime
            .build() // ubs:ignore test harness runtime
            .expect("runtime build")
            .block_on(future)
    }

    #[test]
    fn openrouter_reader_parses_credits() {
        let http = MockHttp::new();
        http.respond_with(200, r#"{"data":{"total_credits":25.0,"total_usage":10.5}}"#);
        let reader =
            OpenRouterUsageReader::with_base_url("sk-or-test".to_string(), "http://x".to_string());
        let usage = run_async(async { reader.fetch(&http).await.expect("fetch") });
        assert_eq!(usage.provider, "openrouter");
        assert_eq!(usage.limit, Some(25.0));
        assert_eq!(usage.used, Some(10.5));
        assert_eq!(usage.remaining, Some(14.5));
    }

    #[test]
    fn moonshot_reader_parses_balance() {
        let http = MockHttp::new();
        http.respond_with(
            200,
            r#"{"code":0,"data":{"available_balance":42.5,"voucher_balance":2.5,"cash_balance":40.0},"status":true}"#,
        );
        let reader =
            MoonshotUsageReader::with_base_url("sk-test".to_string(), "http://x".to_string());
        let usage = run_async(async { reader.fetch(&http).await.expect("fetch") });
        assert_eq!(usage.provider, "moonshotai");
        assert_eq!(usage.remaining, Some(42.5));
        assert!(usage.detail.as_deref().unwrap().contains("voucher 2.50"));
    }

    #[test]
    fn copilot_reader_surfaces_plan_and_quota() {
        let http = MockHttp::new();
        http.respond_with(
            200,
            r#"{"token":"t","sku":"free_limited_copilot","limited_user_quotas":{"chat":32.0,"completions":100.0},"limited_user_reset_date":"2026-09-01"}"#,
        );
        let reader =
            CopilotUsageReader::with_base_url("gho_test".to_string(), "http://x".to_string());
        let usage = run_async(async { reader.fetch(&http).await.expect("fetch") });
        assert_eq!(usage.plan.as_deref(), Some("free_limited_copilot"));
        assert_eq!(usage.remaining, Some(32.0));
        assert_eq!(usage.resets_at.as_deref(), Some("2026-09-01"));
    }

    #[test]
    fn reader_error_on_http_failure() {
        let http = MockHttp::new();
        http.respond_with(401, r#"{"error":"bad key"}"#);
        let reader =
            OpenRouterUsageReader::with_base_url("sk-bad".to_string(), "http://x".to_string());
        let result = run_async(async { reader.fetch(&http).await });
        let err = result.expect_err("401 must fail");
        assert!(err.to_string().contains("401"), "{err}");
    }

    #[test]
    fn render_text_covers_all_row_kinds() {
        let rows = vec![
            UsageStatus::Ready(ProviderUsage {
                provider: "openrouter".to_string(),
                plan: None,
                used: Some(10.5),
                limit: Some(25.0),
                remaining: Some(14.5),
                unit: Some("USD credits".to_string()),
                resets_at: None,
                detail: None,
                source: "test".to_string(),
                fetched_at_ms: 0,
                cache_age_secs: Some(30),
            }),
            UsageStatus::Unavailable {
                provider: "anthropic".to_string(),
                reason: "no public quota endpoint".to_string(),
            },
            UsageStatus::Error {
                provider: "moonshotai".to_string(),
                error: "timed out".to_string(),
            },
        ];
        let text = render_usage_text(&rows);
        assert!(text.contains("10.50 of 25.00 used"), "{text}");
        assert!(text.contains("[cached 30s ago]"), "{text}");
        assert!(text.contains("anthropic: unavailable"), "{text}");
        assert!(text.contains("moonshotai: read failed"), "{text}");

        let empty = render_usage_text(&[]);
        assert!(empty.contains("/login"), "{empty}");
    }

    #[test]
    fn render_json_is_schema_tagged() {
        let json = render_usage_json(&[UsageStatus::Unavailable {
            provider: "anthropic".to_string(),
            reason: "n/a".to_string(),
        }]);
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json"); // ubs:ignore test assertion on fixture JSON
        assert_eq!(value["schema"], USAGE_SCHEMA);
        assert_eq!(value["providers"][0]["status"], "unavailable");
    }
}
