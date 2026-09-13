# pi.rs 模块化复刻后续计划 (crates2.md)

> 本文档是 `crates1.md` 的续篇,定义 **Round 24 起** 的具体执行路径。
> 上游参考:`legacy_pi_mono_code/pi/`(= `https://github.com/earendil-works/pi.git` 快照,已 commit 到本仓库)。

---

## 0. 当前状态摘要(2026-09-13)

| 维度 | 数字 |
|------|------|
| 包结构镜像(11 个同名 crate) | ✅ 100% |
| Cargo check 全绿(lib + bin + workspace) | ✅ 0 error |
| `pi-mono` aggregator surface | ✅ 12/12 package |
| 代码归属镜像(与 upstream 一致) | ⚠️ ~30% |
| Provider 覆盖 | ⚠️ 15% (13 / 87) |
| 195 dead_code warnings | ⚠️ 未收敛 |

**核心矛盾**:`pi-coding-agent` 531,436 LOC vs upstream `pi-coding-agent` 143,033 LOC(4x),因为 7 个 phase-2 crate 的代码都被 Round 18 吸收到这里。

---

## 1. 目标(Round 30 结束时)

| 维度 | 当前 | 目标 |
|------|------|------|
| 代码归属镜像 | ~30% | **50%+** |
| Provider 覆盖 | 15% | **40%+** |
| 7 个空 marker crate | 7 | **0-2**(允许 `pi-server / pi-evals` 仍为 marker,因为上游它们也偏小) |
| Cargo check + Cargo build | ✅ | ✅ |
| Cargo test --workspace | ⏳ | ⏳(不在本计划强制要求) |

---

## 2. 恢复路径总表(7 步,按 ROI 排序)

| Round | 路径 | 目标 crate | 涉及模块 | 价值 | 风险 | 预计 LOC 迁出 |
|-------|------|------------|----------|------|------|---------------|
| 24 | (E) 拆 `pi-protocol` | `pi-protocol` | `rpc / acp / jsonrpc / sdk / http_shim` | +0.5% | 低 | ~1.4K |
| 25 | (G) 拆 `pi-client` | `pi-client` | `web_remote / client / connectors` | +0.5% | 低 | ~2K |
| 26 | (F) 拆 `pi-session-backends` | `pi-session-backends` | `session / session_sqlite / session_store_v2 / session_picker / session_import / session_index / session_test` | +1% | 中 | ~4K |
| 27 | (C) 拆 `pi-chord` | `pi-chord` | `extensions_api / extensions / extensions_js / connectors` | +7% | 中 | ~25K |
| 28 | (D) 拆 `pi-tui` | `pi-tui` | `tui / interactive / interactive_ftui / terminal_images / autocomplete` | +10% | 中 | ~36K |
| 29 | (B) provider 全量补齐 | `pi-ai` | 新增 70+ providers 缺失实现 | +10% | 低 | +50K |
| 30 | (A) agent / evals / server | `pi-agent-core / pi-evals / pi-server` | `agent / agent_cx / agent_hub / handoff / subagents / eval / debug / package_manager / plan / jobs / github / review` | +5% | 高 | ~10K |

预计 Round 30 结束时真实复刻度 **~50%**(上限受 Rust/TS 编码范式差异限制)。

---

## 3. Round 24 — 拆 `pi-protocol`(执行清单)

### 3.1 涉及文件(从 `pi-coding-agent/src/`)
- `rpc.rs`
- `acp.rs`
- `jsonrpc.rs`
- `sdk.rs`
- `http_shim.rs`(已经在 `pi-coding-agent`,需要决定:留在 coding-agent 还是搬到 protocol)
- `validation_broker.rs`(JSON-RPC 验证辅助)

### 3.2 涉及依赖关系
- `pi-coding-agent` 不再依赖这些模块
- `pi-protocol` 依赖:`serde / serde_json / anyhow / thiserror / pi-error`
- 外部 consumer:`bin/pi`(main.rs)使用 `pi_protocol::*`

### 3.3 执行步骤

