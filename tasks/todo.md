# Task: 2026-06-21 README 项目图标接入

更新时间：2026-06-21

用户提供了一张图片，希望把它作为当前项目图标，放到 README 的合适位置。本轮范围限定为文档与静态资产修正：把图片复制进仓库，压缩为 README 友好的图标尺寸，在 `README.md` 和 `README.en.md` 顶部合适位置展示。默认不修改功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 启动时工作树：干净，当前 HEAD 为 `a9e4f0a docs: 移除 README 中的 clawhip 关联表述`。
- 已复习：`tasks/lessons.md`。
- 已确认图片路径：`/Users/zq/Desktop/ai-projs/os-notes/近期工作/AI工作流构建/hermes/2/ChatGPT Image 2026年6月4日 17_41_41 (2).png`。
- 已确认图片尺寸：1254 x 1254，正方形 PNG，适合做 README 顶部图标。

## 本轮执行计划

- [x] 确认 Git 状态、最近提交和图片属性。
  - 已运行：`git status --short --branch`。
  - 已运行：`git log -5 --oneline`。
  - 已运行：`sips -g pixelWidth -g pixelHeight <image>`。
  - 已运行：`file <image>`。

- [x] 复习项目 lessons。
  - 已读：`tasks/lessons.md`。

- [x] 写入 README 图标接入方案。
  - 修改：`README.md`。
  - 修改：`README.en.md`。
  - 修改：`docs/development-status.md`。
  - 修改：`tasks/development-checklist.md`。
  - 修改：`tasks/todo.md` Review。
  - 要求：图标图片必须复制到仓库内静态资产路径，README 不引用用户桌面上的外部路径。
  - 要求：图标放在 README 顶部、标题附近的合适位置，不能破坏中英文切换按钮和能力边界说明。

- [x] 生成仓库内图标资产并更新 README。
  - 新增资产：`docs/assets/branding/hermeship-icon.png`。
  - 做法：将原图压缩为 README 友好的正方形 PNG，再在中英文 README 中引用。
  - 要求：图标在页面顶部可见，但不遮挡语言切换按钮和标题。

- [x] 运行验证。
  - 命令：`rg -n -i "clawhip|template/clawhip|thin adapter|runtime adapter" README.md README.en.md`，预期无匹配。
  - 命令：`rg -n "hermeship-icon.png|img.shields.io|真实 Discord/Hermes live verification pass 尚未获得|Real Discord/Hermes live verification has not passed yet|Slack sink|observer plugin|deterministic source" README.md README.en.md`。
  - 命令：`file docs/assets/branding/hermeship-icon.png`。
  - 命令：`sips -g pixelWidth -g pixelHeight docs/assets/branding/hermeship-icon.png`。
  - 命令：`git diff --check`。
  - 命令：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 命令：`cargo fmt --all -- --check`。
  - 命令：`cargo test observer_plugin`。
  - 命令：`cargo test release_preflight`。
  - 命令：`cargo run -- release preflight 0.1.0`。
  - 命令：`cargo clippy --all-targets -- -D warnings`。
  - 命令：`cargo test`。

- [x] 阶段提交。
  - 提交前检查：`git status --short --branch`、`git diff --stat`、`git diff --name-only`。
  - commit 信息：中文说明 README 图标接入、资产处理、验证和影响。

## Review

- 已将用户提供的正方形 PNG 压缩为 512 x 512 的仓库内图标资产：`docs/assets/branding/hermeship-icon.png`。
- 已在 `README.md` 和 `README.en.md` 顶部标题下方、语言切换按钮上方加入居中项目图标，展示宽度为 180px。
- README 不引用用户桌面上的外部路径，远程仓库可直接加载仓库内资产。
- 已保留中英文切换按钮、Hermeship 独立项目叙述和关键能力边界声明。
- 本轮只修改 README、静态图片资产和状态记录，不修改功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。
- 已运行验证：`rg -n -i "clawhip|template/clawhip|thin adapter|runtime adapter" README.md README.en.md` 无匹配，README 图标引用和关键边界声明检查通过，`file docs/assets/branding/hermeship-icon.png` 确认为 PNG，`sips -g pixelWidth -g pixelHeight docs/assets/branding/hermeship-icon.png` 为 512 x 512，`python3 -m py_compile templates/hermes-plugin/__init__.py`，`cargo fmt --all -- --check`，`cargo test observer_plugin`（13 passed），`cargo test release_preflight`（16 passed），`cargo run -- release preflight 0.1.0`（9 checks ok，`live verification` 只证明记录字段存在），`cargo clippy --all-targets -- -D warnings`，`cargo test`（221 lib tests + 15 bin tests + doctests passed），`git diff --check`。
- 阶段提交前已确认 diff 范围仅包含 README 图标接入、图标资产和状态记录。
