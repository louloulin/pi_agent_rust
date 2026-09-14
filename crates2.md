# crates2.md

## Round 69 — todo 任务列表核心拆分

- [x] 新增 `crates/pi-todo-core`：纯 `TodoList` 状态模型、状态机操作、自动晋级调度、依赖无关渲染。
- [x] `src/todo.rs` 保留 session entry replay、持久化与 `TodoTool` runtime adapter。
- [x] core crate 不依赖文件 IO、session 或 agent runtime。
- [x] workspace 接线完成，已提交 `4b55931ea`、`5483a2fda`。
- [ ] 后续轮次：继续抽离更多 coding-agent runtime adapter，并完善跨 crate API 文档。

进度：约 85%（核心 crate、runtime adapter、workspace 接线和提交已完成；cargo 质量门禁受当前环境缺少 Rust 工具链阻塞）。
