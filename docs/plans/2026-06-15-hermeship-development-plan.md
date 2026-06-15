# Hermeship 开发方案

日期：2026-06-15

## 1. 文档定位

本文是 `hermeship` 的中文开发方案，描述系统目标、边界、架构、事件契约、安全策略、验证策略和发布策略。

具体迭代任务、进度勾选、每阶段验证命令和提交边界统一维护在：

- `tasks/development-checklist.md`

方案文档回答“为什么这样设计、边界在哪里、最终系统长什么样”；开发清单回答“当前做到哪一步、下一步做什么、如何验收”。

## 2. 修正后的目标理解

`hermeship` 不是一个调用现有 `clawhip` binary 的薄适配器。

`hermeship` 的目标是以 `/Users/zq/Desktop/ai-projs/posp/template/clawhip` 为工程模板，为 Hermes 构建一个 Hermes-native 的事件到通知渠道路由项目。它应参考 `clawhip` 的 daemon-first 架构、CLI 形态、事件模型、路由、渲染、sink、安装、配置、运维和 live verification，只把 OpenClaw、Codex、Claude、OMC/OMX 等耦合点替换为 Hermes 的 gateway hooks、plugin hooks、session/agent 生命周期、配置和安装习惯。

正确方向：

- `clawhip` 是架构和功能参考，不是运行时依赖。
- `hermeship` 自己提供 daemon、router、renderer、sink 和 CLI。
- Hermes 通过 hook bridge 或后续 plugin/observer 向 `hermeship` 投递事件。
- 下游通知由 `hermeship` 直接投递到 Discord、Slack 等渠道。

## 3. 背景

`clawhip` 是一个 daemon-first 的事件到频道通知路由器。它接收 CLI、GitHub、git、tmux、自定义事件和 provider-native hook payload，经过标准化、路由、渲染后发送到 Discord/Slack 等下游。

Hermes 是一个跨 CLI、网关、多聊天平台、插件、技能和子代理的个人 AI agent。Hermes 的扩展原则是核心窄、能力在边缘扩展。Hermeship 应遵守这个原则：不把通知逻辑塞进 Hermes 对话上下文，不修改 Hermes 核心，而是在 Hermes 边缘接收生命周期事件并进入独立通知管道。

## 4. 产品目标

第一阶段的 Hermeship 应做到：

- 作为 Hermes-native 的 daemon-first 通知路由器。
- 提供 `hermeship` CLI，覆盖安装、启动、状态、配置、发送、解释路由、release preflight 等运维入口。
- 提供本地 daemon，接收事件并通过队列分发。
- 提供 typed event model，将外部 ingress 标准化为内部事件 envelope。
- 提供 multi-delivery router，一个事件可投递到 0..N 个目的地。
- 提供 renderer/sink 分离，第一版至少支持 Discord，后续支持 Slack。
- 提供 Hermes gateway hook bridge 安装和转发能力。
- 提供 custom event、Hermes lifecycle event、git/GitHub/tmux/cron 等与 clawhip 对齐的事件能力路线。
- 默认 fail-open：Hermeship 失败不能影响 Hermes gateway 或 agent 正常运行。
- 默认保护隐私：不发送完整对话、完整 prompt、provider 请求/响应、tool result body、token、cookie 或 secret。
- 提供可重复的本地测试、fake sink、fake Hermes hook、daemon smoke test 和 live Discord verification runbook。

## 5. 非目标

MVP 不做以下事情：

- 不依赖运行中的 `clawhip` daemon。
- 不通过 `clawhip agent` 或 `clawhip emit` 作为核心投递路径。
- 不修改 Hermes 核心代码。
- 不新增 Hermes model tool。
- 不把通知消息回灌到 Hermes 对话上下文。
- 不复用 Hermes 聊天 bot token 作为通知 bot token。
- 不把 Hermes 塞进 clawhip 已冻结的 Codex/Claude native hook v1 契约。
- 不默认转发完整 `command:*`、完整 tool body、完整 LLM request/response。
- 不在 MVP 中实现所有 clawhip 历史遗留能力；但要按相同架构为 parity 留出清晰路径。

