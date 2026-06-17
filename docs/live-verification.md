# Hermeship Live Verification

本文记录 Hermeship 的 live verification runbook 和结果字段。默认测试仍然只使用本地 deterministic fixture；真实 Discord/Hermes gateway 检查必须显式 opt-in，并且只在凭据、测试频道和回滚窗口确认后执行。

## Scope

Live verification 只验证真实运行路径是否可操作、可观测、可回滚，不替代默认质量门禁。

包含路径：

- fake sink 本地闭环：daemon ingress -> queue -> router -> renderer -> fake sink。
- daemon health：`hermeship status` 能读取本地 daemon `/health`。
- Discord live：真实 Discord 测试频道收到 custom message 和 `hermes.agent.started` 摘要。
- Hermes gateway hook smoke：隔离 `HERMES_HOME` 安装 hook bridge，并在真实 Hermes gateway 环境触发 `gateway:startup`、`agent:start`、`agent:end`。
- rollback：卸载 Hermes hook bridge，并验证 Hermeship-managed hook 不再被加载。

不包含路径：

- Slack sink。
- Hermes plugin/observer。
- 真实 GitHub API source、真实 tmux watch、真实 scheduler 或真实 service manager 自动安装。
- 完整 prompt、完整对话、provider request/response body 或 tool result body 检查；这些内容不得写入本文。

## Safety Rules

- 不记录 Discord token、webhook URL、cookie、secret、API key、完整 prompt、完整对话、provider request/response body 或 tool result body。
- Discord 测试频道使用专用测试频道；记录频道名或脱敏 id，不记录生产频道上下文。
- Discord token 只通过 stdin 或受控环境变量注入，不放入 shell history 或 process argv。
- Hermes gateway smoke 优先使用隔离 `HERMES_HOME`；如必须使用真实 `~/.hermes`，先备份并确认回滚窗口。
- Hermes hook bridge 设计为 fail-open；任何 hook 失败都不得阻断 Hermes gateway 或 agent。
- 如果 live check 未执行，必须记录原因和剩余风险。

## Result Fields

每次结果记录必须包含以下字段。

| 字段 | 说明 |
| --- | --- |
| 日期 | 执行日期和时区 |
| commit | 被验证的 git commit 或明确说明为 runbook-only baseline |
| operator | 执行人或自动化来源 |
| environment | OS、Hermeship binary 来源、daemon URL、隔离 home |
| test channel | Discord 测试频道名或脱敏 id |
| credentials | 可用性状态，不记录实际 secret |
| trigger event | 触发事件，例如 `send`、`hermes.agent.started`、`agent:start` |
| actual message shape | 实际消息摘要形态，不粘贴完整敏感正文 |
| status | `pass`、`fail`、`not_run` 或 `blocked` |
| evidence | 命令退出码、短日志摘要、消息时间戳或截图引用 |
| not executed | 未执行项和原因 |
| remaining risk | 未覆盖风险 |
| rollback | 回滚命令、结果和残留项 |

## Preconditions

本地 deterministic 前置条件：

- 当前分支和工作树已确认。
- `cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 可运行。
- 本地 fixture 遵守 `tests/fixtures/README.md` 的合成脱敏规则。

真实 Discord/Hermes 前置条件：

- 已安装或可运行当前 Hermeship binary。
- 已确认 `hermeship release preflight 0.1.0` 没有 failed check。
- Discord 测试 bot token 可用，且只用于测试 bot。
- Discord 测试频道可用，且允许测试 bot 发送消息。
- Hermes gateway 可在测试环境启动。
- 已确认 `HERMES_HOME`，优先使用一次性隔离目录。
- 已确认 rollback 窗口和负责人。

## Runbook

### 1. Capture Baseline

```bash
git status --short --branch
git rev-parse --short HEAD
cargo run -- release preflight 0.1.0
```

记录：

- 日期：
- commit：
- binary：
- daemon URL：
- test channel：
- credentials availability：

### 2. Fake Sink Local Verification

目的：证明默认本地测试覆盖 fake sink 闭环，不依赖真实 Discord 或 Hermes gateway。

```bash
cargo test dispatch
cargo test sink
```

记录：

- status：
- command exit code：
- observed fake sink target：
- observed format：
- observed event kind：
- remaining risk：

### 3. Daemon Health Verification

目的：证明本地 daemon 可启动，`hermeship status` 能读取 `/health`。

Terminal A:

```bash
HERMESHIP_DAEMON_URL=http://127.0.0.1:25296 hermeship start --port 25296
```

Terminal B:

```bash
HERMESHIP_DAEMON_URL=http://127.0.0.1:25296 hermeship status
```

记录：

- status：
- `hermeship status` output summary：
- queue status：
- configured sinks summary：
- daemon shutdown method：
- remaining risk：

### 4. Discord Live Verification

目的：证明真实 Discord sink 能收到 custom message 和 Hermes agent event 摘要。

前置：确认 `DISCORD_TOKEN` 和 `DISCORD_CHANNEL_ID` 指向测试 bot 与测试频道。

Setup:

```bash
printf '%s' "$DISCORD_TOKEN" | hermeship setup \
  --discord-token-stdin \
  --default-channel "$DISCORD_CHANNEL_ID" \
  --daemon-url http://127.0.0.1:25295
