# Hermeship

<p align="center">
  <img src="docs/assets/branding/hermeship-icon.png" alt="Hermeship 项目图标" width="180">
</p>

<p align="right">
  <a href="./README.md"><img alt="中文" src="https://img.shields.io/badge/%E8%AF%AD%E8%A8%80-%E4%B8%AD%E6%96%87-d97757?style=for-the-badge"></a>
  <a href="./README.en.md"><img alt="English" src="https://img.shields.io/badge/Language-English-8c6f5a?style=for-the-badge"></a>
</p>

Hermeship 是一个面向 Hermes 运行环境的独立、daemon-first 事件通知路由器。它拥有自己的 Hermes 事件契约、Rust daemon、路由、渲染、投递和发布验证流程。

## 项目定位

Hermeship 从 Hermes gateway hooks、可选 Hermes observer plugin、CLI 和本地 deterministic source 命令接收事件，将它们规范化为 typed event envelope，经隐私清洗、队列、dispatcher、router、renderer 和 sink 投递到 Discord 等通知渠道。

Hermeship 的公开运行边界：

- 不修改 Hermes 核心。
- 不把通知消息写回 Hermes 对话上下文。
- 默认不启用 observer plugin，必须由 operator 显式安装并在 Hermes 中手动启用。
- 默认测试和 source 命令走本地 deterministic 路径；真实 Discord/Hermes 验证独立记录。

## 图表

![Hermeship architecture](docs/assets/diagrams/hermeship-architecture.png)

![Hermeship event flow](docs/assets/diagrams/hermeship-event-flow.png)

![Hermes observer framework](docs/assets/diagrams/hermeship-observer-framework.png)

图表源文件位于 `docs/assets/diagrams/*.json`，对应导出为 `.svg` 和 `.png`。它们使用 `fireworks-tech-graph` Style 6（Claude Official）生成。

## 当前状态

已完成：

- Rust CLI、配置模型、安装/卸载 lifecycle 和 release preflight。
- daemon `GET /health`、`POST /event`、`POST /api/hermes/hook`。
- `IncomingEvent -> EventEnvelope` typed event 管道。
- 默认隐私清洗、bounded queue、dispatcher、0..N multi-delivery router。
- `compact`、`inline`、`alert`、`raw` 安全渲染。
- Discord bot token/channel 和 Discord webhook sink。
- fake sink、fake HTTP、fake Hermes home 等本地 deterministic 测试路径。
- Hermes gateway hook bridge 安装/卸载。
- 可选 Hermes observer plugin scaffold、install/enable CLI、typed Rust observer body。
- Git/GitHub/tmux/cron 本地 deterministic source CLI 路径。
- memory filesystem scaffold。

仍未完成或不默认启用：

- 真实 Discord/Hermes live verification pass 尚未获得。
- `release preflight` 的 `live verification` ok 只证明 `docs/live-verification.md` 记录字段存在，不证明真实 live pass。
- Slack sink 不在当前默认范围。
- 真实 GitHub API source、真实 tmux watch、真实 scheduler、真实 service manager 自动安装尚未实现。
- observer plugin 不会自动启用，也不会自动修改 Hermes 配置。

## 快速开始

开发期安装：

```bash
cargo install --path .
hermeship install
```

写入 Discord 配置时优先使用 stdin，避免 token 进入 shell history 或 process argv：

```bash
printf '%s' "$DISCORD_TOKEN" | hermeship setup \
  --discord-token-stdin \
  --default-channel <discord-channel-id> \
  --daemon-url http://127.0.0.1:25295
```

也可以从环境变量读取 token：

```bash
hermeship setup --discord-token-env HERMESHIP_SETUP_DISCORD_TOKEN
```

启动和检查 daemon：

```bash
hermeship start
hermeship status
```

默认 daemon endpoint：

```text
http://127.0.0.1:25295
```

公开 HTTP API：

| Method | Path | 作用 |
| --- | --- | --- |
| `GET` | `/health` | 返回 daemon、队列和 sink 配置健康信息 |
| `POST` | `/event` | 接收通用 `IncomingEvent` JSON |
| `POST` | `/api/hermes/hook` | 接收 Hermes gateway hook envelope |

## 配置

常用命令：

