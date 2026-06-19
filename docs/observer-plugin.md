# Hermeship Observer Plugin Contract

本文记录 Milestone 10.1 的 Hermes plugin / observer 研究结论。当前阶段只定义契约、安装启用方式、事件映射、隐私边界和验证策略；不实现 plugin 模板，不新增 Hermeship canonical event 代码。

## Decision

- Milestone 9.3 真实 Discord/Hermes live pass 已被用户豁免，用于解除 Milestone 10 门禁。
- 该豁免不代表真实 live verification 已通过；`docs/live-verification.md` 的 `blocked`/`not_run` 记录仍然成立。
- Milestone 10.1 先做 observer 契约研究；Milestone 10.2 再实现可选 Hermes observer plugin scaffold。
- Slack sink 仍不在当前默认范围内。

## Hermes Plugin Surface

Hermes general plugins由 `hermes_cli/plugins.py` 管理。

Discovery sources:

1. Bundled plugins: `<repo>/plugins/<name>/` and selected nested plugin categories.
2. User plugins: `$HERMES_HOME/plugins/<name>/`, normally `~/.hermes/plugins/<name>/`.
3. Project plugins: `./.hermes/plugins/<name>/`, only when `HERMES_ENABLE_PROJECT_PLUGINS` is truthy.
4. Pip entry points: `hermes_agent.plugins`.

Directory plugin requirements:

- `plugin.yaml` or `plugin.yml`.
- `__init__.py`.
- `register(ctx)` function.
- Observer hooks are registered with `ctx.register_hook("<hook>", callback)`.

Activation:

- Standalone user and entry-point plugins are opt-in through `plugins.enabled` in Hermes `config.yaml`.
- `plugins.disabled` wins even when the plugin is listed in `plugins.enabled`.
- `hermes plugins enable <name>` resolves nested plugin keys and writes the canonical key into `plugins.enabled`.
- Project plugins are disabled unless `HERMES_ENABLE_PROJECT_PLUGINS=1`.

Fail-open:

- `PluginManager.invoke_hook()` wraps each callback in `try/except`.
- Non-`None` callback return values are collected, but observer callbacks should return `None`.
- A failing observer callback must not break Hermes runtime.

## Observer Hooks In Scope

The first Hermeship observer plugin should register only observational hooks. It must not register middleware and must not return behavior-changing directives.

### Session Hooks

| Hermes hook | Hermeship event | Required safe fields |
| --- | --- | --- |
| `on_session_start` | `hermes.observer.session.started` | `session_id`, `platform`, `model` |
| `on_session_end` | `hermes.observer.session.ended` | `session_id`, `platform`, `model`, `completed`, `interrupted`, `reason`, `task_id`, `turn_id`, `api_request_id` |
| `on_session_finalize` | `hermes.observer.session.finalized` | `session_id`, `platform`, `reason` |
| `on_session_reset` | `hermes.observer.session.reset` | `session_id`, `platform`, `reason` when present |

Notes:

- Existing gateway hook events already map `session:start`, `session:end`, and `session:reset` into canonical Hermes session events.
- Observer session events should use the `hermes.observer.*` namespace to avoid claiming parity with gateway lifecycle events.

### LLM / API Hooks

| Hermes hook | Hermeship event | Required safe fields |
| --- | --- | --- |
| `pre_api_request` | `hermes.observer.api.request.started` | `session_id`, `task_id`, `turn_id`, `api_request_id`, `platform`, `model`, `provider`, `api_mode`, `api_call_count`, `message_count`, `tool_count`, `approx_input_tokens`, `request_char_count`, `max_tokens` |
| `post_api_request` | `hermes.observer.api.request.finished` | `session_id`, `task_id`, `turn_id`, `api_request_id`, `platform`, `model`, `provider`, `api_mode`, `api_call_count`, `api_duration`, `finish_reason`, `message_count`, `response_model`, `assistant_content_chars`, `assistant_tool_call_count`, safe token usage summary |
| `api_request_error` | `hermes.observer.api.request.failed` | `session_id`, `task_id`, `turn_id`, `api_request_id`, `platform`, `model`, `provider`, `api_mode`, `api_call_count`, `error_type`, `error_message`, `duration_ms` when present |
| `pre_llm_call` | `hermes.observer.llm.started` | `session_id`, `platform`, `model`, `is_first_turn`, `message_chars`, `history_count` |
| `post_llm_call` | `hermes.observer.llm.finished` | `session_id`, `platform`, `model`, `response_chars` when present |

Privacy requirements:

- Do not forward `user_message`.
- Do not forward `conversation_history`.
- Do not forward `request_messages`.
- Do not forward raw `request`, raw `response`, assistant message objects, provider request bodies, provider response bodies, tool result bodies, or full prompts.
- Only forward counts, booleans, timings, provider/model names and bounded error summaries.

### Tool Hooks

| Hermes hook | Hermeship event | Required safe fields |
| --- | --- | --- |
| `pre_tool_call` | `hermes.observer.tool.started` | `session_id`, `task_id`, `turn_id`, `api_request_id`, `tool_call_id`, `tool_name`, `arg_keys`, `arg_chars` |
| `post_tool_call` | `hermes.observer.tool.finished` | `session_id`, `task_id`, `turn_id`, `api_request_id`, `tool_call_id`, `tool_name`, `status`, `duration_ms`, `result_chars`, `error_type`, `error_message` |

Behavior requirements:

