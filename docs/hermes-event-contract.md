# Hermeship Hermes Event Contract

本文记录 Hermeship 当前已实现的 Hermes gateway hook input、canonical event、payload 字段、route metadata 和隐私规则。

## Ingress Paths

Hermeship 接收三类公开事件输入：

- `POST /event`：通用 `IncomingEvent`。
- `POST /api/hermes/hook`：Hermes hook envelope。
- `hermeship hermes hook --payload <json-or->`：CLI wrapper，最终 POST 到 daemon。

`hermeship emit` 和 source CLI 也会构造 `IncomingEvent` 并投递 `/event`。

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

Additional body-derived keys:

- GitHub: `owner`, `repo`, `repo_name`, `number`, `branch`, `base_branch`, `workflow`, `status`, `tag`
- tmux: `session`, `session_name`, `window`, `pane`, `keyword`, `minutes`
- cron: `cron_job_id`, `cron_schedule`

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
