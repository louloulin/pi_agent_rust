//! CLI argument parsing using Clap.

use clap::error::ErrorKind;
use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionCliFlag {
    pub name: String,
    pub value: Option<String>,
}

impl ExtensionCliFlag {
    pub fn display_name(&self) -> String {
        format!("--{}", self.name)
    }
}

#[derive(Debug)]
pub struct ParsedCli {
    pub cli: Cli,
    pub extension_flags: Vec<ExtensionCliFlag>,
}

#[derive(Debug, Clone, Copy)]
struct LongOptionSpec {
    takes_value: bool,
    optional_value: bool,
}

const ROOT_SUBCOMMANDS: &[&str] = &[
    "install",
    "remove",
    "update",
    "update-index",
    "context-preview",
    "swarm-progress",
    "swarm-replay-preview",
    "validation-broker",
    "search",
    "info",
    "list",
    "config",
    "doctor",
    "migrate",
    "usage",
    "gc",
    "review",
    "rules",
    "handoff",
    "commit",
    "worktree",
    "completions",
    "__complete",
    "token",
    "stats",
    "profile",
    "import",
    "grievances",
    "self-update",
    "web",
    "gallery",
];

fn known_long_option(name: &str) -> Option<LongOptionSpec> {
    let (takes_value, optional_value) = match name {
        "version"
        | "continue"
        | "resume"
        | "no-session"
        | "no-migrations"
        | "no-mouse-capture"
        | "print"
        | "rpc"
        | "acp"
        | "verbose"
        | "no-tools"
        | "no-extensions"
        | "plan-mode"
        | "plan-yolo"
        | "yolo"
        | "auto-approve"
        | "explain-extension-policy"
        | "explain-repair-policy"
        | "no-skills"
        | "no-context-files"
        // bd-cv653.3.12 / bd-cv653.7.12 / bd-cv653.7.12.1: the pre-parser
        // must pass these through to clap — any top-level long missing from
        // this match gets silently diverted to extension-flag extraction.
        | "crash-test"
        | "profile"
        | "no-prompt-templates"
        | "no-themes"
        | "list-providers"
        | "refresh-models"
        | "persist-models"
        | "trust"
        // ftui migration flags (bd-cv653.9.1). Listed unconditionally: on
        // non-ftui builds clap still rejects them with a proper error instead
        // of the pre-parser silently diverting them to extension flags.
        | "ftui"
        | "classic"
        | "classic-tui"
        | "charmed"
        | "bubbletea"
        | "inline"
        | "hide-cwd-in-prompt" => (false, false),
        "provider"
        | "model"
        | "api-key"
        | "models"
        | "smol"
        | "slow"
        | "plan"
        | "advisor"
        | "approval-mode"
        | "thinking"
        | "system-prompt"
        | "append-system-prompt"
        | "session"
        | "session-dir"
        | "add-dir"
        | "session-durability"
        | "mode"
        | "tools"
        | "extension"
        | "mcp-config"
        | "extension-policy"
        | "repair-policy"
        | "skill"
        | "prompt-template"
        | "theme"
        | "theme-path"
        | "max-tool-iterations"
        | "max-time"
        | "request-timeout"
        | "export"
        | "fetch-models" => (true, false),
        "list-models" => (true, true),
        _ => return None,
    };
    Some(LongOptionSpec {
        takes_value,
        optional_value,
    })
}

fn is_known_short_flag(token: &str) -> bool {
    if !token.starts_with('-') || token.starts_with("--") {
        return false;
    }
    let body = &token[1..];
    if body.is_empty() {
        return false;
    }
    body.chars()
        .all(|ch| matches!(ch, 'v' | 'c' | 'r' | 'p' | 'e'))
}

fn short_flag_expects_value(token: &str) -> bool {
    if !is_known_short_flag(token) {
        return false;
    }

    let body = &token[1..];
    body.find('e')
        .is_some_and(|index| index.eq(&(body.len() - 1)))
}

fn is_negative_numeric_token(token: &str) -> bool {
    if !token.starts_with('-') || token.eq("-") || token.starts_with("--") {
        return false;
    }
    token.parse::<i64>().is_ok() || token.parse::<f64>().is_ok_and(f64::is_finite)
}