## 6. 设计原则

- **架构同构**：优先参考 `clawhip` 的 daemon、source、dispatcher、router、renderer、sink 模块边界。
- **Hermes 原生**：接入点、命名、安装和配置面向 Hermes，而不是 OpenClaw/Codex/Claude。
- **失败开放**：hook bridge、daemon、sink、配置失败都不能中断 Hermes。
- **边界清晰**：Hermes 只产生事件，Hermeship 负责路由和投递，下游渠道只接收通知。
- **事件小而稳**：事件承载摘要和结构化元数据，不承载大段内容。
- **隐私默认安全**：默认丢弃高风险字段，递归脱敏敏感 key，正文摘录必须显式启用。
- **配置兼容演进**：TOML schema 首版稳定，后续只做向后兼容扩展。
- **先契约后实现**：先锁定 Hermes hook context、CLI event shape、daemon API、route/filter 语义，再写代码。
- **先验证再提交**：每个阶段完成后运行对应验证命令并提交。

## 7. 技术栈

Hermeship 应采用与 clawhip 接近的 Rust daemon-first 技术栈：

- Rust 2024 edition。
- `tokio`：异步运行时和队列。
- `axum`：本地 daemon HTTP API。
- `clap`：CLI。
- `serde` / `serde_json`：事件和配置序列化。
- `toml`：配置文件。
- `reqwest`：Discord/Slack/GitHub 等 HTTP 调用。
- `time`、`uuid`：事件时间戳和 ID。
- `tempfile`、fake sink/fake bridge：测试辅助。

Hermes hook bridge 使用 Python 标准库脚本，因为 Hermes gateway hook 运行 `handler.py`。该脚本应尽量薄：读取 `event_type/context`，调用 `hermeship hermes hook` 或向本地 daemon POST 事件，捕获所有异常并 fail-open。bridge 不应要求 Hermes gateway Python 环境能 import `hermeship` Python package。

## 8. 总体架构

```text
Hermes gateway hooks / Hermes plugins / CLI / git / GitHub / tmux / cron
  -> source ingress
  -> IncomingEvent
  -> typed EventEnvelope
  -> mpsc queue
  -> Dispatcher
  -> Router
  -> Renderer
  -> Sink
  -> Discord / Slack / webhook
```

核心模块建议对齐 clawhip：

| 模块 | 职责 |
| --- | --- |
| `src/main.rs` | CLI dispatch 入口 |
| `src/cli.rs` | `clap` 命令树 |
| `src/config.rs` | TOML 配置、默认值、兼容迁移 |
| `src/client.rs` | daemon client |
| `src/daemon.rs` | HTTP server、队列、source 启动 |
| `src/events.rs` | 外部 `IncomingEvent` 和兼容别名 |
| `src/event/` | typed event body/envelope/compat |
| `src/router.rs` | route/filter/multi-delivery |
| `src/dispatch.rs` | queue consumer、render、sink 调用 |
| `src/render/` | compact/alert/inline/raw renderer |
| `src/sink/` | Discord、Slack、fake sink |
| `src/source/` | Hermes、git、GitHub、tmux、workspace source |
| `src/hooks/` | Hermes hook bridge install、prompt/delivery helpers |
| `src/native_hooks.rs` 或 `src/hermes_hooks.rs` | Hermes hook payload normalization |
| `src/lifecycle.rs` | install/update/uninstall/service |
| `src/release_preflight.rs` | 发布一致性检查 |

## 9. CLI 设计

Hermeship CLI 应参考 clawhip 命令形态，但使用 Hermes 命名：

```bash
hermeship start
hermeship status
hermeship setup
hermeship config
hermeship send --channel <id> --message "..."
hermeship emit <event> --payload '{"...": "..."}'
hermeship hermes hook --provider gateway --payload '{"event":"agent:start"}'
hermeship hermes install-hooks --scope global
hermeship git commit ...
hermeship github issue-opened ...
hermeship tmux keyword ...
hermeship cron run <id>
hermeship explain <event> --payload '{"...": "..."}'
hermeship release preflight <version>
hermeship install
hermeship uninstall
```

