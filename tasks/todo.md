# Task: Milestone 2.1 - IncomingEvent 与格式

启动时间：2026-06-15 本次开发会话

本阶段目标：建立 Hermeship 的第一版事件入口模型和 `emit` 解析路径，不进入 daemon、typed envelope、privacy、router、renderer、sink、hook bridge、install 或 release preflight 实现。

- [x] 复习 `tasks/lessons.md`，确认阶段完成后必须验证并提交。
- [x] 复习阶段上下文。
  - 阅读：`docs/development-status.md`
  - 阅读：`docs/plans/2026-06-15-hermeship-development-plan.md`
  - 阅读：`tasks/development-checklist.md`
  - 阅读：`tasks/todo.md`
- [x] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 预期：当前分支为 `codex/milestone-1-cli`；最新已完成阶段提交为 `70c8f03 chore: 增加 Rust 质量门禁与仓库基础`；启动时不要混入无关改动。
- [x] 明确本阶段边界。
  - 只处理 `IncomingEvent`、`RoutingMetadata`、`MessageFormat` 复用策略、`emit` 参数解析和基础 Hermes fixture。
  - 不实现 daemon、client、typed `EventEnvelope`、隐私清洗、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
- [x] 检查现有代码和参考实现。
  - 查看：`src/config.rs`
  - 查看：`src/cli.rs`
  - 查看：`src/lib.rs`
  - 查看：`/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/events.rs`
  - 查看：`/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/event/compat.rs`
  - 完成标准：确认当前 `MessageFormat` 的单一定义/复用方式，并明确事件入口与后续 typed event 的边界。
  - 记录：当前 Hermeship 的 `MessageFormat` 已定义在 `src/config.rs`；本阶段采用 `src/events.rs` 重导出该类型并复用同一 enum，避免重复定义。
- [x] 先写失败测试。
  - 新建或修改：`src/events.rs`
  - 修改：`src/cli.rs`
  - 覆盖：payload JSON 合并、非法 format、奇数 key/value 拒绝、字段别名、空 payload。
  - 命令：`cargo test events`
  - 命令：`cargo test cli`
- [x] 新增 Hermes 事件 fixture。
  - 新建：`tests/fixtures/hermes/agent_start.json`
  - 新建：`tests/fixtures/hermes/session_end.json`
  - 新建：`tests/fixtures/hermes/invalid_payload.json`
  - 完成标准：fixture 只使用合成脱敏样例，不包含真实 token、cookie、secret、完整 prompt、完整对话或 provider request/response body。
- [x] 实现最小事件入口。
  - 新增：`IncomingEvent`
  - 新增：`RoutingMetadata`
  - 复用或重导出当前 `MessageFormat`，避免产生两套不一致 enum。
  - 将 `hermeship emit` 的 `--payload`、`--channel`、`--mention`、`--format`、`--template` 和 key/value 参数接入真实事件构造路径。
- [x] 运行任务 2.1 验证命令。
  - `cargo test events`
  - `cargo test cli`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- [x] 更新 `tasks/development-checklist.md`。
  - 勾选任务 2.1 已完成项。
  - 在运行状态日志顶部记录本阶段实现、验证和提交状态。
- [x] 更新 `tasks/todo.md` Review。
  - 记录实现、验证、边界和剩余风险。
- [x] 提交任务 2.1。
  - commit：`feat: 增加 IncomingEvent 事件入口`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- 已完成 Milestone 2.1 的入口事件层：新增 `src/events.rs`，实现 `IncomingEvent`、`RoutingMetadata`、字段别名反序列化、缺省/null payload 归一为空对象，以及 `tests/fixtures/hermes/` 下的合成 Hermes payload fixture。
- 已采用单一 `MessageFormat` 策略：`src/config.rs` 保留唯一 enum 定义并新增 `from_label()`，`src/events.rs` 只做重导出，后续 route/config/event 比较不会出现两套格式类型。
- 已将 `hermeship emit` 和 `hermeship explain` 的参数解析接入 `EventArgs::into_event()`，支持 `--payload`、`--channel`、`--mention`、`--format`、`--template`、任意 `--key value`、JSON typed value 和 `--agent`/`--session`/`--elapsed`/`--error` 别名；非法 format、奇数 key/value、缺少 `--` 前缀会返回错误。
- Red/Green 记录：实现前 `cargo test events` / `cargo test cli` 失败于缺少 `events::MessageFormat`、`IncomingEvent`、`RoutingMetadata` 和 `EventArgs::into_event`；实现后目标测试通过。
- 已验证：`cargo test events`、`cargo test cli`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 均通过。
- 边界保持：本阶段未实现 daemon、client、typed `EventEnvelope`、privacy 清洗、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
