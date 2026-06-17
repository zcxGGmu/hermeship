# Task: Milestone 9.3 Live Check Handoff - 下一入口

更新时间：2026-06-17

本文件是当前开发工作台。Milestone 9.2 Live Verification Runbook 已完成并提交，当前任务是更新交接状态，确保下次启动可直接从 Milestone 9.3 首次 Live Check 继续。

Hermeship 仍然是 Hermes-native daemon-first event router，不调用 clawhip runtime，不依赖运行中的 clawhip daemon。默认测试只使用本地 deterministic fixture；真实 Discord/Hermes live check 必须以凭据可用性和用户确认范围为准。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 当前工作树：本次交接提交完成后应为干净；下次启动必须以 `git status --short --branch` 为准。
- 最新文档阶段提交：`2e60902 docs: 增加 live verification runbook`。
- 最新交接基线提交：`252ad6a docs: 更新 Hermeship Milestone 9.2 交接入口`。
- 最新功能阶段提交：`0b12de3 feat: 增加 cron 与 memory scaffold`。
- Milestone 0 到 Milestone 8.4 已完成并提交。
- Milestone 9.1 已完成并提交：README、operations、event contract 和 architecture 已对齐。
- Milestone 9.2 已完成并提交：`docs/live-verification.md` runbook 已创建。
- Milestone 9.3 未完成：首次真实 Discord/Hermes live check 尚未执行。
- Milestone 10 未完成：Hermes plugin / observer 研究尚未启动。
- 下一入口：Milestone 9.3 首次 Live Check。

## 当前执行计划

- [x] 复核当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 记录：当前分支为 `codex/milestone-1-cli`；启动时工作树干净；最近提交为 `2e60902 docs: 增加 live verification runbook`、`252ad6a docs: 更新 Hermeship Milestone 9.2 交接入口`、`1c52655 docs: 增加 Hermeship 运维与事件契约`。

- [x] 确认当前完成/未完成状态。
  - 已完成：Milestone 0 到 8.4、Milestone 9.1、Milestone 9.2。
  - 未完成：Milestone 9.3 首次 Live Check、Milestone 10 Hermes plugin / observer。
  - 明确边界：默认不执行真实 Discord/Hermes live check，不实现 Slack sink，不启动 Hermes plugin/observer，除非凭据可用且用户确认范围。

- [x] 写入本轮交接文档计划。
  - 更新：`docs/development-status.md`，把状态入口、阶段表、未完成项、下一步入口和下次启动提示词对齐到 Milestone 9.3。
  - 更新：`tasks/development-checklist.md`，增加本次交接运行状态日志。
  - 更新：`tasks/todo.md`，记录本轮交接任务和 Review。
  - 验证计划：
    - `rg -n "Milestone 9\\.3|2e60902|docs/live-verification\\.md|真实 Discord/Hermes|Hermes plugin / observer|cargo test release_preflight" docs/development-status.md tasks/todo.md tasks/development-checklist.md README.md`
    - `cargo test release_preflight`
    - `cargo run -- release preflight 0.1.0`
    - `cargo fmt --all -- --check`
    - `cargo clippy --all-targets -- -D warnings`
    - `cargo test`

- [x] 更新 `docs/development-status.md`。
  - 记录：已将状态入口、阶段表、未完成项、下一步入口和下次启动提示词对齐到 Milestone 9.3。
- [x] 更新 `tasks/development-checklist.md` 运行状态日志。
  - 记录：已增加 Milestone 9.3 下一入口交接更新，明确 9.2 已完成、9.3/10 未完成。
- [x] 运行交接验证命令。
  - 记录：交接关键词 `rg` 通过；`cargo test release_preflight` 12 passed；`cargo run -- release preflight 0.1.0` 全部 ok；`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 均通过；`cargo test` 为 194 lib tests + 15 bin tests passed。
- [x] 提交本次交接文档更新。
  - 记录：提交标题使用 `docs: 更新 Hermeship Milestone 9.3 交接状态`。

## Review

- 已更新 `docs/development-status.md`：明确 Milestone 0 到 9.2 已完成，Milestone 9.3 和 Milestone 10 未完成，下一入口为 Milestone 9.3 首次 Live Check。
- 已更新 `tasks/development-checklist.md`：新增 Milestone 9.3 下一入口交接更新。
- 已更新本文件：当前工作台切换到 Milestone 9.3 入口，记录凭据/用户确认边界。
- 已验证：`rg -n "Milestone 9\\.3|2e60902|docs/live-verification\\.md|真实 Discord/Hermes|Hermes plugin / observer|cargo test release_preflight" docs/development-status.md tasks/todo.md tasks/development-checklist.md README.md`、`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（all ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 本次交接提交标题：`docs: 更新 Hermeship Milestone 9.3 交接状态`；下次启动后用 `git log -3 --oneline` 确认实际 hash。
