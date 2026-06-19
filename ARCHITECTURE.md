# Hermeship Architecture

Hermeship is a Hermes-native daemon-first event router. It mirrors the broad shape of `template/clawhip`, but owns its own runtime and contracts for Hermes.

## Goals

- Receive Hermes lifecycle events without modifying Hermes core.
- Normalize CLI, Hermes hook and local deterministic source events into one typed event model.
- Route one event to 0..N deliveries using structured metadata filters.
- Render safe notification summaries.
- Deliver to Discord in the MVP runtime.
- Keep default tests local, deterministic and credential-free.

## Non-Goals

- No runtime dependency on `clawhip`.
- No call-through to a running `clawhip` daemon.
- No notification messages injected into Hermes conversations.
- No Slack sink in the current milestone.
- No automatic Hermes observer plugin enablement; Hermeship can install the optional template, but operators still enable it explicitly in Hermes.
- No real GitHub polling, real tmux watch, real scheduler or service-manager automation in the completed milestones.

## Data Flow

```text
Hermes gateway hook
Hermes observer plugin (optional template)
CLI send/emit
local git/GitHub/tmux/cron commands
        |
        v
IncomingEvent / HermesHookEnvelope
        |
        v
privacy::sanitize_payload
        |
        v
event::compat::from_incoming_event
        |
        v
EventEnvelope
        |
        v
bounded tokio mpsc queue
        |
        v
Dispatcher
        |
        +--> Router
        |      - event glob matching
        |      - metadata filters
        |      - delivery target resolution
        |
        +--> DefaultRenderer
        |      - compact / inline / alert / raw
        |      - safe template tokens
        |
        +--> Sink registry
               - Discord sink
               - FakeSink for tests
```

## Process Model

Hermeship runs as a local daemon:

```bash
hermeship start
```

Default daemon endpoint:

```text
http://127.0.0.1:25295
```

HTTP API:

| Method | Path | Responsibility |
| --- | --- | --- |
| `GET` | `/health` | Return version, status, queue health and configured sinks |
| `POST` | `/event` | Accept generic `IncomingEvent` JSON |
| `POST` | `/api/hermes/hook` | Accept Hermes hook envelope |

CLI commands such as `send`, `emit`, `hermes hook`, `git`, `github`, `tmux` and `cron` submit events to the daemon. `explain` is local-only and does not enqueue or deliver.

## Module Boundaries

| Module | Responsibility |
| --- | --- |
| `src/main.rs` | CLI command execution and top-level orchestration |
| `src/cli.rs` | `clap` command tree and argument parsing |
| `src/config.rs` | `AppConfig`, route schema, env overrides, config serialization |
| `src/client.rs` | Local daemon HTTP client |
| `src/daemon.rs` | Axum server, HTTP ingress, health endpoint, bounded queue |
| `src/events.rs` | External `IncomingEvent` and routing metadata helpers |
| `src/hermes.rs` | Hermes hook envelope normalization |
| `src/event/` | Typed `EventEnvelope`, body variants and compat mapping |
| `src/privacy.rs` | Recursive payload sanitizer and excerpt policy |
| `src/router.rs` | Route matching, metadata filters, diagnostics and delivery resolution |
| `src/render/` | Default renderer and output formats |
| `src/dispatch.rs` | Queue consumer and route-render-sink pipeline |
| `src/sink/` | Sink trait, fake sink and Discord sink |
| `src/hooks.rs` | Hermes hook bridge install/uninstall |
| `src/observer_plugin.rs` | Optional Hermes observer plugin template install and enable guidance |
| `src/lifecycle.rs` | Local install/setup/uninstall |
| `src/release_preflight.rs` | Release consistency checks |
| `src/source/` | Git/GitHub/tmux deterministic source builders |
| `src/cron.rs` | Configured cron run event builder |
| `src/memory.rs` | Local filesystem memory scaffold |
| `templates/hermes-plugin/` | Optional Hermes observer plugin scaffold |

## Event Model

External input starts as `IncomingEvent`:

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

Internal runtime uses typed `EventEnvelope`:

```rust
pub struct EventEnvelope {
    pub id: Uuid,
    pub timestamp: OffsetDateTime,
    pub source: String,
    pub body: EventBody,
    pub metadata: EventMetadata,
}
```

Current canonical body families:

- Hermes: `hermes.gateway.started`, `hermes.session.started`, `hermes.session.finished`, `hermes.session.reset`, `hermes.agent.started`, `hermes.agent.step`, `hermes.agent.finished`, `hermes.agent.failed`
- Git: `git.commit`, `git.branch-changed`
- GitHub: `github.issue-opened`, `github.pr-opened`, `github.check-failed`, `github.release-published`
- tmux: `tmux.keyword`, `tmux.stale`
- cron: `cron.run`
- custom: unknown kinds and `hermeship send`

See `docs/hermes-event-contract.md` for field-level rules.

## Routing

Routes live in config:

