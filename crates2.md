# pi.rs 模块化进度（Round 78）

## Round 78 — 哈希与编码核心拆分 ✅

- 新增 `pi-hash-core`，承载无运行时/文件 IO 依赖的 SHA-1/SHA-256/SHA-384/SHA-512、MD5、BLAKE3 摘要实现与 `HashDigest` trait。
- 提供统一的算法解析、原始摘要、十六进制和标准 base64 编解码 API；非法 hex 输入显式返回错误。
- `coding-agent`/`pi` 保留 agent runtime、文件 IO 和 QuickJS crypto hostcall 编排；核心 crate 不反向依赖 runtime。
- 单元测试覆盖算法名称归一化、SHA-256/MD5/BLAKE3 已知向量、hex/base64 往返及非法输入。

验证：`cargo fmt --all -- --check`、`cargo test -p pi-hash-core`、`cargo check -p pi --lib`。

进度：Round 78 哈希摘要核心 crate 已创建并加入 workspace；当前整体模块化进度约 77%。
