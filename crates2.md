## Round 78 — `pi-git-core` Git 解析核心 ✅

新增 `crates/pi-git-core`，迁入可复用且无副作用的 Git porcelain/ref/commit 解析：
- `summarize_porcelain`：staged、unstaged、untracked、deleted 及总数统计
- `porcelain_paths`：路径提取与 rename `old -> new` 解析
- `parse_head` / `parse_ref_name`：branch 与 detached HEAD 解析
- `canonical_commit_oid`：40 位十六进制 commit OID 校验与规范化
- `parse_gitdir_marker`：`.git` worktree marker 解析

`pi` 的 `doctor` 与 context preview 已复用新 crate；Git 命令执行、可信 executable、环境隔离和 agent runtime 仍保留在 coding-agent。

验证：`cargo check -p pi-git-core`、`cargo test -p pi-git-core`、`cargo check --lib`、`cargo test --lib`。
