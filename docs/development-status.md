# Hermeship 开发状态

最后更新：2026-06-15 20:04:29 CST

本文是下次启动 Codex 会话时的状态入口。执行开发前仍以 `tasks/development-checklist.md` 的 checkbox 为准；当前阶段计划维护在 `tasks/todo.md`。

## 当前结论

- Hermeship 的目标已经锁定：完全参考 `/Users/zq/Desktop/ai-projs/posp/template/clawhip` 的项目形态、架构和功能，只把 OpenClaw/Codex/Claude/OMC/OMX 等耦合替换为 Hermes 适配。
- Hermeship 不是调用现有 `clawhip` 的 thin adapter，也不依赖运行中的 `clawhip` daemon。
- 主实现语言确定为 Rust，采用 daemon-first 架构；Python 只用于 Hermes gateway hook bridge 模板 `handler.py`。
- 方案文档与执行清单已经拆分：方案文档维护架构和边界，`tasks/development-checklist.md` 和 `tasks/todo.md` 维护可勾选进度。
- 默认测试策略已经确定：使用本地 fixture、fake sink、fake HTTP、fake Hermes home、fake hermeship binary；真实 Discord/Hermes 只进入 live verification。
- 当前开发分支：`codex/milestone-1-cli`。
- 当前最新已完成阶段提交：`70c8f03 chore: 增加 Rust 质量门禁与仓库基础`。
- 当前工作树在本次状态更新前为干净状态；如后续继续开发，仍需先运行 `git status --short --branch` 确认。
- 当前下一步：从 Milestone 2 开始，优先执行任务 2.1：`IncomingEvent` 与格式。

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
- 已运行 Red/Green：实现前 `cargo test cli` 因缺少 CLI 类型失败，实现后通过。
- 已运行任务 1.1 验证命令：
  - `cargo fmt --all -- --check`
  - `cargo test cli`
  - `cargo run -- --help`
- 已提交：`d03170e chore: 搭建 Hermeship Rust CLI 骨架`。

### Milestone 1.2：配置模型

- 已新增 `src/config.rs`，并在 `src/lib.rs` 导出 `hermeship::config`。
- 已实现配置模型：`AppConfig`、`DaemonConfig`、`ProvidersConfig`、`DiscordConfig`、`DefaultsConfig`、`PrivacyConfig`、`HermesConfig`、`RouteRule`、`MessageFormat`。
- 已实现默认配置路径：`HERMESHIP_CONFIG` 优先，否则 `$HOME/.hermeship/config.toml`。
- 已实现默认配置与 TOML 加载：缺失配置返回默认值，非法 TOML 返回错误，未知 key 按前向兼容策略忽略。
- 已实现空值归一化：空 token/channel/webhook/mention/template/filter 值会被清理，空 route sink 回退为 `discord`。
- 已实现配置环境变量覆盖：`HERMESHIP_DAEMON_URL`、`HERMESHIP_DISCORD_TOKEN`、`HERMESHIP_DEFAULT_CHANNEL`、`HERMESHIP_DRY_RUN`。
- 已将 `hermeship config path`、`hermeship config show`、`hermeship config verify` 从占位解析接入真实配置逻辑。
- 已运行 Red/Green：实现前 `cargo test config` 因缺少配置类型和路径函数失败，实现后通过。
- 已运行任务 1.2 验证命令：
  - `cargo fmt --all -- --check`
  - `cargo test config`
  - `cargo run -- config show`
  - `cargo test`
- 已提交：`50723af feat: 实现 hermeship 配置模型与 config CLI`。

### Milestone 1.3：质量门禁与仓库基础

- 已扩展 `.gitignore`：保留 `/target/`，新增本地编辑器临时文件、日志、临时目录、测试输出和覆盖率输出规则。
- 已确认 `.gitignore` 不忽略源码、文档、fixture 或 `Cargo.lock`。
- 已在 `README.md` 新增 Development Quality Gates，明确阶段提交前运行：
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- 已新增 fixture 目录：
  - `tests/fixtures/hermes/`
  - `tests/fixtures/privacy/`
  - `tests/fixtures/routes/`
  - `tests/fixtures/discord/`
