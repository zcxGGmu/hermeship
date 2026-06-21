# Task: 2026-06-21 README 联合工作流图补充

更新时间：2026-06-21

用户要求在 README 中补充一张 `Hermeship + GitHub + Discord + Codex/OpenCode` 的联合工作流程图，并指定使用 `ui-ux-pro-max` 进行设计，动态性更强。本轮范围限定为 README、图表静态资产和状态记录；不修改 Rust 功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 当前 HEAD：`b4074ae docs: 删除 README 顶部重复标题`。
- 已复习：`tasks/lessons.md`。
- 已使用技能：`ui-ux-pro-max`、`fireworks-tech-graph`。
- 现有 README 图表资产位于 `docs/assets/diagrams/`，当前已有 architecture、event flow、observer framework 三组 JSON/SVG/PNG。

## 本轮执行计划

- [x] 读取设计与图表约束。
  - 读取：`tasks/lessons.md`。
  - 读取：`ui-ux-pro-max/SKILL.md`。
  - 读取：`fireworks-tech-graph/SKILL.md`。
  - 读取：`fireworks-tech-graph/references/style-6-claude-official.md`。
  - 读取：`fireworks-tech-graph/references/icons.md`。
  - 运行：`ui-ux-pro-max/scripts/search.py ... --design-system`。

- [x] 新增联合工作流图资产。
  - 新增：`docs/assets/diagrams/hermeship-github-discord-codex-workflow.json`。
  - 新增：`docs/assets/diagrams/hermeship-github-discord-codex-workflow.svg`。
  - 新增：`docs/assets/diagrams/hermeship-github-discord-codex-workflow.png`。
  - 要求：Style 6 暖色背景，动态流线、反馈回路、脉冲节点、步骤编号。
  - 要求：图中不暗示真实 GitHub polling 已完成；GitHub 路径标注为 local deterministic source / future API polling boundary。

- [x] 接入中英文 README。
  - 修改：`README.md`。
  - 修改：`README.en.md`。
  - 要求：放入现有“图表 / Diagrams”区域。
  - 要求：中英文说明分文件维护，不混排。

- [x] 更新状态记录。
  - 修改：`docs/development-status.md`。
  - 修改：`tasks/development-checklist.md`。
  - 修改：`tasks/todo.md` Review。

- [x] 运行验证。
  - 命令：`python3 -c "import json; ..."` 检查新增 JSON。
  - 命令：`python3 -c "import xml.etree.ElementTree as ET; ..."` 检查新增 SVG。
  - 命令：`file docs/assets/diagrams/hermeship-github-discord-codex-workflow.png`。
  - 命令：`rg -n "hermeship-github-discord-codex-workflow|Codex|OpenCode|Discord|GitHub" README.md README.en.md`。
  - 命令：`rg -n -i "clawhip|template/clawhip|thin adapter|runtime adapter" README.md README.en.md`，预期无匹配。
  - 命令：`git diff --check`。
  - 命令：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 命令：`cargo fmt --all -- --check`。
  - 命令：`cargo test observer_plugin`。
  - 命令：`cargo test release_preflight`。
  - 命令：`cargo run -- release preflight 0.1.0`。
  - 命令：`cargo clippy --all-targets -- -D warnings`。
  - 命令：`cargo test`。

- [x] 阶段提交。
  - commit 信息：中文说明 README 联合工作流图、验证和影响。

## Review

- 已新增联合工作流图资产：`docs/assets/diagrams/hermeship-github-discord-codex-workflow.json`、`.svg`、`.png`，PNG 为 1280 x 760。
- 已将联合工作流图接入 `README.md` 与 `README.en.md` 的图表区域，保持中英文分文件维护。
- 图表采用 Style 6 暖色技术图风格，并加入轨道流线、脉冲环、步骤编号和反馈弧线来表达更强动态性。
- 已保留真实能力边界：真实 GitHub API polling 仍是后续范围，当前 GitHub source 路径保持 local deterministic；本轮未执行真实 Discord/Hermes live check、未实现 Slack sink、未自动启用 Hermes observer plugin。
- 已运行验证：新增 JSON/SVG/PNG 检查、README 引用检查、公开 README 关联词检查无匹配、`git diff --check`、`python3 -m py_compile templates/hermes-plugin/__init__.py`、`cargo fmt --all -- --check`、`cargo test observer_plugin`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
