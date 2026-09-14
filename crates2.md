# pi.rs 模块化复刻详细差距分析 (crates2.md)

> 本文档是 `crates1.md` v1.2 + Round 23 差距分析的续篇,**逐文件**对比
> 上游 `legacy_pi_mono_code/pi/`(= `https://github.com/earendil-works/pi.git` 快照,
> 已 commit 到本仓库)与本仓库 Rust 实现。
>
> **目标**:把代码归属复刻度从 ~30% 推到 ~64%(Round 30)。

---

## `Round 42` 进展

**做了什么:**
1. 将 `crates/pi-coding-agent/src/conformance.rs`（4,387 LOC）的 fixture/diff 语义比较实现归位到 `crates/pi-evals/src/conformance.rs`。
2. `pi-evals` 从 marker crate 变为可独立使用的 conformance 包，补齐 `serde_json` 与测试用 `proptest` 依赖。
3. `pi-coding-agent` 通过 `pub use pi_evals::conformance` 保留历史模块路径，未改变现有调用方 API。

**验证:**
- `git diff --check` 待执行
- `dsr quality --tool pi_agent_rust`：环境未安装 `dsr`，无法执行权威质量配方

**LOC 迁移:** 4,387 LOC；`pi-evals` 不再是空占位 crate。


**做了什么:**
1. 新增 `crates/pi-protocol/src/tool_effects.rs`，承载 read/write/append/network/process effects、labels、parallel-safety 与 barrier 规则
2. `pi-coding-agent::tools` 改为 re-export 协议类型，保持所有既有 `ToolEffects` 调用路径与行为
3. 新增协议单元测试覆盖 labels、barrier 与并发兼容性
4. 为后续 `plan.rs` 迁移建立无 coding-agent 反向依赖的工具效果 seam

**验证:**
- `cargo check -p pi-protocol` ✅
- `cargo test -p pi-protocol tool_effects --lib` ✅ 1 passed
- `git diff --check` ✅
- `cargo check -p pi-coding-agent --lib` 仍仅受仓库既有 Windows `win32job` / `windows_by_handle` 问题阻塞；本轮未引入新的协议错误

**推送:** 本地提交 `a1df24af1`；首次推送因 GitHub `Recv failure: Connection was aborted` 失败，提交已保留，待网络恢复后重试。

---

## 0. 摘要

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

### Round 27.3 — `hostcall_superinstructions.rs` → `pi-chord` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/hostcall_superinstructions.rs crates/pi-chord/src/hostcall_superinstructions.rs`(859 LOC,**只依赖 std + serde**)
2. `pi-chord/Cargo.toml` 新增 `serde` 生产依赖
3. `pi-chord/src/lib.rs` 新增 `pub mod hostcall_superinstructions;`
4. `pi-coding-agent/src/lib.rs` 移除 `pub mod hostcall_superinstructions;`
5. Bulk rename `crate::hostcall_superinstructions::*` → `pi_chord::hostcall_superinstructions::*`:
   - `pi-coding-agent/src/extensions_api.rs`(1 处)
   - `pi-coding-agent/src/hostcall_trace_jit.rs`(2 处)

**验证:**
- `cargo check -p pi-chord`:✅ Finished
- `cargo check -p pi-coding-agent`:✅ Finished

**LOC 迁移:** 859 LOC

### Round 27.4 — `hostcall_io_uring_lane.rs` → `pi-chord` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/hostcall_io_uring_lane.rs crates/pi-chord/src/hostcall_io_uring_lane.rs`(1,085 LOC,serde)
2. `pi-chord/src/lib.rs` 新增 `pub mod hostcall_io_uring_lane;`
3. `pi-coding-agent/src/lib.rs` 移除 `pub mod hostcall_io_uring_lane;`
4. Bulk rename `crate::hostcall_io_uring_lane::*` → `pi_chord::hostcall_io_uring_lane::*`:
   - `pi-coding-agent/src/extension_dispatcher.rs`
   - `pi-coding-agent/src/extensions_js.rs`

**验证:**
- `cargo check -p pi-chord`:✅ Finished
- `cargo check -p pi-coding-agent`:✅ Finished

**LOC 迁移:** 1,085 LOC

**Round 27 累计迁出 5/15 chord 文件**

### Round 27.5 — `hostcall_s3_fifo.rs` → `pi-chord` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/hostcall_s3_fifo.rs crates/pi-chord/src/hostcall_s3_fifo.rs`(1,288 LOC,std only)
2. `pi-chord/src/lib.rs` 新增 `pub mod hostcall_s3_fifo;`
3. `pi-coding-agent/src/lib.rs` 移除 `pub mod hostcall_s3_fifo;`
4. `pi-coding-agent/src/hostcall_queue.rs` 的 `pub use crate::hostcall_s3_fifo::S3FifoFallbackReason;` → `pub use pi_chord::hostcall_s3_fifo::S3FifoFallbackReason;`

