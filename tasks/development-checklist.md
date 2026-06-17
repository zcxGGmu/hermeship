# Hermeship 开发进度清单

本文用于跟踪 `hermeship` 的后续迭代开发进度。

方案文档：`docs/plans/2026-06-15-hermeship-development-plan.md`

状态入口：`docs/development-status.md`

## 跟踪规则

- [ ] 同一时间只推进一个 milestone。
- [ ] 每次实现会话结束前更新本清单。
- [ ] 只有运行过对应验证命令后，才能勾选任务完成。
- [ ] 每完成一个阶段任务就提交一次。
- [ ] commit 信息使用中文，说明完成内容、验证结果和影响。
- [ ] 方案文档只维护设计边界，执行进度只维护在本清单。
- [ ] MVP 阶段不修改 Hermes 核心。
- [ ] MVP 阶段不依赖运行中的 `clawhip`。
- [ ] `template/clawhip` 只作为架构和实现参考，不作为运行时依赖。
- [ ] Hermes plugin/observer 必须等 gateway hook bridge MVP 完成 live verification 后再启动。
- [ ] 未执行的 live check 必须记录原因和剩余风险。

## 测试执行规则

- [ ] 默认测试只使用本地 deterministic fixture，不依赖真实 Discord token、真实 Hermes gateway、真实 GitHub、真实 tmux session 或外网状态。
- [ ] 单元测试优先覆盖纯逻辑：CLI parse、配置加载、事件规范化、隐私清洗、路由匹配、渲染、payload 构造。
- [ ] 契约测试必须锁定公开边界：CLI help/subcommand、HTTP ingress schema、Hermes hook payload、配置 schema、文档中的公开命令。
- [ ] 集成测试必须使用 fake sink、fake HTTP server、fake Hermes home、fake hermeship binary 和 fixture payloads。
- [ ] E2E smoke 只验证本地 binary/daemon/hook bridge 的最小闭环，不要求外部凭据。
- [ ] Live verification 与默认 CI 分离，真实 Discord/Hermes 结果只写入 `docs/live-verification.md`。
- [ ] 每个 bug fix 或行为变更都先补回归测试，再修实现，并确认测试在修复前能暴露问题或至少覆盖原始风险。
- [ ] 每个阶段完成前运行该阶段验证命令；发布前运行全局完成定义。

## 全局完成定义

- [ ] `cargo fmt --all -- --check` 通过。
- [ ] `cargo clippy --all-targets -- -D warnings` 通过。
- [ ] `cargo test` 通过。
- [ ] `cargo test` 不要求外部凭据、真实 Hermes、真实 Discord、真实 GitHub、真实 tmux 或非本地网络。
- [ ] `cargo run -- --help` 退出码为 0。
- [ ] `cargo run -- status` 能在 daemon 运行时返回健康信息。
- [ ] `cargo run -- send --channel <test-channel> --message "..."` 已通过 fake sink 或 live sink 验证。
- [ ] `cargo run -- emit hermes.agent.started --payload '{"session_id":"demo"}'` 能进入 daemon 队列。
- [ ] `cargo run -- explain hermes.agent.started --payload '{"session_id":"demo"}'` 能展示 route 匹配结果。
- [ ] `hermeship hermes install-hooks --home /tmp/hermeship-test-home --force` 写入 hook 文件。
- [ ] 在一次性 `HERMES_HOME` 中测试过 hook 加载和回滚路径。
- [ ] fake sink 记录过 daemon -> router -> renderer -> sink 的端到端 delivery。
- [ ] fake HTTP 覆盖过 Discord 2xx、4xx、5xx、rate limit、token 缺失、channel 缺失。
- [ ] fake Hermes home 与 fake hermeship binary 覆盖过 Python `handler.py` 的 stdin payload、binary missing、timeout、fail-open。
- [ ] Discord live delivery 已验证，或明确记录未验证原因。
- [ ] 日志和测试 fixture 中没有完整对话、prompt、provider 请求/响应、token、cookie 或 secret。
- [ ] 文档中的公开命令已被 CLI parse 测试、smoke 测试或 release preflight 覆盖。
- [ ] README、operations 文档和实际 CLI 一致。

## Milestone 0：契约与仓库基线

目标：先锁定“参考 clawhip、适配 Hermes”的真实边界，避免再次滑回 thin adapter 方向。

- [x] 确认当前分支和远程。
  - 命令：`git status --short --branch`
  - 完成标准：分支、远程和未提交变更清楚。
- [x] 复习 lessons。
  - 文件：`tasks/lessons.md`
  - 完成标准：确认 Hermeship 不是 thin adapter。
- [x] 固化 clawhip 参考路径。
  - 路径：`/Users/zq/Desktop/ai-projs/posp/template/clawhip`
  - 阅读：`ARCHITECTURE.md`、`Cargo.toml`、`src/cli.rs`、`src/main.rs`、`src/daemon.rs`、`src/events.rs`、`src/event/compat.rs`、`src/router.rs`、`src/render/default.rs`
  - 完成标准：记录可移植模块和必须替换的 OpenClaw/Codex/Claude 耦合点。
  - 记录：可移植模块包括 Rust 2024 CLI、daemon HTTP API、mpsc queue、typed event compat、dispatcher、multi-delivery router、renderer/sink split、config/lifecycle/release preflight；必须替换的耦合点包括 OpenClaw/Codex/Claude/OMC/OMX 命名、provider-native hook 契约、agent/session 事件别名、prompt delivery/tmux wrapper 语义。
- [x] 固化 Hermes 参考路径。
  - 路径：`/Users/zq/Desktop/ai-projs/posp/agents-contributions/hermes-agent`
  - 阅读：`gateway/hooks.py`、`hermes_cli/plugins.py`
  - 完成标准：确认 gateway hook 事件、context 字段、plugin hook 后续能力。
  - 补充阅读：`gateway/run.py`、`gateway/slash_commands.py`
  - 记录：gateway hook 支持 `gateway:startup`、`session:start`、`session:end`、`session:reset`、`agent:start`、`agent:step`、`agent:end`、`command:*`，hook 错误被捕获并记录，不阻塞主流程；plugin hook 提供 `on_session_start`、`on_session_end`、`pre_tool_call`、`post_tool_call`、`pre_llm_call`、`post_llm_call`、`api_request_error`、`subagent_start`、`subagent_stop` 等后续 observer 能力。
- [x] 确认项目技术栈。
  - 决策：Rust 2024 daemon-first，Python 只用于 Hermes hook bridge 模板。
  - 完成标准：记录到本清单决策记录。
- [x] 更新 README 项目定位。
  - 文件：`README.md`
  - 完成标准：说明 Hermeship 是 Hermes-native event router，不是 clawhip runtime client。
- [x] 验证 Milestone 0。
  - 命令：`rg -n "Hermes 到 clawhip 的适配层|通过 clawhip 已有 CLI 入口|python -m hermeship|src/hermeship|pyproject.toml|pytest|ruff|ClawhipClient|clawhip_client|HERMESHIP_CLAWHIP" docs/plans README.md`
  - 命令：`rg -n "Hermes 到 clawhip 的适配层|通过 clawhip 已有 CLI 入口|python -m hermeship|src/hermeship|pyproject.toml|pytest|ruff|ClawhipClient|clawhip_client|HERMESHIP_CLAWHIP" tasks/development-checklist.md | rg -v "rg -n"`
  - 预期：无旧 Python/thin-adapter 方案残留。
- [x] 提交 Milestone 0。
  - commit：`docs: 明确 hermeship 完整项目方向`

## Milestone 1：Rust 项目骨架与质量门禁

目标：建立与 clawhip 相似的 Rust CLI/daemon 工程骨架。

### 任务 1.1：Cargo 项目与 CLI 入口

- [x] 新建 Cargo metadata。
  - 新建：`Cargo.toml`
  - 包含：package metadata、Rust 2024、依赖 `anyhow`、`tokio`、`axum`、`clap`、`serde`、`serde_json`、`toml`、`reqwest`、`time`、`uuid`。
- [x] 新建基础源码文件。
  - 新建：`src/main.rs`
  - 新建：`src/cli.rs`
  - 新建：`src/lib.rs`
- [x] 实现最小 `hermeship --help`。
  - 子命令占位：`start`、`status`、`send`、`emit`、`explain`、`config`、`hermes`、`install`、`uninstall`、`release`。
- [x] 增加 CLI parse 单元测试。
  - 文件：`src/cli.rs`
  - 覆盖：`send`、`emit --payload`、`hermes hook --payload`、`hermes install-hooks`。
- [x] 增加 CLI 合约测试 fixture。
  - 新建：`tests/fixtures/cli/public_commands.txt`
  - 覆盖：`start`、`status`、`send`、`emit`、`explain`、`config`、`hermes hook`、`hermes install-hooks`、`install`、`uninstall`、`release preflight`。
  - 完成标准：fixture 中每条公开命令都能被 clap parse 或返回预期 help/error，不因文档漂移而静默失效。
- [x] 验证任务 1.1。
  - 命令：`cargo fmt --all -- --check`
  - 命令：`cargo test cli`
  - 命令：`cargo run -- --help`
- [x] 提交任务 1.1。
  - commit：`chore: 搭建 hermeship Rust CLI 骨架`

### 任务 1.2：配置模型

- [x] 新建配置模块。
  - 新建：`src/config.rs`
  - 包含：`AppConfig`、`DaemonConfig`、`ProvidersConfig`、`DiscordConfig`、`DefaultsConfig`、`PrivacyConfig`、`HermesConfig`、`RouteRule`。
- [x] 实现默认配置路径。
  - 默认：`~/.hermeship/config.toml`
  - 环境变量：`HERMESHIP_CONFIG`
- [x] 实现默认配置与 TOML 加载。
  - 要求：缺失配置返回默认值；非法 TOML 返回错误。
- [x] 实现 config CLI。
  - `hermeship config path`
  - `hermeship config show`
  - `hermeship config verify`
- [x] 编写配置测试。
  - 覆盖：默认值、env override、非法 TOML、未知 key、空 channel/token 归一化。
- [x] 验证任务 1.2。
  - 命令：`cargo test config`
  - 命令：`cargo run -- config show`
- [x] 提交任务 1.2。
  - commit：`feat: 实现 hermeship 配置模型与 config CLI`

### 任务 1.3：质量门禁与仓库基础

- [x] 增加 `.gitignore`。
  - 包含：`target/`、临时日志、测试输出。
- [x] 增加 rustfmt/clippy 约束说明。
  - 文件：`README.md` 或 `docs/development.md`
- [x] 增加测试 fixture 目录。
  - 新建：`tests/fixtures/hermes/`
  - 新建：`tests/fixtures/privacy/`
  - 新建：`tests/fixtures/routes/`
  - 新建：`tests/fixtures/discord/`
  - 新建：`tests/fixtures/cli/`
  - 完成标准：fixture 不包含真实 token、cookie、prompt、完整对话或 provider request/response body。
- [x] 确认基础门禁通过。
  - 命令：`cargo fmt --all -- --check`
  - 命令：`cargo clippy --all-targets -- -D warnings`
  - 命令：`cargo test`
- [x] 提交任务 1.3。
  - commit：`chore: 增加 Rust 质量门禁与仓库基础`

## Milestone 2：事件模型与兼容层

目标：实现 clawhip 风格的 `IncomingEvent -> typed EventEnvelope` 管道。

### 任务 2.1：IncomingEvent 与格式

- [x] 新建事件入口模型。
  - 新建：`src/events.rs`
  - 类型：`IncomingEvent`、`MessageFormat`、`RoutingMetadata`。
- [x] 实现 `MessageFormat`。
  - 支持：`compact`、`inline`、`alert`、`raw`。
- [x] 实现 `emit` 参数解析。
  - 支持：`--channel`、`--mention`、`--format`、`--template`、`--payload`、任意 `--key value`。
- [x] 编写测试。
  - 覆盖：payload JSON 合并、非法 format、奇数 key/value 拒绝、字段别名。
- [x] 增加事件 fixture。
  - 新建：`tests/fixtures/hermes/agent_start.json`
  - 新建：`tests/fixtures/hermes/session_end.json`
  - 新建：`tests/fixtures/hermes/invalid_payload.json`
  - 完成标准：fixture 能驱动 CLI emit、daemon ingress 和 hook normalization 测试。
- [x] 验证任务 2.1。
  - 命令：`cargo test events`
  - 命令：`cargo test cli`
  - 命令：`cargo fmt --all -- --check`
  - 命令：`cargo clippy --all-targets -- -D warnings`
  - 命令：`cargo test`
- [x] 提交任务 2.1。
  - commit：`feat: 增加 IncomingEvent 事件入口`

