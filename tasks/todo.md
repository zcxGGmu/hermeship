# Task: Milestone 10 后续 - Typed Rust Observer Body

更新时间：2026-06-19

本轮任务基于已完成的 Milestone 10.3 Observer Plugin install/enable CLI，继续推进 `hermes.observer.*` 的 Rust typed event body、路由字段和安全渲染。范围限定为本地 deterministic 开发与文档状态同步；默认不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 启动时工作树：干净，`git status --short --branch` 只显示分支行。
- 当前 HEAD：`4714fc9 docs: 更新 Hermeship 最新开发状态`。
- 最近 5 个提交：`4714fc9`、`803aefa`、`5d4c534`、`f352222`、`eb64408`。
- 最新状态文档提交：`4714fc9 docs: 更新 Hermeship 最新开发状态`。
- 最新 Milestone 10.3 功能阶段提交：`803aefa feat: 增加 Hermes observer plugin 安装启用 CLI`。
- 最新 Milestone 10.2 功能阶段提交：`f352222 feat: 增加可选 Hermes observer plugin scaffold`。
- 最新 Milestone 10.1 契约研究提交：`93aa9ec docs: 完成 Hermes observer plugin 契约研究`。
- Milestone 9.3 已完成 `blocked`/`not_run` 记录；真实 Discord/Hermes live verification 仍未获得 `pass`。
- `release preflight` 的 `live verification` ok 只证明 `docs/live-verification.md` 记录字段存在，不断言真实 live pass。

## 本轮执行计划

- [x] 复习 lessons、确认 Git 状态和最近提交。
  - 已读：`tasks/lessons.md`。
  - 已运行：`git status --short --branch`。
  - 已运行：`git log -5 --oneline`。

- [x] 阅读当前状态入口、契约、运维和 preflight 文件。
  - 已读：`docs/development-status.md`。
  - 已读：`tasks/development-checklist.md`。
  - 已读：`tasks/todo.md`。
  - 已读：`docs/live-verification.md`。
  - 已读：`README.md`。
  - 已读：`ARCHITECTURE.md`。
  - 已读：`docs/operations.md`。
  - 已读：`docs/hermes-event-contract.md`。
  - 已读：`docs/observer-plugin.md`。
  - 已读：`docs/plans/2026-06-15-hermeship-development-plan.md`。
  - 已读：`src/release_preflight.rs`。

- [x] 写 typed observer body 失败测试。
  - 文件：`src/event/compat.rs`。
  - 目标：`hermes.observer.tool.started`、`hermes.observer.api.request.failed`、`hermes.observer.subagent.finished` 不再落入 `Custom`，而是映射到新的 typed observer body。
  - 目标：保留 canonical kind 原样，`source=hermes` 或 `source=plugin` 语义不破坏现有 route candidate。
  - 目标：断言 raw prompt、request/response body、tool result、child summary 只作为长度/计数字段进入 typed body，不被原样保存。

- [x] 写 observer router filter 失败测试。
  - 文件：`src/router.rs`。
  - 目标：route filter 可匹配 `observer_hook`、`observer_category`、`tool_name`、`status`、`model`、`provider`、`session_id`、`child_role`。
  - 目标：`repo_name` 等既有 filter 不受影响。

- [x] 写 observer renderer 失败测试。
  - 文件：`src/render/default.rs`。
  - 目标：compact/inline 能输出可读 observer 摘要，例如 tool/api/subagent/session/approval。
  - 目标：raw JSON 只输出受控字段和 *_chars / *_count，不输出 raw command、tool result、request、response、child goal 或 child summary。

- [x] 实现最小 typed observer body。
  - 文件：`src/event/body.rs`。
  - 文件：`src/event/mod.rs`。
  - 文件：`src/event/compat.rs`。
  - 建议结构：新增一个通用 `HermesObserverEvent`，字段覆盖 observer schema version、category、hook/action、session/task/turn/api/tool/subagent/model/status/timing/counts/error summary 等 safe fields。
  - 边界：不改 Python observer plugin 的 safe-field forwarding，不扩大 payload 原文转发。

- [x] 接入 router 和 renderer。
  - 文件：`src/router.rs`。
  - 文件：`src/render/default.rs`。
  - 目标：observer typed body 可以被结构化 filter 使用，并获得比 Custom fallback 更有用的默认 compact/raw 渲染。

