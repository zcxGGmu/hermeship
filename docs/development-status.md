# Hermeship 开发状态

最后更新：2026-06-21 README 顶部去重标题

本文是下次启动 Codex 会话时的状态入口。执行开发前仍以 `tasks/development-checklist.md` 的 checkbox 为准；当前阶段计划维护在 `tasks/todo.md`。

## 当前结论

- Hermeship 的目标已经锁定：完全参考 `/Users/zq/Desktop/ai-projs/posp/template/clawhip` 的项目形态、架构和功能，只把 OpenClaw/Codex/Claude/OMC/OMX 等耦合替换为 Hermes 适配。
- Hermeship 不是调用现有 `clawhip` 的 thin adapter，也不依赖运行中的 `clawhip` daemon。
- 主实现语言确定为 Rust，采用 daemon-first 架构；Python 当前用于 Hermes gateway hook bridge 模板 `handler.py` 和可选 Hermes observer plugin 模板。
- 方案文档与执行清单已经拆分：方案文档维护架构和边界，`tasks/development-checklist.md` 和 `tasks/todo.md` 维护可勾选进度。
- 默认测试策略已经确定：使用本地 fixture、fake sink、fake HTTP、fake Hermes home、fake hermeship binary；真实 Discord/Hermes 只进入 live verification。
- 当前开发分支：`codex/milestone-1-cli`。
- 最新 typed observer body 功能阶段提交：`6053cdf feat: 增加 typed observer body 并收紧安全边界`。
- 最新 Milestone 10.3 功能阶段提交：`803aefa feat: 增加 Hermes observer plugin 安装启用 CLI`。
- 最新 Milestone 10.2 功能阶段提交：`f352222 feat: 增加可选 Hermes observer plugin scaffold`。
- 最新状态续接提交：`b76a007 docs: 记录 Hermeship 本地验证续接`。
- 最新状态文档提交：`95a53d5 docs: 更新 Hermeship 最新开发状态与启动提示词`。
- 最新 Milestone 10.1 契约研究提交：`93aa9ec docs: 完成 Hermes observer plugin 契约研究`。
- 最新 live 记录提交：`bc4c027 docs: 记录 Hermeship live verification 结果`。
- 最新交接提交：`6be5661 docs: 更新 Hermeship Milestone 9.3 交接状态`。
- 最新文档阶段提交：`2e60902 docs: 增加 live verification runbook`。
- 最新功能阶段提交：`6053cdf feat: 增加 typed observer body 并收紧安全边界`；上一功能阶段提交为 `803aefa feat: 增加 Hermes observer plugin 安装启用 CLI`。
- 当前最新功能阶段：Milestone 10 后续 Typed Rust Observer Body 本地 deterministic parity 与安全 hardening 已完成并由 `6053cdf` 提交。
- 当前最新文档阶段：Milestone 10.1 Hermes Observer 契约研究由 `93aa9ec` 完成。
- 当前工作台：`tasks/todo.md` 已切换为“2026-06-21 README 顶部去重标题”。
- 本轮根据用户反馈删除 `README.md` 与 `README.en.md` 顶部 banner 下方重复的可见 `Hermeship` 标题行；banner 继续展示项目名，副标题和语言切换保留。
- 本轮将 README 顶部 banner 的 `alt` 改为 `Hermeship`，因为移除可见标题后 banner 不再是纯装饰图，需要保留读屏入口识别。
- 本轮 README 顶部品牌区改为统一仓库内 banner：`docs/assets/branding/hermeship-lockup.png`，由 `HERMES-HIP` wordmark 与项目图标合成，替代了表格拼接的双图布局。
- 本轮 README 顶部语义层级改为：brand banner 承载项目名，下面只保留简短副标题和对称语言切换；不再显示重复的 `h1` 项目名。
- 本轮参考 `hermes-agent` 顶部 `assets/banner.png` 的黑底、黄橙高对比、像素块风格，为 README 新增仓库内 `HERMES-HIP` wordmark：`docs/assets/branding/hermeship-wordmark.svg`。
- 本轮已将 `README.md` 与 `README.en.md` 顶部改为统一品牌 lockup：左侧 `HERMES-HIP` 艺术字与右侧项目图标合成为 `docs/assets/branding/hermeship-lockup.png`，再由单一居中图片展示；语言切换按钮保留在标题下方。
- 本轮 README HERMES-HIP 艺术字接入只更新公开文档、静态品牌资产和状态记录，不改变功能代码、不新增真实 live pass、不实现 Slack sink、不自动启用 Hermes observer plugin。
- 本轮 README HERMES-HIP 艺术字接入已验证：SVG XML 解析通过，使用 bundled Node `sharp` 渲染 SVG 预览并完成视觉抽查，README wordmark/icon/语言切换引用检查通过，公开 README 相关项目残留关键词检查无匹配，关键能力边界声明检查通过，`python3 -m py_compile templates/hermes-plugin/__init__.py`，`cargo fmt --all -- --check`，`cargo test observer_plugin`（13 passed），`cargo test release_preflight`（16 passed），`cargo run -- release preflight 0.1.0`（9 checks ok，`live verification` 只证明记录字段存在），`cargo clippy --all-targets -- -D warnings`，`cargo test`（221 lib tests + 15 bin tests + doctests passed），`git diff --check`。
- 本轮 README 顶部品牌区优化已验证：`docs/assets/branding/hermeship-lockup.png` 为 1280 x 360 PNG，`docs/assets/branding/hermeship-wordmark.svg` 仍可正常解析，README 仅引用仓库内统一 lockup，不再依赖 table 拼接或 Shields 语言徽章；公开 README 相关项目残留关键词检查无匹配，关键能力边界声明检查通过，`python3 -m py_compile templates/hermes-plugin/__init__.py`，`cargo fmt --all -- --check`，`cargo test observer_plugin`（13 passed），`cargo test release_preflight`（16 passed），`cargo run -- release preflight 0.1.0`（9 checks ok，`live verification` 只证明记录字段存在），`cargo clippy --all-targets -- -D warnings`，`cargo test`（221 lib tests + 15 bin tests + doctests passed），`git diff --check`。
- 本轮根据用户指定图片接入 README 项目图标：新增仓库内资产 `docs/assets/branding/hermeship-icon.png`，由原始 1254 x 1254 PNG 压缩为 512 x 512 PNG，并在 `README.md` 与 `README.en.md` 顶部标题下方展示。
- 本轮 README 图标接入只更新公开文档、静态图片资产和状态记录，不改变功能代码、不新增真实 live pass、不实现 Slack sink、不自动启用 Hermes observer plugin。
- 本轮 README 图标接入已验证：`rg -n -i "clawhip|template/clawhip|thin adapter|runtime adapter" README.md README.en.md` 无匹配，README 图标引用和关键边界声明检查通过，图标资产确认为 512 x 512 PNG，`python3 -m py_compile templates/hermes-plugin/__init__.py`，`cargo fmt --all -- --check`，`cargo test observer_plugin`（13 passed），`cargo test release_preflight`（16 passed），`cargo run -- release preflight 0.1.0`（9 checks ok，`live verification` 只证明记录字段存在），`cargo clippy --all-targets -- -D warnings`，`cargo test`（221 lib tests + 15 bin tests + doctests passed），`git diff --check`。
- 本轮根据用户反馈优化 README 公开项目定位：`README.md` 与 `README.en.md` 不再出现 `clawhip`、`template/clawhip`、thin adapter 或 runtime adapter 相关表述，对外只呈现 Hermeship 作为独立 Hermes-native daemon-first 事件通知路由器。
- 本轮已更新 `tasks/lessons.md`，记录“公开 README 必须是 Hermeship 独立叙述”的规则；内部开发历史可以保留参考来源，公开 README 不写内部参考来源。
- 本轮 README 独立叙述优化只更新公开文档和状态记录，不改变功能代码、不新增真实 live pass、不实现 Slack sink、不自动启用 Hermes observer plugin。
- 本轮 README 独立叙述优化已验证：`rg -n -i "clawhip|template/clawhip|thin adapter|runtime adapter" README.md README.en.md` 无匹配，`rg -n -i "claw" README.md README.en.md` 无匹配，关键边界声明检查通过，`python3 -m py_compile templates/hermes-plugin/__init__.py`，`cargo fmt --all -- --check`，`cargo test observer_plugin`（13 passed），`cargo test release_preflight`（16 passed），`cargo run -- release preflight 0.1.0`（9 checks ok，`live verification` 只证明记录字段存在），`cargo clippy --all-targets -- -D warnings`，`cargo test`（221 lib tests + 15 bin tests + doctests passed），`git diff --check`。
- 本轮根据用户反馈修正 README 多语言结构：根 `README.md` 保留中文入口，新增 `README.en.md` 作为英文入口，并在两个文件顶部添加语言切换按钮；根 README 已移除多余的 `## 中文` 包装层。
- 本轮已更新 `tasks/lessons.md`，记录“README 多语言不要混排”的规则。
- 当前 README 文档入口为分文件双语结构：`README.md` 是中文入口，`README.en.md` 是英文入口；上一轮新增的 `docs/assets/diagrams/` 3 组 Style 6（Claude Official）图表资产仍作为两个 README 的共享图表。
- 本轮 README 语言切换修正只更新项目说明结构，不改变功能代码、不新增真实 live pass、不实现 Slack sink、不自动启用 Hermes observer plugin。
- 本轮 README 语言切换修正已验证：README 语言链接/混排检查通过，关键边界声明检查通过，`python3 -m py_compile templates/hermes-plugin/__init__.py`，`cargo fmt --all -- --check`，`cargo test observer_plugin`（13 passed），`cargo test release_preflight`（16 passed），`cargo run -- release preflight 0.1.0`（9 checks ok，`live verification` 只证明记录字段存在），`cargo clippy --all-targets -- -D warnings`，`cargo test`（221 lib tests + 15 bin tests + doctests passed），`git diff --check`。
- 本轮已验证图表资产：3 个 JSON 解析通过、3 个 SVG XML 解析通过、3 个 PNG 均为 1280x760，并完成视觉抽查。
- 本轮已重新运行默认本地验证：`python3 -m py_compile templates/hermes-plugin/__init__.py`、`cargo test observer_plugin`（13 passed）、`cargo test release_preflight`（16 passed）、`cargo run -- release preflight 0.1.0`（9 checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（221 lib tests + 15 bin tests + doctests passed）。
- `cargo run -- release preflight 0.1.0` 的 `live verification` check 只证明 `docs/live-verification.md` 记录字段存在，不断言真实 Discord/Hermes live pass。
- 最近 Milestone 10.3、typed observer body 阶段和本轮本地验证续接均未提供 Discord credentials、测试频道、Hermes gateway 测试环境或明确执行确认，因此未执行真实 Discord/Hermes live check。
- 本轮用户已明确要求进入 Milestone 10，记录为“真实 live pass 被用户豁免”；Milestone 10 已解锁，Slack sink 仍不在当前默认范围内。
- Milestone 10.1 Observer 契约研究已由 `93aa9ec` 完成并提交：新增 `docs/observer-plugin.md`。
- Milestone 10.2 Observer Plugin MVP scaffold 已由 `f352222` 完成并提交：新增 `templates/hermes-plugin/plugin.yaml` 与 `templates/hermes-plugin/__init__.py`，实现 fail-open safe-field forwarding 到 `POST /event`，并扩展 release preflight 与 Python smoke 覆盖。
- Milestone 10.3 Observer Plugin install/enable CLI 已在本轮完成：新增 `src/observer_plugin.rs`，`hermeship hermes install-plugin` 可把模板安装到 `$HERMES_HOME/plugins/hermeship-observer/`，安装器拒绝 symlinked plugin directory、模板文件和 marker 文件；`hermeship hermes enable-plugin` 只输出手动启用指引，不调用真实 Hermes 或修改 Hermes config。
- Milestone 10 后续 Typed Rust Observer Body 已在本轮完成并补安全 hardening：`hermes.observer.*` 进入 `EventBody::HermesObserver`，保留 canonical kind，派生 `observer_category` / `observer_action`，仅保存 allowlisted safe fields、计数、长度、canonical status、bounded exception-class `error_type` 和布尔摘要；raw `session_key`、secret-shaped error type、非 canonical reason/status/error text 不进入 typed body/raw/explain；observer event core `provider` 固定为 `hermes`，API provider 通过 `observer_provider` 匹配；router 支持 observer typed filter，并通过 `observer_<field>` 暴露 body 同名字段而不覆盖 core metadata；renderer 支持 compact/inline/raw 安全输出。
- 下次继续开发前必须先运行 `git status --short --branch` 确认工作树，只在预期文档/代码变更上继续。
- 当前下一步：如提供凭据和明确确认，可补做 Milestone 9.3 真实 Discord/Hermes live check；否则继续真实 observer 使用反馈修正或其他明确需求，不默认实现 Slack sink。