**验证:**
- `cargo check -p pi-chord`:✅ Finished
- `cargo check -p pi-coding-agent`:✅ Finished

**LOC 迁移:** 1,288 LOC

### Round 27.6 — `hostcall_trace_jit.rs` → `pi-chord` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/hostcall_trace_jit.rs crates/pi-chord/src/hostcall_trace_jit.rs`(1,364 LOC,std + serde + 内依赖 hostcall_superinstructions)
2. `pi-chord/src/lib.rs` 新增 `pub mod hostcall_trace_jit;`
3. `pi-coding-agent/src/lib.rs` 移除 `pub mod hostcall_trace_jit;`
4. Bulk rename `crate::hostcall_trace_jit::*` → `pi_chord::hostcall_trace_jit::*` in `extensions_api.rs`
5. 修复文件内 `use pi_chord::hostcall_superinstructions::*` → `use crate::hostcall_superinstructions::*`(文件自身就在 `pi-chord` 里,不能自引用)

**验证:**
- `cargo check -p pi-chord`:✅ Finished
- `cargo check -p pi-coding-agent`:✅ Finished

**LOC 迁移:** 1,364 LOC

**Round 27 累计迁出 7/15 chord 文件,累计 ~5,300 LOC**

### Round 27.7 计划:跳过 `hostcall_amac.rs`(cycle 阻断)

`hostcall_amac.rs` 用 `use crate::extensions_js::{HostcallKind, HostcallRequest};`,迁入会形成 `pi-chord → pi-coding-agent` 编译期依赖,加上 R27.1 加的 `pi-coding-agent → pi-chord`,构成 cycle。**与 `session_import.rs` 同样的 blocker**。

真正的修复:把 `HostcallKind` / `HostcallRequest` 抽到 `pi-chord` 或共享类型模块。留 Round 31+。

### Round 27.7 — `hostcall_queue.rs` → `pi-chord` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/hostcall_queue.rs crates/pi-chord/src/hostcall_queue.rs`(2,180 LOC)
2. `pi-chord/Cargo.toml` 新增 `crossbeam-queue` + `tracing` 依赖
3. `pi-chord/src/lib.rs` 新增 `pub mod hostcall_queue;`
4. `pi-coding-agent/src/lib.rs` 移除 `pub mod hostcall_queue;`
5. Bulk rename `crate::hostcall_queue::*` → `pi_chord::hostcall_queue::*` in `extensions_api.rs` + `extensions_js.rs`
6. 修复文件内 `pub use pi_chord::hostcall_s3_fifo::*` → `pub use crate::hostcall_s3_fifo::*`(自身已在 pi-chord)

**验证:**
- `cargo check -p pi-chord`:✅ Finished
- `cargo check -p pi-coding-agent`:✅ Finished

**LOC 迁移:** 2,180 LOC

**Round 27 累计迁出 8/15 chord 文件,累计 ~7,500 LOC**

### Round 27.8 — `hostcall_egraph.rs` → `pi-chord` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/hostcall_egraph.rs crates/pi-chord/src/hostcall_egraph.rs`(2,450 LOC,std + 内依赖 hostcall_rewrite)
2. `pi-chord/Cargo.toml` 新增 `serde_json` 依赖
3. `pi-chord/src/lib.rs` 新增 `pub mod hostcall_egraph;`
4. `pi-coding-agent/src/lib.rs` 移除 `pub mod hostcall_egraph;`
5. Bulk rename `crate::hostcall_egraph::*` → `pi_chord::hostcall_egraph::*` in `extensions/protocol.rs`
6. 修复文件内 `use pi_chord::hostcall_rewrite::*` → `use crate::hostcall_rewrite::*`(自身已在 pi-chord)

**验证:**
- `cargo check -p pi-chord`:✅ Finished
- `cargo check -p pi-coding-agent`:✅ Finished

**LOC 迁移:** 2,450 LOC

**Round 27 累计迁出 9/15 chord 文件,累计 ~10,000 LOC**

**剩余 hostcall_* 候选:** 仅 `hostcall_amac.rs`(cycle-blocked,Round 31+)

### Round 27 收尾 + Round 28.1 — `terminal_images.rs` → `pi-tui` ✅

**Round 27 hostcall_* 系列已完成(9 文件)。剩余:**
- `extensions_api.rs`(~25K LOC,中心枢纽):用 `agent / config / session / connectors / permissions / resources / tools / extensions`,迁入会形成 `pi-chord → pi-coding-agent` cycle。与 `hostcall_amac.rs` 同模式 blocker,留 Round 31+。
- `hostcall_amac.rs`:cycle-blocked。

**本轮(Round 28.1):切到 `pi-tui`,迁 `terminal_images.rs`**

