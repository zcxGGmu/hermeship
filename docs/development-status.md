# Hermeship 开发状态

最后更新：2026-06-15 19:26:15 CST

本文是下次启动 Codex 会话时的状态入口。执行开发前仍以 `tasks/development-checklist.md` 的 checkbox 为准；当前阶段计划维护在 `tasks/todo.md`。

## 当前结论

- Hermeship 的目标已经锁定：完全参考 `/Users/zq/Desktop/ai-projs/posp/template/clawhip` 的项目形态、架构和功能，只把 OpenClaw/Codex/Claude/OMC/OMX 等耦合替换为 Hermes 适配。
- Hermeship 不是调用现有 `clawhip` 的 thin adapter，也不依赖运行中的 `clawhip` daemon。
- 主实现语言确定为 Rust，采用 daemon-first 架构；Python 只用于 Hermes gateway hook bridge 模板 `handler.py`。
- 方案文档与执行清单已经拆分：方案文档维护架构和边界，`tasks/development-checklist.md` 和 `tasks/todo.md` 维护可勾选进度。
- 默认测试策略已经确定：使用本地 fixture、fake sink、fake HTTP、fake Hermes home、fake hermeship binary；真实 Discord/Hermes 只进入 live verification。
- 当前开发分支：`codex/milestone-1-cli`。
- 当前最新阶段提交：`d03170e chore: 搭建 Hermeship Rust CLI 骨架`。

## 已完成

- 已记录项目习惯：每完成一阶段任务，必须验证并提交；后续会话启动时先复习 `tasks/lessons.md`。
- 已重写方案文档：`docs/plans/2026-06-15-hermeship-development-plan.md`。
- 已重写阶段性开发清单：`tasks/development-checklist.md`。
- 已将测试计划集成到方案文档和开发清单。
- 已完成 Milestone 0：契约与仓库基线。
  - 已复核 `template/clawhip` 指定参考文件，确认可移植形态为 Rust CLI、daemon、typed event、dispatcher、multi-delivery router、renderer/sink split、config/lifecycle/release preflight。
  - 已复核 Hermes gateway hook 与 plugin 参考源码，确认 MVP 先使用 gateway hook bridge，plugin/observer 后续推进。
  - 已更新 `README.md`，明确 Hermeship 是 Hermes-native daemon-first event router，不是 clawhip runtime client。
  - 已运行旧 Python/thin-adapter 方向过滤搜索，正文无旧方案残留。
- 已完成 Milestone 1.1：Cargo 项目与 CLI 入口。
  - 已创建 Rust 2024 工程骨架：`Cargo.toml`、`Cargo.lock`、`src/lib.rs`、`src/main.rs`、`src/cli.rs`。
  - 已新增最小 `.gitignore`，忽略 `/target/`，避免验证产物进入提交。
  - 已实现最小 `clap` CLI 命令树：`start`、`status`、`send`、`emit`、`explain`、`config`、`hermes`、`install`、`uninstall`、`release`。
  - 已新增 CLI parse 单元测试，覆盖 `send`、`emit --payload`、`hermes hook --payload`、`hermes install-hooks`。
  - 已新增公开命令 fixture：`tests/fixtures/cli/public_commands.txt`，并断言必备公开命令前缀存在。
  - 已运行 Red/Green：实现前 `cargo test cli` 因缺少 CLI 类型失败，实现后通过。
  - 已运行任务 1.1 验证命令：
    - `cargo fmt --all -- --check`
    - `cargo test cli`
    - `cargo run -- --help`
  - 已提交：`d03170e chore: 搭建 Hermeship Rust CLI 骨架`。
- 已完成并提交以下阶段性提交：
  - `d69dbb4 docs: 重写 hermeship 完整项目方案`
  - `9771968 docs: 集成 hermeship 测试计划`
  - `71fe032 docs: 更新 hermeship 开发状态`
  - `af57c49 docs: 明确 hermeship 完整项目方向`
  - `880c0b1 docs: 更新 Milestone 1 续接状态`
  - `d03170e chore: 搭建 Hermeship Rust CLI 骨架`

## 未完成

- Milestone 1.2：配置模型尚未开始。
  - 尚未创建 `src/config.rs`。
  - 尚未实现 `AppConfig`、`DaemonConfig`、`ProvidersConfig`、`DiscordConfig`、`DefaultsConfig`、`PrivacyConfig`、`HermesConfig`、`RouteRule`。
  - 尚未实现默认配置路径 `~/.hermeship/config.toml` 和环境变量 `HERMESHIP_CONFIG`。
  - 尚未实现 TOML 加载、默认值、非法 TOML 错误、未知 key 处理、空值归一化。
  - 现有 `config` CLI 仍是 Milestone 1.1 的占位解析，尚未接入真实配置逻辑。
