# Task: 2026-06-23 README 信息架构优化

更新时间：2026-06-23

用户要求根据此前建议，对标 `gajae-code` 与 `Kocoro` 优化 README。本轮范围限定为中英文 README、状态记录和任务清单；不修改 Rust 功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 当前 HEAD：`0f2dad5 docs: 补充 README 设计原则`。
- 已复习：`tasks/lessons.md`。
- 已使用 `ui-ux-pro-max` 做 README 文档体验评估。
- 公开 README 必须保持 Hermeship 独立叙述，不写 `clawhip`、`template/clawhip`、thin adapter、runtime adapter。
- 默认完成后推送到远程 `main`。

## 本轮执行计划

- [x] 读取输入和约束。
  - 阅读：`tasks/lessons.md`。
  - 阅读：`README.md` 与 `README.en.md` 当前结构。
  - 复核：对标仓库 README 的首屏、目录、快速上手、能力说明和限制说明模式。
  - 运行：`ui-ux-pro-max/scripts/search.py ... --design-system`。

- [x] 优化中英文 README 信息架构。
  - 修改：`README.md`。
  - 修改：`README.en.md`。
  - 增加：目录 / Contents。
  - 增加：30 秒本地试跑 / 30-second local smoke。
  - 增加：能力矩阵 / Capability matrix。
  - 增加：工作流入口 / Workflow surface。
  - 增加：Known Limitations。
  - 增加：Troubleshooting。
  - 调整：图表分组和说明。
  - 要求：保留真实能力边界，不暗示 live pass 或真实 GitHub polling 已完成。

- [x] 更新状态记录。
  - 修改：`docs/development-status.md`。
  - 修改：`tasks/development-checklist.md`。
  - 修改：`tasks/todo.md` Review。

- [x] 运行验证。
  - 命令：`rg -n "目录|Contents|30 秒|30-second|能力矩阵|Capability Matrix|工作流入口|Workflow Surface|Known Limitations|Troubleshooting" README.md README.en.md`。
  - 命令：`rg -n "GitHub API polling|live verification|Slack sink|observer plugin|local deterministic|real live pass" README.md README.en.md`。
  - 命令：`rg -n -i "clawhip|template/clawhip|thin adapter|runtime adapter" README.md README.en.md`，预期无匹配。
  - 命令：`git diff --check`。
  - 命令：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 命令：`cargo fmt --all -- --check`。
  - 命令：`cargo test observer_plugin`。
  - 命令：`cargo test release_preflight`。
  - 命令：`cargo run -- release preflight 0.1.0`。
  - 命令：`cargo clippy --all-targets -- -D warnings`。
  - 命令：`cargo test`。

- [x] 阶段提交并推送。
  - commit 信息：中文说明 README 信息架构优化、验证和影响。
  - 推送：`origin/codex/milestone-1-cli` 与 `origin/main`。

## Review

- 已按用户确认的优化方向重排 `README.md` 和 `README.en.md`，新增目录、30 秒本地试跑、能力矩阵、工作流入口、已知限制和 Troubleshooting。
- 已将图表拆成架构总览、事件与路由、Observer 边界和联合工作流四组，避免连续堆图。
- 已将原“当前状态 / Current Capability Boundary”长列表改为可扫描的能力矩阵，并保留真实能力边界：真实 GitHub API polling 未实现、真实 Discord/Hermes live verification pass 未获得、Slack sink 不在默认范围、observer plugin 需显式启用。
- 已保留中英文 README 分文件维护，不混排；公开 README 未出现 `clawhip`、`template/clawhip`、thin adapter 或 runtime adapter。
- 已运行验证：README 目录和关键章节检查、关键能力边界声明检查、公开 README 关联词检查无匹配、`git diff --check`、`python3 -m py_compile templates/hermes-plugin/__init__.py`、`cargo fmt --all -- --check`、`cargo test observer_plugin`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
