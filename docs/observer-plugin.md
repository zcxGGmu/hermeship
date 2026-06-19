# Hermeship Observer Plugin Contract

本文记录 Hermes plugin / observer 的契约、安装启用方式、事件映射、隐私边界和验证策略。Milestone 10.1 先完成研究，Milestone 10.2 增加可选 plugin scaffold，Milestone 10.3 增加 install/enable CLI；本轮后续开发已为 `hermes.observer.*` 增加 Rust typed observer body。

## Decision

- Milestone 9.3 真实 Discord/Hermes live pass 已被用户豁免，用于解除 Milestone 10 门禁。
- 该豁免不代表真实 live verification 已通过；`docs/live-verification.md` 的 `blocked`/`not_run` 记录仍然成立。
- Milestone 10.1 先做 observer 契约研究；Milestone 10.2 再实现可选 Hermes observer plugin scaffold。
- Milestone 10 后续已新增 typed Rust observer body；`hermes.observer.*` 不再依赖 `Custom` fallback。
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
| `on_session_end` | `hermes.observer.session.ended` | `session_id`, `platform`, `model`, `completed`, `interrupted`, `reason_chars`, `has_reason`, `task_id`, `turn_id`, `api_request_id` |
| `on_session_finalize` | `hermes.observer.session.finalized` | `session_id`, `platform`, `reason_chars`, `has_reason` |
| `on_session_reset` | `hermes.observer.session.reset` | `session_id`, `platform`, `reason_chars`, `has_reason` when present |

Notes:

- Existing gateway hook events already map `session:start`, `session:end`, and `session:reset` into canonical Hermes session events.
- Observer session events should use the `hermes.observer.*` namespace to avoid claiming parity with gateway lifecycle events.

### LLM / API Hooks

| Hermes hook | Hermeship event | Required safe fields |
| --- | --- | --- |
| `pre_api_request` | `hermes.observer.api.request.started` | `session_id`, `task_id`, `turn_id`, `api_request_id`, `platform`, `model`, `provider`, `api_mode`, `api_call_count`, `message_count`, `tool_count`, `approx_input_tokens`, `request_char_count`, `max_tokens` |
| `post_api_request` | `hermes.observer.api.request.finished` | `session_id`, `task_id`, `turn_id`, `api_request_id`, `platform`, `model`, `provider`, `api_mode`, `api_call_count`, `api_duration`, `finish_reason`, `message_count`, `response_model`, `assistant_content_chars`, `assistant_tool_call_count`, safe token usage summary |
| `api_request_error` | `hermes.observer.api.request.failed` | `session_id`, `task_id`, `turn_id`, `api_request_id`, `platform`, `model`, API `provider`, `api_mode`, `api_call_count`, bounded exception-class `error_type` or `error_type_chars`, `has_error_type`, `error_message_chars`, `has_error_message`, `duration_ms` when present |
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
| `post_tool_call` | `hermes.observer.tool.finished` | `session_id`, `task_id`, `turn_id`, `api_request_id`, `tool_call_id`, `tool_name`, canonical `status` or `status_chars`, `duration_ms`, `result_chars`, bounded exception-class `error_type` or `error_type_chars`, `has_error_type`, `error_message_chars`, `has_error_message` |

Behavior requirements:

- The Hermeship observer plugin must not return `{"action": "block"}` from `pre_tool_call`.
- It must not register `transform_tool_result`.
- It must not forward raw args, command strings, tool outputs or result JSON.
- `arg_keys` may list top-level argument names; `arg_chars` may record serialized size.

### Approval Hooks

| Hermes hook | Hermeship event | Required safe fields |
| --- | --- | --- |
| `pre_approval_request` | `hermes.observer.approval.requested` | `session_key_chars`, `has_session_key`, `surface`, `pattern_key`, `pattern_keys`, `description_chars`, `command_chars`, `turn_id`, `tool_call_id` |
| `post_approval_response` | `hermes.observer.approval.responded` | same fields plus `choice` |

Privacy requirements:

- Do not forward full shell command text.
- Do not forward full approval description.
- Do not forward raw `session_key`; keep only length and presence metadata.
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
- `hermes.observer.*` events now enter a typed Rust observer body, preserving the canonical kind while storing only allowlisted safe fields.

## Rendering And Routing Strategy

Milestone 10.1 intentionally did not add typed Rust observer bodies. Milestone 10.2 first shipped the plugin scaffold with `Custom` fallback. After Milestone 10.3, Hermeship now has a typed observer body:

- Use `hermes.observer.*` event names with `EventBody::HermesObserver`.
- Keep canonical kind unchanged, for example `hermes.observer.tool.finished`.
- Derive `observer_category` and `observer_action` from the namespace, for example `tool` and `finished`, or `api` and `request.failed`.
- Route examples can match `hermes.observer.tool.*`, `hermes.observer.api.*`, `hermes.observer.subagent.*`.
- Router filters can match `observer_category`, `observer_action`, `tool_name`, canonical `status`, `model`, `api_mode`, `child_role`, canonical `child_status` and other typed safe observer fields.
- Observer body fields that share names with core metadata are also exposed as `observer_<field>` route keys, for example `observer_provider` and `observer_session_id`; the core metadata keys keep metadata priority.
- For `hermes.observer.*`, core metadata `provider` remains `hermes`; API provider names from observer hooks are body fields and can be matched as `observer_provider`.
- Default renderer emits compact/inline observer summaries and raw JSON from typed allowlisted fields only.

The typed body remains intentionally generic instead of adding one Rust variant per hook. This keeps the Rust contract stable while making observer events routeable and renderable.

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

Typed observer body verification adds:

```bash
cargo test observer
```

This covers observer body conversion, route filters, compact/inline/raw rendering, and the existing Python plugin smoke checks.

## Open Follow-ups After Milestone 10.3

- Decided for 10.2: `templates/hermes-plugin/__init__.py` POSTs directly with Python standard library `urllib.request` to keep Hermes plugin delivery independent from the Hermeship CLI binary path.
- Decided for 10.3: `hermeship hermes install-plugin` installs the optional observer plugin template, while `hermeship hermes enable-plugin` remains instruction-only to preserve explicit operator opt-in.
- Decided after 10.3: `hermes.observer.*` uses a typed Rust observer body with generic safe fields, structured route filters and safe renderer support.
- Decided for 10.2: release preflight requires observer plugin template files and contract keywords once the scaffold lands.
