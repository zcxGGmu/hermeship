# Task: Milestone 3.1 - Daemon health 与 client

更新时间：2026-06-15 23:35:35 CST

本阶段目标：在已完成 `IncomingEvent -> EventEnvelope` 和 privacy 清洗纯逻辑的基础上，建立 Hermeship daemon 的最小 health/status 闭环。实现本地 daemon health endpoint、daemon client health 查询和 `hermeship status` 的真实逻辑，为后续 `/event` ingress 与队列做准备。

本阶段边界：只实现 daemon health/status 与 client health 查询；不实现 event ingress、HTTP `/event`、Hermes hook ingress、队列入队、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。privacy sanitizer 在后续 ingress/daemon 事件入队前接入，任务 3.1 不扩大到事件处理。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 最新功能阶段提交：`175009d feat: 增加 Hermes 事件隐私清洗`。
- 已完成 Milestone 2.3：`src/privacy.rs`、`sanitize_payload`、`redact_value`、`excerpt_policy`、隐私 fixture 与 10 个 privacy 回归测试。
- `src/main.rs` 中 `start` 和 `status` 已替换为真实 health/status 行为；后续 `send`、`emit`、`explain`、`hermes hook` 等仍按对应 milestone 保持 placeholder。

## 执行计划

- [x] 复习项目规则与状态入口。
  - 阅读：`tasks/lessons.md`
  - 阅读：`docs/development-status.md`
  - 阅读：`docs/plans/2026-06-15-hermeship-development-plan.md`
  - 阅读：`tasks/development-checklist.md`
  - 阅读：`tasks/todo.md`

- [x] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 预期：当前分支为 `codex/milestone-1-cli`；最新功能阶段提交为 `175009d feat: 增加 Hermes 事件隐私清洗`；启动时不要混入无关改动。

- [x] 检查现有代码边界。
  - 查看：`src/cli.rs`
  - 查看：`src/main.rs`
  - 查看：`src/config.rs`
  - 查看：`src/lib.rs`
  - 查看：`src/events.rs`
  - 查看：`src/event/mod.rs`
  - 查看：`src/event/body.rs`
  - 查看：`src/event/compat.rs`
  - 查看：`src/privacy.rs`
  - 查看：`tests/fixtures/README.md`
  - 完成标准：确认任务 3.1 只替换 `start`/`status` placeholder，不接入 event ingress 或路由投递。

- [x] 先写失败测试。
  - 新建或修改：`src/daemon.rs`
  - 新建或修改：`src/client.rs`
  - 必要时修改：`src/main.rs`
  - 覆盖：health response schema、默认 host/port、configured sinks 摘要、daemon unavailable 时 client 返回清晰错误、`status` 不 panic。
  - 命令：`cargo test daemon`
  - 预期：实现前测试失败于缺少 daemon/client 模块或真实 health/status 逻辑。

- [x] 新建 daemon/client 模块。
  - 新建：`src/daemon.rs`
  - 新建：`src/client.rs`
  - 修改：`src/lib.rs`
  - 导出：`hermeship::daemon`、`hermeship::client`

- [x] 实现 daemon health。
  - `hermeship start` 使用 `AppConfig.daemon` 默认监听 `127.0.0.1:25295`，支持 CLI `--port` 覆盖。
  - `/health` 返回 version、status、queue 状态、configured sinks。
  - 当前阶段只暴露 health endpoint，不实现 `/event` 或队列消费。

- [x] 实现 daemon client 与 status CLI。
  - `client` 使用 `DaemonConfig::base_url()` 构造 health URL。
  - `hermeship status` 调用 `/health` 并打印可读摘要。
  - daemon 不可用时返回清晰错误；命令可以非 0 退出，但不能 panic。

- [x] 运行任务 3.1 验证命令。
  - `cargo test daemon`
  - `cargo run -- status`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

- [x] 更新开发状态文档。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 必要时更新：`docs/development-status.md`
  - 完成标准：记录实现、验证、边界和剩余风险。

- [x] 提交任务 3.1。
  - commit：`feat: 增加 hermeship daemon health`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- 已完成 Milestone 3.1 的 daemon health/status 最小闭环。
- 新增 `src/daemon.rs`：typed `HealthResponse`、`QueueHealth`、configured sinks 摘要、`/health` Axum route、listener 绑定和 daemon serve 入口。
- 新增 `src/client.rs`：daemon health client、base URL 规范化、2 秒 timeout、daemon unreachable 和非 2xx 错误摘要。
- 更新 `src/main.rs`：`hermeship start` 加载配置并启动 daemon；`hermeship status` 调用 `/health` 并打印 version、status、queue 和 configured sinks。
- 已确认本阶段未实现 event ingress、`/event`、Hermes hook ingress、队列入队、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
- Red 验证：实现前 `cargo test daemon` 失败于缺少 `DaemonClient`、`HealthResponse`、`bind_listener` 和 `serve_listener`。
- Green/门禁验证：`cargo test daemon` 4 passed；`cargo run -- status` 在 daemon 未运行时返回 `daemon is not reachable at http://127.0.0.1:25295/health` 且无 panic；`cargo fmt --all -- --check` 通过；`cargo clippy --all-targets -- -D warnings` 通过；`cargo test` 45 passed。
- 下一步：Milestone 3.2 实现 `/event` ingress 与队列；仍不进入 router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
