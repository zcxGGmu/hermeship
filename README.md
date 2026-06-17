# Hermeship

Hermeship 是 Hermes-native daemon-first event router。它从 Hermes gateway hooks、CLI、本地 source scaffold 接收事件，规范化为 typed event envelope，经队列、router、renderer 和 sink 投递到 Discord 等通知渠道。

Hermeship 不是 clawhip thin adapter：

- 不调用 `clawhip` binary。
- 不依赖运行中的 `clawhip` daemon。
- 不修改 Hermes 核心。
- 不把通知消息写回 Hermes 对话上下文。

`/Users/zq/Desktop/ai-projs/posp/template/clawhip` 只作为架构和行为参考。

## Current State

Milestone 0 到 Milestone 9.1 已完成。当前已实现：

- Rust CLI、配置模型、质量门禁。
- daemon `/health`、`/event`、`/api/hermes/hook`。
- `IncomingEvent -> EventEnvelope` typed event 管道。
- 默认隐私清洗、路由、渲染、dispatcher。
- fake sink、Discord sink、Discord 失败语义。
- Hermes gateway hook bridge 安装和安全卸载。
- 本地 install/setup/uninstall lifecycle 和 release preflight。
- deterministic Git/GitHub/tmux/cron source CLI 路径。
- memory filesystem scaffold。
- README、operations、Hermes event contract 和 architecture 文档。

仍未完成：

- `docs/live-verification.md` runbook。
- 真实 live verification 记录。
- Slack sink。
- Hermes plugin/observer。
- 真实 GitHub API source、真实 tmux watch、真实 scheduler、真实 service manager 自动安装。

## Architecture

```text
Hermes gateway hooks / CLI / git / GitHub / tmux / cron
  -> source ingress
  -> IncomingEvent
  -> typed EventEnvelope
  -> bounded queue
  -> Dispatcher
  -> Router
  -> Renderer
  -> Sink
  -> Discord
```

关键模块：

- `src/cli.rs`：公开命令树。
- `src/config.rs`：TOML 配置、默认值、env override、route schema。
- `src/daemon.rs`：本地 HTTP daemon、队列和 ingress。
- `src/events.rs`：外部 `IncomingEvent`。
- `src/event/`：typed event body、metadata 和兼容映射。
- `src/privacy.rs`：payload 清洗和摘录策略。
- `src/router.rs`：event glob、metadata filter、0..N delivery。
- `src/render/`：`compact`、`inline`、`alert`、`raw` 渲染。
- `src/sink/`：Discord sink 和 fake sink。
- `src/hooks.rs`：Hermes gateway hook bridge 安装/卸载。
- `src/lifecycle.rs`：本地 install/setup/uninstall。
- `src/release_preflight.rs`：发布一致性检查。

更详细的模块边界见 `ARCHITECTURE.md`。

## Install

开发期本地安装：

```bash
cargo install --path .
hermeship install
```

`hermeship install` 默认创建：

```text
~/.hermeship/
  config.toml
  hooks/
  logs/
  state/
```

可先 dry-run：

```bash
hermeship install --dry-run
```

## Configure

推荐从 stdin 写入 Discord token，避免 token 出现在 shell history 或 process argv：

```bash
printf '%s' "$DISCORD_TOKEN" | hermeship setup \
  --discord-token-stdin \
  --default-channel <discord-channel-id> \
  --daemon-url http://127.0.0.1:25295
```

也可以从环境变量读取：

```bash
hermeship setup --discord-token-env HERMESHIP_SETUP_DISCORD_TOKEN
```

常用配置检查：

```bash
hermeship config path
hermeship config show
hermeship config verify
```

环境变量覆盖：

- `HERMESHIP_CONFIG`
- `HERMESHIP_DAEMON_URL`
- `HERMESHIP_DISCORD_TOKEN`
- `HERMESHIP_DEFAULT_CHANNEL`
- `HERMESHIP_DRY_RUN`

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
```

## Run

启动 daemon：

```bash
hermeship start
```

检查状态：

```bash
hermeship status
```

daemon 默认监听 `http://127.0.0.1:25295`，提供：

- `GET /health`
- `POST /event`
- `POST /api/hermes/hook`

## Hermes Hooks

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

hook handler 只使用 Python 标准库，调用：

```bash
hermeship hermes hook --payload -
```

handler 默认 fail-open：找不到 binary、daemon 不可用、子进程失败或超时都只输出短诊断，不向 Hermes 抛异常。

卸载 hook：

```bash
hermeship hermes uninstall-hooks --home ~/.hermes
```

Hermeship 只通过 `.hermeship-managed.json` marker 删除自己管理且未被用户修改的 hook 文件。

## Send And Emit

发送 custom message：

```bash
hermeship send --channel <discord-channel-id> --message "hermeship smoke"
```

发送 Hermes event：

```bash
hermeship emit hermes.agent.started --payload '{"session_id":"demo","platform":"telegram"}'
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

## Local Source Commands

这些命令是本地 deterministic source path。它们构造 Hermeship 事件并 POST 到 daemon，不访问真实 GitHub API、不读取真实 tmux session、不运行真实 scheduler。

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

Memory scaffold is local filesystem-only:

```bash
hermeship memory init --root /tmp/hermeship-memory --project Hermeship --channel ops --agent codex --date 2026-06-17
hermeship memory status --root /tmp/hermeship-memory --project Hermeship --channel ops --agent codex --date 2026-06-17
```

## Privacy

Hermeship routes summaries and structured metadata, not full conversations.

Default sanitizer behavior:

- recursively redacts token, cookie, secret, API key, password and authorization-like keys;
- drops full `message`, `response`, `conversation_history`, provider request/response bodies and tool result bodies;
- keeps safe summaries such as `message_chars`, `response_chars`, `has_message`, `has_response`;
- only emits message/response excerpts when explicitly enabled in config, after sanitizer and length bounding.

`raw` rendering is still safe JSON: it serializes typed, controlled fields and sanitized payload summaries rather than arbitrary original payload bodies.

## rollback

Hook rollback:

```bash
hermeship hermes uninstall-hooks --home ~/.hermes
```

Local Hermeship files are preserved by default:

```bash
hermeship uninstall
```

Destructive local rollback requires explicit flags and a Hermeship-managed home marker:

```bash
hermeship uninstall --remove-state --remove-config --remove-hooks --hermes-home ~/.hermes
```

More operational detail is in `docs/operations.md`.

## Live Check

Live verification is separate from default tests. It needs an explicit test Discord channel, Discord credentials and a Hermes gateway environment.

Planned manual flow:

```bash
hermeship start
hermeship status
hermeship send --channel <discord-channel-id> --message "hermeship live check"
hermeship emit hermes.agent.started --payload '{"session_id":"live-check"}'
hermeship hermes install-hooks --scope global --force
hermeship hermes uninstall-hooks --home ~/.hermes
```

Results must be recorded in `docs/live-verification.md` without tokens, cookies, secrets, full prompts, full conversations or provider request/response bodies. If credentials are unavailable, the record must say which live checks were not run and what risk remains.

## Release Preflight

```bash
hermeship release preflight 0.1.0
```

Preflight checks local release consistency: Cargo version, `Cargo.lock`, public CLI fixture, docs command coverage, hook templates, fixture policy, service template and live verification status. Missing live verification is `pending`, not a default local failure.

## Development Gates

Before a stage commit:

```bash
cargo test release_preflight
cargo run -- release preflight 0.1.0
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Default tests must stay local and deterministic. They must not require real Discord, real Hermes gateway, real GitHub state, real tmux sessions, external credentials or non-local network state.
