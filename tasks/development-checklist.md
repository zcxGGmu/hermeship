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

- [ ] 新建配置模块。
  - 新建：`src/config.rs`
  - 包含：`AppConfig`、`DaemonConfig`、`ProvidersConfig`、`DiscordConfig`、`DefaultsConfig`、`PrivacyConfig`、`HermesConfig`、`RouteRule`。
- [ ] 实现默认配置路径。
  - 默认：`~/.hermeship/config.toml`
  - 环境变量：`HERMESHIP_CONFIG`
- [ ] 实现默认配置与 TOML 加载。
  - 要求：缺失配置返回默认值；非法 TOML 返回错误。
- [ ] 实现 config CLI。
  - `hermeship config path`
  - `hermeship config show`
  - `hermeship config verify`
- [ ] 编写配置测试。
  - 覆盖：默认值、env override、非法 TOML、未知 key、空 channel/token 归一化。
- [ ] 验证任务 1.2。
  - 命令：`cargo test config`
  - 命令：`cargo run -- config show`
- [ ] 提交任务 1.2。
  - commit：`feat: 实现 hermeship 配置模型`

### 任务 1.3：质量门禁与仓库基础

- [ ] 增加 `.gitignore`。
  - 包含：`target/`、临时日志、测试输出。
- [ ] 增加 rustfmt/clippy 约束说明。
  - 文件：`README.md` 或 `docs/development.md`
- [ ] 增加测试 fixture 目录。
  - 新建：`tests/fixtures/hermes/`
  - 新建：`tests/fixtures/privacy/`
  - 新建：`tests/fixtures/routes/`
  - 新建：`tests/fixtures/discord/`
  - 新建：`tests/fixtures/cli/`
  - 完成标准：fixture 不包含真实 token、cookie、prompt、完整对话或 provider request/response body。
- [ ] 确认基础门禁通过。
  - 命令：`cargo fmt --all -- --check`
  - 命令：`cargo clippy --all-targets -- -D warnings`
  - 命令：`cargo test`
- [ ] 提交任务 1.3。
  - commit：`chore: 增加 Rust 质量门禁`

## Milestone 2：事件模型与兼容层

目标：实现 clawhip 风格的 `IncomingEvent -> typed EventEnvelope` 管道。

### 任务 2.1：IncomingEvent 与格式

- [ ] 新建事件入口模型。
  - 新建：`src/events.rs`
  - 类型：`IncomingEvent`、`MessageFormat`、`RoutingMetadata`。
- [ ] 实现 `MessageFormat`。
  - 支持：`compact`、`inline`、`alert`、`raw`。
- [ ] 实现 `emit` 参数解析。
  - 支持：`--channel`、`--mention`、`--format`、`--template`、`--payload`、任意 `--key value`。
- [ ] 编写测试。
  - 覆盖：payload JSON 合并、非法 format、奇数 key/value 拒绝、字段别名。
- [ ] 增加事件 fixture。
  - 新建：`tests/fixtures/hermes/agent_start.json`
  - 新建：`tests/fixtures/hermes/session_end.json`
  - 新建：`tests/fixtures/hermes/invalid_payload.json`
  - 完成标准：fixture 能驱动 CLI emit、daemon ingress 和 hook normalization 测试。
- [ ] 验证任务 2.1。
  - 命令：`cargo test events`
  - 命令：`cargo test cli`
- [ ] 提交任务 2.1。
  - commit：`feat: 增加 IncomingEvent 事件入口`

### 任务 2.2：Typed EventEnvelope

- [ ] 新建 typed event 模块。
  - 新建：`src/event/mod.rs`
  - 新建：`src/event/body.rs`
  - 新建：`src/event/compat.rs`
- [ ] 定义 `EventEnvelope`、`EventBody`、`EventMetadata`、`EventPriority`。
- [ ] 实现 Hermes event body。
  - `HermesGatewayStarted`
  - `HermesSessionStarted`
  - `HermesSessionFinished`
  - `HermesSessionReset`
  - `HermesAgentStarted`
  - `HermesAgentStep`
  - `HermesAgentFinished`
  - `HermesAgentFailed`
  - `Custom`
- [ ] 实现 canonical kind。
  - `gateway:startup` -> `hermes.gateway.started`
  - `session:start` -> `hermes.session.started`
  - `session:end` -> `hermes.session.finished`
  - `session:reset` -> `hermes.session.reset`
  - `agent:start` -> `hermes.agent.started`
  - `agent:step` -> `hermes.agent.step`
  - `agent:end` -> `hermes.agent.finished`