## 阶段状态总览

| 阶段 | 状态 | 备注 |
| --- | --- | --- |
| Milestone 0 | 已完成并提交 | 契约与仓库基线 |
| Milestone 1.1 - 1.3 | 已完成并提交 | Rust 项目骨架、配置模型、质量门禁 |
| Milestone 2.1 - 2.3 | 已完成并提交 | 事件模型、typed envelope、隐私清洗 |
| Milestone 3.1 - 3.3 | 已完成并提交 | daemon health、event ingress、Hermes hook ingress |
| Milestone 4.1 - 4.3 | 已完成并提交 | router、renderer、dispatcher 与 fake sink |
| Milestone 5.1 - 5.3 | 已完成并提交 | Discord sink、sink 失败语义、本地端到端 smoke |
| Milestone 6 | 已完成并提交 | `f6f98a3 feat: 支持 Hermes hook bridge 安装` |
| Milestone 7 | 已完成并提交 | `162efcd feat: 增加安装生命周期与发布预检` |
| Milestone 8.1 | 已完成并提交 | `1536b6a feat: 增加 Git Source 本地事件路径` |
| Milestone 8.2 | 已完成并提交 | `91d13d8 feat: 完成 GitHub Source 本地确定性路径并修复回归` |
| Milestone 8.3 | 已完成并提交 | `3745bb8 feat: 增加 tmux 事件 source` |
| Milestone 8.4 | 已完成并提交 | `0b12de3 feat: 增加 cron 与 memory scaffold` |
| Milestone 9.1 | 已完成并提交 | `1c52655 docs: 增加 Hermeship 运维与事件契约` |
| Milestone 9.2 | 已完成并提交 | `docs/live-verification.md` runbook |
| Milestone 9.3 | 部分完成 / 阻塞 | 已完成 blocked/not_run 记录；真实 Live Check 未通过 |
| Milestone 10 解锁 | 已完成 | 用户已豁免 Milestone 9.3 真实 live pass 对 Milestone 10 的阻塞；不代表 live pass 已通过 |
| Milestone 10.1 | 已完成并提交 | `93aa9ec docs: 完成 Hermes observer plugin 契约研究` |
| Milestone 10.2 | 已完成并提交 | `f352222 feat: 增加可选 Hermes observer plugin scaffold` |
| Milestone 10.3 | 已完成并提交 | `803aefa feat: 增加 Hermes observer plugin 安装启用 CLI` |
| Milestone 10 后续 | 部分完成 | typed Rust observer body 与安全 hardening 已由 `6053cdf` 完成并提交；真实使用反馈修正按后续需求推进 |

## 完成与未完成边界

已完成：

- Milestone 0 到 8.4、9.1、9.2 已完成并提交。
- Milestone 9.3 的 `blocked`/`not_run` 状态记录已完成；真实 live pass 没有完成。
- Milestone 10.1 Observer 契约研究已由 `93aa9ec` 完成并提交。
- Milestone 10.2 Observer Plugin MVP scaffold 已由 `f352222` 完成并提交。
- Milestone 10.3 Observer Plugin install/enable CLI 已由 `803aefa` 完成并提交。
- Milestone 10 后续 typed Rust observer body 与安全 hardening 已由 `6053cdf feat: 增加 typed observer body 并收紧安全边界` 完成并提交。
- 本地 deterministic 验证、release preflight observer plugin template check、Python compile/smoke 覆盖已完成。

未完成：

- 真实 Discord/Hermes live verification pass。
- 真实 GitHub API source、真实 tmux watch、真实 scheduler、真实 service manager 自动安装。
- Slack sink；除非后续明确纳入范围，否则默认不做。

## 已完成范围

- 已记录项目习惯：每完成一阶段任务，必须验证并提交；后续会话启动时先复习 `tasks/lessons.md`。
- 已重写方案文档：`docs/plans/2026-06-15-hermeship-development-plan.md`。
- 已重写阶段性开发清单：`tasks/development-checklist.md`。
- 已将测试计划集成到方案文档和开发清单。
- 已新增 `ARCHITECTURE.md` 和 `docs/hermes-event-contract.md`，并重写 README / operations 作为 Milestone 9.1 文档入口。
- Milestone 0 到 Milestone 8.4 已完成并提交。
- Milestone 9.1 已完成并提交：README、operations、Hermes event contract 和 architecture 文档已对齐当前能力。
- Milestone 9.2 已完成并提交：`docs/live-verification.md` runbook 已创建。
- Milestone 9.3 已完成“未执行原因和剩余风险记录”：`docs/live-verification.md` 已有 `blocked`/`not_run` 结果；这不是真实 live pass。
- Milestone 10.1 已完成 Observer 契约研究：`docs/observer-plugin.md` 记录 plugin discovery、hook mapping、隐私边界、fail-open、`/event` ingress 和 10.2 follow-up。
- Milestone 10.2 已完成可选 Observer Plugin MVP scaffold：`templates/hermes-plugin/` 提供 Hermes directory plugin 模板，release preflight 和 smoke 测试已覆盖模板契约。
- Milestone 10.3 已完成 observer plugin install/enable CLI automation：安装模板仍是显式命令，启用仍由 operator 手动执行 `hermes plugins enable hermeship-observer`；安装器不会跟随 symlink 覆盖插件目录外目标。
- Milestone 10 后续已完成 typed Rust observer body 与安全 hardening：`hermes.observer.*` 不再走 `Custom` fallback，默认 renderer 和 route filter 可使用 typed safe observer fields；raw `session_key`、secret-shaped `error_type`、非 canonical 自由文本、object-style approval 摘要和 observer metadata shadowing 均有回归覆盖。

### Milestone 0：契约与仓库基线

- 已复核 `template/clawhip` 指定参考文件，确认可移植形态为 Rust CLI、daemon、typed event、dispatcher、multi-delivery router、renderer/sink split、config/lifecycle/release preflight。
- 已复核 Hermes gateway hook 与 plugin 参考源码，确认 MVP 先使用 gateway hook bridge，plugin/observer 后续推进。
- 已更新 `README.md`，明确 Hermeship 是 Hermes-native daemon-first event router，不是 clawhip runtime client。
- 已运行旧 Python/thin-adapter 方向过滤搜索，正文无旧方案残留。
- 已提交：`af57c49 docs: 明确 hermeship 完整项目方向`。

### Milestone 1.1：Cargo 项目与 CLI 入口

- 已创建 Rust 2024 工程骨架：`Cargo.toml`、`Cargo.lock`、`src/lib.rs`、`src/main.rs`、`src/cli.rs`。
- 已实现最小 `clap` CLI 命令树：`start`、`status`、`send`、`emit`、`explain`、`config`、`hermes`、`install`、`uninstall`、`release`。
- 已新增 CLI parse 单元测试，覆盖 `send`、`emit --payload`、`hermes hook --payload`、`hermes install-hooks`。
- 已新增公开命令 fixture：`tests/fixtures/cli/public_commands.txt`，并断言必备公开命令前缀存在。
- 已运行验证：`cargo fmt --all -- --check`、`cargo test cli`、`cargo run -- --help`。
- 已提交：`d03170e chore: 搭建 Hermeship Rust CLI 骨架`。

### Milestone 1.2：配置模型

- 已新增 `src/config.rs`，并在 `src/lib.rs` 导出 `hermeship::config`。
- 已实现配置模型：`AppConfig`、`DaemonConfig`、`ProvidersConfig`、`DiscordConfig`、`DefaultsConfig`、`PrivacyConfig`、`HermesConfig`、`RouteRule`、`MessageFormat`。
- 已实现默认配置路径：`HERMESHIP_CONFIG` 优先，否则 `$HOME/.hermeship/config.toml`。
- 已实现默认配置与 TOML 加载：缺失配置返回默认值，非法 TOML 返回错误，未知 key 按前向兼容策略忽略。
- 已实现空值归一化和环境变量覆盖：`HERMESHIP_DAEMON_URL`、`HERMESHIP_DISCORD_TOKEN`、`HERMESHIP_DEFAULT_CHANNEL`、`HERMESHIP_DRY_RUN`。
- 已将 `hermeship config path`、`hermeship config show`、`hermeship config verify` 接入真实配置逻辑。
- 已运行验证：`cargo fmt --all -- --check`、`cargo test config`、`cargo run -- config show`、`cargo test`。
- 已提交：`50723af feat: 实现 hermeship 配置模型与 config CLI`。

### Milestone 1.3：质量门禁与仓库基础