### 任务 2.2：Typed EventEnvelope

- [x] 新建 typed event 模块。
  - 新建：`src/event/mod.rs`
  - 新建：`src/event/body.rs`
  - 新建：`src/event/compat.rs`
- [x] 定义 `EventEnvelope`、`EventBody`、`EventMetadata`、`EventPriority`。
- [x] 实现 Hermes event body。
  - `HermesGatewayStarted`
  - `HermesSessionStarted`
  - `HermesSessionFinished`
  - `HermesSessionReset`
  - `HermesAgentStarted`
  - `HermesAgentStep`
  - `HermesAgentFinished`
  - `HermesAgentFailed`
  - `Custom`
- [x] 实现 canonical kind。
  - `gateway:startup` -> `hermes.gateway.started`
  - `session:start` -> `hermes.session.started`
  - `session:end` -> `hermes.session.finished`
  - `session:reset` -> `hermes.session.reset`
  - `agent:start` -> `hermes.agent.started`
  - `agent:step` -> `hermes.agent.step`
  - `agent:end` -> `hermes.agent.finished`
- [x] 编写 compat 测试。
  - 覆盖：所有 Hermes gateway hook event、未知 event -> custom、缺失 session_id 的降级。
- [x] 验证任务 2.2。
  - 命令：`cargo test event`
- [x] 提交任务 2.2。
  - commit：`feat: 实现 Hermes typed event model`

### 任务 2.3：隐私与 payload 清洗

- [x] 新建隐私模块。
  - 新建：`src/privacy.rs`
  - 函数：`sanitize_payload`、`redact_value`、`excerpt_policy`。
- [x] 实现敏感 key 递归脱敏。
  - key：`token`、`api_key`、`authorization`、`password`、`secret`、`cookie`。
- [x] 实现正文默认禁发。
  - 默认删除 `message`、`response`、`conversation_history`、`request`、`provider_response`、`tool_result`。
  - 默认保留 `message_chars`、`response_chars`、`has_message`、`has_response`。
- [x] 实现 opt-in 摘录。
  - 配置：`privacy.include_message_excerpt`、`privacy.include_response_excerpt`。
  - 要求：先脱敏，再截断。
- [x] 编写隐私测试。
  - 覆盖：短文本不原样泄漏、嵌套 secret、list、非字符串、原始 payload 不被原地修改。
- [x] 增加隐私回归 fixture。
  - 新建：`tests/fixtures/privacy/sensitive_payload.json`
  - 完成标准：测试断言输出不包含原始 `message`、`response`、`conversation_history`、`request`、`provider_response`、`tool_result`、token、cookie、secret。
- [x] 验证任务 2.3。
  - 命令：`cargo test privacy`
  - 命令：`cargo test event`
  - 命令：`cargo test events`
  - 命令：`cargo fmt --all -- --check`
  - 命令：`cargo clippy --all-targets -- -D warnings`
  - 命令：`cargo test`
- [x] 提交任务 2.3。
  - commit：`feat: 增加 Hermes 事件隐私清洗`

## Milestone 3：Daemon、队列与 HTTP ingress

目标：建立本地 daemon-first runtime。

### 任务 3.1：Daemon health 与 client

- [x] 新建 daemon/client 模块。
  - 新建：`src/daemon.rs`
  - 新建：`src/client.rs`
- [x] 实现 `hermeship start`。
  - 默认监听：`127.0.0.1:25295`。
- [x] 实现 `/health`。
  - 返回：version、status、queue 状态、configured sinks。
- [x] 实现 `hermeship status`。
  - 调用 daemon `/health`。
- [x] 编写 daemon health 测试。
  - 使用随机端口或 test server。
  - 覆盖：健康响应 schema、队列状态、configured sinks、daemon 未运行时 client 错误。
- [x] 验证任务 3.1。
  - 命令：`cargo test daemon`
  - 命令：`cargo run -- status`
  - 预期：daemon 未运行时返回清晰错误；不 panic。
- [x] 提交任务 3.1。
  - commit：`feat: 增加 hermeship daemon health`

### 任务 3.2：Event ingress 与队列

- [x] 实现 `/event`。
  - 接收 `IncomingEvent`。
  - 规范化并转为 `EventEnvelope`。
  - 写入 `tokio::mpsc` 队列。
- [x] 实现 `hermeship emit`。
  - 通过 client POST `/event`。
- [x] 实现 `hermeship send`。
  - 作为 custom event 发送。
- [x] 编写 ingress 测试。
  - 覆盖：有效事件入队、非法 payload 4xx、daemon unavailable 错误。
  - 要求：使用随机端口和本地 test queue，不绑定固定端口。
- [x] 验证任务 3.2。
  - 命令：`cargo test daemon`
  - 命令：`cargo test event`
  - 命令：`cargo run -- emit hermes.agent.started --payload '{"session_id":"demo"}'`
- [x] 提交任务 3.2。
  - commit：`0b63e49 feat: 增加 daemon event ingress`

### 任务 3.3：Hermes hook ingress

- [x] 实现 `/api/hermes/hook`。
  - 接收：`provider`、`source`、`event`、`context`。
  - 输出：标准 `IncomingEvent`。
- [x] 实现 `hermeship hermes hook --payload`。
  - 支持 stdin `--payload -`。
- [x] 编写 Hermes hook normalization 测试。
  - 覆盖：`gateway:startup`、`session:start`、`session:end`、`session:reset`、`agent:start`、`agent:end`。
  - 要求：复用 `tests/fixtures/hermes/`，并断言隐私清洗发生在入队前。
- [x] 验证任务 3.3。
  - 命令：`cargo test hermes`
  - 命令：`printf '%s' '{"event":"agent:start","context":{"session_id":"demo"}}' | cargo run -- hermes hook --payload -`
  - 命令：`cargo fmt --all -- --check`
  - 命令：`cargo clippy --all-targets -- -D warnings`
  - 命令：`cargo test`
- [x] 提交任务 3.3。
  - commit：`7b10816 feat: 增加 Hermes hook ingress`

## Milestone 4：Router、Renderer、Dispatcher

目标：移植 clawhip 核心事件分发管道。

### 任务 4.1：Router

- [x] 新建 router 模块。
  - 新建：`src/router.rs`
  - 类型：`ResolvedDelivery`、`SinkTarget`、`DeliveryExplanation`。
- [x] 实现 route match。
  - 支持 event glob。
  - 支持 filter map。
  - 支持 0..N delivery。
- [x] 实现 route candidates。
  - `hermes.agent.*`、`hermes.session.*`、`hermes.*`。
- [x] 实现 `hermeship explain`。
  - 展示 matched routes、failed filters、delivery target。
- [x] 编写 router 测试。
  - 覆盖：多 route、filter 命中/未命中、缺 channel、template/format/mention 继承。
- [x] 编写 `explain` 合约测试。
  - 覆盖：route 命中原因、filter 失败原因、无 route、delivery target。
  - 完成标准：`explain` 输出可用于定位配置问题，不只返回布尔结果。
- [x] 验证任务 4.1。
  - 命令：`cargo test router`
  - 命令：`cargo run -- explain hermes.agent.started --payload '{"platform":"telegram","session_id":"demo"}'`
- [x] 提交任务 4.1。
  - commit：`864e7f4 feat: 实现多投递路由`

### 任务 4.2：Renderer

- [x] 新建 render 模块。
  - 新建：`src/render/mod.rs`
  - 新建：`src/render/default.rs`
- [x] 实现默认 renderer。
  - 支持：`compact`、`inline`、`alert`、`raw`。
- [x] 实现 Hermes 事件渲染。
  - gateway/session/agent/custom。
- [x] 实现 template 渲染。
  - 支持 `{session_id}`、`{platform}`、`{project}`、`{event}` 等上下文 token。
- [x] 编写 renderer 测试。
  - 覆盖：所有格式、缺字段降级、raw JSON、template token。
- [x] 验证任务 4.2。
  - 命令：`cargo test render`
- [x] 提交任务 4.2。
  - commit：`feat: 增加 Hermes 默认渲染器`

### 任务 4.3：Dispatcher 与 fake sink

- [x] 新建 dispatch/sink 模块。
  - 新建：`src/dispatch.rs`
  - 新建：`src/sink/mod.rs`
  - 新建：`src/sink/fake.rs`
- [x] 实现 `Sink` trait。
- [x] 实现 fake sink。
  - 用于测试保存 delivery。
- [x] 实现 dispatcher。
  - 从队列读取 event。
  - route -> render -> sink。
  - 单个 delivery 失败不影响其他 delivery。
- [x] 编写 dispatcher 测试。
  - 覆盖：多投递、单 sink failure、无 route、render failure。
- [x] 编写 dispatcher E2E 测试。
  - 使用：`tests/fixtures/hermes/agent_start.json` + fake sink。
  - 断言：daemon 入队事件经过 route -> render -> fake sink 后保存 delivery，且 message 不泄漏敏感字段。
- [x] 验证任务 4.3。
  - 记录：原计划命令 `cargo test dispatch sink` 为无效 Cargo 语法，实际验证已拆分执行。
  - 命令：`cargo test dispatch`
  - 命令：`cargo test sink`
  - 命令：`cargo fmt --all -- --check`
  - 命令：`cargo clippy --all-targets -- -D warnings`
  - 命令：`cargo test`
- [x] 提交任务 4.3。
  - commit：`feat: 实现事件 dispatcher 与 fake sink`

## Milestone 5：Discord Sink 与基础 Live Path

目标：实现第一条真实通知链路。

### 任务 5.1：Discord 配置与 payload

- [x] 新建 Discord sink。
  - 新建：`src/sink/discord.rs`
- [x] 支持 bot token + channel。
- [x] 支持 webhook URL。
- [x] 实现 Discord message payload。
  - 内容长度限制。
  - mention 前缀。
  - allowed mentions 策略。
- [x] 编写 Discord sink 单元测试。
  - 使用 fake HTTP 或 request builder 测试。
  - 覆盖：webhook payload、bot channel payload、allowed mentions、消息长度截断。
- [x] 验证任务 5.1。
  - 命令：`cargo test discord`
  - 命令：`cargo fmt --all -- --check`
  - 命令：`cargo clippy --all-targets -- -D warnings`
  - 命令：`cargo test`
- [x] 提交任务 5.1。
  - commit：`0cd6e4e feat: 增加 Discord sink`

### 任务 5.2：Sink 失败语义

- [x] 测试 token 缺失。
  - 预期：delivery failed，不 panic。
- [x] 测试非 2xx。
  - 预期：记录 status/body tail。
- [x] 测试 rate limit。
  - 预期：尊重 retry 信息或记录明确诊断。
- [x] 使用 fake HTTP server 覆盖 Discord 失败矩阵。
  - 覆盖：2xx、4xx、5xx、429 rate limit、空 token、空 channel。
  - 完成标准：默认测试不访问真实 Discord API。
- [x] 测试多个 delivery 其中一个失败。
  - 预期：其他 delivery 继续。
- [x] 验证任务 5.2。
  - 命令：`cargo test sink`
  - 命令：`cargo test dispatch`
  - 命令：`cargo fmt --all -- --check`
  - 命令：`cargo clippy --all-targets -- -D warnings`
  - 命令：`cargo test`
- [x] 提交任务 5.2。
  - commit：`ea9b789 feat: 完善 sink 失败处理`

### 任务 5.3：本地端到端 smoke

- [x] 编写 daemon + fake sink E2E。
  - 启动 test daemon。
  - POST `/api/hermes/hook`。
  - 断言 fake sink 收到渲染消息。
  - 断言默认隐私保护生效。
- [x] 验证 `send`。
  - 命令：`cargo test daemon`
  - 覆盖：`daemon_send_command_posts_custom_event_to_daemon` 使用本地 test daemon 和 client 验证 `send` 构造 custom event 并 POST `/event`。
- [x] 验证 `emit`。
  - 命令：`cargo test daemon`
  - 覆盖：`daemon_emit_command_posts_event_to_daemon` 使用本地 test daemon 和 client 验证 `emit` 构造 Hermes event 并 POST `/event`。
- [x] 验证任务 5.3。
  - 命令：`cargo test dispatch`
  - 命令：`cargo test daemon`
  - 命令：`cargo fmt --all -- --check`
  - 命令：`cargo clippy --all-targets -- -D warnings`
  - 命令：`cargo test`
- [x] 提交任务 5.3。
  - commit：`test: 增加 daemon 到 sink 的端到端覆盖`

## Milestone 6：Hermes Hook Bridge 安装

目标：让 Hermes gateway 能通过 hook bridge 投递到 Hermeship。

### 任务 6.1：Hook 模板

