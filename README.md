# Hermeship

Hermeship is a Hermes-native event-to-channel notification router. It keeps notification delivery outside Hermes gateway sessions so lifecycle events can reach Discord, Slack, webhooks, or other sinks without polluting the agent conversation context.

The project is currently ready for Milestone 2.1 implementation. The Rust CLI skeleton, configuration model, repository quality gates, and fixture directory baseline are implemented; daemon, event pipeline, routing, sinks, and Hermes hook installation are still pending.

## Project Direction

Hermeship follows the architecture and product shape of `/Users/zq/Desktop/ai-projs/posp/template/clawhip`, but it is a native Hermes project:

- Hermeship provides its own daemon, CLI, event model, router, renderer, dispatcher, sink implementations, install flow, and verification tooling.
- `template/clawhip` is an architecture and behavior reference only.
- Hermeship does not call an existing clawhip binary as its delivery path.
- Hermeship does not require a running clawhip daemon.
- Hermeship does not modify Hermes core or inject notification messages back into Hermes conversations.

The intended architecture is daemon-first:

```text
Hermes gateway hooks / Hermes plugins / CLI / git / GitHub / tmux / cron
  -> source ingress
  -> IncomingEvent
  -> typed EventEnvelope
  -> queue
  -> Dispatcher
  -> Router
  -> Renderer
  -> Sink
  -> Discord / Slack / webhook
```

## Implementation Boundary

The primary implementation language is Rust, matching the daemon-first reference shape:

- Rust 2024 for the CLI, daemon, config, event model, routing, rendering, dispatch, sinks, lifecycle, and tests.
- `tokio` for async runtime and queues.
- `axum` for the local daemon HTTP API.
- `clap` for CLI parsing.
- `serde`, `serde_json`, and `toml` for public contracts and configuration.
- `reqwest` for outbound sink delivery.

Python is only planned for the Hermes gateway hook bridge template:

- `~/.hermes/hooks/hermeship/HOOK.yaml`
- `~/.hermes/hooks/hermeship/handler.py`

The hook handler must use Python standard library APIs, forward a compact event envelope to `hermeship hermes hook` or the local daemon, catch all errors, and fail open so Hermes gateway behavior is not blocked by Hermeship.

## Hermes Integration

Hermes gateway hooks are the MVP integration point. The reference Hermes gateway hook system supports:

- `gateway:startup`
- `session:start`
- `session:end`
- `session:reset`
- `agent:start`
- `agent:step`
- `agent:end`
- `command:*`

Hermeship will normalize those events into canonical Hermeship events such as:

- `hermes.gateway.started`
- `hermes.session.started`
- `hermes.session.finished`
- `hermes.session.reset`
- `hermes.agent.started`
- `hermes.agent.step`
- `hermes.agent.finished`
- `hermes.agent.failed`

Hermes plugin/observer integration is a later phase after the gateway hook bridge is implemented and verified.

## Privacy Defaults

Hermeship should route event summaries and structured metadata, not full conversations. Defaults must avoid sending:

- full prompts or conversations
- provider request or response bodies
- tool result bodies
- tokens, cookies, secrets, API keys, or authorization headers

Message and response excerpts must be explicit opt-in, sanitized, and bounded.

## Planned CLI Shape

The public command surface will be implemented incrementally:

```bash
hermeship start
hermeship status
hermeship setup
hermeship config show
hermeship config path
hermeship config verify
hermeship send --channel <id> --message "hello"
hermeship emit hermes.agent.started --payload '{"session_id":"demo"}'
hermeship explain hermes.agent.started --payload '{"session_id":"demo"}'
hermeship hermes hook --provider gateway --payload '{"event":"agent:start"}'
hermeship hermes install-hooks --home ~/.hermes --force
hermeship install
hermeship uninstall
hermeship release preflight <version>
```

## Verification Policy

Default tests must be local and deterministic. They must not depend on:

- real Discord tokens
- a real Hermes gateway
- real GitHub state
- real tmux sessions
- external network availability

Live verification is separate from default test runs and will be recorded in `docs/live-verification.md` when implemented.

## Development Quality Gates

Before each stage commit, run the baseline Rust quality gate:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

`cargo fmt` is the formatting authority. Code should be committed only after it matches rustfmt output.

`cargo clippy --all-targets -- -D warnings` is the lint gate for application code, tests, examples, and benches. New warnings should be fixed instead of allowed unless a later design document records a narrow exception.

Default tests must stay deterministic and must not require external credentials, a real Hermes gateway, real Discord, real GitHub, real tmux, or non-local network state. Use local fixtures, fake sinks, fake HTTP servers, fake Hermes homes, and fake binaries for regression coverage.

## Development Status

Current state:

- Architecture and test strategy are documented in `docs/plans/2026-06-15-hermeship-development-plan.md`.
- Execution progress is tracked in `tasks/development-checklist.md`.
- Session handoff status is tracked in `docs/development-status.md`.
- Milestone 0 establishes the repository baseline and project positioning.
- Milestone 1.1 created the Rust 2024 Cargo project and minimal CLI command tree.
- Milestone 1.2 implemented the configuration model and real `config path/show/verify` logic.
- Milestone 1.3 completed repository quality gates, fixture directories, and rustfmt/clippy documentation.

Next implementation phase is Milestone 2.1: `IncomingEvent`, message format handling, `emit` parsing, and initial Hermes fixture payloads.
