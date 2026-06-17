# Task: Milestone 9.1 文档与事件契约

更新时间：2026-06-17

本文件是当前开发工作台。本轮从 Milestone 9.1 继续，只处理 README、operations、事件契约和架构文档一致性，以及对应的本地 deterministic 验证。Hermeship 仍然是 Hermes-native daemon-first event router，不调用 clawhip runtime，不依赖运行中的 clawhip daemon。

本轮边界：默认不实现 Slack sink，不启动 Hermes plugin/observer，不执行真实 live verification，除非清单和凭据条件明确允许。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 当前工作树：`git status --short --branch` 仅显示分支行，工作树干净。
- 文档交接提交：`3852e60 docs: 更新 Hermeship Milestone 9 交接状态`。
- 最新功能阶段提交：`0b12de3 feat: 增加 cron 与 memory scaffold`。
- 最近提交基线：`3852e60 docs: 更新 Hermeship Milestone 9 交接状态`、`0b12de3 feat: 增加 cron 与 memory scaffold`、`6c9af3e docs: 更新 Hermeship Milestone 8.4 交接状态`。
- Milestone 0 到 Milestone 8.4 已完成。
- Milestone 9 到 Milestone 10 未完成。
- 下一入口：Milestone 9 文档与 live verification。

## 当前执行计划

- [x] 复习 lessons 与交接状态。
  - `tasks/lessons.md`
  - `docs/development-status.md`
  - `docs/plans/2026-06-15-hermeship-development-plan.md`
  - `tasks/development-checklist.md`
  - `tasks/todo.md`
  - 记录：已确认阶段完成后必须验证并提交、方案/清单分离、Hermeship 不是 thin adapter；本阶段先做 Milestone 9.1 文档对齐。

- [x] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 记录：当前分支为 `codex/milestone-1-cli`；工作树启动时干净；最近提交为 `3852e60 docs: 更新 Hermeship Milestone 9 交接状态`、`0b12de3 feat: 增加 cron 与 memory scaffold`、`6c9af3e docs: 更新 Hermeship Milestone 8.4 交接状态`。

- [x] 确认 Milestone 9.1 计划。
  - 文件：`tasks/development-checklist.md`
  - 入口：`## Milestone 9：文档与 Live Verification`
  - 下一项：任务 9.1 README 与运维文档。
  - 记录：本阶段只更新 README、operations、event contract 和 architecture；真实 live check 留到 Milestone 9.2/9.3。

- [x] 写入本轮实施计划。
  - 文件：`tasks/todo.md`
  - 计划边界：只做文档一致性和本地 deterministic 验证，不新增 Slack sink、Hermes plugin/observer、真实 live verification 或新的 runtime 行为。
  - 文件结构：
    - 更新：`README.md`，改成当前可执行入口、安装配置、daemon、hooks、send/emit/explain、运维和 live check 的用户入口。
    - 更新：`docs/operations.md`，对齐安装、setup、start、install-hooks、uninstall、rollback 和常见故障。
    - 新建：`docs/hermes-event-contract.md`，描述 Hermes hook input、canonical events、payload 字段、隐私规则和递归脱敏边界。
    - 新建：`ARCHITECTURE.md`，描述 Hermeship 实际模块、daemon-first 数据流和与 clawhip 的差异。
    - 复核：`src/cli.rs`、`src/release_preflight.rs`、`tests/fixtures/cli/public_commands.txt`、`tests/fixtures/README.md`，确保文档命令和 preflight 覆盖不漂移。
  - 验证计划：
    - 先更新四份文档，再用 `rg -n "hermeship start|hermes install-hooks|hermes.agent|Discord|rollback" README.md docs ARCHITECTURE.md` 检查公开入口。
    - 再运行 `cargo test release_preflight` 和 `cargo run -- release preflight 0.1.0`，确认文档命令与 preflight 一致。
    - 最后运行 `cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 作为阶段门禁。

- [x] 更新 README、operations、event contract 和 architecture 文档。
- [x] 复核公开命令、fixture policy 和 release preflight 的文档一致性。
- [x] 运行 Milestone 9.1 验证命令。
- [x] 更新开发清单运行日志和本文件 Review。
- [ ] 提交 Milestone 9.1。
  - commit 信息使用中文，说明文档变更、验证结果和影响。

## Review

- 已完成：Milestone 9.1 文档更新与验证。`README.md`、`docs/operations.md`、`docs/hermes-event-contract.md` 和 `ARCHITECTURE.md` 已对齐当前 CLI、hook bridge、事件契约、路由/渲染边界和 rollback 说明；`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 均通过，live verification 仍保持 pending。

## Milestone 8.2 完成记录

