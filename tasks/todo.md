# Task: Milestone 2.1 - IncomingEvent 与格式

启动时间：下次开发会话启动时确认

本阶段目标：建立 Hermeship 的第一版事件入口模型和 `emit` 解析路径，不进入 daemon、typed envelope、privacy、router、renderer、sink、hook bridge、install 或 release preflight 实现。

- [ ] 复习 `tasks/lessons.md`，确认阶段完成后必须验证并提交。
- [ ] 复习阶段上下文。
  - 阅读：`docs/development-status.md`
  - 阅读：`docs/plans/2026-06-15-hermeship-development-plan.md`
  - 阅读：`tasks/development-checklist.md`
  - 阅读：`tasks/todo.md`
- [ ] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 预期：当前分支为 `codex/milestone-1-cli`；最新已完成阶段提交为 `70c8f03 chore: 增加 Rust 质量门禁与仓库基础`；启动时不要混入无关改动。
- [ ] 明确本阶段边界。
  - 只处理 `IncomingEvent`、`RoutingMetadata`、`MessageFormat` 复用策略、`emit` 参数解析和基础 Hermes fixture。
  - 不实现 daemon、client、typed `EventEnvelope`、隐私清洗、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
- [ ] 检查现有代码和参考实现。
  - 查看：`src/config.rs`
  - 查看：`src/cli.rs`
  - 查看：`src/lib.rs`
  - 查看：`/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/events.rs`
  - 查看：`/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/event/compat.rs`
  - 完成标准：确认当前 `MessageFormat` 的单一定义/复用方式，并明确事件入口与后续 typed event 的边界。
- [ ] 先写失败测试。
  - 新建或修改：`src/events.rs`
  - 修改：`src/cli.rs`
  - 覆盖：payload JSON 合并、非法 format、奇数 key/value 拒绝、字段别名、空 payload。
  - 命令：`cargo test events`
  - 命令：`cargo test cli`
- [ ] 新增 Hermes 事件 fixture。
  - 新建：`tests/fixtures/hermes/agent_start.json`
  - 新建：`tests/fixtures/hermes/session_end.json`
  - 新建：`tests/fixtures/hermes/invalid_payload.json`
  - 完成标准：fixture 只使用合成脱敏样例，不包含真实 token、cookie、secret、完整 prompt、完整对话或 provider request/response body。
- [ ] 实现最小事件入口。
  - 新增：`IncomingEvent`
  - 新增：`RoutingMetadata`
  - 复用或重导出当前 `MessageFormat`，避免产生两套不一致 enum。
  - 将 `hermeship emit` 的 `--payload`、`--channel`、`--mention`、`--format`、`--template` 和 key/value 参数接入真实事件构造路径。
- [ ] 运行任务 2.1 验证命令。
  - `cargo test events`
  - `cargo test cli`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- [ ] 更新 `tasks/development-checklist.md`。
  - 勾选任务 2.1 已完成项。
  - 在运行状态日志顶部记录本阶段实现、验证和提交状态。
- [ ] 更新 `tasks/todo.md` Review。
  - 记录实现、验证、边界和剩余风险。
- [ ] 提交任务 2.1。
  - commit：`feat: 增加 IncomingEvent 事件入口`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- 待任务 2.1 实施、验证和提交后填写。