```bash
hermeship config path
hermeship config show
hermeship config verify
```

主要环境变量：

- `HERMESHIP_CONFIG`
- `HERMESHIP_DAEMON_URL`
- `HERMESHIP_DISCORD_TOKEN`
- `HERMESHIP_DEFAULT_CHANNEL`
- `HERMESHIP_DRY_RUN`
- `HERMESHIP_HOME`
- `HERMES_HOME`

最小 route 示例：

```toml
[defaults]
channel = "123456789012345678"
format = "compact"

[providers.discord]
token = ""
default_channel = "123456789012345678"

[[routes]]
event = "hermes.agent.*"
sink = "discord"
channel = "123456789012345678"
format = "compact"
filter = { platform = "telegram", project = "Hermeship" }
```

## Hermes Gateway Hooks

安装 Hermes gateway hook bridge：

```bash
hermeship hermes install-hooks --scope global --force
```

默认写入：

```text
~/.hermes/hooks/hermeship/
  HOOK.yaml
  handler.py
  .hermeship-managed.json
```

`handler.py` 只使用 Python 标准库，调用：

```bash
hermeship hermes hook --payload -
```

bridge 设计为 fail-open：找不到 binary、daemon 不可用、子进程失败或超时都只输出短诊断，不向 Hermes 抛异常。

卸载：

```bash
hermeship hermes uninstall-hooks --home ~/.hermes
```

Hermeship 只删除 `.hermeship-managed.json` marker 记录且 checksum 未变化的文件。用户修改过的 hook 文件会保留。

## Hermes Observer Plugin

可选 observer plugin 模板位于：

```text
templates/hermes-plugin/
  plugin.yaml
  __init__.py
```

安装模板：

```bash
hermeship hermes install-plugin --home ~/.hermes --force
```

查看启用指引：

```bash
hermeship hermes enable-plugin --home ~/.hermes --dry-run
```

真正启用仍由 operator 在 Hermes 中执行：

```bash
hermes plugins enable hermeship-observer
```

observer plugin 只注册观察类 hook，callback 返回 `None`，并向 Hermeship daemon `POST /event` 发送 `hermes.observer.*` summary event。它不会使用 `/api/hermes/hook`，不会注册 middleware，不会返回 block/action 指令。

它不转发 raw prompt、conversation history、request/response body、shell command、tool output、child goal、child summary、raw approval `session_key` 或任意错误/原因原文。`hermes.observer.*` 事件进入 typed Rust observer body；与 core metadata 同名的 body 字段通过 `observer_<field>` 路由键访问，不覆盖 core metadata。

本地检查：

```bash
python3 -m py_compile templates/hermes-plugin/__init__.py
cargo test observer_plugin
```

## 发送、事件和路由解释

发送 custom message：

```bash
hermeship send --channel <discord-channel-id> --message "hermeship smoke"
```

发送 Hermes event：

```bash
hermeship emit hermes.agent.started --payload '{"session_id":"demo","platform":"telegram","project":"Hermeship"}'
```

解释路由，不入队、不投递：

```bash
hermeship explain hermes.agent.started --payload '{"session_id":"demo","platform":"telegram"}'
```

直接模拟 Hermes hook ingress：

```bash
printf '%s' '{"event":"agent:start","context":{"session_id":"demo","agent_name":"codex"}}' \
  | hermeship hermes hook --payload -
```

事件契约见 `docs/hermes-event-contract.md`。

## 本地 Source 命令

这些命令当前是本地 deterministic source path。它们构造 Hermeship 事件并 POST 到 daemon，不访问真实 GitHub API、不读取真实 tmux session、不运行真实 scheduler。

```bash
hermeship git commit --repo hermeship --branch main --commit 1234567890abcdef1234567890abcdef12345678 --summary "ship git source"
hermeship git branch-changed --repo hermeship --old-branch main --new-branch codex/milestone-8-git

hermeship github issue-opened --owner posp --repo hermeship --number 42 --title "Add deterministic GitHub source"
hermeship github pr-opened --owner posp --repo hermeship --number 17 --title "Ship GitHub source" --branch codex/milestone-8-github
hermeship github check-failed --owner posp --repo hermeship --workflow ci --status failure --branch main
hermeship github release-published --owner posp --repo hermeship --tag v0.1.0

hermeship tmux keyword --session hermes-agent --keyword FAILED --line "build FAILED at deterministic fixture"
hermeship tmux stale --session hermes-agent --pane %2 --minutes 15 --last-line "waiting for agent output"
hermeship tmux watch --session hermes-agent --keywords FAILED,complete --stale-minutes 10 --tmux-output $'hermes-agent\tmain\t%1\t0\tbash\tready'
hermeship tmux list --tmux-output $'hermes-agent\tmain\t%1\t0\tbash\tready'

hermeship cron run dev-followup
```