- [x] 创建 Hermes hook 模板目录。
  - 新建：`templates/hermes-hook/HOOK.yaml`
  - 新建：`templates/hermes-hook/handler.py`
- [x] `HOOK.yaml` 声明事件。
  - `gateway:startup`
  - `session:start`
  - `session:end`
  - `session:reset`
  - `agent:start`
  - `agent:end`
  - 记录：`agent:step` 与 `command:*` 当前不进入默认安装模板，因为 `enable_agent_step` 与 `enable_command_events` 默认关闭。
- [x] `handler.py` 只使用标准库。
  - 不 import Hermeship package。
  - 调用 `hermeship hermes hook --payload -`。
  - 超时 fail-open。
- [x] 编写模板测试。
  - 覆盖：manifest 可解析、handler 包含 fail-open 逻辑、不包含 secret。
  - 要求：`handler.py` 只使用 Python 标准库。
- [x] 验证任务 6.1。
  - 命令：`cargo test hooks`
- [x] 提交任务 6.1。
  - 记录：随 Milestone 6 阶段提交统一归档。

### 任务 6.2：Installer

- [x] 新建 hooks installer。
  - 新建：`src/hooks.rs`
  - 函数：`install_hermes_hooks(options)`。
- [x] 实现 CLI。
  - `hermeship hermes install-hooks --home <path> --force`
- [x] 支持 dry-run。
  - 打印将写入的文件，不修改磁盘。
- [x] 编写 installer 测试。
  - 覆盖：首次安装、不覆盖、force 覆盖、dry-run、返回路径。
  - 使用：fake Hermes home。
- [x] 验证任务 6.2。
  - 命令：`cargo test hooks`
  - 命令：`cargo run -- hermes install-hooks --home /tmp/hermeship-test-home --force`
  - 命令：`find /tmp/hermeship-test-home/hooks/hermeship -maxdepth 1 -type f -print`
- [x] 提交任务 6.2。
  - 记录：随 Milestone 6 阶段提交统一归档。

### 任务 6.3：Bridge smoke 与回滚

- [x] 编写 Python handler smoke test。
  - 使用临时 `HERMES_HOME`。
  - 直接 import/exec handler module。
  - fake `hermeship` binary 验证 stdin payload。
  - 覆盖：binary missing、调用 timeout、子进程失败时 fail-open。
- [x] 实现 uninstall/remove hooks。
  - `hermeship hermes uninstall-hooks --home <path>`
- [x] 编写回滚测试。
  - 安装 -> 卸载 -> 确认目录删除或 marker 删除。
- [x] 验证任务 6.3。
  - 命令：`cargo test hooks`
  - 命令：`cargo run -- hermes uninstall-hooks --home /tmp/hermeship-test-home`
- [x] 提交任务 6.3。
  - 记录：随 Milestone 6 阶段提交统一归档。

## Milestone 7：安装、生命周期与运维 CLI

目标：补齐 clawhip 风格的可运维项目表面。

### 任务 7.1：Install/Setup

- [x] 实现 `hermeship install`。
  - 创建 `~/.hermeship`。
  - scaffold `config.toml`。
  - 输出下一步命令。
- [x] 实现 `hermeship setup`。
  - 支持设置 Discord token、default channel、daemon URL。
  - 不打印 secret。
- [x] 编写 install/setup 测试。
  - 使用临时 HOME。
- [x] 验证任务 7.1。
  - 命令：`cargo test lifecycle`
- [x] 提交任务 7.1。
  - commit：`feat: 增加 hermeship install setup`
  - 记录：本阶段合并为 Milestone 7 统一提交。

### 任务 7.2：Service 与 Uninstall

- [x] 增加 systemd service 模板。
  - 新建：`deploy/hermeship.service`
- [x] 增加 launchd 文档或模板。
  - macOS 先文档化，是否实现视环境决定。
- [x] 实现 `hermeship uninstall`。
  - 可选删除 config/state/service/hooks。
- [x] 编写 lifecycle 测试。
  - 覆盖：不误删、force/remove-config 行为。
- [x] 验证任务 7.2。
  - 命令：`cargo test lifecycle`
- [x] 提交任务 7.2。
  - commit：`feat: 增加安装生命周期管理`
  - 记录：本阶段合并为 Milestone 7 统一提交。

### 任务 7.3：Release preflight

- [x] 新建 release preflight。
  - 新建：`src/release_preflight.rs`
- [x] 检查项目一致性。
  - CLI help。
  - 配置示例。
  - docs 命令。
  - hook 模板包含。
  - 测试夹具完整性。
  - live verification 必填字段。
- [x] 实现 CLI。
  - `hermeship release preflight <version>`
- [x] 验证任务 7.3。
  - 命令：`cargo run -- release preflight 0.1.0`
- [x] 提交任务 7.3。
  - commit：`chore: 增加 release preflight`
  - 记录：本阶段合并为 Milestone 7 统一提交。

## Milestone 8：clawhip 功能 Parity 扩展

目标：按 clawhip 能力补齐非 Hermes 专属 sources。

### 任务 8.1：Git Source

- [x] 新建 git source。
  - 新建：`src/source/git.rs`
- [x] 实现 commit/branch 事件。
- [x] 实现 `hermeship git commit` 和 `hermeship git branch-changed`。
- [x] 编写测试。
  - 覆盖：repo/branch/commit summary、路由 metadata。
- [x] 验证任务 8.1。
  - 命令：`cargo test git`
- [x] 提交任务 8.1。
  - commit：`feat: 增加 git 事件 source`
  - 记录：`1536b6a feat: 增加 Git Source 本地事件路径`

### 任务 8.2：GitHub Source

- [x] 新建 GitHub source。
  - 新建：`src/source/github.rs`
- [x] 实现 issue/PR/CI/release 事件。
- [x] 实现 `hermeship github ...` CLI。
- [x] 编写测试。
  - 使用 fixture，不依赖真实 GitHub。
- [x] 验证任务 8.2。
  - 命令：`cargo test github`
- [x] 提交任务 8.2。
  - commit：`feat: 增加 GitHub 事件 source`
  - 记录：`91d13d8 feat: 完成 GitHub Source 本地确定性路径并修复回归`

### 任务 8.3：Tmux Source

- [x] 新建 tmux source。
  - 新建：`src/source/tmux.rs`
- [x] 实现 keyword/stale 事件。
- [x] 实现 `hermeship tmux keyword/stale/watch/list`。
- [x] 编写测试。
  - fake tmux 命令输出。
- [x] 验证任务 8.3。
  - 命令：`cargo test tmux`
- [x] 提交任务 8.3。
  - commit：`feat: 增加 tmux 事件 source`
  - 记录：`3745bb8 feat: 增加 tmux 事件 source`

### 任务 8.4：Cron 与 Memory Scaffold

- [x] 新建 cron 模块。
  - 新建：`src/cron.rs`
- [x] 支持 configured cron job run。
- [x] 新建 memory scaffold。
  - 新建：`src/memory.rs`
  - CLI：`hermeship memory init/status`
- [x] 编写测试。
- [x] 验证任务 8.4。
  - 命令：`cargo test cron`
  - 命令：`cargo test memory`
- [x] 提交任务 8.4。
  - commit：`feat: 增加 cron 与 memory scaffold`
  - 记录：随本阶段提交完成

## Milestone 9：文档与 Live Verification

目标：证明 Hermeship 的真实使用路径可操作、可回滚。

### 任务 9.1：README 与运维文档

- [ ] 重写 README。
  - 内容：项目定位、安装、配置、启动 daemon、安装 Hermes hooks、send/emit/explain、live check。
- [ ] 新增 operations 文档。
  - 新建：`docs/operations.md`
  - 内容：安装、更新、回滚、常见故障。
- [ ] 新增 event contract 文档。
  - 新建：`docs/hermes-event-contract.md`
  - 内容：Hermes hook input、canonical events、payload 字段、隐私规则。
- [ ] 新增 architecture 文档。
  - 新建：`ARCHITECTURE.md`
  - 参考 clawhip，但使用 Hermeship 实际模块。
- [ ] 验证任务 9.1。
  - 命令：`rg -n "hermeship start|hermes install-hooks|hermes.agent|Discord|rollback" README.md docs ARCHITECTURE.md`
- [ ] 提交任务 9.1。
  - commit：`docs: 增加 Hermeship 运维与事件契约`

### 任务 9.2：Live Verification Runbook

- [ ] 新增 live verification 文档。
  - 新建：`docs/live-verification.md`
- [ ] 记录 fake sink 验证。
- [ ] 记录 daemon health 验证。
- [ ] 记录 Discord live 验证。
- [ ] 记录 Hermes gateway hook smoke。
- [ ] 记录回滚验证。
- [ ] 每条 live 记录包含 commit、时间、测试频道、触发事件、实际消息形态、未执行项和剩余风险。
- [ ] live 记录不得包含 token、cookie、secret、完整 prompt、完整对话或 provider request/response body。
- [ ] 验证任务 9.2。
  - 命令：`rg -n "HERMES_HOME|Discord|hermeship status|agent:start|rollback" docs/live-verification.md`
- [ ] 提交任务 9.2。
  - commit：`docs: 增加 live verification runbook`

### 任务 9.3：首次 Live Check

- [ ] 启动 Hermeship daemon。
  - 命令：`hermeship start`
- [ ] 确认 daemon status。
  - 命令：`hermeship status`
- [ ] 发送 custom live message。
  - 命令：`hermeship send --channel <id> --message "hermeship live check"`
- [ ] 发送 Hermes sample event。
  - 命令：`hermeship emit hermes.agent.started --payload '{"session_id":"live-check"}'`
- [ ] 安装 Hermes hooks。
  - 命令：`hermeship hermes install-hooks --force`
- [ ] 触发真实 Hermes gateway event。
  - 记录：平台、频道、时间、消息形态。
- [ ] 执行回滚。
  - 命令：`hermeship hermes uninstall-hooks`
- [ ] 如凭据不可用，记录阻塞原因和剩余风险。
- [ ] 提交 live verification 记录。
  - commit：`docs: 记录 Hermeship live verification 结果`

## Milestone 10：Hermes Plugin / Observer 研究

目标：在 gateway hook MVP 稳定后，增加更高保真 telemetry。

门禁：Milestone 1-9 完成或被明确豁免前，不启动本阶段。

### 任务 10.1：Observer 契约研究

- [ ] 复读 Hermes plugin 文档和源码。
  - 文件：`/Users/zq/Desktop/ai-projs/posp/agents-contributions/hermes-agent/hermes_cli/plugins.py`
- [ ] 确认 plugin 安装与启用机制。
  - 重点：`~/.hermes/plugins`、`plugins.enabled`、entry point。
- [ ] 起草 observer event mapping。
  - 新建：`docs/observer-plugin.md`
- [ ] 决定 observer mode 进入 v0.2 还是更晚。
  - 记录决策到本清单。

### 任务 10.2：Observer Plugin MVP

- [ ] 创建 plugin 模板。
  - 新建：`templates/hermes-plugin/plugin.yaml`
  - 新建：`templates/hermes-plugin/__init__.py`
- [ ] 支持 hook。
  - `pre_tool_call`
  - `post_tool_call`
  - `post_llm_call`
  - `api_request_error`
  - `subagent_start`
  - `subagent_stop`
- [ ] 默认隐私保护。
  - 不转发 request/response body。
- [ ] 编写 plugin smoke test。
- [ ] 提交 observer plugin。
  - commit：`feat: 增加可选 Hermes observer plugin`

## 运行状态日志

最新记录放在最上方。