#[allow(clippy::too_many_lines)] // Argument normalization needs single-pass stateful parsing.
fn preprocess_extension_flags(raw_args: &[String]) -> (Vec<String>, Vec<ExtensionCliFlag>) {
    if raw_args.is_empty() {
        return (vec!["pi".to_string()], Vec::new());
    }
    let mut filtered = Vec::with_capacity(raw_args.len());
    filtered.push(raw_args[0].clone());
    let mut extracted = Vec::new();
    let mut expecting_value = false;
    let mut in_subcommand = false;
    let mut index = 1usize;
    while index < raw_args.len() {
        let token = &raw_args[index];
        if token.eq("--") {
            filtered.extend(raw_args[index..].iter().cloned());
            break;
        }
        if expecting_value {
            filtered.push(token.clone());
            expecting_value = false;
            index += 1;
            continue;
        }
        if in_subcommand {
            filtered.push(token.clone());
            index += 1;
            continue;
        }
        if token.starts_with("--") && token.len() > 2 {
            let without_prefix = &token[2..];
            let (name, has_inline_value) = without_prefix
                .split_once('=')
                .map_or((without_prefix, false), |(name, _)| (name, true));
            if let Some(spec) = known_long_option(name) {
                filtered.push(token.clone());
                if spec.takes_value && !has_inline_value && !spec.optional_value {
                    expecting_value = true;
                } else if spec.takes_value && !has_inline_value && spec.optional_value {
                    let has_value = raw_args
                        .get(index + 1)
                        .is_some_and(|next| !next.starts_with('-') || next.eq("-"));
                    expecting_value = has_value;
                }
                index += 1;
                continue;
            }
            let (name, inline_value) = without_prefix
                .split_once('=')
                .map_or((without_prefix, None), |(name, value)| {
                    (name, Some(value.to_string()))
                });
            if name.is_empty() {
                filtered.push(token.clone());
                index += 1;
                continue;
            }
            let mut value = inline_value;
            if value.is_none() {
                let next = raw_args.get(index + 1);
                if let Some(next) = next
                    && next.ne("--")
                    && (!next.starts_with('-') || next.eq("-") || is_negative_numeric_token(next))
                {
                    value = Some(next.clone());
                    index += 1;
                }
            }
            extracted.push(ExtensionCliFlag {
                name: name.to_string(),
                value,
            });
            index += 1;
            continue;
        }
        if token.eq("-e") {
            filtered.push(token.clone());
            expecting_value = true;
            index += 1;
            continue;
        }
        if is_known_short_flag(token) {
            filtered.push(token.clone());
            expecting_value = short_flag_expects_value(token);
            index += 1;
            continue;
        }
        if token.starts_with('-') {
            filtered.push(token.clone());
            index += 1;
            continue;
        }
        if ROOT_SUBCOMMANDS.contains(&token.as_str()) {
            in_subcommand = true;
        }
        filtered.push(token.clone());
        index += 1;
    }
    (filtered, extracted)
}

pub fn parse_with_extension_flags(raw_args: Vec<String>) -> Result<ParsedCli, clap::Error> {
    if raw_args.is_empty() {
        let cli = Cli::try_parse_from(["pi"])?;
        return Ok(ParsedCli {
            cli,
            extension_flags: Vec::new(),
        });
    }

    match Cli::try_parse_from(raw_args.clone()) {
        Ok(_) => {
            // We do NOT return early here because `Cli` has trailing varargs for `message`.
            // If the user provided `pi hello --unknown flag`, clap might happily parse
            // `--unknown flag` into `message`. We must preprocess extension flags first!
        }
        Err(err) => {
            if matches!(
                err.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                return Err(err);
            }
        }
    }

    let (filtered_args, extension_flags) = preprocess_extension_flags(&raw_args);
    if extension_flags.is_empty() {
        let cli = Cli::try_parse_from(raw_args)?;
        return Ok(ParsedCli {
            cli,
            extension_flags: Vec::new(),
        });
    }

    let cli = Cli::try_parse_from(filtered_args)?;
    Ok(ParsedCli {
        cli,
        extension_flags,
    })
}

/// Pi - AI coding agent CLI
#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)] // CLI flags are naturally boolean
#[command(name = "pi")]
#[command(version, about, long_about = None, disable_version_flag = true)]
#[command(after_help = "Examples:
  pi \"explain this code\"              Start new session with message
  pi @file.rs \"review this\"           Include file in context
  pi -c                                Continue previous session
  pi -r                                Resume from session picker
  pi -p \"what is 2+2\"                 Print mode (non-interactive)
  pi --model claude-opus-4 \"help\"     Use specific model
")]
pub struct Cli {
    // === Help & Version ===
    /// Print version information
    #[arg(short = 'v', long)]
    pub version: bool,

    // === Model Configuration ===
    /// LLM provider (e.g., anthropic, openai, google).
    /// Run --list-providers for canonical IDs + aliases.
    #[arg(long, env = "PI_PROVIDER")]
    pub provider: Option<String>,

    /// Model ID (e.g., claude-opus-4, gpt-4o)
    #[arg(long, env = "PI_MODEL")]
    pub model: Option<String>,

    /// API key (overrides environment variable)
    #[arg(long)]
    pub api_key: Option<String>,

    /// Model patterns for Ctrl+P cycling (comma-separated, supports globs)
    #[arg(long)]
    pub models: Option<String>,

    /// Model spec for the `smol` role (cheap/fast work, e.g. subagent fan-out).
    /// Format: provider/model with optional :thinking suffix (bd-cv653.3.1).
    #[arg(long, value_name = "PROVIDER/MODEL")]
    pub smol: Option<String>,

    /// Model spec for the `slow` role (deep reasoning).
    #[arg(long, value_name = "PROVIDER/MODEL")]
    pub slow: Option<String>,

    /// Model spec for the `plan` role (plan mode).
    #[arg(long, value_name = "PROVIDER/MODEL")]
    pub plan: Option<String>,

    /// Model spec for the `advisor` role (turn-review second model).
    #[arg(long, value_name = "PROVIDER/MODEL")]
    pub advisor: Option<String>,

    /// Start in plan mode: read-only planning until a plan is approved
    /// (bd-cv653.3.5).
    #[arg(long)]
    pub plan_mode: bool,

    /// Auto-approve submitted plans without review (unattended runs).
    #[arg(long)]
    pub plan_yolo: bool,