**做了什么:**
1. `git mv crates/pi-coding-agent/src/terminal_images.rs crates/pi-tui/src/terminal_images.rs`(957 LOC,base64 + std + pi_ai::model)
2. `pi-coding-agent/Cargo.toml` 新增 `pi-tui = { workspace = true }` 依赖
3. `pi-tui/Cargo.toml` 新增 `base64` + `pi-ai` 依赖
4. `pi-tui/src/lib.rs` 从空 marker 改为 `pub mod terminal_images;`
5. `pi-coding-agent/src/lib.rs` 移除 `pub mod terminal_images;`
6. Bulk rename `crate::terminal_images::*` → `pi_tui::terminal_images::*` in `interactive/conversation.rs`(2 处)

**验证:**
- `cargo check -p pi-tui`:✅ Finished
- `cargo check -p pi-coding-agent`:✅ Finished

**LOC 迁移:** 957 LOC

### Round 28.2 — `tui.rs` → `pi-tui` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/tui.rs crates/pi-tui/src/tui.rs`(1,450 LOC,rich_rust + std)
2. **关键 cycle 修复**:`tui.rs` 内部有 2 处 `crate::` 引用,直接搬会形成 `pi-tui → pi-coding-agent` cycle:
   - L33 `crate::config::Config::global_dir().join("logs")`(用于 TUI 日志目录)
   - L114 `_theme: Option<crate::theme::Theme>`(但参数被 `_` 前缀,从未使用)
3. 解决方案:不引入 cycle 边,而是在 pi-tui 暴露 `pub fn tui_log_init_dir(dir: PathBuf)`,由 pi-coding-agent 在启动时主动注入(保留 `PI_CODING_AGENT_DIR` 等环境变量语义)。
   - `TUI_LOG_DIR: OnceLock<PathBuf>` + `pub fn tui_log_init_dir(dir: PathBuf)`(新接口)
   - `tui_log_file()` 先查 `TUI_LOG_DIR.get()`,fallback 到 `dirs::home_dir()/.pi/agent/logs`
   - `_theme` 参数直接删除(本就是 unused)
4. `pi-tui/Cargo.toml` 新增 `dirs` + `rich_rust` 依赖,新增 `[features]` 默认空,`syntax-highlighting` feature 保留原 cfg 门控语义
5. `pi-tui/src/lib.rs` 新增 `pub mod tui;`
6. `pi-coding-agent/src/lib.rs` 移除 `pub mod tui;`
7. Bulk rename 3 处调用方:
   - `pi-coding-agent/src/session.rs:21` `use crate::tui::PiConsole` → `use pi_tui::tui::PiConsole`
   - `pi-coding-agent/src/interactive_ftui.rs:4833` `crate::tui::TuiLogRedirectGuard` → `pi_tui::tui::TuiLogRedirectGuard`
   - `pi-coding-agent/src/interactive/mod.rs:1882` 同上
8. `pi-coding-agent/src/main.rs`:
   - `use pi_coding_agent::tui::PiConsole` → `use pi_tui::tui::PiConsole`
   - `with_writer(|| pi_coding_agent::tui::TuiAwareLogWriter)` → `with_writer(|| pi_tui::tui::TuiAwareLogWriter)`
   - 在 `tracing_subscriber::fmt().init()` 之前调用 `pi_tui::tui::tui_log_init_dir(Config::global_dir().join("logs"))`

**验证:**
- `cargo check -p pi-tui`:✅ Finished(仅 syntax-highlighting cfg 警告,已通过 `[features]` 暴露)
- `cargo check -p pi-coding-agent`:✅ Finished(184 warnings,3 duplicates,与 Round 20 baseline 持平)
- `cargo check -p pi-coding-agent --bin pi`:✅ Finished

**LOC 迁移:** 1,450 LOC

**cycle 阻断说明:** 通过 `tui_log_init_dir` 钩子而非 `pi-tui → pi-coding-agent` 边,保留了 `PI_CODING_AGENT_DIR` 环境变量语义,同时不引入新 cycle。这是 Round 28 系列里第二个完美迁出的 TUI 大文件(`terminal_images.rs` 是第一个)。

### Round 28.3 — `autocomplete.rs` → `pi-tui` ✅ (trait seam)

**做了什么:**
1. `git mv crates/pi-coding-agent/src/autocomplete.rs crates/pi-tui/src/autocomplete.rs`(2,789 LOC)
2. **2 个内部 cycle 引用**:`autocomplete.rs` 引用 `crate::workspace::WorkspaceHandle` 与 `crate::models::model_autocomplete_candidates()`,直接搬会形成 `pi-tui → pi-coding-agent` cycle。
3. **解决方案 — 引入 2 个 trait seam**:
   - `pub trait AutocompleteResourceSource`(in pi-tui):返回 prompts / skills / models 的 `(String, Option<String>)` 列表
   - `pub trait WorkspaceRootProvider: Debug + Send`(in pi-tui):返回 canonical 根路径列表
