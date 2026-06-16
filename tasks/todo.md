# Task: Milestone 3.3 - Hermes hook ingress

更新时间：2026-06-16 Milestone 3.3 待执行

本阶段目标：在已完成 daemon `/health`、通用 `/event` ingress、隐私清洗和队列入队的基础上，接入 Hermes gateway hook 的 ingress normalization。实现 daemon `POST /api/hermes/hook`、`hermeship hermes hook --payload` client 投递路径、stdin payload 支持，以及 Hermes hook envelope -> `IncomingEvent` 的转换。

本阶段边界：只实现 Hermes hook ingress normalization 和复用既有 `/event` 入队管道；不实现 router、renderer、dispatcher、sink、hook bridge install、install/uninstall lifecycle 或 release preflight。隐私 sanitizer 仍必须在事件入队前生效，默认测试只使用本地 deterministic fixture。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 最新功能阶段提交：`0b63e49 feat: 增加 daemon event ingress`。
- 启动时应先确认工作树状态：`git status --short --branch`。
- 已完成 Milestone 3.1：daemon `/health`、typed `HealthResponse`、`QueueHealth`、daemon listener、`hermeship start`、`hermeship status`。
- 已完成 Milestone 3.2：daemon 通用 `POST /event`、`EventAcceptedResponse`、bounded `tokio::mpsc` queue、入队前 privacy sanitizer、`DaemonClient::post_event()`、`hermeship emit`、`hermeship send`。
- 已完成事件前置能力：`IncomingEvent`、`RoutingMetadata`、typed `EventEnvelope`、Hermes canonical mapping、privacy sanitizer、Hermes 合成 fixture。
- 当前 `explain`、`hermes hook`、install、release 仍按各自 milestone 保持 placeholder；任务 3.3 只替换 `hermes hook` ingress/client 路径。
- 当前 daemon 队列只入队不消费，达到容量后 `/event` 会返回 503；dispatcher/consumer 在 Milestone 4.3 实现。

## 已完成

- [x] Milestone 0：契约与仓库基线。
- [x] Milestone 1.1：Cargo 项目与 CLI 入口。
- [x] Milestone 1.2：配置模型。
- [x] Milestone 1.3：质量门禁与仓库基础。
- [x] Milestone 2.1：IncomingEvent 与格式。
- [x] Milestone 2.2：Typed EventEnvelope。
- [x] Milestone 2.3：隐私与 payload 清洗。
- [x] Milestone 3.1：Daemon health 与 client。
- [x] Milestone 3.2：Event ingress 与队列。

## 未完成

- [ ] Milestone 3.3：Hermes hook ingress。
- [ ] Milestone 4：Router、Renderer、Dispatcher。
- [ ] Milestone 5：Discord Sink 与基础 Live Path。
- [ ] Milestone 6：Hermes Hook Bridge 安装。
- [ ] Milestone 7：安装、生命周期与运维 CLI。
- [ ] Milestone 8：clawhip 功能 Parity 扩展。
- [ ] Milestone 9：文档与 Live Verification。
- [ ] Milestone 10：Hermes Plugin / Observer 研究。

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
  - 预期：当前分支为 `codex/milestone-1-cli`；最新功能阶段提交为 `0b63e49 feat: 增加 daemon event ingress`；启动时不要混入无关改动。

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
  - 完成标准：确认任务 3.3 只接入 `/api/hermes/hook`、Hermes hook payload normalization 和 `hermeship hermes hook`，不进入 route/render/dispatch/sink/hook install。

- [ ] 先写失败测试。
  - 修改：`src/daemon.rs`
  - 修改：`src/client.rs`
  - 修改：`src/main.rs`
  - 必要时新增或修改 Hermes hook normalization helper 模块。
  - 覆盖：daemon `/api/hermes/hook` 接收 Hermes hook envelope 并入队、`hermeship hermes hook --payload` POST 到 daemon、`--payload -` 从 stdin 读取、gateway hook event 映射、隐私清洗、非法 payload 返回清晰错误、daemon unavailable 返回清晰错误。
  - 命令：`cargo test hermes`
  - 预期：实现前测试失败于缺少 `/api/hermes/hook` route、Hermes hook normalization、stdin payload 和 client POST hook 能力。

- [ ] 实现 Hermes hook envelope 模型与 normalization。
  - 接收字段：`provider`、`source`、`event`、`context`。
  - 默认 provider/source：`hermes` / `gateway`。
  - 输出：标准 `IncomingEvent`，payload 包含 provider/source/event context metadata。
  - 复用既有 Hermes canonical mapping，不重写 `EventEnvelope` 转换逻辑。

- [ ] 实现 daemon `/api/hermes/hook`。
  - 接收 Hermes hook envelope JSON。
  - 转换为 `IncomingEvent`。
  - 复用 `/event` 的清洗、typed conversion 和队列入队路径。
  - 返回：与 `/event` 一致的 `EventAcceptedResponse`。

- [ ] 实现 daemon client hook POST。
  - 在 `src/client.rs` 增加 hook URL 和 POST 方法，或复用 `post_event()` 的公共内部 helper。
  - daemon 不可用、非 2xx、无效响应时返回清晰错误，错误中包含 `/api/hermes/hook`。

- [ ] 实现 `hermeship hermes hook --payload`。
  - 支持 JSON 字符串 payload。
  - 支持 `--payload -` 从 stdin 读取。
  - 通过 `DaemonClient` POST `/api/hermes/hook`。
  - 打印 queued hook 摘要。

- [ ] 编写 Hermes hook ingress 测试。
  - 覆盖：`gateway:startup`、`session:start`、`session:end`、`session:reset`、`agent:start`、`agent:end`。
  - 覆盖：`agent:end` 成功/失败 mapping。
  - 覆盖：`message`、`response`、token/cookie/secret 入队前被清洗。
  - 覆盖：`--payload -` stdin 读取。
  - 覆盖：daemon unavailable client 错误。
  - 要求：使用随机端口和本地 test queue，不绑定固定端口，不依赖真实 Hermes gateway。

- [ ] 运行任务 3.3 验证命令。
  - `cargo test hermes`
  - `printf '%s' '{"event":"agent:start","context":{"session_id":"demo"}}' | cargo run -- hermes hook --payload -`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

- [ ] 更新开发状态文档。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 必要时更新：`docs/development-status.md`
  - 完成标准：记录实现、验证、边界和剩余风险。

- [ ] 提交任务 3.3。
  - commit：`feat: 增加 Hermes hook ingress`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- 待任务 3.3 实施、验证和提交后填写。
- 上一阶段 Milestone 3.2 已完成并提交：`0b63e49 feat: 增加 daemon event ingress`。