    /// Tool approval mode: always-ask (default), write, or yolo (bd-cv653.3.19).
    #[arg(long, value_parser = ["always-ask", "write", "yolo"])]
    pub approval_mode: Option<String>,

    /// Shorthand alias for --approval-mode yolo (bd-cv653.3.19).
    #[arg(long, alias = "auto-approve")]
    pub yolo: bool,

    /// HTTP request timeout in seconds for provider API calls.
    ///
    /// Bounds connect + request + first-response-header latency for each
    /// provider request. `0` disables the timeout entirely (unbounded).
    ///
    /// When unset, the default is provider-aware: 60s for cloud providers and
    /// 600s (10 minutes) for local providers (Ollama, LM Studio) where the
    /// first request can block while the model loads into memory. Raise this if
    /// a local model's cold start exceeds the default. Equivalent to the
    /// `PI_HTTP_REQUEST_TIMEOUT_SECS` env var and the `requestTimeoutSecs`
    /// setting. See pi_agent_rust#90.
    #[arg(long, value_name = "SECONDS", env = "PI_HTTP_REQUEST_TIMEOUT_SECS")]
    pub request_timeout: Option<u64>,

    // === Thinking/Reasoning ===
    /// Extended thinking level
    #[arg(long, value_parser = ["off", "minimal", "low", "medium", "high", "xhigh", "max"])]
    pub thinking: Option<String>,

    // === System Prompt ===
    /// Override system prompt
    #[arg(long)]
    pub system_prompt: Option<String>,

    /// Append to system prompt (text or file path)
    #[arg(long)]
    pub append_system_prompt: Option<String>,

    // === Session Management ===
    /// Continue previous session
    #[arg(short = 'c', long)]
    pub r#continue: bool,

    /// Select session from picker UI
    #[arg(short = 'r', long)]
    pub resume: bool,

    /// Use specific session file path
    #[arg(long)]
    pub session: Option<String>,

    /// Directory for session storage/lookup
    #[arg(long)]
    pub session_dir: Option<String>,

    /// Don't save session (ephemeral)
    #[arg(long)]
    pub no_session: bool,

    /// Launch the FrankenTUI interactive stack (default when built with `ftui`).
    #[cfg(feature = "ftui")]
    #[arg(long)]
    pub ftui: bool,

    /// Force the classic charmed_rust TUI stack instead of the default ftui stack.
    #[arg(long, aliases = ["classic-tui", "charmed", "bubbletea"])]
    pub classic: bool,

    /// With ftui: run inline (UI at the bottom, shell scrollback
    /// preserved) instead of the alternate screen.
    #[cfg(feature = "ftui")]
    #[arg(long)]
    pub inline: bool,

