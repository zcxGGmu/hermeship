import json
import os
import urllib.request


DEFAULT_DAEMON_URL = "http://127.0.0.1:25295"
DEFAULT_TIMEOUT_SECS = 2.0
MAX_TIMEOUT_SECS = 5.0
OBSERVER_SCHEMA_VERSION = 1
MAX_TEXT_CHARS = 240
MAX_ARG_KEYS = 16
MAX_ARG_KEY_CHARS = 64
MAX_PATTERN_KEYS = 16
MAX_PATTERN_KEY_CHARS = 64
TRUTHY_VALUES = {"1", "true", "yes", "on", "y", "enabled"}
CANONICAL_STATUS = {
    "ok": "ok",
    "success": "ok",
    "succeeded": "ok",
    "done": "done",
    "completed": "done",
    "complete": "done",
    "failed": "failed",
    "failure": "failed",
    "error": "failed",
    "interrupted": "interrupted",
    "cancelled": "interrupted",
    "canceled": "interrupted",
    "timeout": "timeout",
    "timed-out": "timeout",
    "running": "running",
    "started": "started",
    "pending": "pending",
    "skipped": "skipped",
}


def register(ctx):
    ctx.register_hook("on_session_start", _callback(
        "hermes.observer.session.started",
        ("session_id", "platform", "model"),
    ))
    ctx.register_hook("on_session_end", _callback(
        "hermes.observer.session.ended",
        (
            "session_id",
            "platform",
            "model",
            "completed",
            "interrupted",
            "reason",
            "task_id",
            "turn_id",
            "api_request_id",
        ),
    ))
    ctx.register_hook("on_session_finalize", _callback(
        "hermes.observer.session.finalized",
        ("session_id", "platform", "reason"),
    ))
    ctx.register_hook("on_session_reset", _callback(
        "hermes.observer.session.reset",
        ("session_id", "platform", "reason"),
    ))
    ctx.register_hook("pre_api_request", _callback(
        "hermes.observer.api.request.started",
        (
            "session_id",
            "task_id",
            "turn_id",
            "api_request_id",
            "platform",
            "model",
            "provider",
            "api_mode",
            "api_call_count",
            "message_count",
            "tool_count",
            "approx_input_tokens",
            "request_char_count",
            "max_tokens",
        ),
        _add_request_summary,
    ))
    ctx.register_hook("post_api_request", _callback(
        "hermes.observer.api.request.finished",
        (
            "session_id",
            "task_id",
            "turn_id",
            "api_request_id",
            "platform",
            "model",
            "provider",
            "api_mode",
            "api_call_count",
            "api_duration",
            "finish_reason",
            "message_count",
            "response_model",
            "assistant_content_chars",
            "assistant_tool_call_count",
        ),
        _add_response_summary,
    ))
    ctx.register_hook("api_request_error", _callback(
        "hermes.observer.api.request.failed",
        (
            "session_id",
            "task_id",
            "turn_id",
            "api_request_id",
            "platform",
            "model",
            "provider",
            "api_mode",
            "api_call_count",
            "error_type",
            "duration_ms",
        ),
        _add_error_summary,
    ))
    ctx.register_hook("pre_llm_call", _callback(
        "hermes.observer.llm.started",
        ("session_id", "platform", "model", "is_first_turn", "message_chars", "history_count"),
        _add_llm_request_summary,
    ))
    ctx.register_hook("post_llm_call", _callback(
        "hermes.observer.llm.finished",
        ("session_id", "platform", "model", "response_chars"),
        _add_llm_response_summary,
    ))
    ctx.register_hook("pre_tool_call", _callback(
        "hermes.observer.tool.started",
        ("session_id", "task_id", "turn_id", "api_request_id", "tool_call_id", "tool_name"),
        _add_tool_request_summary,
    ))
    ctx.register_hook("post_tool_call", _callback(
        "hermes.observer.tool.finished",
        (
            "session_id",
            "task_id",
            "turn_id",
            "api_request_id",
            "tool_call_id",
            "tool_name",
            "status",
            "duration_ms",
            "error_type",
        ),
        _add_tool_result_summary,
    ))
    ctx.register_hook("pre_approval_request", _callback(
        "hermes.observer.approval.requested",
        ("surface", "pattern_key", "turn_id", "tool_call_id"),
        _add_approval_summary,
    ))
    ctx.register_hook("post_approval_response", _callback(
        "hermes.observer.approval.responded",
        ("surface", "pattern_key", "turn_id", "tool_call_id", "choice"),
        _add_approval_summary,
    ))
    ctx.register_hook("subagent_start", _callback(
        "hermes.observer.subagent.started",
        (
            "parent_session_id",
            "parent_turn_id",
            "parent_subagent_id",
            "child_session_id",
            "child_subagent_id",
            "child_role",
        ),
        _add_subagent_start_summary,
    ))
    ctx.register_hook("subagent_stop", _callback(
        "hermes.observer.subagent.finished",
        (
            "parent_session_id",
            "parent_turn_id",
            "child_session_id",
            "child_role",
            "child_status",
            "duration_ms",
        ),
        _add_subagent_stop_summary,
    ))
    return None