- [x] 运行 targeted red/green 验证。
  - 已运行：`cargo test observer`（20 passed）。
  - 已运行：`cargo test router`（17 lib-filtered tests + 1 bin-filtered test passed）。
  - 已运行：`cargo test render`（25 lib-filtered tests + 1 bin-filtered test passed）。

- [x] 更新文档和状态记录。
  - 文件：`README.md`。
  - 文件：`ARCHITECTURE.md`。
  - 文件：`docs/operations.md`。
  - 文件：`docs/hermes-event-contract.md`。
  - 文件：`docs/observer-plugin.md`。
  - 文件：`docs/development-status.md`。
  - 文件：`tasks/development-checklist.md`。
  - 目标：把 typed observer body 从未完成项移到已完成项，并保留真实 live pass、Slack sink、自动启用 plugin 仍未完成/非目标的边界。

- [x] 运行最终验证。
  - 命令：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 命令：`cargo test observer_plugin`。
  - 命令：`cargo test release_preflight`。
  - 命令：`cargo run -- release preflight 0.1.0`。
  - 命令：`cargo fmt --all -- --check`。
  - 命令：`cargo clippy --all-targets -- -D warnings`。
  - 命令：`cargo test`。
  - 已验证：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 已验证：`cargo test observer_plugin`（13 passed）。
  - 已验证：`cargo test release_preflight`（15 passed）。
  - 已验证：`cargo run -- release preflight 0.1.0`（9 checks ok；当时 `live verification` ok 仅证明记录字段存在，不代表真实 live pass）。
  - 已验证：`cargo fmt --all -- --check`。
  - 已验证：`cargo clippy --all-targets -- -D warnings`。
  - 已验证：`cargo test`（214 lib tests + 15 bin tests passed）。

- [x] 处理 typed observer body 代码审查反馈。
  - 移除或摘要化 `session_key`，避免 approval observer payload 在 typed body、raw renderer 或 route explain 中暴露原值。
  - 对 `error_type`、`reason`、`status` 等自由文本字段做 canonical/长度摘要处理，避免错误原文或任意多行文本进入 renderer/raw/explain。
  - 修复 observer body fields 覆盖 route context 中同名 metadata key 的问题，保留 metadata 优先级，body 同名字段使用 `observer_*` 别名或不覆盖。
  - 保留已补的 Rust observer string field 单行和长度限制，并用回归测试覆盖。

- [x] 写并运行 review 回归测试。
  - `session_key` 注入 secret-like 值时，typed body、compact/inline/raw、route explain 均不包含原值，只包含长度/存在性摘要。
  - `error_type`、`reason`、`status` 注入多行完整错误文本时，输出只包含 canonical code 或长度/存在性摘要。
  - metadata `provider` 与 observer body `provider` 不一致时，route filter 仍以 metadata 为准，body 值只能通过 `observer_provider` 匹配。
  - observer string/list 字段继续保持单行和长度上限。

- [x] 重新运行最终验证。
  - 命令：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 命令：`cargo test observer_plugin`。
  - 命令：`cargo test release_preflight`。
  - 命令：`cargo run -- release preflight 0.1.0`。
  - 命令：`cargo fmt --all -- --check`。
  - 命令：`cargo clippy --all-targets -- -D warnings`。
  - 命令：`cargo test`。
  - 已验证：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 已验证：`cargo test observer_plugin`（13 passed）。
  - 已验证：`cargo test release_preflight`（15 passed）。
  - 已验证：`cargo run -- release preflight 0.1.0`（9 checks ok；当时 `live verification` ok 仅证明记录字段存在，不断言真实 live pass）。
  - 已验证：`cargo fmt --all -- --check`。
  - 已验证：`cargo clippy --all-targets -- -D warnings`。
  - 已验证：`cargo test`（218 lib tests + 15 bin tests + doctests passed）。

- [x] 处理最终代码审查反馈。
  - 修复 object-style approval hook context 无法收集 `session_key` 并生成安全摘要的问题。
  - 修复 observer API provider 入站时覆盖 core metadata `provider` 的风险，保持 core provider 为 `hermes`，API provider 只作为 observer body 字段。
  - 收紧 `error_type` safe-code 规则，避免 secret-shaped 单 token 原样进入 typed body/raw/explain。
  - 澄清 release preflight CLI 输出：`live verification` ok 仅代表记录字段存在，不断言真实 live pass。
  - 将文档中 `error_type` 的 canonical 表述改为 bounded safe code 或长度/存在性摘要。

