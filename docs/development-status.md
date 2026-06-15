# Hermeship 开发状态

最后更新：2026-06-15 21:15:53 CST

本文是下次启动 Codex 会话时的状态入口。执行开发前仍以 `tasks/development-checklist.md` 的 checkbox 为准；当前阶段计划维护在 `tasks/todo.md`。

## 当前结论

- Hermeship 的目标已经锁定：完全参考 `/Users/zq/Desktop/ai-projs/posp/template/clawhip` 的项目形态、架构和功能，只把 OpenClaw/Codex/Claude/OMC/OMX 等耦合替换为 Hermes 适配。
- Hermeship 不是调用现有 `clawhip` 的 thin adapter，也不依赖运行中的 `clawhip` daemon。
- 主实现语言确定为 Rust，采用 daemon-first 架构；Python 只用于 Hermes gateway hook bridge 模板 `handler.py`。
- 方案文档与执行清单已经拆分：方案文档维护架构和边界，`tasks/development-checklist.md` 和 `tasks/todo.md` 维护可勾选进度。
- 默认测试策略已经确定：使用本地 fixture、fake sink、fake HTTP、fake Hermes home、fake hermeship binary；真实 Discord/Hermes 只进入 live verification。
- 当前开发分支：`codex/milestone-1-cli`。
- 当前最新功能阶段提交：`b799415 feat: 实现 Hermes typed event model`。
- 当前工作树在本次状态更新前为干净状态；如后续继续开发，仍需先运行 `git status --short --branch` 确认。
- 当前下一步：从 Milestone 2.3 继续，优先执行任务 2.3：隐私与 payload 清洗。

## 已完成

- 已记录项目习惯：每完成一阶段任务，必须验证并提交；后续会话启动时先复习 `tasks/lessons.md`。
- 已重写方案文档：`docs/plans/2026-06-15-hermeship-development-plan.md`。
- 已重写阶段性开发清单：`tasks/development-checklist.md`。
- 已将测试计划集成到方案文档和开发清单。

### Milestone 0：契约与仓库基线

- 已复核 `template/clawhip` 指定参考文件，确认可移植形态为 Rust CLI、daemon、typed event、dispatcher、multi-delivery router、renderer/sink split、config/lifecycle/release preflight。
- 已复核 Hermes gateway hook 与 plugin 参考源码，确认 MVP 先使用 gateway hook bridge，plugin/observer 后续推进。
- 已更新 `README.md`，明确 Hermeship 是 Hermes-native daemon-first event router，不是 clawhip runtime client。
- 已运行旧 Python/thin-adapter 方向过滤搜索，正文无旧方案残留。
- 已提交：`af57c49 docs: 明确 hermeship 完整项目方向`。

### Milestone 1.1：Cargo 项目与 CLI 入口

- 已创建 Rust 2024 工程骨架：`Cargo.toml`、`Cargo.lock`、`src/lib.rs`、`src/main.rs`、`src/cli.rs`。
- 已实现最小 `clap` CLI 命令树：`start`、`status`、`send`、`emit`、`explain`、`config`、`hermes`、`install`、`uninstall`、`release`。
- 已新增 CLI parse 单元测试，覆盖 `send`、`emit --payload`、`hermes hook --payload`、`hermes install-hooks`。
- 已新增公开命令 fixture：`tests/fixtures/cli/public_commands.txt`，并断言必备公开命令前缀存在。
- 已运行验证：`cargo fmt --all -- --check`、`cargo test cli`、`cargo run -- --help`。
- 已提交：`d03170e chore: 搭建 Hermeship Rust CLI 骨架`。

### Milestone 1.2：配置模型

- 已新增 `src/config.rs`，并在 `src/lib.rs` 导出 `hermeship::config`。
- 已实现配置模型：`AppConfig`、`DaemonConfig`、`ProvidersConfig`、`DiscordConfig`、`DefaultsConfig`、`PrivacyConfig`、`HermesConfig`、`RouteRule`、`MessageFormat`。
- 已实现默认配置路径：`HERMESHIP_CONFIG` 优先，否则 `$HOME/.hermeship/config.toml`。
- 已实现默认配置与 TOML 加载：缺失配置返回默认值，非法 TOML 返回错误，未知 key 按前向兼容策略忽略。
- 已实现空值归一化和环境变量覆盖：`HERMESHIP_DAEMON_URL`、`HERMESHIP_DISCORD_TOKEN`、`HERMESHIP_DEFAULT_CHANNEL`、`HERMESHIP_DRY_RUN`。
- 已将 `hermeship config path`、`hermeship config show`、`hermeship config verify` 接入真实配置逻辑。
- 已运行验证：`cargo fmt --all -- --check`、`cargo test config`、`cargo run -- config show`、`cargo test`。
- 已提交：`50723af feat: 实现 hermeship 配置模型与 config CLI`。

