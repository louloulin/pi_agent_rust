# pi.rs 模块化进度（crates2）

## Round 67：CLI 命令与参数核心拆分

**已完成：**
1. 新增 `crates/pi-cli-core`，承载 `Cli`、`Commands`、参数预解析、扩展参数提取、参数校验与 Clap help 生成。
2. `pi::cli` 保留为兼容 facade，通过 `pub use pi_cli_core::*` 维持旧 API 路径；运行时入口 `src/main.rs` 无需改动。
3. `ftui` feature 透传到核心 crate，保证 feature-gated CLI 参数与旧构建行为一致。

**验证：**
- `cargo fmt --all -- --check`：未执行，环境缺少 `cargo`。
- `cargo check --lib --no-default-features`：未执行，环境缺少 `cargo`。
- CLI 定向测试：未执行，环境缺少 `cargo`。

**说明：**
本轮迁移保持纯命令解析、参数校验和 help 生成与运行时解耦；`pi::cli::*` facade 仅负责兼容导出，后续可将 `pi-cli-core` 独立发布或被其他 front-end 复用。