def _callback(event_type, fields, extra_builder=None):
    def callback(*args, **kwargs):
        try:
            context = _context(args, kwargs)
            payload = _base_payload()
            _copy_fields(payload, context, fields)
            if extra_builder is not None:
                extra_builder(payload, context)
            _post_event(event_type, payload)
        except Exception:
            return None
        return None

    return callback


def _base_payload():
    return {
        "provider": "hermes",
        "source": "plugin",
        "observer_schema_version": OBSERVER_SCHEMA_VERSION,
    }


def _post_event(event_type, payload):
    if _observer_disabled():
        return
    envelope = {"type": event_type, "payload": payload}
    body = json.dumps(envelope, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
    request = urllib.request.Request(
        _event_url(),
        data=body,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=_timeout_secs()) as response:
            response.read()
    except Exception:
        return


def _event_url():
    base = os.environ.get("HERMESHIP_DAEMON_URL", "").strip() or DEFAULT_DAEMON_URL
    return base.rstrip("/") + "/event"


def _timeout_secs():
    raw = os.environ.get("HERMESHIP_OBSERVER_TIMEOUT_SECS", "").strip()
    if not raw:
        return DEFAULT_TIMEOUT_SECS
    try:
        timeout = float(raw)
    except ValueError:
        return DEFAULT_TIMEOUT_SECS
    return min(timeout, MAX_TIMEOUT_SECS) if timeout > 0 else DEFAULT_TIMEOUT_SECS


def _observer_disabled():
    raw = os.environ.get("HERMESHIP_OBSERVER_DISABLED", "").strip().lower()
    return raw in TRUTHY_VALUES


def _context(args, kwargs):
    values = {}
    for source in args:
        if isinstance(source, dict):
            values.update(source)
        elif source is not None:
            _copy_object_attrs(values, source)
    values.update(kwargs)
    nested = values.get("context")
    if isinstance(nested, dict):
        merged = dict(nested)
        merged.update(values)
        return merged
    return values


def _copy_object_attrs(target, source):
    for key in _known_source_keys():
        try:
            value = getattr(source, key)
        except Exception:
            continue
        if value is not None:
            target[key] = value


def _known_source_keys():
    return {
        "session_id",
        "platform",
        "model",
        "completed",
        "interrupted",
        "reason",
        "task_id",
        "turn_id",
        "api_request_id",
        "provider",
        "api_mode",
        "api_call_count",
        "message_count",
        "tool_count",
        "approx_input_tokens",
        "request_char_count",
        "max_tokens",
        "api_duration",
        "finish_reason",
        "response_model",
        "assistant_content_chars",
        "assistant_tool_call_count",
        "error_type",
        "error_message",
        "error_summary",
        "duration_ms",
        "is_first_turn",
        "message_chars",
        "history_count",
        "response_chars",
        "tool_call_id",
        "tool_name",
        "status",
        "session_key",
        "surface",
        "pattern_key",
        "pattern_keys",
        "choice",
        "parent_session_id",
        "parent_turn_id",
        "parent_subagent_id",
        "child_session_id",
        "child_subagent_id",
        "child_role",
        "child_status",
        "arguments",
        "args",
        "request",
        "messages",
        "request_messages",
        "response",
        "assistant_content",
        "assistant_message",
        "tool_calls",
        "usage",
        "result",
        "tool_result",
        "output",
        "error",
        "description",
        "command",
        "child_goal",
        "goal",
        "child_summary",
        "summary",
        "conversation_history",
    }


def _copy_fields(payload, context, fields):
    for key in fields:
        _put_observer_field(payload, key, _get(context, key))


def _put_observer_field(payload, key, value):
    if key in {"reason", "session_key"}:
        _put_text_summary(payload, key, value)
        return
    if key in {"status", "child_status"}:
        status = _canonical_status(value)
        if status is None:
            _put_text_summary(payload, key, value)
        else:
            payload[key] = status
        return
    if key == "error_type":
        code = _safe_code(value)
        if code is None:
            _put_text_summary(payload, key, value)
        else:
            payload[key] = code
        return
    _put(payload, key, value)


def _put_text_summary(payload, key, value):
    chars = _count_chars(value)
    if chars is not None:
        payload[f"{key}_chars"] = chars
        payload[f"has_{key}"] = chars > 0


def _safe_code(value):
    if not isinstance(value, str):
        return None
    text = value.strip()
    if not text or len(text) > 64:
        return None
    if (
        text.isidentifier()
        and text.endswith("Error")
        and any(ch.isalpha() for ch in text)
    ):
        return text
    return None


def _canonical_status(value):
    if not isinstance(value, str):
        return None
    return CANONICAL_STATUS.get(value.strip().lower().replace("_", "-"))


def _put(payload, key, value):
    safe = _safe_scalar(value)
    if safe is not None:
        payload[key] = safe


def _get(context, key, *aliases):
    for candidate in (key,) + aliases:
        if isinstance(context, dict) and candidate in context:
            return context[candidate]
    return None


def _safe_scalar(value):
    if value is None:
        return None
    if isinstance(value, bool):
        return value
    if isinstance(value, int):
        return value
    if isinstance(value, float):
        return value
    if isinstance(value, str):
        return _bounded(value)
    return None


def _bounded(value, limit=MAX_TEXT_CHARS):
    text = str(value).replace("\n", " ").strip()
    if len(text) <= limit:
        return text
    return text[:limit]


def _count_chars(value):
    if value is None:
        return None
    if isinstance(value, str):
        return len(value)
    try:
        return len(json.dumps(value, ensure_ascii=False, separators=(",", ":")))
    except Exception:
        return len(str(value))


def _count_items(value):
    if isinstance(value, (list, tuple)):
        return len(value)
    return None


def _safe_int(value):
    if isinstance(value, bool):
        return None
    if isinstance(value, int):
        return value
    if isinstance(value, float):
        return int(value)
    if isinstance(value, str):
        try:
            return int(value)
        except ValueError:
            return None
    return None


def _add_request_summary(payload, context):
    request = _get(context, "request")
    messages = _get(context, "request_messages", "messages")
    if "request_char_count" not in payload:
        chars = _count_chars(request)
        if chars is None:
            chars = _count_chars(messages)
        _put(payload, "request_char_count", chars)
    if "message_count" not in payload:
        _put(payload, "message_count", _count_items(messages))


def _add_response_summary(payload, context):
    response = _get(context, "response")
    assistant = _get(context, "assistant_content", "assistant_message")
    if "assistant_content_chars" not in payload:
        chars = _count_chars(assistant)
        if chars is None:
            chars = _count_chars(response)
        _put(payload, "assistant_content_chars", chars)
    if "assistant_tool_call_count" not in payload:
        _put(payload, "assistant_tool_call_count", _count_items(_get(context, "tool_calls")))
    _add_token_usage(payload, context)


def _add_token_usage(payload, context):
    usage = _get(context, "usage")
    for key in ("input_tokens", "output_tokens", "total_tokens", "prompt_tokens", "completion_tokens"):
        value = _get(context, key)
        if value is None and isinstance(usage, dict):
            value = usage.get(key)
        value = _safe_int(value)
        if value is not None:
            payload[key] = value


def _add_error_summary(payload, context):
    if "error_type" not in payload:
        error = _get(context, "error")
        if error is not None:
            payload["error_type"] = type(error).__name__
    if "error_message" not in payload:
        _put(payload, "error_message", _get(context, "error_summary"))


def _add_llm_request_summary(payload, context):
    if "message_chars" not in payload:
        message = _get(context, "user_message", "message")
        _put(payload, "message_chars", _count_chars(message))
    if "history_count" not in payload:
        _put(payload, "history_count", _count_items(_get(context, "conversation_history")))


def _add_llm_response_summary(payload, context):
    if "response_chars" not in payload:
        response = _get(context, "response", "assistant_content", "assistant_message")
        _put(payload, "response_chars", _count_chars(response))


def _add_tool_request_summary(payload, context):
    args = _get(context, "arguments", "args")
    if isinstance(args, dict):
        keys = sorted(_bounded(key, MAX_ARG_KEY_CHARS) for key in args.keys())
        payload["arg_keys"] = keys[:MAX_ARG_KEYS]
        payload["arg_key_count"] = len(keys)
    if args is not None:
        payload["arg_chars"] = _count_chars(args)


def _add_tool_result_summary(payload, context):
    result = _get(context, "result", "tool_result", "output")
    if result is not None:
        payload["result_chars"] = _count_chars(result)
    _add_error_summary(payload, context)


def _add_approval_summary(payload, context):
    session_key = _get(context, "session_key")
    if session_key is not None:
        payload["session_key_chars"] = _count_chars(session_key)
        payload["has_session_key"] = True
    pattern_keys = _get(context, "pattern_keys")
    if isinstance(pattern_keys, (list, tuple)):
        keys = [_bounded(value, MAX_PATTERN_KEY_CHARS) for value in pattern_keys]
        payload["pattern_keys"] = keys[:MAX_PATTERN_KEYS]
        payload["pattern_key_count"] = len(keys)
    description = _get(context, "description")
    command = _get(context, "command")
    if description is not None:
        payload["description_chars"] = _count_chars(description)
    if command is not None:
        payload["command_chars"] = _count_chars(command)


def _add_subagent_start_summary(payload, context):
    goal = _get(context, "child_goal", "goal")
    if goal is not None:
        payload["child_goal_chars"] = _count_chars(goal)


def _add_subagent_stop_summary(payload, context):
    summary = _get(context, "child_summary", "summary")
    if summary is not None:
        payload["child_summary_chars"] = _count_chars(summary)