```

Terminal A:

```bash
hermeship start
```

Terminal B:

```bash
hermeship status
hermeship send --channel "$DISCORD_CHANNEL_ID" --message "hermeship live check"
hermeship emit hermes.agent.started --payload '{"session_id":"live-check","platform":"manual","project":"Hermeship"}'
```

记录：

- Discord status：
- test channel：
- trigger event：`send`、`hermes.agent.started`
- actual message shape：custom text summary、Hermes agent compact summary
- message timestamp：
- not executed：
- remaining risk：

### 5. Hermes Gateway Hook Smoke

目的：证明 Hermes gateway hook bridge 安装、Hermes hook ingress 和 fail-open 边界可操作。

优先使用隔离 Hermes home：

```bash
export HERMES_HOME=/tmp/hermeship-live-hermes-home
hermeship hermes install-hooks --home "$HERMES_HOME" --force
find "$HERMES_HOME/hooks/hermeship" -maxdepth 1 -type f -print
printf '%s' '{"event":"agent:start","context":{"session_id":"live-check","agent_name":"codex","project":"Hermeship"}}' \
  | hermeship hermes hook --payload -
```

真实 Hermes gateway smoke 需要在确认测试环境后执行：

1. 使用相同 `HERMES_HOME` 启动 Hermes gateway。
2. 触发 `gateway:startup`。
3. 发起一条测试消息，触发 `agent:start` 和 `agent:end`。
4. 在 Discord 测试频道确认 Hermes agent 摘要消息。

记录：

- Hermes status：
- `HERMES_HOME`：
- installed files：
- trigger event：`gateway:startup`、`agent:start`、`agent:end`
- actual message shape：
- handler fail-open observations：
- not executed：
- remaining risk：

### 6. Rollback

目的：证明 Hermeship-managed hook 可安全卸载，且本地状态回滚路径明确。

```bash
hermeship hermes uninstall-hooks --home "$HERMES_HOME"
test ! -e "$HERMES_HOME/hooks/hermeship/.hermeship-managed.json"
test ! -e "$HERMES_HOME/hooks/hermeship/HOOK.yaml"
test ! -e "$HERMES_HOME/hooks/hermeship/handler.py"
hermeship uninstall
```

如果 `HOOK.yaml` 或 `handler.py` 仍存在，说明文件被用户修改过或不再匹配 Hermeship-managed checksum。此时不能把 rollback 记为 pass；必须记录 preserved files、人工处理动作和 Hermes gateway restart 状态。

如需显式删除本地状态、日志、配置和 Hermeship-managed hooks：

```bash
hermeship uninstall --remove-state --remove-config --remove-hooks --hermes-home "$HERMES_HOME"
```

记录：

- rollback status：
- hook files removed：
- preserved user-modified files：
- config/state/logs preserved or removed：
- gateway restart required：
- remaining risk：

## Current Results

### 2026-06-17 - Milestone 9.3 Live Check Not Run

#### Baseline

| 字段 | 记录 |
| --- | --- |
| 日期 | 2026-06-17 Asia/Shanghai |
| commit | `6be5661` |
| operator | Codex local development session |
| environment | `/Users/zq/Desktop/ai-projs/posp/hermeship`, branch `codex/milestone-1-cli`, default daemon URL `http://127.0.0.1:25295` |
| test channel | not provided |
| credentials | Discord credentials not provided; Hermes gateway test environment not confirmed |
| trigger event | planned `send`, `hermes.agent.started`, `agent:start`, `agent:end` only |
| actual message shape | not observed in real Discord or real Hermes gateway |
| status | `blocked` for real live verification |
| evidence | Startup instructions explicitly defaulted to not executing real Discord/Hermes live check without credentials and user confirmation; `git status --short --branch` was clean on `codex/milestone-1-cli`; `git log -3 --oneline` showed `6be5661`, `2e60902`, `252ad6a` |
| not executed | manual daemon live session, real Discord custom message, real Discord Hermes event summary, isolated or real Hermes gateway hook smoke, real rollback |
| remaining risk | Real Discord token validity, channel permissions, rate-limit behavior, real Hermes gateway hook loading, gateway restart/cache behavior, handler behavior under the operator environment and real rollback remain unverified |
| rollback | no live hook or Discord state changed; rollback commands remain documented below |