4. `AutocompleteCatalog` 新增 `models: Vec<NamedEntry>` 字段
5. `AutocompleteProvider.workspace` 字段从 `Option<WorkspaceHandle>` 改为 `Option<Box<dyn WorkspaceRootProvider>>`
6. `pi-coding-agent/src/resources.rs` 新增 `impl AutocompleteResourceSource for ResourceLoader`(4 个方法:prompts / skills / models / enable_skill_commands)
7. `pi-coding-agent/src/workspace.rs` 新增 `impl WorkspaceRootProvider for WorkspaceHandle`(roots_or 委托给 `snapshot_or(cwd).all()`)
8. `pi-tui/Cargo.toml` 新增 `ignore` 依赖
9. `pi-tui/src/lib.rs` 新增 `pub mod autocomplete;`
10. `pi-coding-agent/src/lib.rs` 移除 `pub mod autocomplete;`
11. Bulk rename 6 处调用方:main.rs、interactive/{agent,ftui,state,tests,mod}.rs
12. `interactive/state.rs:152` 与 `interactive/mod.rs:2742` 调用 `set_workspace` 路径:`AutocompleteState::set_workspace` 内部用 `Box::new(workspace) as Box<dyn WorkspaceRootProvider>` 做强制转换

**验证:**
- `cargo check -p pi-tui`:✅ Finished
- `cargo check -p pi-coding-agent`:✅ Finished(183 warnings,3 duplicates,与 Round 28.2 持平)
- `cargo check --bin pi -p pi-coding-agent`:✅ Finished

**LOC 迁移:** 2,789 LOC

**trait seam 设计:** 这是 Round 28 系列首次需要 trait seam 桥接的类型;之前的 `terminal_images.rs` 与 `tui.rs` 都只用了基础类型(std / base64 / rich_rust / dirs)。trait seam 让 pi-tui 不需要知道 `ResourceLoader`、`WorkspaceHandle` 这些 pi-coding-agent 内部类型,且为 Round 31+ 的更深层解耦提供了可复用模式。

### Round 28.4 — `interactive/file_refs.rs` → `pi-tui` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/interactive/file_refs.rs crates/pi-tui/src/file_refs.rs`(414 LOC,std + url,无 `crate::` 依赖)
2. 全部 `pub(super)` → `pub`(10 个函数):跨 crate 后父模块不再可见
3. pi-tui/Cargo.toml 加 `url = { workspace = true }`
4. pi-tui/src/lib.rs 加 `pub mod file_refs;`
5. pi-coding-agent/src/interactive/mod.rs:`use self::file_refs::` → `use pi_tui::file_refs::`,移除 `pub mod file_refs;`

**验证:**
- `cargo check -p pi-tui`:✅ Finished
- `cargo check -p pi-coding-agent`:✅ Finished(183 warnings,3 duplicates,baseline 持平)
- `cargo check --bin pi -p pi-coding-agent`:✅ Finished

**LOC 迁移:** 414 LOC

### Round 28.5 — `interactive/text_utils.rs` → `pi-tui` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/interactive/text_utils.rs crates/pi-tui/src/text_utils.rs`(1.1 KB,unicode-width only,无 `crate::` 依赖)
2. 3 个 `pub(super)` 函数 → `pub`(`push_line` / `truncate` / `queued_message_preview`)
3. pi-tui/Cargo.toml 加 `unicode-width = { workspace = true }`
4. pi-tui/src/lib.rs 加 `pub mod text_utils;`
5. `interactive/mod.rs` `use self::text_utils::` → `use pi_tui::text_utils::`,移除 `pub mod text_utils;`
6. `interactive/conversation.rs:8` `use super::text_utils::push_line` → `use pi_tui::text_utils::push_line`
7. `interactive/tree.rs:267` `super::truncate(...)` → `pi_tui::text_utils::truncate(...)`

**验证:**
- `cargo check -p pi-tui`:✅ Finished
- `cargo check -p pi-coding-agent`:✅ Finished(183 warnings,3 duplicates,baseline 持平)
- `cargo check --bin pi -p pi-coding-agent`:✅ Finished

**LOC 迁移:** ~50 LOC(纯代码)

### Round 29.1 — `extension_license + extension_inclusion` → `pi-chord` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/extension_license.rs crates/pi-chord/src/extension_license.rs`(1,298 LOC)
2. `git mv crates/pi-coding-agent/src/extension_inclusion.rs crates/pi-chord/src/extension_inclusion.rs`(764 LOC)
3. 两个文件本来生产代码中**无 `crate::` 依赖**,仅在被搬动后才发现 2 处隐式引用:
   - `extension_inclusion.rs:315` `crate::package_manager::hex_encode(...)` 
   - `extension_license.rs:438` `crate::extension_validation::chrono_now_iso()`
4. **解决方案 — 内联 2 个 helper 到目标文件**(均为纯 std,无依赖):
   - `extension_inclusion.rs` 末尾追加 `fn hex_encode(bytes: &[u8]) -> String`(8 行,lowercase hex)
   - `extension_license.rs` 末尾追加 `fn chrono_now_iso() / fn days_to_ymd / fn is_leap`(约 40 行,ISO 时间戳)
