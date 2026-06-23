# Hermeship Static Website Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a single-page static Hermeship website that presents the project as a daemon-first Hermes event notification router with clear workflow, capabilities, architecture, and honest operating boundaries.

**Architecture:** Add a standalone `site/` directory containing plain HTML, CSS, and JavaScript. Reuse existing brand and diagram assets from `docs/assets/` through relative paths, and keep the site independent from Rust runtime behavior.

**Tech Stack:** Static HTML5, plain CSS, vanilla JavaScript, existing PNG/SVG assets, local static server for verification.

---

## File Structure

- Create: `site/index.html` - semantic single-page website content, navigation anchors, CTA links, image references, and code examples.
- Create: `site/css/styles.css` - dark developer-tool design system, responsive layout, accessible focus states, reduced-motion handling, and component styles.
- Create: `site/js/main.js` - mobile navigation, active section highlighting, and copy-to-clipboard enhancement.
- Create: `site/assets/branding/*` and `site/assets/diagrams/*` - copied existing PNG assets so `site/` can be served as an isolated static root.
- Modify: `tasks/todo.md` - track implementation progress and review notes.
- Reference only: `docs/superpowers/specs/2026-06-23-hermeship-static-website-design.md`, `README.md`, `README.en.md`, `docs/assets/branding/*`, `docs/assets/diagrams/*.png`.

### Task 1: Static HTML Content

**Files:**
- Create: `site/index.html`
- Modify: `tasks/todo.md`

- [x] **Step 1: Add implementation progress entry**

  Update the Hermeship static website task in `tasks/todo.md` so the confirmed shape and implementation start are recorded:

  ```markdown
  - [x] 确认网站形态：新增独立静态站点目录 `site/`，不引入 npm/Vite/Next/Astro。
  ```

- [x] **Step 2: Create semantic `site/index.html`**

  Create a complete HTML document with these sections and ids:

  ```text
  top
  quickstart
  workflow
  capabilities
  architecture
  boundaries
  ```

  The page must include:

  ```html
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <meta name="description" content="Hermeship 是面向 Hermes 的 daemon-first 事件通知路由器。" />
  <link rel="stylesheet" href="./css/styles.css" />
  <script src="./js/main.js" defer></script>
  ```

- [x] **Step 3: Add honest product copy**

  Include these claims in the page content:

  ```text
  Hermeship 是面向 Hermes 的 daemon-first 事件通知路由器。
  接收 Hermes gateway hooks、可选 observer plugin、CLI 和本地 deterministic source 事件。
  事件经过 normalize、privacy scrub、route、render 和 sink delivery。
  ```

  Also include these explicit boundaries:

  ```text
  GitHub API polling 仍是后续范围。
  Slack sink 不在默认范围。
  Real Discord/Hermes live verification pass 未获得。
  Observer plugin 需要 operator 显式安装并手动启用。
  ```

- [x] **Step 4: Add CTA and terminal examples**

  Use these actions:

  ```text
  查看 README -> https://github.com/zcxGGmu/hermes-hip/blob/main/README.md
  English README -> https://github.com/zcxGGmu/hermes-hip/blob/main/README.en.md
  架构文档 -> https://github.com/zcxGGmu/hermes-hip/blob/main/ARCHITECTURE.md
  GitHub -> https://github.com/zcxGGmu/hermes-hip
  ```

  Include this local smoke command block:

  ```bash
  cargo run -- start
  cargo run -- status
  cargo run -- explain hermes.agent.started --payload '{"session_id":"demo","platform":"telegram","project":"Hermeship"}'
  cargo run -- emit hermes.agent.started --payload '{"session_id":"demo","platform":"telegram","project":"Hermeship"}'
  cargo run -- release preflight 0.1.0
  ```

- [x] **Step 5: Verify HTML references**

  Run:

  ```bash
  rg -n "\.\/css\/styles\.css|\.\/js\/main\.js" site/index.html
  rg -n "GitHub API polling|Slack sink|live verification|Observer plugin" site/index.html
  ```

  Expected: matching lines show CSS/JS references and all required boundary statements.

### Task 2: CSS Design System and Responsive Layout

**Files:**
- Create: `site/css/styles.css`
- Modify: `tasks/todo.md`

- [x] **Step 1: Add CSS tokens**

  Define CSS variables for the static site:

  ```css
  :root {
    --bg: #090b10;
    --bg-alt: #10141d;
    --surface: #151a24;
    --surface-strong: #1b2230;
    --text: #f8fafc;
    --muted: #a8b3c7;
    --subtle: #748198;
    --line: rgba(255, 255, 255, 0.12);
    --brand: #f8c84a;
    --brand-strong: #ffad1f;
    --run: #38d996;
    --link: #7dd3fc;
    --danger: #fb7185;
    --shadow: 0 24px 70px rgba(0, 0, 0, 0.42);
    --radius: 8px;
    --max: 1180px;
    --sans: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    --mono: "JetBrains Mono", "SFMono-Regular", Consolas, "Liberation Mono", monospace;
  }
  ```