MVP CLI 必须包含：

- `start`
- `status`
- `send`
- `emit`
- `explain`
- `config show/path/verify`
- `hermes hook`
- `hermes install-hooks`
- `install`
- `uninstall`

`git`、`github`、`tmux`、`cron`、`memory` 可以分阶段实现，但接口风格应提前预留。

## 10. Daemon API

本地 daemon 默认监听：

```text
http://127.0.0.1:25295
```

建议 API：

| 方法 | 路径 | 职责 |
| --- | --- | --- |
| `GET` | `/health` | daemon health/status |
| `POST` | `/event` | 接收通用 `IncomingEvent` |
| `POST` | `/api/hermes/hook` | 接收 Hermes hook envelope |
| `POST` | `/api/native/hook` | 后续 provider-agnostic ingress |
| `GET` | `/api/routes/explain` | route debug，可由 CLI 包装 |
| `GET` | `/api/update/status` | 后续 update 状态 |

所有 ingress 在进入队列前必须：

1. 解析 JSON。
2. 规范化事件名和 payload。
3. 转为 typed envelope。
4. 拒绝或降级高风险 payload。
5. 返回可诊断响应。

## 11. Hermes 接入

### Gateway Hook Bridge

Hermes 已支持 `~/.hermes/hooks/<name>/HOOK.yaml + handler.py`，事件包括：

- `gateway:startup`
- `session:start`
- `session:end`
- `session:reset`
- `agent:start`
- `agent:step`
- `agent:end`
- `command:*`

MVP 安装：

```text
~/.hermes/hooks/hermeship/
  HOOK.yaml
  handler.py
```

`handler.py` 设计要求：

- 只使用 Python 标准库。
- 不导入 Hermeship Rust/Python package。
- 将 `event_type` 和 `context` 序列化为 JSON。
- 优先调用 `hermeship hermes hook --payload -`。
- 可选支持直接 POST `http://127.0.0.1:25295/api/hermes/hook`。
- 默认超时 2 秒。
- 捕获所有异常并打印短诊断，不抛给 Hermes。

### Hermes Plugin / Observer

后续阶段可提供 Hermes plugin，用于更高保真事件：

- `on_session_start`
- `on_session_end`
- `pre_tool_call`
- `post_tool_call`
- `pre_llm_call`
- `post_llm_call`
- `api_request_error`
- `subagent_start`
- `subagent_stop`

plugin 不是 MVP 的第一入口。只有 gateway hook MVP 稳定后才启动。

## 12. 事件模型

外部 ingress 使用 `IncomingEvent`：

```rust
pub struct IncomingEvent {
    pub kind: String,
    pub channel: Option<String>,
    pub mention: Option<String>,
    pub format: Option<MessageFormat>,
    pub template: Option<String>,
    pub payload: serde_json::Value,
}
```

内部使用 typed envelope：

```rust
pub struct EventEnvelope {
    pub id: Uuid,
    pub timestamp: OffsetDateTime,
    pub source: String,
    pub body: EventBody,
    pub metadata: EventMetadata,
}
```

首批 `EventBody`：

- `Custom`
- `HermesGatewayStarted`
- `HermesSessionStarted`
- `HermesSessionFinished`
- `HermesSessionReset`
- `HermesAgentStarted`
- `HermesAgentStep`
- `HermesAgentFinished`
- `HermesAgentFailed`
- `GitCommit`
- `GitBranchChanged`
- `GitHubIssue`
- `GitHubPR`
- `TmuxKeyword`
- `TmuxStale`

## 13. Hermes 事件契约

Hermes hook envelope：

```json
{
  "provider": "hermes",
  "source": "gateway",
  "event": "agent:start",
  "context": {
    "platform": "telegram",
    "user_id": "...",
    "chat_id": "...",
    "thread_id": "",
    "chat_type": "group",
    "session_id": "...",
    "message": "..."
  }
}
```

规范化事件：