    /// Session durability mode: strict, balanced, or throughput
    #[arg(
        long,
        value_parser = ["strict", "balanced", "throughput"]
    )]
    pub session_durability: Option<String>,

    /// Skip startup migrations for legacy config/session/layout paths
    #[arg(long)]
    pub no_migrations: bool,

    /// Disable terminal mouse capture in the interactive TUI.
    ///
    /// Pi normally captures all mouse motion to enable in-app wheel scrolling.
    /// On Windows / CMD.exe / Windows Terminal that capture blocks the
    /// terminal-native click-to-select / right-click-paste / Shift-Insert
    /// behaviour, making it effectively impossible to copy out the OAuth
    /// authorization URL (which is ~600 characters). Setting this flag (or
    /// `disable_mouse_capture: true` in settings, or `PI_NO_MOUSE_CAPTURE=1`)
    /// turns the capture off so terminal-native copy/paste keeps working.
    /// In-app mouse wheel scrolling is sacrificed; users can still scroll
    /// with Page Up/Down or arrow keys.
    ///
    /// Note: the env-var path is intentionally read in `run_interactive`
    /// (not via `#[arg(env = "...")]` here) so the truthiness semantics
    /// stay "only `=1` is truthy", matching how `PI_HARDWARE_CURSOR`
    /// behaves and avoiding clap's bool-env ambiguity where `=0` /
    /// `=false` may otherwise set the flag to true.
    #[arg(long)]
    pub no_mouse_capture: bool,

    // === Mode & Output ===
    /// Output mode for print mode (text, json, rpc)
    #[arg(long, value_parser = ["text", "json", "rpc"])]
    pub mode: Option<String>,

    /// Non-interactive mode (process & exit)
    #[arg(short = 'p', long)]
    pub print: bool,

    /// Start in RPC mode (alias for --mode rpc)
    #[arg(long, conflicts_with_all = ["mode", "print"])]
    pub rpc: bool,

    /// Start in ACP (Agent Client Protocol) mode for Zed editor integration.
    /// Reads JSON-RPC 2.0 requests from stdin and writes responses to stdout.
    #[arg(long)]
    pub acp: bool,

    /// Force verbose startup
    #[arg(long)]
    pub verbose: bool,

    // === Tools ===
    /// Disable all built-in tools
    #[arg(long)]
    pub no_tools: bool,

    /// Specific tools to enable (comma-separated). Default: the essential
    /// set plus discoverable tools behind the xdev dispatcher (bd-cv653.1.6);
    /// `subagent` stays opt-in only.
    #[arg(
        long,
        value_name = "TOOLS",
        default_value = "read,bash,edit,write,grep,find,ls,hashline_edit,web_search,ast_grep,ast_edit,lsp,debug,ask,todo,submit_plan,jobs,hub,current_time"
    )]
    pub tools: String,

    // === Extensions ===
    /// Load extension file (can use multiple times)
    #[arg(short = 'e', long, action = clap::ArgAction::Append)]
    pub extension: Vec<String>,

    /// Extra MCP server config file (can be repeated; highest precedence
    /// over .pi/mcp.json, .agents/mcp.json, ~/.pi/agent/mcp.json, and
    /// discovered foreign configs)
    #[arg(long, value_name = "PATH", action = clap::ArgAction::Append)]
    pub mcp_config: Vec<PathBuf>,

    /// Disable extension discovery
    #[arg(long)]
    pub no_extensions: bool,

    /// Trust this workspace: allow project-local .pi/settings.json packages
    /// and .pi/extensions to load and execute (persisted for the current
    /// content digest; content changes re-prompt)
    #[arg(long)]
    pub trust: bool,

    /// Extension capability policy: safe, balanced, or permissive (legacy alias: standard)
    #[arg(long, value_name = "PROFILE")]
    pub extension_policy: Option<String>,

    /// Print the resolved extension policy with per-capability decisions and exit
    #[arg(long)]
    pub explain_extension_policy: bool,

    /// Repair policy mode: off, suggest, auto-safe, or auto-strict
    #[arg(long, value_name = "MODE")]
    pub repair_policy: Option<String>,

    /// Print the resolved repair policy and exit
    #[arg(long)]
    pub explain_repair_policy: bool,

    // === Skills ===
    /// Load skill file/directory (can use multiple times)
    #[arg(long, action = clap::ArgAction::Append)]
    pub skill: Vec<String>,

    /// Disable skill discovery and configured skills (explicit --skill paths still load)
    #[arg(long)]
    pub no_skills: bool,

    // === Context files ===
    /// Disable AGENTS.md / CLAUDE.md discovery and loading (the global
    /// agent-dir file, the cwd, and every ancestor directory), plus the
    /// foreign-format workspace rules import. Use it when a host composes the
    /// whole system prompt itself (`--system-prompt`) and must not pick up
    /// ambient project instructions. Separate from `--no-skills` (gh #216).
    #[arg(long, env = "PI_NO_CONTEXT_FILES")]
    pub no_context_files: bool,

    // === Prompt Templates ===
    /// Load prompt template file/directory (can use multiple times)
    #[arg(long, action = clap::ArgAction::Append)]
    pub prompt_template: Vec<String>,

    /// Disable prompt template discovery
    #[arg(long)]
    pub no_prompt_templates: bool,

    // === Themes ===
    /// Select active theme (built-in name, discovered theme name, or theme JSON path)
    #[arg(long)]
    pub theme: Option<String>,

    /// Add theme file/directory to discovery (can use multiple times)
    #[arg(long = "theme-path", action = clap::ArgAction::Append)]
    pub theme_path: Vec<String>,

    /// Disable theme discovery
    #[arg(long)]
    pub no_themes: bool,

    // === System prompt modifiers ===
    /// Hide the current working directory from the system prompt.
    #[arg(long, env = "PI_HIDE_CWD_IN_PROMPT")]
    pub hide_cwd_in_prompt: bool,

    /// Maximum tool-call iterations per agent turn before stopping.
    /// Default: 50. Clamped to [1, 1000]; values outside the range fall back
    /// to 50 with a warning. Pairs with the iteration-aware-handoff protocol —
    /// at 80% of the cap, a one-shot steering message is injected so the agent
    /// can begin a graceful handoff rather than being silently killed at the
    /// ceiling. Override per-invocation via this flag, or globally via the
    /// `PI_MAX_TOOL_ITERATIONS` env var (read at agent start; invalid values
    /// fall back to the default with a warning, never abort startup).
    //
    // NOTE: `env =` is intentionally NOT set here. Clap's env wiring is strict
    // (an unparseable value aborts startup with a clap error), which would
    // defeat the lenient resolver semantics expected for this knob. The env
    // var is read inside `resolve_max_tool_iterations` instead, where bad
    // values warn-and-fall-back rather than fail the run.
    #[arg(long, value_name = "N")]
    pub max_tool_iterations: Option<usize>,

    /// Wall-clock cap for a run in seconds (bd-cv653.3.7): the agent pauses
    /// politely at the NEXT TURN BOUNDARY with a 'time cap reached' marker
    /// (never mid-tool-call), flushes session state, and exits 0 in print
    /// mode. Distinct from --request-timeout (per-request) and
    /// --max-tool-iterations (per-turn count).
    #[arg(long, value_name = "SECONDS")]
    pub max_time: Option<u64>,

    /// Additional workspace roots (bd-cv653.3.12): grant the agent access to
    /// extra directories beyond the primary cwd. Repeatable. Tools and the
    /// extension filesystem connector can then touch paths under ANY root;
    /// paths outside all roots stay fail-closed.
    #[arg(long = "add-dir", value_name = "DIR")]
    pub add_dir: Vec<std::path::PathBuf>,
    // === Export & Listing ===
    /// Export session file to HTML

    /// Inject an intentional panic to verify the crash-bundle pipeline
    /// (bd-cv653.7.12). Hidden smoke hook.
    #[arg(long, hide = true)]
    pub crash_test: bool,
    /// Start the sampling profiler for this run and write folded stacks
    /// under <agent-dir>/profiles/ (bd-cv653.7.12.1). Requires the
    /// `profiler` feature.
    #[arg(long)]
    pub profile: bool,

    // === Export & Listing ===
    /// Export session file to HTML
    #[arg(long)]
    pub export: Option<String>,

    /// List available models (optional fuzzy search pattern)
    #[arg(long)]
    #[allow(clippy::option_option)]
    // This is intentional: None = not set, Some(None) = set without value, Some(Some(x)) = set with value
    pub list_models: Option<Option<String>>,

    /// List all supported providers with aliases and auth env keys
    #[arg(long)]
    pub list_providers: bool,

    /// Fetch the live model catalog from a provider's `/v1/models` endpoint
    /// (OpenAI-compatible providers only). Falls back to the static registry
    /// when the live call fails. Long-lived library callers reuse successful
    /// results in-process for 5 minutes; separate CLI invocations do not share
    /// that cache. Set `PI_DISABLE_MODEL_CACHE=1` to bypass it.
    #[arg(long, value_name = "PROVIDER")]
    pub fetch_models: Option<String>,

    /// When used with `--fetch-models`, ignore any cached entry and require a
    /// successful fresh network call. Live-refresh failures are reported
    /// instead of being disguised as static-registry results.
    #[arg(long, requires = "fetch_models")]
    pub refresh_models: bool,

    /// Persist a verified live or same-process cached `--fetch-models` catalog
    /// to `models.fetched.json`. Static fallback results are never persisted.
    #[arg(long, requires = "fetch_models")]
    pub persist_models: bool,

    // === Subcommands ===
    #[command(subcommand)]
    pub command: Option<Commands>,

    // === Positional Arguments ===
    /// Messages and @file references
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