### 2026-06-17 - Milestone 8.4 Cron 与 Memory Scaffold 本地 deterministic parity

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认启动时工作树干净，最近提交为 `6c9af3e docs: 更新 Hermeship Milestone 8.4 交接状态`、`3745bb8 feat: 增加 tmux 事件 source`、`9cf4341 docs: 更新 Hermeship Milestone 8.3 交接状态`。
- [x] 已阅读 `src/cli.rs`、`src/main.rs`、`src/config.rs`、`src/events.rs`、`src/event/`、`src/source/git.rs`、`src/source/github.rs`、`src/source/tmux.rs`、`src/router.rs`、`src/render/`、`src/dispatch.rs`、`src/lifecycle.rs`、`src/release_preflight.rs`、`tests/fixtures/README.md` 与方案文档 parity/source 章节。
- [x] 已先写失败测试并运行 Red：`cargo test cron` 在实现前失败于缺少 `CronCommands`、`CronConfig`、`CronJob`、`EventBody::CronRun` 和 `configured_run_event()`；`cargo test memory` 在实现前失败于缺少 `MemoryCommands`、memory CLI enum 和 memory API。
- [x] 已新增 `src/cron.rs`，实现 configured cron job run 的本地 deterministic `cron.run` 事件构造；只读取本地配置，不实现真实 scheduler、系统 cron 或外部 cron daemon。
- [x] 已新增 `CronConfig` / `CronJob` 配置模型，并接入配置归一化与验证；默认没有 cron jobs，保持向后兼容。
- [x] 已新增 typed cron body：`CronRunEvent`，并将 `cron.run` 接入 `IncomingEvent -> EventEnvelope` conversion、route metadata、默认 compact/raw 渲染和 daemon submit 路径。
- [x] 已新增 `src/memory.rs`，实现 `hermeship memory init/status` 的本地 filesystem scaffold：`MEMORY.md`、`memory/README.md`、daily/project/topic shards、可选 channel/agent shards，以及 handoffs/archive `.gitkeep`。
- [x] 已确认 memory scaffold 要求显式 `--date <YYYY-MM-DD>`，默认不覆盖现有文件，只有 `--force` 覆盖生成文件；slug/date validation 防止路径穿越和非 deterministic 名称。
- [x] 已根据代码审查补充 symlink 防护：memory root、目录、文件写入和 markdown 扫描均拒绝 symlink，避免 `--force` 写到 root 外或 `status` 扫描 root 外目标；并补充真实日历日期校验。
- [x] 已更新公开命令 fixture、release preflight、README、方案文档 CLI 示例和开发状态文档，覆盖 `cron run`、`memory init`、`memory status`。
- [x] 已运行验证：`cargo test cron`、`cargo test memory`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- [x] 已确认本阶段没有实现真实 scheduler、系统 cron 安装、数据库 memory store、真实 live verification、Slack sink 或 Hermes plugin/observer。
- [x] 提交状态：随本阶段提交完成。

### 2026-06-17 - Milestone 8.3 完成后交接更新

- [x] 已确认当前分支为 `codex/milestone-1-cli`，文档更新前工作树干净，最近提交为 `3745bb8 feat: 增加 tmux 事件 source`、`9cf4341 docs: 更新 Hermeship Milestone 8.3 交接状态`、`91d13d8 feat: 完成 GitHub Source 本地确定性路径并修复回归`。
- [x] 已更新 `docs/development-status.md`，明确 Milestone 0 到 Milestone 8.3 已完成，Milestone 8.4 到 Milestone 10 未完成，下一入口为 Milestone 8.4 Cron 与 Memory Scaffold。
- [x] 已更新 `tasks/todo.md`，将当前工作台切换并固定为 Milestone 8.4 Cron 与 Memory Scaffold。
- [x] 已确认本次只做文档交接更新，不进入 Milestone 8.4 实现，不执行真实 live verification、Slack sink、真实 tmux watch 或 Hermes plugin/observer。
- [x] 本次文档验证：`git diff --check`、状态文档一致性搜索、`cargo run -- release preflight 0.1.0`。

### 2026-06-17 - Milestone 8.3 Tmux Source 本地 deterministic parity

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认启动时工作树干净，最近提交为 `9cf4341 docs: 更新 Hermeship Milestone 8.3 交接状态`、`91d13d8 feat: 完成 GitHub Source 本地确定性路径并修复回归`、`9d8b05c docs: 更新 Hermeship Milestone 8.2 交接入口`。
- [x] 已阅读 `src/cli.rs`、`src/main.rs`、`src/config.rs`、`src/events.rs`、`src/event/`、`src/source/git.rs`、`src/source/github.rs`、`src/router.rs`、`src/render/`、`src/dispatch.rs`、`src/lifecycle.rs`、`src/release_preflight.rs`、`tests/fixtures/README.md` 与方案文档 parity/source 章节。
- [x] 已先写失败测试并运行 Red：`cargo test tmux` 在实现前失败于缺少 `source::tmux` API、`TmuxCommands`、`Commands::Tmux` 和 tmux typed body variants；审查回归测试在修复前失败于 `watch/list` 报表原样回显 command/last_line。
- [x] 已新增 `src/source/tmux.rs`，实现 deterministic `keyword_event`、`stale_event`、`parse_tmux_panes_output`、`watch_plan_from_output`、`format_watch_plan` 和 `format_pane_list`；只处理显式 fake tmux 输出，不调用真实 `tmux`、不读取真实 session、不启动真实 watch loop。
- [x] 已新增 typed tmux body：`TmuxKeywordEvent`、`TmuxStaleEvent`，并将 `tmux.keyword`、`tmux.stale` 接入 `IncomingEvent -> EventEnvelope` conversion；`tmux.stale` 标记为 high priority。
- [x] 已接入 CLI：`hermeship tmux keyword`、`hermeship tmux stale`、`hermeship tmux watch`、`hermeship tmux list`；keyword/stale 复用现有 `DaemonClient::post_event()` 投递 `/event`，watch/list 只输出本地 deterministic 报表。
- [x] 已扩展 router 与默认 renderer：tmux route filter 可用 session/session_name、window、pane、keyword、minutes；compact/raw 输出受控摘要。
- [x] 已根据代码审查收紧 `watch/list` 报表隐私边界：不再原样输出 fake tmux input 中的 command 或 last_line，只输出 command 是否存在和 last_line 字符数，并补充 token/path/authorization 回归测试。
- [x] 已更新公开命令 fixture、release preflight 和 README/方案 CLI 示例，要求覆盖四个 tmux 公开命令；README watch/list 示例改为可复制的 tab 分隔 `$'...'` 形式。
- [x] 已运行验证：`cargo test tmux`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 均通过。
- [x] 已确认本阶段没有实现真实 tmux session 读取、真实 tmux watch、cron、memory、真实 live verification、Slack sink 或 Hermes plugin/observer。
- [x] 提交状态：已提交 `3745bb8 feat: 增加 tmux 事件 source`。

### 2026-06-17 - Milestone 8.2 完成后交接更新

- [x] 已复习 `tasks/lessons.md`，确认阶段完成后必须验证并提交、方案/清单分离、Hermeship 不是 thin adapter。
- [x] 已确认当前分支为 `codex/milestone-1-cli`，文档更新前工作树干净，最近提交为 `91d13d8 feat: 完成 GitHub Source 本地确定性路径并修复回归`、`9d8b05c docs: 更新 Hermeship Milestone 8.2 交接入口`、`a6bd734 docs: 更新 Hermeship Milestone 8.1 交接状态`。
- [x] 已更新 `docs/development-status.md`，明确 Milestone 0 到 Milestone 8.2 已完成并提交，Milestone 8.3 到 Milestone 10 未完成，下一入口为 Milestone 8.3 Tmux Source。
- [x] 已更新 `tasks/todo.md`，将当前工作台切换并固定为 Milestone 8.3 Tmux Source 本地 deterministic parity。
- [x] 已确认本次只做文档交接更新，不进入 Milestone 8.3 实现，不执行真实 live verification、Slack sink、真实 GitHub/tmux 路径或 Hermes plugin/observer。
- [x] 本次文档验证：`git diff --check`、过期 8.2/旧功能提交状态搜索、`git status --short --branch`。

### 2026-06-17 - Milestone 8.2 GitHub Source 本地 deterministic parity

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认启动时工作树干净，最近提交为 `9d8b05c docs: 更新 Hermeship Milestone 8.2 交接入口`、`a6bd734 docs: 更新 Hermeship Milestone 8.1 交接状态`、`1536b6a feat: 增加 Git Source 本地事件路径`。
- [x] 已阅读 `src/cli.rs`、`src/main.rs`、`src/config.rs`、`src/events.rs`、`src/event/`、`src/source/git.rs`、`src/router.rs`、`src/render/`、`src/dispatch.rs`、`src/lifecycle.rs`、`src/release_preflight.rs`、`tests/fixtures/README.md` 与方案文档 parity/source 章节。
- [x] 已先写失败测试并运行 Red：`cargo test github` 在实现前失败于缺少 `source::github` API、`GithubCommands`、`Commands::Github` 和 GitHub typed body variants。
- [x] 已新增 `src/source/github.rs`，实现 deterministic `issue_opened_event`、`pull_request_opened_event`、`check_failed_event` 和 `release_published_event`；只构造 `IncomingEvent`，不访问真实 GitHub API、不依赖外网、不读取 token 或 webhook secret。
- [x] 已新增 typed GitHub body：`GithubIssueEvent`、`GithubPullRequestEvent`、`GithubCheckEvent`、`GithubReleaseEvent`，并将 `github.issue-opened`、`github.pr-opened`、`github.check-failed`、`github.release-published` 接入 `IncomingEvent -> EventEnvelope` conversion。
- [x] 已接入 CLI：`hermeship github issue-opened`、`hermeship github pr-opened`、`hermeship github check-failed`、`hermeship github release-published`，命令通过现有 `DaemonClient::post_event()` 投递到 `/event`。
- [x] 已扩展 router 与默认 renderer：GitHub route filter 可用 owner、repo_name、number、branch、base_branch、workflow、status、tag 等结构化字段；compact 输出安全摘要；raw JSON 只输出受控字段，不展开 issue/PR body、URL、token、secret 或 provider response。
- [x] 已根据代码审查修复 GitHub route metadata poisoning：router filter 中的 `repo_name` 由已校验 typed body 覆盖，避免直接 POST 用原始 `repo_name` 绕过 body repo。
- [x] 已更新公开命令 fixture、release preflight 和方案 CLI 示例，要求覆盖四个 GitHub 公开命令。
- [x] 已运行验证：`cargo test github`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 均通过。
- [x] 已确认本阶段没有实现真实 GitHub API source、GitHub webhook receiver、GitHub credential handling、tmux source、cron、memory、真实 live verification、Slack sink 或 Hermes plugin/observer。
- [x] 提交状态：已提交 `91d13d8 feat: 完成 GitHub Source 本地确定性路径并修复回归`。

### 2026-06-17 - Milestone 8.1 完成后交接更新

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`。
- [x] 已确认当前分支为 `codex/milestone-1-cli`，文档更新前工作树干净，最近提交为 `a6bd734 docs: 更新 Hermeship Milestone 8.1 交接状态`、`1536b6a feat: 增加 Git Source 本地事件路径`、`475f2a3 docs: 更新 Hermeship Milestone 8 开发入口`。
- [x] 已确认 Milestone 8.1 Git Source 本地 deterministic parity 已完成并提交，最新功能阶段提交为 `1536b6a feat: 增加 Git Source 本地事件路径`。
- [x] 已更新 `docs/development-status.md`，明确当前交接工作台已切换到 Milestone 8.2 GitHub Source。
- [x] 已更新 `tasks/todo.md`，从已完成的 Milestone 8.1 工作台切换为 Milestone 8.2 GitHub Source 执行计划，并保留 8.1 Review。
- [x] 历史记录：当时未完成范围为 Milestone 8.2 GitHub Source、Milestone 8.3 tmux、Milestone 8.4 cron/memory、Milestone 9 文档与 live verification、Milestone 10 Hermes plugin/observer；其中 Milestone 8.2 已在 `91d13d8` 完成。
- [x] 本次只做文档交接更新，不进入 Milestone 8.2 实现，不修改功能代码，不执行真实 live verification、Slack sink 或 Hermes plugin/observer。
- [x] 本次文档验证：`git diff --check`、`git status --short --branch`。

### 2026-06-17 - Milestone 8.1 Git Source 本地 deterministic parity

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认启动时工作树干净，最近提交为 `475f2a3 docs: 更新 Hermeship Milestone 8 开发入口`、`162efcd feat: 增加安装生命周期与发布预检`、`64e8641 docs: 更新 Hermeship 最新开发状态`。
- [x] 已阅读 `src/cli.rs`、`src/main.rs`、`src/config.rs`、`src/events.rs`、`src/event/`、`src/router.rs`、`src/render/`、`src/dispatch.rs`、`src/lifecycle.rs`、`src/release_preflight.rs`、`tests/fixtures/README.md` 与方案文档 parity/source 章节。
- [x] 已先写失败测试并运行 Red：`cargo test git` 在实现前失败于缺少 `source::git` API、`GitCommands`、`Commands::Git` 和 `EventBody::GitCommit` / `GitBranchChanged` variants。
- [x] 已新增 `src/source/mod.rs` 与 `src/source/git.rs`，实现 deterministic `commit_event` 和 `branch_changed_event`，只构造 `IncomingEvent`，不执行真实 `git` 命令、不轮询 repo、不访问远端。
- [x] 已根据代码审查补强 Git 输入校验：source 与 compat 均拒绝非 7-64 hex commit、空 summary、多行 summary 和过长 display field，避免畸形 Git payload 绕过 raw renderer 字段级防护。
- [x] 已新增 typed Git body：`GitCommitEvent`、`GitBranchChangedEvent`，并将 `git.commit` 与 `git.branch-changed` 接入 `IncomingEvent -> EventEnvelope` conversion。
- [x] 已接入 CLI：`hermeship git commit` 与 `hermeship git branch-changed`，命令通过现有 `DaemonClient::post_event()` 投递到 `/event`。
- [x] 已扩展默认 renderer 和 raw JSON：Git compact 输出 repo/branch/short commit/summary/author 摘要；raw 只输出受控字段，不展开完整 diff、commit body、repo path、worktree path 或 author email。
- [x] 已覆盖 source 构造、typed conversion、route metadata filter、CLI parse、public command fixture、daemon submit 和 renderer 隐私边界。
- [x] 已更新 release preflight 的公开命令检查，要求 fixture 覆盖 `git commit` 与 `git branch-changed`。
- [x] 已运行验证：`cargo test git`（11 lib-filtered tests + 2 bin-filtered tests passed）、`cargo test release_preflight`（6 passed）、`cargo run -- release preflight 0.1.0`（本地 checks ok，live verification pending）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（150 lib tests + 10 bin tests passed）均通过。
- [x] 已确认本阶段没有实现真实 git polling source、GitHub source、tmux source、cron、memory、真实 live verification、Slack sink 或 Hermes plugin/observer。
- [x] 提交状态：已提交 `1536b6a feat: 增加 Git Source 本地事件路径`。

### 2026-06-16 - Milestone 7 完成后交接更新

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`。
- [x] 已确认当前分支为 `codex/milestone-1-cli`，更新前工作树干净，最新提交为 `162efcd feat: 增加安装生命周期与发布预检`。
- [x] 已更新 `docs/development-status.md`，明确 Milestone 0 到 Milestone 7 已完成并提交，Milestone 8 到 Milestone 10 未完成。
- [x] 已更新 `tasks/todo.md`，切换为 Milestone 8 的下次启动工作台，并记录当前基线、边界、下次执行计划和 Review。
- [x] 已修正文档中过期的 Milestone 7 提交引用，统一为实际 HEAD `162efcd`。
- [x] 本次只做文档交接更新，不进入 Milestone 8 实现，不修改功能代码，不执行真实 live verification、Slack sink 或 Hermes plugin/observer。
- [x] 本次文档验证：`git diff --check`、过期 Milestone 7 提交号搜索、`git status --short --branch`。

