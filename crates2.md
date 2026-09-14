# pi.rs 模块化改造进度与插件协议兼容性证据

## Round 62 — 插件协议兼容性盘点与测试

本轮目标是盘点 pi_agent_rust 与 pi-mono/pi.dev 插件生态之间的纯协议边界，并把可重复验证的证据固定到 fixture/test；本轮不修改 crypto_shim 或 hostcall_amac。

### 覆盖项

| 协议面 | Rust 证据 | pi-mono/pi.dev 对照 | 结论 |
|---|---|---|---|
| Agent lifecycle / tool events | src/agent.rs 的 AgentEvent；tests/plugin_protocol_compatibility.rs | legacy_pi_mono_code/pi-mono/packages/coding-agent 的事件与 RPC 示例 | agent_start、turn_start、tool_execution_start/end 的 type 为 snake_case，字段为 camelCase |
| Extension dispatch | src/extensions.rs:extension_event_from_agent | pi.on(...) event hook surface | 已验证 AgentEvent 序列化结果与 dispatch 名称一致 |
| Extension UI request | src/extensions/protocol.rs:ExtensionUiRequest::to_rpc_event | examples/rpc-extension-ui.ts | 已验证 extension_ui_request、id、method、扁平 payload 与 capabilityPrompt 的 host-authoritative 覆盖 |
| Extension protocol envelope | docs/schema/extension_protocol.json | docs/rpc.md 与 pi-mono RPC 示例 | 已用 schema validator 验证 register、host_call、host_result(error) fixture |
| Fixture provenance | tests/fixtures/plugin_protocol_compatibility.json | pi-mono/pi.dev | 固定 json-lines、snake_case type、camelCase field 约定，便于后续回归 |

### 当前覆盖率与边界

- 本轮新增 4 个协议回归测试：fixture wire metadata、AgentEvent/dispatch 对照、Extension UI envelope、protocol schema validation。
- 覆盖的最小 wire cases：4 个 lifecycle/tool event、2 个 UI request、3 个 extension protocol envelope（register、host_call、host_result error）。
- 已覆盖的兼容边界：事件类型命名、camelCase 字段、UI 请求 payload 扁平化、typed envelope 防 payload spoof、协议 schema required/error 分支。
- 尚未声称运行时插件生态完全兼容：JS QuickJS/Node/Bun shim、npm stub、capability policy 行为仍由既有 extension conformance/stress suites 覆盖，不在本轮纯协议 fixture 范围内。

### 验证命令

    cargo test --test plugin_protocol_compatibility
    cargo fmt --check
    git diff --check

进度：Round 62 插件协议盘点与纯协议测试项 100%；整体模块化改造进度不在本轮重新估算。
