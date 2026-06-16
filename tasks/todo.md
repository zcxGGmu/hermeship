# Task: Milestone 4.3 - Dispatcher 与 fake sink

更新时间：2026-06-16 Milestone 4.3 已完成

本阶段目标：在已完成 typed event、privacy sanitizer、daemon ingress、Hermes hook ingress、Router 与 Renderer 的基础上，实现第一版 dispatcher 和 fake sink。Dispatcher 负责从事件队列消费 `EventEnvelope`，执行 route -> render -> sink 管道；fake sink 负责在本地测试中记录 delivery，不依赖真实 Discord、Hermes gateway 或外网。

本阶段边界：只实现 dispatcher 与 fake sink；不实现 Discord sink、Hermes hook bridge install、install/uninstall lifecycle 或 release preflight。默认测试仍只使用本地 deterministic fixture。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 最新功能阶段提交：`d4303ae feat: 增加 Hermes 默认渲染器`。
- 启动时应先确认工作树状态：`git status --short --branch`。
- 已完成 Milestone 3.1：daemon `/health`、typed `HealthResponse`、`QueueHealth`、daemon listener、`hermeship start`、`hermeship status`。
- 已完成 Milestone 3.2：daemon 通用 `POST /event`、`EventAcceptedResponse`、bounded `tokio::mpsc` queue、入队前 privacy sanitizer、`DaemonClient::post_event()`、`hermeship emit`、`hermeship send`。
- 已完成 Milestone 3.3：daemon `POST /api/hermes/hook`、`HermesHookEnvelope` normalization、`DaemonClient::post_hermes_hook()`、`hermeship hermes hook --payload` inline/stdin。
- 已完成 Milestone 4.1：`Router`、`ResolvedDelivery`、`SinkTarget`、`DeliveryExplanation`、event glob、metadata filter、0..N delivery、`hermeship explain` 本地 route explain、Discord webhook 诊断脱敏。
- 已完成 Milestone 4.2：`Renderer` trait、`DefaultRenderer`、`RenderedMessage`、compact/inline/alert/raw 四种格式、安全 template token 和 raw 安全 JSON 摘要。
- 当前 daemon 队列已在默认 daemon router 中启动 dispatcher consumer；真实 Discord 投递仍在 Milestone 5 实现。
- 当前 install、release、Hermes hook bridge install 仍保持后续 milestone placeholder。

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
- [x] Milestone 4.1：Router。
- [x] Milestone 4.2：Renderer。
- [x] Milestone 4.3：Dispatcher 与 fake sink。

## 当前待执行

- [ ] Milestone 5：Discord Sink 与基础 Live Path。

## 后续未完成

- [ ] Milestone 6：Hermes Hook Bridge 安装。
- [ ] Milestone 7：安装、生命周期与运维 CLI。
- [ ] Milestone 8：clawhip 功能 Parity 扩展。
- [ ] Milestone 9：文档与 Live Verification。
- [ ] Milestone 10：Hermes Plugin / Observer 研究。

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
  - 预期：当前分支为 `codex/milestone-1-cli`；最新功能阶段提交为 `d4303ae feat: 增加 Hermes 默认渲染器`；启动时不要混入无关改动。

- [x] 检查现有代码边界。
  - 查看：`src/cli.rs`
  - 查看：`src/main.rs`
  - 查看：`src/config.rs`
  - 查看：`src/daemon.rs`
  - 查看：`src/events.rs`
  - 查看：`src/event/mod.rs`
  - 查看：`src/event/body.rs`
  - 查看：`src/event/compat.rs`
  - 查看：`src/privacy.rs`
  - 查看：`src/router.rs`
  - 查看：`src/render/mod.rs`
  - 查看：`src/render/default.rs`
  - 查看：`tests/fixtures/README.md`
  - 必要时参考：`/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/dispatch.rs`、`/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/sink/`
  - 完成标准：确认本阶段只实现 dispatcher/fake sink，不进入 Discord sink 或 hook install。