- [ ] 编写 compat 测试。
  - 覆盖：所有 Hermes gateway hook event、未知 event -> custom、缺失 session_id 的降级。
- [ ] 验证任务 2.2。
  - 命令：`cargo test event`
- [ ] 提交任务 2.2。
  - commit：`feat: 实现 Hermes typed event model`

### 任务 2.3：隐私与 payload 清洗

- [ ] 新建隐私模块。
  - 新建：`src/privacy.rs`
  - 函数：`sanitize_payload`、`redact_value`、`excerpt_policy`。
- [ ] 实现敏感 key 递归脱敏。
  - key：`token`、`api_key`、`authorization`、`password`、`secret`、`cookie`。
- [ ] 实现正文默认禁发。
  - 默认删除 `message`、`response`、`conversation_history`、`request`、`provider_response`、`tool_result`。
  - 默认保留 `message_chars`、`response_chars`、`has_message`、`has_response`。
- [ ] 实现 opt-in 摘录。
  - 配置：`privacy.include_message_excerpt`、`privacy.include_response_excerpt`。
  - 要求：先脱敏，再截断。
- [ ] 编写隐私测试。
  - 覆盖：短文本不原样泄漏、嵌套 secret、list、非字符串、原始 payload 不被原地修改。
- [ ] 增加隐私回归 fixture。
  - 新建：`tests/fixtures/privacy/sensitive_payload.json`
  - 完成标准：测试断言输出不包含原始 `message`、`response`、`conversation_history`、`request`、`provider_response`、`tool_result`、token、cookie、secret。
- [ ] 验证任务 2.3。
  - 命令：`cargo test privacy`
- [ ] 提交任务 2.3。
  - commit：`feat: 增加 Hermes 事件隐私清洗`

## Milestone 3：Daemon、队列与 HTTP ingress

目标：建立本地 daemon-first runtime。

### 任务 3.1：Daemon health 与 client

- [ ] 新建 daemon/client 模块。
  - 新建：`src/daemon.rs`
  - 新建：`src/client.rs`
- [ ] 实现 `hermeship start`。
  - 默认监听：`127.0.0.1:25295`。
- [ ] 实现 `/health`。
  - 返回：version、status、queue 状态、configured sinks。
- [ ] 实现 `hermeship status`。
  - 调用 daemon `/health`。
- [ ] 编写 daemon health 测试。
  - 使用随机端口或 test server。
  - 覆盖：健康响应 schema、队列状态、configured sinks、daemon 未运行时 client 错误。
- [ ] 验证任务 3.1。
  - 命令：`cargo test daemon`
  - 命令：`cargo run -- status`
  - 预期：daemon 未运行时返回清晰错误；不 panic。
- [ ] 提交任务 3.1。
  - commit：`feat: 增加 hermeship daemon health`

### 任务 3.2：Event ingress 与队列

- [ ] 实现 `/event`。
  - 接收 `IncomingEvent`。
  - 规范化并转为 `EventEnvelope`。
  - 写入 `tokio::mpsc` 队列。
- [ ] 实现 `hermeship emit`。
  - 通过 client POST `/event`。
- [ ] 实现 `hermeship send`。
  - 作为 custom event 发送。
- [ ] 编写 ingress 测试。
  - 覆盖：有效事件入队、非法 payload 4xx、daemon unavailable 错误。
  - 要求：使用随机端口和本地 test queue，不绑定固定端口。
- [ ] 验证任务 3.2。
  - 命令：`cargo test daemon event`
  - 命令：`cargo run -- emit hermes.agent.started --payload '{"session_id":"demo"}'`
- [ ] 提交任务 3.2。
  - commit：`feat: 增加 daemon event ingress`

### 任务 3.3：Hermes hook ingress

- [ ] 实现 `/api/hermes/hook`。
  - 接收：`provider`、`source`、`event`、`context`。
  - 输出：标准 `IncomingEvent`。
- [ ] 实现 `hermeship hermes hook --payload`。
  - 支持 stdin `--payload -`。
- [ ] 编写 Hermes hook normalization 测试。
  - 覆盖：`gateway:startup`、`session:start`、`session:end`、`session:reset`、`agent:start`、`agent:end`。
  - 要求：复用 `tests/fixtures/hermes/`，并断言隐私清洗发生在入队前。
