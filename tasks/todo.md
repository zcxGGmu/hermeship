# Task: Milestone 10.3 Observer Plugin Install/Enable CLI Automation

更新时间：2026-06-19

本轮目标是在已完成的 Milestone 10.2 可选 Hermes observer plugin scaffold 基础上，增加本地 deterministic 的 install/enable CLI 自动化入口。范围限定为 observer plugin 模板安装、dry-run、force 覆盖、marker-based 安全卸载或启用指引；不默认安装、不默认启用、不调用真实 Hermes CLI、不修改真实 Hermes config、不执行真实 Discord/Hermes live check、不实现 Slack sink。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 启动时分支确认：`git status --short --branch` 显示 `## codex/milestone-1-cli`。
- 最近 5 个提交：`5d4c534`、`f352222`、`eb64408`、`93aa9ec`、`0d0d354`。
- 最新状态文档提交：`5d4c534 docs: 更新 Hermeship 最新开发状态`。
- 最新 Milestone 10.2 功能阶段提交：`f352222 feat: 增加可选 Hermes observer plugin scaffold`。
- 最新 Milestone 10.1 契约研究提交：`93aa9ec docs: 完成 Hermes observer plugin 契约研究`。
- Milestone 9.3 已完成 `blocked`/`not_run` 记录；真实 Discord/Hermes live verification 仍未获得 `pass`。
- release preflight 的 `live verification` ok 只证明 `docs/live-verification.md` 必填字段存在，不证明真实 live pass。
- 本轮未提供 Discord credentials、测试频道、Hermes gateway 测试环境和明确执行确认，因此不执行真实 Discord/Hermes live check。

## 本轮执行计划

- [x] 复习 lessons、确认 Git 状态和最近提交。
  - 已读：`tasks/lessons.md`。
  - 已运行：`git status --short --branch`。
  - 已运行：`git log -5 --oneline`。

- [x] 阅读当前状态、文档和 release preflight 语义。
  - 已读：`docs/development-status.md`。
  - 已读：`tasks/development-checklist.md`。
  - 已读：`tasks/todo.md`。
  - 已读：`docs/observer-plugin.md`。
  - 已读：`docs/live-verification.md`。
  - 已读：`README.md`。
  - 已读：`ARCHITECTURE.md`。
  - 已读：`docs/operations.md`。
  - 已读：`docs/hermes-event-contract.md`。
  - 已读：`docs/plans/2026-06-15-hermeship-development-plan.md`。
  - 已读：`src/release_preflight.rs`。

- [x] 设计本轮最小范围并记录计划。
  - 方案：新增 `hermeship hermes install-plugin` 和 `hermeship hermes enable-plugin`。
  - `install-plugin` 只把 `templates/hermes-plugin/plugin.yaml` 与 `__init__.py` 安装到 `<HERMES_HOME>/plugins/hermeship-observer/`，支持 `--home`、`--force`、`--dry-run`，并写入 `.hermeship-managed.json` 供后续安全卸载扩展；安装器拒绝 symlinked plugin directory、模板文件和 marker 文件。
  - `enable-plugin` 不执行真实 `hermes plugins enable`，不修改 Hermes `config.yaml`；只输出 operator 应执行的 `hermes plugins enable hermeship-observer` 指令和目标 plugin 目录，用于避免默认自动启用。
  - 暂不实现 `uninstall-plugin`，除非实现过程中发现 marker lifecycle 无法完整验证；本轮优先保持最小影响。

- [x] 实现前 check-in。
  - 等待用户确认是否按上述 `install-plugin` + `enable-plugin` 最小方案执行。
  - 已确认：用户回复“继续”。

- [x] Red：新增失败测试。
  - 文件：`src/cli.rs`。
  - 覆盖：`hermes install-plugin --home /tmp/hermes --dry-run --force` 和 `hermes enable-plugin --home /tmp/hermes --dry-run` 能被 clap parse。
  - 文件：新建 `src/observer_plugin.rs` 或在合适模块增加测试。
  - 覆盖：dry-run 只报告路径、不写盘；install 写入 `plugin.yaml` 和 `__init__.py`；无 `--force` 不覆盖本地文件；`--force` 覆盖模板；marker 路径不能逃逸 plugin dir；plugin dir、模板文件和 marker symlink 均被拒绝。
  - 文件：`src/release_preflight.rs`、`tests/fixtures/cli/public_commands.txt`。
  - 覆盖：公开命令和文档命令缺失时 preflight 失败。
  - 预期：实现前 `cargo test observer_plugin`、`cargo test cli` 或 `cargo test release_preflight` 出现预期失败。
  - 已验证 Red：`cargo test observer_plugin` 实现前失败于缺少 observer plugin installer API；`cargo test cli` 实现前失败于 `main.rs` 未覆盖 `HermesCommands::InstallPlugin` 和 `HermesCommands::EnablePlugin`。

- [x] Green：实现 observer plugin installer 和 CLI 分发。
  - 新建：`src/observer_plugin.rs`。
  - 修改：`src/lib.rs` 导出 `observer_plugin`。
  - 修改：`src/cli.rs` 增加 `InstallPluginArgs`、`EnablePluginArgs` 和 `HermesCommands::{InstallPlugin, EnablePlugin}`。
  - 修改：`src/main.rs` 分发新命令并打印安装/启用指引。
  - 修改：`tests/fixtures/cli/public_commands.txt` 增加新公开命令。
  - 修改：`src/release_preflight.rs` 将新命令纳入 public/docs command 检查。
  - 已验证 Green：`cargo test observer_plugin`（13 passed，包含 symlink 安全回归和 stale marker 回归）、`cargo test cli`（25 lib-filtered + 1 bin-filtered passed）、`cargo test release_preflight`（15 passed）。

