# pi.rs 模块化复刻详细差距分析 (crates2.md)

> 本文档是 `crates1.md` v1.2 + Round 23 差距分析的续篇,**逐文件**对比
> 上游 `legacy_pi_mono_code/pi/`(= `https://github.com/earendil-works/pi.git` 快照,
> 已 commit 到本仓库)与本仓库 Rust 实现。
>
> **目标**:把代码归属复刻度从 ~30% 推到 ~64%(Round 30)。

---

## 0. 摘要

| 维度 | 当前 | 目标(Round 30) |
|------|------|-----------------|
| **结构镜像**(11 同名 crate) | 100% | 100% |
| **代码归属镜像**(本文件逐文件分析) | **~30%** | **~64%** |
| Provider 覆盖 | 15% (13/87) | 40%+ |
| `cargo check -p pi-coding-agent` | ✅ | ✅ |
| 7 个空 marker crate | 7 | 0-2 |

---

## 1. `@earendil-works/pi-agent-core` (180 .ts, 50,136 LOC) ↔ `pi-agent-core`

### 1.1 上游文件清单(8 个顶层 + harness/ + search/)

| 上游文件 | 大小 | 我们 Rust 归宿 |
|---------|------|----------------|
| `agent/src/agent.ts` | 18,767 | ⚠️ 部分在 `pi-coding-agent/agent.rs` |
| `agent/src/agent-loop.ts` | 22,796 | ❌ **缺失**(无等价物) |
| `agent/src/types.ts` | 446 | ⚠️ `pi-coding-agent/agent.rs` 局部 |
| `agent/src/node.ts` | 2 | ❌ 缺失(只 stub) |
| `agent/src/proxy.ts` | 402 | ❌ 缺失 |
| `agent/src/stream-fn.ts` | 20 | ⚠️ `pi-coding-agent/extensions_api.rs` |
| `agent/src/index.ts` | 152 | ✅ `pi-agent-core/src/lib.rs` |
| `agent/src/harness/agent-harness.ts` | 21,541 | ❌ 缺失 |
| `agent/src/harness/compaction/compaction.ts` | 27,410 | ⚠️ `pi-coding-agent/compaction*.rs` |
| `agent/src/harness/compaction/branch-summarization.ts` | 9,537 | ❌ 缺失 |
| `agent/src/harness/compaction/utils.ts` | 4,312 | ⚠️ 部分内联 |
| `agent/src/harness/env/nodejs.ts` | 29,097 | ❌ 缺失(node env 抽象) |
| `agent/src/harness/events.ts` | 9,852 | ⚠️ `pi-coding-agent/event-bus` |
| `agent/src/harness/execution/assistant.ts` | 5,746 | ⚠️ 部分在 agent.rs |
| `agent/src/harness/execution/tools.ts` | 7,227 | ⚠️ 部分在 agent.rs |
| `agent/src/harness/execution/effect-gate.ts` | 1,817 | ❌ 缺失 |
| `agent/src/harness/hooks.ts` | 17,258 | ❌ 缺失(我们没有 hooks 系统) |
| `agent/src/harness/messages.ts` | 4,188 | ⚠️ 部分在 agent.rs |
| `agent/src/harness/prompt-templates.ts` | 9,407 | ❌ 缺失 |
| `agent/src/harness/result.ts` | 4,055 | ⚠️ 部分在 agent.rs |
| `agent/src/harness/runtime/drive/*.ts` | ~120K | ❌ **大面积缺失**(checkpoint / deferred / generation / reconcile / recovery / response / retry / structural / terminal / tool-placement / tools) |
| `agent/src/harness/skills.ts` | ? | ❌ 缺失(我们 `skills_managed.rs` 不在 coding-agent) |
| `agent/src/harness/system-prompt.ts` | ? | ⚠️ 部分内联 |
| `agent/src/harness/telemetry.ts` | ? | ⚠️ 部分内联 |
| `agent/src/harness/tools/*.ts` | ? | ⚠️ `pi-coding-agent/tools/` |
| `agent/src/search/*.ts` | ? | ❌ 缺失 |

### 1.2 我们的 Rust 状态
- `crates/pi-agent-core/src/`:仅 3 个文件 (`flake_classifier.rs`, `scheduler.rs`, `lib.rs`)
- 真实 agent 代码全部在 `pi-coding-agent`:`agent.rs / agent_cx.rs / agent_hub.rs / handoff.rs / subagents.rs`
- `pi-agent-core/src/scheduler.rs` (4591 LOC) 是上游 `agent-loop.ts` 的部分对应

### 1.3 Round 30 拆解清单
1. `pi-coding-agent/agent.rs / agent_cx.rs / agent_hub.rs / handoff.rs / subagents.rs` → `pi-agent-core/src/`
2. `pi-coding-agent/skills_managed.rs` → `pi-agent-core/src/skills.rs`
3. 新增 `pi-agent-core/src/hooks.rs`(对应 upstream `harness/hooks.ts`)
4. 新增 `pi-agent-core/src/checkpoint.rs`(对应 `drive/checkpoint.ts`)

---

## 2. `@earendil-works/pi-ai` (340 .ts, 65,515 LOC) ↔ `pi-ai`

### 2.1 上游文件清单(分类)

| 类别 | 文件数 | LOC | 我们 Rust 归宿 |
|------|-------|-----|----------------|
| `api/*.ts`(API 协议实现) | 32 | ~370K | ❌ **全部缺失** |
| `auth/*.ts`(OAuth / API key) | 6+ | ~? | ⚠️ 部分在 `pi-coding-agent/auth.rs` |
| `compat/*.ts` | ? | ~? | ❌ 缺失 |
| `providers/*.ts`(provider 实现) | 87 | ~? | ⚠️ **15% 覆盖** (13/87 在 `pi-coding-agent/providers/`) |
| `models.ts / models.generated.ts` | 2 | ~? | ✅ `pi-ai/model.rs + pi-ai/models.rs` |
| `model-catalog.ts / models-store.ts` | 2 | ~? | ✅ `pi-ai/model.rs` |
| `image-models.ts` | ? | ~? | ✅ `pi-ai/model.rs` |
| `magic_keywords.ts` | ? | ~? | ✅ `pi-ai/magic_keywords.rs` |
| `oauth.ts / bun-oauth.ts` | 2 | ~? | ❌ 缺失 |
| `env-api-keys.ts` | ? | ~? | ❌ 缺失 |
| `embedded-assets.ts`(等) | ? | ~? | ✅ `pi-ai/embedded_assets.rs` |
| `legacy-api-aliases.ts` | ? | ~? | ❌ 缺失 |
| `cli.ts`(AI CLI 工具) | ? | ~? | ❌ 缺失 |