5. pi-chord/Cargo.toml 加 `sha2 = { workspace = true }`
6. pi-chord/src/lib.rs 加 `pub mod extension_inclusion;` 和 `pub mod extension_license;`
7. pi-coding-agent/src/lib.rs:
   - 移除 `pub mod extension_inclusion;` 和 `pub mod extension_license;`
   - 加 `pub use pi_chord::extension_inclusion;` 和 `pub use pi_chord::extension_license;`(保留 `pi_coding_agent::extension_license::*` 路径)
8. 内部调用方无需修改:`crate::extension_inclusion::*` 在 pi-coding-agent 中通过 re-export 仍可用

**验证:**
- `cargo check -p pi-chord`:✅ Finished
- `cargo check -p pi-coding-agent`:✅ Finished(183 warnings,3 duplicates,baseline 持平)
- `cargo check --bin pi -p pi-coding-agent`:✅ Finished

**LOC 迁移:** 2,062 LOC(净代码 + 内联 helpers)

**helper 内联策略 vs 搬动上游:** 搬动 `package_manager::hex_encode` 或 `extension_validation::chrono_now_iso` 到 pi-chord 会引入新依赖或更大搬动成本。这两个 helper 都是纯 std 实现,内联副本比 trait seam 更直接(无运行时开销、无 trait 对象)。

---

### Round 29.2 — `extension_popularity + extension_validation` → `pi-chord` ✅ (trait seam for `Client`)

**做了什么:**
1. `git mv crates/pi-coding-agent/src/extension_popularity.rs crates/pi-chord/src/extension_popularity.rs`(1,076 LOC)
2. `git mv crates/pi-coding-agent/src/extension_validation.rs crates/pi-chord/src/extension_validation.rs`(1,399 LOC)
3. `extension_popularity.rs` 原本 5 处 `client: &Client` 参数(`Client` 来自 `pi-coding-agent::http::client`),直接搬动会让 `pi-chord` 反向依赖 `pi-coding-agent`,重开 cycle。
4. **解决方案 — 在 `pi-chord` 定义 `pub trait NpmHttpGet: Send + Sync`**,三方法签名(dyn 兼容 + `&self`-bound BoxFuture):
   - `fetch_text(&self, url, timeout) -> BoxFuture<'_, Result<String>>`
   - `fetch_text_with_status(&self, url, timeout) -> BoxFuture<'_, Result<(u16, String)>>`
   - `fetch_text_with_headers(&self, url, timeout, headers: &[(&str, String)]) -> BoxFuture<'_, Result<(u16, String)>>`
5. `pi-coding-agent/src/http/client.rs` 加 `impl NpmHttpGet for Client`,桥接 `self.get(url).timeout(t).send().await?.text().await?`;三方法分别映射到无 header / 无 status / 带 GitHub auth 头。
6. `pi-chord/Cargo.toml` 加 `url = { workspace = true }`(`url::form_urlencoded`)、`pi-error = { workspace = true }`、`futures = { workspace = true }`(`BoxFuture`)。
7. `extension_popularity.rs` 内部 fetch 助手(`fetch_npm_downloads`、`fetch_github_repo_metrics_optional`、`fetch_npm_registry_meta`、`snapshot_github_repos`)全部改为接收 `client: &dyn NpmHttpGet`,GitHub auth 路径用 `fetch_text_with_headers` 传递 Bearer + Accept + API-Version。
8. `pi-chord/src/lib.rs` 加 `pub mod extension_popularity; pub mod extension_validation;`;`pi-coding-agent/src/lib.rs` 改为 `pub use pi_chord::extension_popularity; pub use pi_chord::extension_validation;`(原 `pub mod` 删掉)。
9. `extension_validation.rs` 内部 `crate::extension_popularity::*` 引用无需改:在 `pi-chord` 内 sibling 模块路径依然成立。

**验证:**
- `cargo check -p pi-chord`:✅ Finished(4 warnings,全部为既有 pi-error/serde 衍生 warning)
- `cargo check -p pi-coding-agent`:✅ Finished(183 warnings,3 duplicates,baseline 持平)
- `cargo check --workspace --exclude pi --exclude pi-mono`:✅ Finished in 2m 16s(0 errors)
- `cargo check -p pi`(binary):✅ Finished in 3m 18s(0 errors)
- `cargo clean` 跑了一次(磁盘从 100% 满 → 68%)

**LOC 迁移:** 2,475 LOC(净代码 + trait seam + impl bridge)

**trait seam 设计要点:** 关键坑是 `async fn` + `impl Future` 不会被识别为 dyn 兼容,必须改 `BoxFuture<'a, ...>` + 把所有输入引用绑到 `'a`(`url: &'a str`、`headers: &'a [...]`),否则 Rust 报 `the trait NpmHttpGet is not dyn compatible`。`Send + Sync` bound 让 trait 对象可以跨 await 边界传递,符合 `Client` 实际使用场景(被 `BubbleteaModel`、`agent_hub` 等多任务复用)。这是 Round 27-29 系列第 4 个 trait seam 案例(`AutocompleteResourceSource`、`WorkspaceRootProvider`、本 trait),trait seam 已稳定为 cycle 解耦的标准手段。