/// Package management subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Install extension/skill/prompt/theme from source
    Install {
        /// Package source (npm:pkg, git:url, or local path)
        source: String,
        /// Install locally (project) instead of globally
        #[arg(short = 'l', long)]
        local: bool,
    },

    /// Remove package from settings
    Remove {
        /// Package source to remove
        source: String,
        /// Remove from local (project) settings
        #[arg(short = 'l', long)]
        local: bool,
    },

    /// Update packages
    Update {
        /// Specific source to update (or all if omitted)
        source: Option<String>,
    },

    /// Refresh extension index cache from remote sources
    #[command(name = "update-index")]
    UpdateIndex,

    /// Manage pi-iso agent worktrees (bd-cv653.5.2)
    #[command(name = "worktree")]
    Worktree {
        /// `list` live agent worktrees or `clean` stale ones
        #[arg(value_parser = ["list", "clean"])]
        action: String,
        /// Reap worktrees older than this many days (clean only, default 1)
        #[arg(long, default_value = "1")]
        older_than_days: u64,
    },

    /// Print shell completion script from the live CLI graph (bd-cv653.7.2)
    #[command(name = "completions")]
    Completions {
        /// bash | zsh | fish
        #[arg(value_parser = ["bash", "zsh", "fish"])]
        shell: String,
    },

    /// Dynamic completion protocol (bd-cv653.7.2): answer candidates for a
    /// value-taking flag from the live registry/session index.
    #[command(name = "__complete", hide = true)]
    Complete {
        /// The flag being completed (`--model`, `--session`, ...). Hyphen
        /// values are allowed because the completed token IS a flag.
        #[arg(allow_hyphen_values = true)]
        flag: String,
        /// Prefix typed so far (may be empty)
        #[arg(default_value = "", allow_hyphen_values = true)]
        prefix: String,
    },

    /// Count tokens in text (or @file) against the active counter
    /// (bd-cv653.7.1) — price a prompt before sending it.
    #[command(name = "token")]
    Token {
        /// Text to count, or @file to read from disk
        input: String,
    },

    /// Render folded profiler stacks (bd-cv653.7.12.1): top functions by
    /// inclusive samples from a `.folded` snapshot.
    #[command(name = "profile")]
    Profile {
        /// Path to a `.folded` snapshot (defaults to the newest under
        /// <agent-dir>/profiles/)
        #[arg(long)]
        input: Option<PathBuf>,
        /// How many top rows to print
        #[arg(long, default_value_t = 25)]
        top: usize,
    },

    /// Aggregate local session usage: tokens/cost by provider, model, day;
    /// tool-call frequency; compactions (bd-cv653.7.7). All local — no
    /// network.
    #[command(name = "stats")]
    Stats {
        /// Only entries at/after this RFC 3339 timestamp (day prefixes work)
        #[arg(long)]
        since: Option<String>,
        /// Only entries at/before this RFC 3339 timestamp (day prefixes work)
        #[arg(long)]
        until: Option<String>,
        /// Only sessions under project dirs whose name contains this text
        #[arg(long)]
        project: Option<String>,
        /// Only assistant messages from this provider
        #[arg(long)]
        provider: Option<String>,
        /// Only assistant messages from this model
        #[arg(long)]
        model: Option<String>,
        /// Output format: text | json | markdown
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Import a foreign session into a native continuable pi session
    /// (bd-cv653.6.4): Claude Code or Codex JSONL.
    #[command(name = "import")]
    Import {
        /// Import from Claude Code (~/.claude/projects/**/*.jsonl)
        #[arg(long, conflicts_with = "from_codex")]
        from_claude: Option<String>,
        /// Import from Codex (~/.codex/sessions/**/*.jsonl)
        #[arg(long, conflicts_with = "from_claude")]
        from_codex: Option<String>,
    },

    /// Generate structured cross-session/cross-agent handoff brief (bd-cv653.3.17)
    #[command(name = "handoff")]
    Handoff {
        /// Delivery target: human | bead:<id> | agent:<thread_id>
        #[arg(long, default_value = "human")]
        to: String,
        /// Output file path for markdown brief (sidecar .json written alongside)
        #[arg(short, long)]
        out: Option<PathBuf>,
        /// Session ID or session file path (defaults to latest active session)
        #[arg(short, long)]
        session: Option<String>,
        /// Print generated handoff markdown directly to stdout
        #[arg(long)]
        print: bool,
    },

    /// Manage time-traveling stream rules (TTSR) (bd-cv653.3.4)
    #[command(name = "rules")]
    Rules {
        #[command(subcommand)]
        command: RulesCommands,
    },

    /// Manage per-project grievances ledger (bd-cv653.3.4)
    #[command(name = "grievances")]
    Grievances {
        #[command(subcommand)]
        command: GrievancesCommands,
    },

    /// Create dependency-ordered atomic commits from working tree changes (bd-cv653.3.14)
    #[command(name = "commit")]
    Commit {
        /// Dry-run mode: plan and preview atomic commits without writing to git
        #[arg(short = 'n', long)]
        dry_run: bool,
        /// Include lockfiles (Cargo.lock, etc.) in commit planning (excluded by default)
        #[arg(long)]
        include_lockfiles: bool,
        /// Automatically stage all untracked files
        #[arg(short = 'a', long)]
        all: bool,
        /// Optional bead / issue reference to annotate conventional commit messages
        #[arg(short = 'b', long)]
        bead: Option<String>,
        /// Optional custom commit message prefix
        #[arg(short = 'm', long)]
        message: Option<String>,
    },

    /// Verified in-place self-updater for Pi binary (bd-cv653.7.10)
    #[command(name = "self-update")]
    SelfUpdate {
        /// Target version to update to (e.g. v0.2.0 or 0.2.0; defaults to latest release)
        #[arg(long)]
        version: Option<String>,
        /// Check for available updates without applying any binary changes
        #[arg(long)]
        check: bool,
    },

    /// Prioritized parallel code review with ship verdict (bd-cv653.3.11)
    #[command(name = "review")]
    Review {
        /// Target to review: uncommitted (default), commit range (e.g. main..HEAD), or branch
        #[arg(value_name = "TARGET")]
        target: Option<String>,
        /// Fail with non-zero exit code if findings meet or exceed severity (P0, P1, P2)
        #[arg(long, value_name = "SEVERITY")]
        fail_on: Option<String>,
        /// Output format: text (default), json, or markdown
        #[arg(long, default_value = "text", value_parser = ["text", "json", "markdown"])]
        format: String,
        /// Minimum confidence threshold for findings (0.0 to 1.0)
        #[arg(long, default_value_t = 0.70)]
        confidence_threshold: f64,
        /// Maximum number of findings to report
        #[arg(long, default_value_t = 50)]
        max_findings: usize,
        /// Optional path to write output report to
        #[arg(short, long)]
        out: Option<PathBuf>,
    },

    /// Prune stale sessions, artifacts, and caches per retention policy (bd-cv653.7.11)
    #[command(name = "gc")]
    Gc {
        /// Retention window (e.g. 30d, 7d, 24h, or integer days; default: 30d)
        #[arg(long, default_value = "30d")]
        older_than: String,
        /// Number of most recent prunable sessions to preserve per project; named/pinned sessions are always kept and do not consume a slot (default: 5)
        #[arg(long, default_value_t = 5)]
        keep_last: usize,
        /// Include extension transpile caches and temporary runtime caches
        #[arg(long, default_value_t = true)]
        caches: bool,
        /// Perform dry run: analyze and print the reclamation plan without modifying disk (default: true)
        #[arg(long)]
        dry_run: bool,
        /// Confirm destructive sweep and move pruned items to trash
        #[arg(long, short = 'y')]
        yes: bool,
        /// Empty the trash directory permanently
        #[arg(long)]
        empty_trash: bool,
        /// Restore a previously trashed session by filename or ID
        #[arg(long)]
        restore: Option<String>,
        /// Output format: text (default) or json
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,
    },

    /// Preview the semantic context bundle Pi would use for a task
    #[command(name = "context-preview")]
    ContextPreview {
        /// Output format: text (default) or json
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,
        /// Bead ID to anchor the preview around
        #[arg(long)]
        bead: Option<String>,
        /// Changed path to anchor related context; repeatable
        #[arg(long = "changed-path", action = clap::ArgAction::Append)]
        changed_paths: Vec<String>,
        /// Failing command to match validation context
        #[arg(long = "failing-command")]
        failing_command: Option<String>,
        /// Maximum selected bundle items
        #[arg(long, default_value_t = 24)]
        max_items: usize,
        /// Maximum selected bundle bytes
        #[arg(long, default_value_t = 32 * 1024)]
        max_bytes: u64,
        /// Task query text used to score candidate context
        #[arg(trailing_var_arg = true)]
        query: Vec<String>,
    },

    /// Evaluate a normalized swarm progress SLO snapshot without live mutations
    #[command(name = "swarm-progress")]
    SwarmProgress {
        /// Normalized ProgressSloEvaluationInput JSON to evaluate
        #[arg(long)]
        input: String,
        /// Optional operator baseline; must match input.time_window.comparison_baseline
        #[arg(long)]
        since: Option<String>,
        /// Output format for stdout when no output path is supplied
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,
        /// Write schema-governed progress SLO JSON; refuses to overwrite
        #[arg(long = "out-json")]
        out_json: Option<String>,
        /// Write concise progress SLO text; refuses to overwrite
        #[arg(long = "out-text")]
        out_text: Option<String>,
    },

    /// Preview an offline swarm replay trace and policy comparison
    #[command(name = "swarm-replay-preview")]
    SwarmReplayPreview {
        /// Normalized pi.swarm.replay_trace.v1 JSON to replay
        #[arg(long)]
        trace: String,
        /// Baseline policy to compare; repeatable, defaults to all built-in policies
        #[arg(long = "policy", action = clap::ArgAction::Append)]
        policies: Vec<String>,
        /// Output format for stdout when no output path is supplied
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,
        /// Write schema-governed preview JSON; refuses to overwrite
        #[arg(long = "out-json")]
        out_json: Option<String>,
        /// Write concise preview text; refuses to overwrite
        #[arg(long = "out-text")]
        out_text: Option<String>,
        /// Override generation timestamp for deterministic fixtures
        #[arg(long = "generated-at")]
        generated_at: Option<String>,
    },

    /// Inspect and mutate validation-broker slot leases
    #[command(name = "validation-broker")]
    ValidationBroker {
        #[command(subcommand)]
        command: ValidationBrokerCommand,
    },

    /// Show detailed information about an extension
    Info {
        /// Extension name or id to look up
        name: String,
    },

    /// Search available extensions by keyword
    Search {
        /// Search query (e.g. "git", "auto commit")
        query: String,
        /// Filter results by tag
        #[arg(long)]
        tag: Option<String>,
        /// Sort results: relevance, name
        #[arg(long, default_value = "relevance")]
        sort: String,
        /// Maximum number of results
        #[arg(long, default_value = "25")]
        limit: usize,
    },

    /// List installed packages
    List,

    /// Open configuration UI
    Config {
        /// Print configuration summary as text (non-interactive)
        #[arg(long)]
        show: bool,
        /// Print path and precedence details only
        #[arg(long)]
        paths: bool,
        /// Print configuration details as JSON
        #[arg(long)]
        json: bool,
    },

    /// Diagnose environment health and extension compatibility
    Doctor {
        /// Extension path to check (omit to run all environment checks)
        path: Option<String>,
        /// Output format: text (default), json, markdown
        #[arg(long, default_value = "text")]
        format: String,
        /// Extension policy profile to check against
        #[arg(long)]
        policy: Option<String>,
        /// Automatically fix safe issues (missing dirs, permissions)
        #[arg(long)]
        fix: bool,
        /// Run specific categories: config,dirs,auth,shell,sessions,swarm,extensions
        #[arg(long)]
        only: Option<String>,
    },

    /// Migrate session files from JSONL v1 to v2 segment format
    /// Show provider usage/quota state (bd-cv653.7.4)
    Usage {
        /// Output format: text (default) or json
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,
        /// Force live reads (skip the 60s cache)
        #[arg(long)]
        refresh: bool,
    },

    /// Serve the agent session over a Web interface via WebSocket frame diffs (bd-cv653.10.1)
    Web {
        /// Port to bind web server (default: 8080)
        #[arg(long, default_value_t = 8080)]
        port: u16,
        /// Network interface binding mode: loopback (default), tailscale, lan
        #[arg(long, default_value = "loopback", value_parser = ["loopback", "tailscale", "lan"])]
        bind: String,
        /// Connect in view-only mode (disallows input from web clients)
        #[arg(long)]
        view_only: bool,
        /// Maximum concurrent connected web viewers (default: 4)
        #[arg(long, default_value_t = 4)]
        max_viewers: usize,
    },

    /// Visual component gallery harness (bd-cv653.9.10)
    Gallery {
        /// Output format: text (default) or json
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,
    },

    Migrate {
        /// Path to specific session JSONL file (or directory to migrate all)
        path: String,
        /// Dry-run: validate migration without persisting changes
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ValidationBrokerCommand {
    /// Print current slot-store status without mutating it
    Status {
        /// Append-only validation slot JSONL store
        #[arg(long)]
        store: String,
        /// Output format when no output path is supplied
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,
        /// Write schema-governed JSON; refuses to overwrite
        #[arg(long = "out-json")]
        out_json: Option<String>,
        /// Write concise text; refuses to overwrite
        #[arg(long = "out-text")]
        out_text: Option<String>,
        /// Override report timestamp for deterministic fixtures
        #[arg(long = "generated-at")]
        generated_at: Option<String>,
    },

    /// Plan whether to run, narrow, wait, coalesce, or surface a blocker
    Plan {
        /// ValidationAdmissionRequestContext JSON
        #[arg(long)]
        request: String,
        /// ValidationBrokerInputSnapshot JSON
        #[arg(long)]
        inputs: String,
        /// Append-only validation slot JSONL store to inspect
        #[arg(long)]
        store: String,
        /// Optional ValidationAdmissionPolicy JSON
        #[arg(long)]
        policy: Option<String>,
        /// Output format when no output path is supplied
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,
        /// Write schema-governed JSON; refuses to overwrite
        #[arg(long = "out-json")]
        out_json: Option<String>,
        /// Write concise text; refuses to overwrite
        #[arg(long = "out-text")]
        out_text: Option<String>,
        /// Override report timestamp for deterministic fixtures
        #[arg(long = "generated-at")]
        generated_at: Option<String>,
    },

    /// Acquire a slot by appending an active lease record
    Acquire {
        /// ValidationSlotRequest JSON
        #[arg(long)]
        request: String,
        /// Append-only validation slot JSONL store
        #[arg(long)]
        store: String,
        /// Lease start timestamp in UTC RFC3339
        #[arg(long = "started-at")]
        started_at: String,
        /// Lease expiry timestamp in UTC RFC3339
        #[arg(long = "expires-at")]
        expires_at: String,
        /// Output format when no output path is supplied
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,
        /// Write schema-governed JSON; refuses to overwrite
        #[arg(long = "out-json")]
        out_json: Option<String>,
        /// Write concise text; refuses to overwrite
        #[arg(long = "out-text")]
        out_text: Option<String>,
    },

    /// Renew a slot owned by the caller
    Renew {
        /// Append-only validation slot JSONL store
        #[arg(long)]
        store: String,
        /// Slot ID to renew
        #[arg(long = "slot-id")]
        slot_id: String,
        /// Owning agent name
        #[arg(long)]
        owner: String,
        /// New heartbeat timestamp in UTC RFC3339
        #[arg(long = "heartbeat-at")]
        heartbeat_at: String,
        /// New expiry timestamp in UTC RFC3339
        #[arg(long = "expires-at")]
        expires_at: String,
        /// Output format when no output path is supplied
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,
        /// Write schema-governed JSON; refuses to overwrite
        #[arg(long = "out-json")]
        out_json: Option<String>,
        /// Write concise text; refuses to overwrite
        #[arg(long = "out-text")]
        out_text: Option<String>,
    },

    /// Release a slot owned by the caller
    Release {
        /// Append-only validation slot JSONL store
        #[arg(long)]
        store: String,
        /// Slot ID to release
        #[arg(long = "slot-id")]
        slot_id: String,
        /// Owning agent name
        #[arg(long)]
        owner: String,
        /// Release timestamp in UTC RFC3339
        #[arg(long)]
        at: String,
        /// Release reason
        #[arg(long)]
        reason: String,
        /// Output format when no output path is supplied
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,
        /// Write schema-governed JSON; refuses to overwrite
        #[arg(long = "out-json")]
        out_json: Option<String>,
        /// Write concise text; refuses to overwrite
        #[arg(long = "out-text")]
        out_text: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum RulesCommands {
    /// List configured stream rules
    List {
        /// Show global rules in addition to project rules
        #[arg(long)]
        global: bool,
    },
    /// Add a new stream rule
    Add {
        /// Rule identifier (e.g. no-box-leak)
        #[arg(short, long)]
        id: String,
        /// Rule display name
        #[arg(short, long)]
        name: String,
        /// Matching regex pattern
        #[arg(short, long)]
        pattern: String,
        /// Reminder directive body injected on match
        #[arg(short, long)]
        body: String,
        /// Save to global settings (~/.pi/agent/stream-rules.json) instead of project
        #[arg(long)]
        global: bool,
        /// Optional turn cooldown in turns
        #[arg(long)]
        cooldown: Option<usize>,
    },
    /// Remove a stream rule by ID
    Remove {
        /// Rule ID
        id: String,
    },
    /// Test a regex pattern or existing rule against sample text
    Test {
        /// Regex pattern or rule ID
        pattern: String,
        /// Sample text to test against
        sample: String,
    },
    /// Export stream rules as JSON
    Export,
    /// Import stream rules from JSON file or stdin
    Import {
        /// File path or "-" for stdin
        path: String,
        /// Import to global rules
        #[arg(long)]
        global: bool,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum GrievancesCommands {
    /// List recorded grievances
    List,
    /// Record a user complaint / grievance
    Add {
        /// Complaint description
        complaint: String,
    },
    /// Forge a stream rule from a grievance
    ForgeRule {
        /// Grievance ID
        id: String,
    },
}

impl Cli {
    /// Get file arguments (prefixed with @)
    pub fn file_args(&self) -> Vec<&str> {
        self.args
            .iter()
            .filter(|a| a.starts_with('@'))
            .map(|a| a.strip_prefix('@').unwrap_or(a))
            .collect()
    }

    /// Get message arguments (not prefixed with @)
    pub fn message_args(&self) -> Vec<&str> {
        self.args
            .iter()
            .filter(|a| !a.starts_with('@'))
            .map(String::as_str)
            .collect()
    }

    /// Get enabled tools as a list
    pub fn enabled_tools(&self) -> Vec<&str> {
        if self.no_tools {
            vec![]
        } else {
            let mut seen = HashSet::new();
            self.tools
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .filter(|name| seen.insert(*name))
                .collect()
        }
    }
}