- 已保留 `tests/fixtures/cli/` 和 `tests/fixtures/cli/public_commands.txt`。
- 已新增 `tests/fixtures/README.md`，明确 fixture 只能使用合成脱敏样例，不得包含真实 Discord token、真实 Hermes gateway 数据、真实 GitHub/tmux 状态、cookie、secret、完整 prompt、完整对话或 provider request/response body。
- 首次运行 `cargo clippy --all-targets -- -D warnings` 发现既有 lint：`AppConfig`/`MessageFormat` 的手写 `Default` 可 derive，CLI fixture 测试 helper 存在多余 `.into_iter()`；已用最小代码改动修复。
- 已运行任务 1.3 验证命令：
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- 已提交：`70c8f03 chore: 增加 Rust 质量门禁与仓库基础`。

## 未完成

- Milestone 2.1：`IncomingEvent` 与格式尚未开始。
  - `src/events.rs` 尚未创建。
  - `IncomingEvent`、`RoutingMetadata` 尚未实现。
  - 当前 `MessageFormat` 已存在于 `src/config.rs`，Milestone 2.1 需要先确定复用或重导出策略，避免产生两套格式 enum。
  - `emit` 参数解析尚未接入真实事件构造路径。
  - `tests/fixtures/hermes/agent_start.json`、`session_end.json`、`invalid_payload.json` 尚未创建。
- Milestone 2.2：Typed `EventEnvelope` 尚未实现。
- Milestone 2.3：隐私与 payload 清洗尚未实现。
- Milestone 3 到 Milestone 10 均未执行。
- daemon、client、typed event、隐私清洗、router、renderer、dispatcher、Discord sink、Hermes hook bridge、安装/回滚、release preflight、live verification 均未实现。
- live Discord verification 凭据是否可用尚未确认。
- Slack sink、git/GitHub/tmux parity 是否进入 `0.1.0` 尚未最终确认。
- macOS launchd 是否与 systemd 同期实现尚未最终确认。

## 下一步入口

从 `tasks/development-checklist.md` 的 **Milestone 2：事件模型与兼容层** 继续，优先执行 **任务 2.1：IncomingEvent 与格式**。

建议第一段工作：

1. 复习 `tasks/lessons.md`、本文、方案文档、开发清单和 `tasks/todo.md`。
2. 确认当前分支、最新提交和未提交变更：
   - `git status --short --branch`
   - `git log -3 --oneline`
3. 确认最新已完成阶段提交为 `70c8f03 chore: 增加 Rust 质量门禁与仓库基础`。
4. 读取当前相关代码与参考实现：
   - `src/config.rs`
   - `src/cli.rs`
   - `/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/events.rs`
   - `/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/event/compat.rs`