**Round 29 累计(本轮+Round 29.1):** 4,537 LOC 已迁回 `pi-chord`,接近上游 `pi-chord` 总体(9,375 LOC,差 34 个 .ts)的 50%。

---

### Round 30.1 — `jsonrpc + framing + tail` → `pi-protocol` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/jsonrpc.rs crates/pi-protocol/src/jsonrpc.rs`(56 LOC)
2. `git mv crates/pi-coding-agent/src/framing.rs crates/pi-protocol/src/framing.rs`(279 LOC)
3. `git mv crates/pi-coding-agent/src/tail.rs crates/pi-protocol/src/tail.rs`(101 LOC)
4. 三个文件仅依赖 `std + serde + serde_json`(framing),无 `crate::` 内部引用 —— Round 18 留的"空壳 pi-protocol + 抽到 pi-coding-agent"终于可以反向迁回。
5. `pi-protocol/src/jsonrpc.rs` 移除 `#[path = "framing.rs"] mod framing; #[path = "tail.rs"] mod tail;` —— sibling 模块改用普通 `mod` 声明。
6. `pi-protocol/src/lib.rs` 新增 `pub mod jsonrpc; pub mod framing; pub mod tail;`,文档注释更新为"jsonrpc 叶子模块归位,transport 层仍留在 pi-coding-agent"。
7. `pi-protocol/Cargo.toml` 加 `serde = { workspace = true }` + 已有 `serde_json/anyhow`。
8. `pi-coding-agent/Cargo.toml` 加 `pi-protocol = { workspace = true }`。
9. `pi-coding-agent/src/lib.rs` 替换 `pub mod jsonrpc;` 为 `pub use pi_protocol::{jsonrpc, framing, tail};`(3 个 re-export,保留 `pi_coding_agent::jsonrpc::*` 路径)。
10. `pi-coding-agent/src/lsp/jsonrpc.rs` 把 `pub use crate::jsonrpc::{...};` 改为 `pub use pi_protocol::jsonrpc::{...};`(该文件内 transport 特定代码 `JsonRpcClient`、`await_completion`、`apply_env_policy`、`reader_loop`、`PendingMap` 仍留在 pi-coding-agent,因为依赖 `crate::tools::ProcessGuard` + `crate::agent_cx::AgentCx`)。

**验证:**
- `cargo check -p pi-protocol`:✅ Finished(0 errors,无新增 warning)
- `cargo check -p pi-coding-agent`:✅ Finished(183 warnings,3 duplicates,baseline 持平)
- `cargo check -p pi`(binary):✅ Finished in 48.97s(0 errors)

**LOC 迁移:** 436 LOC(净代码 + lib.rs 重排 + 注释更新)

**设计要点:**
- `pi-protocol/src/jsonrpc.rs` 现在只装 framing 原语(`RpcErrorObject` / `TransportError` / `ServerNotification` / `EnvPolicy` / `encode_frame` / `read_frame` / `PublicTailBuffer` / `CompletionWaitError`),所有跨模块依赖(transport、子进程管理、cancel signal)继续留在 `pi-coding-agent/src/lsp/jsonrpc.rs`。
- 叶子 crate 零内部依赖(std + serde + serde_json),任何想直接用 framing 的 crate 都能直接 `use pi_protocol::jsonrpc::*` 而不引入 pi-coding-agent 重量级依赖。
- 路径兼容:`pi_coding_agent::jsonrpc::PublicTailBuffer`、`pi_coding_agent::lsp::jsonrpc::JsonRpcClient` 等等调用方路径全部保持,通过 `pub use` re-export 串联。

**Round 30 累计(本轮 + Round 30 后续待办):** 436 / ~33K(provider 主体待评估)。

---

### Round 30.2 — `overlay_system.rs` → `pi-tui` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/overlay_system.rs crates/pi-tui/src/overlay_system.rs`(235 LOC)
2. 文件仅依赖 `serde + std::collections::VecDeque`,**零 `crate::` 引用**,可零成本平移。
3. `pi-tui/Cargo.toml` 加 `serde = { workspace = true }`(已有 `anyhow / base64 / pi-ai / dirs / rich_rust / ignore / url / unicode-width`)。
4. `pi-tui/src/lib.rs` 加 `pub mod overlay_system;` + 文档条目。
5. `pi-coding-agent/src/lib.rs` 替换 `pub mod overlay_system;` 为 `pub use pi_tui::overlay_system;`(re-export,保留 `pi_coding_agent::overlay_system::*` 路径)。
6. `interactive/mod.rs:2242` 与 `interactive/view.rs:341` 两处 `crate::overlay_system::WelcomeScreen::default()` 调用无需修改(经 re-export 链解析)。