#### Daemon Health Verification

| 字段 | 记录 |
| --- | --- |
| 日期 | 2026-06-17 Asia/Shanghai |
| commit | `6be5661` |
| test channel | not applicable |
| credentials | not applicable |
| trigger event | planned `hermeship start` and `hermeship status` only |
| actual message shape | no notification message |
| status | `not_run` for manual live daemon check |
| evidence | Default deterministic daemon coverage remains in tests; no long-running manual daemon was started in this stage |
| not executed | `hermeship start`, `hermeship status` against a live operator daemon |
| remaining risk | Operator-specific port binding, process supervision and local daemon lifecycle outside the deterministic test harness remain unverified in this live record |
| rollback | no daemon process was started by this live record |

#### Discord Live Verification

| 字段 | 记录 |
| --- | --- |
| 日期 | 2026-06-17 Asia/Shanghai |
| commit | `6be5661` |
| test channel | not provided |
| credentials | Discord credentials not provided |
| trigger event | planned `hermeship send --channel <id> --message "hermeship live check"` and `hermeship emit hermes.agent.started --payload '{"session_id":"live-check"}'` |
| actual message shape | not observed in real Discord |
| status | `not_run` |
| evidence | Real Discord live check was not executed because no Discord token, test channel or explicit execution confirmation was available |
| not executed | Discord custom message and Discord Hermes agent summary |
| remaining risk | Discord auth, channel id correctness, bot permissions, API rate-limit behavior and live delivery formatting remain unverified beyond fake HTTP and local sink coverage |
| rollback | no Discord message was sent by this live record |

#### Hermes Gateway Hook Smoke

| 字段 | 记录 |
| --- | --- |
| 日期 | 2026-06-17 Asia/Shanghai |
| commit | `6be5661` |
| test channel | not provided |
| credentials | Hermes gateway test environment not confirmed |
| trigger event | planned isolated `HERMES_HOME` install smoke and real `gateway:startup`, `agent:start`, `agent:end` only |
| actual message shape | not observed in real Hermes/Discord |
| status | `not_run` |
| evidence | Real Hermes gateway smoke was not executed because no Hermes gateway test environment and explicit execution confirmation were available |
| not executed | isolated `HERMES_HOME` hook install smoke, direct `hermeship hermes hook --payload -` smoke, real Hermes gateway startup and agent lifecycle trigger |
| remaining risk | Real Hermes hook discovery, hook cache/reload behavior, gateway payload shape and fail-open behavior under a live Hermes gateway remain unverified |
| rollback | no Hermes hook was installed by this live record |

#### Rollback Verification

| 字段 | 记录 |
| --- | --- |
| 日期 | 2026-06-17 Asia/Shanghai |
| commit | `6be5661` |
| test channel | not applicable |
| credentials | not applicable |
| trigger event | planned `hermeship hermes uninstall-hooks` only |
| actual message shape | not applicable |
| status | `not_run` |
| evidence | Real rollback was not executed because no live hook install was executed in this stage |
| not executed | `hermeship hermes uninstall-hooks` against an operator Hermes home and post-uninstall hook-file checks |
| remaining risk | User-modified hook preservation, marker checksum behavior against a real Hermes home and Hermes gateway reload behavior remain unverified in live conditions |
| rollback | commands documented; no files changed by this live record |

本轮完成的是 Milestone 9.3 的真实状态记录，而不是一次真实 live pass。真实 Discord/Hermes live verification 仍需要凭据、测试频道、Hermes gateway 测试环境和明确执行确认后重新运行。

### 2026-06-17 - Milestone 9.2 Runbook Creation

#### Baseline