- 已扩展 `.gitignore`：保留 `/target/`，新增本地编辑器临时文件、日志、临时目录、测试输出和覆盖率输出规则。
- 已确认 `.gitignore` 不忽略源码、文档、fixture 或 `Cargo.lock`。
- 已在 `README.md` 新增 Development Quality Gates。
- 已新增 fixture 目录：`tests/fixtures/hermes/`、`tests/fixtures/privacy/`、`tests/fixtures/routes/`、`tests/fixtures/discord/`。
- 已新增 `tests/fixtures/README.md`，明确 fixture 只能使用合成脱敏样例。
- 已运行验证：`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 已提交：`70c8f03 chore: 增加 Rust 质量门禁与仓库基础`。

### Milestone 2.1：IncomingEvent 与格式

- 已新增 `src/events.rs`，并在 `src/lib.rs` 导出 `hermeship::events`。
- 已实现 `IncomingEvent`：字段包含 `kind`、`channel`、`mention`、`format`、`template`、`payload`。
- 已实现 `RoutingMetadata`：覆盖 Hermes gateway 元数据和后续路由需要的通用字段，如 `tool`、`provider`、`source`、`platform`、`session_id`、`project`、`repo_path`、`branch`。
- 已采用单一 `MessageFormat` 策略：`src/config.rs` 保留唯一 enum 定义并新增 `from_label()`；`src/events.rs` 通过 `pub use crate::config::MessageFormat` 重导出。
- 已支持 `IncomingEvent` 反序列化字段别名：`type`、`kind`、`event`。
- 已支持缺省 payload 和 `payload: null` 归一为空对象；无显式 payload 时，top-level extra 字段进入 payload。
- 已将 `hermeship emit` 和 `hermeship explain` 的参数解析接入 `EventArgs::into_event()`。
- 已新增 Hermes 合成 fixture：`tests/fixtures/hermes/agent_start.json`、`session_end.json`、`invalid_payload.json`。
- 已运行验证：`cargo test events`、`cargo test cli`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 已提交：`5584b13 feat: 完成 Hermes 入口事件模型与 emit 解析`。

### Milestone 2.2：Typed EventEnvelope

- 已新增 `src/event/mod.rs`、`src/event/body.rs`、`src/event/compat.rs`，并在 `src/lib.rs` 导出 `hermeship::event`。
- 已定义 `EventEnvelope`、`EventBody`、`EventMetadata`、`EventPriority`。
- 已实现 Hermes event body：`HermesGatewayStarted`、`HermesSessionStarted`、`HermesSessionFinished`、`HermesSessionReset`、`HermesAgentStarted`、`HermesAgentStep`、`HermesAgentFinished`、`HermesAgentFailed`、`Custom`。
- 已实现 canonical mapping：`gateway:startup`、`session:start`、`session:end`、`session:reset`、`agent:start`、`agent:step`、`agent:end`；显式失败的 `agent:end` 会转为 `hermes.agent.failed`。
- 已实现 `IncomingEvent -> EventEnvelope` conversion，保留 route hint 并提取 provider/source/platform/chat/session/agent/project/repo metadata。
- 已覆盖未知 event -> `Custom`、缺失 `session_id` 降级、fixture conversion、大小写不敏感失败状态。
- 已运行验证：`cargo test event`、`cargo test events`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 已提交：`b799415 feat: 实现 Hermes typed event model`。

### Milestone 2.3：隐私与 payload 清洗

- 已新增 `src/privacy.rs`，并在 `src/lib.rs` 导出 `hermeship::privacy`。
- 已实现 `sanitize_payload`、`redact_value`、`excerpt_policy`，保持为纯 `serde_json::Value` 清洗逻辑。
- 已默认递归脱敏敏感 key：`token`、`api_key`、`authorization`、`password`、`secret`、`cookie`；支持大小写不敏感、camelCase 和常见缩写 key 匹配。
- 已默认删除完整正文类字段：`message`、`response`、`conversation_history`、`request`、`provider_response`、`tool_result`；同时清洗 `messages`、`prompt`、`user_message`、`assistant_response`、`provider_request`、`provider_request_body`、`provider_response_body`、`tool_results`、`tool_result_body` 等同类高风险别名。
- 已保留安全摘要：`message_chars`、`response_chars`、`has_message`、`has_response`；非法摘要字段类型会被丢弃，computed summary 不会被原始 payload 覆盖。
- 已实现 opt-in 摘录：`include_message_excerpt`、`include_response_excerpt`、`max_excerpt_chars`；摘录先经过完整 sanitizer，再按 char 边界截断。
- 已新增合成 fixture：`tests/fixtures/privacy/sensitive_payload.json`，不包含真实 token、cookie、secret、完整 prompt、完整对话或 provider request/response body。
- 已根据代码审查修复摘要字段泄漏、`Authorization: Bearer ...` / `api_key = ...` inline secret 泄漏、URL query secret 泄漏、camelCase/acronym alias 绕过、结构化摘录泄漏和 fixture body hygiene 问题。
- 已运行验证：`cargo test privacy`（10 passed）、`cargo test event`（14 passed）、`cargo test events`（6 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（41 passed）。
- 已提交：`175009d feat: 增加 Hermes 事件隐私清洗`。

### Milestone 3.1：Daemon health 与 client

- 已新增 `src/daemon.rs`，并在 `src/lib.rs` 导出 `hermeship::daemon`。
- 已新增 `src/client.rs`，并在 `src/lib.rs` 导出 `hermeship::client`。
- 已实现 typed `HealthResponse` 与 `QueueHealth`。
- 已实现 daemon `/health` endpoint，返回 version、status、queue 状态和 configured sinks 摘要。
- 已实现 daemon listener 绑定与 `serve_listener()`，测试可使用随机端口。
- 已实现 `hermeship start`：加载配置、支持 `--port` 覆盖、验证配置并启动 daemon。
- 已实现 `hermeship status`：通过 `DaemonClient` 调用 `/health` 并打印可读摘要。
- 已实现 client base URL 规范化、2 秒 health timeout、daemon unavailable 清晰错误和非 2xx 错误摘要。
- 已覆盖 health response schema、队列状态、configured sinks、随机端口 HTTP `/health` 和 daemon 未运行错误。
- 本阶段没有实现 event ingress、`/event`、Hermes hook ingress、队列入队、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
- 已运行验证：`cargo test daemon`（4 passed）、`cargo run -- status`（daemon 未运行时返回清晰错误且无 panic）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（45 passed）。
- 已提交：`ff5c589 feat: 增加 hermeship daemon health`。

### Milestone 3.2：Event ingress 与队列

- 已实现 daemon 通用 `POST /event` endpoint，接收 `IncomingEvent` JSON。
- 已在入队前接入 `privacy::sanitize_payload()`，再使用 `event::compat::from_incoming_event()` 转为 typed `EventEnvelope`。
- 已新增 bounded `tokio::mpsc` queue scaffold；本阶段只入队，不消费、不路由、不渲染、不投递。
- 已新增 typed `EventAcceptedResponse`，返回 event id、canonical kind、queued 状态和 queue health。
- 已将 `/health` queue 状态改为真实 pending/capacity/status。
- 已实现 `DaemonClient::event_url()` 与 `DaemonClient::post_event()`，覆盖 daemon unavailable、非 2xx 和无效响应错误。
- 已将 `hermeship emit` 和 `hermeship send` 替换为 daemon client POST `/event` 路径，输出 queued 摘要。
- 已调整 `IncomingEvent::custom()` 使用安全 `summary` 字段承载显式 send 文本，避免与 Hermes 对话正文 `message` 隐私语义冲突。
- 已覆盖有效 fixture 入队、入队前隐私清洗、非法 JSON 4xx、缺失 event kind 4xx、daemon unavailable、queue full 503、health pending、send/emit client 投递。
- 本阶段没有实现 Hermes hook ingress、router、renderer、dispatcher、sink、hook bridge、install 或 release preflight。
- 已运行验证：`cargo test daemon`（11 passed + bin 2 passed）、`cargo test event`（21 passed + bin 2 passed）、临时 daemon 下 `cargo run -- emit hermes.agent.started --payload '{"session_id":"demo"}'` 返回 queued 摘要、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（52 passed + bin 2 passed）。
- 已提交：`0b63e49 feat: 增加 daemon event ingress`。

### Milestone 3.3：Hermes hook ingress

- 已新增 `src/hermes.rs`，并在 `src/lib.rs` 导出 `hermeship::hermes`。
- 已实现 `HermesHookEnvelope`：接收 `provider`、`source`、`event`/`event_type`、`context`，默认 provider/source 为 `hermes`/`gateway`。
- 已实现 Hermes hook envelope -> `IncomingEvent` normalization，payload 保留 provider/source/event/context metadata，并复用既有 Hermes canonical mapping。
- 已实现 daemon `POST /api/hermes/hook` endpoint，复用 `/event` 的入队前 privacy sanitizer、typed conversion 和 bounded queue `try_send` 管道。
- 已实现 `DaemonClient::hermes_hook_url()` 与 `DaemonClient::post_hermes_hook()`，daemon unavailable、非 2xx 和无效响应错误包含 `/api/hermes/hook`。
- 已将 `hermeship hermes hook --payload` 替换为真实 daemon client POST 路径，支持 inline JSON 和 `--payload -` stdin，输出 queued 摘要。
- 已覆盖 hook envelope 默认值、`event_type` alias、gateway/session/agent mapping、`agent:end` 成功/失败 mapping、daemon hook 入队、入队前隐私清洗、缺失 event 4xx、daemon unavailable、CLI stdin 和 client 投递。
- 本阶段没有实现 router、renderer、dispatcher、sink、hook bridge install、install/uninstall lifecycle 或 release preflight。
- 已运行验证：`cargo test hermes`（14 lib tests + 3 bin tests passed）、临时 daemon 下 `printf '%s' '{"event":"agent:start","context":{"session_id":"demo"}}' | cargo run -- hermes hook --payload -` 返回 queued 摘要、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（61 lib tests + 5 bin tests passed）。
- 已提交：`7b10816 feat: 增加 Hermes hook ingress`。

### Milestone 4.1：Router

- 已新增 `src/router.rs`，并在 `src/lib.rs` 导出 `hermeship::router`。
- 已实现 `Router`、`ResolvedDelivery`、`SinkTarget`、`DeliveryExplanation`，支持 event glob、route candidates、结构化 metadata filter、disabled route 诊断、missing target 诊断和 0..N delivery。
- 已将 `hermeship explain` 从 placeholder 替换为本地 route explain：加载配置、清洗 payload、转 typed `EventEnvelope`、打印 route candidates、matched/skipped routes、failed filters 和 delivery target。
- `explain` 不调用 daemon、不入队、不渲染、不投递；renderer、dispatcher、sink、hook bridge install、install/uninstall lifecycle 和 release preflight 仍属于后续 milestone。
- 已覆盖多 route 多投递、filter 命中/未命中、disabled route、missing target、无 route、event hint/default channel fallback、route-level channel/format/template/mention 继承、explain 输出契约和 webhook 诊断脱敏。
- 已根据代码审查修复 Discord webhook 诊断泄漏风险：`explain` human/serialized diagnostics 不输出完整 webhook URL，内部 delivery target 仍保留原值供后续 dispatcher 使用。
- 已运行验证：`cargo test router`（6 lib tests + 1 bin test passed）、`cargo run -- explain hermes.agent.started --payload '{"platform":"telegram","session_id":"demo"}'` 返回 no routes/no deliveries 诊断、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（67 lib tests + 6 bin tests passed）。
- 已提交：`864e7f4 feat: 实现多投递路由`。

### Milestone 4.2：Renderer

- 已新增 `src/render/mod.rs`、`src/render/default.rs`，并在 `src/lib.rs` 导出 `hermeship::render`。
- 已实现 `Renderer` trait、`DefaultRenderer` 和 `RenderedMessage`，输入为 `EventEnvelope` 与 `ResolvedDelivery`，输出 deterministic 可投递文本。
- 已支持 `compact`、`inline`、`alert`、`raw` 四种格式，并覆盖 Hermes gateway/session/agent/custom 事件。
- 已实现 route/template 安全 token：`{event}`、`{canonical_kind}`、`{source}`、`{provider}`、`{platform}`、`{session_id}`、`{agent_name}`、`{project}`、`{channel}`；未批准 token 保持原样。
- 已将 `raw` 固定为安全 JSON 输出：忽略 template/mention，不直接序列化 typed 自由文本，只输出长度/存在性摘要并清洗 nested payload。
- 已覆盖测试：所有格式、缺字段降级、template token、route-level format/template/mention、raw+template、direct typed free-text raw 泄漏回归和未批准 token。
- 已运行验证：`cargo test render`（10 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（74 lib tests + 6 bin tests passed）。
- 已确认本阶段没有实现 dispatcher、sink、hook bridge install、install/uninstall lifecycle 或 release preflight。
- 已提交：`d4303ae feat: 增加 Hermes 默认渲染器`。

### Milestone 4.3：Dispatcher 与 fake sink

- 已新增 `src/dispatch.rs`、`src/sink/mod.rs`、`src/sink/fake.rs`，并在 `src/lib.rs` 导出 `dispatch` 与 `sink`。
- 已实现 object-safe `Sink` trait、`SinkMessage`、`FakeSink` 和 `FakeDelivery`，用于本地测试记录 target、format、rendered content、event kind 和 route index。
- 已实现 `Dispatcher`、`DispatchReport`、`DeliveryOutcome` 和 `DeliveryStatus`，支持单事件与队列消费，执行 `Router::resolve -> Renderer::render -> Sink::send`。
- 已实现单个 delivery 失败不阻断其他 delivery；render failure、missing sink 和 sink failure 都能在 report 中观察。
- 已将默认 daemon queue 接入 dispatcher consumer，生产 daemon 不再只入队不消费；本阶段未注册真实 sink，Discord sink 仍在 Milestone 5。
- 已覆盖多投递、单 sink failure、无 route、render failure、missing sink、队列消费、daemon ingress -> dispatcher -> fake sink E2E 和隐私不泄漏。
- 原计划命令 `cargo test dispatch sink` 是无效 Cargo 语法，执行时返回 `unexpected argument 'sink'`；实际验证拆分为 `cargo test dispatch` 与 `cargo test sink`。
- 已运行验证：`cargo test dispatch`（8 passed）、`cargo test sink`（8 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（87 lib tests + 6 bin tests passed）。
- 已确认本阶段没有实现 Discord sink、Hermes hook bridge install、install/uninstall lifecycle 或 release preflight。
- 已提交：`a336e01 feat: 实现事件 dispatcher 与 fake sink`。

### Milestone 5.1：Discord 配置与 payload

- 已新增 `src/sink/discord.rs`，并在 `src/sink/mod.rs` 导出 Discord sink 模块。
- 已实现 Discord payload/request builder：`content`、`allowed_mentions`、2000 字符内容截断和 webhook `wait=true`。
- 已处理代码审查反馈：webhook URL 会精确移除已有 `wait` query 参数并追加 `wait=true`，避免 `wait=false` 或 `await=true` 误判。
- 已支持 Discord bot token + channel delivery，以及 route webhook delivery。
- 已将 `SinkMessage` 增加 `mention` 字段，dispatcher 会从 `ResolvedDelivery` 传递 route/event mention；Discord `allowed_mentions` 只允许显式 mention 中的 user/role id，正文中其他 mention 默认不 ping。
- 已将 daemon dispatcher registry 接入真实 Discord sink，生产 daemon 不再只注册空 sink map；缺少 token/channel/webhook 时返回 sink failure 诊断，不 panic。
- 已覆盖测试：webhook payload、bot channel request、allowed mentions、内容长度截断、token/channel/webhook 缺失诊断、fake HTTP webhook 投递、dispatcher mention 传递和 daemon sink registry。
- 已运行验证：`cargo test discord`（9 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（97 lib tests + 6 bin tests passed）。
- 本阶段没有实现 Hermes hook bridge install、install/uninstall lifecycle、release preflight、真实 live verification、Slack sink 或 Discord 失败矩阵深化。
- 已提交：`0cd6e4e feat: 增加 Discord sink`。

### Milestone 5.2：Sink 失败语义

- 已深化 Discord sink 失败语义：非 2xx 返回清晰错误，包含 HTTP status 和受控 body tail。
- 已实现 Discord 429 rate limit 诊断：解析 JSON `retry_after` 并输出 `retry_after=<seconds>s`；缺失或非法时输出 `retry_after=unknown`。
- 已确认本阶段不引入 sleep/retry 状态机、DLQ、circuit breaker 或 clawhip runtime 依赖。
- 已覆盖 token 缺失经 dispatcher 报告 `SinkFailed`、空 channel/request builder 错误、fake HTTP 500、fake HTTP 429、429 fallback 和多 delivery 中一个失败时其他继续。
- 已处理代码审查建议：fake HTTP server await 增加 2 秒超时，避免未来请求未发出时测试挂住。
- 已运行验证：`cargo test sink`（23 passed）、`cargo test dispatch`（11 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（102 lib tests + 6 bin tests passed）。
- 本阶段没有实现 Hermes hook bridge install、install/uninstall lifecycle、release preflight、真实 live verification、Slack sink 或 Hermes plugin/observer。
- 已提交：`ea9b789 feat: 完善 sink 失败处理`。

### Milestone 5.3：本地端到端 smoke

- 已新增 daemon 内部 sink registry 注入 helper，生产 `daemon_router()` 仍使用真实 Discord sink registry，测试可通过同一条内部 queue consumer 路径注入 `FakeSink`。
- 已新增本地 deterministic smoke：随机端口 test daemon 接收 `POST /api/hermes/hook`，内部 dispatcher 执行 `Router -> DefaultRenderer -> FakeSink`，最终 fake sink 记录渲染后的 delivery。
- 已断言 smoke 不依赖真实 Discord、真实 Hermes gateway、外网或真实凭据；不启动固定端口 daemon。
- 已断言 fake sink 收到 `hermes.agent.started`、`DiscordChannel("ops")`、`compact` 格式、agent/platform/session 摘要和 message/response 字符数摘要。
- 已断言默认隐私保护生效，不泄漏完整 message、response、token、cookie 或 secret。
- 已通过现有本地 test daemon/client 测试继续覆盖 `send` 和 `emit` 进入 `/event` 的本地路径。
- 已运行验证：`cargo test dispatch`、`cargo test daemon`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 本阶段没有实现 Hermes hook bridge install、install/uninstall lifecycle、release preflight、真实 live verification、Slack sink 或 Hermes plugin/observer。
- 已提交：`026e80c test: 增加 daemon 到 sink 的端到端覆盖`。

### Milestone 6：Hermes Hook Bridge 安装

- 已新增 Hermes gateway hook 模板：`templates/hermes-hook/HOOK.yaml` 与 `templates/hermes-hook/handler.py`。
- `HOOK.yaml` 声明默认启用的 `gateway:startup`、`session:start`、`session:end`、`session:reset`、`agent:start` 与 `agent:end`；`agent:step` 与 `command:*` 当前不默认安装，避免绕过默认关闭的 Hermes 配置开关。
- `handler.py` 只使用 Python 标准库，不 import Hermeship package；暴露 `handle(event_type, context)`，将 compact JSON 通过 stdin 传给 `hermeship hermes hook --payload -`。
- handler 对 missing binary、子进程失败和 timeout 全部 fail-open，只输出短诊断，不向 Hermes 抛异常；安装时会把当前 hermeship binary 路径渲染进 handler，仍支持 `HERMESHIP_BIN` 覆盖。
- 已新增 `src/hooks.rs`，实现 `install_hermes_hooks`、`uninstall_hermes_hooks`、默认 Hermes home 解析、dry-run、force 覆盖、不覆盖已有文件、`.hermeship-managed.json` 安全卸载 marker 和路径报告。
- 已接入 CLI：`hermeship hermes install-hooks --home <path> --force --dry-run` 与 `hermeship hermes uninstall-hooks --home <path> --dry-run`。
- 已更新公开命令 fixture，覆盖 `hermes uninstall-hooks` 解析。
- 已完成本地 deterministic CLI 验证：安装到 `/tmp/hermeship-test-home`，确认 `HOOK.yaml` 与 `handler.py` 写入，再卸载删除 Hermeship hook 目录。
- 已运行验证：`cargo test hooks`（19 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（120 lib tests + 6 bin tests passed）。
- 本阶段没有实现 release preflight、真实 live verification、Slack sink、Hermes plugin/observer 或通用服务 lifecycle。
- 已提交：`f6f98a3 feat: 支持 Hermes hook bridge 安装`。

### Milestone 7：安装、生命周期与运维 CLI

- 已新增 `src/lifecycle.rs`，实现 `hermeship install`、`hermeship setup` 和 `hermeship uninstall` 的本地 deterministic 路径。
- `install` 创建 Hermeship home、`state/`、`hooks/`、`logs/` 和默认 `config.toml`；支持 `--home`、`--force`、`--dry-run`，不会启动 daemon 或安装真实 service。
- `setup` 支持通过 stdin/env 写入 Discord token、default channel 和 daemon URL；报告输出将 token 脱敏，`config show` 默认脱敏。
- `uninstall` 默认保留用户 config/state/hooks；只有显式 `--remove-config`、`--remove-state`、`--remove-hooks` 才删除对应路径，Hermes gateway hook 删除复用 Milestone 6 marker-based safe uninstall。
- 已新增 `deploy/hermeship.service` systemd user service 模板和 `docs/operations.md` 运维说明；本阶段不运行 `systemctl` 或 `launchctl`。
- 已新增 `src/release_preflight.rs`，检查 `Cargo.toml`/`Cargo.lock` 版本一致性、公开 CLI fixture、文档命令、hook 模板、fixture policy、service 模板和 live verification 状态；缺失 live verification 记录为 `pending`，不阻塞默认本地 preflight。
- 已更新 CLI 公开命令 fixture，覆盖 `setup`、带参数 `install`/`uninstall` 和 `release preflight`。
- 已根据代码审查修复 lifecycle 安全边界：`setup` 不再接受明文 token argv，改用 stdin/env；`config show` 默认脱敏；写配置时使用私有权限；`install` 写入 home marker；destructive `uninstall` 必须验证 marker；`--remove-hooks` 默认使用 Hermes home；release preflight 纳入 `docs/operations.md`。
- 已运行验证：`cargo test lifecycle`（10 passed）、`cargo test release_preflight`（6 passed）、`cargo test cli`（17 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（139 lib tests + 8 bin tests passed）。
- 本阶段没有实现真实 live verification、Slack sink、Hermes plugin/observer、真实 systemd/launchd 安装或外部网络发布自动化。

### Milestone 8.1：Git Source 本地 deterministic parity

- 已新增 `src/source/mod.rs` 与 `src/source/git.rs`，实现 `GitCommitInput`、`GitBranchChangedInput`、`commit_event()` 和 `branch_changed_event()`。
- Git source 当前只基于显式 CLI 输入构造 `IncomingEvent`；不执行真实 `git` 命令、不轮询 repo、不访问远端、不依赖 clawhip runtime。
- Git commit source 与 compat 均会拒绝非 7-64 hex commit、空 summary、多行 summary 和过长 display field，避免畸形 payload 绕过 raw renderer 字段级防护。
- 已新增 typed Git body：`GitCommitEvent` 与 `GitBranchChangedEvent`，并将 `git.commit` / `git.branch-changed` 接入 `IncomingEvent -> EventEnvelope` conversion。
- 已接入 CLI：`hermeship git commit` 与 `hermeship git branch-changed`，命令复用现有 `DaemonClient::post_event()` 投递 `/event`。
- 已扩展 router/renderer：route filter 可用 `repo_name`、`repo_path`、`worktree_path`、`branch` metadata；默认 compact 渲染输出 repo/branch/short commit/summary/author 摘要。
- raw JSON 渲染只输出受控 Git 字段，不展开完整 diff、commit body、repo path、worktree path 或 author email。
- 已更新公开命令 fixture和 release preflight 检查，要求覆盖 `git commit` 与 `git branch-changed`。
- 已覆盖测试：source 构造、typed conversion、route metadata filter、CLI parse、daemon submit、renderer 隐私边界和 release preflight。
- 已运行 Red：`cargo test git` 在实现前失败于缺少 `source::git` API、`GitCommands`、`Commands::Git` 和 Git typed body variants。
- 已运行验证：`cargo test git`（11 lib-filtered tests + 2 bin-filtered tests passed）、`cargo test release_preflight`（6 passed）、`cargo run -- release preflight 0.1.0`（本地 checks ok，live verification pending）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（150 lib tests + 10 bin tests passed）。
- 本阶段没有实现真实 git polling source、GitHub source、tmux source、cron、memory、真实 live verification、Slack sink 或 Hermes plugin/observer。
- 已提交：`1536b6a feat: 增加 Git Source 本地事件路径`。

### Milestone 8.2：GitHub Source 本地 deterministic parity

- 已新增 `src/source/github.rs`，实现 `GithubIssueInput`、`GithubPullRequestInput`、`GithubCheckInput`、`GithubReleaseInput` 以及 `issue_opened_event()`、`pull_request_opened_event()`、`check_failed_event()`、`release_published_event()`。
- GitHub source 当前只基于显式 CLI 输入构造 `IncomingEvent`；不访问真实 GitHub API、不注册 webhook receiver、不依赖外网、不读取 GitHub token 或 webhook secret。
- GitHub source 与 compat 均会拒绝无效 number、无效 status、非 7-64 hex commit、多行 title 和过长 display field，避免畸形 payload 绕过 raw renderer 字段级防护。
- 已新增 typed GitHub body：`GithubIssueEvent`、`GithubPullRequestEvent`、`GithubCheckEvent`、`GithubReleaseEvent`，并将 `github.issue-opened`、`github.pr-opened`、`github.check-failed`、`github.release-published` 接入 `IncomingEvent -> EventEnvelope` conversion。
- 已接入 CLI：`hermeship github issue-opened`、`hermeship github pr-opened`、`hermeship github check-failed`、`hermeship github release-published`，命令复用现有 `DaemonClient::post_event()` 投递 `/event`。
- 已扩展 router/renderer：route filter 可用 owner、repo_name、number、branch、base_branch、workflow、status、tag 等结构化字段；默认 compact 渲染输出 repo/owner/编号/branch/status/tag/title/author 摘要。
- raw JSON 渲染只输出受控 GitHub 字段，不展开完整 issue/PR body、URL、provider response、token、cookie 或 secret。
- 已根据代码审查修复 GitHub route metadata poisoning：router filter 中的 `repo_name` 由已校验 typed body 覆盖，避免直接 POST 用原始 `repo_name` 绕过 body repo。
- 已更新公开命令 fixture、release preflight 检查和方案 CLI 示例，要求覆盖四个 GitHub 公开命令。
- 已覆盖测试：source 构造、typed conversion、route metadata filter、route metadata poisoning 回归、CLI parse、public command fixture、daemon submit、renderer 隐私边界和 release preflight。
- 已运行 Red：`cargo test github` 在实现前失败于缺少 `source::github` API、`GithubCommands`、`Commands::Github` 和 GitHub typed body variants；review regression 测试在修复前失败于 route metadata poisoning 与 docs preflight 覆盖缺口。
- 已运行验证：`cargo test github`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`（本地 checks ok，live verification pending）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 本阶段没有实现真实 GitHub API source、GitHub webhook receiver、GitHub credential handling、tmux source、cron、memory、真实 live verification、Slack sink 或 Hermes plugin/observer。
- 已提交：`91d13d8 feat: 完成 GitHub Source 本地确定性路径并修复回归`。

