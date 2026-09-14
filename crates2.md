# pi.rs 模块化改造记录

## Round 95：`pi-plan-core`

新增 `crates/pi-plan-core`，承载计划领域的纯数据与生命周期逻辑：`Plan`、`PlanStep`、`PlanStatus`、步骤完成状态、校验和 serde 序列化。agent runtime、提交工具和 UI 不进入该 crate；当前 checkout 是旧版单 crate，因此后续 coding-agent 接入可通过 workspace path dependency 完成。

验证：`cargo test -p pi-plan-core`
