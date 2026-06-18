# Task: Milestone 10 Hermes Plugin / Observer 研究入口

更新时间：2026-06-18

本轮任务是按用户要求进入 Milestone 10。由于本轮仍未提供 Discord credentials、测试频道、Hermes gateway 测试环境和明确执行 live check 的条件，用户的“先进入 10”记录为：**真实 live pass 被用户豁免，Milestone 10 门禁解除**。

该豁免不代表真实 Discord/Hermes live verification 已通过；`docs/live-verification.md` 中 Milestone 9.3 的 `blocked`/`not_run` 记录仍然成立。Slack sink 仍不在当前默认范围内。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 启动时工作树：干净，`git status --short --branch` 只显示分支行。
- 当前 HEAD：`0d0d354 docs: 记录 Hermeship 本地验证续接状态`。
- 最近 5 个提交：`0d0d354`、`92790ef`、`589c9e2`、`3f2e758`、`9602856`。
- 最新状态文档提交：`92790ef docs: 更新 Hermeship 最新开发状态与下次启动提示词`。
- 最新状态续接提交：`0d0d354 docs: 记录 Hermeship 本地验证续接状态`。
- 最新 live 记录提交：`bc4c027 docs: 记录 Hermeship live verification 结果`。
- 最新功能阶段提交：`0b12de3 feat: 增加 cron 与 memory scaffold`。
- Milestone 9.3：blocked/not_run 记录已完成；真实 live pass 被用户豁免用于解除 Milestone 10 门禁。
- Milestone 10：已解锁，进入 Hermes plugin / observer 研究入口。

## 当前执行计划

- [x] 记录真实 live pass 豁免决策。
  - 触发：用户明确要求“先进入 10”。
  - 记录：`docs/development-status.md`。
  - 记录：`tasks/development-checklist.md`。
  - 边界：不新增 `docs/live-verification.md` pass 结果，不把 release preflight 解释成真实 live pass。

- [x] 复核 Milestone 10 现有清单。
  - 已读：`tasks/development-checklist.md` 的 Milestone 10。
  - 结论：先做 10.1 Observer 契约研究，再决定 10.2 Observer Plugin MVP 代码实现。

- [x] 初步阅读 Hermes plugin loader。
  - 已读：`/Users/zq/Desktop/ai-projs/posp/agents-contributions/hermes-agent/hermes_cli/plugins.py`。
  - 初步确认：Hermes 支持目录 plugin、user plugin、project plugin、pip entry point；目录 plugin 需要 `plugin.yaml` 和 `__init__.py` 的 `register(ctx)`；hooks 包括 `pre_tool_call`、`post_tool_call`、`pre_llm_call`、`post_llm_call`、`api_request_error`、`on_session_start`、`on_session_end`、`subagent_start`、`subagent_stop` 等。

- [x] 设计 check-in。
  - 推荐方案：先完成 10.1 Observer 契约研究，输出 `docs/observer-plugin.md`，锁定事件 mapping、隐私边界、安装启用方式、fail-open 行为和本地测试策略；暂不写 plugin 代码。
  - 确认：用户回复 `ok` 后，按“10.1 研究文档优先，然后再实现 10.2 plugin scaffold”的顺序推进。

- [x] 完成 Milestone 10.1 Observer 契约研究。
  - 新增：`docs/observer-plugin.md`。
  - 范围：plugin discovery、`plugins.enabled`/`plugins.disabled`、observer hook mapping、safe fields、fail-open、`POST /event` ingress、Milestone 10.2 follow-up。
  - 边界：10.1 不创建 plugin 模板，不新增 Rust typed observer event bodies，不改变 release preflight。

- [x] 阶段验证与提交准备。
  - 默认验证：`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
  - 已验证：`rg -n "hermes.observer|plugins.enabled|pre_tool_call|post_tool_call|api_request_error|subagent_start|fail-open|request body|response body" docs/observer-plugin.md`。
  - 已验证：`git diff --check`。
  - 已验证：`cargo test release_preflight`（12 passed；bin 侧筛选 0 tests）。
  - 已验证：`cargo run -- release preflight 0.1.0`（8 checks ok；不证明真实 live pass）。
  - 已验证：`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed；doc tests 0）。
  - 提交信息：`docs: 完成 Hermes observer plugin 契约研究`，正文说明变更、验证和影响。

- [x] 提交 Milestone 10.1 文档。
  - 提交前检查：`git status --short --branch`、`git diff --stat`、`git diff --name-only`。
  - 提交后检查：`git status --short --branch`、`git log -5 --oneline`。

## Review

- 已记录用户豁免 Milestone 9.3 真实 live pass，用于解除 Milestone 10 门禁。
- 已更新 `docs/development-status.md` 和 `tasks/development-checklist.md` 的当前状态与运行日志。
- 已将本文件切换为 Milestone 10 Hermes plugin / observer 研究入口。
- 已完成 Milestone 10.1 Observer 契约研究文档：`docs/observer-plugin.md`。
- 已完成本地验证：observer 文档关键词检查、`git diff --check`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 已确认 `release preflight` 的 `live verification` ok 仍然只代表 `docs/live-verification.md` 字段存在，不代表真实 Discord/Hermes live pass。
- 尚未修改功能代码，尚未创建 observer plugin 模板；10.2 仍待后续执行。