### Milestone 1.3：质量门禁与仓库基础

- 已扩展 `.gitignore`：保留 `/target/`，新增本地编辑器临时文件、日志、临时目录、测试输出和覆盖率输出规则。
- 已确认 `.gitignore` 不忽略源码、文档、fixture 或 `Cargo.lock`。
- 已在 `README.md` 新增 Development Quality Gates。
- 已新增 fixture 目录：`tests/fixtures/hermes/`、`tests/fixtures/privacy/`、`tests/fixtures/routes/`、`tests/fixtures/discord/`。
- 已新增 `tests/fixtures/README.md`，明确 fixture 只能使用合成脱敏样例。
- 已运行验证：`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 已提交：`70c8f03 chore: 增加 Rust 质量门禁与仓库基础`。

### Milestone 2.1：IncomingEvent 与格式

- 已新增 `src/events.rs`，并在 `src/lib.rs` 导出 `hermeship::events`。
- 已实现 `IncomingEvent`：字段包含 `kind`、`channel`、`mention`、`format`、`template`、`payload`。
- 已实现 `RoutingMetadata`：覆盖 Hermes gateway 元数据和后续路由需要的通用字段，如 `tool`、`provider`、`source`、`platform`、`session_id`、`project`、`repo_path`、`branch`。
- 已采用单一 `MessageFormat` 策略：`src/config.rs` 保留唯一 enum 定义并新增 `from_label()`；`src/events.rs` 通过 `pub use crate::config::MessageFormat` 重导出。
- 已支持 `IncomingEvent` 反序列化字段别名：`type`、`kind`、`event`。
- 已支持缺省 payload 和 `payload: null` 归一为空对象；无显式 payload 时，top-level extra 字段进入 payload。
- 已将 `hermeship emit` 和 `hermeship explain` 的参数解析接入 `EventArgs::into_event()`。
- 已新增 Hermes 合成 fixture：`tests/fixtures/hermes/agent_start.json`、`session_end.json`、`invalid_payload.json`。
- 已运行验证：`cargo test events`、`cargo test cli`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 已提交：`5584b13 feat: 完成 Hermes 入口事件模型与 emit 解析`。

### Milestone 2.2：Typed EventEnvelope

- 已新增 `src/event/mod.rs`、`src/event/body.rs`、`src/event/compat.rs`，并在 `src/lib.rs` 导出 `hermeship::event`。
- 已定义 `EventEnvelope`、`EventBody`、`EventMetadata`、`EventPriority`。
- 已实现 Hermes event body：`HermesGatewayStarted`、`HermesSessionStarted`、`HermesSessionFinished`、`HermesSessionReset`、`HermesAgentStarted`、`HermesAgentStep`、`HermesAgentFinished`、`HermesAgentFailed`、`Custom`。
- 已实现 canonical mapping：`gateway:startup`、`session:start`、`session:end`、`session:reset`、`agent:start`、`agent:step`、`agent:end`；显式失败的 `agent:end` 会转为 `hermes.agent.failed`。
- 已实现 `IncomingEvent -> EventEnvelope` conversion，保留 route hint 并提取 provider/source/platform/chat/session/agent/project/repo metadata。
- 已覆盖未知 event -> `Custom`、缺失 `session_id` 降级、fixture conversion、大小写不敏感失败状态。
- 已运行验证：`cargo test event`、`cargo test events`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 已提交：`b799415 feat: 实现 Hermes typed event model`。

## 未完成

- Milestone 2.3：隐私与 payload 清洗尚未实现。
  - `src/privacy.rs` 尚未创建。
  - `sanitize_payload`、`redact_value`、`excerpt_policy` 尚未实现。
  - 敏感 key 递归脱敏尚未实现。
  - `message`、`response`、`conversation_history`、`request`、`provider_response`、`tool_result` 默认禁发尚未实现。
  - opt-in 摘录策略尚未实现。
  - `tests/fixtures/privacy/sensitive_payload.json` 尚未创建。
- Milestone 3 到 Milestone 10 均未执行。
- daemon、client、HTTP ingress、队列、router、renderer、dispatcher、Discord sink、Hermes hook bridge、安装/回滚、release preflight、live verification 均未实现。
- live Discord verification 凭据是否可用尚未确认。
- Slack sink、git/GitHub/tmux parity 是否进入 `0.1.0` 尚未最终确认。
- macOS launchd 是否与 systemd 同期实现尚未最终确认。

## 下一步入口

从 `tasks/development-checklist.md` 的 **Milestone 2：事件模型与兼容层** 继续，优先执行 **任务 2.3：隐私与 payload 清洗**。

建议第一段工作：

1. 复习 `tasks/lessons.md`、本文、方案文档、开发清单和 `tasks/todo.md`。
2. 确认当前分支、最新提交和未提交变更：
   - `git status --short --branch`
   - `git log -3 --oneline`
3. 确认最新已完成功能阶段提交为 `b799415 feat: 实现 Hermes typed event model`。
4. 读取当前相关代码：
   - `src/events.rs`
   - `src/event/mod.rs`
   - `src/event/body.rs`
   - `src/event/compat.rs`
   - `src/config.rs`
   - `tests/fixtures/README.md`
5. 从任务 2.3 继续，先写失败测试，再实现 `src/privacy.rs`。
6. 注意本阶段只实现 privacy 清洗纯逻辑与 fixture；不进入 daemon、client、HTTP ingress、队列、router、renderer、sink、hook bridge、install 或 release preflight。
7. 运行任务 2.3 验证命令：
   - `cargo test privacy`
   - `cargo test event`
   - `cargo test events`
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`
8. 更新 `tasks/development-checklist.md` 的运行状态日志和 `tasks/todo.md` 的 Review。
9. 阶段完成后必须验证并提交，commit 信息使用详细中文，说明变更、验证和影响。