| Hermes hook | Hermeship canonical event | 状态 |
| --- | --- | --- |
| `gateway:startup` | `hermes.gateway.started` | MVP |
| `session:start` | `hermes.session.started` | MVP |
| `session:end` | `hermes.session.finished` | MVP |
| `session:reset` | `hermes.session.reset` | MVP |
| `agent:start` | `hermes.agent.started` | MVP |
| `agent:step` | `hermes.agent.step` | 后续可默认关闭 |
| `agent:end` | `hermes.agent.finished` | MVP |
| `agent:end` with explicit failure | `hermes.agent.failed` | 仅当 payload 明确失败 |
| `command:*` | `hermes.command.<name>` | 后续 opt-in |

说明：

- Hermes gateway 当前 `agent:end` 成功路径主要提供 `response`，真实异常路径未必触发失败 hook。MVP 不声明完整捕获所有 agent failure。
- `message`、`response` 默认不进入通知正文；只保留长度、存在性和可选 opt-in 摘要。
- `origin = "hermeship"` 用于防递归。

## 14. 路由设计

路由配置参考 clawhip：

```toml
[[routes]]
event = "hermes.agent.*"
filter = { platform = "telegram", project = "hermes" }
sink = "discord"
channel = "1234567890"
mention = "<@123>"
format = "compact"
```

路由要求：

- 一个事件可匹配多条 route。
- route filter 基于结构化 metadata，不基于渲染文本。
- 支持 glob event，例如 `hermes.*`、`hermes.agent.*`、`git.*`。
- 支持 route-level format/template/mention/channel。
- `explain` 命令能展示匹配和未匹配原因。

## 15. Renderer 设计

消息格式：

- `compact`：默认，低噪声一行摘要。
- `inline`：密集上下文摘要。
- `alert`：高优先级前缀和 mention。
- `raw`：调试用 JSON。

Hermes 默认 compact 示例：

```text
hermes agent started (platform=telegram, session=abc123, project=hermes)
hermes agent finished (platform=telegram, session=abc123, response_chars=412)
hermes session reset (platform=discord, session=abc123)
```

renderer 不应读取文件、不应调用网络、不应访问 Hermes 内部状态。

## 16. Sink 设计

MVP sink：

- Discord bot token + channel delivery。
- Discord webhook delivery。
- Fake sink for tests。

后续 sink：

- Slack webhook。
- Generic webhook。
- Local JSONL audit sink。

sink 必须处理：

- rate limit。
- 非 2xx 响应。
- token 缺失。
- channel 缺失。
- 单个 delivery 失败不影响其他 delivery。

## 17. 配置设计

默认配置：

```text
~/.hermeship/config.toml
```

示例：

```toml
[daemon]
host = "127.0.0.1"
port = 25295

[providers.discord]
token = ""
default_channel = ""

[defaults]
format = "compact"
project = "hermes"
dry_run = false

[privacy]
include_message_excerpt = false
include_response_excerpt = false
max_excerpt_chars = 240
dedupe_window_secs = 30
redact_keys = ["token", "api_key", "authorization", "password", "secret", "cookie"]

[hermes]
hook_timeout_secs = 2.0
enable_agent_step = false
enable_command_events = false

[[routes]]
event = "hermes.agent.*"
sink = "discord"
channel = ""
format = "compact"
```

配置优先级：

1. CLI flags。
2. 环境变量。
3. repo-local `.hermeship/config.toml`，后续 opt-in。
4. `~/.hermeship/config.toml`。
5. 内置默认值。

环境变量：

- `HERMESHIP_CONFIG`
- `HERMESHIP_DAEMON_URL`
- `HERMESHIP_DRY_RUN`
- `HERMESHIP_DISCORD_TOKEN`
- `HERMESHIP_DEFAULT_CHANNEL`
- `HERMESHIP_HERMES_HOME`

## 18. 隐私与安全

默认禁止发送：

- `conversation_history`
- 完整 prompt
- 完整 user message
- 完整 assistant response
- provider request
- provider response
- tool result body
- token
- cookie
- secret
- API key
- password
- authorization header

处理规则：