### 2026-06-16 - Milestone 7 安装、生命周期与运维 CLI

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认启动时工作树干净，最近提交为 `64e8641 docs: 更新 Hermeship 最新开发状态`、`f6f98a3 feat: 支持 Hermes hook bridge 安装`、`1ad3c7c docs: 更新 Hermeship Milestone 6 开发入口`。
- [x] 已阅读 `src/cli.rs`、`src/main.rs`、`src/hooks.rs`、`src/config.rs`、`src/daemon.rs`、`src/client.rs`、`tests/fixtures/README.md`、方案文档安装/回滚章节，并参考 `template/clawhip` lifecycle/preflight 模式。
- [x] 已先写失败测试并运行 Red：`cargo test lifecycle` 在实现前失败于缺少 `InstallOptions`、`SetupOptions`、`UninstallOptions`、`install/setup/uninstall` 和 `SERVICE_TEMPLATE`；`cargo test release_preflight` 在实现前失败于缺少 preflight API。
- [x] 已新增 `src/lifecycle.rs`，实现本地 deterministic `install`、`setup`、`uninstall`，支持 dry-run、force、显式删除开关、stdin/env token 输入和 token 输出脱敏。
- [x] 已新增 `deploy/hermeship.service` 和 `docs/operations.md`，记录 systemd user service 模板与 launchd 手动示例；本阶段不执行真实 service manager 操作。
- [x] 已新增 `src/release_preflight.rs`，覆盖 Cargo 版本一致性、公开 CLI fixture、文档命令、hook 模板、fixture policy、service 模板和 live verification pending 语义。
- [x] 已接入 CLI：`hermeship setup`、`hermeship install --home --force --dry-run`、`hermeship uninstall --home --hermes-home --remove-config --remove-state --remove-hooks --dry-run`、`hermeship release preflight <version>`。
- [x] 已完成本地 deterministic CLI smoke：`install --dry-run`、临时目录 `install`、`setup` 脱敏输出、`uninstall --dry-run`、`release preflight 0.1.0`。
- [x] 已根据代码审查修复安全边界：`setup` 不再接受明文 token argv，改用 stdin/env；`config show` 默认脱敏；写配置时使用私有权限；`install` 写入 home marker；destructive `uninstall` 必须验证 marker；`--remove-hooks` 默认使用 Hermes home；release preflight 纳入 `docs/operations.md`。
- [x] 已运行验证：`cargo test lifecycle`（10 passed）、`cargo test release_preflight`（6 passed）、`cargo test cli`（17 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（139 lib tests + 8 bin tests passed）均通过。
- [x] 已确认本阶段没有实现真实 live verification、Slack sink、Hermes plugin/observer、真实 systemd/launchd 安装或外部网络发布自动化。
- [x] 提交状态：已提交 `162efcd feat: 增加安装生命周期与发布预检`。

### 2026-06-16 - Milestone 6 完成后交接更新

- [x] 已确认当前分支为 `codex/milestone-1-cli`，工作树在更新前干净，最新提交为 `f6f98a3 feat: 支持 Hermes hook bridge 安装`。
- [x] 已更新 `docs/development-status.md`，明确 Milestone 0 到 Milestone 6 已完成并提交，Milestone 7 到 Milestone 10 未完成。
- [x] 已更新 `tasks/todo.md`，将最新功能阶段提交改为 `f6f98a3 feat: 支持 Hermes hook bridge 安装`，并明确下一入口为 Milestone 7 安装、生命周期与运维 CLI。
- [x] 本次只做文档交接更新，不进入 Milestone 7 实现，不修改功能代码，不执行 release preflight、真实 live verification、Slack sink 或 Hermes plugin/observer。

### 2026-06-16 - Milestone 6 Hermes Hook Bridge 安装

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认启动时工作树干净：`git status --short --branch` 只有分支行；最近提交为 `1ad3c7c`、`026e80c`、`cb4cca8`。
- [x] 已阅读 `src/cli.rs`、`src/main.rs`、`src/hermes.rs`、`src/client.rs`、`src/daemon.rs`、`src/config.rs`、`tests/fixtures/README.md` 和方案文档 Hermes Hook Bridge 章节。
- [x] 已先写失败测试并运行 Red：`cargo test hooks` 在实现前失败于缺少 hook 模板常量、安装/卸载 API、`--home`/`--dry-run` CLI 字段和 `uninstall-hooks` 子命令。
- [x] 已新增 `templates/hermes-hook/HOOK.yaml`，声明 `gateway:startup`、`session:start`、`session:end`、`session:reset`、`agent:start` 与 `agent:end`；`agent:step` 与 `command:*` 默认不安装，避免绕过默认关闭的 Hermes 配置开关。
- [x] 已新增 `templates/hermes-hook/handler.py`，只使用 Python 标准库，暴露 `handle(event_type, context)`，通过 stdin 调用 `hermeship hermes hook --payload -`，并对 binary missing、子进程失败和 timeout fail-open。
- [x] 已新增 `src/hooks.rs` installer，支持 fake Hermes home、首次安装、不覆盖、`--force` 覆盖、dry-run 只报告不写盘、返回计划/写入/跳过路径，并写入 `.hermeship-managed.json` 用于安全卸载。
- [x] 已接入 CLI：`hermeship hermes install-hooks --home <path> --force --dry-run` 与 `hermeship hermes uninstall-hooks --home <path> --dry-run`。
- [x] 已完成本地 deterministic CLI 验证：`cargo run -- hermes install-hooks --home /tmp/hermeship-test-home --force` 写入 hook 文件；`find /tmp/hermeship-test-home/hooks/hermeship -maxdepth 1 -type f -print` 显示 `HOOK.yaml` 与 `handler.py`；`cargo run -- hermes uninstall-hooks --home /tmp/hermeship-test-home` 删除 Hermeship hook 目录。
- [x] 已运行验证：`cargo test hooks`（19 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（120 lib tests + 6 bin tests passed）均通过。
- [x] 已确认本阶段没有实现 release preflight、真实 live verification、Slack sink 或 Hermes plugin/observer；默认测试只使用 fake Hermes home、fake hermeship binary、本地 handler 和本地 CLI。
- [x] 提交状态：已提交 `f6f98a3 feat: 支持 Hermes hook bridge 安装`。

### 2026-06-16 - Milestone 5.3 本地端到端 smoke

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认启动时工作树干净：`git status --short --branch` 只有分支行；最近提交为 `cb4cca8`、`ea9b789`、`debb28b`。
- [x] 已阅读 `src/daemon.rs`、`src/client.rs`、`src/router.rs`、`src/render/mod.rs`、`src/render/default.rs`、`src/dispatch.rs`、`src/sink/mod.rs`、`src/sink/fake.rs` 和 `tests/fixtures/README.md`，确认本阶段只做本地 deterministic smoke，不进入 hook bridge install、release preflight、真实 live verification 或 Slack sink。
- [x] 已先写失败测试并运行 Red：`cargo test daemon` 在实现前失败于缺少 `daemon_router_with_sink_registry`，正好暴露出生产 daemon router 无法注入 fake sink 的测试缺口。
- [x] 已在 `src/daemon.rs` 增加内部 sink registry 注入 helper，生产 `daemon_router()` 仍走真实 Discord sink registry，测试可注入 `FakeSink` 并复用同一条 queue consumer 路径。
- [x] 已新增 deterministic daemon + fake sink smoke：随机端口 test daemon 接收 `POST /api/hermes/hook`，内部 dispatcher 执行 `Router -> DefaultRenderer -> FakeSink`，fake sink 记录渲染后的 delivery。
- [x] 已通过本地路径验证 `send` 与 `emit`：`cargo test daemon` 覆盖 `daemon_send_command_posts_custom_event_to_daemon` 和 `daemon_emit_command_posts_event_to_daemon`，二者都只走本地 test daemon/client，不需要真实 Discord/Hermes。
- [x] 已确认默认隐私保护生效：smoke payload 中的完整 message、response、token、cookie、secret 不会出现在 fake sink 消息中。
- [x] 已运行验证：`cargo test dispatch`（12 passed）、`cargo test daemon`（19 lib-filtered tests + 4 bin-filtered tests passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（103 lib tests + 6 bin tests passed）均通过。
- [x] 已更新 `tasks/todo.md` Review，并将下一入口切到 Milestone 6 Hermes Hook Bridge 安装。
- [x] 提交状态：已提交 `026e80c test: 增加 daemon 到 sink 的端到端覆盖`。

### 2026-06-16 - Milestone 5.2 Sink 失败语义

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认启动时工作树干净：`git status --short --branch` 只有分支行；最近提交为 `debb28b`、`0cd6e4e`、`88fa7a3`。
- [x] 已阅读 `src/sink/discord.rs`、`src/sink/mod.rs`、`src/sink/fake.rs`、`src/dispatch.rs`、`src/daemon.rs`、`src/router.rs`、`src/config.rs` 与 `tests/fixtures/README.md`。
- [x] 已参考 `template/clawhip/src/discord.rs`，只借鉴非 2xx、429 retry-after、缺失 token/channel 与 best-effort 多投递语义；未引入 clawhip DLQ、circuit breaker 或 runtime 依赖。
- [x] 已先写失败测试并运行 Red：`cargo test sink -- --nocapture` 在实现前失败于 `sink_reports_non_2xx_status_and_body_tail` 与 `sink_reports_rate_limit_retry_after_from_429_body`。
- [x] 已实现 Discord 非 2xx 失败诊断：错误包含 HTTP status 和受控 body tail，且不输出 webhook URL 或 token。
- [x] 已实现 Discord 429 rate limit 语义：解析 JSON body 中的 `retry_after` 并输出明确 `retry_after=<seconds>s` 诊断；本阶段不做 sleep/retry 状态机。
- [x] 已覆盖 token 缺失经 dispatcher 报告 `SinkFailed`、空 channel/request builder 错误、fake HTTP 500、429 和多 delivery 一失败一成功继续投递。
- [x] 已处理代码审查建议：fake HTTP server await 增加超时保护，并新增 429 缺失 `retry_after` 时 `retry_after=unknown` 的 fallback 测试。
- [x] 已运行验证：`cargo test sink`（23 passed）、`cargo test dispatch`（11 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（102 lib tests + 6 bin tests passed）均通过。
- [x] 已确认本阶段没有实现 Hermes hook bridge install、install/uninstall lifecycle、release preflight、真实 live verification 或 Slack sink。
- [x] 提交状态：已提交 `ea9b789 feat: 完善 sink 失败处理`。

