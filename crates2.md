# pi.rs 模块化进度（Round 72）

## Round 72 — providers 客户端核心拆分

- 新增 `pi-providers-core`，仅依赖标准库，承载 provider 注册、大小写/空白无关的别名解析、显式 API 覆盖、去重 fallback 链与线程安全健康状态过滤。
- `pi_agent_rust` 通过 facade re-export `ProviderRegistry`、`ProviderRoute`、`ProviderHealth`、`HealthTracker` 与 `ProviderRegistration`；HTTP client、具体 provider 实现和 auth 仍留在 coding-agent。
- 新增 3 个纯逻辑回归测试，覆盖 alias/API、fallback 去重和 unavailable 健康状态。

验证：`cargo test -p pi-providers-core` ✅（3 passed）；`git diff --check` ✅。`cargo check -p pi_agent_rust --lib` 仍受当前 Windows nightly 基线问题阻塞：`signal_hook` 的 Unix-only API 与 `windows_by_handle` unstable API（均与本轮无关）。

进度：Round 72 providers core 完成；HTTP/auth 未迁移，后续可在该 seam 上继续接入具体 provider factory。