- 递归脱敏敏感 key。
- 短 message/response 也不能因“未超过截断长度”而原样发送。
- 默认只发送 `message_chars`、`response_chars`、`has_message`、`has_response`。
- `include_message_excerpt` 和 `include_response_excerpt` 必须显式启用。
- 启用摘录后也必须先脱敏再截断。
- 默认不转发 `command:*`。
- live verification 使用专用测试频道和专用通知 bot。

## 19. 失败处理

Hermeship 必须 fail-open。

| 失败类型 | 行为 | 记录 |
| --- | --- | --- |
| hook bridge 找不到 binary | 跳过 | stderr 短诊断 |
| daemon 不可用 | 跳过或落本地诊断 | endpoint |
| config 不存在 | 使用默认值 | config path |
| config 非法 | 拒绝启动 daemon；hook 跳过 | 解析错误摘要 |
| payload 非法 | 返回 4xx 或跳过 | event kind |
| sink token 缺失 | delivery failed，不影响 daemon | sink name |
| Discord rate limit | 尊重响应，短重试或记录 | status、retry |
| 单个 route/sink 失败 | 继续其他 delivery | delivery id |
| 重复事件 | dedupe suppress | dedupe key |
| 递归事件 | drop | origin |

Hermes hook bridge 的异常不能逃逸到 Hermes。

## 20. 安装与回滚

安装：

```bash
cargo install --path .
hermeship install
hermeship setup
hermeship hermes install-hooks --scope global
hermeship start
hermeship status
```

Hermes hook 目录：

```text
~/.hermes/hooks/hermeship/
  HOOK.yaml
  handler.py
```

Hermeship config/state：

```text
~/.hermeship/
  config.toml
  state/
  hooks/
  logs/
```

回滚：

```bash
rm -rf ~/.hermes/hooks/hermeship
hermeship uninstall
```

如果 Hermes gateway 缓存 hooks，需要重启 Hermes gateway。

## 21. 测试策略

Hermeship 的测试策略按“契约先行、纯逻辑优先、集成可重复、live 单独记录”组织。默认测试不能依赖真实 Hermes gateway、真实 Discord、真实 GitHub 或真实 tmux session；这些外部依赖只进入 live verification。

### 21.1 测试分层

| 层 | 目的 | 运行环境 |
| --- | --- | --- |
| 单元测试 | 验证纯逻辑正确性 | 默认 `cargo test` |
| 契约测试 | 锁定 CLI、HTTP API、Hermes hook payload、配置 schema | 默认 `cargo test` |
| 集成测试 | 验证 daemon、队列、router、renderer、sink 串联 | 默认 `cargo test`，使用 fake sink/fake HTTP |
| E2E smoke | 验证本地 binary、daemon、hook bridge 的最小闭环 | 本地可重复，不能需要真实凭据 |
| Live verification | 验证真实 Discord/Hermes gateway 投递 | 手动或显式 opt-in，结果写入文档 |

### 21.2 测试矩阵

| 层 | 必测内容 |
| --- | --- |
| CLI | subcommand parse、help、错误参数、README/runbook 命令不漂移 |
| config | 默认值、非法 TOML、env override、未知 key、空值归一化、向后兼容 |
| events | normalize、canonical kind、typed conversion、未知事件降级 |
| privacy | 敏感 key 递归脱敏、短正文不泄漏、摘录 opt-in、原始 payload 不被原地修改 |
| Hermes hook | gateway payload、session/agent payload、失败 context、隐私过滤、防递归 |
| daemon | `/health`、`/event`、`/api/hermes/hook`、非法 payload、daemon unavailable |
| router | glob、filter、多 delivery、无 route、`explain` 匹配/未匹配原因 |
| renderer | compact/inline/alert/raw、template token、缺字段降级 |
| dispatcher | queue -> route -> render -> sink，单个 delivery 失败不阻断其他 delivery |
| sink | fake sink、Discord payload、非 2xx、rate limit、token 缺失、channel 缺失 |
| bridge | `HOOK.yaml`、Python handler fail-open、binary missing、timeout、stdin payload |
| install | config scaffold、hook install、service 文件、dry-run、force、不误删、回滚 |
| live | daemon status、Discord delivery、Hermes gateway hook smoke、回滚记录 |

### 21.3 必备测试夹具

