# Hermeship 开发状态

最后更新：2026-06-16 Milestone 4.3 已完成，Milestone 5.1 待执行

本文是下次启动 Codex 会话时的状态入口。执行开发前仍以 `tasks/development-checklist.md` 的 checkbox 为准；当前阶段计划维护在 `tasks/todo.md`。

## 当前结论

- Hermeship 的目标已经锁定：完全参考 `/Users/zq/Desktop/ai-projs/posp/template/clawhip` 的项目形态、架构和功能，只把 OpenClaw/Codex/Claude/OMC/OMX 等耦合替换为 Hermes 适配。
- Hermeship 不是调用现有 `clawhip` 的 thin adapter，也不依赖运行中的 `clawhip` daemon。
- 主实现语言确定为 Rust，采用 daemon-first 架构；Python 只用于 Hermes gateway hook bridge 模板 `handler.py`。
- 方案文档与执行清单已经拆分：方案文档维护架构和边界，`tasks/development-checklist.md` 和 `tasks/todo.md` 维护可勾选进度。
- 默认测试策略已经确定：使用本地 fixture、fake sink、fake HTTP、fake Hermes home、fake hermeship binary；真实 Discord/Hermes 只进入 live verification。
- 当前开发分支：`codex/milestone-1-cli`。
- 当前最新功能阶段提交：`a336e01 feat: 实现事件 dispatcher 与 fake sink`。
- 下次继续开发前必须先运行 `git status --short --branch` 确认工作树，只在预期文档/代码变更上继续。
- 当前下一步：继续 Milestone 5，执行 Discord Sink 与基础 Live Path。

## 已完成

- 已记录项目习惯：每完成一阶段任务，必须验证并提交；后续会话启动时先复习 `tasks/lessons.md`。
- 已重写方案文档：`docs/plans/2026-06-15-hermeship-development-plan.md`。
- 已重写阶段性开发清单：`tasks/development-checklist.md`。
- 已将测试计划集成到方案文档和开发清单。

### Milestone 0：契约与仓库基线

- 已复核 `template/clawhip` 指定参考文件，确认可移植形态为 Rust CLI、daemon、typed event、dispatcher、multi-delivery router、renderer/sink split、config/lifecycle/release preflight。
- 已复核 Hermes gateway hook 与 plugin 参考源码，确认 MVP 先使用 gateway hook bridge，plugin/observer 后续推进。
- 已更新 `README.md`，明确 Hermeship 是 Hermes-native daemon-first event router，不是 clawhip runtime client。
- 已运行旧 Python/thin-adapter 方向过滤搜索，正文无旧方案残留。
- 已提交：`af57c49 docs: 明确 hermeship 完整项目方向`。

### Milestone 1.1：Cargo 项目与 CLI 入口

- 已创建 Rust 2024 工程骨架：`Cargo.toml`、`Cargo.lock`、`src/lib.rs`、`src/main.rs`、`src/cli.rs`。
- 已实现最小 `clap` CLI 命令树：`start`、`status`、`send`、`emit`、`explain`、`config`、`hermes`、`install`、`uninstall`、`release`。
- 已新增 CLI parse 单元测试，覆盖 `send`、`emit --payload`、`hermes hook --payload`、`hermes install-hooks`。
- 已新增公开命令 fixture：`tests/fixtures/cli/public_commands.txt`，并断言必备公开命令前缀存在。
- 已运行验证：`cargo fmt --all -- --check`、`cargo test cli`、`cargo run -- --help`。
- 已提交：`d03170e chore: 搭建 Hermeship Rust CLI 骨架`。

### Milestone 1.2：配置模型

