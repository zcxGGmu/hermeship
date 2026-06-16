# Task: Milestone 8.1 - Git Source 本地 deterministic parity

更新时间：2026-06-17 Milestone 8.1 已完成，提交待生成

本文件是当前开发工作台。本轮已完成 Milestone 8.1 Git Source 的本地 deterministic parity 路径。Hermeship 仍然是 Hermes-native daemon-first event router，不调用 clawhip runtime，不依赖运行中的 clawhip daemon。

本次边界：实现 `hermeship git commit` 与 `hermeship git branch-changed` 的本地事件构造、typed conversion、路由 metadata 和默认渲染；不实现真实 git polling source、真实 GitHub、tmux、cron、memory、live verification、Slack sink 或 Hermes plugin/observer。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 启动时工作树：`git status --short --branch` 只显示分支行，工作树干净。
- 最新文档交接提交：`475f2a3 docs: 更新 Hermeship Milestone 8 开发入口`。
- 最新功能阶段提交：`162efcd feat: 增加安装生命周期与发布预检`。
- 最近提交：`475f2a3 docs: 更新 Hermeship Milestone 8 开发入口`、`162efcd feat: 增加安装生命周期与发布预检`、`64e8641 docs: 更新 Hermeship 最新开发状态`。
- Milestone 0 到 Milestone 7 已完成并提交。
- Milestone 8.1 已完成并验证，提交 hash 待本次提交生成后回填。
- Milestone 8.2 到 Milestone 10 未完成。

## 已完成能力

- 已实现 Rust CLI/config/event/privacy/daemon/router/renderer/dispatcher/sinks/lifecycle/preflight 主路径。
- 已实现 daemon `/health`、`/event`、`/api/hermes/hook`、bounded queue、privacy sanitizer、DaemonClient health/event/hook POST。
- 已实现 Router、DefaultRenderer、Dispatcher、Sink trait、FakeSink、Discord sink、sink 失败语义、本地 daemon -> fake sink smoke。
- 已实现 Hermes hook bridge 模板、install-hooks/uninstall-hooks、安全卸载 marker、handler fail-open smoke。
- 已实现 `hermeship install`、`setup`、`uninstall` 和 `release preflight <version>` 的本地 deterministic 路径。
- 已新增 `deploy/hermeship.service` 与 `docs/operations.md`；本阶段不真实执行 `systemctl` 或 `launchctl`。

## 未完成范围

- Milestone 8 剩余：GitHub、tmux、cron 和 memory scaffold。
- Milestone 9：README/architecture/event contract/live verification runbook 与首次 live check。
- Milestone 10：Hermes plugin / observer 研究与可选 MVP。
- 真实 live verification 尚未执行。
- Slack sink 尚未实现。
- Hermes plugin/observer 尚未启动。
- 真实 systemd/launchd 安装自动化尚未实现。

## 当前执行计划

- [x] 复习启动文档。
  - `tasks/lessons.md`
  - `docs/development-status.md`
  - `docs/plans/2026-06-15-hermeship-development-plan.md`
  - `tasks/development-checklist.md`
  - `tasks/todo.md`

- [x] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 结果：分支为 `codex/milestone-1-cli`，启动时工作树干净，最近提交为 `475f2a3`、`162efcd`、`64e8641`。

- [x] 确认 Milestone 8 计划。
  - 文件：`tasks/development-checklist.md`
  - 入口：`## Milestone 8：clawhip 功能 Parity 扩展`
  - 第一项：任务 8.1 Git Source。

- [x] 阅读 Milestone 8 相关代码和 fixture 规则。
  - `src/cli.rs`
  - `src/main.rs`
  - `src/config.rs`
  - `src/events.rs`
  - `src/event/mod.rs`
  - `src/event/body.rs`
  - `src/event/compat.rs`
  - `src/router.rs`
  - `src/render/mod.rs`
  - `src/render/default.rs`
  - `src/dispatch.rs`
  - `src/lifecycle.rs`
  - `src/release_preflight.rs`
  - `tests/fixtures/README.md`
  - 方案文档中 CLI、source/parity、测试矩阵和发布章节。