### Milestone 8.3：Tmux Source 本地 deterministic parity

- 已新增 `src/source/tmux.rs`，实现 `TmuxKeywordInput`、`TmuxStaleInput`、`TmuxWatchInput`、`TmuxPane`、`TmuxWatchPlan` 以及 `keyword_event()`、`stale_event()`、`parse_tmux_panes_output()`、`watch_plan_from_output()`、`format_watch_plan()`、`format_pane_list()`。
- Tmux source 当前只基于显式 CLI 输入和 fake tmux output 构造本地 deterministic 事件/报表；不调用真实 `tmux`、不读取真实 session、不启动真实 watch loop。
- 已新增 typed tmux body：`TmuxKeywordEvent`、`TmuxStaleEvent`，并将 `tmux.keyword`、`tmux.stale` 接入 `IncomingEvent -> EventEnvelope` conversion；`tmux.stale` 为 high priority。
- 已接入 CLI：`hermeship tmux keyword`、`hermeship tmux stale`、`hermeship tmux watch`、`hermeship tmux list`；keyword/stale 复用现有 `DaemonClient::post_event()` 投递 `/event`，watch/list 只输出本地 deterministic 报表。
- 已扩展 router/renderer：route filter 可用 session/session_name、window、pane、keyword、minutes；默认 compact 渲染输出 tmux 摘要，raw JSON 只输出受控字段，不展开 pane capture、buffer、完整 pane output、history、token、cookie 或 secret。
- 已根据代码审查收紧 `watch/list` 报表隐私边界：不再原样输出 fake tmux input 中的 command 或 last_line，只输出 command 是否存在和 last_line 字符数，并补充 token/path/authorization 回归测试。
- 已更新公开命令 fixture、release preflight 检查、README 和方案 CLI 示例，要求覆盖四个 tmux 公开命令；README watch/list 示例使用可复制的 tab 分隔 `$'...'` 形式。
- 已覆盖测试：source 构造、typed conversion、route metadata filter、CLI parse、keyword/stale daemon submit、watch/list deterministic 报表、报表隐私边界、invalid fake tmux output、renderer 隐私边界和 release preflight。
- 已运行 Red：`cargo test tmux` 在实现前失败于缺少 `source::tmux` API、`TmuxCommands`、`Commands::Tmux` 和 tmux typed body variants；审查回归测试在修复前失败于 `watch/list` 报表原样回显 command/last_line。
- 已运行验证：`cargo test tmux`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`（本地 checks ok，live verification pending）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 本阶段没有实现真实 tmux session 读取、真实 tmux watch、cron、memory、真实 live verification、Slack sink 或 Hermes plugin/observer。
- 已提交：`3745bb8 feat: 增加 tmux 事件 source`。

### Milestone 8.4：Cron 与 Memory Scaffold 本地 deterministic parity

- 已新增 `src/cron.rs`，从 `[[cron.jobs]]` 配置中解析 enabled job，并构造 `cron.run` 本地 deterministic `IncomingEvent`。
- Cron 当前只支持 `hermeship cron run <id>` 立即投递已配置 job；不实现真实 scheduler、不安装系统 cron、不启动外部 cron daemon。
- 已新增 `CronConfig` 与 `CronJob` 配置模型，默认没有 job，配置加载会归一化并验证 job id、5-field schedule 和单行 summary。
- 已新增 typed cron body：`CronRunEvent`，并将 `cron.run` 接入 `IncomingEvent -> EventEnvelope` conversion。
- 已接入 CLI：`hermeship cron run <id>` 复用现有 `DaemonClient::post_event()` 投递 `/event`。
- 已扩展 router/renderer：route filter 可用 `cron_job_id` 和 `cron_schedule`；默认 compact 渲染输出 job/schedule/summary，raw JSON 只输出受控 cron 字段，不展开 payload 中的 message、token 或 secret。
- 已新增 `src/memory.rs`，实现 `hermeship memory init/status` 的本地 filesystem scaffold：`MEMORY.md`、`memory/README.md`、daily/project/topic shards、可选 channel/agent shards，以及 handoffs/archive `.gitkeep`。
- Memory scaffold 要求显式 `--date <YYYY-MM-DD>`，并校验真实日历日期；默认不覆盖已存在文件，`--force` 才覆盖生成文件；root、目录和文件路径均拒绝 symlink，避免写入或扫描 root 外目标。
- 已更新公开命令 fixture、release preflight 检查、README 和方案 CLI 示例，要求覆盖 `cron run`、`memory init`、`memory status`。
- 已覆盖测试：cron config validation、cron source 构造、typed conversion、route metadata filter、CLI parse、daemon submit、renderer 隐私边界、memory init/status scaffold、幂等不覆盖、slug/date validation、symlink 拒绝和 release preflight。
- 已运行 Red：`cargo test cron` 在实现前失败于缺少 `CronCommands`、`CronConfig`、`CronJob`、`EventBody::CronRun` 和 `configured_run_event()`；`cargo test memory` 在实现前失败于缺少 `MemoryCommands`、memory CLI enum 和 memory API。
- 已运行验证：`cargo test cron`、`cargo test memory`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 本阶段没有实现真实 scheduler、系统 cron 安装、数据库 memory store、真实 live verification、Slack sink 或 Hermes plugin/observer。
- 已提交：`0b12de3 feat: 增加 cron 与 memory scaffold`。

### Milestone 9.1：README 与运维文档

- 已重写 `README.md`，将项目状态从计划态改为当前可执行入口，覆盖项目定位、安装、配置、daemon、Hermes hooks、send/emit/explain、local source commands、隐私、rollback、live check 和 release preflight。
- 已扩展 `docs/operations.md`，补齐 install、setup、config、daemon、Hermes hooks、smoke commands、source commands、service template、update、rollback、release preflight 和常见故障。
- 已新增 `docs/hermes-event-contract.md`，记录 Hermes hook envelope、`IncomingEvent`、canonical Hermes events、payload 字段、route metadata、rendering contract、privacy contract 和 fail-open 边界。
- 已新增 `ARCHITECTURE.md`，记录 daemon-first 数据流、模块边界、事件模型、router、renderer、sink、hook bridge、privacy、install/rollback 和验证策略。
- 已更新 `tasks/development-checklist.md` 的 Milestone 9.1 checkbox 和运行状态日志。
- 已确认本阶段不实现 Slack sink、Hermes plugin/observer 或真实 live verification；`docs/live-verification.md` 仍由 Milestone 9.2 创建。
- 已运行验证：`rg -n "hermeship start|hermes install-hooks|hermes.agent|Discord|rollback" README.md docs ARCHITECTURE.md`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 已提交：`1c52655 docs: 增加 Hermeship 运维与事件契约`。

### Milestone 9.2：Live Verification Runbook

- 已新增 `docs/live-verification.md`，作为真实 live verification runbook 和结果记录模板。
- 已覆盖 fake sink、daemon health、Discord live、Hermes gateway hook smoke 和 rollback 的步骤、字段和剩余风险记录。
- 已根据文档审查修正 runbook 可执行性：`hermeship start` 使用 Terminal A/B，rollback 检查 `.hermeship-managed.json`、`HOOK.yaml`、`handler.py` 残留，Current Results 分项记录各路径状态。
- 已明确真实 Discord/Hermes live check 本阶段未执行，原因是当前未提供 Discord credentials、测试频道、Hermes gateway 测试环境和显式执行确认。
- 已更新 `README.md`、本文件、`tasks/development-checklist.md` 和 `tasks/todo.md`，下一入口切换为 Milestone 9.3 首次 Live Check。
- 已运行验证：`rg -n "HERMES_HOME|Discord|hermeship status|agent:start|rollback" docs/live-verification.md`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 已提交：`2e60902 docs: 增加 live verification runbook`。

### Milestone 9.3：首次 Live Check 记录

- 已新增 `docs/live-verification.md` 的 Milestone 9.3 当前结果记录，状态为 `blocked`/`not_run`。
- 已明确真实 Discord/Hermes live check 本阶段未执行，原因是当前未提供 Discord credentials、测试频道、Hermes gateway 测试环境和显式执行确认。
- 已记录 daemon health、Discord live、Hermes gateway hook smoke 和 rollback 的未执行项、实际消息形态缺失和剩余风险。
- 已更新 `tasks/development-checklist.md`：真实 daemon、Discord、Hermes hook smoke 和 rollback 项保持未勾选；只勾选“凭据不可用时记录阻塞原因和剩余风险”及本阶段记录提交。
- 已更新 `tasks/todo.md`：记录 Milestone 9.3 live check 的实际状态和验证结果。
- 已运行验证：`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（all checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 已提交：`bc4c027 docs: 记录 Hermeship live verification 结果`。
- 当前结论：Milestone 9.3 已完成“未执行原因和风险记录”，但没有完成真实 live verification pass；进入 Milestone 10 前必须由用户明确确认是否豁免真实 live pass。