- 已新增 `src/config.rs`，并在 `src/lib.rs` 导出 `hermeship::config`。
- 已实现配置模型：`AppConfig`、`DaemonConfig`、`ProvidersConfig`、`DiscordConfig`、`DefaultsConfig`、`PrivacyConfig`、`HermesConfig`、`RouteRule`、`MessageFormat`。
- 已实现默认配置路径：`HERMESHIP_CONFIG` 优先，否则 `$HOME/.hermeship/config.toml`。
- 已实现默认配置与 TOML 加载：缺失配置返回默认值，非法 TOML 返回错误，未知 key 按前向兼容策略忽略。
- 已实现空值归一化和环境变量覆盖：`HERMESHIP_DAEMON_URL`、`HERMESHIP_DISCORD_TOKEN`、`HERMESHIP_DEFAULT_CHANNEL`、`HERMESHIP_DRY_RUN`。
- 已将 `hermeship config path`、`hermeship config show`、`hermeship config verify` 接入真实配置逻辑。
- 已运行验证：`cargo fmt --all -- --check`、`cargo test config`、`cargo run -- config show`、`cargo test`。
- 已提交：`50723af feat: 实现 hermeship 配置模型与 config CLI`。

### Milestone 1.3：质量门禁与仓库基础

- 已扩展 `.gitignore`：保留 `/target/`，新增本地编辑器临时文件、日志、临时目录、测试输出和覆盖率输出规则。
- 已确认 `.gitignore` 不忽略源码、文档、fixture 或 `Cargo.lock`。
- 已在 `README.md` 新增 Development Quality Gates。
- 已新增 fixture 目录：`tests/fixtures/hermes/`、`tests/fixtures/privacy/`、`tests/fixtures/routes/`、`tests/fixtures/discord/`。
- 已新增 `tests/fixtures/README.md`，明确 fixture 只能使用合成脱敏样例。
- 已运行验证：`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 已提交：`70c8f03 chore: 增加 Rust 质量门禁与仓库基础`。

### Milestone 2.1：IncomingEvent 与格式

- 已新增 `src/events.rs`，并在 `src/lib.rs` 导出 `hermeship::events`。
- 已实现 `IncomingEvent`：字段包含 `kind`、`channel`、`mention`、`format`、`template`、`payload`。
- 已实现 `RoutingMetadata`：覆盖 Hermes gateway 元数据和后续路由需要的通用字段，如 `tool`、`provider`、`source`、`platform`、`session_id`、`project`、`repo_path`、`branch`。
- 已采用单一 `MessageFormat` 策略：`src/config.rs` 保留唯一 enum 定义并新增 `from_label()`；`src/events.rs` 通过 `pub use crate::config::MessageFormat` 重导出。
- 已支持 `IncomingEvent` 反序列化字段别名：`type`、`kind`、`event`。
- 已支持缺省 payload 和 `payload: null` 归一为空对象；无显式 payload 时，top-level extra 字段进入 payload。
- 已将 `hermeship emit` 和 `hermeship explain` 的参数解析接入 `EventArgs::into_event()`。
- 已新增 Hermes 合成 fixture：`tests/fixtures/hermes/agent_start.json`、`session_end.json`、`invalid_payload.json`。
- 已运行验证：`cargo test events`、`cargo test cli`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 已提交：`5584b13 feat: 完成 Hermes 入口事件模型与 emit 解析`。

### Milestone 2.2：Typed EventEnvelope

- 已新增 `src/event/mod.rs`、`src/event/body.rs`、`src/event/compat.rs`，并在 `src/lib.rs` 导出 `hermeship::event`。
- 已定义 `EventEnvelope`、`EventBody`、`EventMetadata`、`EventPriority`。
- 已实现 Hermes event body：`HermesGatewayStarted`、`HermesSessionStarted`、`HermesSessionFinished`、`HermesSessionReset`、`HermesAgentStarted`、`HermesAgentStep`、`HermesAgentFinished`、`HermesAgentFailed`、`Custom`。
- 已实现 canonical mapping：`gateway:startup`、`session:start`、`session:end`、`session:reset`、`agent:start`、`agent:step`、`agent:end`；显式失败的 `agent:end` 会转为 `hermes.agent.failed`。
- 已实现 `IncomingEvent -> EventEnvelope` conversion，保留 route hint 并提取 provider/source/platform/chat/session/agent/project/repo metadata。
- 已覆盖未知 event -> `Custom`、缺失 `session_id` 降级、fixture conversion、大小写不敏感失败状态。
- 已运行验证：`cargo test event`、`cargo test events`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 已提交：`b799415 feat: 实现 Hermes typed event model`。

### Milestone 2.3：隐私与 payload 清洗

- 已新增 `src/privacy.rs`，并在 `src/lib.rs` 导出 `hermeship::privacy`。
- 已实现 `sanitize_payload`、`redact_value`、`excerpt_policy`，保持为纯 `serde_json::Value` 清洗逻辑。
- 已默认递归脱敏敏感 key：`token`、`api_key`、`authorization`、`password`、`secret`、`cookie`；支持大小写不敏感、camelCase 和常见缩写 key 匹配。
- 已默认删除完整正文类字段：`message`、`response`、`conversation_history`、`request`、`provider_response`、`tool_result`；同时清洗 `messages`、`prompt`、`user_message`、`assistant_response`、`provider_request`、`provider_request_body`、`provider_response_body`、`tool_results`、`tool_result_body` 等同类高风险别名。
- 已保留安全摘要：`message_chars`、`response_chars`、`has_message`、`has_response`；非法摘要字段类型会被丢弃，computed summary 不会被原始 payload 覆盖。
- 已实现 opt-in 摘录：`include_message_excerpt`、`include_response_excerpt`、`max_excerpt_chars`；摘录先经过完整 sanitizer，再按 char 边界截断。
- 已新增合成 fixture：`tests/fixtures/privacy/sensitive_payload.json`，不包含真实 token、cookie、secret、完整 prompt、完整对话或 provider request/response body。
- 已根据代码审查修复摘要字段泄漏、`Authorization: Bearer ...` / `api_key = ...` inline secret 泄漏、URL query secret 泄漏、camelCase/acronym alias 绕过、结构化摘录泄漏和 fixture body hygiene 问题。
- 已运行验证：`cargo test privacy`（10 passed）、`cargo test event`（14 passed）、`cargo test events`（6 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（41 passed）。
- 已提交：`175009d feat: 增加 Hermes 事件隐私清洗`。

### Milestone 3.1：Daemon health 与 client

- 已新增 `src/daemon.rs`，并在 `src/lib.rs` 导出 `hermeship::daemon`。
- 已新增 `src/client.rs`，并在 `src/lib.rs` 导出 `hermeship::client`。
- 已实现 typed `HealthResponse` 与 `QueueHealth`。
- 已实现 daemon `/health` endpoint，返回 version、status、queue 状态和 configured sinks 摘要。
- 已实现 daemon listener 绑定与 `serve_listener()`，测试可使用随机端口。
- 已实现 `hermeship start`：加载配置、支持 `--port` 覆盖、验证配置并启动 daemon。
- 已实现 `hermeship status`：通过 `DaemonClient` 调用 `/health` 并打印可读摘要。
- 已实现 client base URL 规范化、2 秒 health timeout、daemon unavailable 清晰错误和非 2xx 错误摘要。
- 已覆盖 health response schema、队列状态、configured sinks、随机端口 HTTP `/health` 和 daemon 未运行错误。
- 本阶段没有实现 event ingress、`/event`、Hermes hook ingress、队列入队、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
- 已运行验证：`cargo test daemon`（4 passed）、`cargo run -- status`（daemon 未运行时返回清晰错误且无 panic）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（45 passed）。
- 已提交：`ff5c589 feat: 增加 hermeship daemon health`。

### Milestone 3.2：Event ingress 与队列

- 已实现 daemon 通用 `POST /event` endpoint，接收 `IncomingEvent` JSON。
- 已在入队前接入 `privacy::sanitize_payload()`，再使用 `event::compat::from_incoming_event()` 转为 typed `EventEnvelope`。
- 已新增 bounded `tokio::mpsc` queue scaffold；本阶段只入队，不消费、不路由、不渲染、不投递。
- 已新增 typed `EventAcceptedResponse`，返回 event id、canonical kind、queued 状态和 queue health。
- 已将 `/health` queue 状态改为真实 pending/capacity/status。
- 已实现 `DaemonClient::event_url()` 与 `DaemonClient::post_event()`，覆盖 daemon unavailable、非 2xx 和无效响应错误。
- 已将 `hermeship emit` 和 `hermeship send` 替换为 daemon client POST `/event` 路径，输出 queued 摘要。
- 已调整 `IncomingEvent::custom()` 使用安全 `summary` 字段承载显式 send 文本，避免与 Hermes 对话正文 `message` 隐私语义冲突。
- 已覆盖有效 fixture 入队、入队前隐私清洗、非法 JSON 4xx、缺失 event kind 4xx、daemon unavailable、queue full 503、health pending、send/emit client 投递。
- 本阶段没有实现 Hermes hook ingress、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
- 已运行验证：`cargo test daemon`（11 passed + bin 2 passed）、`cargo test event`（21 passed + bin 2 passed）、临时 daemon 下 `cargo run -- emit hermes.agent.started --payload '{"session_id":"demo"}'` 返回 queued 摘要、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（52 passed + bin 2 passed）。
- 已提交：`0b63e49 feat: 增加 daemon event ingress`。

### Milestone 3.3：Hermes hook ingress

- 已新增 `src/hermes.rs`，并在 `src/lib.rs` 导出 `hermeship::hermes`。
- 已实现 `HermesHookEnvelope`：接收 `provider`、`source`、`event`/`event_type`、`context`，默认 provider/source 为 `hermes`/`gateway`。
- 已实现 Hermes hook envelope -> `IncomingEvent` normalization，payload 保留 provider/source/event/context metadata，并复用既有 Hermes canonical mapping。
- 已实现 daemon `POST /api/hermes/hook` endpoint，复用 `/event` 的入队前 privacy sanitizer、typed conversion 和 bounded queue `try_send` 管道。
- 已实现 `DaemonClient::hermes_hook_url()` 与 `DaemonClient::post_hermes_hook()`，daemon unavailable、非 2xx 和无效响应错误包含 `/api/hermes/hook`。
- 已将 `hermeship hermes hook --payload` 替换为真实 daemon client POST 路径，支持 inline JSON 和 `--payload -` stdin，输出 queued 摘要。
- 已覆盖 hook envelope 默认值、`event_type` alias、gateway/session/agent mapping、`agent:end` 成功/失败 mapping、daemon hook 入队、入队前隐私清洗、缺失 event 4xx、daemon unavailable、CLI stdin 和 client 投递。
- 本阶段没有实现 router、renderer、dispatcher、sink、hook bridge install、install/uninstall lifecycle 或 release preflight。
- 已运行验证：`cargo test hermes`（14 lib tests + 3 bin tests passed）、临时 daemon 下 `printf '%s' '{"event":"agent:start","context":{"session_id":"demo"}}' | cargo run -- hermes hook --payload -` 返回 queued 摘要、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（61 lib tests + 5 bin tests passed）。
- 已提交：`7b10816 feat: 增加 Hermes hook ingress`。

### Milestone 4.1：Router

- 已新增 `src/router.rs`，并在 `src/lib.rs` 导出 `hermeship::router`。
- 已实现 `Router`、`ResolvedDelivery`、`SinkTarget`、`DeliveryExplanation`，支持 event glob、route candidates、结构化 metadata filter、disabled route 诊断、missing target 诊断和 0..N delivery。
- 已将 `hermeship explain` 从 placeholder 替换为本地 route explain：加载配置、清洗 payload、转 typed `EventEnvelope`、打印 route candidates、matched/skipped routes、failed filters 和 delivery target。
- `explain` 不调用 daemon、不入队、不渲染、不投递；renderer、dispatcher、sink、hook bridge install、install/uninstall lifecycle 和 release preflight 仍属于后续 milestone。
- 已覆盖多 route 多投递、filter 命中/未命中、disabled route、missing target、无 route、event hint/default channel fallback、route-level channel/format/template/mention 继承、explain 输出契约和 webhook 诊断脱敏。
- 已根据代码审查修复 Discord webhook 诊断泄漏风险：`explain` human/serialized diagnostics 不输出完整 webhook URL，内部 delivery target 仍保留原值供后续 dispatcher 使用。
- 已运行验证：`cargo test router`（6 lib tests + 1 bin test passed）、`cargo run -- explain hermes.agent.started --payload '{"platform":"telegram","session_id":"demo"}'` 返回 no routes/no deliveries 诊断、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（67 lib tests + 6 bin tests passed）。
- 已提交：`864e7f4 feat: 实现多投递路由`。

### Milestone 4.2：Renderer

- 已新增 `src/render/mod.rs`、`src/render/default.rs`，并在 `src/lib.rs` 导出 `hermeship::render`。
- 已实现 `Renderer` trait、`DefaultRenderer` 和 `RenderedMessage`，输入为 `EventEnvelope` 与 `ResolvedDelivery`，输出 deterministic 可投递文本。
- 已支持 `compact`、`inline`、`alert`、`raw` 四种格式，并覆盖 Hermes gateway/session/agent/custom 事件。
- 已实现 route/template 安全 token：`{event}`、`{canonical_kind}`、`{source}`、`{provider}`、`{platform}`、`{session_id}`、`{agent_name}`、`{project}`、`{channel}`；未批准 token 保持原样。
- 已将 `raw` 固定为安全 JSON 输出：忽略 template/mention，不直接序列化 typed 自由文本，只输出长度/存在性摘要并清洗 nested payload。
- 已覆盖测试：所有格式、缺字段降级、template token、route-level format/template/mention、raw+template、direct typed free-text raw 泄漏回归和未批准 token。
- 已运行验证：`cargo test render`（10 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（74 lib tests + 6 bin tests passed）。
- 已确认本阶段没有实现 dispatcher、sink、hook bridge install、install/uninstall lifecycle 或 release preflight。
- 已提交：`d4303ae feat: 增加 Hermes 默认渲染器`。

### Milestone 4.3：Dispatcher 与 fake sink

- 已新增 `src/dispatch.rs`、`src/sink/mod.rs`、`src/sink/fake.rs`，并在 `src/lib.rs` 导出 `dispatch` 与 `sink`。
- 已实现 object-safe `Sink` trait、`SinkMessage`、`FakeSink` 和 `FakeDelivery`，用于本地测试记录 target、format、rendered content、event kind 和 route index。
- 已实现 `Dispatcher`、`DispatchReport`、`DeliveryOutcome` 和 `DeliveryStatus`，支持单事件与队列消费，执行 `Router::resolve -> Renderer::render -> Sink::send`。
- 已实现单个 delivery 失败不阻断其他 delivery；render failure、missing sink 和 sink failure 都能在 report 中观察。
- 已将默认 daemon queue 接入 dispatcher consumer，生产 daemon 不再只入队不消费；本阶段未注册真实 sink，Discord sink 仍在 Milestone 5。
- 已覆盖多投递、单 sink failure、无 route、render failure、missing sink、队列消费、daemon ingress -> dispatcher -> fake sink E2E 和隐私不泄漏。
- 原计划命令 `cargo test dispatch sink` 是无效 Cargo 语法，执行时返回 `unexpected argument 'sink'`；实际验证拆分为 `cargo test dispatch` 与 `cargo test sink`。
- 已运行验证：`cargo test dispatch`（8 passed）、`cargo test sink`（8 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（87 lib tests + 6 bin tests passed）。
- 已确认本阶段没有实现 Discord sink、Hermes hook bridge install、install/uninstall lifecycle 或 release preflight。
- 已提交：`a336e01 feat: 实现事件 dispatcher 与 fake sink`。

## 未完成

- Milestone 5 到 Milestone 10 均未执行。
- Discord sink、Hermes hook bridge、安装/回滚、release preflight、live verification 均未实现。
- 默认 daemon queue 已有 dispatcher consumer；真实 Discord 投递仍在 Milestone 5 实现。
- live Discord verification 凭据是否可用尚未确认。
- Slack sink、git/GitHub/tmux parity 是否进入 `0.1.0` 尚未最终确认。
- macOS launchd 是否与 systemd 同期实现尚未最终确认。

## 下一步入口

从 `tasks/development-checklist.md` 的 **Milestone 5：Discord Sink 与基础 Live Path** 继续，优先执行 **任务 5.1：Discord 配置与 payload**。

建议第一段工作：

1. 复习 `tasks/lessons.md`、本文、方案文档、开发清单和 `tasks/todo.md`。
2. 确认当前分支、最新提交和未提交变更：
   - `git status --short --branch`
   - `git log -3 --oneline`
3. 确认 Milestone 4.3 Dispatcher 与 fake sink 已完成，并从 `tasks/development-checklist.md` 的任务 5.1 继续。
4. 读取当前相关代码：
   - `src/cli.rs`
   - `src/config.rs`
   - `src/events.rs`
   - `src/event/mod.rs`
   - `src/event/body.rs`
   - `src/event/compat.rs`
   - `src/privacy.rs`
   - `src/router.rs`
   - `src/render/mod.rs`
   - `src/render/default.rs`
   - `src/dispatch.rs`
   - `src/sink/mod.rs`
   - `src/sink/fake.rs`
   - `tests/fixtures/README.md`
5. 从任务 5.1 继续，先写失败测试，再实现 Discord sink payload 与配置接入。
6. 注意任务 5.1 只实现 Discord sink 与 payload，不进入 hook bridge install 或 release preflight。
7. 运行任务 5.1 验证命令：
   - `cargo test discord`
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`
8. 更新 `tasks/development-checklist.md` 的运行状态日志和 `tasks/todo.md` 的 Review。
9. 阶段完成后必须验证并提交，commit 信息使用详细中文，说明变更、验证和影响。