- [x] 先写失败测试。
  - 新建：`src/dispatch.rs`
  - 新建：`src/sink/mod.rs`
  - 新建：`src/sink/fake.rs`
  - 必要时修改：`src/lib.rs`
  - 覆盖：多投递、单 sink failure、无 route、render failure、fake sink 保存 rendered delivery。
  - 命令：`cargo test dispatch sink`
  - 预期：实现前测试失败于缺少 dispatch/sink 模块、fake sink、dispatcher pipeline。

- [x] 实现 sink trait 与 fake sink。
  - 新建：`src/sink/mod.rs`
  - 新建：`src/sink/fake.rs`
  - 类型：`Sink` trait、fake sink 记录结构、fake sink error 注入或等价测试辅助。
  - 完成标准：fake sink 可保存 target、format、rendered message、event kind 和 route index，不调用网络。

- [x] 实现 dispatcher。
  - 新建：`src/dispatch.rs`
  - 输入：`EventEnvelope` 或队列 receiver、`Router`、`DefaultRenderer`、sink registry/fake sink。
  - 行为：route -> render -> sink；一个 delivery 失败不能阻断其他 delivery。
  - 完成标准：无 route 时返回可诊断结果，不 panic；render/sink failure 可被测试观察。

- [x] 编写 dispatcher 测试。
  - 覆盖：多 route 多投递。
  - 覆盖：单 sink failure 不阻断其他 delivery。
  - 覆盖：无 route 时不投递。
  - 覆盖：render failure 或 unsupported sink 诊断。
  - 要求：使用本地 fixture 和 fake sink，不依赖 daemon、Discord 或外网。

- [x] 编写 dispatcher E2E 测试。
  - 使用：`tests/fixtures/hermes/agent_start.json` + fake sink。
  - 断言：事件经过 route -> render -> fake sink 后保存 delivery。
  - 断言：rendered message 不泄漏 token、cookie、secret、完整 prompt、完整对话或 provider request/response body。

- [x] 运行任务 4.3 验证命令。
  - `cargo test dispatch sink`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

- [x] 更新开发状态文档。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 必要时更新：`docs/development-status.md`
  - 完成标准：记录实现、验证、边界和剩余风险。

- [x] 提交任务 4.3。
  - commit：`feat: 实现事件 dispatcher 与 fake sink`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- 已新增 `src/dispatch.rs`、`src/sink/mod.rs`、`src/sink/fake.rs`，并在 `src/lib.rs` 导出 `dispatch` 与 `sink`。
- 已实现 `Dispatcher`、`DispatchReport`、`DeliveryOutcome` 和 `DeliveryStatus`，支持单事件与队列消费，执行 `Router::resolve -> Renderer::render -> Sink::send`。
- 已实现 object-safe `Sink` trait、`SinkMessage`、`FakeSink` 和 `FakeDelivery`；fake sink 记录 target、format、rendered content、event kind、route index，并支持按 route index 注入确定性失败。
- 已将默认 daemon queue 接入 dispatcher consumer，避免生产路径只入队不消费；本阶段不注册真实 sink，Discord sink 仍在 Milestone 5。
- 已覆盖测试：多投递、单 sink failure 不阻断后续 delivery、无 route、render failure、missing sink、队列消费、daemon ingress -> dispatcher -> fake sink E2E 和隐私不泄漏。
- 原计划命令 `cargo test dispatch sink` 是无效 Cargo 语法，执行时返回 `unexpected argument 'sink'`；实际验证拆分为 `cargo test dispatch` 与 `cargo test sink`。
- 已运行验证：`cargo test dispatch`（8 passed）、`cargo test sink`（8 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（87 lib tests + 6 bin tests passed）均通过。
- 本阶段未实现 Discord sink、Hermes hook bridge install、install/uninstall lifecycle 或 release preflight。
