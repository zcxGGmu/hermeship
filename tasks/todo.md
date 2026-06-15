# Task: Milestone 2.3 - 隐私与 Payload 清洗

启动时间：下次开发会话启动时确认

本阶段目标：在 Milestone 2.1 `IncomingEvent` 和 Milestone 2.2 typed `EventEnvelope` 的基础上，实现第一版隐私清洗纯逻辑。默认不得外发完整正文、完整对话、provider request/response、tool result body 或 secret；只保留安全摘要和结构化 metadata。本阶段不进入 daemon、client、HTTP ingress、队列、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。

- [ ] 复习 `tasks/lessons.md`，确认阶段完成后必须验证并提交。
- [ ] 复习阶段上下文。
  - 阅读：`docs/development-status.md`
  - 阅读：`docs/plans/2026-06-15-hermeship-development-plan.md`
  - 阅读：`tasks/development-checklist.md`
  - 阅读：`tasks/todo.md`
- [ ] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 预期：当前分支为 `codex/milestone-1-cli`；最新已完成功能阶段提交为 `b799415 feat: 实现 Hermes typed event model`；启动时不要混入无关改动。
- [ ] 明确本阶段边界。
  - 只处理 privacy 清洗纯逻辑、测试和合成 fixture。
  - 不实现 daemon、client、HTTP ingress、队列、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
- [ ] 检查现有代码。
  - 查看：`src/events.rs`
  - 查看：`src/event/mod.rs`
  - 查看：`src/event/body.rs`
  - 查看：`src/event/compat.rs`
  - 查看：`src/config.rs`
  - 查看：`src/lib.rs`
  - 查看：`tests/fixtures/README.md`
  - 完成标准：确认 privacy 模块只接收/返回 `serde_json::Value` 或清洗结果，不读取文件、不访问网络、不依赖真实 Hermes 或 Discord。
- [ ] 先写失败测试。
  - 新建或修改：`src/privacy.rs`
  - 覆盖：敏感 key 递归脱敏、正文默认禁发、短文本不原样泄漏、list/object 混合结构、非字符串值、原始 payload 不被原地修改、opt-in 摘录先脱敏再截断。
  - 命令：`cargo test privacy`
- [ ] 新建隐私模块。
  - 新建：`src/privacy.rs`
  - 在 `src/lib.rs` 导出 `hermeship::privacy`。
  - 函数：`sanitize_payload`、`redact_value`、`excerpt_policy`。
- [ ] 实现敏感 key 递归脱敏。
  - 默认 key：`token`、`api_key`、`authorization`、`password`、`secret`、`cookie`。
  - key 匹配大小写不敏感。
  - list/object 任意嵌套都必须处理。
  - 非字符串 secret 值也必须替换为固定脱敏标记。
- [ ] 实现正文默认禁发。
  - 默认删除或替换：`message`、`response`、`conversation_history`、`request`、`provider_response`、`tool_result`。
  - 默认保留安全摘要：`message_chars`、`response_chars`、`has_message`、`has_response`。
  - 短文本也不能因为长度低于截断阈值而原样泄漏。
- [ ] 实现 opt-in 摘录。
  - 配置来源：`PrivacyConfig` 的 `include_message_excerpt`、`include_response_excerpt`、`max_excerpt_chars`。
  - 摘录字段建议使用 `message_excerpt`、`response_excerpt`。
  - 必须先脱敏再截断。
  - 截断必须按 char 边界处理，避免切坏 UTF-8。
- [ ] 增加隐私回归 fixture。
  - 新建：`tests/fixtures/privacy/sensitive_payload.json`
  - 完成标准：fixture 是合成样例，不包含真实 token、cookie、secret、完整 prompt、完整对话或 provider request/response body。
  - 测试断言输出不包含原始 `message`、`response`、`conversation_history`、`request`、`provider_response`、`tool_result`、token、cookie、secret。
- [ ] 运行任务 2.3 验证命令。
  - `cargo test privacy`
  - `cargo test event`
  - `cargo test events`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- [ ] 更新 `tasks/development-checklist.md`。
  - 勾选任务 2.3 已完成项。
  - 在运行状态日志顶部记录本阶段实现、验证和提交状态。
- [ ] 更新 `tasks/todo.md` Review。
  - 记录实现、验证、边界和剩余风险。
- [ ] 提交任务 2.3。
  - commit：`feat: 增加 Hermes 事件隐私清洗`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- 待任务 2.3 实施、验证和提交后填写。
