# Hermeship Operations

本文记录 Hermeship 当前已实现的本地运维路径。默认命令只做本地文件和本地 daemon 操作，不会自动运行 `systemctl`、`launchctl` 或外部发布流程。

## Scope

当前运维能力包括：

- `hermeship install`：创建 Hermeship home、config、state、logs、hooks 目录。
- `hermeship setup`：写入 default channel、daemon URL 和 Discord token。
- `hermeship start` / `hermeship status`：启动并检查本地 daemon。
- `hermeship hermes install-hooks` / `hermeship hermes uninstall-hooks`：安装和卸载 Hermes gateway hook bridge。
- `hermeship uninstall`：保留或显式删除本地配置、状态和 hook。
- `hermeship release preflight`：本地发布一致性检查。

当前不自动安装真实 systemd/launchd service，不执行真实 Discord/Hermes live verification，不实现 Slack sink 或 Hermes plugin/observer。

## Install

```bash
hermeship install
```

默认创建：

```text
~/.hermeship/
  .hermeship-managed.json
  config.toml
  hooks/
  logs/
  state/
```

可指定 home：

```bash
hermeship install --home /tmp/hermeship-home
```

可 dry-run：

```bash
hermeship install --dry-run
```

如果 `config.toml` 已存在，默认保留；需要覆盖生成配置时使用：

```bash
hermeship install --force
```

## Setup

推荐通过 stdin 写 Discord token：

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

`--discord-token-stdin` 和 `--discord-token-env` 避免把 token 放进 shell history 或 process argv。命令输出会将 `providers.discord.token` 脱敏。

只更新 default channel 或 daemon URL 也可以：

```bash
hermeship setup --default-channel <discord-channel-id>
hermeship setup --daemon-url http://127.0.0.1:25295
```

dry-run：

```bash
hermeship setup --default-channel <discord-channel-id> --dry-run
```

## Config

```bash
hermeship config path
hermeship config show
hermeship config verify
```

`config show` 默认脱敏 Discord token。

主要环境变量：

- `HERMESHIP_CONFIG`
- `HERMESHIP_DAEMON_URL`
- `HERMESHIP_DISCORD_TOKEN`
- `HERMESHIP_DEFAULT_CHANNEL`
- `HERMESHIP_DRY_RUN`
- `HERMESHIP_HOME`
- `HERMES_HOME`

## Daemon

启动：

```bash
hermeship start
```

指定端口：

```bash
hermeship start --port 25296
```

检查健康：

```bash
hermeship status
```

默认 endpoint：

- `GET http://127.0.0.1:25295/health`
- `POST http://127.0.0.1:25295/event`
- `POST http://127.0.0.1:25295/api/hermes/hook`

`status` 在 daemon 未运行时应返回清晰错误，不 panic。

## Hermes Hooks

安装：

```bash
hermeship hermes install-hooks --scope global --force
```

指定 Hermes home：

```bash
hermeship hermes install-hooks --home ~/.hermes --force
```

dry-run：

```bash
hermeship hermes install-hooks --home /tmp/hermeship-test-home --dry-run
```

安装文件：

```text
~/.hermes/hooks/hermeship/
  HOOK.yaml
  handler.py
  .hermeship-managed.json
```

`HOOK.yaml` 当前默认注册：

- `gateway:startup`
- `session:start`
- `session:end`
- `session:reset`
- `agent:start`
- `agent:end`

`agent:step` 和 `command:*` 当前不默认安装。

卸载：

```bash
hermeship hermes uninstall-hooks --home ~/.hermes
```

Hermeship 只删除 `.hermeship-managed.json` marker 记录且 checksum 未变化的文件。用户修改过的 hook 文件会保留，需要人工检查。

## Smoke Commands

发送 custom message：

```bash
hermeship send --channel <discord-channel-id> --message "hermeship smoke"
```

发送 Hermes canonical event：

```bash
hermeship emit hermes.agent.started --payload '{"session_id":"demo","platform":"telegram"}'
```

解释路由：

```bash
hermeship explain hermes.agent.started --payload '{"session_id":"demo","platform":"telegram"}'
```