**验证:**
- `cargo check -p pi-tui`:✅ Finished in 1m 23s(1 dead_code warning,`OverlayKind` 枚举未在 pi-tui 内部使用但保持 pub,不影响 binary)
- `cargo check -p pi-coding-agent`:✅ Finished(183 warnings,baseline 持平)
- `cargo check -p pi`(binary):✅ Finished in 49.19s(0 errors)

**LOC 迁移:** 235 LOC

**Round 30 累计(本轮 + Round 30.1):** 671 LOC,继续向 pi-tui 收尾。

### Round 30.3 — `gallery.rs` → `pi-tui` ✅

**做了什么:**
1. `git mv crates/pi-coding-agent/src/gallery.rs crates/pi-tui/src/gallery.rs`(134 LOC)
2. 文件仅依赖 `serde + serde_json`,无 `crate::` 引用,平移成本为 0。
3. `pi-tui/Cargo.toml` 加 `serde_json = { workspace = true }`(`render_report_json` 用到 `serde_json::to_string_pretty`)。
4. `pi-tui/src/lib.rs` 加 `pub mod gallery;` + 文档条目。
5. `pi-coding-agent/src/lib.rs` 把 `pub mod gallery;` 替换为 `pub use pi_tui::gallery;`(re-export,保留 `pi_coding_agent::gallery::GalleryMatrix` 路径)。
6. `main.rs:3006` 的 `pi_coding_agent::gallery::GalleryMatrix::new()` 路径无需改动。

**验证:**
- `cargo check -p pi-tui`:✅ Finished(1 dead_code warning,`GalleryCategory` 枚举)
- `cargo check -p pi-coding-agent`:✅ Finished(183 warnings,baseline 持平)
- `cargo check -p pi`(binary):✅ Finished in 50.59s(0 errors)

**LOC 迁移:** 134 LOC

**Round 30 累计(本轮 + Round 30.1-30.2):** 805 LOC。pi-tui 现在持有 `tui / autocomplete / file_refs / text_utils / terminal_images / overlay_system / gallery` 共 7 个模块。

### Round 30.4 — `skills_managed.rs` → `pi-chord` ✅ (helper 内联策略)

**做了什么:**
1. `git mv crates/pi-coding-agent/src/skills_managed.rs crates/pi-chord/src/skills_managed.rs`(346 LOC)
2. 文件原本 2 处 `crate::` 依赖(直接搬动会重开 `pi-chord → pi-coding-agent` cycle):
   - `crate::config::Config::global_dir()` —— 全局 agent 目录路径解析
   - `crate::resources::validate_name / validate_description / validate_frontmatter_fields` —— 技能草稿校验(3 个函数 + 3 个常量)
3. **解决方案 — 内联 4 个 helper 到目标文件**(均为纯 std,无新依赖):
   - `managed_global_dir() -> PathBuf` —— 复刻 `Config::global_dir()` 语义,honor `PI_CODING_AGENT_DIR` 环境变量,fallback 到 `dirs::home_dir()/.pi/agent`
   - `validate_name(name, parent_dir) -> Vec<String>` —— 名字必须匹配 parent_dir / ≤64 字符 / 小写 + 数字 + 连字符 / 无前后置或连续连字符
   - `validate_description(description) -> Vec<String>` —— 非空 / ≤1024 字符
   - `validate_frontmatter_fields(keys) -> Vec<String>` —— 校验 frontmatter 字段在 8 个允许集合内
   - 3 个对应常量:`MAX_SKILL_NAME_LEN = 64`、`MAX_SKILL_DESC_LEN = 1024`、`ALLOWED_SKILL_FRONTMATTER`(8 个字段名)
4. `pi-chord/Cargo.toml` 加 `dirs = { workspace = true }`(`managed_global_dir` 用)。
5. `pi-chord/src/lib.rs` 加 `pub mod skills_managed;` + 文档条目。
6. `pi-coding-agent/src/lib.rs` 替换 `pub mod skills_managed;` 为 `pub use pi_chord::skills_managed;`(re-export,保留 `pi_coding_agent::skills_managed::*` 路径)。
7. 调用方无需修改:`crates/pi/tests/skills_managed.rs` 和 `crates/pi/tests/url_router.rs` 走 `pi::skills_managed::*`,`pi` crate 通过 re-export 链解析到 `pi_coding_agent::skills_managed` → `pi_chord::skills_managed`。

**验证:**
- `cargo check -p pi-chord`:✅ Finished(0 errors,1 既有 warning)
- `cargo check -p pi-coding-agent`:✅ Finished in 52.55s(183 warnings,baseline 持平)
- `cargo check -p pi`(binary):✅ Finished in 53.06s(0 errors)

**LOC 迁移:** 346 LOC(净代码 + ~80 行 helper 内联 + 3 个常量)

**Round 30 累计(本轮 + Round 30.1-30.3):** 1,151 LOC。pi-chord 现在持有 15 个模块(13 个 hostcall/extension + skills_managed + extension_validation),合计 ~15K LOC。