- Milestone 1.3：质量门禁与仓库基础尚未开始。
  - `.gitignore` 目前只有 `/target/`，完整临时日志/测试输出规则尚未扩展。
  - rustfmt/clippy 约束说明尚未补充到 README 或开发文档。
  - `tests/fixtures/hermes/`、`tests/fixtures/privacy/`、`tests/fixtures/routes/`、`tests/fixtures/discord/` 等 fixture 目录尚未创建。
- Milestone 2 到 Milestone 10 均未执行。
- daemon、client、事件模型、隐私清洗、router、renderer、dispatcher、Discord sink、Hermes hook bridge、安装/回滚、release preflight、live verification 均未实现。
- live Discord verification 凭据是否可用尚未确认。
- Slack sink、git/GitHub/tmux parity 是否进入 `0.1.0` 尚未最终确认。
- macOS launchd 是否与 systemd 同期实现尚未最终确认。

## 下一步入口

从 `tasks/development-checklist.md` 的 **Milestone 1：Rust 项目骨架与质量门禁** 继续，优先执行 **任务 1.2：配置模型**。

建议第一段工作：

1. 复习 `tasks/lessons.md`、本文、方案文档、开发清单和 `tasks/todo.md`。
2. 确认当前分支和未提交变更：`git status --short --branch`。
3. 更新 `tasks/todo.md` 为 Milestone 1.2 的本阶段计划。
4. 参考 `/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/config.rs` 和现有 Hermeship CLI 结构，但不要复用 clawhip runtime 或调用 clawhip daemon。
5. 先写配置模型测试，再实现最小配置模块。
6. 实现 `hermeship config path`、`hermeship config show`、`hermeship config verify` 的真实配置逻辑。
7. 运行任务 1.2 验证命令：
   - `cargo test config`
   - `cargo run -- config show`
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
- Milestone 0：契约与仓库基线已经完成并提交。
- Milestone 1.1：Cargo 项目与 CLI 入口已经完成并提交，最新阶段提交是 d03170e chore: 搭建 Hermeship Rust CLI 骨架。
- 已创建 Rust 2024 工程骨架：Cargo.toml、Cargo.lock、src/lib.rs、src/main.rs、src/cli.rs。
- 已实现最小 clap CLI 命令树：start、status、send、emit、explain、config、hermes、install、uninstall、release。
- 已新增 CLI parse 测试和 tests/fixtures/cli/public_commands.txt。
- 已通过验证：cargo fmt --all -- --check、cargo test cli、cargo run -- --help。
- Hermeship 目标是完全参考 /Users/zq/Desktop/ai-projs/posp/template/clawhip 的架构和功能形态，为 Hermes 做原生适配。
- 不是 thin adapter，不调用现有 clawhip runtime，也不依赖运行中的 clawhip daemon。
- 主实现语言是 Rust，采用 daemon-first 架构；Python 只用于 Hermes gateway hook bridge 的 handler.py 模板。
- 方案文档只维护架构和边界，执行进度维护在 tasks/development-checklist.md 和 tasks/todo.md。

请从 tasks/development-checklist.md 的 Milestone 1 继续，优先执行任务 1.2：配置模型：
1. 先更新 tasks/todo.md 写明本阶段计划。
2. 参考 /Users/zq/Desktop/ai-projs/posp/template/clawhip/src/config.rs 和现有 Hermeship CLI 结构，但不要复用 clawhip runtime 或调用 clawhip daemon。
3. 先写配置模型测试，再实现 src/config.rs。
4. 配置模型至少包含 AppConfig、DaemonConfig、ProvidersConfig、DiscordConfig、DefaultsConfig、PrivacyConfig、HermesConfig、RouteRule。
5. 实现默认配置路径 ~/.hermeship/config.toml 和环境变量 HERMESHIP_CONFIG。
6. 实现默认配置与 TOML 加载：缺失配置返回默认值，非法 TOML 返回错误，覆盖 env override、未知 key、空值归一化。
7. 将 config CLI 从占位解析接入真实逻辑：hermeship config path、hermeship config show、hermeship config verify。
8. 运行任务 1.2 验证命令：cargo test config、cargo run -- config show。
9. 更新 tasks/development-checklist.md 的运行状态日志和 tasks/todo.md 的 Review。
10. 阶段完成后必须验证并提交，commit 信息使用详细中文，说明变更、验证和影响。

开发过程中请遵守：
- 每完成一阶段任务就提交一次。
- 每次开始先复习 tasks/lessons.md。
- 方案文档只维护架构和边界，执行进度维护在 tasks/development-checklist.md 和 tasks/todo.md。
- 默认测试不得依赖真实 Discord token、真实 Hermes gateway、真实 GitHub、真实 tmux 或外网状态。
- 若出现偏差，先停止并重新规划，不要硬推。
```