```toml
[[routes]]
event = "hermes.agent.*"
filter = { platform = "telegram", project = "Hermeship" }
sink = "discord"
channel = "123456789012345678"
format = "compact"
```

Router behavior:

- event glob matching supports exact kind and `*` patterns;
- one event can match multiple routes;
- filters use structured metadata and selected typed body fields;
- unsupported sinks are skipped with diagnostics;
- missing target is reported as a skipped route;
- `DiscordWebhook` diagnostics redact the webhook URL.

Target resolution order:

1. route webhook;
2. route channel;
3. event channel hint;
4. `[defaults].channel`;
5. missing delivery target.

Format resolution order:

1. event format hint;
2. route format;
3. `[defaults].format`.

## Rendering

`DefaultRenderer` is pure and deterministic. It does not read files, call network APIs or inspect Hermes state.

Formats:

- `compact`: concise default summary.
- `inline`: canonical kind plus pipe-delimited fields.
- `alert`: `ALERT: ` plus compact summary.
- `raw`: safe JSON for diagnostics.

Templates support only approved tokens:

- `{event}`
- `{canonical_kind}`
- `{source}`
- `{provider}`
- `{platform}`
- `{session_id}`
- `{agent_name}`
- `{project}`
- `{channel}`

Unrecognized tokens remain literal.

## Sinks

Current production sink:

- Discord bot token + channel.
- Discord webhook.

Testing sink:

- `FakeSink`, which records target, format, rendered content, event kind and route index.

Failure behavior:

- one delivery failure does not block other deliveries;
- missing token/channel returns a sink failure report, not panic;
- non-2xx Discord responses include HTTP status and bounded body tail;
- Discord 429 diagnostics include `retry_after` when available.

## Hermes Hook Bridge

Installed files:

```text
~/.hermes/hooks/hermeship/
  HOOK.yaml
  handler.py
  .hermeship-managed.json
```

`handler.py`:

- uses only Python standard library;
- serializes `event_type` and `context` into compact JSON;
- calls `hermeship hermes hook --payload -`;
- supports `HERMESHIP_BIN` override;
- has a default 2 second timeout;
- catches all exceptions and returns `None`.

The bridge is fail-open by design. Hermeship failures must not stop Hermes gateway execution.

## Hermes Observer Plugin

Milestone 10.2 adds an optional Hermes directory plugin scaffold:

```text
~/.hermes/plugins/hermeship-observer/
  plugin.yaml
  __init__.py
```

The repository template lives in `templates/hermes-plugin/`. Hermeship can install it locally with:

```bash
hermeship hermes install-plugin --home ~/.hermes --force
hermeship hermes enable-plugin --home ~/.hermes --dry-run
```

The installer validates each path with `symlink_metadata`, rejects symlinked plugin directories/files/markers, and only overwrites regular files.

`enable-plugin` only prints operator instructions. Operators still enable it manually with:

```bash
hermes plugins enable hermeship-observer
```

The plugin registers observer hooks only and posts generic `IncomingEvent` payloads to `POST /event`. It does not use `/api/hermes/hook`, because that endpoint is specific to gateway hook envelopes.

Observer events use the `hermes.observer.*` namespace and currently fall through the existing `Custom` event body path. This avoids expanding typed Rust event bodies before real observer usage stabilizes.

The plugin forwards only safe fields, counts, lengths, statuses and bounded summaries. It must not forward raw prompts, conversation history, request/response bodies, shell commands, tool result bodies, child goals or child summaries. Every callback returns `None`; daemon failures, serialization failures and HTTP timeouts fail open.

## Privacy

Privacy protection is part of ingress and rendering:

- daemon sanitizes payload before enqueue;
- sensitive keys are redacted recursively;
- full message/response/conversation/provider/tool bodies are removed by default;
- safe summaries such as `message_chars`, `response_chars`, `has_message`, `has_response` are preserved;
- opt-in excerpts are sanitized first and length-bounded;
- raw rendering emits typed controlled JSON, not arbitrary original payloads.

## Install And rollback

Local install is filesystem-scoped:

```bash
hermeship install
hermeship setup --default-channel <channel-id>
hermeship hermes install-hooks --scope global --force
hermeship start
```

rollback paths:

```bash
hermeship hermes uninstall-hooks --home ~/.hermes
hermeship uninstall
hermeship uninstall --remove-state --remove-config --remove-hooks --hermes-home ~/.hermes
```

Destructive uninstall requires explicit flags and a Hermeship-managed home marker.

## Verification

Default verification is local and deterministic:

```bash
cargo test release_preflight
cargo run -- release preflight 0.1.0
python3 -m py_compile templates/hermes-plugin/__init__.py
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Default tests rely on local fixtures, fake sink, fake HTTP, fake Hermes home and fake binaries. They must not require real Discord credentials, real Hermes gateway, real GitHub state, real tmux sessions or external network state.

Live Discord/Hermes checks are separate and recorded in `docs/live-verification.md`.