- [x] 写失败测试：Git source 事件构造。
  - 新建：`src/source/mod.rs`
  - 新建：`src/source/git.rs`
  - 覆盖：commit 事件包含 `repo`、`repo_name`、`repo_path`、`worktree_path`、`branch`、`commit`、`short_commit`、`summary`、`author_name`、`author_email`。
  - 覆盖：branch changed 事件包含 `repo`、`repo_name`、`repo_path`、`worktree_path`、`old_branch`、`new_branch`、`branch`。
  - 隐私边界：不包含完整 diff、完整 commit body、token、cookie、secret。

- [x] 写失败测试：typed conversion 与 route metadata。
  - 修改：`src/event/body.rs`
  - 修改：`src/event/mod.rs`
  - 修改：`src/event/compat.rs`
  - 覆盖：`git.commit` -> `EventBody::GitCommit`，metadata 提取 repo/branch/path，priority 为 low。
  - 覆盖：`git.branch-changed` -> `EventBody::GitBranchChanged`，route filter 可用 `repo_name` 和 `branch`。

- [x] 写失败测试：CLI 解析与 daemon submit。
  - 修改：`src/cli.rs`
  - 修改：`src/main.rs`
  - 修改：`tests/fixtures/cli/public_commands.txt`
  - 覆盖：`hermeship git commit ...` 和 `hermeship git branch-changed ...` 能 parse。
  - 覆盖：命令会构造 `IncomingEvent` 并 POST 到 daemon test queue。

- [x] 写失败测试：默认 renderer。
  - 修改：`src/render/default.rs`
  - 覆盖：commit compact/inline/alert/raw 不泄漏完整 commit body 或 secret。
  - 覆盖：branch changed compact 可读。

- [x] 运行 Red 验证。
  - 命令：`cargo test git`
  - 结果：失败于缺少 `source::git` API、`GitCommands`、`Commands::Git` 和 `EventBody::GitCommit` / `GitBranchChanged` variants。

- [x] 实现最小本地 deterministic Git Source。
  - 只构造 Hermeship 自己的 `IncomingEvent`/`EventEnvelope`。
  - 不调用 clawhip runtime。
  - 不依赖运行中的 clawhip daemon 之外的本地 test daemon。
  - 不执行真实 `git` 命令、不轮询 repo、不访问远端。

- [x] 运行 Milestone 8.1 验证。
  - `cargo test git`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
  - 结果：`cargo test git` 通过 11 个 lib-filtered + 2 个 bin-filtered 测试；`cargo test release_preflight` 通过 6 个测试；`cargo run -- release preflight 0.1.0` 本地 checks ok，live verification pending；`cargo fmt --all -- --check` 退出码 0；`cargo clippy --all-targets -- -D warnings` 退出码 0；`cargo test` 通过 150 个 lib 测试、10 个 bin 测试和 doc-tests。

- [x] 更新开发状态并提交。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 更新：`docs/development-status.md`
  - commit 信息使用中文，说明变更、验证和影响；提交 hash 待本次提交生成后回填。

## Review

- 已完成 Milestone 8.1 Git Source 本地 deterministic parity。
- 新增 `src/source/git.rs`，提供 `git.commit` 和 `git.branch-changed` 的 `IncomingEvent` 构造；本阶段不执行真实 `git`、不轮询 repo、不访问远端。
- 已根据代码审查补强 Git 输入校验：source 与 compat 都拒绝无效 commit SHA、空 summary、多行 summary 和过长 display field。
- 已新增 typed Git body，并让 `git.commit` / `git.branch-changed` 进入现有 `IncomingEvent -> EventEnvelope -> Router -> Renderer -> Sink` 管线。
- 已接入 CLI：`hermeship git commit` 与 `hermeship git branch-changed`，并更新公开命令 fixture与 release preflight 检查。
- 默认 renderer 输出 Git 安全摘要；raw JSON 不展开完整 diff、commit body、repo path、worktree path 或 author email，并有回归测试覆盖。
- 已验证：`cargo test git`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 未进入范围：真实 git polling source、GitHub source、tmux source、cron、memory、live verification、Slack sink、Hermes plugin/observer。
- 下一入口：Milestone 8.2 GitHub Source；继续默认使用本地 deterministic fixture，不依赖真实 GitHub 或外网。
