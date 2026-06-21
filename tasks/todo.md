# Task: 2026-06-21 README 语言切换修正

更新时间：2026-06-21

用户反馈：中英文 README 混在一起，要求添加切换按钮。本轮范围限定为文档结构修正：根 `README.md` 保留中文入口，新增 `README.en.md` 作为英文入口，并在两个文件顶部添加语言切换按钮。默认不修改功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 启动时工作树：干净，当前 HEAD 为 `6a6dc62 docs: 增加 Hermeship 双语 README 与 Claude Official 图表`。
- 已复习：`tasks/lessons.md`。
- 已确认上一轮 README 将中文和英文放在同一文件中，阅读体验不符合用户预期。

## 本轮执行计划

- [x] 确认 Git 状态和当前 README 结构。
  - 已运行：`git status --short --branch`。
  - 已运行：`git log -3 --oneline`。
  - 已读：`README.md` 中文和英文段落。

- [x] 拆分 README 语言入口。
  - 修改：`README.md`。
  - 新增：`README.en.md`。
  - 要求：`README.md` 顶部提供中文/English 切换按钮，正文只保留中文。
  - 要求：`README.en.md` 顶部提供中文/English 切换按钮，正文只保留英文。
  - 要求：保留关键边界声明：真实 live pass 未完成、Slack 不默认实现、observer plugin 手动启用、source 命令 deterministic-only。

- [x] 更新状态记录。
  - 修改：`docs/development-status.md`。
  - 修改：`tasks/development-checklist.md`。
  - 修改：`tasks/todo.md` Review。

- [x] 运行验证。
  - 命令：`rg -n "README.en.md|img.shields.io|## English|## 中文" README.md README.en.md`。
  - 命令：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 命令：`cargo fmt --all -- --check`。
  - 命令：`cargo test observer_plugin`。
  - 命令：`cargo test release_preflight`。
  - 命令：`cargo run -- release preflight 0.1.0`。
  - 命令：`cargo clippy --all-targets -- -D warnings`。
  - 命令：`cargo test`。
  - 命令：`git diff --check`。

- [x] 阶段提交。
  - 提交前检查：`git status --short --branch`、`git diff --stat`、`git diff --name-only`。
  - commit 信息：中文说明 README 语言拆分、切换按钮、验证和影响。

## Review

- 已按用户反馈修正 README 多语言结构：根 `README.md` 保留中文入口，新增 `README.en.md` 作为英文入口，避免中英文混在同一长文中。
- 已在 `README.md` 和 `README.en.md` 顶部添加语言切换按钮，中文按钮指向 `README.md`，English 按钮指向 `README.en.md`。
- 已移除根 README 中多余的 `## 中文` 包装层，中文入口正文直接进入项目内容。
- 已更新 `tasks/lessons.md`，记录“README 多语言不要混排”的规则。
- 已保留关键能力边界声明：真实 Discord/Hermes live pass 未完成、`release preflight` 不证明真实 live pass、Slack sink 不在默认范围、observer plugin 需要手动启用、source 命令仍是 deterministic-only。
- 已更新 `docs/development-status.md` 和 `tasks/development-checklist.md`，记录本轮 README 语言切换修正。
- 本轮只修改文档结构和状态记录，不修改功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。
- 已运行验证：README 语言链接/混排检查通过，关键边界声明检查通过，`python3 -m py_compile templates/hermes-plugin/__init__.py`，`cargo fmt --all -- --check`，`cargo test observer_plugin`（13 passed），`cargo test release_preflight`（16 passed），`cargo run -- release preflight 0.1.0`（9 checks ok，`live verification` 只证明记录字段存在），`cargo clippy --all-targets -- -D warnings`，`cargo test`（221 lib tests + 15 bin tests + doctests passed），`git diff --check`。
- 阶段提交前已确认 diff 范围仅包含 README 拆分和状态记录。
