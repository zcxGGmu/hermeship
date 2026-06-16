# Task: Milestone 5.2 - Sink 失败语义

更新时间：2026-06-16 Milestone 5.2 待执行

本阶段目标：在已完成 Discord sink payload、bot channel、webhook、allowed mentions 和 daemon sink registry 的基础上，完善 sink 失败语义。重点覆盖 Discord token/channel 缺失、非 2xx、429 rate limit、fake HTTP 失败矩阵，以及多个 delivery 中一个失败时其他 delivery 继续。

本阶段边界：只深化 Discord sink/dispatcher 的失败处理与测试；不实现 Hermes hook bridge install、install/uninstall lifecycle、release preflight、真实 live verification 或 Slack sink。默认测试必须继续使用本地 deterministic fake HTTP/request builder，不依赖真实 Discord、真实 Hermes 或外网状态。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 最新功能阶段提交：`0cd6e4e feat: 增加 Discord sink`。
- 启动时应先确认工作树状态：`git status --short --branch`。
- 已完成 Milestone 0 到 Milestone 5.1。
- 已实现 daemon `/health`、`/event`、`/api/hermes/hook`、bounded queue、privacy sanitizer、DaemonClient health/event/hook POST、`hermeship start/status/emit/send/hermes hook`。
- 已实现 Router、DefaultRenderer、Dispatcher、Sink trait、FakeSink 和 daemon queue consumer。
- 已实现 Discord sink：payload/request builder、bot token + channel、webhook URL、webhook `wait=true`、allowed mentions、2000 字符截断和 daemon sink registry。
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

## 当前待执行

- [ ] Milestone 5.2：Sink 失败语义。

## 后续未完成

- [ ] Milestone 5.3：本地端到端 smoke。
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
  - 预期：当前分支为 `codex/milestone-1-cli`；最新功能阶段提交为 `0cd6e4e feat: 增加 Discord sink`；启动时不要混入无关改动。

- [ ] 检查现有代码边界。
  - 查看：`src/sink/discord.rs`
  - 查看：`src/sink/mod.rs`
  - 查看：`src/sink/fake.rs`
  - 查看：`src/dispatch.rs`
  - 查看：`src/daemon.rs`
  - 查看：`src/router.rs`
  - 查看：`src/config.rs`
  - 查看：`tests/fixtures/README.md`
  - 必要时参考：`/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/discord.rs`
  - 完成标准：确认本阶段只实现 Discord sink 失败语义，不进入 hook bridge install、release preflight、真实 live verification 或 Slack sink。

- [ ] 先写失败测试。
  - 修改：`src/sink/discord.rs`
  - 必要时修改：`src/dispatch.rs`
  - 覆盖：token 缺失通过 dispatcher 报告 `SinkFailed`，不 panic。
  - 覆盖：channel 缺失通过 request builder 或 dispatcher 返回清晰错误。
  - 覆盖：Discord 4xx/5xx 响应包含 HTTP status 和 body tail。
  - 覆盖：Discord 429 rate limit 可解析 `retry_after` 并给出明确诊断或确定性重试行为。
  - 覆盖：多个 delivery 中一个失败时其他 delivery 继续。
  - 命令：`cargo test sink`
  - 命令：`cargo test dispatch`
  - 预期：实现前测试失败于当前 sink 失败语义不足。

- [ ] 实现 Discord 非 2xx 失败诊断。
  - 修改：`src/sink/discord.rs`
  - 行为：HTTP 非 2xx 返回 `anyhow::Error`，包含 status 和截断后的 body tail。
  - 完成标准：错误信息可诊断但不泄露 webhook URL 或 token。

- [ ] 实现 Discord 429 rate limit 语义。
  - 修改：`src/sink/discord.rs`
  - 行为：解析 Discord JSON body 中的 `retry_after`；本阶段可选择确定性短重试或明确记录 retry-after 诊断，但必须测试锁定行为。
  - 完成标准：429 行为不依赖 sleep 猜测，不访问真实 Discord。

- [ ] 完善 dispatcher 多 delivery 失败覆盖。
  - 修改：`src/dispatch.rs`
  - 行为：一个 Discord delivery 失败后，后续 delivery 仍继续；report 中可观察成功与失败。

- [ ] 运行任务 5.2 验证命令。
  - `cargo test sink`
  - `cargo test dispatch`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

- [ ] 更新开发状态文档。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 必要时更新：`docs/development-status.md`
  - 完成标准：记录实现、验证、边界和剩余风险。

- [ ] 提交任务 5.2。
  - commit：`feat: 完善 sink 失败处理`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- 待任务 5.2 实施、验证和提交后填写。
- 上一阶段 Milestone 5.1 已完成并提交：`0cd6e4e feat: 增加 Discord sink`。
