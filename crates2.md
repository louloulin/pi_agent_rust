# pi.rs 模块化拆分记录

## Round 89 — `pi-media-tools-core`

新增 `crates/pi-media-tools-core`，承载媒体处理的纯逻辑：`MediaAsset`、`MediaTransform`、媒体扩展名到 MIME 的映射，以及大小/扩展名校验。该 crate 不依赖 runtime、文件系统、provider、异步或 UI。

`src/media_tools.rs` 保留 `InspectImageTool`、`ReadMediaTool`、`GenerateImageTool`、`TtsTool` 的 runtime/tool adapter，并通过重导出维持现有 `pi::media_tools` API 兼容。