- [x] 复习启动文档。
  - `tasks/lessons.md`
  - `docs/development-status.md`
  - `docs/plans/2026-06-15-hermeship-development-plan.md`
  - `tasks/development-checklist.md`
  - `tasks/todo.md`
  - 记录：已复习阶段完成后验证并提交、方案/清单分离、Hermeship 不是 thin adapter 的 lessons。

- [x] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 完成标准：确认仍在 `codex/milestone-1-cli`，并识别是否存在未提交变更。
  - 记录：当前分支为 `codex/milestone-1-cli`；工作树启动时干净；最近提交为 `9d8b05c docs: 更新 Hermeship Milestone 8.2 交接入口`、`a6bd734 docs: 更新 Hermeship Milestone 8.1 交接状态`、`1536b6a feat: 增加 Git Source 本地事件路径`。

- [x] 确认 Milestone 8.2 计划。
  - 文件：`tasks/development-checklist.md`
  - 入口：`## Milestone 8：clawhip 功能 Parity 扩展`
  - 下一项：任务 8.2 GitHub Source。
  - 记录：本阶段只实现 GitHub Source 的本地 deterministic parity：issue、PR、CI/check、release 事件、CLI、typed conversion、route metadata、默认安全渲染和 fixture/release preflight 覆盖。

- [x] 阅读 Milestone 8.2 相关代码和 fixture 规则。
  - `src/cli.rs`
  - `src/main.rs`
  - `src/config.rs`
  - `src/events.rs`
  - `src/event/mod.rs`
  - `src/event/body.rs`
  - `src/event/compat.rs`
  - `src/source/git.rs`
  - `src/router.rs`
  - `src/render/mod.rs`
  - `src/render/default.rs`
  - `src/dispatch.rs`
  - `src/lifecycle.rs`
  - `src/release_preflight.rs`
  - `tests/fixtures/README.md`
  - 方案文档中 CLI、source/parity、测试矩阵和发布章节。
  - 关注点：复用 Milestone 8.1 Git Source 的模块边界、输入校验、route metadata、renderer raw 安全边界和 release preflight 公开命令覆盖方式。

- [x] 写失败测试：GitHub source 事件构造。
  - 新建：`src/source/github.rs`
  - 覆盖：issue、pull request、CI/check、release 事件的本地 deterministic `IncomingEvent` 构造。
  - 隐私边界：不包含完整 issue/PR body、token、cookie、secret 或外部 API 响应正文。
  - 记录：已覆盖 source 构造和输入校验；不访问真实 GitHub API。

- [x] 写失败测试：typed conversion 与 route metadata。
  - 修改：`src/event/body.rs`
  - 修改：`src/event/mod.rs`
  - 修改：`src/event/compat.rs`
  - 覆盖：GitHub 事件进入 typed `EventEnvelope`，metadata 可用于 repo、owner、branch、number、status 等 route filter。
  - 记录：已覆盖 issue/PR/check/release typed body、直接 POST 输入校验和 route metadata poisoning 回归。

- [x] 写失败测试：CLI 解析与 daemon submit。
  - 修改：`src/cli.rs`
  - 修改：`src/main.rs`
  - 修改：`tests/fixtures/cli/public_commands.txt`
  - 覆盖：`hermeship github ...` 子命令能 parse，并通过 `DaemonClient::post_event()` 投递 `/event`。
  - 记录：已覆盖 `github issue-opened`、`github pr-opened`、`github check-failed`、`github release-published` 解析和 issue 命令 daemon submit。

- [x] 写失败测试：默认 renderer。
  - 修改：`src/render/default.rs`
  - 覆盖：compact/inline/alert/raw 输出安全 GitHub 摘要，不泄漏完整 body 或 secret。
  - 记录：已覆盖 compact 与 raw；raw 不输出 URL、body、token、secret 或 provider response。

- [x] 运行 Red 验证。
  - 命令：`cargo test github`
  - 预期：实现前失败于缺少 `source::github` API、CLI 子命令和 GitHub typed event variants。
  - 记录：Red 已确认，失败点为缺少 `source::github` API、`GithubCommands`、`Commands::Github` 和 GitHub typed body variants；review 回归测试也先失败于 route metadata poisoning 和 docs preflight 覆盖缺口。

- [x] 实现最小本地 deterministic GitHub Source。
  - 只构造 Hermeship 自己的 `IncomingEvent`/`EventEnvelope`。
  - 不调用 clawhip runtime。
  - 不访问真实 GitHub API。
  - 不依赖外网、真实 GitHub token 或 webhook secret。
  - 记录：已实现 source/typed/CLI/router/render/preflight；未新增网络调用或凭据处理。