MVP 必须先实现以下测试夹具，再依赖相关功能验收：

- **fake sink**：保存每个 delivery 的 target、format、rendered message、metadata，用于 daemon 到 sink 的 E2E。
- **fake HTTP server**：模拟 Discord webhook/bot API，覆盖 2xx、4xx、5xx、rate limit。
- **fake Hermes home**：临时 `HERMES_HOME`，用于安装/卸载 `HOOK.yaml` 和 `handler.py`。
- **fake hermeship binary**：用于 Python `handler.py` smoke test，确认 handler 会把 JSON payload 写入 stdin。
- **fixture payloads**：固定 Hermes gateway hook 样例、非法 payload、敏感字段 payload、路由 payload。

### 21.4 不变量回归

以下行为任何阶段都不能破坏，必须用测试覆盖：

- Hermes hook bridge 失败不能向 Hermes 抛异常。
- 默认不发送完整 message、response、conversation history、provider request/response、tool result body。
- 敏感 key 在任意嵌套层级都必须脱敏。
- 一个 sink 或 route 失败不能阻断其他 delivery。
- `cargo test` 不能依赖真实 Discord token、真实 Hermes gateway、真实 GitHub、真实 tmux。
- `explain` 结果必须能说明 route 命中和 filter 失败原因。
- 文档中的公开命令必须能被 CLI parse 测试或 smoke 测试覆盖。

### 21.5 CI 与 Live 分离

默认 CI 或本地常规验证运行：

基础验证命令：

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- --help
cargo run -- emit hermes.agent.started --payload '{"session_id":"demo"}'
cargo run -- explain hermes.agent.started --payload '{"session_id":"demo"}'
```

这些命令必须不需要外部凭据。真实网络验证只通过 live runbook 执行，并且必须记录：

- 执行日期。
- daemon 版本或 commit。
- 测试频道。
- 触发事件。
- 实际消息形态。
- 未执行项、原因和剩余风险。

## 22. Live Verification

前置条件：

- Hermes 已安装。
- Hermes gateway 可启动。
- Hermeship daemon 可运行。
- Discord 测试 bot token 可用。
- Discord 测试频道可用。

验证流程：

1. `hermeship status` 返回 healthy。
2. `hermeship send --channel <id> --message "hermeship live check"` 到达 Discord。
3. `hermeship emit hermes.agent.started --payload ...` 到达 Discord。
4. `hermeship hermes install-hooks --scope global` 写入 hook。
5. 使用隔离 `HERMES_HOME` 加载 hook，确认 handler fail-open。
6. 启动真实 Hermes gateway，触发 `gateway:startup`。
7. 发送一条测试消息，确认 `agent:start` 和 `agent:end` 通知。
8. 运行回滚，确认 hook 被移除。

未执行的 live check 必须记录原因和剩余风险。

## 23. 版本与发布

版本策略：

- `0.1.0`：Rust daemon、CLI、config、router、renderer、Discord sink、Hermes gateway hook ingress、install/status/send/emit/explain。
- `0.2.0`：git/GitHub/tmux sources、cron、Slack sink、release preflight。
- `0.3.0`：Hermes plugin/observer、更细粒度 tool/LLM telemetry。
- `1.0.0`：配置 schema、事件契约、安装、回滚、live verification 稳定。

发布前必须运行：

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- --help
cargo run -- release preflight <version>
```

## 24. 参考

- `/Users/zq/Desktop/ai-projs/posp/template/clawhip/ARCHITECTURE.md`
- `/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/cli.rs`
- `/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/main.rs`
- `/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/daemon.rs`
- `/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/events.rs`
- `/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/event/compat.rs`
- `/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/router.rs`
- `/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/render/default.rs`
- `/Users/zq/Desktop/ai-projs/posp/template/clawhip/docs/live-verification.md`
- `/Users/zq/Desktop/ai-projs/posp/agents-contributions/hermes-agent/gateway/hooks.py`
- `/Users/zq/Desktop/ai-projs/posp/agents-contributions/hermes-agent/hermes_cli/plugins.py`
- `tasks/development-checklist.md`