```bash
# Step 1: 物理迁移
git mv crates/pi-coding-agent/src/{rpc,acp,jsonrpc,sdk,validation_broker}.rs \
       crates/pi-protocol/src/

# Step 2: http_shim 暂时保留(它依赖很多内部 helper,不优先搬)

# Step 3: 更新 pi-protocol/Cargo.toml:加 pi-error / serde / serde_json
# Step 4: 更新 pi-protocol/src/lib.rs:pub mod rpc; pub mod acp; ...
# Step 5: 更新 pi-coding-agent/Cargo.toml:加 pi-protocol 依赖
# Step 6: 更新 pi-coding-agent/src/lib.rs:把 pub mod rpc/acp/... 删掉
# Step 7: 修复 bin/pi 的 use 路径 pi_coding_agent::rpc::X → pi_protocol::rpc::X
# Step 8: cargo check -p pi-protocol -p pi-coding-agent --bin pi
```

### 3.4 预期回退预案

如果出现循环依赖(pi-protocol → pi-coding-agent 因为某些类型需要),采用:
- **方案 a**:把共享类型移到 `pi-error`(如 `RpcId / MethodId`)
- **方案 b**:`pi-protocol` 暴露 trait,`pi-coding-agent` 实现

---

## 4. Round 25 — 拆 `pi-client`(执行清单)

### 4.1 涉及文件
- `web_remote.rs`(主)
- `connectors/*.rs`(http 客户端实现)
- `client.rs`(如有)

### 4.2 依赖关系
- `pi-client` 依赖 `pi-protocol`(Round 24 拆出后)+ `pi-error`
- `pi-coding-agent` 不再需要直接拥有 web_remote

### 4.3 执行步骤

```bash
git mv crates/pi-coding-agent/src/web_remote.rs crates/pi-client/src/
git mv crates/pi-coding-agent/src/connectors crates/pi-client/src/
# 更新 Cargo.toml + lib.rs + bin/pi use 路径
```

---

## 5. Round 26 — 拆 `pi-session-backends`(执行清单)

### 5.1 涉及文件
- `session.rs`
- `session_sqlite.rs`
- `session_store_v2.rs`
- `session_picker.rs`
- `session_import.rs`
- `session_index.rs`
- `session_test.rs`
- `compaction_snap.rs`(session 内核心)

### 5.2 依赖关系
- `pi-session-backends` 依赖 `pi-error` + `serde_json` + `sqlite` 栈
- 上游对应 `pi-session-backends` 包:`@earendil-works/pi-session-backends`

### 5.3 风险点
- `session.rs` 大量 agent event 流依赖,需要先把 agent event 流抽象成 trait
- 否则 `pi-session-backends → pi-agent-core → pi-coding-agent` 循环

---

## 6. Round 27 — 拆 `pi-chord`(最大工程)

### 6.1 涉及文件
- `extensions_api.rs`(超大,~25K LOC)
- `extensions/mod.rs`(目录)
- `extensions/*.rs`(10+ 文件)
- `extensions_js.rs`
- `connectors/*.rs`(如果 Round 25 没搬完)

### 6.2 上游结构(`legacy_pi_mono_code/pi/packages/chord/src/`)
```
api.ts
bundler.ts
context/        # chord 上下文
delta/          # chord delta 同步
facets/         # chord facets 系统
index.ts
json.ts
node/           # chord node 实现
node.ts
services/       # chord services
types.ts
```

### 6.3 上游 `chord` 包描述(README)
> Standalone application-composition runtime for services, replicated state, RPC, and plugins

### 6.4 拆解策略

`pi-chord` 不只搬 extensions。还需要引入真正的 chord facets/services 抽象:

| Rust 模块 | 对应 upstream chord 概念 |
|-----------|-------------------------|
| `extensions_api::ExtensionManager` | `services/` |
| `extensions_api::PolicyDecision / PolicyProfile` | `facets/` |
| `extensions_api::hostcall_*` | `node/` |
| `extensions_api::validate_*` | `bundler.ts` 验证机制 |

**架构改动**:把 chord facets 抽象成 trait,让 `pi-coding-agent` 通过 trait 调用而不是直接依赖具体类型。

---

## 7. Round 28 — 拆 `pi-tui`(第二大工程)

### 7.1 涉及文件
- `tui.rs`(~15K LOC)
- `interactive.rs`(~10K)
- `interactive_ftui.rs`(~5K)
- `terminal_images.rs`
- `autocomplete.rs`

