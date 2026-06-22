# Task: 2026-06-22 README 设计原则补充

更新时间：2026-06-22

用户要求基于指定文章的核心设计思想补充 README，并特别要求不要以转述口吻写。本轮范围限定为公开 README、状态记录和 lessons；不修改 Rust 功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 当前 HEAD：`8e035c2 docs: 记录默认推送主分支偏好`。
- 已复习：`tasks/lessons.md`。
- 公开 README 仍必须保持 Hermeship 独立叙述，不写外部参考来源，不写 `clawhip`、`template/clawhip`、thin adapter、runtime adapter。
- 默认完成后推送到远程 `main`。

## 本轮执行计划

- [x] 阅读输入材料与现有规则。
  - 阅读：`tasks/lessons.md`。
  - 阅读：用户指定文章。
  - 阅读：`README.md` 与 `README.en.md` 当前结构。

- [x] 补充中英文 README。
  - 修改：`README.md`。
  - 修改：`README.en.md`。
  - 要求：使用 Hermeship 自身口吻写设计原则，不写成“文章认为/参考某项目”的转述。
  - 要求：保持中英文分文件维护，不混排。
  - 要求：保留真实能力边界声明。

- [x] 更新状态记录。
  - 修改：`docs/development-status.md`。
  - 修改：`tasks/development-checklist.md`。
  - 如有必要，修改：`tasks/lessons.md`。
  - 修改：`tasks/todo.md` Review。

- [x] 运行验证。
  - 命令：`rg -n -i "clawhip|template/clawhip|thin adapter|runtime adapter|what you need|article|blog|based on|参考" README.md README.en.md`，预期无匹配。
  - 命令：`rg -n "设计原则|Design Principles|GitHub API polling|live verification|Slack sink|observer plugin|local deterministic" README.md README.en.md`。
  - 命令：`git diff --check`。
  - 命令：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 命令：`cargo fmt --all -- --check`。
  - 命令：`cargo test observer_plugin`。
  - 命令：`cargo test release_preflight`。
  - 命令：`cargo run -- release preflight 0.1.0`。
  - 命令：`cargo clippy --all-targets -- -D warnings`。
  - 命令：`cargo test`。

- [x] 阶段提交并推送。
  - commit 信息：中文说明 README 设计原则补充、验证和影响。
  - 推送：`origin/codex/milestone-1-cli` 与 `origin/main`。

## Review

- 已阅读用户指定文章，并将其核心思想转化为 Hermeship 自身的 README 设计原则，没有在公开 README 中写外部文章或参考项目转述。
- 已在 `README.md` 增加 `设计原则`，在 `README.en.md` 增加 `Design Principles`，保持中英文分文件维护。
- 新增内容强调 Hermeship 是独立协作控制面：通知逻辑离开 agent 上下文，人负责方向和工程判断，daemon 负责 typed、可解释、可失败的事件反馈闭环。
- 已保留真实能力边界：真实 GitHub/API/tmux/cron/live 路径仍需显式 operator 控制；本轮未执行真实 Discord/Hermes live check、未实现 Slack sink、未自动启用 Hermes observer plugin。
- 已更新 `tasks/lessons.md`，记录“README 吸收外部思想时必须项目原生化”的规则。
- 已运行验证：公开 README 外部转述/关联词检查无匹配、设计原则与关键边界声明检查、`git diff --check`、`python3 -m py_compile templates/hermes-plugin/__init__.py`、`cargo fmt --all -- --check`、`cargo test observer_plugin`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