## 下次启动提示词

```text
请在 /Users/zq/Desktop/ai-projs/posp/hermeship 继续 Hermeship 开发。

启动后请先阅读：
- tasks/lessons.md
- docs/development-status.md
- docs/plans/2026-06-15-hermeship-development-plan.md
- tasks/development-checklist.md
- tasks/todo.md

当前状态：
- 当前分支是 codex/milestone-1-cli。
- 最新功能阶段提交：b799415 feat: 实现 Hermes typed event model。
- Milestone 0 已完成并提交：af57c49 docs: 明确 hermeship 完整项目方向。
- Milestone 1.1 已完成并提交：d03170e chore: 搭建 Hermeship Rust CLI 骨架。
- Milestone 1.2 已完成并提交：50723af feat: 实现 hermeship 配置模型与 config CLI。
- Milestone 1.3 已完成并提交：70c8f03 chore: 增加 Rust 质量门禁与仓库基础。
- Milestone 2.1 已完成并提交：5584b13 feat: 完成 Hermes 入口事件模型与 emit 解析。
- Milestone 2.2 已完成并提交：b799415 feat: 实现 Hermes typed event model。
- 已实现 src/events.rs：IncomingEvent、RoutingMetadata、字段别名反序列化、空/null payload 归一，以及 MessageFormat 的单一复用/重导出策略。
- 已实现 src/event/：EventEnvelope、EventBody、EventMetadata、EventPriority、Hermes canonical mapping、IncomingEvent -> EventEnvelope conversion。
- Hermes canonical mapping 已覆盖 gateway:startup、session:start、session:end、session:reset、agent:start、agent:step、agent:end；显式失败的 agent:end 映射为 hermes.agent.failed；未知 event 降级为 Custom。
- 已通过验证：cargo test event、cargo test events、cargo fmt --all -- --check、cargo clippy --all-targets -- -D warnings、cargo test。
- Hermeship 是 Hermes-native daemon-first event router，不是 thin adapter，不调用 clawhip runtime，也不依赖运行中的 clawhip daemon。
- 方案文档只维护架构和边界，执行进度维护在 tasks/development-checklist.md 和 tasks/todo.md。

请从 tasks/development-checklist.md 的 Milestone 2 继续，优先执行任务 2.3：隐私与 payload 清洗：
1. 先复习 tasks/lessons.md，并确认当前分支、最新提交和未提交变更：git status --short --branch、git log -3 --oneline。
2. 确认 tasks/todo.md 的 Milestone 2.3 计划。
3. 阅读 src/events.rs、src/event/mod.rs、src/event/body.rs、src/event/compat.rs、src/config.rs、tests/fixtures/README.md。
4. 先写失败测试，再实现 src/privacy.rs。
5. 实现 sanitize_payload、redact_value、excerpt_policy。
6. 默认递归脱敏敏感 key：token、api_key、authorization、password、secret、cookie。
7. 默认禁止完整正文外发：message、response、conversation_history、request、provider_response、tool_result；保留 message_chars、response_chars、has_message、has_response 等安全摘要。
8. 实现 opt-in 摘录策略：include_message_excerpt、include_response_excerpt、max_excerpt_chars；必须先脱敏再截断。
9. 新增 tests/fixtures/privacy/sensitive_payload.json，必须是合成脱敏样例，不得包含真实 token、cookie、secret、完整 prompt、完整对话或 provider request/response body。
10. 本阶段不要实现 daemon、client、HTTP ingress、队列、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
11. 运行验证：cargo test privacy、cargo test event、cargo test events、cargo fmt --all -- --check、cargo clippy --all-targets -- -D warnings、cargo test。
12. 更新 tasks/development-checklist.md 的运行状态日志和 tasks/todo.md 的 Review。
13. 阶段完成后必须验证并提交，commit 信息使用详细中文，说明变更、验证和影响。
```
