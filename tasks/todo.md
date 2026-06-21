# Task: 2026-06-21 README 顶部去重标题

更新时间：2026-06-21

用户要求删除图片下方重复出现的 `Hermeship` 文字。本轮范围只改 `README.md` 和 `README.en.md` 顶部：保留统一 banner、保留副标题和语言切换，并把 banner 的 `alt` 改为项目名以维持可访问性。不修改 Rust 功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 当前 HEAD：`f3e5afe docs: 优化 README 顶部品牌区`。
- 已复习：`tasks/lessons.md`。
- 目前 README 顶部只应保留 banner、副标题和语言切换，不再保留可见的 `Hermeship` 标题行。

## 本轮执行计划

- [x] 删除 README 顶部重复标题。
  - 文件：`README.md`
  - 文件：`README.en.md`
  - 结果：移除 banner 下方的可见 `Hermeship` 标题，保留 banner 与副标题。

- [x] 保留可访问性。
  - 结果：banner 的 `alt` 改为 `Hermeship`，避免读屏时完全丢失项目名入口。

- [x] 同步状态记录和 lessons。
  - 文件：`tasks/lessons.md`
  - 文件：`docs/development-status.md`
  - 文件：`tasks/development-checklist.md`
  - 文件：`tasks/todo.md`

- [x] 运行验证。
  - 命令：`sed -n '1,30p' README.md`
  - 命令：`sed -n '1,30p' README.en.md`
  - 命令：`rg -n "<h1 align=\"center\">Hermeship</h1>|alt=\"Hermeship\"|<table|img.shields.io" README.md README.en.md`
  - 命令：`rg -n -i "clawhip|template/clawhip|thin adapter|runtime adapter" README.md README.en.md`
  - 命令：`git diff --check`
  - 命令：`python3 -m py_compile templates/hermes-plugin/__init__.py`
  - 命令：`cargo fmt --all -- --check`
  - 命令：`cargo test observer_plugin`
  - 命令：`cargo test release_preflight`
  - 命令：`cargo run -- release preflight 0.1.0`
  - 命令：`cargo clippy --all-targets -- -D warnings`
  - 命令：`cargo test`

- [x] 阶段提交。
  - commit 信息：中文说明 README 顶部去重标题、验证和影响。

## Review

- 已删除 README 顶部 banner 下方重复的可见 `Hermeship` 标题行。
- 已保留统一 banner、语言切换和副标题；banner 现在使用 `alt="Hermeship"`，避免读屏丢失项目名。
- 已更新 `tasks/lessons.md`，新增“banner 已含项目名时不要重复标题”的规则。
- 本轮只修改公开 README、静态品牌语义和状态记录，不修改功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。
- 已运行验证：README 顶部检查、公开 README 关联词检查无匹配、`git diff --check`、`python3 -m py_compile templates/hermes-plugin/__init__.py`、`cargo fmt --all -- --check`、`cargo test observer_plugin`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
