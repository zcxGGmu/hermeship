# Hermeship

Hermeship is a Hermes-native event-to-channel notification router. It keeps notification delivery outside Hermes gateway sessions so lifecycle events can reach Discord, Slack, webhooks, or other sinks without polluting the agent conversation context.

The project has completed Milestone 8.4. The Rust CLI skeleton, configuration model, repository quality gates, event model, privacy sanitization, daemon HTTP ingress, Hermes hook ingress, router, renderer, dispatcher, fake sink, Discord sink, sink failure handling, local daemon-to-fake-sink smoke coverage, Hermes hook bridge installation, local lifecycle CLI, release preflight, deterministic Git/GitHub/tmux source CLI paths, configured cron run events, and local memory scaffolding are implemented. Live verification, Slack sink, and Hermes plugin/observer work are still pending.

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
hermeship git commit --repo hermeship --branch main --commit <sha> --summary "ship git source"
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
hermeship install
hermeship uninstall
hermeship release preflight <version>
```

## Operations

Local install and rollback commands are deterministic and file-system scoped:

```bash
hermeship install
hermeship setup --default-channel <channel-id> --discord-token-stdin
hermeship hermes install-hooks --scope global --force
hermeship uninstall --remove-state --remove-config --remove-hooks --hermes-home ~/.hermes
hermeship release preflight 0.1.0
```

`hermeship setup --discord-token-stdin` writes the token to local config without putting it in shell history or process argv, and redacts it from command output. Service manager integration is currently template-only: see `deploy/hermeship.service` and `docs/operations.md`.

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
- Milestone 0 through Milestone 8.4 are complete.
- Milestone 1 completed the Rust project skeleton, CLI command tree, configuration model, repository quality gates, and fixture baseline.
- Milestone 2 completed `IncomingEvent`, typed `EventEnvelope`, Hermes canonical mapping, and privacy sanitization.
- Milestone 3 completed daemon `/health`, `/event`, `/api/hermes/hook`, bounded queue ingress, and daemon client POST paths.
- Milestone 4 completed router, default renderer, dispatcher, fake sink, and queue consumer wiring.
- Milestone 5 completed Discord sink payload/request handling, sink failure semantics, and deterministic local daemon-to-fake-sink smoke coverage.
- Milestone 6 completed Hermes gateway hook bridge templates, install/uninstall, safe marker-based rollback, and fail-open handler smoke coverage.
- Milestone 7 completed local install/setup/uninstall lifecycle CLI, service template documentation, and local release preflight.
- Milestone 8.1 completed deterministic Git source CLI events, typed Git event conversion, route metadata, and default rendering.
- Milestone 8.2 completed deterministic GitHub source CLI events, typed GitHub event conversion, route metadata, and default rendering.
- Milestone 8.3 completed deterministic tmux source CLI events, typed tmux event conversion, route metadata, default rendering, and privacy-scoped watch/list reports.
- Milestone 8.4 completed configured cron run events, typed cron event conversion, route metadata, default rendering, local memory init/status scaffold, and public command preflight coverage.

Next implementation phase is Milestone 9: documentation and live verification planning. Keep Slack sink and Hermes plugin/observer out of Milestone 9 unless the checklist is explicitly updated.
