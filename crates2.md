# crates2.md

## Round 69 — todo 任务列表核心拆分

- [x] 新增 `crates/pi-todo-core`：纯 `TodoList` 状态模型、状态机操作、自动晋级调度、依赖无关渲染。
- [x] `src/todo.rs` 保留 session entry replay、持久化与 `TodoTool` runtime adapter。
- [x] core crate 不依赖文件 IO、session 或 agent runtime。
- [ ] 后续轮次：继续抽离更多 coding-agent runtime adapter，并完善跨 crate API 文档。

进度：约 70%（本轮核心 crate 与 adapter 已完成；workspace 接线、全量质量门禁与提交推送待完成）。
