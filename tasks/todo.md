# Task: Milestone 6 - Hermes Hook Bridge 安装

更新时间：2026-06-16 Milestone 5.3 已完成，Milestone 6 待执行

本阶段目标：让 Hermes gateway 能通过本地 hook bridge 将生命周期事件投递到 Hermeship。先实现可安装的 hook 模板和本地 deterministic handler smoke，再实现安装、卸载和回滚路径。

本阶段边界：只做本地 deterministic hook bridge 模板、安装器、handler fail-open smoke 和回滚测试；不实现 release preflight、真实 live verification、Slack sink、Hermes plugin/observer 或外网验证。默认测试不能依赖真实 Discord、真实 Hermes gateway、外网状态或真实凭据。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 最新功能阶段提交：`026e80c test: 增加 daemon 到 sink 的端到端覆盖`。
- 启动时应先确认工作树状态：`git status --short --branch`。
- 已完成 Milestone 0 到 Milestone 5.3。
- 已实现 daemon `/health`、`/event`、`/api/hermes/hook`、bounded queue、privacy sanitizer、DaemonClient health/event/hook POST、`hermeship start/status/emit/send/hermes hook`。
- 已实现 Router、DefaultRenderer、Dispatcher、Sink trait、FakeSink、daemon queue consumer、Discord sink 和本地 daemon -> fake sink smoke。
- 当前 Hermes hook bridge install、install/uninstall lifecycle、release preflight、真实 live verification、Slack sink 和 Hermes plugin/observer 仍未完成。

## 已完成

- [x] Milestone 0：契约与仓库基线。
- [x] Milestone 1.1：Cargo 项目与 CLI 入口。
- [x] Milestone 1.2：配置模型。
- [x] Milestone 1.3：质量门禁与仓库基础。
- [x] Milestone 2.1：IncomingEvent 与格式。
- [x] Milestone 2.2：Typed EventEnvelope。
- [x] Milestone 2.3：隐私与 payload 清洗。
- [x] Milestone 3.1：Daemon health 与 client。
- [x] Milestone 3.2：Event ingress 与队列。
- [x] Milestone 3.3：Hermes hook ingress。
- [x] Milestone 4.1：Router。
- [x] Milestone 4.2：Renderer。
- [x] Milestone 4.3：Dispatcher 与 fake sink。
- [x] Milestone 5.1：Discord 配置与 payload。
- [x] Milestone 5.2：Sink 失败语义。
- [x] Milestone 5.3：本地端到端 smoke。

## 当前待执行

- [ ] Milestone 6：Hermes Hook Bridge 安装。

## 后续未完成

- [ ] Milestone 7：安装、生命周期与运维 CLI。
- [ ] Milestone 8：clawhip 功能 Parity 扩展。
- [ ] Milestone 9：文档与 Live Verification。
- [ ] Milestone 10：Hermes Plugin / Observer 研究。

## 执行计划

- [ ] 复习项目规则与状态入口。
  - 阅读：`tasks/lessons.md`
  - 阅读：`docs/development-status.md`
  - 阅读：`docs/plans/2026-06-15-hermeship-development-plan.md`
  - 阅读：`tasks/development-checklist.md`
  - 阅读：`tasks/todo.md`

- [ ] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 预期：当前分支为 `codex/milestone-1-cli`；最新功能阶段提交为 `026e80c test: 增加 daemon 到 sink 的端到端覆盖`；启动时不要混入无关改动。

- [ ] 检查 Milestone 6 相关代码和参考文档。
  - 查看：`src/cli.rs`
  - 查看：`src/main.rs`
  - 查看：`src/hermes.rs`
  - 查看：`src/client.rs`
  - 查看：`src/daemon.rs`
  - 查看：`src/config.rs`
  - 查看：`tests/fixtures/README.md`
  - 查看：`docs/plans/2026-06-15-hermeship-development-plan.md` 的 Hermes Hook Bridge 章节。
  - 完成标准：确认本阶段只做本地 hook bridge 安装和 fail-open handler smoke，不进入 release preflight、真实 live verification 或 Slack sink。

