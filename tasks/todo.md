# Task: 集成 Hermeship 测试计划

- [x] 复习 `tasks/lessons.md`。
- [x] 将分层测试策略写入 `docs/plans/2026-06-15-hermeship-development-plan.md`。
- [x] 将测试执行规则写入 `tasks/development-checklist.md`。
- [x] 验证测试计划与 Rust daemon-first 方向一致。
- [x] 验证旧 Python/thin-adapter 方案没有回流。
- [x] 提交本阶段文档更新。

## Review

- 已在方案文档 `## 21. 测试策略` 中补入分层测试、测试矩阵、必备 fixture、不变量回归、CI/live 分离。
- 已在开发清单中补入测试执行规则、全局完成定义扩展，以及 CLI、隐私、daemon、dispatcher、Discord fake HTTP、Hermes hook bridge、release preflight、live verification 的阶段性测试门禁。
- 已验证旧 Python/thin-adapter 方案没有正文残留；清单中仅保留用于自检的 `rg` 命令文本。
- 已运行 `git diff --check`，无空白错误。
