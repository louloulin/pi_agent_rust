# pi.rs 模块化进度（Round 77）

## Round 77 — HTML 转义与 URL 组件核心拆分 ✅

- 新增 `pi-html-core`，承载无运行时/UI 依赖的 HTML escape、百分号 URL 编解码、query pair 解析、OAuth code/state 片段解析和 URL query 构建逻辑。
- `pi`/coding-agent 保留 session HTML 渲染、OAuth 网络/凭据流程与 UI；后续接入 facade 时仅依赖 `pi-html-core` 的纯函数，不反向依赖 runtime。
- 新增 4 个单元测试，覆盖 HTML 特殊字符、Unicode URL 编解码、query/fragment OAuth 输入和 query URL 构建。

验证：待完成 wiring 后运行 `cargo fmt --all -- --check`、`cargo test -p pi-html-core`、`cargo check -p pi --lib`。

进度：Round 77 核心 crate 已创建；当前整体模块化进度约 76%，HTML/OAuth 逻辑 facade 接入尚未完成。