### 2026-06-16 - Milestone 5.2 入口交接更新

- [x] 已确认最新功能阶段提交为 `0cd6e4e feat: 增加 Discord sink`。
- [x] 已确认 Milestone 5.1 完成：Discord sink payload/request builder、bot channel、webhook、allowed mentions、长度截断、daemon sink registry 均已实现并验证。
- [x] 历史记录：当时已将 `docs/development-status.md` 更新为 Milestone 5.1 完成后的 Milestone 5.2 入口；当前入口已由上方 Milestone 5.2 完成记录取代。
- [x] 历史记录：当时已将 `tasks/todo.md` 切换为下一阶段任务：Milestone 5.2 `Sink 失败语义`；当前计划已切换为 Milestone 5.3。
- [x] 历史记录：当时入口为 Milestone 5.2；当前入口为 Milestone 5.3 本地端到端 smoke。
- [x] 历史记录：当时已确认任务 5.2 边界：不实现 Hermes hook bridge install、install/uninstall lifecycle、release preflight、真实 live verification 或 Slack sink。

### 2026-06-16 - Milestone 5.1 Discord Sink 与基础 Live Path

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认启动时工作树干净：`git status --short --branch` 只有分支行；最近提交为 `88fa7a3`、`a336e01`、`049fe75`。
- [x] 已阅读 `src/config.rs`、`src/router.rs`、`src/render/mod.rs`、`src/render/default.rs`、`src/dispatch.rs`、`src/sink/mod.rs`、`src/sink/fake.rs`、`src/daemon.rs` 与 `tests/fixtures/README.md`。
- [x] 已参考 `template/clawhip/src/sink/discord.rs` 与 `template/clawhip/src/discord.rs`，只借鉴 Discord bot/webhook HTTP 路径和 webhook `wait=true` 行为，不依赖 clawhip runtime。
- [x] 已先写失败测试并运行 Red：`cargo test discord` 在实现前失败于缺少 `DiscordSink`、`DiscordMessagePayload`、Discord 长度常量和 `SinkMessage.mention`。
- [x] 已新增 `src/sink/discord.rs` 并在 `src/sink/mod.rs` 导出 `discord` 模块，实现 Discord payload/request builder、allowed mentions、内容长度截断、bot channel 和 webhook 发送路径。
- [x] 已将 `SinkMessage` 增加 `mention` 字段，并在 dispatcher 中从 `ResolvedDelivery` 传递给 sink；Discord payload 只允许显式 route/event mention 产生真实 Discord mention，正文中的其他 mention 默认不 ping。
- [x] 已将 daemon dispatcher registry 接入真实 Discord sink，默认生产 daemon 不再因缺失 `"discord"` sink 而无法投递；token/channel/webhook 缺失时以 sink failure 诊断，不 panic。
- [x] 已处理代码审查反馈：webhook URL 会精确移除已有 `wait` query 参数并追加 `wait=true`，避免 `wait=false` 或 `await=true` 误判。
- [x] 已覆盖测试：webhook JSON payload、bot channel request payload、allowed mentions、消息长度截断、token/channel/webhook 缺失诊断、fake HTTP webhook 投递、dispatcher mention 传递和 daemon sink registry。
- [x] 已运行验证：`cargo test discord`（9 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（97 lib tests + 6 bin tests passed）均通过。
- [x] 已确认本阶段没有实现 Hermes hook bridge install、install/uninstall lifecycle、release preflight、真实 live verification 或 Slack sink；Discord 4xx/5xx/rate limit 失败矩阵留在 Milestone 5.2。
- [x] 提交状态：已提交 `0cd6e4e feat: 增加 Discord sink`。

### 2026-06-16 - Milestone 5.1 开发入口交接

- [x] 已确认最新功能阶段为 `a336e01 feat: 实现事件 dispatcher 与 fake sink`，Milestone 0 到 Milestone 4.3 均已完成并提交。
- [x] 历史记录：当时已将 `docs/development-status.md` 更新为 Milestone 4.3 已完成、Milestone 5.1 入口状态；当前入口已由上方 Milestone 5.2 记录取代。
- [x] 历史记录：当时已将 `tasks/todo.md` 更新为 Milestone 5.1 执行计划；当前计划已切换为 Milestone 5.2。
- [x] 历史记录：当时未完成范围包括 Discord sink、Hermes hook bridge install、install/uninstall lifecycle、release preflight、live verification、Slack sink 和 Hermes plugin/observer；其中 Discord sink 已在 Milestone 5.1 完成。
- [x] 历史记录：当时下次开发入口是 Milestone 5 的任务 5.1；当前入口已由上方 Milestone 5.2 完成记录取代。

### 2026-06-16 - Milestone 4.3 Dispatcher 与 fake sink

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认启动时工作树干净：`git status --short --branch` 只有分支行；最近提交为 `049fe75`、`d4303ae`、`b07d880`。
- [x] 已阅读 `src/cli.rs`、`src/main.rs`、`src/config.rs`、`src/daemon.rs`、`src/events.rs`、`src/event/mod.rs`、`src/event/body.rs`、`src/event/compat.rs`、`src/privacy.rs`、`src/router.rs`、`src/render/mod.rs`、`src/render/default.rs` 与 `tests/fixtures/README.md`，确认本阶段只实现 dispatcher 与 fake sink。
- [x] 已先写失败测试并运行 Red：`cargo test dispatch sink` 被 Cargo 语法拒绝；`cargo test dispatch` / `cargo test sink` 在实现前失败于缺少 `Dispatcher`、`Sink`、`SinkMessage` 和 `FakeSink`。
- [x] 已新增 `src/dispatch.rs`、`src/sink/mod.rs`、`src/sink/fake.rs`，并在 `src/lib.rs` 导出 `dispatch` 与 `sink`。
- [x] 已实现 `Dispatcher`、`DispatchReport`、`DeliveryOutcome` 和 `DeliveryStatus`，支持单事件与队列消费，执行 `Router::resolve -> Renderer::render -> Sink::send`。
- [x] 已实现 object-safe `Sink` trait、`SinkMessage`、`FakeSink` 和 `FakeDelivery`；fake sink 记录 target、format、rendered content、event kind 和 route index，并支持按 route index 注入确定性失败。
- [x] 已将默认 daemon queue 接入 dispatcher consumer，避免生产路径只入队不消费；真实 Discord 投递仍在 Milestone 5。
- [x] 已覆盖测试：多投递、单 sink failure 不阻断后续 delivery、无 route、render failure、missing sink、队列消费、daemon ingress -> dispatcher -> fake sink E2E 和隐私不泄漏。
- [x] 已运行验证：`cargo test dispatch`（8 passed）、`cargo test sink`（8 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（87 lib tests + 6 bin tests passed）均通过。
- [x] 已确认本阶段没有实现 Discord sink、Hermes hook bridge install、install/uninstall lifecycle 或 release preflight。
- [x] 提交状态：已提交 `a336e01 feat: 实现事件 dispatcher 与 fake sink`。

### 2026-06-16 - Milestone 4.2 Renderer

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认启动时工作树干净：`git status --short --branch` 只有分支行；最近提交为 `b07d880`、`864e7f4`、`7bc2bff`。
- [x] 已阅读 `src/cli.rs`、`src/main.rs`、`src/config.rs`、`src/events.rs`、`src/event/mod.rs`、`src/event/body.rs`、`src/event/compat.rs`、`src/privacy.rs`、`src/router.rs` 与 `tests/fixtures/README.md`，并参考 `template/clawhip/src/render/default.rs` 确认 renderer 边界。
- [x] 已先写失败测试并运行 Red：`cargo test render` 在实现前失败于缺少 `DefaultRenderer` 与 `RenderedMessage`；收到代码审查后又补充 raw+template、direct typed free-text 和未批准 template token 回归测试，并确认它们在修复前失败。
- [x] 已新增 `src/render/mod.rs` 和 `src/render/default.rs`，并在 `src/lib.rs` 导出 `hermeship::render`。
- [x] 已实现 `Renderer` trait、`DefaultRenderer` 与 `RenderedMessage`，输入为 `EventEnvelope` 和 `ResolvedDelivery`，输出 deterministic 可投递文本。
- [x] 已支持 `compact`、`inline`、`alert`、`raw` 四种格式，覆盖 Hermes gateway/session/agent/custom 事件、缺字段降级和 `hermes.agent.failed` 安全错误摘要。
- [x] 已实现安全 template token：`{event}`、`{canonical_kind}`、`{source}`、`{provider}`、`{platform}`、`{session_id}`、`{agent_name}`、`{project}`、`{channel}`；未批准 token 保持原样。
- [x] 已处理代码审查反馈：`raw` 永远输出 JSON，忽略 template/mention；raw 不直接序列化 typed 自由文本，只输出长度/存在性摘要并清洗 nested payload。
- [x] 已覆盖测试：所有格式、缺字段降级、raw JSON、template token、route-level `format`/`template`/`mention` 组合、raw+template、direct typed free-text raw 泄漏回归、未批准 token。
- [x] 已运行验证：`cargo test render`（10 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（74 lib tests + 6 bin tests passed）均通过。
- [x] 已确认本阶段没有实现 dispatcher、sink、hook bridge install、install/uninstall lifecycle 或 release preflight。
- [x] 剩余风险：daemon 队列仍只入队不消费；dispatcher 与 fake sink 在 Milestone 4.3 实现，Discord sink 和 live verification 仍在后续 milestone。
- [x] 提交状态：随本阶段提交 `d4303ae feat: 增加 Hermes 默认渲染器` 一并完成。

### 2026-06-16 - Milestone 4.1 Router

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认启动时工作树干净：`git status --short --branch` 只有分支行；最近提交为 `7bc2bff`、`7b10816`、`12713ed`。
- [x] 已阅读 `src/cli.rs`、`src/main.rs`、`src/config.rs`、`src/events.rs`、`src/event/mod.rs`、`src/event/body.rs`、`src/event/compat.rs`、`src/privacy.rs`、`src/hermes.rs` 与 `tests/fixtures/README.md`，确认本阶段只实现 Router 与 `explain`。
- [x] 已先写失败测试并运行 Red：`cargo test router` 在实现前失败于缺少 `Router`、`SinkTarget`、`glob_match` 和真实 explain helper。
- [x] 已新增 `src/router.rs` 并导出 `hermeship::router`，实现 `Router`、`ResolvedDelivery`、`SinkTarget`、`DeliveryExplanation`、event glob、route candidates、结构化 metadata filter、disabled route 与 missing target 诊断。
- [x] 已将 `hermeship explain` 从 placeholder 替换为本地诊断路径：加载配置、清洗 payload、转 typed `EventEnvelope`、调用 router explain 并打印 route candidates、matched/skipped routes、failed filters 与 delivery target；不调用 daemon、不入队、不渲染、不投递。
- [x] 已覆盖测试：多 route 多投递、filter 命中/未命中、disabled route、missing target、无 route、event hint/default channel fallback、route-level channel/format/template/mention 继承、explain 输出契约和 webhook 诊断脱敏。
- [x] 已处理代码审查反馈：`explain` human/serialized diagnostics 不再输出完整 Discord webhook URL，内部 `SinkTarget::DiscordWebhook(String)` 仍保留原值供后续 dispatcher 使用。
- [x] 已确认本阶段没有实现 renderer、dispatcher、sink、hook bridge install、install/uninstall lifecycle 或 release preflight。
- [x] 已运行验证：`cargo test router`（6 lib tests + 1 bin test passed）、`cargo run -- explain hermes.agent.started --payload '{"platform":"telegram","session_id":"demo"}'` 返回 no routes/no deliveries 诊断、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（67 lib tests + 6 bin tests passed）均通过。
- [x] 剩余风险：Router 目前只产出 Discord channel/webhook delivery plan，不实际渲染或投递；dispatcher/consumer 仍在 Milestone 4.3 实现。
- [x] 提交状态：随本阶段提交 `864e7f4 feat: 实现多投递路由` 一并完成。