### 2026-06-20：本地验证续接与状态记录

- 已按启动要求复习 `tasks/lessons.md`，确认阶段完成后必须验证并提交，且不能把未验证、未完成或无关工作混入阶段提交。
- 已确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `b76a007 docs: 记录 Hermeship 本地验证续接`、`95a53d5 docs: 更新 Hermeship 最新开发状态与启动提示词`、`608704e docs: 记录 Hermeship 本地验证续接状态`、`c226514 docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`6053cdf feat: 增加 typed observer body 并收紧安全边界`。
- 已阅读本轮关键上下文：本文、开发清单、当前 todo、observer plugin open follow-ups、live verification 边界和 fixture 规范。
- 已将 `tasks/todo.md` 切换为本轮“2026-06-20 本地验证续接与状态记录”工作台。
- 本轮未提供 Discord credentials、测试频道、Hermes gateway 测试环境或明确执行确认，因此未执行真实 Discord/Hermes live check。
- 本轮没有真实 observer 使用反馈输入；`docs/observer-plugin.md` 的 open follow-ups 仍依赖真实环境或后续明确需求，因此本轮不修改功能代码。
- 本轮默认不实现 Slack sink，不自动启用 Hermes observer plugin，不新增 `docs/live-verification.md` 真实 pass 结果。
- 已重新运行默认本地验证：`python3 -m py_compile templates/hermes-plugin/__init__.py`、`cargo test observer_plugin`（13 passed）、`cargo test release_preflight`（16 passed）、`cargo run -- release preflight 0.1.0`（9 checks ok，`live verification` 输出为记录字段存在且不声明真实 pass）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（221 lib tests + 15 bin tests + doctests passed）。
- 已确认 `cargo run -- release preflight 0.1.0` 的 `live verification` check 只是验证 `docs/live-verification.md` 中 `日期`、`commit`、`Discord`、`Hermes`、`回滚` 等字段存在，不执行真实 Discord/Hermes live verification，也不代表真实 live pass。

### 2026-06-19：本地验证续接与状态记录

- 已按启动要求复习 `tasks/lessons.md`，确认阶段完成后必须验证并提交，且不能把未验证、未完成或无关工作混入阶段提交。
- 已确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `95a53d5 docs: 更新 Hermeship 最新开发状态与启动提示词`、`608704e docs: 记录 Hermeship 本地验证续接状态`、`c226514 docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`6053cdf feat: 增加 typed observer body 并收紧安全边界`、`4714fc9 docs: 更新 Hermeship 最新开发状态`。
- 已阅读本轮指定上下文：本文、开发清单、当前 todo、live verification runbook、README、架构、运维、事件契约、方案文档、release preflight 和 fixture 规范。
- 已将 `tasks/todo.md` 切换为本轮“本地验证续接与状态记录”工作台。
- 本轮未提供 Discord credentials、测试频道、Hermes gateway 测试环境或明确执行确认，因此未执行真实 Discord/Hermes live check。
- 本轮没有真实 observer 使用反馈输入，因此未修改功能代码；Slack sink 仍不在当前默认范围内。
- 已重新运行默认本地验证：`python3 -m py_compile templates/hermes-plugin/__init__.py`、`cargo test observer_plugin`（13 passed）、`cargo test release_preflight`（16 passed）、`cargo run -- release preflight 0.1.0`（9 checks ok，`live verification` 输出为记录字段存在且不声明真实 pass）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（221 lib tests + 15 bin tests + doctests passed）。
- 已确认 `cargo run -- release preflight 0.1.0` 的 `live verification` check 只是验证 `docs/live-verification.md` 中 `日期`、`commit`、`Discord`、`Hermes`、`回滚` 等字段存在，不执行真实 Discord/Hermes live verification，也不代表真实 live pass。
- 本轮只更新状态记录和当前工作台，不新增 `docs/live-verification.md` 真实结果；真实 Discord/Hermes live verification 仍待凭据、测试频道、Hermes gateway 测试环境和用户确认。