- [x] 补最终审查回归测试并验证 Red/Green。
  - `cargo test observer_provider_metadata_is_hermes_even_when_body_provider_is_api_provider --lib`。
  - `cargo test observer_secret_shaped_error_type_is_summarized --lib`。
  - `cargo test observer_plugin_smoke_registers_hooks_and_forwards_safe_fields --lib`。
  - `cargo test preflight_live_verification_ok_says_record_fields_only --lib`。
  - 已验证 Red：`cargo test observer --lib` 在修复前失败于 secret-shaped `error_type` 原样保存、observer API provider 覆盖 core metadata、object-style approval 缺少 `session_key_chars`；`cargo test preflight_live_verification_ok_says_record_fields_only --lib` 在修复前失败于旧 preflight 文案。
  - 已验证 Green：`cargo test observer --lib`（26 passed）、`cargo test preflight_live_verification_ok_says_record_fields_only --lib`（1 passed）、`cargo test release_preflight --lib`（16 passed）、`cargo test router --lib`（18 passed）、`cargo test render --lib`（26 passed）。

- [x] 更新 Review 并提交。
  - 文件：`docs/development-status.md`。
  - 文件：`tasks/development-checklist.md`。
  - 文件：`tasks/todo.md`。
  - 提交前检查：`git status --short --branch`、`git diff --stat`、`git diff --name-only`。
  - commit 信息：中文详细说明 typed observer body、安全 hardening、验证结果和影响。
  - 已更新 Review；提交将在验证与 `git diff --check` 后执行。

## Review

- 已新增 typed Rust observer body：`hermes.observer.*` 现在进入 `EventBody::HermesObserver`，保留 canonical kind，并派生 `observer_category`、`observer_action` 和 `observer_schema_version`。
- 已实现 observer safe-field allowlist 与安全 hardening：只保存结构化 id、canonical status、bounded exception-class `error_type`、计数、长度、token usage、role/status 等安全字段；`session_key`、`reason`、secret-shaped 或非 safe `error_type`、非 canonical `status` / `child_status`、`error_message` / `error_summary` 均只转为 `*_chars` / `has_*` 摘要。
- 已接入 observer route filter 和 renderer：route 可按 observer category/action/tool/status/model/api/subagent 字段匹配；observer body 中与 core metadata 同名的字段通过 `observer_<field>` alias 匹配，不覆盖 `provider`、`source`、`platform`、`session_id` 等 core metadata；`hermes.observer.*` core `provider` 固定为 `hermes`，API provider 通过 `observer_provider` 匹配；compact/inline/raw 输出均不包含 raw command、tool result、request/response、child goal、child summary、raw session key 或任意错误/原因原文。
- 已更新 README、ARCHITECTURE、operations、Hermes event contract、observer contract、development status 和 development checklist，明确 typed body 与安全 hardening 已完成，同时保留真实 live pass 未完成、Slack sink 不在默认范围、observer plugin 不自动启用的边界。
- 本轮未执行真实 Discord/Hermes live check，未新增 `docs/live-verification.md` 真实 pass 结果，未实现 Slack sink，未自动启用 Hermes observer plugin。
- 已运行 targeted Red/Green 验证：新增 `session_key` / 自由文本 / metadata shadowing 回归测试在修复前失败；最终审查反馈新增 object-style approval `session_key` 摘要、observer API provider metadata、secret-shaped `error_type` 和 preflight live 文案回归测试，也均先失败后修复通过。
- 已运行阶段内 targeted 验证：`cargo test observer --lib`（26 passed）、`cargo test router --lib`（18 passed）、`cargo test render --lib`（26 passed）、`cargo test release_preflight --lib`（16 passed）。
- 已运行最终验证：`python3 -m py_compile templates/hermes-plugin/__init__.py`、`cargo test observer_plugin`（13 passed）、`cargo test release_preflight`（16 passed）、`cargo run -- release preflight 0.1.0`（9 checks ok，`live verification` 输出为记录字段存在且不声明真实 pass）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（221 lib tests + 15 bin tests + doctests passed）。