### 2026-06-16 - Milestone 3.3 Hermes hook ingress

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认启动时工作树干净：`git status --short --branch` 只有分支行；最近提交为 `12713ed`、`0b63e49`、`dbe6597`。
- [x] 已阅读 `src/cli.rs`、`src/main.rs`、`src/config.rs`、`src/client.rs`、`src/daemon.rs`、`src/events.rs`、`src/event/mod.rs`、`src/event/body.rs`、`src/event/compat.rs`、`src/privacy.rs` 与 `tests/fixtures/README.md`，确认本阶段只实现 Hermes hook ingress。
- [x] 已先写失败测试并运行 Red：`cargo test hermes` 在实现前失败于缺少 `HermesHookEnvelope`、`DaemonClient::hermes_hook_url()` 与 `DaemonClient::post_hermes_hook()`。
- [x] 已新增 `src/hermes.rs`，实现 `HermesHookEnvelope` 与 normalization：接收 `provider`、`source`、`event`/`event_type`、`context`，默认 provider/source 为 `hermes`/`gateway`，输出标准 `IncomingEvent`。
- [x] 已实现 daemon `POST /api/hermes/hook`，并抽出共享入队函数复用 `/event` 的 privacy sanitizer、typed conversion 和 bounded queue `try_send` 管道。
- [x] 已实现 `DaemonClient::hermes_hook_url()` 与 `post_hermes_hook()`，错误信息包含 `/api/hermes/hook`。
- [x] 已将 `hermeship hermes hook --payload` 从 placeholder 替换为 daemon client POST 路径，支持 inline JSON 与 `--payload -` stdin，输出 queued 摘要；`hermes install-hooks`、install、release、explain 仍保持后续 milestone placeholder。
- [x] 已覆盖测试：hook envelope 默认值与 `event_type` alias、gateway/session/agent mapping、`agent:end` failure mapping、daemon hook 入队、入队前隐私清洗、缺失 event 4xx、daemon unavailable、CLI stdin 读取和 hook client 投递。
- [x] 已确认本阶段没有实现 router、renderer、dispatcher、sink、hook bridge install、install/uninstall lifecycle 或 release preflight。
- [x] 已运行验证：`cargo test hermes`（14 lib tests + 3 bin tests passed）、临时 daemon 下 `printf '%s' '{"event":"agent:start","context":{"session_id":"demo"}}' | cargo run -- hermes hook --payload -` 返回 queued 摘要、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（61 lib tests + 5 bin tests passed）均通过。
- [x] 剩余风险：daemon 队列仍只入队不消费；队列满时 hook ingress 会与 `/event` 一样返回 503，dispatcher/consumer 会在 Milestone 4.3 实现。
- [x] 提交状态：随本阶段提交 `7b10816 feat: 增加 Hermes hook ingress` 一并完成。

### 2026-06-16 - Milestone 3.2 Event ingress 与队列

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认启动时工作树干净：`git status --short --branch` 只有分支行；最近提交为 `dbe6597`、`ff5c589`、`2e74184`。
- [x] 已先写失败测试并运行 Red：`cargo test daemon` 在实现前失败于缺少 `daemon_router_with_queue` 与 `DaemonClient::post_event`；后续针对 `send --message` 消息被 sanitizer 吃掉的问题补充失败测试并修复。
- [x] 已实现 daemon 通用 `POST /event`：接收 `IncomingEvent`，入队前调用 `privacy::sanitize_payload()`，再通过 `event::compat::from_incoming_event()` 转为 `EventEnvelope`，最后写入 bounded `tokio::mpsc` 队列。
- [x] 已新增 typed `EventAcceptedResponse`，返回 event id、canonical kind、queued 状态和 queue health；`/health` 现在报告真实 queue pending/capacity/status。
- [x] 已实现 `DaemonClient::post_event()`、`event_url()`、daemon unavailable 和非 2xx 清晰错误。
- [x] 已将 `hermeship emit` 与 `hermeship send` 从 placeholder 替换为 client POST `/event`，并打印 queued 摘要；`explain`、`hermes hook`、install、release 仍保持后续 milestone placeholder。
- [x] 已调整 `IncomingEvent::custom()` 使用安全 `summary` 字段承载显式 `send --message` 文本，避免与 Hermes 对话正文 `message` 字段共用隐私语义。
- [x] 已覆盖 ingress 测试：有效 fixture 入队、隐私清洗后入队、非法 JSON 4xx、缺失 kind 4xx、daemon unavailable、queue full 503、health pending、`send`/`emit` client 投递。
- [x] 已确认本阶段没有实现 Hermes hook ingress、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
- [x] 已运行验证：`cargo test daemon`（11 passed + bin 2 passed）、`cargo test event`（21 passed + bin 2 passed）、临时 daemon 下 `cargo run -- emit hermes.agent.started --payload '{"session_id":"demo"}'` 返回 queued 摘要、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（52 passed + bin 2 passed）均通过。
- [x] 剩余风险：本阶段 daemon 只负责入队，不启动 consumer；队列满时 `/event` 返回 503，dispatcher/consumer 会在 Milestone 4.3 实现。
- [x] 提交状态：随本阶段提交 `0b63e49 feat: 增加 daemon event ingress` 一并完成。

### 2026-06-16 - Milestone 3.2 入口交接更新

- [x] 已确认当前最新功能阶段提交为 `ff5c589 feat: 增加 hermeship daemon health`。
- [x] 已确认 Milestone 3.1 完成：daemon `/health`、typed health response、daemon client、`hermeship start`、`hermeship status` 均已实现并验证。
- [x] 已将 `tasks/todo.md` 切换为下一阶段任务：Milestone 3.2 `Event ingress 与队列`。
- [x] 当前下一步为 Milestone 3.2：实现通用 `/event` ingress、`IncomingEvent -> EventEnvelope` conversion、隐私清洗和队列入队。
- [x] 已确认任务 3.2 边界：不实现 Hermes hook ingress、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
- [x] 任务 3.2 验证命令使用分开的 Cargo filter：`cargo test daemon`、`cargo test event`，不要把两个 filter 合并为一个 `cargo test` 命令。

### 2026-06-15 - Milestone 3.1 Daemon health 与 client

- [x] 已复习 `tasks/lessons.md`、`docs/development-status.md`、方案文档、`tasks/development-checklist.md` 与 `tasks/todo.md`，并确认当前分支为 `codex/milestone-1-cli`。
- [x] 已确认最新提交为 `2e74184 docs: 更新 Hermeship Milestone 3.1 交接状态`，上一功能阶段提交为 `175009d feat: 增加 Hermes 事件隐私清洗`。
- [x] 已阅读 `src/cli.rs`、`src/main.rs`、`src/config.rs`、`src/lib.rs`、`src/events.rs`、`src/event/mod.rs`、`src/event/body.rs`、`src/event/compat.rs`、`src/privacy.rs` 与 `tests/fixtures/README.md`，确认本阶段只接入 daemon health/status。
- [x] 已先写失败测试并确认 `cargo test daemon` 在实现前失败于缺少 `DaemonClient`、`HealthResponse`、`bind_listener` 与 `serve_listener`。
- [x] 已新增 `src/daemon.rs` 与 `src/client.rs`，并在 `src/lib.rs` 导出 daemon/client 模块。
- [x] 已实现 `/health`、typed `HealthResponse`、`QueueHealth`、daemon listener 绑定、`hermeship start` daemon 启动路径和 `hermeship status` client 查询路径。
- [x] 已实现 client 基础 URL 规范化、2 秒健康检查超时和非 2xx 错误摘要。
- [x] 已确认本阶段没有实现 event ingress、队列入队、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
- [x] 已运行验证：`cargo test daemon`（4 passed）、`cargo run -- status`（daemon 未运行时返回清晰错误且无 panic）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（45 passed）均通过。
- [x] 提交状态：随本阶段提交 `feat: 增加 hermeship daemon health` 一并完成。

### 2026-06-15 - Milestone 3.1 入口交接更新

- [x] 已将 `docs/development-status.md`、`README.md` 和 `tasks/todo.md` 更新到 Milestone 3.1 入口。
- [x] 已确认当前最新功能阶段提交为 `175009d feat: 增加 Hermes 事件隐私清洗`。
- [x] 当前下一步为 Milestone 3.1：Daemon health 与 client。
- [x] 已确认任务 3.1 边界：只实现 daemon health/status 与 client health 查询，不实现 event ingress、队列、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。

### 2026-06-15 - Milestone 2.3 隐私与 payload 清洗

- [x] 已在 `codex/milestone-1-cli` 分支执行本阶段；启动时工作树无未提交代码变更，最新提交为 `dae54b5 docs: 更新 Hermeship Milestone 2.3 交接状态`。
- [x] 已确认本阶段只实现 privacy 清洗纯逻辑、测试和合成 fixture；未实现 daemon、client、HTTP ingress、队列、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
- [x] 已先写失败测试并运行 Red：`cargo test privacy` 在实现前失败于缺少 `redact_value`、`sanitize_payload` 和 `excerpt_policy`。
- [x] 已新增 `src/privacy.rs`，并在 `src/lib.rs` 导出 `hermeship::privacy`。
- [x] 已实现 `sanitize_payload`、`redact_value`、`excerpt_policy`；默认递归脱敏 `token`、`api_key`、`authorization`、`password`、`secret`、`cookie`，并支持大小写不敏感、camelCase 和常见缩写 key 匹配。
- [x] 已默认删除完整正文类字段：`message`、`response`、`conversation_history`、`request`、`provider_response`、`tool_result`；同时清洗 `messages`、`prompt`、`user_message`、`assistant_response`、`provider_request`、`provider_request_body`、`provider_response_body`、`tool_results`、`tool_result_body` 等同类高风险别名。
- [x] 已保留安全摘要：`message_chars`、`response_chars`、`has_message`、`has_response`；短正文默认不原样泄漏，非法摘要字段类型会被丢弃，computed summary 不会被原始 payload 覆盖。
- [x] 已实现 opt-in 摘录：`include_message_excerpt`、`include_response_excerpt` 和 `max_excerpt_chars`；摘录先经过完整 sanitizer，再按 char 边界截断。
- [x] 已新增合成 fixture：`tests/fixtures/privacy/sensitive_payload.json`；fixture 不包含真实 token、cookie、secret、完整 prompt、完整对话或 provider request/response body。
- [x] 已根据 code/security review 修复摘要字段泄漏、`Authorization: Bearer ...` / `api_key = ...` inline secret 泄漏、URL query secret 泄漏、camelCase/acronym alias 绕过、结构化摘录泄漏和 fixture body hygiene 问题。
- [x] 已运行验证：`cargo test privacy`（10 passed）、`cargo test event`（14 passed）、`cargo test events`（6 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（41 passed）均通过。
- [x] 提交状态：随本阶段提交 `feat: 增加 Hermes 事件隐私清洗` 一并完成。

### 2026-06-15 - Milestone 2.3 入口交接更新

- [x] 已将 `docs/development-status.md`、`README.md` 和 `tasks/todo.md` 更新到 Milestone 2.3 入口。
- [x] 已确认当前最新功能阶段提交为 `b799415 feat: 实现 Hermes typed event model`。
- [x] 下次开发从 Milestone 2.3 隐私与 payload 清洗继续，不再沿用 Milestone 2.2 入口。

### 2026-06-15 - Milestone 2.2 Typed EventEnvelope

- [x] 已在 `codex/milestone-1-cli` 分支执行本阶段；启动时工作树无未提交代码变更，最新提交为 `fdd175d docs: 更新 Hermeship Milestone 2.2 交接状态`。
- [x] 已确认本阶段只实现 typed `EventEnvelope`、Hermes event body、canonical mapping 和 `IncomingEvent -> EventEnvelope` conversion；未实现 daemon、client、HTTP ingress、队列、privacy 清洗、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
- [x] 已参考 `template/clawhip/src/event/mod.rs`、`body.rs`、`compat.rs` 的 envelope/body/compat 分层，只移植结构和转换模式，不依赖 clawhip runtime。
- [x] 已先写失败测试并运行 Red：`cargo test event` 在实现前失败于 `from_incoming_event()` 未实现。
- [x] 已新增 `src/event/mod.rs`、`src/event/body.rs`、`src/event/compat.rs`，并在 `src/lib.rs` 导出 `hermeship::event`。
- [x] 已定义 `EventEnvelope`、`EventBody`、`EventMetadata`、`EventPriority`；`EventMetadata` 保留 route hint 和 Hermes provider/source/platform/chat/session/agent/project 等 metadata。
- [x] 已实现 Hermes canonical mapping：`gateway:startup`、`session:start`、`session:end`、`session:reset`、`agent:start`、`agent:step`、`agent:end`；显式失败的 `agent:end` 转为 `hermes.agent.failed`；未知 event 降级为 `Custom`。
- [x] 已复用 Hermes 合成 fixture：`tests/fixtures/hermes/agent_start.json` 和 `session_end.json` 覆盖 typed conversion；fixture 不包含真实 token、cookie、secret、完整 prompt、完整对话或 provider request/response body。
- [x] 已运行验证：`cargo test event`、`cargo test events`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 均通过。
- [x] 提交状态：随本阶段提交 `feat: 实现 Hermes typed event model` 一并完成。