### 2026-06-18：Milestone 10 解锁与本地验证续接

- 已按启动要求复习 `tasks/lessons.md`，确认阶段完成后必须验证并提交，且不能把未验证、未完成或无关工作混入阶段提交。
- 已确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `92790ef docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`589c9e2 docs: 记录 Hermeship 本地验证续接状态`、`3f2e758 docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`9602856 docs: 记录 Hermeship 本地验证续接状态`、`01d601a docs: 更新 Hermeship 最新开发状态与下次启动提示词`。
- 已阅读本轮指定上下文：本文、开发清单、当前 todo、live verification runbook、README、架构、运维、事件契约、方案文档、release preflight 和 fixture 规范。
- 已将 `tasks/todo.md` 切换为本轮“本地验证续接与状态记录”工作台。
- 本轮未提供 Discord credentials、测试频道、Hermes gateway 测试环境或明确执行确认，因此未执行真实 Discord/Hermes live check。
- 已重新运行默认本地验证：`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（8 checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 已确认 `cargo run -- release preflight 0.1.0` 的 `live verification` check 只是验证 `docs/live-verification.md` 中 `日期`、`commit`、`Discord`、`Hermes`、`回滚` 等字段存在，不执行真实 Discord/Hermes live verification，也不代表真实 live pass。
- 用户已明确要求进入 Milestone 10，已记录“真实 live pass 被用户豁免”的决策；Milestone 10 解锁为 Hermes plugin / observer 研究入口。
- 本轮不新增 `docs/live-verification.md` 真实结果；Milestone 10.1 Observer 契约研究记录见下一节。

### 2026-06-18：Milestone 10.1 Observer 契约研究

- 已按用户确认继续 Milestone 10，完成 10.1 研究文档，不创建 plugin scaffold。
- 已新增 `docs/observer-plugin.md`，记录 Hermes plugin discovery、`plugins.enabled`/`plugins.disabled`、目录 plugin 结构、project plugin opt-in、entry point 机制和 fail-open 行为。
- 已定义 `hermes.observer.*` 事件 mapping，覆盖 session、API/LLM、tool、approval 和 subagent observer hooks。
- 已明确隐私边界：不转发 raw prompts、user messages、conversation history、request/response bodies、tool result bodies、完整 shell commands、完整 child goals 或 summaries。
- 已决定 10.1 不新增 Rust typed observer event bodies；Milestone 10.2 MVP 先通过 `POST /event` 发送通用 `IncomingEvent`，让 `hermes.observer.*` 使用 `Custom` fallback。
- 已确认 Milestone 10.2 后续范围：创建可选 plugin 模板、Python smoke 测试、必要时再扩展 release preflight；Slack sink 仍不在默认范围。
- 已运行验证：observer 文档关键词检查、`git diff --check`、`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（8 checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 已确认 `release preflight` 的 `live verification` ok 只证明 `docs/live-verification.md` 必填字段存在，不证明真实 Discord/Hermes live pass。
- 已提交：`93aa9ec docs: 完成 Hermes observer plugin 契约研究`。

### 2026-06-19：Milestone 10.2 后最新开发状态与下次启动提示词更新

- 已按用户要求更新最新开发状态入口和下次启动提示词，明确 Milestone 10.2 已由 `f352222 feat: 增加可选 Hermes observer plugin scaffold` 完成并提交。
- 已确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `f352222 feat: 增加可选 Hermes observer plugin scaffold`、`eb64408 docs: 更新 Hermeship 最新开发状态`、`93aa9ec docs: 完成 Hermes observer plugin 契约研究`、`0d0d354 docs: 记录 Hermeship 本地验证续接状态`、`92790ef docs: 更新 Hermeship 最新开发状态与下次启动提示词`。
- 已确认完成范围：Milestone 0 到 8.4、9.1、9.2 已完成并提交；Milestone 9.3 已完成 `blocked`/`not_run` 状态记录但真实 live pass 未完成；Milestone 10.1 Observer 契约研究已完成并提交；Milestone 10.2 Observer Plugin MVP scaffold 已完成并提交。
- 已确认未完成范围：真实 Discord/Hermes live verification pass、observer plugin install/enable CLI automation、typed Rust observer event body、真实 GitHub/tmux/scheduler/service automation、Slack sink。
- 本轮只更新状态记录和下次启动提示词，不修改功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink。
- 已更新 `tasks/todo.md` 为本轮“更新最新开发状态与下次启动提示词”工作台。
- 验证记录见 `tasks/todo.md` Review 和 `tasks/development-checklist.md` 最新运行状态日志。

### 2026-06-19：前次最新开发状态与下次启动提示词更新

- 已按用户要求更新最新开发状态入口和下次启动提示词，明确完成范围、未完成范围、阻塞项和下一步入口。
- 已确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `93aa9ec docs: 完成 Hermes observer plugin 契约研究`、`0d0d354 docs: 记录 Hermeship 本地验证续接状态`、`92790ef docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`589c9e2 docs: 记录 Hermeship 本地验证续接状态`、`3f2e758 docs: 更新 Hermeship 最新开发状态与下次启动提示词`。
- 已将最新 Milestone 10.1 契约研究提交记录为 `93aa9ec docs: 完成 Hermes observer plugin 契约研究`。
- 当时已确认 Milestone 0 到 8.4、Milestone 9.1、Milestone 9.2 已完成并提交；Milestone 9.3 只完成 blocked/not_run 记录，真实 live pass 未完成；Milestone 10.1 已完成并提交；Milestone 10.2 当时尚未启动。
- 当时只更新状态记录和下次启动提示词，不执行真实 Discord/Hermes live check，不实现 Slack sink，未创建 Hermes observer plugin scaffold。
- 已更新 `tasks/todo.md` 为本轮“最新开发状态与下次启动提示词更新”工作台。
- 已运行验证：状态文档一致性搜索、`git diff --check`、`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（8 checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 已确认 `release preflight` 的 `live verification` ok 只证明 `docs/live-verification.md` 必填字段存在，不证明真实 Discord/Hermes live pass。

### 2026-06-19：Milestone 10.2 Observer Plugin MVP scaffold

- 已新增 `templates/hermes-plugin/plugin.yaml`，定义可选 Hermes directory plugin `hermeship-observer`。
- 已新增 `templates/hermes-plugin/__init__.py`，使用 Python 标准库 `urllib.request` 直接向 Hermeship daemon `POST /event` 发送 `hermes.observer.*` summary event。
- 已注册 observer hooks：`on_session_start`、`on_session_end`、`on_session_finalize`、`on_session_reset`、`pre_api_request`、`post_api_request`、`api_request_error`、`pre_llm_call`、`post_llm_call`、`pre_tool_call`、`post_tool_call`、`pre_approval_request`、`post_approval_response`、`subagent_start`、`subagent_stop`。
- 已实现 fail-open safe-field forwarding：callback 始终返回 `None`；daemon unavailable、HTTP timeout、序列化或字段访问异常不会抛给 Hermes；支持 `HERMESHIP_DAEMON_URL`、`HERMESHIP_OBSERVER_TIMEOUT_SECS` 和 `HERMESHIP_OBSERVER_DISABLED`。
- 已保持隐私边界：只转发 provider/source/schema version、安全 id、状态、计数、长度、safe token usage 和 bounded error summary；不转发 raw prompt、conversation history、request/response body、shell command、tool output、tool result JSON、child goal 或 child summary。
- 已扩展 `src/release_preflight.rs`：新增 observer plugin template check，要求 `plugin.yaml` / `__init__.py` 和关键契约文本存在，且禁止 `transform_tool_result`、block action、`/api/hermes/hook`、Discord token 相关文本。
- 已新增本地 Python compile/smoke 覆盖：`python3 -m py_compile templates/hermes-plugin/__init__.py`；`cargo test observer_plugin` 使用 fake ctx 和 monkeypatched HTTP client 验证 hook 注册、`/event` payload、disabled 开关、fail-open 和 forbidden raw field 不泄漏。
- 已更新 `README.md`、`ARCHITECTURE.md`、`docs/operations.md`、`docs/hermes-event-contract.md` 和 `docs/observer-plugin.md`，说明 10.2 scaffold、手动安装启用方式、preflight 覆盖和剩余边界。
- 本轮未执行真实 Discord/Hermes live check，未新增 `docs/live-verification.md` 真实 pass 结果，未实现 Slack sink，未新增 observer install/enable CLI，未新增 typed Rust observer body。