| 字段 | 记录 |
| --- | --- |
| 日期 | 2026-06-17 Asia/Shanghai |
| commit | `252ad6a` runbook-only baseline before this stage commit |
| operator | Codex local development session |
| environment | `/Users/zq/Desktop/ai-projs/posp/hermeship`, branch `codex/milestone-1-cli` |
| test channel | not provided |
| credentials | Discord credentials not provided; Hermes gateway test environment not confirmed |
| status | `pass` for runbook creation |
| evidence | `docs/live-verification.md` created; release preflight live verification fields present |
| not executed | real Discord delivery, real Hermes gateway hook smoke |
| remaining risk | Real Discord permission/token/channel behavior and real Hermes gateway hook loading remain unverified until Milestone 9.3 |
| rollback | rollback commands documented; real rollback not executed because real hook install was not executed |

#### Fake Sink Local Verification

| 字段 | 记录 |
| --- | --- |
| 日期 | 2026-06-17 Asia/Shanghai |
| commit | `252ad6a` runbook-only baseline before this stage commit |
| test channel | not applicable |
| trigger event | local test events through dispatcher/fake sink |
| actual message shape | deterministic fake sink records rendered delivery metadata; no real Discord message |
| status | `pass` |
| evidence | `cargo test` passed, including fake sink and dispatcher coverage |
| not executed | no external Discord or Hermes gateway call |
| remaining risk | Does not prove Discord API permission, channel delivery, or real Hermes gateway hook loading |
| rollback | not applicable |

#### Daemon Health Verification

| 字段 | 记录 |
| --- | --- |
| 日期 | 2026-06-17 Asia/Shanghai |
| commit | `252ad6a` runbook-only baseline before this stage commit |
| test channel | not applicable |
| trigger event | daemon health test server and `hermeship status` client coverage |
| actual message shape | health summary only; no notification message |
| status | `pass` |
| evidence | `cargo test` passed, including daemon health and CLI status coverage |
| not executed | no manually started long-running daemon in this stage |
| remaining risk | Does not prove operator-specific port/process supervision outside the deterministic test harness |
| rollback | not applicable |

#### Discord Live Verification

| 字段 | 记录 |
| --- | --- |
| 日期 | 2026-06-17 Asia/Shanghai |
| commit | `252ad6a` runbook-only baseline before this stage commit |
| test channel | not provided |
| credentials | Discord credentials not provided |
| trigger event | `send`, `hermes.agent.started` planned only |
| actual message shape | not observed in real Discord |
| status | `not_run` |
| evidence | not run by design in Milestone 9.2 |
| not executed | real Discord custom message and real Discord Hermes agent summary |
| remaining risk | Discord token validity, bot permissions, channel id, rate limits and real delivery remain unverified until Milestone 9.3 |
| rollback | no real Discord state changed |

#### Hermes Gateway Hook Smoke

| 字段 | 记录 |
| --- | --- |
| 日期 | 2026-06-17 Asia/Shanghai |
| commit | `252ad6a` runbook-only baseline before this stage commit |
| test channel | not provided |
| credentials | Hermes gateway test environment not confirmed |
| trigger event | `agent:start` smoke payload documented; real `gateway:startup`, `agent:start`, `agent:end` planned only |
| actual message shape | not observed in real Hermes/Discord |
| status | `not_run` |
| evidence | not run by design in Milestone 9.2 |
| not executed | isolated `HERMES_HOME` install smoke, real Hermes gateway startup, real agent lifecycle trigger |
| remaining risk | Real Hermes hook loading, gateway cache/restart behavior and fail-open behavior in the operator environment remain unverified until Milestone 9.3 |
| rollback | no real hook installed |

#### Rollback Verification

| 字段 | 记录 |
| --- | --- |
| 日期 | 2026-06-17 Asia/Shanghai |
| commit | `252ad6a` runbook-only baseline before this stage commit |
| test channel | not applicable |
| trigger event | rollback commands documented only |
| actual message shape | not applicable |
| status | `not_run` for real rollback; `pass` for documented rollback fields |
| evidence | rollback runbook includes marker, `HOOK.yaml`, `handler.py`, preserved-file and gateway restart checks |
| not executed | real `hermeship hermes uninstall-hooks` against a live Hermes home |
| remaining risk | User-modified hook preservation and actual Hermes gateway reload behavior remain unverified until Milestone 9.3 |
| rollback | commands documented; real rollback not executed because real hook install was not executed |

本轮默认不执行真实 Discord/Hermes live check，原因是 Milestone 9.2 的范围是创建 runbook，且当前未提供真实凭据、测试频道、Hermes gateway 测试环境和显式执行确认。剩余风险进入 Milestone 9.3。