- [x] 运行 Milestone 8.2 验证。
  - `cargo test github`
  - `cargo test release_preflight`
  - `cargo run -- release preflight 0.1.0`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
  - 记录：上述命令均已通过；release preflight 仍将 live verification 标记为 pending。

- [x] 更新开发状态并提交。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 更新：`docs/development-status.md`
  - commit 信息使用中文，说明变更、验证和影响。
  - 记录：已更新开发清单、当前工作台和开发状态；本记录随本阶段提交完成。

## Review

- Milestone 8.3 Tmux Source 本地 deterministic parity 已实现并提交：`3745bb8 feat: 增加 tmux 事件 source`。
- 已新增 `src/source/tmux.rs`，提供 keyword/stale 的本地 `IncomingEvent` 构造，以及 watch/list 对 fake tmux 输出的 deterministic 解析和报表；本阶段不调用真实 `tmux`、不读取真实 session、不启动真实 watch loop。
- 已新增 typed tmux body，并让 `tmux.keyword` / `tmux.stale` 进入现有 `IncomingEvent -> EventEnvelope -> Router -> Renderer -> Sink` 管线；`tmux.stale` 使用 high priority。
- 已接入 CLI：`hermeship tmux keyword`、`hermeship tmux stale`、`hermeship tmux watch`、`hermeship tmux list`，并更新公开命令 fixture、README 示例与 release preflight 检查。
- 默认 renderer 输出 tmux 安全摘要；raw JSON 不展开 pane capture、buffer、完整 pane output、history、token、cookie 或 secret。
- 已根据代码审查收紧 watch/list 报表：不再原样输出 command 或 last_line，只显示 command 是否存在和 last_line 字符数，并补充 token/path/authorization 回归测试。
- 已验证：`cargo test tmux`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 当前交接下一入口：Milestone 8.4 Cron 与 Memory Scaffold；继续默认使用本地 deterministic fixture。
- 未进入范围：真实 tmux session 读取、真实 tmux watch、cron/memory 以外能力、live verification、Slack sink、Hermes plugin/observer。

- Milestone 8.2 GitHub Source 本地 deterministic parity 已实现并提交：`91d13d8 feat: 完成 GitHub Source 本地确定性路径并修复回归`。
- 已新增 `src/source/github.rs`，提供 issue、PR、check/CI、release 的本地 `IncomingEvent` 构造；本阶段不访问真实 GitHub API、不依赖外网、不读取 token 或 webhook secret。
- 已新增 typed GitHub body，并让 `github.issue-opened`、`github.pr-opened`、`github.check-failed`、`github.release-published` 进入现有 `IncomingEvent -> EventEnvelope -> Router -> Renderer -> Sink` 管线。
- 已接入 CLI：`hermeship github issue-opened`、`hermeship github pr-opened`、`hermeship github check-failed`、`hermeship github release-published`，并更新公开命令 fixture 与 release preflight 检查。
- 默认 renderer 输出 GitHub 安全摘要；raw JSON 不展开完整 issue/PR body、URL、provider response、token、cookie 或 secret。
- 已根据代码审查补充并修复 GitHub route metadata poisoning 回归：router filter 中的 `repo_name` 使用 typed body 的已校验 repo 覆盖原始 payload metadata。
- 已验证：`cargo test github`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 当时交接下一入口：Milestone 8.3 Tmux Source；继续默认使用本地 deterministic fixture，不依赖真实 tmux session。
- 未进入范围：真实 GitHub API source、GitHub webhook receiver、GitHub credential handling、真实 git polling source、tmux source、cron、memory、live verification、Slack sink、Hermes plugin/observer。

- Milestone 8.1 Git Source 本地 deterministic parity 已完成并提交：`1536b6a feat: 增加 Git Source 本地事件路径`。
- 已完成 `src/source/git.rs`，提供 `git.commit` 和 `git.branch-changed` 的 `IncomingEvent` 构造；本阶段不执行真实 `git`、不轮询 repo、不访问远端。
- 已新增 typed Git body，并让 `git.commit` / `git.branch-changed` 进入现有 `IncomingEvent -> EventEnvelope -> Router -> Renderer -> Sink` 管线。
- 已接入 CLI：`hermeship git commit` 与 `hermeship git branch-changed`，并更新公开命令 fixture 与 release preflight 检查。
- 默认 renderer 输出 Git 安全摘要；raw JSON 不展开完整 diff、commit body、repo path、worktree path 或 author email。
- 已验证：`cargo test git`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 当时交接已将下一入口切到 Milestone 8.2 GitHub Source；8.2 已在本阶段完成。
- 未进入范围：真实 GitHub API source、真实 git polling source、tmux source、cron、memory、live verification、Slack sink、Hermes plugin/observer。