Memory scaffold 是本地 filesystem-only：

```bash
hermeship memory init --root /tmp/hermeship-memory --project Hermeship --channel ops --agent codex --date 2026-06-17
hermeship memory status --root /tmp/hermeship-memory --project Hermeship --channel ops --agent codex --date 2026-06-17
```

## 路由、渲染和投递语义

Router 行为：

- event glob 支持 exact kind 和 `*` pattern，例如 `hermes.agent.*`。
- 一个事件可命中 0..N 条 route。
- route filter 基于 structured metadata 和 selected typed body fields，不依赖渲染文本。
- unsupported sink、missing target、disabled route 都会产生可诊断结果。

Target resolution 顺序：

1. route webhook；
2. route channel；
3. event channel hint；
4. `[defaults].channel`；
5. missing delivery target。

Format resolution 顺序：

1. event format hint；
2. route format；
3. `[defaults].format`。

支持格式：

- `compact`
- `inline`
- `alert`
- `raw`

安全模板 token：

- `{event}`
- `{canonical_kind}`
- `{source}`
- `{provider}`
- `{platform}`
- `{session_id}`
- `{agent_name}`
- `{project}`
- `{channel}`

当前生产 sink 是 Discord bot token/channel 与 Discord webhook；测试 sink 是 `FakeSink`。

## 隐私与安全

Hermeship 路由摘要和结构化 metadata，不路由完整对话。

默认 sanitizer：

- 递归 redacts token、cookie、secret、API key、password 和 authorization-like keys。
- 默认删除完整 `message`、`response`、`conversation_history`、provider request/response body 和 tool result body。
- 保留 `message_chars`、`response_chars`、`has_message`、`has_response` 等安全摘要。
- message/response excerpt 必须显式启用，且先 sanitizer 再按长度截断。

`raw` rendering 仍然是安全 JSON：它输出 typed controlled fields 和 sanitized payload summaries，而不是任意原始 payload。

## 回滚

只回滚 Hermes hook：

```bash
hermeship hermes uninstall-hooks --home ~/.hermes
```

保留配置和状态的本地卸载：

```bash
hermeship uninstall
```

显式删除本地状态、日志、配置和 Hermeship-managed hooks：

```bash
hermeship uninstall --remove-state --remove-config --remove-hooks --hermes-home ~/.hermes
```

destructive uninstall 要求 Hermeship home 中存在 `.hermeship-managed.json`，避免误删非 Hermeship 目录。

## Live Verification

Live verification 与默认本地测试分离。真实 Discord/Hermes 检查需要：

- Discord 测试 bot token；
- Discord 测试频道；
- Hermes gateway 测试环境；
- 明确执行确认和 rollback 窗口。

当前真实 Discord/Hermes live verification 尚未获得 `pass`。已有 `blocked` / `not_run` 记录见 `docs/live-verification.md`。默认开发和验证不执行真实 Discord/Hermes live check。

## Release Preflight 和开发门禁

发布预检：

```bash
hermeship release preflight 0.1.0
```

开发阶段提交前默认运行：

```bash
cargo test release_preflight
cargo run -- release preflight 0.1.0
python3 -m py_compile templates/hermes-plugin/__init__.py
cargo test observer_plugin
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

默认测试必须保持本地 deterministic，不要求真实 Discord、真实 Hermes gateway、真实 GitHub state、真实 tmux session、外部 credentials 或非本地网络状态。

## 进一步阅读

- `ARCHITECTURE.md`
- `docs/operations.md`
- `docs/hermes-event-contract.md`
- `docs/observer-plugin.md`
- `docs/live-verification.md`
- `docs/development-status.md`
- `tasks/development-checklist.md`
