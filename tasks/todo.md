# Task: Milestone 3.2 - Event ingress 与队列

更新时间：2026-06-16 00:01:45 CST

本阶段目标：在已完成 daemon `/health` 与 `DaemonClient` 的基础上，建立 Hermeship daemon 的通用事件入口和内存队列。实现 `/event` HTTP ingress、`IncomingEvent -> EventEnvelope` conversion、入队前隐私清洗、`hermeship emit` 与 `hermeship send` 的 client 投递路径，为后续 Hermes hook ingress、router、renderer、dispatcher 和 sink 做准备。

本阶段边界：只实现通用 event ingress、队列入队、`emit`/`send` 到 daemon 的 client 路径；不实现 Hermes hook ingress、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。隐私 sanitizer 必须在事件入队前接入，默认测试仍只使用本地 deterministic fixture。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 最新功能阶段提交：`ff5c589 feat: 增加 hermeship daemon health`。
- 已完成 Milestone 3.1：`src/daemon.rs`、`src/client.rs`、typed `HealthResponse`、`QueueHealth`、daemon `/health`、daemon listener、`hermeship start`、`hermeship status`。
- 已完成事件前置能力：`IncomingEvent`、`EventEnvelope`、Hermes canonical mapping、privacy sanitizer、Hermes 合成 fixture。
- 当前 `send`、`emit`、`explain`、`hermes hook` 仍按各自 milestone 保持 placeholder；任务 3.2 只替换 `send` 和 `emit` 的 daemon event client 路径，不接入 `explain`、Hermes hook、路由或投递。

## 执行计划

- [ ] 复习项目规则与状态入口。
  - 阅读：`tasks/lessons.md`
  - 阅读：`docs/development-status.md`
  - 阅读：`docs/plans/2026-06-15-hermeship-development-plan.md`
  - 阅读：`tasks/development-checklist.md`
  - 阅读：`tasks/todo.md`

- [ ] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 预期：当前分支为 `codex/milestone-1-cli`；最新功能阶段提交为 `ff5c589 feat: 增加 hermeship daemon health`；启动时不要混入无关改动。

- [ ] 检查现有代码边界。
  - 查看：`src/cli.rs`
  - 查看：`src/main.rs`
  - 查看：`src/config.rs`
  - 查看：`src/client.rs`
  - 查看：`src/daemon.rs`
  - 查看：`src/events.rs`
  - 查看：`src/event/mod.rs`
  - 查看：`src/event/body.rs`
  - 查看：`src/event/compat.rs`
  - 查看：`src/privacy.rs`
  - 查看：`tests/fixtures/README.md`
  - 完成标准：确认任务 3.2 只接入 `/event`、队列和 `send`/`emit`，不进入后续 route/render/dispatch/sink。

- [ ] 先写失败测试。
  - 修改：`src/daemon.rs`
  - 修改：`src/client.rs`
  - 必要时修改：`src/main.rs`
  - 覆盖：有效事件入队、入队前隐私清洗、非法 JSON/非法 event 返回清晰错误、daemon unavailable 时 client 返回清晰错误、`send` 构造 custom event 后 POST `/event`。
  - 命令：`cargo test daemon`
  - 预期：实现前测试失败于缺少 `/event` route、队列状态和 client POST event 能力。

- [ ] 实现 daemon `/event`。
  - 接收：`IncomingEvent` JSON。
  - 处理：用 `privacy::sanitize_payload()` 清洗 payload，再转为 typed `EventEnvelope`。
  - 入队：写入本地 `tokio::mpsc` queue。
  - 返回：事件 id、canonical kind、queued 状态和 queue 状态摘要。
  - 当前阶段只入队，不消费、不路由、不渲染、不投递。

- [ ] 实现 daemon client event POST。
  - 在 `src/client.rs` 增加 POST `/event` 方法。
  - 成功时解析 typed response。
  - daemon 不可用、非 2xx、无效响应时返回清晰错误。

- [ ] 实现 `hermeship emit`。
  - 复用 `EventArgs::into_event()`。
  - 通过 `DaemonClient` POST `/event`。
  - 打印 queued event 摘要。

- [ ] 实现 `hermeship send`。
  - 复用 `IncomingEvent::custom(channel, message)`。
  - 通过 `DaemonClient` POST `/event`。
  - 本阶段不实现 sink delivery。

- [ ] 编写 ingress 测试。
  - 覆盖：有效 Hermes fixture 入队。
  - 覆盖：敏感 payload 入队前清洗，不泄漏 token、cookie、secret、完整 message/response。
  - 覆盖：队列状态在 health response 中可见。
  - 覆盖：daemon unavailable client 错误。
  - 要求：使用随机端口和本地 test queue，不绑定固定端口，不依赖外部服务。

- [ ] 运行任务 3.2 验证命令。
  - `cargo test daemon`
  - `cargo test event`
  - `cargo run -- emit hermes.agent.started --payload '{"session_id":"demo"}'`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

- [ ] 更新开发状态文档。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 必要时更新：`docs/development-status.md`
  - 完成标准：记录实现、验证、边界和剩余风险。

- [ ] 提交任务 3.2。
  - commit：`feat: 增加 daemon event ingress`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- 待任务 3.2 实施、验证和提交后填写。
- 上一阶段 Milestone 3.1 已完成并提交：`ff5c589 feat: 增加 hermeship daemon health`。
