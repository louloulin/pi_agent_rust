# pi.rs 模块化进度（Round 73）

## Round 73 — events 事件总线核心拆分 ✅

- 新增 `pi-events-core`，迁入无 runtime/UI 依赖的订阅 ID、事件过滤（精确名/前缀）及线程安全发布分发核心。
- `pi::sdk::EventListeners` 改用 `pi-events-core::EventBus`，保留原有 `subscribe` / `unsubscribe` / `notify` API；回调快照后执行，支持回调内修改订阅。
- `AgentEvent` 实现核心 crate 的 `EventType`，runtime 事件类型映射留在 coding-agent；UI 与 agent runtime 行为不迁移。
- workspace 注册 `pi-events-core`，无新增第三方依赖。

验证：本环境未安装 `cargo`，无法执行 cargo test/check；已执行 `git diff --check`。

进度：Round 73 events 核心完成，累计模块化进度按既有 crates2.md 口径更新。