### 7.2 上游结构(`legacy_pi_mono_code/pi/packages/tui/src/`)
```
alt-screen-search.ts
autocomplete.ts
components/          # ~40 文件,所有 UI 组件
editor-component.ts
fuzzy.ts
keybindings.ts
keys.ts
layout.ts
markdown.ts
mouse-region.ts
terminal.ts
tui-alt-screen.ts
...
```

### 7.3 风险点
- TUI 强依赖 agent event 流 → 需要抽象成 `EventSource` trait
- 大量 `bubbletea / ftui` 集成代码需要清理依赖图

---

## 8. Round 29 — Provider 全量补齐

### 8.1 当前覆盖(13 / 87)

我们已实现:`anthropic / azure / bedrock / cohere / copilot / cursor / gemini / gitlab / openai / openai_responses / vertex / model_fetch`

### 8.2 上游缺失 providers(74 个)
- `amazon-bedrock` ✅
- `ant-ling` ❌
- `anthropic` ✅
- `azure-openai-responses` ✅
- `baseten` ❌
- `cerebras` ❌
- `cloudflare-ai-gateway` ❌
- `cloudflare-workers-ai` ❌
- `deepseek` ❌
- `faux`(测试用) ❌
- `fireworks` ❌
- `github-copilot` ✅
- `google` ✅
- `google-vertex` ✅
- `groq` ❌
- `huggingface` ❌
- `mistral` ❌
- `moonshot` ❌
- `nvidia` ❌
- `ollama` ❌
- `openai-completions` ❌
- `openai-responses` ✅
- `openrouter` ❌
- `perplexity` ❌
- `pi-mono`(internal) ❌
- `qwen` ❌
- `replicate` ❌
- `sambanova` ❌
- `together` ❌
- `vercel-ai-gateway` ❌
- `xai` ❌
- ... (更多)

### 8.3 执行策略
- 不在 Round 29 一次性实现所有 74 个
- 优先实现常用:`openai-completions / openai-responses / anthropic-messages / google-generative-ai / bedrock-converse`
- 这些上游 `pi-ai/src/api/` 都有完整实现,可以直接复制

---

## 9. Round 30 — 收尾

### 9.1 涉及文件
- `agent.rs / agent_cx.rs / agent_hub.rs / handoff.rs / subagents.rs` → `pi-agent-core`
- `eval / debug / lsp` → `pi-evals`
- `package_manager / plan / jobs / github / review / xdev` → `pi-server`

### 9.2 风险点
- 重新引入 `pi-coding-agent → pi-agent-core` 循环风险
- 需要把 agent event 流抽象成 trait 后才能干净迁移

---

## 10. 风险与约束

1. **循环依赖**:每轮拆解前先画依赖图,任何反向引用 `pub use pi-coding-agent::*` 在被拆 crate 都视为违规
2. **trait 抽象成本**:Round 27/28/30 需要先抽象 trait,工作量大
3. **二进制入口**:`bin/pi` 的 `main.rs` 在每轮都可能需要更新 `use` 路径,这是发现路径遗漏的最好测试
4. **dead_code warnings**:每轮拆解可能产生新的 dead_code,因为一些 helper 不再被使用 — 用 `#[allow(dead_code)]` 收敛

---

## 11. 完成度估算(每轮目标)

| Round | 代码归属完成度 | 备注 |
|-------|----------------|------|
| 现在 | ~30% | Round 23 基线 |
| 24 后 | ~30.5% | 拆 protocol |
| 25 后 | ~31% | 拆 client |
| 26 后 | ~32% | 拆 session-backends |
| 27 后 | ~39% | 拆 chord(最大单步) |
| 28 后 | ~49% | 拆 tui |
| 29 后 | ~59% | provider 补齐 |
| 30 后 | ~64% | agent/evals/server 收尾 |

注:这些是**代码归属复刻度**,不是结构镜像度(结构镜像已经 100%)。

---

## 12. 与 crates1.md 的关系

| 文档 | 范围 |
|------|------|
| `crates1.md` | Round 1-23,11 个 phase-2 crate 骨架 + 224 文件迁移 + 50 个 leaf inline + cargo check 全绿 + pi-mono aggregator |
| `crates2.md`(本文件) | Round 24-30,把代码归属镜像从 30% 推到 64% |

---

> 本文档版本:v1.0(2026-09-13)
> 与 Multica issue `01a08d97` 绑定,分支 `feature/crates0911`
> 参考:`legacy_pi_mono_code/pi/packages/*/src/`(earendil-works/pi 快照)