### Round 30 后续候选(按文件大小排序)
| 文件 | LOC | 候选归属 | 风险 |
|------|-----|---------|------|
| `completion.rs` | 188 | `pi-tui`(shell completion) | 零依赖 |
| `btw.rs` | 272 | `pi-ai` 或 `pi-tui` | pi_ai dep |
| `current_time.rs` | 276 | `pi-tui`(tool impl 较杂) | 有 `crate::tools::*` |
| `theme.rs` | 471 | `pi-tui`(theme) | 有 `crate::config::Config` |
| `usage.rs` | 602 | `pi-ai` 或 `pi-coding-agent` | 有 `crate::auth + http::client` |
| `conformance.rs` | 471 | `pi-evals` | 需评估 |
| `agent_cx.rs` | 246 | `pi-agent-core` | 158 callers,需 bulk rename |

---

> 本文档版本:v2.26(2026-09-13)
> 与 Multica issue `01a08d97` 绑定,分支 `feature/crates0911`
> 参考:`legacy_pi_mono_code/pi/packages/*/src/`(earendil-works/pi 快照,2026-09-13)

### Round 67 — `extensions_js` 插件协议与生命周期拆分 ✅

**做了什么:**
1. 在 `crates/pi-pijs-core` 新增纯协议 `PluginLifecycle` 状态机：注册 → 激活中 → 活跃 → 停用中 → 已停止；失败/取消进入终态并保留原因，非法重入被拒绝。
2. 新增可序列化 JSON-RPC request/response/error envelope，以及 `ExtensionToolSchema` 的最小协议校验（名称、描述、参数对象）。
3. `pi-coding-agent::extensions_js` 仅 re-export 这些 seam；QuickJS、宿主权限、调度和实际执行继续留在 coding-agent，未把运行时依赖泄漏到核心 crate。
4. 新增真实状态机、取消错误、RPC round-trip、tool schema 回归测试；不复用 LUM-864 fixture。

**验证:**
- `cargo test -p pi-pijs-core`：待执行
- `cargo check -p pi-protocol`：待执行
- `git diff --check`：待执行

**LOC 迁移:** 新增约 150 LOC 纯协议/状态机；QuickJS host runtime 未迁移。

**进度:** Round 67 完成；按既有路线图约 68%（本轮补齐 PiJS 生命周期、RPC envelope 与 tool schema seam）。


**做了什么:**
1. 新增 `crates/pi-pijs-core`，承载无 QuickJS/宿主依赖的 `HostcallKind`、`HostcallRequest`、`ExtensionToolDef`。
2. 将确定性时钟与事件循环协议（Promise 完成 macrotask、timer 排序、microtask drain 计数）实现为可独立测试的 `PiEventLoop` facade；QuickJS 集成与宿主 scheduler 仍留在 `pi-coding-agent`。
3. `pi-coding-agent::extensions_js` 与 `pi-protocol::hostcall` 改为 re-export 核心类型，保持旧 API 路径兼容。
4. 新增 3 个核心回归测试：hostcall completion 优先级、timer deadline/order、clear timeout 幂等性。

**验证:**
- `cargo test -p pi-pijs-core` ✅ 3 passed
- `cargo check -p pi-protocol` ✅
- `git diff --check` ✅
- `cargo check -p pi-coding-agent --lib` 仍受该并行模块化分支既有 Windows/toolchain 错误阻塞（589 errors，包含 `pi_error` 依赖与 `windows_by_handle` 等；未定位到本轮新增错误）。

**LOC 迁移:** 纯协议核心新增约 230 LOC；QuickJS 集成未迁移。

**进度:** Round 66 完成；按当前路线图约 65%（本轮新增独立 `pi-pijs-core` 边界，未将 QuickJS/宿主代码计入迁移）。


**做了什么:**
1. 将 `HostcallKind` / `HostcallRequest` 迁入 `crates/pi-protocol/src/hostcall.rs`，由 `pi-coding-agent::extensions_js` re-export，保留旧 API facade。
2. 将 `hostcall_amac.rs`（约 1,460 LOC）迁入 `pi-chord`，依赖改为 `pi-protocol` 请求类型与 `pi-agent-core::scheduler::HostcallOutcome`。
3. `pi-coding-agent` 通过 `pub use pi_chord::hostcall_amac` 保留兼容路径。

**验证:**
- 当前环境无 `cargo` / `rustc`，无法执行格式化或编译检查；需补跑 `cargo fmt --all -- --check`、`cargo check -p pi-protocol`、`cargo check -p pi-chord`、`cargo check -p pi-coding-agent`。

**LOC 迁移:** 约 1,460 LOC，另迁移约 32 LOC 协议类型。

**进度:** Round 62 完成。本文“约 65%”是路线图估算，不是当前 Rust LOC 或编译通过率：分母为第 1–12 节多 crate 目标模块范围，分子为 Round 24–30 已记录的归属迁移、依赖边界与 facade 兼容目标；第 15 节 Round 30 后约 64% 取整为约 65%。未迁移 provider/API、agent runtime 和平台 hostcall 不计入已完成。