5. 从任务 2.1 继续，先写失败测试，再实现事件入口模型。
6. 注意本阶段只实现 `IncomingEvent`、格式解析、`emit` 参数解析和基础 Hermes fixture，不进入 daemon、typed envelope、隐私清洗、router、renderer、sink 或 hook bridge。
7. 运行任务 2.1 验证命令：
   - `cargo test events`
   - `cargo test cli`
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
- Milestone 0：契约与仓库基线已经完成并提交，提交为 af57c49 docs: 明确 hermeship 完整项目方向。
- Milestone 1.1：Cargo 项目与 CLI 入口已经完成并提交，提交为 d03170e chore: 搭建 Hermeship Rust CLI 骨架。
- Milestone 1.2：配置模型已经完成并提交，提交为 50723af feat: 实现 hermeship 配置模型与 config CLI。
- Milestone 1.3：质量门禁与仓库基础已经完成并提交，提交为 70c8f03 chore: 增加 Rust 质量门禁与仓库基础。
- 已创建 Rust 2024 工程骨架：Cargo.toml、Cargo.lock、src/lib.rs、src/main.rs、src/cli.rs、src/config.rs。
- 已实现最小 clap CLI 命令树：start、status、send、emit、explain、config、hermes、install、uninstall、release。
- 已实现配置模型：AppConfig、DaemonConfig、ProvidersConfig、DiscordConfig、DefaultsConfig、PrivacyConfig、HermesConfig、RouteRule、MessageFormat。
- 已实现默认配置路径 HERMESHIP_CONFIG 或 ~/.hermeship/config.toml。
- 已实现配置 TOML 加载、默认值、非法 TOML 错误、未知 key 忽略、空值归一化和环境变量覆盖。
- 已将 hermeship config path、hermeship config show、hermeship config verify 接入真实配置逻辑。
- 已扩展 .gitignore，新增 README 质量门禁说明，并创建 tests/fixtures/hermes、privacy、routes、discord 目录及 fixture 脱敏规则。
- 已通过验证：cargo fmt --all -- --check、cargo clippy --all-targets -- -D warnings、cargo test。
- Hermeship 目标是完全参考 /Users/zq/Desktop/ai-projs/posp/template/clawhip 的架构和功能形态，为 Hermes 做原生适配。
- 不是 thin adapter，不调用现有 clawhip runtime，也不依赖运行中的 clawhip daemon。
- 主实现语言是 Rust，采用 daemon-first 架构；Python 只用于 Hermes gateway hook bridge 的 handler.py 模板。
- 方案文档只维护架构和边界，执行进度维护在 tasks/development-checklist.md 和 tasks/todo.md。

请从 tasks/development-checklist.md 的 Milestone 2 继续，优先执行任务 2.1：IncomingEvent 与格式：
1. 先复习 tasks/lessons.md，并确认当前分支、最新提交和未提交变更：git status --short --branch、git log -3 --oneline。
2. 先更新或确认 tasks/todo.md 的本阶段计划。
3. 读取当前相关代码 src/config.rs、src/cli.rs，以及参考实现 /Users/zq/Desktop/ai-projs/posp/template/clawhip/src/events.rs 和 /Users/zq/Desktop/ai-projs/posp/template/clawhip/src/event/compat.rs。
4. 先写失败测试，再实现 src/events.rs，包含 IncomingEvent、RoutingMetadata，并处理 MessageFormat 的单一复用/重导出策略，避免出现两套不一致格式 enum。
5. 将 hermeship emit 的参数解析接入真实事件构造路径，覆盖 --payload、--channel、--mention、--format、--template 和 key/value 参数错误。
6. 新增 Hermes fixture：tests/fixtures/hermes/agent_start.json、session_end.json、invalid_payload.json，fixture 不得包含真实 Discord token、真实 Hermes gateway 数据、真实 GitHub/tmux 状态、cookie、secret、完整 prompt、完整对话或 provider request/response body。
7. 本阶段不要实现 daemon、typed EventEnvelope、privacy 清洗、router、renderer、sink、hook bridge、install 或 release preflight。
8. 运行任务 2.1 验证命令：cargo test events、cargo test cli、cargo fmt --all -- --check、cargo clippy --all-targets -- -D warnings、cargo test。
9. 更新 tasks/development-checklist.md 的运行状态日志和 tasks/todo.md 的 Review。
10. 阶段完成后必须验证并提交，commit 信息使用详细中文，说明变更、验证和影响。

开发过程中请遵守：
- 每完成一阶段任务就提交一次。
- 每次开始先复习 tasks/lessons.md。
- 方案文档只维护架构和边界，执行进度维护在 tasks/development-checklist.md 和 tasks/todo.md。
- 默认测试不得依赖真实 Discord token、真实 Hermes gateway、真实 GitHub、真实 tmux 或外网状态。
- 若出现偏差，先停止并重新规划，不要硬推。
```
