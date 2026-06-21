# Task: 2026-06-21 README HERMES-HIP 艺术字接入

更新时间：2026-06-21

用户希望参考 `hermes-agent` README 最开头的项目横幅艺术字，为 Hermeship 增加 `HERMES-HIP` 艺术字，并把艺术字放在当前项目图标的左侧。本轮范围限定为 README 和静态品牌资产修正：新增仓库内 wordmark 资产，更新 `README.md` 和 `README.en.md` 顶部布局。默认不修改功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 启动时工作树：干净，当前 HEAD 为 `9349330 docs: 为 README 接入 Hermeship 项目图标`。
- 已复习：`tasks/lessons.md`。
- 已确认参考项目 README 顶部使用 `assets/banner.png` 横幅艺术字。
- 已确认参考横幅尺寸：1145 x 196，黑底、黄到橙色、像素块风格。
- 当前项目已有图标资产：`docs/assets/branding/hermeship-icon.png`，512 x 512 PNG。

## 本轮执行计划

- [x] 确认 Git 状态、最近提交和参考 banner 风格。
  - 已运行：`git status --short --branch`。
  - 已运行：`git log -5 --oneline`。
  - 已运行：`curl -L .../README.md | sed -n '1,80p'`。
  - 已运行：`curl -L .../assets/banner.png -o /tmp/hermes-agent-banner.png`。
  - 已运行：`file /tmp/hermes-agent-banner.png`。
  - 已运行：`sips -g pixelWidth -g pixelHeight /tmp/hermes-agent-banner.png`。

- [x] 复习项目 lessons。
  - 已读：`tasks/lessons.md`。
  - 约束：公开 README 仍不能出现 `clawhip` / `template/clawhip` / adapter 关联表述。

- [x] 新增 HERMES-HIP wordmark 资产。
  - 新增：`docs/assets/branding/hermeship-wordmark.svg`。
  - 要求：文字为 `HERMES-HIP`。
  - 要求：风格参考 hermes-agent 顶部横幅：黑底、黄/橙高对比、像素/块状感。
  - 要求：仓库内自包含，不引用外部图片、字体或远程资源。

- [x] 更新 README 顶部布局。
  - 修改：`README.md`。
  - 修改：`README.en.md`。
  - 要求：艺术字放在项目图标左侧，形成横向品牌头部。
  - 要求：保留语言切换按钮、Hermeship 独立项目叙述和关键能力边界声明。

- [x] 更新状态记录。
  - 修改：`docs/development-status.md`。
  - 修改：`tasks/development-checklist.md`。
  - 修改：`tasks/todo.md` Review。

- [x] 运行验证。
  - 命令：`python3 - <<'PY' ... ElementTree.parse('docs/assets/branding/hermeship-wordmark.svg') ... PY`。
  - 命令：`rg -n "hermeship-wordmark.svg|hermeship-icon.png|img.shields.io" README.md README.en.md`。
  - 命令：`rg -n -i "clawhip|template/clawhip|thin adapter|runtime adapter" README.md README.en.md`，预期无匹配。
  - 命令：`rg -n "真实 Discord/Hermes live verification pass 尚未获得|Real Discord/Hermes live verification has not passed yet|Slack sink|observer plugin|deterministic source" README.md README.en.md`。
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
  - commit 信息：中文说明 README HERMES-HIP 艺术字接入、验证和影响。

## Review

- 已参考 `hermes-agent` 顶部 `assets/banner.png` 的黑底、黄橙高对比、像素块风格，为 Hermeship 新增 `HERMES-HIP` wordmark 资产：`docs/assets/branding/hermeship-wordmark.svg`。
- 已用仓库内自包含 SVG 实现 wordmark，不引用外部图片、远程字体或远程资源。
- 已在 `README.md` 和 `README.en.md` 顶部改为横向品牌头部：左侧 `HERMES-HIP` 艺术字，右侧现有 `docs/assets/branding/hermeship-icon.png` 项目图标。
- 已保留语言切换按钮、Hermeship 独立项目叙述和关键能力边界声明；公开 README 仍未出现 `clawhip`、`template/clawhip`、thin adapter 或 runtime adapter 关联表述。
- 已使用 bundled Node `sharp` 将 `docs/assets/branding/hermeship-wordmark.svg` 渲染为 `/tmp/hermeship-wordmark-preview.png` 并完成视觉抽查；清理了首版 SVG 右侧多余黄色块。
- 本轮只修改 README、静态品牌资产和状态记录，不修改功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。
- 已运行验证：SVG XML 解析通过，README wordmark/icon/语言切换引用检查通过，公开 README 相关项目残留关键词检查无匹配，关键能力边界声明检查通过，`git diff --check`，`python3 -m py_compile templates/hermes-plugin/__init__.py`，`cargo fmt --all -- --check`，`cargo test observer_plugin`（13 passed），`cargo test release_preflight`（16 passed），`cargo run -- release preflight 0.1.0`（9 checks ok，`live verification` 只证明记录字段存在），`cargo clippy --all-targets -- -D warnings`，`cargo test`（221 lib tests + 15 bin tests + doctests passed）。
- 阶段提交前已确认 diff 范围仅包含 README 顶部品牌布局、wordmark SVG 资产和状态记录。