- [ ] 验证任务 3.3。
  - 命令：`cargo test hermes`
  - 命令：`printf '%s' '{"event":"agent:start","context":{"session_id":"demo"}}' | cargo run -- hermes hook --payload -`
- [ ] 提交任务 3.3。
  - commit：`feat: 增加 Hermes hook ingress`

## Milestone 4：Router、Renderer、Dispatcher

目标：移植 clawhip 核心事件分发管道。

### 任务 4.1：Router

- [ ] 新建 router 模块。
  - 新建：`src/router.rs`
  - 类型：`ResolvedDelivery`、`SinkTarget`、`DeliveryExplanation`。
- [ ] 实现 route match。
  - 支持 event glob。
  - 支持 filter map。
  - 支持 0..N delivery。
- [ ] 实现 route candidates。
  - `hermes.agent.*`、`hermes.session.*`、`hermes.*`。
- [ ] 实现 `hermeship explain`。
  - 展示 matched routes、failed filters、delivery target。
- [ ] 编写 router 测试。
  - 覆盖：多 route、filter 命中/未命中、缺 channel、template/format/mention 继承。
- [ ] 编写 `explain` 合约测试。
  - 覆盖：route 命中原因、filter 失败原因、无 route、delivery target。
  - 完成标准：`explain` 输出可用于定位配置问题，不只返回布尔结果。
- [ ] 验证任务 4.1。
  - 命令：`cargo test router`
  - 命令：`cargo run -- explain hermes.agent.started --payload '{"platform":"telegram","session_id":"demo"}'`
- [ ] 提交任务 4.1。
  - commit：`feat: 实现多投递路由`

### 任务 4.2：Renderer

- [ ] 新建 render 模块。
  - 新建：`src/render/mod.rs`
  - 新建：`src/render/default.rs`
- [ ] 实现默认 renderer。
  - 支持：`compact`、`inline`、`alert`、`raw`。
- [ ] 实现 Hermes 事件渲染。
  - gateway/session/agent/custom。
- [ ] 实现 template 渲染。
  - 支持 `{session_id}`、`{platform}`、`{project}`、`{event}` 等上下文 token。
- [ ] 编写 renderer 测试。
  - 覆盖：所有格式、缺字段降级、raw JSON、template token。
- [ ] 验证任务 4.2。
  - 命令：`cargo test render`
- [ ] 提交任务 4.2。
  - commit：`feat: 增加 Hermes 默认渲染器`

### 任务 4.3：Dispatcher 与 fake sink

- [ ] 新建 dispatch/sink 模块。
  - 新建：`src/dispatch.rs`
  - 新建：`src/sink/mod.rs`
  - 新建：`src/sink/fake.rs`
- [ ] 实现 `Sink` trait。
- [ ] 实现 fake sink。
  - 用于测试保存 delivery。
- [ ] 实现 dispatcher。
  - 从队列读取 event。
  - route -> render -> sink。
  - 单个 delivery 失败不影响其他 delivery。
- [ ] 编写 dispatcher 测试。
  - 覆盖：多投递、单 sink failure、无 route、render failure。
- [ ] 编写 dispatcher E2E 测试。
  - 使用：`tests/fixtures/hermes/agent_start.json` + fake sink。
  - 断言：daemon 入队事件经过 route -> render -> fake sink 后保存 delivery，且 message 不泄漏敏感字段。
- [ ] 验证任务 4.3。
  - 命令：`cargo test dispatch sink`
- [ ] 提交任务 4.3。
  - commit：`feat: 实现事件 dispatcher 与 fake sink`

## Milestone 5：Discord Sink 与基础 Live Path

目标：实现第一条真实通知链路。

### 任务 5.1：Discord 配置与 payload

- [ ] 新建 Discord sink。
  - 新建：`src/sink/discord.rs`
- [ ] 支持 bot token + channel。
- [ ] 支持 webhook URL。
- [ ] 实现 Discord message payload。
  - 内容长度限制。
  - mention 前缀。
  - allowed mentions 策略。
- [ ] 编写 Discord sink 单元测试。
  - 使用 fake HTTP 或 request builder 测试。
  - 覆盖：webhook payload、bot channel payload、allowed mentions、消息长度截断。