- [x] **Step 2: Style global layout**

  Implement:

  ```css
  * { box-sizing: border-box; }
  html { scroll-behavior: smooth; scroll-padding-top: 88px; }
  body { margin: 0; min-height: 100vh; overflow-x: hidden; background: var(--bg); color: var(--text); font-family: var(--sans); }
  img { max-width: 100%; display: block; }
  :focus-visible { outline: 2px solid var(--brand); outline-offset: 3px; }
  ```

- [x] **Step 3: Style core components**

  Add styles for:

  ```text
  .nav
  .hero
  .panel
  .terminal
  .flow
  .matrix
  .diagram-grid
  .boundary-list
  .site-footer
  ```

  Use border radius no larger than `8px` for cards, buttons, and panels.

- [x] **Step 4: Add responsive behavior**

  Add media queries so:

  ```text
  Desktop hero uses two columns.
  Mobile hero uses one column.
  Desktop diagrams can use two columns.
  Mobile diagrams use one column.
  Navigation links collapse behind a 44px hamburger button below 860px.
  ```

- [x] **Step 5: Add reduced-motion support**

  Add:

  ```css
  @media (prefers-reduced-motion: reduce) {
    *, *::before, *::after {
      animation-duration: 0.001ms !important;
      animation-iteration-count: 1 !important;
      scroll-behavior: auto !important;
      transition-duration: 0.001ms !important;
    }
  }
  ```

- [x] **Step 6: Verify CSS quality**

  Run:

  ```bash
  rg -n "border-radius: (9|1[0-9]|[2-9][0-9])px|orb|bokeh|emoji" site/css/styles.css
  git diff --check
  ```

  Expected: no matches for oversized radius/decorative terms, and no whitespace errors.

### Task 3: JavaScript Enhancements

**Files:**
- Create: `site/js/main.js`
- Modify: `tasks/todo.md`

- [x] **Step 1: Implement mobile nav**

  Add vanilla JavaScript that toggles `.is-open` on the nav links when the menu button is clicked, updates `aria-expanded`, closes on Escape, and closes after clicking a nav link.

- [x] **Step 2: Implement active section highlighting**

  Use `IntersectionObserver` to add `.is-active` to the matching navigation link as each section enters the viewport. If `IntersectionObserver` is unavailable, leave links usable without active state.

- [x] **Step 3: Implement copy enhancement**

  Add copy handling for buttons with `[data-copy-target]`. The handler should read text from the target element, call `navigator.clipboard.writeText` when available, and temporarily change the button label to `已复制`.

- [x] **Step 4: Verify script references**

  Run:

  ```bash
  rg -n "aria-expanded|IntersectionObserver|clipboard|data-copy-target" site/js/main.js site/index.html
  ```

  Expected: matching lines show all three enhancement paths.

### Task 4: Static Verification and Review

**Files:**
- Modify: `tasks/todo.md`
- Modify: `docs/superpowers/plans/2026-06-23-hermeship-static-website.md`

- [x] **Step 1: Serve the site locally**

  Run:

  ```bash
  python3 -m http.server 4187 --directory site
  ```

  Keep the server running while visual checks execute.

- [x] **Step 2: Verify desktop and mobile rendering**

  Use browser automation against:

  ```text
  http://127.0.0.1:4187/
  ```

  Check at:

  ```text
  1440x1000
  390x844
  ```

  Required observations:

  ```text
  Hero content is visible.
  Navigation is usable.
  No blank primary image area.
  No horizontal overflow.
  Boundary statements are visible.
  ```

- [x] **Step 3: Run static content checks**

  Run:

  ```bash
  rg -n "Hermeship|daemon-first|ingest|normalize|privacy scrub|GitHub API polling|Slack sink|live verification|Observer plugin" site/index.html
  rg -n -i -f /tmp/hermeship-site-forbidden-terms site/index.html
  git diff --check
  ```

  Expected: first command shows required content; second command has no matches; third command passes.

- [x] **Step 4: Update task review**

  Update `tasks/todo.md` Review with:

  ```text
  静态官网已实现于 `site/`。
  已复用现有品牌和图表资产。
  已验证桌面与移动端渲染、导航、边界声明和静态检查。
  未引入前端构建系统，未修改 Rust 功能代码。
  ```

- [x] **Step 5: Commit and push**

  Run:

  ```bash
  git status --short
  git add site docs/superpowers/plans/2026-06-23-hermeship-static-website.md tasks/todo.md
  git commit -m "site: 增加 Hermeship 静态官网"
  git push origin codex/milestone-1-cli
  git push origin HEAD:main
  ```

  Expected: the commit contains only the static site, plan, and task log.