- [x] 更新文档状态。
  - 修改：`README.md` observer plugin 章节，说明 CLI 可安装模板但仍不自动启用。
  - 修改：`ARCHITECTURE.md` observer plugin 与 non-goals。
  - 修改：`docs/operations.md` observer plugin 安装/启用操作。
  - 修改：`docs/observer-plugin.md` follow-up 状态。
  - 修改：`docs/development-status.md` 和 `tasks/development-checklist.md` 运行状态日志。
  - 不新增真实 `docs/live-verification.md` pass 结果。

- [x] 运行验证。
  - 命令：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 命令：`cargo test observer_plugin`。
  - 命令：`cargo test release_preflight`。
  - 命令：`cargo run -- release preflight 0.1.0`。
  - 命令：`cargo fmt --all -- --check`。
  - 命令：`cargo clippy --all-targets -- -D warnings`。
  - 命令：`cargo test`。
  - 已验证：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 已验证：`cargo test observer_plugin`（13 passed，包含 symlink 安全回归和 stale marker 回归）。
  - 已验证：`cargo test release_preflight`（15 passed）。
  - 已验证：`cargo run -- release preflight 0.1.0`（9 checks ok；`live verification` ok 仍只证明文档字段存在）。
  - 已验证：`cargo fmt --all -- --check`。
  - 已验证：`cargo clippy --all-targets -- -D warnings`。
  - 已验证：`cargo test`（207 lib tests + 15 bin tests passed）。

- [x] 处理 code review blocker：observer plugin 安装器 symlink 写入风险。
  - 审查结论：`install-plugin --force` 和 marker 写入当前会跟随 symlink，可能覆盖 plugin 目录外文件。
  - 修复计划：先补 Unix-only 回归测试覆盖 plugin dir symlink、模板文件 symlink、marker symlink；再用 `symlink_metadata` 拒绝 symlink，并确保目标存在时必须是普通文件、plugin dir 必须是目录。
  - 额外覆盖：精确校验 `.hermeship-managed.json` 中的 managed entries 和 checksum，确保跳过的本地改动文件不会被误记录为 managed，并在全部文件都变成本地修改时清空旧 marker entries。
  - 已验证 Red：补充测试后 `cargo test observer_plugin` 失败于 3 个 symlink negative cases，旧实现返回 `Ok` 并写入 symlink 目标。
  - 已修复：`src/observer_plugin.rs` 先预检 plugin dir、模板文件和 marker，拒绝 symlink 与非普通文件；写入使用同目录临时文件后 rename，marker symlink 预检失败时不会先写入模板文件。
  - 已验证 Green：`cargo test observer_plugin`（13 passed）。

- [x] 更新 Review 并提交。
  - 更新本文件 Review：记录实际修改、验证结果、未完成事项和后续入口。
  - 提交前检查：`git status --short --branch`、`git diff --stat`、`git diff --name-only`。
  - commit 信息使用详细中文，说明变更、验证和影响。

## Review

- 已新增 `src/observer_plugin.rs`，实现可选 Hermes observer plugin 模板安装：dry-run 只报告目标路径，真实安装写入 `plugin.yaml`、`__init__.py` 和 `.hermeship-managed.json`，无 `--force` 保留本地文件，`--force` 覆盖为仓库模板。
- 已根据代码审查修复 observer plugin 安装器路径安全边界：plugin dir、模板文件和 marker 均使用 `symlink_metadata` 拒绝 symlink；目标存在时必须是普通文件；写入前先完成 marker 预检，避免 marker symlink 失败时产生部分模板写入；全部模板都被本地修改并跳过时会把旧 marker entries 清空。
- 已新增 `hermeship hermes install-plugin` 和 `hermeship hermes enable-plugin`；`enable-plugin` 只输出 `hermes plugins enable hermeship-observer` 指引，不执行真实 Hermes CLI，不修改 Hermes `config.yaml`，保持 operator opt-in。
- 已更新 public command fixture、release preflight、README、ARCHITECTURE、operations、observer contract、development status 和 development checklist，记录 10.3 完成范围与剩余边界。
- 已保留边界：本轮未执行真实 Discord/Hermes live check，未新增真实 live pass 记录，未实现 Slack sink，未新增 typed Rust observer body。
- 二次代码审查无 Critical / Important；已处理 stale marker minor。残余的 hostile-concurrent TOCTOU 场景需要目录 fd / `openat` 级别实现，超出当前本地 CLI threat model，未在本轮扩大范围。
- 已运行验证：`python3 -m py_compile templates/hermes-plugin/__init__.py`；`cargo test observer_plugin`（13 passed，包含 symlink 安全回归和 stale marker 回归）；`cargo test release_preflight`（15 passed）；`cargo run -- release preflight 0.1.0`（9 checks ok）；`cargo fmt --all -- --check`；`cargo clippy --all-targets -- -D warnings`；`cargo test`（207 lib tests + 15 bin tests passed）。