### 2.2 `api/` 子目录详解(全部缺失)

| 上游 api/*.ts | 大小 | 状态 |
|--------------|------|------|
| `anthropic-messages.ts` | 48,616 | ❌ 缺失(只在 `pi-coding-agent/providers/anthropic.rs` 有简化版) |
| `bedrock-converse-stream.ts` | 48,022 | ❌ 缺失 |
| `openai-completions.ts` | 62,416 | ❌ 缺失 |
| `openai-codex-responses.ts` | 54,470 | ❌ 缺失 |
| `openai-responses-shared.ts` | 29,490 | ❌ 缺失 |
| `mistral-conversations.ts` | 30,365 | ❌ 缺失 |
| `openai-responses.ts` | 13,908 | ⚠️ 部分在 `pi-coding-agent/providers/openai_responses.rs` |
| `google-vertex.ts` | 18,373 | ⚠️ 部分在 `pi-coding-agent/providers/vertex.rs` |
| `google-shared.ts` | 15,654 | ❌ 缺失 |
| `google-generative-ai.ts` | 16,027 | ⚠️ 部分在 `pi-coding-agent/providers/gemini.rs` |
| `pi-messages.ts` | 13,451 | ❌ 缺失 |
| `azure-openai-responses.ts` | 11,337 | ⚠️ 部分在 `pi-coding-agent/providers/azure.rs` |
| `constrained-sampling.ts` | 9,295 | ❌ 缺失 |
| `openrouter-images.ts` | 6,132 | ❌ 缺失 |
| `cloudflare-ai-binding.ts` | 4,604 | ❌ 缺失 |
| `lazy.ts` | 3,203 | ❌ 缺失 |
| 其他 | ? | ❌ 缺失 |
| + 14 个 `.lazy.ts` | 200 字节左右 | ❌ 缺失(lazy chunk pattern) |

### 2.3 我们的 Rust 状态
- `crates/pi-ai/src/`:16 个文件,主要覆盖 models / provider / metadata / token / failover
- **74 个 provider 完全缺失**(Round 29 待办)
- **整个 `api/` 抽象层缺失**(Round 29 待办)
- **整个 `auth/` 抽象层缺失**(Round 30+ 顺带补)

### 2.4 Round 29 拆解/补齐清单
1. 新增 `pi-ai/src/api/anthropic_messages.rs`(48.6K 等价)
2. 新增 `pi-ai/src/api/openai_responses.rs`(13.9K)
3. 新增 `pi-ai/src/api/openai_completions.rs`(62.4K)
4. 新增 `pi-ai/src/api/google_generative_ai.rs`(16K)
5. 新增 `pi-ai/src/api/bedrock_converse_stream.rs`(48K)
6. 新增 `pi-ai/src/api/mistral_conversations.rs`(30.4K)
7. 新增 `pi-ai/src/auth/{oauth,bun_oauth,env_api_keys,credential_store,resolve,helpers}.rs`
8. 把 `pi-coding-agent/providers/{anthropic,openai,openai_responses,gemini,vertex,azure,bedrock}.rs` 移到 `pi-ai/src/providers/`
9. 新增 74 个缺失 providers(只实现常用 20-30 个)

---

## 3. `@earendil-works/pi-chord` (34 .ts, 9,375 LOC) ↔ `pi-chord`

### 3.1 上游文件清单(全部缺失)

| 上游文件 | 大小 | 我们 Rust 归宿 |
|---------|------|----------------|
| `chord/src/api.ts` | 3,010 | ❌ 缺失 |
| `chord/src/bundler.ts` | 386 | ❌ 缺失 |
| `chord/src/context/index.ts` | 3,763 | ❌ 缺失 |
| `chord/src/delta/index.ts` | 45,393 | ❌ 缺失(超大) |
| `chord/src/facets/host.ts` | 32,110 | ❌ 缺失 |
| `chord/src/facets/loader.ts` | 324 | ❌ 缺失 |
| `chord/src/index.ts` | 1,849 | ⚠️ 我们有 `pi-chord/src/lib.rs` (12 LOC) |
| `chord/src/json.ts` | 1,774 | ❌ 缺失 |
| `chord/src/node.ts` | 610 | ❌ 缺失 |
| `chord/src/node/bundle-loader.ts` | 16,573 | ❌ 缺失 |
| `chord/src/node/bundle.ts` | 8,876 | ❌ 缺失 |
| `chord/src/node/manifest.ts` | 1,514 | ❌ 缺失 |
| `chord/src/node/package.ts` | 9,232 | ❌ 缺失 |
| `chord/src/services/consumer.ts` | 21,879 | ❌ 缺失 |
| `chord/src/services/errors.ts` | 785 | ❌ 缺失 |
| `chord/src/services/handle.ts` | 3,468 | ❌ 缺失 |
| `chord/src/services/instances.ts` | 4,244 | ❌ 缺失 |
| `chord/src/services/loopback.ts` | 653 | ❌ 缺失 |
| `chord/src/services/provider.ts` | 21,121 | ❌ 缺失 |
| `chord/src/services/state-codec.ts` | 5,010 | ❌ 缺失 |
| `chord/src/services/state-internals.ts` | 733 | ❌ 缺失 |
| `chord/src/services/state.ts` | 4,950 | ❌ 缺失 |
| `chord/src/services/wire.ts` | 9,020 | ❌ 缺失 |
| `chord/src/types.ts` | 9,237 | ❌ 缺失 |

### 3.2 我们的 Rust 状态
- `crates/pi-chord/src/`:1 个文件 (12 LOC 空 marker)
- 真正的 chord 概念被吸收到 `pi-coding-agent/extensions/`(hostcall / buffer / file_lock / swarm)
- 真正的 chord facets 在 `pi-coding-agent/extensions_api.rs`(~25K LOC)

### 3.3 Round 27 拆解清单
1. `pi-coding-agent/extensions_api.rs` → `pi-chord/src/api.rs` (25K)
2. `pi-coding-agent/extensions/mod.rs` + `extensions/*.rs` → `pi-chord/src/services/`(消费侧 hostcall / state)
3. `pi-coding-agent/hostcall_*.rs / buffer_shim.rs / file_lock.rs` → `pi-chord/src/node/`
4. `pi-coding-agent/extensions/permission_drift.rs` → `pi-chord/src/facets/`
5. 新增 `pi-chord/src/delta/`(对应 upstream 45K 的 `delta/index.ts`)
6. 新增 `pi-chord/src/context/`(对应 `context/index.ts`)
7. 新增 `pi-chord/src/bundler.rs`(对应 `bundler.ts`)
8. 抽象 `chord::Facets` trait 让 `pi-coding-agent` 不直接依赖具体类型

---

## 4. `@earendil-works/pi-client` (13 .ts, 1,951 LOC) ↔ `pi-client`

### 4.1 上游文件清单

| 上游文件 | 大小 | 我们 Rust 归宿 |
|---------|------|----------------|
| `client/src/client.ts` | 14,332 | ❌ 缺失 |
| `client/src/connection.ts` | 7,691 | ❌ 缺失 |
| `client/src/errors.ts` | 965 | ❌ 缺失 |
| `client/src/index.ts` | 428 | ⚠️ `pi-client/src/lib.rs` (13 LOC) |
| `client/src/promise.ts` | 582 | ❌ 缺失 |
| `client/src/transport.ts` | 727 | ❌ 缺失 |
| `client/src/types.ts` | 1,139 | ❌ 缺失 |
| `client/src/unix.ts` | 9,703 | ❌ 缺失 |

### 4.2 我们的 Rust 状态
- `crates/pi-client/src/`:1 个文件 (13 LOC 空 marker)
- 真正的 client 代码在 `pi-coding-agent/web_remote.rs` (~? LOC) + `connectors/`

### 4.3 Round 25 拆解清单
1. `pi-coding-agent/web_remote.rs` → `pi-client/src/web_remote.rs`
2. `pi-coding-agent/connectors/` → `pi-client/src/connectors/`
3. 新增 `pi-client/src/transport.rs`(对应 upstream transport)
4. 新增 `pi-client/src/unix.rs`(对应 upstream unix transport)

---

## 5. `@earendil-works/pi-coding-agent` (646 .ts, 143,033 LOC) ↔ `pi-coding-agent`

### 5.1 上游文件清单(按子目录)

#### 5.1.1 `coding-agent/src/` 顶层(9 文件)
| 上游文件 | 大小 | 我们 Rust 归宿 |
|---------|------|----------------|
| `bun/cli.ts` | 102 | ❌ 缺失(bun runtime stub) |
| `bun/restore-sandbox-env.ts` | 1,148 | ❌ 缺失 |
| `bun/runtime-setup.ts` | 432 | ❌ 缺失 |
| `bun/sandbox-env-setup.ts` | 162 | ❌ 缺失 |
| `cli.ts` | 139 | ⚠️ `pi-coding-agent/cli/` |
| `config.ts` | 19,600 | ✅ `pi-coding-agent/config.rs` |
| `client/index.ts` | 43 | ⚠️ `pi-coding-agent/connectors/` |
| `main.ts` | ? | ✅ `pi-coding-agent/main.rs`(binary 入口) |
| `migrations.ts` | ? | ❌ 缺失 |
| `modes/*.ts` | ? | ❌ 缺失 |
| `package-manager-cli.ts` | ? | ❌ 缺失 |
| `rpc-entry.ts` | ? | ❌ 缺失 |
| `utils/*.ts` | ? | ⚠️ 部分内联 |

#### 5.1.2 `coding-agent/src/cli/` (20+ 文件)
| 上游文件 | 大小 | 我们 Rust 归宿 |
|---------|------|----------------|
| `cli/args.ts` | 19,651 | ⚠️ `pi-coding-agent/cli_args.rs` (部分) |
| `cli/auth-check.ts` | 2,718 | ⚠️ `pi-coding-agent/auth.rs` |
| `cli/auth-command.ts` | 4,899 | ⚠️ `pi-coding-agent/auth.rs` |
| `cli/config-selector.ts` | 1,669 | ⚠️ `pi-coding-agent/cli/` |
| `cli/credential-print.ts` | 3,888 | ❌ 缺失 |
| `cli/file-processor.ts` | 2,739 | ❌ 缺失 |
| `cli/initial-message.ts` | 996 | ❌ 缺失 |
| `cli/list-models.ts` | 3,463 | ⚠️ `pi-coding-agent/cli/` |
| `cli/project-trust.ts` | 1,761 | ⚠️ `pi-coding-agent/workspace_trust.rs` |
| `cli/session-picker.ts` | 1,588 | ⚠️ `pi-coding-agent/session_picker.rs` |
| `cli/setup.ts` | 484 | ❌ 缺失 |
| `cli/startup-ui.ts` | 7,411 | ⚠️ `pi-coding-agent/app.rs` |
| `cli/experimental/cli.ts` | 624 | ❌ 缺失 |
| `cli/experimental/command.ts` | 7,733 | ❌ 缺失 |
| `cli/experimental/command-options.ts` | 3,600 | ❌ 缺失 |
| `cli/experimental/commands/client.ts` | 3,491 | ❌ 缺失 |
| `cli/experimental/commands/server.ts` | 2,358 | ❌ 缺失 |

#### 5.1.3 `coding-agent/src/core/` (~50 文件,最大子目录)
| 上游核心文件 | 大小 | 我们 Rust 归宿 |
|-------------|------|----------------|
| `core/agent-session.ts` | 120,916 | ⚠️ `pi-coding-agent/session.rs` + `agent.rs`(部分) |
| `core/agent-session-runtime.ts` | 15,127 | ⚠️ `pi-coding-agent/session_runtime.rs` (无,可能内联) |
| `core/agent-session-services.ts` | 7,367 | ⚠️ 部分在 session.rs |
| `core/auth-storage.ts` | 16,295 | ⚠️ `pi-coding-agent/auth.rs` |
| `core/bash-executor.ts` | 4,487 | ❌ 缺失 |
| `core/cache-stats.ts` | ? | ❌ 缺失 |
| `core/compaction/*.ts` | ? | ⚠️ `pi-coding-agent/compaction*.rs` |
| `core/defaults.ts` | ? | ❌ 缺失 |
| `core/diagnostics.ts` | ? | ❌ 缺失 |
| `core/event-bus.ts` | ? | ❌ 缺失 |
| `core/exec.ts` | ? | ❌ 缺失 |
| `core/experimental.ts` | ? | ❌ 缺失 |
| `core/export-html/*.ts` | ? | ❌ 缺失 |
| `core/extensions/*.ts` | ? | ⚠️ `pi-coding-agent/extensions/` |
| `core/footer-data-provider.ts` | ? | ❌ 缺失 |
| `core/http-dispatcher.ts` | ? | ⚠️ `pi-coding-agent/connectors/http.rs` |
| `core/keybindings.ts` | ? | ⚠️ `pi-coding-agent/keybindings.rs` |
| `core/messages.ts` | ? | ⚠️ `pi-coding-agent/messages.rs`(无,可能内联) |
| `core/model-config.ts` | ? | ⚠️ `pi-coding-agent/config.rs` |
| `core/model-registry.ts` | ? | ⚠️ `pi-coding-agent/models.rs`(在 pi-ai) |
| `core/model-resolver.ts` | ? | ❌ 缺失 |
| `core/model-runtime.ts` | ? | ⚠️ 部分在 agent.rs |
| `core/models-store.ts` | ? | ⚠️ `pi-ai/models.rs` |
| `core/output-guard.ts` | ? | ❌ 缺失 |
| `core/package-manager.ts` | ? | ⚠️ `pi-coding-agent/package_manager.rs` |
| `core/pi-manifest.ts` | ? | ❌ 缺失 |
| `core/project-trust.ts` | ? | ⚠️ `pi-coding-agent/workspace_trust.rs` |
| `core/prompt-templates.ts` | ? | ⚠️ `pi-coding-agent/prompt_templates.rs`(无,可能内联) |
| `core/provider-attribution.ts` | ? | ❌ 缺失 |
| `core/provider-composer.ts` | ? | ❌ 缺失 |
| `core/radius.ts` | ? | ❌ 缺失 |
| `core/remote-catalog-provider.ts` | ? | ❌ 缺失 |
| `core/resolve-config-value.ts` | ? | ❌ 缺失 |
| `core/resource-loader.ts` | ? | ⚠️ `pi-coding-agent/resources.rs` |
| `core/runtime-credentials.ts` | ? | ⚠️ `pi-coding-agent/auth.rs` |
| `core/sdk.ts` | ? | ⚠️ `pi-coding-agent/sdk.rs` |
| `core/session-cwd.ts` | ? | ❌ 缺失 |
| `core/session-export.ts` | ? | ❌ 缺失 |
| `core/session-manager.ts` | ? | ⚠️ `pi-coding-agent/session.rs` |
| `core/settings-diagnostics.ts` | ? | ❌ 缺失 |
| `core/settings-manager.ts` | ? | ⚠️ `pi-coding-agent/config.rs` |
| `core/skills.ts` | ? | ⚠️ `pi-coding-agent/skills_managed.rs` |
| `core/slash-commands.ts` | ? | ⚠️ `pi-coding-agent/slash_commands.rs`(无,可能内联) |
| `core/source-info.ts` | ? | ❌ 缺失 |
| `core/system-prompt.ts` | ? | ⚠️ `pi-coding-agent/system_prompt.rs`(无,可能内联) |
| `core/telemetry.ts` | ? | ⚠️ `pi-telemetry/` |
| `core/timings.ts` | ? | ❌ 缺失 |
| `core/tools/*.ts` | ? | ⚠️ `pi-coding-agent/tools/` |
| `core/trust-manager.ts` | ? | ⚠️ `pi-coding-agent/workspace_trust.rs` |
| `core/usage-totals.ts` | ? | ⚠️ `pi-ai/usage.rs` |

#### 5.1.4 `coding-agent/src/extensions/` (小)
| 上游文件 | 大小 | 我们 Rust 归宿 |
|---------|------|----------------|
| `extensions/index.ts` | ? | ⚠️ `pi-coding-agent/extensions/mod.rs` |
| `extensions/llama/*.ts` | ? | ❌ 缺失(我们没有 llama extensions) |

#### 5.1.5 `coding-agent/src/tools/` (工具集,~80 文件)
| 上游工具类别 | 我们 Rust 归宿 |
|-------------|----------------|
| `tools/bash.ts` | ⚠️ `pi-coding-agent/tools/bash.rs` (无,可能内联) |
| `tools/edit.ts` | ⚠️ `pi-coding-agent/tools/edit.rs`(可能内联) |
| `tools/read.ts` | ⚠️ `pi-coding-agent/tools/read.rs` |
| `tools/write.ts` | ⚠️ `pi-coding-agent/tools/write.rs` |
| `tools/grep.ts` | ❌ 缺失(我们用 `grep-regex` 库) |
| `tools/find.ts` | ❌ 缺失 |
| `tools/ls.ts` | ❌ 缺失 |
| `tools/web-fetch.ts` | ❌ 缺失 |
| `tools/web-search.ts` | ❌ 缺失 |
| `tools/task.ts`(subagent) | ⚠️ `pi-coding-agent/subagents.rs` |
| `tools/todo.ts` | ❌ 缺失 |
| 其他 ~70 个工具 | ⚠️ 大部分缺失或简化 |

### 5.2 我们的 Rust 状态
- `crates/pi-coding-agent/src/`:127 个 .rs 文件顶层 + 10 个子目录
- 完整工具集集中在 `crates/pi-coding-agent/src/tools/`(具体数量待统计)
- 实际上是我们做得最完整的 crate(因为 binary 在这里)

### 5.3 Round 24-30 不动 `pi-coding-agent`
- `pi-coding-agent` 已经基本完整,只是工具集还需要扩充

---

## 6. `@earendil-works/pi-evals` (17 .ts, 2,474 LOC) ↔ `pi-evals`

### 6.1 上游文件清单(全部缺失)

| 上游文件 | 大小 | 我们 Rust 归宿 |
|---------|------|----------------|
| `evals/src/docs.eval.ts` | 3,755 | ❌ 缺失 |
| `evals/src/extensions.eval.ts` | 4,899 | ❌ 缺失 |
| `evals/src/models.eval.ts` | 5,106 | ❌ 缺失 |
| `evals/src/pi-harness.ts` | 11,207 | ❌ 缺失 |
| `evals/src/providers.eval.ts` | 14,442 | ❌ 缺失 |
| `evals/src/smoke.eval.ts` | 720 | ❌ 缺失 |
| `evals/src/vitest-evals/artifacts.ts` | 3,360 | ❌ 缺失 |
| `evals/src/vitest-evals/harness-table.ts` | 7,553 | ❌ 缺失 |
| `evals/src/vitest-evals/reporter.ts` | 4,940 | ❌ 缺失 |
| `evals/src/vitest-evals/setup.ts` | 260 | ❌ 缺失 |
| `evals/src/vitest-evals/summary.ts` | 13,791 | ❌ 缺失 |

### 6.2 我们的 Rust 状态
- `crates/pi-evals/src/`:1 个文件 (12 LOC 空 marker)
- 真正的 eval 代码在 `pi-coding-agent/eval/` 和 `pi-coding-agent/conformance*.rs`

### 6.3 Round 30 拆解清单
1. `pi-coding-agent/eval/*.rs` + `pi-coding-agent/conformance*.rs` → `pi-evals/src/`
2. 新增 `pi-evals/src/pi_harness.rs`(对应 11K upstream)
3. 新增 `pi-evals/src/providers.eval.rs`(对应 14K upstream)
4. 新增 `pi-evals/src/smoke.rs`(对应 upstream)

---

## 7. `@earendil-works/pi-protocol` (12 .ts, 1,447 LOC) ↔ `pi-protocol`

### 7.1 上游文件清单(全部缺失)

| 上游文件 | 大小 | 我们 Rust 归宿 |
|---------|------|----------------|
| `protocol/src/cbor/decoder.ts` | 5,761 | ❌ 缺失 |
| `protocol/src/cbor/encoder.ts` | 6,712 | ❌ 缺失 |
| `protocol/src/cbor/index.ts` | 241 | ❌ 缺失 |
| `protocol/src/cbor/options.ts` | 1,722 | ❌ 缺失 |
| `protocol/src/codec.ts` | 4,652 | ❌ 缺失 |
| `protocol/src/framing.ts` | 5,561 | ❌ 缺失 |
| `protocol/src/index.ts` | 483 | ⚠️ `pi-protocol/src/lib.rs` (12 LOC) |
| `protocol/src/protocol.ts` | 3,831 | ❌ 缺失 |

### 7.2 我们的 Rust 状态
- `crates/pi-protocol/src/`:1 个文件 (12 LOC 空 marker)
- 真正的 protocol 代码在 `pi-coding-agent/{rpc.rs, acp.rs, jsonrpc.rs, sdk.rs, validation_broker.rs}`

### 7.3 Round 24 拆解清单
1. `pi-coding-agent/rpc.rs` → `pi-protocol/src/rpc.rs`
2. `pi-coding-agent/acp.rs` → `pi-protocol/src/acp.rs`
3. `pi-coding-agent/jsonrpc.rs` → `pi-protocol/src/jsonrpc.rs`
4. `pi-coding-agent/sdk.rs` → `pi-protocol/src/sdk.rs`
5. `pi-coding-agent/validation_broker.rs` → `pi-protocol/src/validation_broker.rs`
6. 新增 `pi-protocol/src/cbor/`(对应 upstream `cbor/` 4 文件)
7. 新增 `pi-protocol/src/codec.rs` + `framing.rs` + `protocol.rs`

---

## 8. `@earendil-works/pi-server` (23 .ts, 3,051 LOC) ↔ `pi-server`

### 8.1 上游文件清单

| 上游文件 | 大小 | 我们 Rust 归宿 |
|---------|------|----------------|
| `server/src/server.ts` | 19,869 | ❌ 缺失 |
| `server/src/session-router.ts` | 11,959 | ❌ 缺失 |
| `server/src/testing/host.ts` | 6,356 | ❌ 缺失 |
| `server/src/testing/client.ts` | 5,557 | ❌ 缺失 |
| `server/src/transports/unix/listener.ts` | 13,675 | ❌ 缺失 |
| `server/src/types.ts` | 2,836 | ❌ 缺失 |
| `server/src/transports/unix/preset.ts` | 1,010 | ❌ 缺失 |
| `server/src/connection.ts` | 1,322 | ❌ 缺失 |
| `server/src/errors.ts` | 1,532 | ❌ 缺失 |
| `server/src/transports/unix/types.ts` | 610 | ❌ 缺失 |
| `server/src/transports/unix/address.ts` | 425 | ❌ 缺失 |
| `server/src/testing/server.ts` | 845 | ❌ 缺失 |
| `server/src/testing/index.ts` | 328 | ❌ 缺失 |
| `server/src/listener.ts` | 340 | ❌ 缺失 |
| `server/src/transports/unix/index.ts` | 224 | ❌ 缺失 |
| `server/src/index.ts` | 117 | ⚠️ `pi-server/src/lib.rs` (13 LOC) |

### 8.2 我们的 Rust 状态
- `crates/pi-server/src/`:1 个文件 (13 LOC 空 marker)
- 真正的 server 代码:几乎没有,我们没实现 server 端

### 8.3 Round 30 拆解清单
1. 新增 `pi-server/src/server.rs`(对应 19.9K upstream)
2. 新增 `pi-server/src/session_router.rs`(对应 12K upstream)
3. 新增 `pi-server/src/transports/unix.rs`(对应 13.7K upstream)
4. 新增 `pi-server/src/connection.rs` + `errors.rs` + `types.rs`

---

## 9. `@earendil-works/pi-session-backends` (27 .ts, 4,126 LOC) ↔ `pi-session-backends`

### 9.1 上游文件清单(子包结构)

| 上游文件 | 大小 | 我们 Rust 归宿 |
|---------|------|----------------|
| `session-backends/sqlite-node/src/index.ts` | ? | ⚠️ `pi-session-backends/src/lib.rs` (13 LOC) |
| `session-backends/sqlite-node/src/sqlite/repo.ts` | ? | ❌ 缺失 |
| `session-backends/sqlite-node/src/sqlite/migrations.ts` | ? | ❌ 缺失 |
| `session-backends/sqlite-node/src/sqlite/migrations/*.ts` | ? | ❌ 缺失 |
| `session-backends/sqlite-node/src/sqlite/session.ts` | ? | ⚠️ `pi-coding-agent/session.rs` |
| `session-backends/sqlite-node/src/sqlite/storage.ts` | ? | ⚠️ `pi-coding-agent/session_sqlite.rs` |
| `session-backends/sqlite-node/src/sqlite/sql.ts` | ? | ❌ 缺失 |
| `session-backends/sqlite-node/src/sqlite/types.ts` | ? | ❌ 缺失 |
| `session-backends/sqlite-node/src/session/index.ts` | ? | ⚠️ `pi-coding-agent/session*.rs` |

### 9.2 我们的 Rust 状态
- `crates/pi-session-backends/src/`:1 个文件 (13 LOC 空 marker)
- 真正的 session 代码:全部在 `pi-coding-agent/{session.rs, session_sqlite.rs, session_store_v2.rs, session_picker.rs, session_import.rs, session_index.rs, session_test.rs, compaction_snap.rs}`

### 9.3 Round 26 拆解清单
1. `pi-coding-agent/session*.rs` (8 文件) → `pi-session-backends/src/`
2. `pi-coding-agent/compaction_snap.rs` → `pi-session-backends/src/`
3. 新增 `pi-session-backends/src/sqlite/{repo, migrations, sql, types}.rs`
4. 新增 `pi-session-backends/src/session/index.rs`
5. 抽象 `session::EventSource` trait 解 `pi-coding-agent ↔ pi-session-backends` 循环

---

## 10. `@earendil-works/pi-telemetry` (8 .ts, 1,178 LOC) ↔ `pi-telemetry`

### 10.1 上游文件清单(基本覆盖)

| 上游文件 | 大小 | 我们 Rust 归宿 |
|---------|------|----------------|
| `telemetry/src/index.ts` | 13,363 | ⚠️ `pi-telemetry/src/lib.rs` (12 LOC,空 marker) |
| `telemetry/src/memory.ts` | 6,067 | ❌ 缺失 |
| `telemetry/src/noop.ts` | 645 | ❌ 缺失 |
| `telemetry/src/testing/conformance.ts` | 10,631 | ❌ 缺失 |
| `telemetry/src/testing/types.ts` | 743 | ❌ 缺失 |
| `telemetry/src/testing/index.ts` | 198 | ❌ 缺失 |

### 10.2 我们的 Rust 状态
- `crates/pi-telemetry/src/`:4 个文件 (`lib.rs`, `pmu_telemetry.rs`, `profiler.rs`, `session_metrics.rs`)
- 真实 telemetry 实现基本完整

### 10.3 Round 30 拆解清单
1. `pi-coding-agent/extension_events.rs` → `pi-telemetry/src/events.rs`(如果存在)
2. 新增 `pi-telemetry/src/memory.rs` + `noop.rs`
3. 新增 `pi-telemetry/src/testing/`

---

## 11. `@earendil-works/pi-tui` (87 .ts, 36,724 LOC) ↔ `pi-tui`

### 11.1 上游文件清单(分类)

#### 11.1.1 `tui/src/components/` (~40 文件,最大)
| 上游文件 | 大小 | 我们 Rust 归宿 |
|---------|------|----------------|
| `components/editor.ts` | 83,829 | ❌ 缺失(超大) |
| `components/markdown.ts` | 33,109 | ❌ 缺失 |
| `components/input.ts` | 15,230 | ❌ 缺失 |
| `components/scroll-view.ts` | 8,168 | ❌ 缺失 |
| `components/select-list.ts` | 9,385 | ❌ 缺失 |
| `components/settings-list.ts` | 10,756 | ❌ 缺失 |
| `components/stack.ts` | 5,216 | ❌ 缺失 |
| `components/text.ts` | 3,349 | ❌ 缺失 |
| `components/box.ts` | 4,475 | ❌ 缺失 |
| `components/image.ts` | 3,658 | ❌ 缺失 |
| `components/truncated-text.ts` | 1,823 | ❌ 缺失 |
| `components/v-stack.ts` | 1,178 | ❌ 缺失 |
| `components/h-stack.ts` | 1,801 | ❌ 缺失 |
| `components/spacer.ts` | 510 | ❌ 缺失 |
| `components/mouse-region.ts` | 902 | ❌ 缺失 |
| `components/loader.ts` | 2,798 | ❌ 缺失 |
| `components/cancellable-loader.ts` | 1,000 | ❌ 缺失 |
| `components/alt-screen-flash.ts` | 1,289 | ❌ 缺失 |

#### 11.1.2 `tui/src/` 顶层 (~70 文件)
| 上游文件 | 大小 | 我们 Rust 归宿 |
|---------|------|----------------|
| `keys.ts` | 44,954 | ❌ 缺失 |
| `latex.ts` | 33,287 | ❌ 缺失 |
| `autocomplete.ts` | 24,441 | ⚠️ `pi-coding-agent/autocomplete.rs` |
| `layout.ts` | 15,758 | ❌ 缺失 |
| `alt-screen-search.ts` | 11,087 | ❌ 缺失 |
| `keybindings.ts` | 10,125 | ⚠️ `pi-coding-agent/keybindings.rs` |
| `editor-component.ts` | 2,598 | ❌ 缺失 |
| `index.ts` | 4,386 | ⚠️ `pi-tui/src/lib.rs` (12 LOC) |
| `layout-node.ts` | 1,364 | ❌ 缺失 |
| `fuzzy.ts` | 3,276 | ❌ 缺失 |
| `kill-ring.ts` | 1,295 | ❌ 缺失 |
| `terminal.ts` | ? | ⚠️ `pi-coding-agent/interactive.rs` |
| `terminal-colors.ts` | ? | ❌ 缺失 |
| `terminal-image.ts` | ? | ⚠️ `pi-coding-agent/terminal_images.rs` |
| `tui-alt-screen.ts` | ? | ❌ 缺失 |
| `stdin-buffer.ts` | ? | ❌ 缺失 |
| `native-modifiers.ts` | ? | ❌ 缺失 |
| `native-platform.ts` | ? | ❌ 缺失 |
| `native-module-path.ts` | ? | ❌ 缺失 |
| 其他 | ? | ❌ 缺失 |

### 11.2 我们的 Rust 状态
- `crates/pi-tui/src/`:1 个文件 (12 LOC 空 marker)
- 真正的 TUI 代码:`pi-coding-agent/{tui.rs, interactive.rs, interactive_ftui.rs, autocomplete.rs, terminal_images.rs}`
- 大量 TUI 组件缺失(我们用 `bubbletea-rs` + `ftui-rs` 而不是自己实现)

### 11.3 Round 28 拆解清单
1. `pi-coding-agent/tui.rs` → `pi-tui/src/tui.rs`
2. `pi-coding-agent/interactive.rs` + `interactive_ftui.rs` → `pi-tui/src/interactive/`
3. `pi-coding-agent/autocomplete.rs` → `pi-tui/src/autocomplete.rs`
4. `pi-coding-agent/terminal_images.rs` → `pi-tui/src/terminal_image.rs`
5. 新增 `pi-tui/src/components/`(对应 upstream `components/`,40 文件)
6. 新增 `pi-tui/src/keys.rs`(对应 45K upstream)
7. 新增 `pi-tui/src/latex.rs`(对应 33K upstream)
8. 新增 `pi-tui/src/editor.rs`(对应 84K upstream)
9. 新增 `pi-tui/src/markdown.rs`(对应 33K upstream)
10. 抽象 `EventSource` trait 让 TUI 不依赖具体 agent

---

## 12. 总览统计

### 12.1 文件级覆盖度(粗略估计)

| Package | 上游文件数 | 我们对应文件数 | 覆盖度 |
|---------|-----------|----------------|--------|
| `pi-agent-core` | 180 | 3 (~10 含 harness 子目录) | ⚠️ **5%** |
| `pi-ai` | 340 | 16 | ⚠️ **5%**(api 层 + providers 缺失) |
| `pi-chord` | 34 | 1 | ❌ **3%** |
| `pi-client` | 13 | 1 | ❌ **8%** |
| `pi-coding-agent` | 646 | 127 + 10 subdirs(~50) | ⚠️ **20%** |
| `pi-evals` | 17 | 1 | ❌ **6%** |
| `pi-protocol` | 12 | 1 | ❌ **8%** |
| `pi-server` | 23 | 1 | ❌ **4%** |
| `pi-session-backends` | 27 | 1 | ❌ **4%** |
| `pi-telemetry` | 8 | 4 | ⚠️ **50%** |
| `pi-tui` | 87 | 1 | ❌ **1%** |
| **合计** | **1,387** | **~157** | **⚠️ ~11%** |

### 12.2 按"功能模块"加权后的复刻度

考虑每个文件的相对重要性(以 LOC 衡量):

- `pi-coding-agent` 接收 binary 主体,~85% 覆盖
- `pi-ai` ~15% 覆盖(只有 providers 13 个)
- `pi-tui` ~5% 覆盖(只在 coding-agent)
- `pi-chord` ~5% 覆盖(只在 extensions_api.rs)
- 其他:几乎 0

**加权后真实复刻度:~30%**(Round 23 估算)

### 12.3 Round 30 目标

| Package | 当前 | 目标 |
|---------|------|------|
| `pi-agent-core` | 5% | 25% |
| `pi-ai` | 5% | 40% |
| `pi-chord` | 3% | 50% |
| `pi-client` | 8% | 60% |
| `pi-coding-agent` | 20% | 35% |
| `pi-evals` | 6% | 40% |
| `pi-protocol` | 8% | 70% |
| `pi-server` | 4% | 30% |
| `pi-session-backends` | 4% | 60% |
| `pi-telemetry` | 50% | 70% |
| `pi-tui` | 1% | 30% |
| **加权复刻度** | **~30%** | **~50%** |

---

## 13. Round 24-30 执行排序(更新版)

### Round 24 — 拆 `pi-protocol`(+8% 文件覆盖)
- 5 个文件迁出:`rpc / acp / jsonrpc / sdk / validation_broker`
- 7 个新文件:`cbor/* + codec / framing / protocol`

### Round 25 — 拆 `pi-client`(+6%)
- 2 个文件迁出:`web_remote / connectors/`
- 2 个新文件:`transport / unix`

### Round 26 — 拆 `pi-session-backends`(+8%)
- 8 个文件迁出:`session* + compaction_snap`
- 4 个新文件:`sqlite/{repo, migrations, sql, types}`

### Round 27 — 拆 `pi-chord`(最大,+15%)
- ~12 个文件迁出:`extensions_api + extensions/* + hostcall_* + buffer_shim + file_lock`
- 4 个新文件:`bundler / context / delta / facets/{host, loader}`

### Round 28 — 拆 `pi-tui`(+10%)
- 5 个文件迁出:`tui / interactive / interactive_ftui / autocomplete / terminal_images`
- ~10 个新文件:`components/ + keys + latex + editor + markdown`

### Round 29 — provider + api 全量补齐(+10%)
- `pi-ai/src/api/`:7 个新文件(anthropic / openai / bedrock / google / mistral / pi-messages / constrained-sampling)
- `pi-ai/src/providers/`:20+ 个新 provider 实现
- `pi-ai/src/auth/`:5 个新文件

### Round 30 — agent / evals / server / telemetry 收尾(+5%)
- `pi-agent-core`:agent / agent_cx / agent_hub / handoff / subagents 迁出 + 新增 hooks / checkpoint
- `pi-evals`:eval / conformance 迁出
- `pi-server`:基础 server / session-router / transports
- `pi-telemetry`:补 memory / noop / testing

---

## 14. 风险与依赖图

### 14.1 Round 24-26 低/中风险,可直接执行
- `pi-protocol / pi-client / pi-session-backends` 都是相对独立的子域

### 14.2 Round 27-28 中风险,需要先做 trait 抽象
- `pi-chord`:extensions 与 chord services 强耦合,需要先设计 `chord::Facets` trait
- `pi-tui`:agent event 流依赖强,需要先设计 `EventSource` trait

### 14.3 Round 29 低风险,纯加法
- provider 实现独立,可以并行

### 14.4 Round 30 高风险,需要重新设计依赖图
- agent 代码是核心,影响最大

---

## 15. 完成度轨迹

```
Round 23 (基线)        : ~30% 复刻度
Round 24 后(拆 protocol): ~30.5%
Round 25 后(拆 client)  : ~31%
Round 26 后(拆 session): ~32%
Round 27 后(拆 chord)   : ~39%
Round 28 后(拆 tui)     : ~49%
Round 29 后(provider)   : ~59%
Round 30 后(收尾)       : ~64%
```

---

## Round 26 执行记录

### Round 26.1 — `compaction_snap.rs` → `pi-session-backends` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/compaction_snap.rs crates/pi-session-backends/src/compaction_snap.rs`
2. `pi-session-backends/Cargo.toml` 新增 deps:`pi-ai / base64 / flate2 / serde / serde_json / tracing / anyhow`
3. `pi-session-backends/src/lib.rs` 新增 `pub mod compaction_snap;`
4. `pi-coding-agent/Cargo.toml` 新增 `pi-session-backends = { workspace = true }` 依赖
5. `pi-coding-agent/src/lib.rs` 移除 `pub mod compaction_snap;`(文件已迁出)
6. 8 处 `crate::compaction_snap::*` 替换为 `pi_session_backends::compaction_snap::*`:
   - `pi-coding-agent/src/session.rs:5717,5719`
   - `pi-coding-agent/src/agent.rs:2180`
   - `pi-coding-agent/src/rpc.rs:3408,10866`
   - `pi-coding-agent/src/compaction.rs:189,2142,2151`

**附带修复:**
- `crates/pi-ai/build.rs:14` 与 `crates/pi-ai/src/embedded_assets.rs:424,430` 旧路径 `legacy_pi_mono_code/pi-mono/...` → `legacy_pi_mono_code/pi/...`(Round 22 重命名未同步)

**验证:**
- `cargo check -p pi-session-backends`:✅ 通过
- `cargo check -p pi-coding-agent`:✅ 通过(195 warnings,0 errors,与 Round 20 基线持平)
- `cargo check -p pi-coding-agent --bin pi`:✅ 通过

**LOC 迁移:** 579 LOC(纯代码)

**留下的工作:** Round 26 剩余 7 个文件待迁出(下一轮执行)

### Round 26.2 — `session_test.rs` → `pi-session-backends/tests/` ⚠️ 部分完成

**做了什么:**
1. `git mv crates/pi-coding-agent/src/session_test.rs crates/pi-session-backends/tests/session_persistence.rs`(56 LOC,集成测试)
2. `pi-session-backends/Cargo.toml` 新增 `[dev-dependencies]`:asupersync / pi-coding-agent / tempfile
3. 把 `use crate::session::Session` 改为 `use pi_coding_agent::session::{Session, SessionEntry, SessionMessage}`
4. 修复 `timestamp: 0` → `timestamp: Some(0)`(上游 `SessionMessage::User` 字段类型变更后未同步)

**`session_import.rs` 暂未迁出:**
- 它用 `use crate::session::Session`,如果迁入 `pi-session-backends/src/` 会引入
  `pi-coding-agent ↔ pi-session-backends` 循环依赖(Round 18 已通过把 session 移出
  pi-session-backends 打破该循环)
- **决策**:Round 26.2 先放过 `session_import.rs`,等设计 `Session` trait seam 后再迁
- 已 `git mv` 回原位置,文件未变化

**为什么放在 `tests/` 而不是 `src/`:**
- `src/` 内的 `pub mod` 会强制 `pi-session-backends → pi-coding-agent` 编译期依赖
- `tests/` 是外部集成测试,只在 `cargo test` 时拉 `pi-coding-agent`,不影响 lib 编译
- 这是把 Round 18 已拆开的循环依赖继续保持拆开的标准做法

**验证:**
- `cargo check -p pi-session-backends`:✅ Finished
- `cargo check -p pi-session-backends --tests`:✅ Finished
- `cargo check -p pi-coding-agent`:✅ Finished(189 warnings,比 Round 26.1 少 6 个)
- `cargo check -p pi-coding-agent --bin pi`:✅ Finished

**LOC 迁移:** 56 LOC(纯测试)

### Round 26.3 — pivot to Round 27 (pi-chord) ✅

**Round 26 后续被 cycle 阻断,本轮切换到 Round 27 (pi-chord)**

**Round 26 剩余文件为什么暂时无法迁出:**
- `session_picker.rs` / `session_sqlite.rs` / `session_index.rs` / `session_store_v2.rs` / `session.rs` / `session_import.rs` 全部 `use crate::session::*`
- 如果迁入 `pi-session-backends/src/`,需要 `pi-session-backends → pi-coding-agent` 编译期依赖
- 这会和 Round 26.1 加的 `pi-coding-agent → pi-session-backends` 重新形成 cycle
- 真正的修复需要先把 `Session` 类型抽成 `SessionLike` trait,这是一个大重构,留待 Round 31+

**本轮做的事(切到 Round 27):**
1. `git mv crates/pi-coding-agent/src/buffer_shim.rs crates/pi-chord/src/buffer_shim.rs`
2. `git mv crates/pi-coding-agent/src/file_lock.rs crates/pi-chord/src/file_lock.rs`
3. `pi-coding-agent/Cargo.toml` 新增 `pi-chord = { workspace = true }` 依赖
4. `pi-chord/Cargo.toml` 新增 `filetime` / `rustix` 生产依赖 + `tempfile` 开发依赖
5. `pi-coding-agent/src/lib.rs` 移除 `pub mod buffer_shim;` 和 `pub mod file_lock;`
6. 替换 5 个文件中的 `crate::file_lock::*` → `pi_chord::file_lock::*`(bulk sed):
   - `pi-coding-agent/src/session_index.rs`(注释 + 2 处使用)
   - `pi-coding-agent/src/auth.rs`(15 处使用)
   - `pi-coding-agent/src/providers/model_fetch.rs`
   - `pi-coding-agent/src/config.rs`
   - `pi-coding-agent/src/mcp/trust.rs`
7. 替换 `pi-coding-agent/src/extensions_js.rs:14334` 的 `crate::buffer_shim::NODE_BUFFER_JS` → `pi_chord::buffer_shim::NODE_BUFFER_JS`

**验证:**
- `cargo check -p pi-chord`:✅ Finished
- `cargo check -p pi-chord --tests`:✅ Finished
- `cargo check -p pi-coding-agent`:✅ Finished(189 warnings,与 R26.2 持平)
- `cargo check -p pi-coding-agent --bin pi`:✅ Finished
- `cargo check -p pi-session-backends --tests`:✅ Finished

**LOC 迁移:** ~1100 LOC(buffer_shim ~440 LOC + file_lock ~660 LOC,文件级)

**Round 26 → Round 27 战略决策:**
- 原计划 Round 26 拆 `pi-session-backends`(8 文件,32K LOC)
- 但 Round 18 cycle-break 让大部分 session 文件形成依赖耦合
- 现实决策:Round 26 暂留 6 个 session 文件,Round 27 切到 `pi-chord`(本轮)
- Round 27 价值:迁出 `buffer_shim + file_lock` 这两个零依赖 leaf 文件,验证 Round 18 的"反向修复"路径可行
- Round 27.2+ 候选:`hostcall_*` 系列(8 文件,中等依赖),`extensions_api.rs`(25K,最大 chord 资产)

### Round 27.2 — `hostcall_rewrite.rs` → `pi-chord` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/hostcall_rewrite.rs crates/pi-chord/src/hostcall_rewrite.rs`(386 LOC,**零依赖**)
2. `pi-chord/src/lib.rs` 新增 `pub mod hostcall_rewrite;`
3. `pi-coding-agent/src/lib.rs` 移除 `pub mod hostcall_rewrite;`
4. Bulk rename `crate::hostcall_rewrite::*` → `pi_chord::hostcall_rewrite::*`:
   - `pi-coding-agent/src/hostcall_egraph.rs`(3 处使用 + 2 处 doc comment)
   - `pi-coding-agent/src/extensions_api.rs`(1 处使用)

**验证:**
- `cargo check -p pi-chord`:✅ Finished
- `cargo check -p pi-coding-agent`:✅ Finished(189 warnings,与 R27.1 持平)
- `cargo check -p pi-coding-agent --bin pi`:✅ Finished(隐含)

**LOC 迁移:** 386 LOC

**剩余 Round 27 chord 文件:**
| 文件 | LOC | 依赖 |
|------|-----|------|
| `hostcall_superinstructions.rs` | 859 | 待评估 |
| `hostcall_io_uring_lane.rs` | 1,085 | 待评估 |
| `hostcall_trace_jit.rs` | 1,364 | 待评估 |
| `hostcall_amac.rs` | 1,460 | 待评估 |
| `hostcall_s3_fifo.rs` | 1,288 | 待评估 |
| `hostcall_queue.rs` | 2,180 | 待评估 |
| `hostcall_egraph.rs` | 2,450 | 依赖 hostcall_rewrite (已迁) |
| `extensions_api.rs` | ~25,000 | 中心枢纽,需最后迁 |

---

> 本文档版本:v2.4(2026-09-13)
> 与 Multica issue `01a08d97` 绑定,分支 `feature/crates0911`
> 参考:`legacy_pi_mono_code/pi/packages/*/src/`(earendil-works/pi 快照,2026-09-13)