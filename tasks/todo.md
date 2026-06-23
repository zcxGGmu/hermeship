# Task: 2026-06-23 README 信息架构优化

更新时间：2026-06-23

用户要求根据此前建议，对标 `gajae-code` 与 `Kocoro` 优化 README。本轮范围限定为中英文 README、状态记录和任务清单；不修改 Rust 功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 当前 HEAD：`0f2dad5 docs: 补充 README 设计原则`。
- 已复习：`tasks/lessons.md`。
- 已使用 `ui-ux-pro-max` 做 README 文档体验评估。
- 公开 README 必须保持 Hermeship 独立叙述，不写历史关联表述或适配器叙述。
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
  - 命令：按 `tasks/lessons.md` 的公开叙述禁用词做 README 扫描，预期无匹配。
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
- 已保留中英文 README 分文件维护，不混排；公开 README 未出现历史关联表述或适配器叙述。
- 已运行验证：README 目录和关键章节检查、关键能力边界声明检查、公开 README 关联词检查无匹配、`git diff --check`、`python3 -m py_compile templates/hermes-plugin/__init__.py`、`cargo fmt --all -- --check`、`cargo test observer_plugin`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。

---

# Task: 2026-06-23 Hermeship 静态官网设计

更新时间：2026-06-23

用户要求参考 `https://gajae-code.com/` 的风格样式，为当前 Hermeship 项目设计类似网站。本轮先做规划和确认；在用户确认前不修改站点实现文件。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 当前仓库形态：Rust CLI/daemon 项目，无现成前端构建系统或网站目录。
- 已复习：`tasks/lessons.md`。
- 公开材料必须保持 Hermeship 独立叙述，不写历史关联表述或适配器叙述。
- 默认完成阶段后需要验证、提交，并同步到远程 `main`，除非用户另有指示。

## 参考站点观察

- `gajae-code.com` 是纯 HTML/CSS/JS 静态站，无构建步骤。
- 视觉语言：深色开发者工具站、固定导航、强首屏、渐变品牌字、命令行终端块、分段卡片、方法论流程、能力网格、架构/安装/文档入口。
- 不应复制品牌元素、文案或红黑色系；Hermeship 应使用自身 brand lockup、icon、架构图和 Hermes-native 叙述。

## 拟定范围

- [x] 写入静态官网设计 spec。
  - 位置：`docs/superpowers/specs/2026-06-23-hermeship-static-website-design.md`
  - 说明：单页静态站、中文为主、复用现有品牌和图表资产、拒绝前端框架。
- [x] 确认网站形态：新增独立静态站点目录 `site/`，不引入 npm/Vite/Next/Astro。
- [x] 设计首页信息架构：Hero、30 秒试跑、ingest/normalize/scrub/route/render/deliver 工作流、能力矩阵、架构图、安装配置、验证边界、Footer。
- [x] 复用现有资产：`docs/assets/branding/*` 和 `docs/assets/diagrams/*.png`。
- [x] 使用纯 CSS 设计系统：深色背景、品牌金黄/琥珀、青绿运行态、冷蓝辅助和系统字体/等宽 fallback。
- [x] 实现响应式与可访问性：移动端导航、可见 focus、减少动画支持、图像 alt、无横向滚动。
- [x] 验证静态站点：本地打开或启动轻量静态服务器，使用浏览器检查桌面和移动端截图，运行 `git diff --check`。
- [ ] 记录 Review：在本文件末尾追加实现结果、验证命令和边界说明。

## 待确认

- 推荐方案：单页静态官网，放在 `site/index.html`、`site/css/styles.css`、`site/js/main.js`，通过现有 README 和图表资产讲清 Hermeship 的运行边界。
- 替代方案 A：把网站作为 `docs/index.html` 放进文档目录，路径更集中，但容易和现有 Markdown docs 混杂。
- 替代方案 B：引入 Astro/Vite 做可扩展站点，后续多页更方便，但会给当前 Rust 项目增加前端依赖和维护成本。

## Review

- 静态官网已实现于 `site/`，包含 `index.html`、`css/styles.css`、`js/main.js` 和站内 `assets/`。
- 已复用现有品牌和图表资产；为保证 `site/` 可作为独立静态根服务，已复制所需 PNG 到 `site/assets/` 并改用站内相对路径。
- 已实现 Hero、30 秒本地试跑、ingest/normalize/scrub/route/render/deliver 工作流、能力矩阵、架构图、运行边界和 Footer。
- 已明确真实边界：Observer plugin 需要显式安装并手动启用、Slack sink 不在默认范围、GitHub API polling 仍是后续范围、Real Discord/Hermes live verification pass 未获得。
- 已根据代码审查修复导航可访问性问题：移动端关闭态使用 `visibility: hidden` 且同步 `aria-hidden`；桌面端可见导航移除 `aria-hidden`；并将 Discord sink 状态改为“需配置”。
- 已验证：本地静态服务 `http://127.0.0.1:4187/`，桌面 1440x1000、移动 390x844 渲染，移动端导航打开/关闭，复制按钮，当前章节高亮，图像加载，无横向溢出，边界声明可见。
- 已运行静态检查：关键内容扫描、禁用公开关联表述扫描、CSS 圆角/装饰项扫描、JS hook 扫描和 `git diff --check`。
- 未引入前端构建系统，未修改 Rust 功能代码。
