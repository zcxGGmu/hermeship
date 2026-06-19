# Hermeship Hermes Event Contract

本文记录 Hermeship 当前已实现的 Hermes gateway hook input、canonical event、payload 字段、route metadata 和隐私规则。

## Ingress Paths

Hermeship 接收三类公开事件输入：

- `POST /event`：通用 `IncomingEvent`。
- `POST /api/hermes/hook`：Hermes hook envelope。
- `hermeship hermes hook --payload <json-or->`：CLI wrapper，最终 POST 到 daemon。

`hermeship emit`、source CLI 和可选 Hermes observer plugin 也会构造 `IncomingEvent` 并投递 `/event`。Observer plugin payload 必须使用 `hermes.observer.*` namespace，并进入 typed observer body；真正未知的非 observer 事件才通过 `Custom` fallback 保留已清洗 payload。

## Hermes Hook Envelope

Hermes hook bridge 输入：

```json
{
  "provider": "hermes",
  "source": "gateway",
  "event": "agent:start",
  "context": {
    "platform": "telegram",
    "chat_id": "synthetic-chat",
    "session_id": "synthetic-session",
    "agent_name": "codex",
    "project": "Hermeship",
    "message_chars": 42,
    "has_message": true
  }
}
```

Accepted aliases for the event field:

- `event`
- `event_type`
- `kind`
- `type`

Defaults:

- `provider`: `hermes`
- `source`: `gateway`
- `context`: `{}`

`context` must be a JSON object. Missing or empty event is rejected.

The Hermes hook envelope normalizes to an `IncomingEvent` whose payload contains:

```json
{
  "provider": "hermes",
  "source": "gateway",
  "event": "agent:start",
  "context": {}
}
```

During typed conversion, Hermeship reads recognized fields from either top-level payload or nested `context`.

## IncomingEvent

Generic event shape:

```json
{
  "type": "hermes.agent.started",
  "channel": "ops",
  "mention": "@here",
  "format": "compact",
  "template": "agent {session_id}",
  "payload": {
    "session_id": "synthetic-session"
  }
}
```

Field aliases for event kind:

- `type`
- `kind`
- `event`

Optional route hints:

- `channel`
- `mention`
- `format`: `compact`, `inline`, `alert`, `raw`
- `template`

If `payload` is absent or `null`, top-level extra fields become payload. If `payload` is an object and CLI extra fields are provided, they are merged into payload.

## Canonical Hermes Events

| Hermes hook input | Canonical event | Body |
| --- | --- | --- |
| `gateway:startup` | `hermes.gateway.started` | `HermesGatewayStarted` |
| `gateway.startup` | `hermes.gateway.started` | `HermesGatewayStarted` |
| `session:start` | `hermes.session.started` | `HermesSessionStarted` |
| `session.started` | `hermes.session.started` | `HermesSessionStarted` |
| `session:end` | `hermes.session.finished` | `HermesSessionFinished` |
| `session.finished` | `hermes.session.finished` | `HermesSessionFinished` |
| `session:reset` | `hermes.session.reset` | `HermesSessionReset` |
| `agent:start` | `hermes.agent.started` | `HermesAgentStarted` |
| `agent:step` | `hermes.agent.step` | `HermesAgentStep` |
| `agent:end` | `hermes.agent.finished` | `HermesAgentFinished` |
| `agent:end` with `success=false`, error field, or failed status | `hermes.agent.failed` | `HermesAgentFailed` |
| `agent:failed` | `hermes.agent.failed` | `HermesAgentFailed` |

The installed `HOOK.yaml` currently defaults to:

- `gateway:startup`
- `session:start`
- `session:end`
- `session:reset`
- `agent:start`
- `agent:end`

`agent:step` is supported by compat mapping but is not installed by default. `command:*` is not a default Hermeship hook path in the current milestone.

## Hermes Payload Fields

Recognized Hermes metadata fields:

- `provider`
- `source`
- `platform`
- `user_id`
- `chat_id`
- `thread_id`
- `chat_type`
- `session_id`
- `agent_name`
- `project`

Recognized session body fields:

- `status`
- `session_id`
- `platform`
- `project`
- `message_chars`
- `response_chars`
- `has_message`
- `has_response`
- `success`

Recognized agent body fields:

- `status`
- `agent_name`
- `session_id`
- `platform`
- `project`
- `step_name`
- `step`
- `tool_name`
- `message_chars`
- `response_chars`
- `has_message`
- `has_response`
- `elapsed_secs`
- `success`
- `error_message`
- `error_summary`
- `error`

Agent failure detection is explicit. `agent:end` maps to `hermes.agent.failed` only when the payload contains one of:

- `success: false`
- `error_message`, `error_summary`, or `error`
- `status` equal to `failed`, `failure`, or `error` case-insensitively

## Hermes Observer Events

Optional Hermes observer plugin events keep their `hermes.observer.*` canonical kind and use `HermesObserverEvent`.

Recognized observer namespace examples:

| Event | Category | Action |
| --- | --- | --- |
| `hermes.observer.session.started` | `session` | `started` |
| `hermes.observer.session.ended` | `session` | `ended` |
| `hermes.observer.api.request.started` | `api` | `request.started` |
| `hermes.observer.api.request.finished` | `api` | `request.finished` |
| `hermes.observer.api.request.failed` | `api` | `request.failed` |
| `hermes.observer.llm.started` | `llm` | `started` |
| `hermes.observer.llm.finished` | `llm` | `finished` |
| `hermes.observer.tool.started` | `tool` | `started` |
| `hermes.observer.tool.finished` | `tool` | `finished` |
| `hermes.observer.approval.requested` | `approval` | `requested` |
| `hermes.observer.approval.responded` | `approval` | `responded` |
| `hermes.observer.subagent.started` | `subagent` | `started` |
| `hermes.observer.subagent.finished` | `subagent` | `finished` |

