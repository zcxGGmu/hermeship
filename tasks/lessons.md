# Lessons

## 2026-06-15: 阶段完成后立即验证并提交

- 用户明确要求：每完成一阶段任务，就提交一次。
- 后续会话启动后，先复习 `tasks/lessons.md`，并自动沿用该习惯，不需要用户再次提醒。
- 阶段提交前必须完成相应验证，确认 `git status` 只包含预期变更。
- commit 信息使用中文，并写清楚本阶段完成内容、验证结果和后续影响。
- 不要把未验证、未完成或无关的工作混入阶段提交。

## 2026-06-15: 方案文档与开发清单必须分离

- 用户纠正：方案和清单需要分离开，并转换为中文文档。
- 方案文档只写目标、边界、架构、契约、安全、验证和发布策略。
- 开发清单只写可勾选任务、阶段进度、验证命令、阻塞项和决策记录。
- 后续不要在方案文档中塞入详细执行 checklist；需要跟踪进度时更新 `tasks/development-checklist.md`。

## 2026-06-15: Hermeship 目标不是薄适配器

- 用户纠正：Hermeship 应完全参照 `template/clawhip` 的项目形态、架构和功能实现，只是面向 Hermes 做对应适配。
- 后续不能把目标误解为“调用现有 clawhip 的轻量 adapter”。
- 正确理解：Hermeship 应是 Hermes-native 的事件到通知渠道路由项目，参考 clawhip 的 daemon-first 架构、事件模型、路由、渲染、sink、CLI、安装、配置和 live verification。
- 评估和计划应围绕“复刻/移植 clawhip 能力并替换 OpenClaw/Codex/Claude 耦合为 Hermes 耦合”展开。

## 2026-06-21: README 多语言不要混排

- 用户纠正：中英文 README 混在同一个长文件里阅读体验不好。
- 后续新增或重写多语言 README 时，默认采用分文件入口：根 `README.md` 放中文，`README.en.md` 放英文。
- 两个 README 顶部都应提供清晰语言切换按钮或链接，避免用户需要在同一文件里上下滚动查找另一种语言。
- 更新 README 语言结构时，要保留真实能力边界声明，不能因为拆分语言而丢失 live verification、Slack sink、observer plugin 手动启用等限制。

## 2026-06-21: 公开 README 必须是 Hermeship 独立叙述

- 用户纠正：README 不要出现和 clawhip 相关的内容；Hermeship 对外应呈现为完全独立的项目。
- 后续编辑 `README.md` 和 `README.en.md` 时，不要写 `clawhip`、`template/clawhip`、adapter、thin adapter、runtime adapter 等关联表述。
- 内部开发计划可以保留历史参考来源和架构演进记录，但公开 README 应只描述 Hermeship 自身目标、架构、能力边界、安装配置、验证和运维方式。
- 移除外部参考表述时，仍必须保留真实能力边界声明：live verification 尚未通过、Slack sink 不在默认范围、observer plugin 需要显式安装和手动启用、source 命令仍是 deterministic-only。

## 2026-06-21: README 顶部品牌区不要用表格拼图

- 用户纠正：README 顶部图像和布局问题明显，表格拼接 wordmark 与图标会出现边框、竖线、比例失衡和风格割裂。
- 后续编辑公开 README 顶部品牌区时，优先使用仓库内统一 brand lockup/banner 资产，不用 HTML table 把不同背景的图片硬拼在一起。
- README 顶部应保持清晰层级：品牌图、项目标题语义和语言切换各自只承担一个职责，避免可见文案重复。
- 装饰性品牌图应使用空 `alt`；如果移除了可见项目标题，则品牌图应提供简短 `alt` 文本承载项目名。有意义的图表仍必须保留描述性 alt 文本。

## 2026-06-21: README banner 已含项目名时不要重复标题

- 用户纠正：README 顶部图片下方不应再重复显示 `Hermeship` 文字。
- 后续公开 README 顶部如果使用已包含项目名的 brand lockup/banner，不要在图片下方再放可见的项目名标题。
- 若因此移除可见 `h1`，banner 不再是纯装饰图，应提供简短 `alt` 文本承载项目名，避免读屏用户丢失入口识别。

## 2026-06-22: 远程推送默认主分支

- 用户明确要求：以后默认把本地提交推送到 `main`。
- 后续在未被特别说明的情况下，完成阶段提交后应优先同步到远程主分支，而不是只停留在功能分支。
- 若用户另行指定目标分支，则以用户指定为准。

## 2026-06-22: README 吸收外部思想时必须项目原生化

- 用户明确要求：基于外部文章或参考资料补充 README 时，不要写成“这篇文章认为”“参考某项目”的转述口吻。
- 公开 README 应把设计思想转化为 Hermeship 自身的项目原则、运行边界和工程判断，不暴露外部参考来源。
- 仍必须遵守公开 README 独立叙述规则，不写 `clawhip`、`template/clawhip`、thin adapter、runtime adapter 等关联表述。
