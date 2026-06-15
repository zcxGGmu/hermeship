# Task: 重写 Hermeship 方案与阶段清单

- [x] 复习 `tasks/lessons.md`。
- [x] 记录用户纠正：Hermeship 不是 thin adapter，而是 Hermes-native 的 clawhip-equivalent。
- [x] 重写 `docs/plans/2026-06-15-hermeship-development-plan.md`。
- [x] 重写 `tasks/development-checklist.md`。
- [x] 验证文档中不再保留“调用现有 clawhip”的旧方向。
- [x] 验证方案文档与开发清单继续分离维护。
- [ ] 提交本阶段文档更新。

## Review

- 方案文档已改为 Hermes-native daemon-first event router 方向，明确不依赖运行中的 `clawhip`。
- 开发清单已改为 Rust/clawhip-parity 实现路径，覆盖 CLI、daemon、event model、router、renderer、sink、Hermes hook bridge、安装运维、parity 扩展和 live verification。
- 已更新 lessons，记录 Hermeship 不是 thin adapter 的纠正。
- 已验证旧 Python/thin-adapter 方案正向残留和 diff 空白错误。