### 2026-06-15 - Milestone 2.1 IncomingEvent 与格式

- [x] 已在 `codex/milestone-1-cli` 分支执行本阶段；启动时工作树无未提交代码变更，最新提交为 `b026e9c docs: 更新 Hermeship Milestone 2.1 交接状态`。
- [x] 已确认本阶段只实现 `IncomingEvent`、`RoutingMetadata`、`MessageFormat` 复用/重导出、`emit`/`explain` 参数事件构造和 Hermes fixture；未实现 daemon、typed `EventEnvelope`、privacy 清洗、router、renderer、sink、hook bridge、install 或 release preflight。
- [x] 已采用单一 `MessageFormat` 策略：继续在 `src/config.rs` 定义 enum，新增 `from_label()`；`src/events.rs` 通过 `pub use crate::config::MessageFormat` 提供事件层导出，避免两套不一致格式 enum。
- [x] 已先写失败测试并运行 Red：`cargo test events` / `cargo test cli` 在实现前失败于缺少 `events::MessageFormat`、`IncomingEvent`、`RoutingMetadata` 和 `EventArgs::into_event`。
- [x] 已新增 `src/events.rs`，实现 `IncomingEvent`、`RoutingMetadata`、字段别名反序列化、空/null payload 归一为空对象、缺省 payload 时 top-level extra 字段进入 payload。
- [x] 已将 `hermeship emit` / `hermeship explain` 参数解析接入 `EventArgs::into_event()`，支持 `--payload`、`--channel`、`--mention`、`--format`、`--template` 和任意 `--key value`；支持 `--agent`、`--session`、`--elapsed`、`--error` 别名；非法 format、奇数 key/value 和缺少 `--` 前缀会返回错误。
- [x] 已新增 Hermes fixture：`tests/fixtures/hermes/agent_start.json`、`tests/fixtures/hermes/session_end.json`、`tests/fixtures/hermes/invalid_payload.json`，内容为合成脱敏样例，不包含真实 token、cookie、secret、完整 prompt、完整对话或 provider request/response body。
- [x] 已运行验证：`cargo test events`、`cargo test cli`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 均通过。
- [x] 提交状态：随本阶段提交 `feat: 增加 IncomingEvent 事件入口` 一并完成。

### 2026-06-15 - Milestone 1.3 完成后状态交接更新

- [x] 已将 `docs/development-status.md` 更新到最新已完成阶段提交：`70c8f03 chore: 增加 Rust 质量门禁与仓库基础`。
- [x] 已明确 Milestone 0、1.1、1.2、1.3 已完成，Milestone 2.1 及后续仍未完成。
- [x] 已将 `tasks/todo.md` 切换为下一阶段任务：Milestone 2.1 `IncomingEvent` 与格式。
- [x] 已更新 `README.md` 的 Development Status，移除 Milestone 1.3 仍是下一步的过期描述。
- [x] 下一步执行 Milestone 2.1：先写事件入口和 CLI emit 解析测试，再实现 `src/events.rs` 与 Hermes fixture。

### 2026-06-15 - Milestone 1.3 质量门禁与仓库基础

- [x] 已在 `codex/milestone-1-cli` 分支执行本阶段，启动时工作树干净；最新提交为 `267efba docs: 更新 Hermeship 最新开发状态`，最新功能阶段提交为 `50723af feat: 实现 hermeship 配置模型与 config CLI`。
- [x] 已扩展 `.gitignore`：保留 `/target/`，新增本地编辑器临时文件、日志、临时目录、测试输出和覆盖率输出规则；未忽略源码、文档、fixture 或 `Cargo.lock`。
- [x] 已在 `README.md` 新增 Development Quality Gates，明确阶段提交前运行 `cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- [x] 已新增 fixture 目录：`tests/fixtures/hermes/`、`tests/fixtures/privacy/`、`tests/fixtures/routes/`、`tests/fixtures/discord/`，并保留 `tests/fixtures/cli/`。
- [x] 已新增 `tests/fixtures/README.md`，明确 fixture 只能使用合成脱敏样例，不得包含真实 Discord token、真实 Hermes gateway 数据、真实 GitHub/tmux 状态、cookie、secret、完整 prompt、完整对话或 provider request/response body。
- [x] 首次运行 `cargo clippy --all-targets -- -D warnings` 发现既有 lint：`AppConfig`/`MessageFormat` 的手写 `Default` 可 derive，CLI fixture 测试 helper 存在多余 `.into_iter()`；已用最小代码改动修复。
- [x] 已运行验证：`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 均通过。
- [x] 提交状态：随本阶段提交 `chore: 增加 Rust 质量门禁与仓库基础` 一并完成。

### 2026-06-15 - 最新开发状态交接更新

- [x] 已将 `docs/development-status.md` 更新到最新阶段提交：`50723af feat: 实现 hermeship 配置模型与 config CLI`。
- [x] 已明确 Milestone 0、1.1、1.2 已完成，Milestone 1.3 及后续仍未完成。
- [x] 已将 `tasks/todo.md` 切换为下一阶段任务：Milestone 1.3 质量门禁与仓库基础。
- [x] 已更新 `README.md` 的 Development Status，移除 Rust 尚未开始的过期描述。
- [x] 下一步执行 Milestone 1.3：更新 `.gitignore`、rustfmt/clippy 约束说明和测试 fixture 目录，并运行基础质量门禁。

### 2026-06-15 - Milestone 1.2 配置模型

- [x] 已复习 `tasks/lessons.md`，确认阶段完成后必须验证并提交，且不混入无关改动。
- [x] 已参考 `/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/config.rs` 和 Hermeship 当前 CLI 结构，只借鉴配置 schema、路径/env override、TOML 加载、归一化和 `config` 子命令接入方式。
- [x] 已先写配置测试并运行 Red：`cargo test config` 在实现前失败于缺少 `AppConfig`、`RouteRule`、`MessageFormat` 和 `config_path_from_env`。
- [x] 已新增 `src/config.rs`，包含 `AppConfig`、`DaemonConfig`、`ProvidersConfig`、`DiscordConfig`、`DefaultsConfig`、`PrivacyConfig`、`HermesConfig`、`RouteRule` 和 `MessageFormat`。
- [x] 已实现默认配置路径：`HERMESHIP_CONFIG` 优先，否则 `$HOME/.hermeship/config.toml`。
- [x] 已实现缺失配置返回默认值、非法 TOML 返回错误、未知 key 前向兼容忽略、空 token/channel/webhook/mention/template/filter 归一化、空 route sink 回退为 `discord`。
- [x] 已实现配置环境变量覆盖：`HERMESHIP_DAEMON_URL`、`HERMESHIP_DISCORD_TOKEN`、`HERMESHIP_DEFAULT_CHANNEL`、`HERMESHIP_DRY_RUN`。
- [x] 已将 `hermeship config path`、`hermeship config show`、`hermeship config verify` 接入真实配置逻辑，不再使用 Milestone 1.1 占位输出。
- [x] 已运行验证：`cargo fmt --all -- --check`、`cargo test config`、`cargo run -- config show` 均通过。
- [x] 提交状态：随本阶段提交 `50723af feat: 实现 hermeship 配置模型与 config CLI` 一并完成。

### 2026-06-15 - 开发状态交接更新

- [x] 已更新 `docs/development-status.md`，明确 Milestone 0 和 Milestone 1.1 已完成，Milestone 1.2 及后续仍未完成。
- [x] 已将 `tasks/todo.md` 切换为下一阶段任务：Milestone 1.2 配置模型。
- [x] 下一步执行 Milestone 1.2：先写配置模型测试，再实现 `src/config.rs`，并将 `config path/show/verify` 接入真实配置逻辑。

### 2026-06-15 - Milestone 1.1 Cargo 项目与 CLI 入口

- [x] 已在 `codex/milestone-1-cli` 分支执行本阶段，启动时 `main...origin/main [ahead 5]` 且无未提交变更。
- [x] 已参考 `/Users/zq/Desktop/ai-projs/posp/template/clawhip/Cargo.toml`、`src/main.rs`、`src/cli.rs`，只借鉴 Rust 2024 metadata、`clap` 命令树和 CLI dispatch 形态。
- [x] 已创建 `Cargo.toml`、`Cargo.lock`、`src/lib.rs`、`src/main.rs`、`src/cli.rs` 和 `tests/fixtures/cli/public_commands.txt`。
- [x] 已先写 CLI parse 测试并运行 Red：`cargo test cli` 在实现前失败于缺少 `Cli`、`Commands` 等 CLI 类型。
- [x] 已实现最小 CLI：`start`、`status`、`send`、`emit`、`explain`、`config`、`hermes`、`install`、`uninstall`、`release`。
- [x] 已处理代码审查反馈：新增最小 `.gitignore` 忽略 `/target/`；公开命令 fixture 改为 shell 风格引号示例，并在测试中断言必备命令前缀存在。
- [x] 已运行验证：`cargo fmt --all -- --check`、`cargo test cli`、`cargo run -- --help` 均通过。
- [x] 提交状态：随本阶段提交 `chore: 搭建 hermeship Rust CLI 骨架` 一并完成。

### 2026-06-15 - Milestone 1 续接状态

- [x] Milestone 0 已完成并提交：`af57c49 docs: 明确 hermeship 完整项目方向`。
- [x] `README.md` 已更新为正式项目定位，明确 Hermeship 是 Hermes-native daemon-first event router，不是 clawhip runtime client。
- [x] `tasks/todo.md` 已切换为下一阶段任务：Milestone 1.1 Cargo 项目与 CLI 入口。
- [x] `docs/development-status.md` 已更新为 Milestone 0 完成后的状态入口和下次启动提示词。
- [x] 下一步执行 Milestone 1.1：创建 Rust 2024 工程骨架、CLI parse 测试、公开命令 fixture 和最小 `hermeship --help`。

### 2026-06-15 - Milestone 0

- [x] 已复习 `tasks/lessons.md`，确认 Hermeship 目标是 Hermes-native daemon-first event router。
- [x] 已确认当前分支状态：`main...origin/main [ahead 3]`，启动时无未提交变更。
- [x] 已复核 `template/clawhip` 指定参考文件，确认可移植架构为 Rust CLI、daemon、typed event、router、renderer、sink、source、install 和 verification 表面。
- [x] 已复核 Hermes gateway hook 与 plugin 参考源码，确认 MVP 先接 gateway hooks，plugin/observer 在后续阶段进入。
- [x] 已更新 `README.md`，明确项目定位、实现边界、Hermes 接入、隐私默认值、计划 CLI 和验证策略。
- [x] 已运行 Milestone 0 验证命令，旧 Python/thin-adapter 方向没有回流。
- [x] 已提交 Milestone 0：`af57c49 docs: 明确 hermeship 完整项目方向`。

### 2026-06-15

- [x] 用户纠正：Hermeship 应完全参考 clawhip 项目形态，而不是 thin adapter。
- [x] 更新 lessons，记录正确目标。
- [x] 重写方案文档为 Hermes-native daemon-first event router。
- [x] 重写开发清单为 Rust/clawhip-parity 实现路径。
- [x] 集成测试计划，明确单元/契约/集成/E2E/live verification 分层。
- [x] 扩展开发清单测试门禁，覆盖 fake sink、fake HTTP、fake Hermes home、fake hermeship binary、隐私不变量和 CI/live 分离。
- [x] 新增 `docs/development-status.md`，作为下次启动的状态入口。
- [x] 当前已完成提交：`d69dbb4 docs: 重写 hermeship 完整项目方案`。
- [x] 当前已完成提交：`9771968 docs: 集成 hermeship 测试计划`。
- [x] `README.md` 已更新为正式项目定位。
- [x] Milestone 0 已正式执行并提交。
- [ ] 实现尚未开始。

## 阻塞项

- [ ] 确认 live Discord verification 凭据是否可用。
- [ ] 确认是否需要第一版就支持 Slack sink。
- [ ] 确认 git/GitHub/tmux parity 是否必须进入 `0.1.0`，还是 `0.2.0`。
- [ ] 确认 macOS launchd 是否必须和 systemd 同期实现。

## 决策记录

- [x] Hermeship 不是 thin adapter。
- [x] Hermeship 不依赖运行中的 clawhip。
- [x] Hermeship 以 `template/clawhip` 为架构和功能参考。
- [x] 主实现采用 Rust daemon-first 架构。
- [x] Python 只用于 Hermes gateway hook bridge 模板。
- [x] MVP 不修改 Hermes 核心。
- [x] MVP 首选 Hermes gateway hooks，observer plugin 后续推进。
- [x] 默认不转发完整 message/response/request/response/tool body。
- [x] 方案文档和进度清单分离维护。
