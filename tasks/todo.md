# Task: Milestone 0 - 契约与仓库基线

- [x] 复习 `tasks/lessons.md`，确认 Hermeship 不是 thin adapter，阶段完成后必须验证并提交。
- [x] 确认当前分支、远程和未提交变更。
  - 命令：`git status --short --branch`
  - 当前：`main...origin/main [ahead 3]`，启动时无未提交变更。
- [x] 复核 clawhip 参考源码。
  - 路径：`/Users/zq/Desktop/ai-projs/posp/template/clawhip`
  - 文件：`ARCHITECTURE.md`、`Cargo.toml`、`src/cli.rs`、`src/main.rs`、`src/daemon.rs`、`src/events.rs`、`src/event/compat.rs`、`src/router.rs`、`src/render/default.rs`
  - 产出：记录可移植模块，以及 OpenClaw/Codex/Claude/OMC/OMX 相关耦合点必须替换为 Hermes 接入。
- [x] 复核 Hermes 参考源码。
  - 路径：`/Users/zq/Desktop/ai-projs/posp/agents-contributions/hermes-agent`
  - 文件：`gateway/hooks.py`、`gateway/run.py`、`gateway/slash_commands.py`、`hermes_cli/plugins.py`
  - 产出：确认 gateway hook 事件、context 字段、fail-open 语义，以及 plugin/observer 后续能力。
- [x] 更新 `README.md` 项目定位。
  - 明确 Hermeship 是 Hermes-native daemon-first event router。
  - 明确不调用现有 clawhip runtime，不依赖运行中的 clawhip daemon。
  - 明确 Rust 是主实现语言，Python 只用于 Hermes gateway hook bridge 的 `handler.py` 模板。
- [x] 更新 `tasks/development-checklist.md`。
  - 勾选 Milestone 0 已完成项。
  - 在运行状态日志顶部记录本阶段复核、README 更新、验证和提交状态。
- [x] 运行 Milestone 0 验证命令。
  - `rg -n "Hermes 到 clawhip 的适配层|通过 clawhip 已有 CLI 入口|python -m hermeship|src/hermeship|pyproject.toml|pytest|ruff|ClawhipClient|clawhip_client|HERMESHIP_CLAWHIP" docs/plans README.md`
  - `rg -n "Hermes 到 clawhip 的适配层|通过 clawhip 已有 CLI 入口|python -m hermeship|src/hermeship|pyproject.toml|pytest|ruff|ClawhipClient|clawhip_client|HERMESHIP_CLAWHIP" tasks/development-checklist.md | rg -v "rg -n"`
  - `git diff --check`
- [x] 提交 Milestone 0。
  - commit 信息使用详细中文，说明变更、验证和影响。

## Review

- 已复核 `template/clawhip` 的 daemon-first 参考形态：Rust CLI、Axum daemon、mpsc queue、typed event compat、dispatcher、multi-delivery router、renderer/sink 分离。
- 已复核 Hermes gateway hook 与 plugin 参考源码：MVP 使用 gateway hook bridge，observer plugin 后续推进。
- 已更新 `README.md`，明确 Hermeship 是 Hermes-native event router，不是 clawhip runtime client。
- 已更新 `tasks/development-checklist.md`，记录 Milestone 0 复核结论和运行状态。
- 已运行两条旧 Python/thin-adapter 方向过滤搜索，均无输出。
- 已运行 `git diff --check`，无空白错误。
- 本阶段只更新文档和任务跟踪，没有创建 Rust 工程文件。
