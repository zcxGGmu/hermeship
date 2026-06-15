# Hermeship

Hermeship is a Hermes-native event-to-channel notification router. It keeps notification delivery outside Hermes gateway sessions so lifecycle events can reach Discord, Slack, webhooks, or other sinks without polluting the agent conversation context.

The project is currently in contract and repository-baseline work. Rust implementation has not started yet.

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

## Development Status

Current state:

- Architecture and test strategy are documented in `docs/plans/2026-06-15-hermeship-development-plan.md`.
- Execution progress is tracked in `tasks/development-checklist.md`.
- Session handoff status is tracked in `docs/development-status.md`.
- Milestone 0 establishes the repository baseline and project positioning.
- Rust implementation files are not created yet.

Next implementation phase after Milestone 0 is the Rust CLI and daemon skeleton.
