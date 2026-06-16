# Task: Milestone 4.1 - Router

更新时间：2026-06-16 Milestone 4.1 待执行

本阶段目标：在已完成 daemon `/health`、通用 `/event` ingress、Hermes hook ingress、隐私清洗和队列入队的基础上，实现 Hermeship 的第一版 router 与 `hermeship explain`。Router 应基于 typed `EventEnvelope` 和配置中的 `[[routes]]` 做结构化匹配，支持 event glob、metadata filter、0..N delivery，并提供可诊断的 explain 输出。

本阶段边界：只实现 Router 和 explain；不实现 renderer、dispatcher、sink、hook bridge install、install/uninstall lifecycle 或 release preflight。默认测试仍只使用本地 deterministic fixture，不依赖真实 Hermes gateway、真实 Discord 或外网状态。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 最新功能阶段提交：`7b10816 feat: 增加 Hermes hook ingress`。
- 启动时应先确认工作树状态：`git status --short --branch`。
- 已完成 Milestone 3.1：daemon `/health`、typed `HealthResponse`、`QueueHealth`、daemon listener、`hermeship start`、`hermeship status`。
- 已完成 Milestone 3.2：daemon 通用 `POST /event`、`EventAcceptedResponse`、bounded `tokio::mpsc` queue、入队前 privacy sanitizer、`DaemonClient::post_event()`、`hermeship emit`、`hermeship send`。
- 已完成 Milestone 3.3：daemon `POST /api/hermes/hook`、`HermesHookEnvelope` normalization、`DaemonClient::post_hermes_hook()`、`hermeship hermes hook --payload` inline/stdin。
- 已完成事件前置能力：`IncomingEvent`、`RoutingMetadata`、typed `EventEnvelope`、Hermes canonical mapping、privacy sanitizer、Hermes 合成 fixture。
- 当前 `explain`、install、release 仍按各自 milestone 保持 placeholder；本阶段只替换 `explain`。
- 当前 daemon 队列只入队不消费，达到容量后 `/event` 和 `/api/hermes/hook` 会返回 503；dispatcher/consumer 在 Milestone 4.3 实现。

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
- [x] Milestone 3.3：Hermes hook ingress。

## 后续未完成

- [ ] Milestone 4.1：Router。
- [ ] Milestone 4.2：Renderer。
- [ ] Milestone 4.3：Dispatcher 与 fake sink。
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
  - 预期：当前分支为 `codex/milestone-1-cli`；最新功能阶段提交为 `7b10816 feat: 增加 Hermes hook ingress`；启动时不要混入无关改动。

- [ ] 检查现有代码边界。
  - 查看：`src/cli.rs`
  - 查看：`src/main.rs`
  - 查看：`src/config.rs`
  - 查看：`src/events.rs`
  - 查看：`src/event/mod.rs`
  - 查看：`src/event/body.rs`
  - 查看：`src/event/compat.rs`
  - 查看：`src/privacy.rs`
  - 查看：`src/hermes.rs`
  - 查看：`tests/fixtures/README.md`
  - 必要时参考：`/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/router.rs`
  - 完成标准：确认本阶段只实现 route match 与 `explain`，不进入 renderer/dispatcher/sink。

- [ ] 先写失败测试。
  - 新建或修改：`src/router.rs`
  - 修改：`src/main.rs`
  - 必要时修改：`src/lib.rs`
  - 覆盖：event glob 命中、metadata filter 命中/未命中、多 route 产生多 delivery、无 route、route disabled、缺 channel 的诊断、route-level channel/format/template/mention 继承、`hermeship explain` 输出命中和未命中原因。
  - 命令：`cargo test router`
  - 预期：实现前测试失败于缺少 router 模块、route match、delivery explanation 和 explain CLI 真实实现。

- [ ] 实现 router 类型与 route match。
  - 新建：`src/router.rs`
  - 类型：`ResolvedDelivery`、`SinkTarget`、`DeliveryExplanation`。
  - 输入：`AppConfig`、`EventEnvelope`。
  - 支持：event glob，例如 `hermes.*`、`hermes.agent.*`、精确 event。
  - 支持：route `filter` 基于 `EventMetadata` 字段匹配，如 `platform`、`project`、`session_id`、`agent_name`、`source`、`provider`。
  - 输出：一个事件匹配 0..N 个 delivery。

- [ ] 实现 route candidate 与诊断。
  - 对每条 route 记录是否 disabled、event pattern 是否匹配、filter 是否匹配、是否缺 channel/webhook。
  - `DeliveryExplanation` 要能说明 matched routes、failed filters、disabled route 和 missing target。
  - 缺 channel/webhook 不应 panic，应给出诊断并不产生可投递 delivery。

- [ ] 实现 `hermeship explain`。
  - 将 `EventArgs::into_event()` 转为 `IncomingEvent`。
  - 使用当前配置、privacy sanitizer 和 `event::compat::from_incoming_event()` 构造 `EventEnvelope`。
  - 调用 router explain 并打印 canonical event、matched delivery、skipped route reason。
  - 不调用 daemon，不入队，不渲染，不投递。

- [ ] 编写 Router 和 explain 测试。
  - 覆盖：多 route 产生多 delivery。
  - 覆盖：filter 命中/未命中。
  - 覆盖：route disabled。
  - 覆盖：无 route 时 explain 明确显示 no deliveries。
  - 覆盖：`format`、`template`、`mention`、`channel` 从 route 层继承。
  - 覆盖：`explain` 输出包含 route 命中原因、filter 失败原因和 delivery target。
  - 要求：使用本地 config 和 fixture，不依赖 daemon、Discord 或外网。

- [ ] 运行任务 4.1 验证命令。
  - `cargo test router`
  - `cargo run -- explain hermes.agent.started --payload '{"platform":"telegram","session_id":"demo"}'`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

- [ ] 更新开发状态文档。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 必要时更新：`docs/development-status.md`
  - 完成标准：记录实现、验证、边界和剩余风险。

- [ ] 提交任务 4.1。
  - commit：`feat: 实现多投递路由`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- 待任务 4.1 实施、验证和提交后填写。
- 上一阶段 Milestone 3.3 已完成并提交：`7b10816 feat: 增加 Hermes hook ingress`。
