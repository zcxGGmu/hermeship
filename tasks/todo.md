# Task: Milestone 5.3 - 本地端到端 smoke

更新时间：2026-06-16 Milestone 5.3 待执行

本阶段目标：在 Milestone 5.2 已完成 Discord sink 失败语义的基础上，补齐本地可重复的端到端 smoke。重点验证 daemon HTTP ingress、队列、router、renderer、fake sink 和 CLI client 路径能形成最小闭环，同时继续证明默认隐私保护生效。

本阶段边界：只做本地 deterministic smoke，不实现 Hermes hook bridge install、install/uninstall lifecycle、release preflight、真实 live verification 或 Slack sink。默认测试不能依赖真实 Discord、真实 Hermes、外网状态或真实凭据。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 最新功能阶段提交：`ea9b789 feat: 完善 sink 失败处理`。
- 启动时应先确认工作树状态：`git status --short --branch`。
- 已完成 Milestone 0 到 Milestone 5.2。
- 已实现 daemon `/health`、`/event`、`/api/hermes/hook`、bounded queue、privacy sanitizer、DaemonClient health/event/hook POST、`hermeship start/status/emit/send/hermes hook`。
- 已实现 Router、DefaultRenderer、Dispatcher、Sink trait、FakeSink、daemon queue consumer，以及 Discord sink payload/request builder、bot channel/webhook 发送路径、allowed mentions、内容长度截断、非 2xx 诊断、429 retry-after 诊断和本地 fake HTTP 失败矩阵。
- 当前 install、release、Hermes hook bridge install、真实 live verification、Slack sink 仍保持后续 milestone placeholder。

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
- [x] Milestone 5.1：Discord 配置与 payload。
- [x] Milestone 5.2：Sink 失败语义。

## 当前待执行

- [ ] Milestone 5.3：本地端到端 smoke。

## 后续未完成

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
  - 预期：当前分支为 `codex/milestone-1-cli`；最新功能阶段提交为 `ea9b789 feat: 完善 sink 失败处理`；启动时不要混入无关改动。

- [ ] 检查现有代码边界。
  - 查看：`src/daemon.rs`
  - 查看：`src/client.rs`
  - 查看：`src/router.rs`
  - 查看：`src/render/mod.rs`
  - 查看：`src/render/default.rs`
  - 查看：`src/dispatch.rs`
  - 查看：`src/sink/mod.rs`
  - 查看：`src/sink/fake.rs`
  - 查看：`tests/fixtures/README.md`
  - 完成标准：确认本阶段只做本地 smoke，不进入 hook bridge install、release preflight、真实 live verification 或 Slack sink。

- [ ] 先写失败测试。
  - 优先修改：`src/dispatch.rs` 或 `src/daemon.rs`
  - 覆盖：启动 test daemon，POST `/api/hermes/hook`，事件进入 queue 后由 dispatcher route -> render -> fake sink。
  - 覆盖：fake sink 收到渲染后的 Hermes 消息，包含安全 metadata 摘要。
  - 覆盖：默认隐私保护仍生效，不泄漏完整 message、response、token、cookie、secret。
  - 覆盖：本地 smoke 不依赖真实 Discord token、真实 Hermes gateway 或外网。
  - 命令：`cargo test dispatch`
  - 命令：`cargo test daemon`
  - 预期：实现前测试失败于本地 smoke 闭环不足或缺少测试辅助。

- [ ] 实现 daemon + fake sink 本地 E2E smoke。
  - 修改：`src/dispatch.rs` 或 `src/daemon.rs`
  - 行为：使用随机本地端口、test queue、fake sink 和 deterministic fixture 完成 daemon ingress 到 fake sink 的最小闭环。
  - 完成标准：测试不启动真实 daemon 固定端口，不访问真实 Discord/Hermes，不需要外网或凭据。

- [ ] 验证 `send` 本地路径。
  - 可选实现方式：使用 test daemon/client 或 CLI command test 覆盖 `send --channel test --message "hello"` 到 `/event`。
  - 完成标准：验证 send 构造 custom event 并进入 daemon client POST 路径；不做真实 Discord 投递。

- [ ] 验证 `emit` 本地路径。
  - 可选实现方式：使用 test daemon/client 或 CLI command test 覆盖 `emit hermes.agent.started --payload '{"session_id":"demo"}'` 到 `/event`。
  - 完成标准：验证 emit 构造 Hermes event 并进入 daemon client POST 路径；不做真实 Discord 投递。

- [ ] 运行任务 5.3 验证命令。
  - `cargo test dispatch`
  - `cargo test daemon`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

- [ ] 更新开发状态文档。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 必要时更新：`docs/development-status.md`
  - 完成标准：记录实现、验证、边界和剩余风险，并把下一入口切到 Milestone 6。

- [ ] 提交任务 5.3。
  - commit：`test: 增加 daemon 到 sink 的端到端覆盖`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- 待任务 5.3 实施、验证和提交后填写。
- 上一阶段 Milestone 5.2 已完成并提交：`ea9b789 feat: 完善 sink 失败处理`。