- [ ] 任务 6.1：先写 Hook 模板失败测试。
  - 优先新增或修改：`src/hooks/mod.rs` 或合适的 hook 模板测试模块。
  - 覆盖：`templates/hermes-hook/HOOK.yaml` 可解析并声明 gateway/session/agent 事件。
  - 覆盖：`templates/hermes-hook/handler.py` 只使用 Python 标准库，不 import Hermeship package，不包含 secret。
  - 覆盖：handler 通过 stdin 调用 `hermeship hermes hook --payload -`，并具备 timeout/fail-open 逻辑。
  - 命令：`cargo test hooks`
  - 预期：实现前失败于缺少 hook 模板或 hooks 模块。

- [ ] 任务 6.1：实现 Hook 模板。
  - 新建：`templates/hermes-hook/HOOK.yaml`
  - 新建：`templates/hermes-hook/handler.py`
  - 行为：handler 读取 Hermes hook event/context，序列化 compact JSON，调用 `hermeship hermes hook --payload -`，捕获所有异常并 fail-open。
  - 完成标准：`cargo test hooks` 通过；模板不依赖真实 Hermes gateway 或外网。

- [ ] 任务 6.2：先写 Installer 失败测试。
  - 新增：`src/hooks/mod.rs`
  - 覆盖：首次安装、不覆盖、`--force` 覆盖、dry-run 不写磁盘、返回写入路径。
  - 使用：临时 fake Hermes home。
  - 命令：`cargo test hooks`

- [ ] 任务 6.2：实现 Installer 与 CLI。
  - 实现：`install_hermes_hooks(home, force)`。
  - CLI：`hermeship hermes install-hooks --home <path> --force`。
  - 支持 dry-run：打印将写入的文件，不修改磁盘。
  - 验证命令：`cargo test hooks`
  - 验证命令：`cargo run -- hermes install-hooks --home /tmp/hermeship-test-home --force`
  - 验证命令：`find /tmp/hermeship-test-home/hooks/hermeship -maxdepth 1 -type f -print`

- [ ] 任务 6.3：先写 Bridge smoke 与回滚失败测试。
  - 覆盖：fake `hermeship` binary 接收 stdin payload。
  - 覆盖：binary missing、调用 timeout、子进程失败时 handler fail-open。
  - 覆盖：安装 -> 卸载 -> 确认 hook 文件删除或 marker 删除。
  - 命令：`cargo test hooks`

- [ ] 任务 6.3：实现 uninstall/remove hooks。
  - CLI：`hermeship hermes uninstall-hooks --home <path>`。
  - 完成标准：可回滚 fake Hermes home，不误删非 Hermeship 文件。
  - 验证命令：`cargo test hooks`
  - 验证命令：`cargo run -- hermes uninstall-hooks --home /tmp/hermeship-test-home`

- [ ] 运行 Milestone 6 验证命令。
  - `cargo test hooks`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

- [ ] 更新开发状态文档。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 必要时更新：`docs/development-status.md`
  - 完成标准：记录实现、验证、边界和剩余风险，并把下一入口切到 Milestone 7 或 Milestone 6 未完成子任务。

- [ ] 提交 Milestone 6 对应任务。
  - 任务 6.1 commit：`feat: 增加 Hermes hook bridge 模板`
  - 任务 6.2 commit：`feat: 支持安装 Hermes gateway hooks`
  - 任务 6.3 commit：`feat: 支持 Hermes hook 回滚`
  - commit 信息使用中文正文，说明变更、验证和影响。

## Review

- 待 Milestone 6 实施、验证和提交后填写。
- 上一阶段 Milestone 5.3 已完成并提交：`026e80c test: 增加 daemon 到 sink 的端到端覆盖`。