### 2026-06-18：最新开发状态与下次启动提示词更新

- 已按用户要求更新最新开发状态入口和下次启动提示词，明确完成范围、未完成范围、阻塞项和下一步入口。
- 已确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `589c9e2 docs: 记录 Hermeship 本地验证续接状态`、`3f2e758 docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`9602856 docs: 记录 Hermeship 本地验证续接状态`、`01d601a docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`228f8f8 docs: 记录 Hermeship 本地验证续接状态`。
- 已将最新状态续接提交记录为 `589c9e2 docs: 记录 Hermeship 本地验证续接状态`。
- 已确认 Milestone 0 到 8.4、Milestone 9.1、Milestone 9.2 已完成并提交；Milestone 9.3 只完成 blocked/not_run 记录，真实 live pass 未完成；Milestone 10 未启动。
- 本轮不执行真实 Discord/Hermes live check，不记录“真实 live pass 被用户豁免”，不启动 Milestone 10、Slack sink 或 Hermes plugin/observer。
- 已更新 `tasks/todo.md` 为本轮“最新开发状态与下次启动提示词”工作台。
- 已准备下次启动提示词，要求下次先复习 lessons、确认 git 状态、阅读状态文档，并在没有真实 live 条件或用户豁免前不进入 Milestone 10。
- 已运行验证：关键词 `rg` 通过；`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（8 checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 本轮未修改功能代码，不新增 `docs/live-verification.md` 真实结果。

### 2026-06-18：本地验证续接与状态记录

- 已按启动要求复习 `tasks/lessons.md`，确认阶段完成后必须验证并提交，且不能把未验证、未完成或无关工作混入阶段提交。
- 已确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `3f2e758 docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`9602856 docs: 记录 Hermeship 本地验证续接状态`、`01d601a docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`228f8f8 docs: 记录 Hermeship 本地验证续接状态`、`b9fcaed docs: 更新 Hermeship 最新开发状态与下次启动提示词`。
- 已阅读本轮指定上下文：本文、开发清单、当前 todo、live verification runbook、README、架构、运维、事件契约、方案文档、release preflight 和 fixture 规范。
- 已将 `tasks/todo.md` 切换为本轮“本地验证续接与状态记录”工作台。
- 本轮未提供 Discord credentials、测试频道、Hermes gateway 测试环境或明确执行确认，因此未执行真实 Discord/Hermes live check。
- 本轮未记录“真实 live pass 被用户豁免”的决策，因此未启动 Milestone 10、未实现 Slack sink、未研究 Hermes plugin/observer。
- 已重新运行默认本地验证：`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（8 checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 已确认 `cargo run -- release preflight 0.1.0` 的 `live verification` check 只是验证 `docs/live-verification.md` 中 `日期`、`commit`、`Discord`、`Hermes`、`回滚` 等字段存在，不执行真实 Discord/Hermes live verification，也不代表真实 live pass。
- 本轮只更新状态记录和当前工作台，不修改功能代码，不新增 `docs/live-verification.md` 真实结果。

### 2026-06-18：最新开发状态与下次启动提示词更新

- 已按用户要求更新最新开发状态入口和下次启动提示词，明确完成范围、未完成范围、阻塞项和下一步入口。
- 已确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `9602856 docs: 记录 Hermeship 本地验证续接状态`、`01d601a docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`228f8f8 docs: 记录 Hermeship 本地验证续接状态`、`b9fcaed docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`23133f9 docs: 记录 Hermeship 本地验证续接状态`。
- 已将最新状态续接提交记录为 `9602856 docs: 记录 Hermeship 本地验证续接状态`。
- 已确认 Milestone 0 到 8.4、Milestone 9.1、Milestone 9.2 已完成并提交；Milestone 9.3 只完成 blocked/not_run 记录，真实 live pass 未完成；Milestone 10 未启动。
- 本轮不执行真实 Discord/Hermes live check，不记录“真实 live pass 被用户豁免”，不启动 Milestone 10、Slack sink 或 Hermes plugin/observer。
- 已更新 `tasks/todo.md` 为本轮“最新开发状态与下次启动提示词”工作台。
- 已准备下次启动提示词，要求下次先复习 lessons、确认 git 状态、阅读状态文档，并在没有真实 live 条件或用户豁免前不进入 Milestone 10。
- 已运行验证：关键词 `rg` 通过；`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（8 checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 已确认 `cargo run -- release preflight 0.1.0` 的 `live verification` check 只证明 `docs/live-verification.md` 必填字段存在，不证明真实 Discord/Hermes live pass。
- 本轮未修改功能代码，不新增 `docs/live-verification.md` 真实结果。

### 2026-06-18：本地验证续接与状态记录

- 已按启动要求复习 `tasks/lessons.md`，确认阶段完成后必须验证并提交，且不能把未验证、未完成或无关工作混入阶段提交。
- 已确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `01d601a docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`228f8f8 docs: 记录 Hermeship 本地验证续接状态`、`b9fcaed docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`23133f9 docs: 记录 Hermeship 本地验证续接状态`、`28c6fc8 docs: 更新 Hermeship 最新开发状态与下次启动提示词`。
- 已阅读本轮指定上下文：本文、开发清单、当前 todo、live verification runbook、README、架构、运维、事件契约、方案文档、release preflight 和 fixture 规范。
- 已将 `tasks/todo.md` 切换为本轮“本地验证续接与状态记录”工作台。
- 本轮未提供 Discord credentials、测试频道、Hermes gateway 测试环境或明确执行确认，因此未执行真实 Discord/Hermes live check。
- 本轮未记录“真实 live pass 被用户豁免”的决策，因此未启动 Milestone 10、未实现 Slack sink、未研究 Hermes plugin/observer。
- 已重新运行默认本地验证：`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（8 checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 已确认 `cargo run -- release preflight 0.1.0` 的 `live verification` check 只是验证 live verification 文档字段存在，不代表真实 Discord/Hermes live pass。
- 本轮只更新状态记录和当前工作台，不修改功能代码，不新增 `docs/live-verification.md` 真实结果。

### 2026-06-18：最新开发状态与下次启动提示词更新

- 已按用户要求更新最新开发状态入口和下次启动提示词，明确完成范围、未完成范围、阻塞项和下一步入口。
- 已确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `228f8f8 docs: 记录 Hermeship 本地验证续接状态`、`b9fcaed docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`23133f9 docs: 记录 Hermeship 本地验证续接状态`、`28c6fc8 docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`659b8ff docs: 记录 Hermeship 本地验证续接状态`。
- 已将最新状态续接提交记录为 `228f8f8 docs: 记录 Hermeship 本地验证续接状态`。
- 已确认 Milestone 0 到 8.4、Milestone 9.1、Milestone 9.2 已完成并提交；Milestone 9.3 只完成 blocked/not_run 记录，真实 live pass 未完成；Milestone 10 未启动。
- 本轮不执行真实 Discord/Hermes live check，不记录“真实 live pass 被用户豁免”，不启动 Milestone 10、Slack sink 或 Hermes plugin/observer。
- 已更新 `tasks/todo.md` 为本轮“最新开发状态与下次启动提示词”工作台。
- 已准备下次启动提示词，要求下次先复习 lessons、确认 git 状态、阅读状态文档，并在没有真实 live 条件或用户豁免前不进入 Milestone 10。
- 已运行验证：关键词 `rg` 通过；`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（8 checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 本轮未修改功能代码，不新增 `docs/live-verification.md` 真实结果。

### 2026-06-18：本地验证续接与状态记录

- 已按启动要求复习 `tasks/lessons.md`，确认阶段完成后必须验证并提交，且不能把未验证、未完成或无关工作混入阶段提交。
- 已确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `b9fcaed docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`23133f9 docs: 记录 Hermeship 本地验证续接状态`、`28c6fc8 docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`659b8ff docs: 记录 Hermeship 本地验证续接状态`、`93e231a docs: 更新 Hermeship 最新开发状态与下次启动提示词`。
- 已阅读本轮指定上下文：本文、开发清单、当前 todo、live verification runbook、README、架构、运维、事件契约、方案文档、release preflight 和 fixture 规范。
- 本轮未提供 Discord credentials、测试频道、Hermes gateway 测试环境或明确执行确认，因此未执行真实 Discord/Hermes live check。
- 本轮未记录“真实 live pass 被用户豁免”的决策，因此未启动 Milestone 10、未实现 Slack sink、未研究 Hermes plugin/observer。
- 已重新运行默认本地验证：`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（8 checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 已确认 `cargo run -- release preflight 0.1.0` 的 `live verification` check 只是验证 live verification 文档字段存在，不代表真实 Discord/Hermes live pass。
- 本轮只更新状态记录和当前工作台，不修改功能代码，不新增 `docs/live-verification.md` 真实结果。

### 2026-06-18：最新开发状态与下次启动提示词更新

- 已按用户要求更新最新开发状态入口和下次启动提示词，明确完成范围、未完成范围、阻塞项和下一步入口。
- 已确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `23133f9`、`28c6fc8`、`659b8ff`、`93e231a`、`5d9f21f`。
- 已将最新状态续接提交记录为 `23133f9 docs: 记录 Hermeship 本地验证续接状态`。
- 已确认 Milestone 0 到 8.4、Milestone 9.1、Milestone 9.2 已完成并提交；Milestone 9.3 只完成 blocked/not_run 记录，真实 live pass 未完成；Milestone 10 未启动。
- 本轮不执行真实 Discord/Hermes live check，不记录“真实 live pass 被用户豁免”，不启动 Milestone 10、Slack sink 或 Hermes plugin/observer。
- 已更新 `tasks/todo.md` 为本轮“最新开发状态与下次启动提示词”工作台。
- 已运行验证：关键词 `rg` 通过；`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（8 checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 本轮未修改功能代码，不新增 `docs/live-verification.md` 真实结果。

### 2026-06-18：本地验证续接与状态记录

- 已按启动要求复习 `tasks/lessons.md`，确认阶段完成后必须验证并提交，且不能把未验证、未完成或无关工作混入阶段提交。
- 已确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `28c6fc8 docs: 更新 Hermeship 最新开发状态与下次启动提示词`、`659b8ff docs: 记录 Hermeship 本地验证续接状态`、`93e231a docs: 更新 Hermeship 最新开发状态与下次启动提示词`。
- 已阅读本轮指定上下文：本文、开发清单、当前 todo、live verification runbook、README、架构、运维、事件契约、方案文档、release preflight 和 fixture 规范。
- 本轮未提供 Discord credentials、测试频道、Hermes gateway 测试环境或明确执行确认，因此未执行真实 Discord/Hermes live check。
- 本轮未记录“真实 live pass 被用户豁免”的决策，因此未启动 Milestone 10、未实现 Slack sink、未研究 Hermes plugin/observer。
- 已重新运行默认本地验证：`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（8 checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 已确认 `cargo run -- release preflight 0.1.0` 的 `live verification` check 只是验证 live verification 文档字段存在，不代表真实 Discord/Hermes live pass。
- 本轮只更新状态记录和当前工作台，不修改功能代码，不新增 `docs/live-verification.md` 真实结果。

### 2026-06-18：最新开发状态与下次启动提示词更新