Typed observer body stores only allowlisted safe scalar/list fields such as:

- `observer_schema_version`
- `session_id`, `task_id`, `turn_id`, `api_request_id`
- `platform`, `model`, `provider`, `api_mode`
- `tool_call_id`, `tool_name`, canonical `status`, bounded exception-class `error_type`
- `duration_ms`, `api_duration`, `*_count`, `*_chars`, token count fields
- `surface`, `pattern_key`, `pattern_keys`, `choice`, `session_key_chars`, `has_session_key`
- `parent_session_id`, `parent_turn_id`, `parent_subagent_id`, `child_session_id`, `child_subagent_id`, `child_role`, canonical `child_status`

The typed body does not store raw `request`, `response`, `messages`, `assistant_content`, `tool_result`, `result`, `output`, `arguments`, `args`, `command`, `description`, `child_goal`, `child_summary`, `summary`, `conversation_history`, `session_key`, `reason`, `error_message` or `error_summary`. Error text is represented only as `error_message_chars` and `has_error_message`. `error_type` is only stored when it is a bounded exception-class style code such as `RuntimeError`; secret-shaped or otherwise non-safe `error_type`, non-canonical `status`, `child_status` and `reason` values are represented as `*_chars` and `has_*` summaries.

## Other Canonical Events

Hermeship also supports local deterministic source events:

| Event | Source |
| --- | --- |
| `git.commit` | `hermeship git commit` |
| `git.branch-changed` | `hermeship git branch-changed` |
| `github.issue-opened` | `hermeship github issue-opened` |
| `github.pr-opened` | `hermeship github pr-opened` |
| `github.check-failed` | `hermeship github check-failed` |
| `github.release-published` | `hermeship github release-published` |
| `tmux.keyword` | `hermeship tmux keyword` |
| `tmux.stale` | `hermeship tmux stale` |
| `cron.run` | `hermeship cron run` |

Unknown event kinds are preserved as `Custom` with sanitized payload.

## Route Metadata

Router filters operate on structured metadata, not rendered message text.

Common route filter keys:

- `event`
- `canonical_kind`
- `source`
- `provider`
- `platform`
- `user_id`
- `chat_id`
- `thread_id`
- `chat_type`
- `session_id`
- `agent_name`
- `project`
- `repo_name`
- `repo_path`
- `worktree_path`
- `branch`
- `channel`
- `observer_category`
- `observer_action`
- `observer_schema_version`

Additional body-derived keys:

- GitHub: `owner`, `repo`, `repo_name`, `number`, `branch`, `base_branch`, `workflow`, `status`, `tag`
- tmux: `session`, `session_name`, `window`, `pane`, `keyword`, `minutes`
- cron: `cron_job_id`, `cron_schedule`
- Hermes observer: `tool_name`, canonical `status`, `model`, `api_mode`, `duration_ms`, `result_chars`, `child_role`, canonical `child_status` and other typed safe observer fields. Body fields that share names with core metadata also receive `observer_<field>` aliases, and core metadata keeps priority for keys such as `provider`, `source`, `platform` and `session_id`; for observer events, core `provider` remains `hermes`, while API provider names can be matched as `observer_provider`.

Route event patterns support glob `*`, for example:

```toml
[[routes]]
event = "hermes.agent.*"
filter = { platform = "telegram", project = "Hermeship" }
sink = "discord"
channel = "123456789012345678"
format = "compact"
```

For `hermes.agent.started`, route candidates are:

- `hermes.agent.started`
- `hermes.agent.*`
- `hermes.*`

## Rendering Contract

Supported formats:

- `compact`: default one-line summary.
- `inline`: canonical kind plus pipe-delimited fields.
- `alert`: `ALERT: ` prefix plus optional mention.
- `raw`: safe JSON for diagnostics.

Template tokens:

- `{event}`
- `{canonical_kind}`
- `{source}`
- `{provider}`
- `{platform}`
- `{session_id}`
- `{agent_name}`
- `{project}`
- `{channel}`

Unknown template tokens remain unchanged.

## Privacy Contract

Ingress payloads are sanitized before enqueue. Renderer raw output also uses controlled typed fields and sanitized payloads.

Default sanitizer:

- recursively redacts sensitive keys such as `token`, `api_key`, `authorization`, `password`, `secret`, and `cookie`;
- removes full body fields such as `message`, `response`, `conversation_history`, `request`, `provider_response`, and `tool_result`;
- preserves safe summaries such as `message_chars`, `response_chars`, `has_message`, and `has_response`;
- rejects original summary overrides that would replace computed safe summary fields;
- strips inline secrets from strings and URL query parameters where covered by sanitizer tests.

Excerpt behavior:

- `privacy.include_message_excerpt = false` by default.
- `privacy.include_response_excerpt = false` by default.
- excerpts are opt-in, sanitized first, and capped by `privacy.max_excerpt_chars`.

Fixtures, live records, logs and docs must not contain real Discord tokens, cookies, secrets, full prompts, full conversations or provider request/response bodies.

## Fail-Open Boundary

The Hermes gateway hook bridge must not block Hermes:

- missing binary: short stderr diagnostic, return `None`;
- child process non-zero exit: short stderr diagnostic, return `None`;
- timeout: short stderr diagnostic, return `None`;
- malformed context: daemon/CLI may reject the payload, but handler catches subprocess errors.

Daemon ingress can return 4xx/5xx to CLI or HTTP callers, but Hermes hook handler catches those failures and stays fail-open from Hermes gateway perspective.
