# Hermeship

<p align="center">
  <img src="docs/assets/branding/hermeship-icon.png" alt="Hermeship project icon" width="180">
</p>

<p align="right">
  <a href="./README.md"><img alt="中文" src="https://img.shields.io/badge/%E8%AF%AD%E8%A8%80-%E4%B8%AD%E6%96%87-d97757?style=for-the-badge"></a>
  <a href="./README.en.md"><img alt="English" src="https://img.shields.io/badge/Language-English-8c6f5a?style=for-the-badge"></a>
</p>

Hermeship is an independent, Hermes-native, daemon-first event notification router. It owns its Hermes event contracts, Rust daemon, routing, rendering, delivery runtime, and release verification flow.

## What Hermeship Is

Hermeship receives events from Hermes gateway hooks, an optional Hermes observer plugin, CLI commands, and local deterministic source commands. It normalizes those events into typed envelopes, sanitizes payloads, routes deliveries, renders safe summaries, and sends them through sinks such as Discord.

Operational boundaries:

- It does not modify Hermes core.
- It does not write notification messages back into Hermes conversations.
- It does not auto-enable the observer plugin.
- Default tests and source commands use local deterministic paths; real Discord/Hermes verification is tracked separately.

## Diagrams

![Hermeship architecture](docs/assets/diagrams/hermeship-architecture.png)

![Hermeship event flow](docs/assets/diagrams/hermeship-event-flow.png)

![Hermes observer framework](docs/assets/diagrams/hermeship-observer-framework.png)

Diagram sources live in `docs/assets/diagrams/*.json`; each diagram is exported as `.svg` and `.png` with `fireworks-tech-graph` Style 6, Claude Official.

## Current Capability Boundary

Implemented:

- Rust CLI, config model, install/setup/uninstall lifecycle, and release preflight.
- daemon endpoints: `GET /health`, `POST /event`, `POST /api/hermes/hook`.
- typed `IncomingEvent -> EventEnvelope` conversion.
- privacy sanitizer, bounded queue, dispatcher, multi-delivery router, renderer, Discord sink, and fake sink.
- Hermes gateway hook bridge install/uninstall.
- optional Hermes observer plugin template plus install/enable guidance.
- typed Rust observer body for `hermes.observer.*`.
- local deterministic Git/GitHub/tmux/cron source commands.
- local filesystem memory scaffold.

Not implemented or not enabled by default:

- Real Discord/Hermes live verification has not passed yet.
- `release preflight` only checks that live verification record fields exist; it does not prove real live delivery.
- Slack sink is not part of the default scope.
- Real GitHub API polling, real tmux watching, real scheduling, and automatic service-manager installation are not implemented.
- The observer plugin is installed only on explicit command and still requires manual enablement in Hermes.

## Architecture

The runtime pipeline is:

```text
Hermes gateway hooks / optional observer plugin / CLI / local source commands
  -> daemon ingress
  -> privacy sanitizer
  -> typed EventEnvelope
  -> bounded queue
  -> Dispatcher
  -> Router
  -> Renderer
  -> Sink
  -> Discord
```

See `ARCHITECTURE.md` for module boundaries.

## Install And Configure

```bash
cargo install --path .
hermeship install
```

Configure Discord without putting the token in shell history:

```bash
printf '%s' "$DISCORD_TOKEN" | hermeship setup \
  --discord-token-stdin \
  --default-channel <discord-channel-id> \
  --daemon-url http://127.0.0.1:25295
```

You can also read the token from an environment variable:

```bash
hermeship setup --discord-token-env HERMESHIP_SETUP_DISCORD_TOKEN
```

Inspect configuration:

```bash
hermeship config path
hermeship config show
hermeship config verify
```

Start and check the daemon:

```bash
hermeship start
hermeship status
```

Default daemon endpoint:

```text
http://127.0.0.1:25295
```

Public HTTP API:

| Method | Path | Responsibility |
| --- | --- | --- |
| `GET` | `/health` | Return daemon, queue, and configured sink health |
| `POST` | `/event` | Accept generic `IncomingEvent` JSON |
| `POST` | `/api/hermes/hook` | Accept Hermes gateway hook envelopes |

## Hermes Integration

Install the gateway hook bridge:

```bash
hermeship hermes install-hooks --scope global --force
```

Uninstall it safely:

```bash
hermeship hermes uninstall-hooks --home ~/.hermes
```

Install the optional observer plugin template:

```bash
hermeship hermes install-plugin --home ~/.hermes --force
hermeship hermes enable-plugin --home ~/.hermes --dry-run
```

Then enable it manually from Hermes:

```bash
hermes plugins enable hermeship-observer
```

The hook bridge and observer plugin are fail-open. Hermeship failures should not stop Hermes gateway or agent execution.

## Sending Events

```bash
hermeship send --channel <discord-channel-id> --message "hermeship smoke"
hermeship emit hermes.agent.started --payload '{"session_id":"demo","platform":"telegram","project":"Hermeship"}'
hermeship explain hermes.agent.started --payload '{"session_id":"demo","platform":"telegram"}'
```

Simulate a Hermes hook payload:

```bash
printf '%s' '{"event":"agent:start","context":{"session_id":"demo","agent_name":"codex"}}' \
  | hermeship hermes hook --payload -
```

Local deterministic source commands include:

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

These source commands do not currently poll real GitHub, read real tmux sessions, or run a real scheduler.

## Routing, Rendering, And Privacy

Router behavior:

- event glob matching supports exact kinds and `*` patterns.
- one event can resolve to 0..N deliveries.
- route filters use structured metadata and selected typed body fields, not rendered text.
- unsupported sinks, missing targets, and disabled routes produce diagnostics.

Supported formats:

- `compact`
- `inline`
- `alert`
- `raw`

Hermeship routes summaries and structured metadata, not full conversations. Tokens, cookies, secrets, full prompts, full conversations, provider request/response bodies, and tool result bodies must not appear in fixtures, logs, live records, or docs.

`raw` rendering is still safe JSON: it emits typed controlled fields and sanitized payload summaries, not arbitrary original payload.

## Rollback

Rollback only the Hermes hook:

```bash
hermeship hermes uninstall-hooks --home ~/.hermes
```

Uninstall while preserving local config and state:

```bash
hermeship uninstall
```

Explicitly remove local state, logs, config, and Hermeship-managed hooks:

```bash
hermeship uninstall --remove-state --remove-config --remove-hooks --hermes-home ~/.hermes
```

## Live Verification

Live verification is separate from default local tests. Real Discord/Hermes checks require Discord credentials, a test channel, a Hermes gateway test environment, explicit execution confirmation, and a rollback window.

Real Discord/Hermes live verification has not passed yet. Existing `blocked` / `not_run` records live in `docs/live-verification.md`.

## Release Preflight And Development Gates

```bash
hermeship release preflight 0.1.0
```

Run local gates before a stage commit:

```bash
python3 -m py_compile templates/hermes-plugin/__init__.py
cargo test observer_plugin
cargo test release_preflight
cargo run -- release preflight 0.1.0
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Default tests must stay local and deterministic. Do not run the real Discord/Hermes live check unless credentials, a test channel, a Hermes gateway test environment, and explicit execution confirmation are available.
