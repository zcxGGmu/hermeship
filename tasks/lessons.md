# Lessons

## 2026-06-15: 阶段完成后立即验证并提交

- 用户明确要求：每完成一阶段任务，就提交一次。
- 后续会话启动后，先复习 `tasks/lessons.md`，并自动沿用该习惯，不需要用户再次提醒。
- 阶段提交前必须完成相应验证，确认 `git status` 只包含预期变更。
- commit 信息使用中文，并写清楚本阶段完成内容、验证结果和后续影响。
- 不要把未验证、未完成或无关的工作混入阶段提交。

## 2026-06-15: 方案文档与开发清单必须分离

- 用户纠正：方案和清单需要分离开，并转换为中文文档。
- 方案文档只写目标、边界、架构、契约、安全、验证和发布策略。
- 开发清单只写可勾选任务、阶段进度、验证命令、阻塞项和决策记录。
- 后续不要在方案文档中塞入详细执行 checklist；需要跟踪进度时更新 `tasks/development-checklist.md`。

## 2026-06-15: Hermeship 目标不是薄适配器

- 用户纠正：Hermeship 应完全参照 `template/clawhip` 的项目形态、架构和功能实现，只是面向 Hermes 做对应适配。
- 后续不能把目标误解为“调用现有 clawhip 的轻量 adapter”。
- 正确理解：Hermeship 应是 Hermes-native 的事件到通知渠道路由项目，参考 clawhip 的 daemon-first 架构、事件模型、路由、渲染、sink、CLI、安装、配置和 live verification。
- 评估和计划应围绕“复刻/移植 clawhip 能力并替换 OpenClaw/Codex/Claude 耦合为 Hermes 耦合”展开。