- [ ] 验证任务 5.1。
  - 命令：`cargo test discord`
- [ ] 提交任务 5.1。
  - commit：`feat: 增加 Discord sink`

### 任务 5.2：Sink 失败语义

- [ ] 测试 token 缺失。
  - 预期：delivery failed，不 panic。
- [ ] 测试非 2xx。
  - 预期：记录 status/body tail。
- [ ] 测试 rate limit。
  - 预期：尊重 retry 信息或记录明确诊断。
- [ ] 使用 fake HTTP server 覆盖 Discord 失败矩阵。
  - 覆盖：2xx、4xx、5xx、429 rate limit、空 token、空 channel。
  - 完成标准：默认测试不访问真实 Discord API。
- [ ] 测试多个 delivery 其中一个失败。
  - 预期：其他 delivery 继续。
- [ ] 验证任务 5.2。
  - 命令：`cargo test sink dispatch`
- [ ] 提交任务 5.2。
  - commit：`feat: 完善 sink 失败处理`

### 任务 5.3：本地端到端 smoke

- [ ] 编写 daemon + fake sink E2E。
  - 启动 test daemon。
  - POST `/api/hermes/hook`。
  - 断言 fake sink 收到渲染消息。
  - 断言默认隐私保护生效。
- [ ] 验证 `send`。
  - 命令：`cargo run -- send --channel test --message "hello"`
  - 使用 fake/dry-run 模式。
- [ ] 验证 `emit`。
  - 命令：`cargo run -- emit hermes.agent.started --payload '{"session_id":"demo"}'`
- [ ] 提交任务 5.3。
  - commit：`test: 增加 daemon 到 sink 的端到端覆盖`

## Milestone 6：Hermes Hook Bridge 安装

目标：让 Hermes gateway 能通过 hook bridge 投递到 Hermeship。

### 任务 6.1：Hook 模板

- [ ] 创建 Hermes hook 模板目录。
  - 新建：`templates/hermes-hook/HOOK.yaml`
  - 新建：`templates/hermes-hook/handler.py`
- [ ] `HOOK.yaml` 声明事件。
  - `gateway:startup`
  - `session:start`
  - `session:end`
  - `session:reset`
  - `agent:start`
  - `agent:end`
  - `agent:step` 默认可配置禁用。
- [ ] `handler.py` 只使用标准库。
  - 不 import Hermeship package。
  - 调用 `hermeship hermes hook --payload -`。
  - 超时 fail-open。
- [ ] 编写模板测试。
  - 覆盖：manifest 可解析、handler 包含 fail-open 逻辑、不包含 secret。
  - 要求：`handler.py` 只使用 Python 标准库。
- [ ] 验证任务 6.1。
  - 命令：`cargo test hooks`
- [ ] 提交任务 6.1。
  - commit：`feat: 增加 Hermes hook bridge 模板`

### 任务 6.2：Installer

- [ ] 新建 hooks installer。
  - 新建：`src/hooks/mod.rs`
  - 函数：`install_hermes_hooks(home, force)`。
- [ ] 实现 CLI。
  - `hermeship hermes install-hooks --home <path> --force`
- [ ] 支持 dry-run。
  - 打印将写入的文件，不修改磁盘。
- [ ] 编写 installer 测试。
  - 覆盖：首次安装、不覆盖、force 覆盖、dry-run、返回路径。
  - 使用：fake Hermes home。
- [ ] 验证任务 6.2。
  - 命令：`cargo test hooks`
  - 命令：`cargo run -- hermes install-hooks --home /tmp/hermeship-test-home --force`
  - 命令：`find /tmp/hermeship-test-home/hooks/hermeship -maxdepth 1 -type f -print`
- [ ] 提交任务 6.2。
  - commit：`feat: 支持安装 Hermes gateway hooks`

### 任务 6.3：Bridge smoke 与回滚

- [ ] 编写 Python handler smoke test。
  - 使用临时 `HERMES_HOME`。
  - 直接 import/exec handler module。
  - fake `hermeship` binary 验证 stdin payload。
  - 覆盖：binary missing、调用 timeout、子进程失败时 fail-open。
- [ ] 实现 uninstall/remove hooks。
  - `hermeship hermes uninstall-hooks --home <path>`
- [ ] 编写回滚测试。
  - 安装 -> 卸载 -> 确认目录删除或 marker 删除。