- The Hermeship observer plugin must not return `{"action": "block"}` from `pre_tool_call`.
- It must not register `transform_tool_result`.
- It must not forward raw args, command strings, tool outputs or result JSON.
- `arg_keys` may list top-level argument names; `arg_chars` may record serialized size.

### Approval Hooks

| Hermes hook | Hermeship event | Required safe fields |
| --- | --- | --- |
| `pre_approval_request` | `hermes.observer.approval.requested` | `session_key`, `surface`, `pattern_key`, `pattern_keys`, `description_chars`, `command_chars`, `turn_id`, `tool_call_id` |
| `post_approval_response` | `hermes.observer.approval.responded` | same fields plus `choice` |

Privacy requirements:

- Do not forward full shell command text.
- Do not forward full approval description.
- Keep only lengths and policy keys.

### Subagent Hooks

| Hermes hook | Hermeship event | Required safe fields |
| --- | --- | --- |
| `subagent_start` | `hermes.observer.subagent.started` | `parent_session_id`, `parent_turn_id`, `parent_subagent_id`, `child_session_id`, `child_subagent_id`, `child_role`, `child_goal_chars` |
| `subagent_stop` | `hermes.observer.subagent.finished` | `parent_session_id`, `parent_turn_id`, `child_session_id`, `child_role`, `child_status`, `child_summary_chars`, `duration_ms` |

Privacy requirements:

- Do not forward full child goal or full child summary.
- Keep status, role and length metadata.

## Hermeship Ingress Mapping

The plugin should submit generic `IncomingEvent` to Hermeship daemon `POST /event`, not `POST /api/hermes/hook`.

Recommended payload shape:

```json
{
  "type": "hermes.observer.tool.finished",
  "payload": {
    "provider": "hermes",
    "source": "plugin",
    "observer_schema_version": 1,
    "session_id": "synthetic-session",
    "tool_name": "terminal",
    "status": "ok",
    "duration_ms": 42,
    "result_chars": 128
  }
}
```

Rationale:

- `/api/hermes/hook` is gateway-hook-specific and normalizes Hermes gateway envelopes.
- Observer plugin payloads are already Hermeship-shaped events.
- Unknown `hermes.observer.*` events currently degrade to `Custom`, preserving sanitized payload while avoiding premature Rust event model expansion.

## Rendering And Routing Strategy

Milestone 10.1 should not add typed Rust observer bodies. For Milestone 10.2 MVP:

- Use `hermes.observer.*` event names with `Custom` body fallback.
- Route examples can match `hermes.observer.tool.*`, `hermes.observer.api.*`, `hermes.observer.subagent.*`.
- Default renderer will render custom events generically until typed renderer support is added.
- A later milestone may add `EventBody::HermesObserver*` variants if real usage shows stable fields and routing needs.

This keeps the first observer plugin low-risk and avoids expanding Rust canonical contracts before the plugin payload shape is proven.

## Plugin Configuration

The plugin template should use environment variables rather than hard-coded paths:

- `HERMESHIP_DAEMON_URL`: default `http://127.0.0.1:25295`.
- `HERMESHIP_OBSERVER_TIMEOUT_SECS`: default `2`.
- `HERMESHIP_OBSERVER_DISABLED`: truthy value disables delivery.

The plugin should be installed as:

```text
~/.hermes/plugins/hermeship-observer/
  plugin.yaml
  __init__.py
```

The operator must enable it with:

```bash
hermes plugins enable hermeship-observer
```

If Hermes resolves nested keys differently in a future template layout, the enable command must use the canonical key reported by `hermes plugins list`.

## Failure Behavior

The observer plugin must be fail-open:

- If Hermeship daemon is unavailable, hook callbacks return without raising.
- If payload serialization fails, callbacks return without raising.
- If HTTP delivery times out, callbacks return without raising.
- The plugin must not write secrets, raw prompts, raw messages, raw requests or raw responses to stdout/stderr.
- Diagnostics, if any, must be short and secret-free.

## Verification Strategy

Milestone 10.1 documentation verification:

```bash
rg -n "hermes.observer|plugins.enabled|pre_tool_call|post_tool_call|api_request_error|subagent_start|fail-open|request body|response body" docs/observer-plugin.md
cargo test release_preflight
cargo run -- release preflight 0.1.0
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Milestone 10.2 plugin scaffold verification should add:

```bash
python3 -m py_compile templates/hermes-plugin/__init__.py
```

and a local smoke test that imports the template with a fake HTTP client or monkeypatches the send function so hooks can be invoked without real Hermes, Discord, network or credentials.

Milestone 10.3 install/enable CLI verification adds:

```bash
cargo test observer_plugin
cargo test cli
```

`hermeship hermes install-plugin` installs the template into `$HERMES_HOME/plugins/hermeship-observer/` and writes a Hermeship marker. The installer rejects symlinked plugin directories, template files and marker files before writing. `hermeship hermes enable-plugin` only prints manual enable instructions; it does not call `hermes`, edit `config.yaml`, or auto-enable the plugin.

## Open Follow-ups After Milestone 10.3

- Decided for 10.2: `templates/hermes-plugin/__init__.py` POSTs directly with Python standard library `urllib.request` to keep Hermes plugin delivery independent from the Hermeship CLI binary path.
- Decided for 10.3: `hermeship hermes install-plugin` installs the optional observer plugin template, while `hermeship hermes enable-plugin` remains instruction-only to preserve explicit operator opt-in.
- Decide whether typed Rust observer bodies are needed after plugin payloads stabilize.
- Decided for 10.2: release preflight requires observer plugin template files and contract keywords once the scaffold lands.
