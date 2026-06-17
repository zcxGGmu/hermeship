# Task: Milestone 9.2 Live Verification Runbook - 完成记录

更新时间：2026-06-17

本文件是当前开发工作台。Milestone 9.2 Live Verification Runbook 已完成并提交，下一入口为 Milestone 9.3 首次 Live Check。Hermeship 仍然是 Hermes-native daemon-first event router，不调用 clawhip runtime，不依赖运行中的 clawhip daemon。

本阶段边界：创建/补齐 `docs/live-verification.md` runbook；默认不执行真实 Discord/Hermes live verification，不实现 Slack sink，不启动 Hermes plugin/observer，除非清单、凭据可用性和用户确认范围明确允许。下一入口为 Milestone 9.3 首次 Live Check。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 当前工作树：本阶段提交完成后应为干净；下次启动必须以 `git status --short --branch` 为准。
- 最新交接提交：`252ad6a docs: 更新 Hermeship Milestone 9.2 交接入口`。
- 最新文档阶段提交：本阶段提交，标题为 `docs: 增加 live verification runbook`；下次启动后用 `git log -3 --oneline` 确认实际 hash。
- 最新功能阶段提交：`0b12de3 feat: 增加 cron 与 memory scaffold`。
- 最近提交基线：本阶段提交、`252ad6a docs: 更新 Hermeship Milestone 9.2 交接入口`、`1c52655 docs: 增加 Hermeship 运维与事件契约`。
- Milestone 0 到 Milestone 8.4 已完成。
- Milestone 9.1 已完成。
- Milestone 9.2 已完成，Milestone 9.3 和 Milestone 10 未完成。
- 下一入口：Milestone 9.3 首次 Live Check。

## 当前执行计划

- [x] 复习 lessons 与交接状态。
  - `tasks/lessons.md`
  - `docs/development-status.md`
  - `docs/plans/2026-06-15-hermeship-development-plan.md`
  - `tasks/development-checklist.md`
  - `tasks/todo.md`
  - 记录：已确认阶段完成后必须验证并提交、方案/清单分离、Hermeship 不是 thin adapter；本阶段只做 Milestone 9.2 runbook，真实 live check 默认不执行。

- [x] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 记录：当前分支为 `codex/milestone-1-cli`；启动时工作树干净；最近提交为 `252ad6a docs: 更新 Hermeship Milestone 9.2 交接入口`、`1c52655 docs: 增加 Hermeship 运维与事件契约`、`3852e60 docs: 更新 Hermeship Milestone 9 交接状态`。

- [x] 确认 Milestone 9.2 计划。
  - 文件：`tasks/development-checklist.md`
  - 入口：`## Milestone 9：文档与 Live Verification`
  - 下一项：任务 9.2 Live Verification Runbook。
  - 记录：`docs/live-verification.md` 需要包含 fake sink、daemon health、Discord live、Hermes gateway hook smoke、rollback、commit、时间、测试频道、触发事件、实际消息形态、未执行项和剩余风险字段。

- [x] 写入本轮实施计划。
  - 文件：`tasks/todo.md`
  - 计划边界：只做 runbook 和本地 deterministic 文档验证；真实 Discord/Hermes live check 是否执行，必须由凭据可用性和用户确认范围决定。
  - 文件结构：
    - 新建：`docs/live-verification.md`，作为 live verification runbook 和结果记录模板。
    - 可能更新：`tasks/development-checklist.md`，勾选 Milestone 9.2 并补运行状态日志。
    - 更新：`tasks/todo.md` Review，记录 Milestone 9.2 结果。
    - 复核：`src/release_preflight.rs`，确认 live verification pending/pass 字段要求。
  - 验证计划：
    - `rg -n "HERMES_HOME|Discord|hermeship status|agent:start|rollback" docs/live-verification.md`
    - `cargo test release_preflight`
    - `cargo run -- release preflight 0.1.0`
    - `cargo fmt --all -- --check`
    - `cargo clippy --all-targets -- -D warnings`
    - `cargo test`
  - 记录：本轮实施计划已写入当前工作台；实现前已完成 check-in。

- [x] 创建 `docs/live-verification.md`。
  - 记录：已新增 live verification runbook，覆盖 scope、safety rules、result fields、preconditions、runbook 和当前未执行真实 live check 的结果记录。
- [x] 记录 fake sink、daemon health、Discord live、Hermes gateway hook smoke 和 rollback 的 runbook 字段。
  - 记录：已包含 fake sink 本地闭环、`hermeship status` daemon health、Discord live、隔离 `HERMES_HOME` Hermes gateway hook smoke 和 rollback 步骤。
- [x] 明确真实 live check 未执行时的原因和剩余风险。
  - 记录：当前未提供 Discord credentials、测试频道、Hermes gateway 测试环境和显式执行确认；真实 Discord/Hermes live check 留到 Milestone 9.3，剩余风险已写入 `docs/live-verification.md`。
- [x] 运行 Milestone 9.2 验证命令。
  - 记录：`rg -n "HERMES_HOME|Discord|hermeship status|agent:start|rollback" docs/live-verification.md`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 均已通过。
- [x] 更新开发清单运行日志和本文件 Review。
  - 记录：已更新 `tasks/development-checklist.md` 的 Milestone 9.2 checkbox 和运行状态日志；本文件 Review 已记录本阶段结果。
- [x] 提交 Milestone 9.2。
  - 记录：本阶段提交标题为 `docs: 增加 live verification runbook`，提交信息说明 runbook 变更、验证结果和影响。

## Review

- Milestone 9.2 Live Verification Runbook 已完成：新增 `docs/live-verification.md`，覆盖 fake sink、daemon health、Discord live、Hermes gateway hook smoke、rollback、安全规则和结果字段。
- 已根据文档审查修正 runbook 可执行性：前台阻塞的 `hermeship start` 已拆分为 Terminal A/B，rollback 增加 `HOOK.yaml`/`handler.py` 残留检查，Current Results 已按 fake sink、daemon health、Discord live、Hermes hook smoke 和 rollback 分项记录。
- 已记录当前真实 live check 未执行：缺少 Discord credentials、测试频道、Hermes gateway 测试环境和显式执行确认；剩余风险进入 Milestone 9.3。
- 已更新 `README.md`、`docs/development-status.md` 和 `tasks/development-checklist.md`，将当前状态推进到 Milestone 9.2 已完成，下一入口为 Milestone 9.3 首次 Live Check。
- 已验证：`rg -n "HERMES_HOME|Discord|hermeship status|agent:start|rollback" docs/live-verification.md`、`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（live verification 字段检查 ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 本阶段提交标题：`docs: 增加 live verification runbook`；下次启动后用 `git log -3 --oneline` 确认实际 hash。
- 未完成：真实 Discord/Hermes live verification、Slack sink、Hermes plugin/observer。

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