## 下次启动提示词

```text
请在 /Users/zq/Desktop/ai-projs/posp/hermeship 继续 Hermeship 开发。

启动后请先阅读：
- tasks/lessons.md
- docs/development-status.md
- docs/plans/2026-06-15-hermeship-development-plan.md
- tasks/development-checklist.md
- tasks/todo.md

当前状态：
- 当前分支是 codex/milestone-1-cli。
- 最新功能阶段提交：a336e01 feat: 实现事件 dispatcher 与 fake sink。
- Milestone 0 已完成并提交：af57c49 docs: 明确 hermeship 完整项目方向。
- Milestone 1.1 已完成并提交：d03170e chore: 搭建 Hermeship Rust CLI 骨架。
- Milestone 1.2 已完成并提交：50723af feat: 实现 hermeship 配置模型与 config CLI。
- Milestone 1.3 已完成并提交：70c8f03 chore: 增加 Rust 质量门禁与仓库基础。
- Milestone 2.1 已完成并提交：5584b13 feat: 完成 Hermes 入口事件模型与 emit 解析。
- Milestone 2.2 已完成并提交：b799415 feat: 实现 Hermes typed event model。
- Milestone 2.3 已完成并提交：175009d feat: 增加 Hermes 事件隐私清洗。
- Milestone 3.1 已完成并提交：ff5c589 feat: 增加 hermeship daemon health。
- Milestone 3.2 已完成并提交：0b63e49 feat: 增加 daemon event ingress。
- Milestone 3.3 已完成并提交：7b10816 feat: 增加 Hermes hook ingress。
- Milestone 4.1 已完成并提交：864e7f4 feat: 实现多投递路由。
- Milestone 4.2 已完成并提交：d4303ae feat: 增加 Hermes 默认渲染器。
- Milestone 4.3 已完成并提交：a336e01 feat: 实现事件 dispatcher 与 fake sink。
- 已实现 src/events.rs：IncomingEvent、RoutingMetadata、字段别名反序列化、空/null payload 归一，以及 MessageFormat 的单一复用/重导出策略。
- 已实现 src/event/：EventEnvelope、EventBody、EventMetadata、EventPriority、Hermes canonical mapping、IncomingEvent -> EventEnvelope conversion。
- 已实现 src/privacy.rs：sanitize_payload、redact_value、excerpt_policy、敏感 key 递归脱敏、正文默认禁发、安全摘要和 opt-in 摘录。
- 已实现 src/daemon.rs：/health、/event、/api/hermes/hook、HealthResponse、QueueHealth、EventAcceptedResponse、bounded mpsc queue、daemon listener 和 serve 入口。
- 已实现 src/client.rs：DaemonClient health 查询、event POST、Hermes hook POST、base URL 规范化、timeout 和清晰错误。
- 已实现 src/hermes.rs：HermesHookEnvelope、event/event_type alias、provider/source 默认值和 Hermes hook envelope -> IncomingEvent normalization。
- 已实现 src/router.rs：Router、ResolvedDelivery、SinkTarget、DeliveryExplanation、event glob、route candidates、metadata filter、disabled/missing target 诊断和 0..N delivery。
- 已实现 src/render/：Renderer trait、DefaultRenderer、RenderedMessage、compact/inline/alert/raw 四种格式、Hermes gateway/session/agent/custom 渲染、安全 template token、raw 安全 JSON 摘要。
- 已实现 src/dispatch.rs：Dispatcher、DispatchReport、DeliveryOutcome、DeliveryStatus、单事件和队列消费、route -> render -> sink 管道、单 delivery 失败不阻断其他 delivery。
- 已实现 src/sink/：object-safe Sink trait、SinkMessage、FakeSink、FakeDelivery、本地 fake sink 记录和确定性失败注入。
- 已接入 hermeship start/status/emit/send/hermes hook 的真实 daemon health/event/hook 行为，hermes hook 支持 `--payload -` stdin。
- 已接入 hermeship explain 的本地 route explain 行为：加载配置、清洗 payload、转 typed EventEnvelope、展示 matched/skipped routes、failed filters 和 delivery target，不调用 daemon、不入队、不投递。
- Hermes canonical mapping 已覆盖 gateway:startup、session:start、session:end、session:reset、agent:start、agent:step、agent:end；显式失败的 agent:end 映射为 hermes.agent.failed；未知 event 降级为 Custom。
- 已通过验证：cargo test dispatch、cargo test sink、cargo fmt --all -- --check、cargo clippy --all-targets -- -D warnings、cargo test。
- Hermeship 是 Hermes-native daemon-first event router，不是 thin adapter，不调用 clawhip runtime，也不依赖运行中的 clawhip daemon。
- 方案文档只维护架构和边界，执行进度维护在 tasks/development-checklist.md 和 tasks/todo.md。

请从 tasks/development-checklist.md 的 Milestone 5 继续，优先执行任务 5.1：Discord Sink 与基础 Live Path：
1. 先复习 tasks/lessons.md，并确认当前分支、最新提交和未提交变更：git status --short --branch、git log -3 --oneline。
2. 确认 tasks/development-checklist.md 的 Milestone 5.1 计划，并将当前任务计划写入 tasks/todo.md。
3. 阅读 src/config.rs、src/router.rs、src/render/mod.rs、src/render/default.rs、src/dispatch.rs、src/sink/mod.rs、src/sink/fake.rs、tests/fixtures/README.md。
4. 先写失败测试，再实现 Discord sink payload 与配置接入。
5. 本阶段只实现 Discord sink 与 payload，不实现 hook bridge install 或 release preflight。
6. 默认测试仍只使用本地 deterministic fixture。
7. 运行验证：cargo test discord、cargo fmt --all -- --check、cargo clippy --all-targets -- -D warnings、cargo test。
8. 更新 tasks/development-checklist.md 的运行状态日志和 tasks/todo.md 的 Review。
9. 阶段完成后必须验证并提交，commit 信息使用详细中文，说明变更、验证和影响。
```
