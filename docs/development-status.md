# Hermeship 开发状态

最后更新：2026-06-15 16:52:38 CST

本文是下次启动 Codex 会话时的状态入口。执行开发前仍以 `tasks/development-checklist.md` 的 checkbox 为准。

## 当前结论

- Hermeship 的目标已经明确：完全参考 `/Users/zq/Desktop/ai-projs/posp/template/clawhip` 的项目形态、架构和功能，只把 OpenClaw/Codex/Claude 等耦合替换为 Hermes 适配。
- Hermeship 不是调用现有 `clawhip` 的 thin adapter，也不依赖运行中的 `clawhip` daemon。
- 主实现语言已经确定为 Rust，采用 daemon-first 架构；Python 只用于 Hermes gateway hook bridge 模板 `handler.py`。
- 方案文档与执行清单已经拆分：方案文档维护架构和边界，`tasks/development-checklist.md` 维护可勾选进度。
- 测试策略已经集成：默认测试使用本地 fixture/fake sink/fake HTTP/fake Hermes home，真实 Discord/Hermes 只进入 live verification。

## 已完成

- 已记录项目习惯：每完成一阶段任务，必须验证并提交；后续会话启动时先复习 `tasks/lessons.md`。
- 已重写方案文档：`docs/plans/2026-06-15-hermeship-development-plan.md`。
- 已重写阶段性开发清单：`tasks/development-checklist.md`。
- 已将测试计划集成到方案文档和开发清单。
- 已确认旧 Python/thin-adapter 方向没有正文残留。
- 已完成并提交以下阶段性文档提交：
  - `d69dbb4 docs: 重写 hermeship 完整项目方案`
  - `9771968 docs: 集成 hermeship 测试计划`

## 未完成

- Rust 工程实现尚未开始。
- `Cargo.toml`、`src/`、`tests/fixtures/`、`templates/`、`deploy/` 等实现文件尚未创建。
- `README.md` 仍是最小占位内容，尚未更新为正式项目定位和使用说明。
- `tasks/development-checklist.md` 中 Milestone 0 到 Milestone 10 尚未正式执行。
- daemon、CLI、配置模型、事件模型、隐私清洗、router、renderer、dispatcher、Discord sink、Hermes hook bridge、安装/回滚、live verification 均未实现。
- live Discord verification 凭据是否可用尚未确认。
- Slack sink、git/GitHub/tmux parity 是否进入 `0.1.0` 尚未最终确认。
- macOS launchd 是否与 systemd 同期实现尚未最终确认。

## 下一步入口

从 `tasks/development-checklist.md` 的 **Milestone 0：契约与仓库基线** 开始。

建议第一段工作：

1. 复习 `tasks/lessons.md`、本文、方案文档和开发清单。
2. 更新 `tasks/todo.md` 为 Milestone 0 当前任务。
3. 执行 Milestone 0：确认分支、复核 clawhip/Hermes 参考源码、更新 README 项目定位。
4. 运行 Milestone 0 验证命令，确认旧 Python/thin-adapter 方向没有回流。
5. 更新 `tasks/development-checklist.md` 的运行状态日志。
6. 使用详细中文 commit 信息提交 Milestone 0。

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
- Hermeship 目标是完全参考 /Users/zq/Desktop/ai-projs/posp/template/clawhip 的架构和功能形态，为 Hermes 做原生适配。
- 不是 thin adapter，不调用现有 clawhip runtime，也不依赖运行中的 clawhip daemon。
- 主实现语言是 Rust，采用 daemon-first 架构；Python 只用于 Hermes gateway hook bridge 的 handler.py 模板。
- 方案和测试计划已经完成并提交，但 Rust 实现尚未开始。

请从 tasks/development-checklist.md 的 Milestone 0 开始继续：
1. 先更新 tasks/todo.md 写明本阶段计划。
2. 复核 clawhip 和 Hermes 参考源码。
3. 更新 README 项目定位。
4. 运行 Milestone 0 的验证命令，确认旧 Python/thin-adapter 方向没有回流。
5. 更新 tasks/development-checklist.md 的运行状态日志。
6. 阶段完成后必须验证并提交，commit 信息使用详细中文，说明变更、验证和影响。

开发过程中请遵守：
- 每完成一阶段任务就提交一次。
- 每次开始先复习 tasks/lessons.md。
- 方案文档只维护架构和边界，执行进度维护在 tasks/development-checklist.md 和 tasks/todo.md。
- 默认测试不得依赖真实 Discord token、真实 Hermes gateway、真实 GitHub、真实 tmux 或外网状态。
- 若出现偏差，先停止并重新规划，不要硬推。
```