模拟 Hermes hook payload：

```bash
printf '%s' '{"event":"agent:start","context":{"session_id":"demo","agent_name":"codex"}}' \
  | hermeship hermes hook --payload -
```

## Source Commands

本地 deterministic source 命令会构造事件并发送到 daemon：

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
hermeship memory init --root /tmp/hermeship-memory --project Hermeship --channel ops --agent codex --date 2026-06-17
hermeship memory status --root /tmp/hermeship-memory --project Hermeship --channel ops --agent codex --date 2026-06-17
```

当前这些 source 不访问真实 GitHub API、不调用真实 `tmux`、不运行真实 scheduler。

## Service Template

Systemd user service 模板位于：

```text
deploy/hermeship.service
```

当前阶段只提交模板，不自动安装。需要手动安装时，可按本机策略复制到 user service 目录后启用。

macOS launchd 可使用同等语义的用户级 plist，关键点是设置 `HERMESHIP_CONFIG` 并运行 `hermeship start`。示例片段：

```xml
<key>ProgramArguments</key>
<array>
  <string>/Users/you/.cargo/bin/hermeship</string>
  <string>start</string>
</array>
<key>EnvironmentVariables</key>
<dict>
  <key>HERMESHIP_CONFIG</key>
  <string>/Users/you/.hermeship/config.toml</string>
</dict>
```

## Update

开发期更新 binary：

```bash
cargo install --path . --force
hermeship release preflight 0.1.0
```

如果 binary 路径变化，重新安装 Hermes hook bridge，让 `handler.py` 中的默认 binary 路径刷新：

```bash
hermeship hermes install-hooks --scope global --force
```

## rollback

只回滚 Hermes hook：

```bash
hermeship hermes uninstall-hooks --home ~/.hermes
```

保留配置和状态的本地卸载：

```bash
hermeship uninstall
```

显式删除本地状态、日志、配置和 Hermeship-managed Hermes hooks：

```bash
hermeship uninstall --remove-state --remove-config --remove-hooks --hermes-home ~/.hermes
```

如果省略 `--hermes-home`，`--remove-hooks` 会使用 `HERMES_HOME` 或 `~/.hermes`。destructive uninstall 要求 Hermeship home 里存在 `.hermeship-managed.json`，避免误删非 Hermeship 目录。

## Release Preflight

```bash
hermeship release preflight 0.1.0
```

Preflight 本地检查：

- Cargo 版本与 `Cargo.lock`。
- 公开 CLI fixture。
- README、方案文档和 operations 中的公开命令。
- Hermes hook 模板。
- fixture policy。
- service 模板。
- live verification 文档状态。

真实 Discord/Hermes live verification 单独记录；缺失时显示 `pending`。

## Troubleshooting

### daemon unavailable

症状：`hermeship status` 或事件发送返回 daemon unavailable。

检查：

```bash
hermeship config show
hermeship start
hermeship status
```

如果使用自定义端口，确认 `daemon.base_url` 或 `HERMESHIP_DAEMON_URL` 与启动端口一致。

### no delivery target

症状：`explain` 显示 `missing delivery target`。

处理：在 route 中配置 `channel` 或 `webhook`，或设置 `[defaults].channel`。

### Discord token missing

症状：Discord sink failure 提示 token/channel 缺失。

处理：

```bash
printf '%s' "$DISCORD_TOKEN" | hermeship setup --discord-token-stdin
hermeship setup --default-channel <discord-channel-id>
```

### hook installed but no events

检查：

```bash
hermeship hermes install-hooks --home ~/.hermes --force
hermeship status
printf '%s' '{"event":"agent:start","context":{"session_id":"demo"}}' | hermeship hermes hook --payload -
```

如果 Hermes gateway 缓存 hook，重启 Hermes gateway 后再触发测试事件。

### hook uninstall preserves files

如果用户修改过 `HOOK.yaml` 或 `handler.py`，Hermeship 会保留文件。人工对比后再删除：

```bash
ls -la ~/.hermes/hooks/hermeship
```
