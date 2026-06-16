# Task: Milestone 5.1 - Discord Sink 与基础 Live Path

更新时间：2026-06-16 Milestone 5.1 已完成，已提交

本阶段目标：在已完成 daemon ingress、Router、Renderer、Dispatcher 与 fake sink 的基础上，实现第一版 Discord sink payload 与配置接入。Discord sink 负责把 `SinkMessage` 投递到 Discord bot channel 或 Discord webhook；默认测试必须使用本地 deterministic fake HTTP/request builder，不依赖真实 Discord token、真实 Discord API、真实 Hermes gateway 或外网状态。

本阶段边界：只实现 Discord sink 与 payload 构造；不实现 Hermes hook bridge install、install/uninstall lifecycle、release preflight、真实 live verification 或 Slack sink。sink 失败语义完整矩阵可在 Milestone 5.2 深化，但 5.1 必须为 bot channel、webhook、allowed mentions 和内容长度限制打好可测基础。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 最新功能阶段提交：`a336e01 feat: 实现事件 dispatcher 与 fake sink`。
- 启动时应先确认工作树状态：`git status --short --branch`。
- 已完成 Milestone 0 到 Milestone 4.3。
- 已实现 daemon `/health`、`/event`、`/api/hermes/hook`、bounded queue、privacy sanitizer、DaemonClient health/event/hook POST、`hermeship start/status/emit/send/hermes hook`。
- 已实现 Router、ResolvedDelivery、SinkTarget、DeliveryExplanation、event glob、route candidates、metadata filter、disabled/missing target 诊断、0..N delivery。
- 已实现 `hermeship explain` 本地 route explain：加载配置、清洗 payload、转 typed EventEnvelope、展示 matched/skipped routes、failed filters 和 delivery target；不调用 daemon、不入队、不投递。
- 已实现 Renderer trait、DefaultRenderer、RenderedMessage、compact/inline/alert/raw 四种格式、Hermes gateway/session/agent/custom 渲染、安全 template token 和 raw 安全 JSON 摘要。
- 已实现 Dispatcher、DispatchReport、DeliveryOutcome、DeliveryStatus、object-safe Sink trait、SinkMessage、FakeSink、FakeDelivery、route -> render -> sink 管道、队列消费和 fake sink E2E。
- 当前 daemon queue 已有 dispatcher consumer；Discord sink 与 payload 已实现，真实 Discord live delivery 尚未执行。
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
- [x] Milestone 5.1：Discord 配置与 payload。

## 当前待执行

- [ ] Milestone 5.2：Sink 失败语义。

## 后续未完成

- [ ] Milestone 5.2：Sink 失败语义。
- [ ] Milestone 5.3：本地端到端 smoke。
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
  - 预期：当前分支为 `codex/milestone-1-cli`；最新功能阶段提交为 `a336e01 feat: 实现事件 dispatcher 与 fake sink`；启动时不要混入无关改动。

- [x] 检查现有代码边界。
  - 查看：`src/config.rs`
  - 查看：`src/router.rs`
  - 查看：`src/render/mod.rs`
  - 查看：`src/render/default.rs`
  - 查看：`src/dispatch.rs`
  - 查看：`src/sink/mod.rs`
  - 查看：`src/sink/fake.rs`
  - 查看：`src/daemon.rs`
  - 查看：`tests/fixtures/README.md`
  - 必要时参考：`/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/sink/discord.rs`、`/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/discord.rs`
  - 完成标准：确认本阶段只实现 Discord sink/payload，不进入 hook bridge install、release preflight 或 live verification。

- [x] 先写失败测试。
  - 新建：`src/sink/discord.rs`
  - 修改：`src/sink/mod.rs`
  - 必要时修改：`src/dispatch.rs`、`src/daemon.rs`
  - 覆盖：webhook payload、bot channel payload、allowed mentions、content length truncation、missing token/channel 的基础诊断。
  - 命令：`cargo test discord`
  - 预期：实现前测试失败于缺少 Discord sink、payload builder 或配置接入。

- [x] 实现 Discord payload builder。
  - 新建：`src/sink/discord.rs`
  - 类型：Discord payload/request builder、allowed mentions 策略、消息长度限制常量。
  - 完成标准：payload 构造为纯逻辑，可用本地测试覆盖；不访问真实 Discord API。

- [x] 实现 Discord sink。
  - 接入：`Sink` trait。
  - 支持：bot token + channel route。
  - 支持：webhook URL route。
  - 完成标准：可由 dispatcher registry 调用；错误路径返回可诊断 `anyhow::Error`，不 panic。

- [x] 接入 daemon sink registry。
  - 修改：`src/daemon.rs`
  - 行为：默认 daemon 根据配置注册 Discord sink；无 token/webhook/channel 时保持可诊断失败，不阻断 daemon。
  - 完成标准：Milestone 5.1 不做真实 live 投递；默认测试仍使用 fake HTTP 或 request builder。

- [x] 编写 Discord sink 测试。
  - 覆盖：webhook JSON payload。
  - 覆盖：bot channel request payload。
  - 覆盖：mention 前缀和 allowed mentions 策略。
  - 覆盖：content length truncation。
  - 覆盖：token 缺失、channel 缺失、webhook 缺失的基础错误。

- [x] 运行任务 5.1 验证命令。
  - `cargo test discord`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

- [x] 更新开发状态文档。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 必要时更新：`docs/development-status.md`
  - 完成标准：记录实现、验证、边界和剩余风险。

- [x] 提交任务 5.1。
  - commit：`feat: 增加 Discord sink`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- 已按 TDD 执行 Milestone 5.1：`cargo test discord` 在实现前失败于缺少 Discord sink、payload builder、长度常量和 `SinkMessage.mention`；实现后通过。
- 已新增 `src/sink/discord.rs`，实现 Discord bot channel 与 webhook request builder、webhook `wait=true`、`allowed_mentions` 安全策略和 2000 字符内容截断。
- 已处理代码审查反馈：webhook URL 会精确移除已有 `wait` query 参数并追加 `wait=true`，避免 `wait=false` 或 `await=true` 误判。
- 已将 daemon dispatcher registry 接入真实 Discord sink；配置 route 后可走 Discord sink，缺少 token/channel/webhook 会返回可诊断 sink failure，不 panic。
- 已运行验证：`cargo test discord`（9 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（97 lib tests + 6 bin tests passed）。
- 本阶段未实现 Hermes hook bridge install、install/uninstall lifecycle、release preflight、真实 live verification 或 Slack sink；Discord 非 2xx/rate limit 失败矩阵留到 Milestone 5.2。
