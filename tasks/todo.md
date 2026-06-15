# Task: Milestone 1.1 - Cargo 项目与 CLI 入口

- [ ] 复习 `tasks/lessons.md`，确认阶段完成后必须验证并提交。
- [ ] 确认当前分支、远程和未提交变更。
  - 命令：`git status --short --branch`
  - 完成标准：分支、远程和未提交变更清楚；不要混入无关改动。
- [ ] 参考 clawhip Rust 骨架。
  - 路径：`/Users/zq/Desktop/ai-projs/posp/template/clawhip`
  - 文件：`Cargo.toml`、`src/main.rs`、`src/cli.rs`
  - 完成标准：只参考架构和 CLI 形态，不复用或调用 clawhip runtime、binary、daemon。
- [ ] 新建 Cargo metadata。
  - 新建：`Cargo.toml`
  - 包含：package metadata、Rust 2024、依赖 `anyhow`、`tokio`、`axum`、`clap`、`serde`、`serde_json`、`toml`、`reqwest`、`time`、`uuid`。
- [ ] 新建基础源码文件。
  - 新建：`src/lib.rs`
  - 新建：`src/main.rs`
  - 新建：`src/cli.rs`
- [ ] 先写 CLI parse 测试。
  - 文件：`src/cli.rs`
  - 覆盖：`send`、`emit --payload`、`hermes hook --payload`、`hermes install-hooks`。
- [ ] 增加公开命令 fixture。
  - 新建：`tests/fixtures/cli/public_commands.txt`
  - 覆盖：`start`、`status`、`send`、`emit`、`explain`、`config`、`hermes hook`、`hermes install-hooks`、`install`、`uninstall`、`release preflight`。
- [ ] 实现最小 `hermeship --help`。
  - 子命令占位：`start`、`status`、`send`、`emit`、`explain`、`config`、`hermes`、`install`、`uninstall`、`release`。
- [ ] 运行任务 1.1 验证命令。
  - `cargo fmt --all -- --check`
  - `cargo test cli`
  - `cargo run -- --help`
- [ ] 更新 `tasks/development-checklist.md`。
  - 勾选任务 1.1 已完成项。
  - 在运行状态日志顶部记录本阶段实现、验证和提交状态。
- [ ] 提交任务 1.1。
  - commit：`chore: 搭建 hermeship Rust CLI 骨架`
  - commit 信息使用详细中文，说明变更、验证和影响。

## Review

- 待任务 1.1 实施和验证后填写。