- [ ] 验证任务 6.3。
  - 命令：`cargo test hooks`
  - 命令：`cargo run -- hermes uninstall-hooks --home /tmp/hermeship-test-home`
- [ ] 提交任务 6.3。
  - commit：`feat: 支持 Hermes hook 回滚`

## Milestone 7：安装、生命周期与运维 CLI

目标：补齐 clawhip 风格的可运维项目表面。

### 任务 7.1：Install/Setup

- [ ] 实现 `hermeship install`。
  - 创建 `~/.hermeship`。
  - scaffold `config.toml`。
  - 输出下一步命令。
- [ ] 实现 `hermeship setup`。
  - 支持设置 Discord token、default channel、daemon URL。
  - 不打印 secret。
- [ ] 编写 install/setup 测试。
  - 使用临时 HOME。
- [ ] 验证任务 7.1。
  - 命令：`cargo test lifecycle`
- [ ] 提交任务 7.1。
  - commit：`feat: 增加 hermeship install setup`

### 任务 7.2：Service 与 Uninstall

- [ ] 增加 systemd service 模板。
  - 新建：`deploy/hermeship.service`
- [ ] 增加 launchd 文档或模板。
  - macOS 先文档化，是否实现视环境决定。
- [ ] 实现 `hermeship uninstall`。
  - 可选删除 config/state/service/hooks。
- [ ] 编写 lifecycle 测试。
  - 覆盖：不误删、force/remove-config 行为。
- [ ] 验证任务 7.2。
  - 命令：`cargo test lifecycle`
- [ ] 提交任务 7.2。
  - commit：`feat: 增加安装生命周期管理`

### 任务 7.3：Release preflight

- [ ] 新建 release preflight。
  - 新建：`src/release_preflight.rs`
- [ ] 检查项目一致性。
  - CLI help。
  - 配置示例。
  - docs 命令。
  - hook 模板包含。
  - 测试夹具完整性。
  - live verification 必填字段。
- [ ] 实现 CLI。
  - `hermeship release preflight <version>`
- [ ] 验证任务 7.3。
  - 命令：`cargo run -- release preflight 0.1.0`
- [ ] 提交任务 7.3。
  - commit：`chore: 增加 release preflight`

## Milestone 8：clawhip 功能 Parity 扩展

目标：按 clawhip 能力补齐非 Hermes 专属 sources。

### 任务 8.1：Git Source

- [ ] 新建 git source。
  - 新建：`src/source/git.rs`
- [ ] 实现 commit/branch 事件。
- [ ] 实现 `hermeship git commit` 和 `hermeship git branch-changed`。
- [ ] 编写测试。
  - 覆盖：repo/branch/commit summary、路由 metadata。
- [ ] 验证任务 8.1。
  - 命令：`cargo test git`
- [ ] 提交任务 8.1。
  - commit：`feat: 增加 git 事件 source`

### 任务 8.2：GitHub Source

- [ ] 新建 GitHub source。
  - 新建：`src/source/github.rs`
- [ ] 实现 issue/PR/CI/release 事件。
- [ ] 实现 `hermeship github ...` CLI。
- [ ] 编写测试。
  - 使用 fixture，不依赖真实 GitHub。
- [ ] 验证任务 8.2。
  - 命令：`cargo test github`
- [ ] 提交任务 8.2。
  - commit：`feat: 增加 GitHub 事件 source`

### 任务 8.3：Tmux Source

- [ ] 新建 tmux source。
  - 新建：`src/source/tmux.rs`
- [ ] 实现 keyword/stale 事件。
- [ ] 实现 `hermeship tmux keyword/stale/watch/list`。
- [ ] 编写测试。
  - fake tmux 命令输出。
- [ ] 验证任务 8.3。
  - 命令：`cargo test tmux`
- [ ] 提交任务 8.3。
  - commit：`feat: 增加 tmux 事件 source`

### 任务 8.4：Cron 与 Memory Scaffold

- [ ] 新建 cron 模块。
  - 新建：`src/cron.rs`
- [ ] 支持 configured cron job run。
- [ ] 新建 memory scaffold。
  - 新建：`src/memory.rs`
  - CLI：`hermeship memory init/status`
- [ ] 编写测试。
- [ ] 验证任务 8.4。
  - 命令：`cargo test cron memory`
- [ ] 提交任务 8.4。
  - commit：`feat: 增加 cron 与 memory scaffold`

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