- 已按用户要求更新最新开发状态入口和下次启动提示词，明确完成范围、未完成范围、阻塞项和下一步入口。
- 已确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `659b8ff`、`93e231a`、`5d9f21f`、`1841e0e`、`bc4c027`。
- 已将最新状态续接提交记录为 `659b8ff docs: 记录 Hermeship 本地验证续接状态`。
- 已确认 Milestone 0 到 8.4、Milestone 9.1、Milestone 9.2 已完成并提交；Milestone 9.3 只完成 blocked/not_run 记录，真实 live pass 未完成；Milestone 10 未启动。
- 本轮不执行真实 Discord/Hermes live check，不记录“真实 live pass 被用户豁免”，不启动 Milestone 10、Slack sink 或 Hermes plugin/observer。
- 已更新 `tasks/todo.md` 为本轮“最新开发状态与下次启动提示词”工作台。
- 已运行验证：关键词 `rg` 通过；`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（8 checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。

### 2026-06-18：前次本地验证续接与状态记录

- 已按启动要求复习 `tasks/lessons.md`，并确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `93e231a`、`5d9f21f`、`1841e0e`。
- 已阅读本轮指定上下文：本文、开发清单、当前 todo、live verification runbook、README、架构、运维、事件契约、方案文档、release preflight 和 fixture 规范。
- 本轮未提供 Discord credentials、测试频道、Hermes gateway 测试环境或明确执行确认，因此未执行真实 Discord/Hermes live check。
- 本轮未记录“真实 live pass 被用户豁免”的决策，因此未启动 Milestone 10、未实现 Slack sink、未研究 Hermes plugin/observer。
- 已重新运行默认本地验证：`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（8 checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 本轮只更新状态记录和当前工作台，不修改功能代码，不新增 `docs/live-verification.md` 真实结果。

### 2026-06-18：更早本地验证续接与状态记录

- 已按启动要求复习 `tasks/lessons.md`，并确认当前分支为 `codex/milestone-1-cli`；启动时工作树干净，最近提交为 `1841e0e`、`bc4c027`、`6be5661`。
- 已阅读本轮指定上下文：本文、开发清单、当前 todo、live verification runbook、README、架构、运维、事件契约、方案文档、release preflight 和 fixture 规范。
- 本轮未提供 Discord credentials、测试频道、Hermes gateway 测试环境或明确执行确认，因此未执行真实 Discord/Hermes live check。
- 本轮未记录“真实 live pass 被用户豁免”的决策，因此未启动 Milestone 10、未实现 Slack sink、未研究 Hermes plugin/observer。
- 已重新运行默认本地验证：`cargo test release_preflight`（12 passed）、`cargo run -- release preflight 0.1.0`（all checks ok）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（194 lib tests + 15 bin tests passed）。
- 本轮只更新状态记录，不修改功能代码，不新增 `docs/live-verification.md` 真实结果。

## 未完成范围

- Milestone 9.3 真实 Discord/Hermes live verification 尚未获得 `pass`：真实 daemon session、Discord custom message、Hermes sample event、Hermes gateway hook smoke 和 rollback 均未执行。
- Hermes observer plugin install/enable CLI automation 已实现；启用仍是 operator 手动执行，不由 Hermeship 自动修改 Hermes config。
- Hermes observer typed Rust event body 已实现；`hermes.observer.*` 当前通过 `EventBody::HermesObserver` 进入现有 pipeline。
- 真实 GitHub API source、真实 tmux watch、真实 scheduler 和真实 service manager 自动安装尚未实现。
- 默认 daemon queue 已有 dispatcher consumer；Discord sink 已实现并覆盖本地失败矩阵；daemon 到 fake sink 的本地 smoke 已覆盖，真实 Discord live delivery 尚未执行。
- live Discord verification 凭据是否可用尚未确认。
- Slack sink 不在当前默认范围内，除非开发清单明确更新。
- macOS launchd 是否与 systemd 同期实现尚未最终确认。

## 下一步入口

从 `tasks/development-checklist.md` 的 **Milestone 10：Hermes Plugin / Observer** 后续项继续。当前状态是：Milestone 10.1 Observer 契约研究、Milestone 10.2 Observer Plugin MVP scaffold、Milestone 10.3 Observer Plugin install/enable CLI 和 typed Rust observer body 均已完成；真实 Discord/Hermes live verification 仍未获得 `pass`，但用户已豁免该 live pass 对 Milestone 10 的阻塞。

建议第一段工作：

1. 复习 `tasks/lessons.md`、本文、方案文档、开发清单和 `tasks/todo.md`。
2. 确认当前分支、最新提交和未提交变更：
   - `git status --short --branch`
   - `git log -5 --oneline`
3. 确认最新提交包含本轮状态文档更新；最新状态续接提交为 `b76a007 docs: 记录 Hermeship 本地验证续接` 或本轮更新后的最新提交；最新 typed observer body 功能提交为 `6053cdf feat: 增加 typed observer body 并收紧安全边界`；最新 Milestone 10.3 功能提交为 `803aefa feat: 增加 Hermes observer plugin 安装启用 CLI`；最新 Milestone 10.2 功能提交为 `f352222 feat: 增加可选 Hermes observer plugin scaffold`；最新 Milestone 10.1 契约研究提交为 `93aa9ec docs: 完成 Hermes observer plugin 契约研究`；最新 live 记录提交为 `bc4c027 docs: 记录 Hermeship live verification 结果`。
4. 将当前任务计划写入 `tasks/todo.md`。
5. 阅读 `docs/live-verification.md`、`README.md`、`ARCHITECTURE.md`、`docs/operations.md`、`docs/hermes-event-contract.md`、`docs/plans/2026-06-15-hermeship-development-plan.md`、`src/release_preflight.rs`、`tests/fixtures/README.md`。
6. 如果用户提供 Discord credentials、测试频道、Hermes gateway 测试环境和明确执行确认，则继续 Milestone 9.3 真实 live check，并按 `docs/live-verification.md` 记录 daemon status、Discord custom message、Hermes sample event、Hermes gateway hook smoke、rollback、实际消息形态、未执行项和剩余风险。
7. 如果继续研发，优先根据实际需求选择真实 observer 使用反馈修正或补做 live verification；不要重新把真实 live pass 解释为已通过。
8. Slack sink 仍不在当前默认范围内，除非清单明确更新。
9. 运行对应验证命令，至少包含：
   - `python3 -m py_compile templates/hermes-plugin/__init__.py`
   - `cargo test observer_plugin`
   - `cargo test release_preflight`
   - `cargo run -- release preflight 0.1.0`
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`
10. 更新 `docs/development-status.md`、`tasks/development-checklist.md` 的运行状态日志和 `tasks/todo.md` 的 Review。
11. 阶段完成后必须验证并提交，commit 信息使用详细中文，说明变更、验证和影响。

## 下次启动提示词

```text
请在 /Users/zq/Desktop/ai-projs/posp/hermeship 继续 Hermeship 开发。

启动后请先阅读：
- tasks/lessons.md
- docs/development-status.md
- docs/plans/2026-06-15-hermeship-development-plan.md
- tasks/development-checklist.md
- tasks/todo.md

当前状态：
- 当前分支是 codex/milestone-1-cli。
- 最新 typed observer body 功能阶段提交：6053cdf feat: 增加 typed observer body 并收紧安全边界。
- 最新 Milestone 10.3 功能阶段提交：803aefa feat: 增加 Hermes observer plugin 安装启用 CLI。
- 最新 Milestone 10.2 功能阶段提交：f352222 feat: 增加可选 Hermes observer plugin scaffold。
- 最新状态续接提交：b76a007 docs: 记录 Hermeship 本地验证续接；如果本轮已提交更新，则以 git log -1 --oneline 为准。
- 最新状态文档提交：95a53d5 docs: 更新 Hermeship 最新开发状态与启动提示词。
- 最新 Milestone 10.1 契约研究提交：93aa9ec docs: 完成 Hermes observer plugin 契约研究。
- 最新 live 记录提交：bc4c027 docs: 记录 Hermeship live verification 结果。
- 最新 Milestone 9.3 交接提交：6be5661 docs: 更新 Hermeship Milestone 9.3 交接状态。
- 最新文档阶段提交：2e60902 docs: 增加 live verification runbook。
- 上一功能阶段提交：803aefa feat: 增加 Hermes observer plugin 安装启用 CLI。
- Milestone 0 到 Milestone 8.4 已完成并提交。
- Milestone 9.1 已完成并提交：README、docs/operations.md、docs/hermes-event-contract.md、ARCHITECTURE.md 已对齐当前能力和边界。
- Milestone 9.2 已完成并提交：docs/live-verification.md runbook 已创建。
- Milestone 9.3 已完成 blocked/not_run 记录：docs/live-verification.md 有未执行原因和剩余风险；真实 Discord/Hermes live verification 仍未获得 pass。
- Milestone 10 已解锁：10.1 Observer 契约研究已完成并提交，10.2 Observer Plugin MVP scaffold 已完成并提交，10.3 Observer Plugin install/enable CLI 已由 803aefa 完成并提交，typed Rust observer body 与安全 hardening 已由 6053cdf 完成并提交。
- docs/observer-plugin.md 已定义 observer plugin 需要遵守的 hook mapping、safe fields、fail-open、POST /event ingress、typed observer body、隐私边界和验证策略。
- templates/hermes-plugin/ 已提供可选 Hermes observer plugin 模板；`hermeship hermes install-plugin` 可显式安装模板，`hermeship hermes enable-plugin` 只输出启用指引，真正启用仍需 operator 执行 `hermes plugins enable hermeship-observer`。
- Hermeship 是 Hermes-native daemon-first event router，不是 thin adapter，不调用 clawhip runtime，也不依赖运行中的 clawhip daemon。
- 方案文档只维护架构和边界，执行进度维护在 tasks/development-checklist.md 和 tasks/todo.md。
- 默认测试只使用本地 deterministic fixture；真实 Discord/Hermes live verification 需要凭据、测试频道、Hermes gateway 测试环境和用户确认；`release preflight` 的 `live verification` ok 只证明记录字段存在，不断言真实 live pass。
- 默认不执行真实 Discord/Hermes live check，不实现 Slack sink；Hermes plugin / observer 的下一步是根据实际需求推进真实使用反馈或补做 live verification。

请从当前状态继续：
1. 先复习 tasks/lessons.md，并确认当前分支、最新提交和未提交变更：git status --short --branch、git log -5 --oneline。
2. 阅读 docs/development-status.md、tasks/development-checklist.md、tasks/todo.md、docs/live-verification.md、README.md、ARCHITECTURE.md、docs/operations.md、docs/hermes-event-contract.md、docs/plans/2026-06-15-hermeship-development-plan.md、src/release_preflight.rs、tests/fixtures/README.md。
3. 将本轮计划写入 tasks/todo.md。
4. 如果我提供 Discord credentials、测试频道、Hermes gateway 测试环境和明确执行确认，则继续 Milestone 9.3 真实 live check，并按 docs/live-verification.md 记录 daemon status、Discord custom message、Hermes sample event、Hermes gateway hook smoke、rollback、实际消息形态、未执行项和剩余风险；否则不要默认执行真实 Discord/Hermes live check。
5. 如果继续研发，基于已完成的 Milestone 10.3 Observer Plugin install/enable CLI 和 typed Rust observer body 继续：可考虑真实使用反馈修正或补做 live verification；默认不要实现 Slack sink。
6. 运行验证：python3 -m py_compile templates/hermes-plugin/__init__.py、cargo test observer_plugin、cargo test release_preflight、cargo run -- release preflight 0.1.0、cargo fmt --all -- --check、cargo clippy --all-targets -- -D warnings、cargo test。
7. 更新 docs/development-status.md、tasks/development-checklist.md 的运行状态日志和 tasks/todo.md 的 Review。
8. 阶段完成后必须验证并提交，commit 信息使用详细中文，说明变更、验证和影响。
```
