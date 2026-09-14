# Rust crate modularization map

本轮拆分：

- `crates/pi-stats-core`：stats/metric 纯算法，包括 session JSONL 扫描、过滤、token/cost 聚合、provider/model 与 day 分桶、tool-call 统计及文本/Markdown 渲染。无 runtime、UI 或网络依赖。
- `src/stats.rs`：`pi::stats` 兼容 facade，仅重新导出 `pi-stats-core` API。
- `src/main.rs`：继续保留 CLI/runtime 调用，不迁移 UI 或 agent runtime。
