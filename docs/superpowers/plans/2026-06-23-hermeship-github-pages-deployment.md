# Hermeship GitHub Pages Deployment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 `https://zcxggmu.github.io/hermes-hip/` 通过 GitHub Pages 正常访问 Hermeship 静态官网。

**Architecture:** 使用 GitHub Actions 将仓库内 `site/` 作为 Pages 发布根目录，保持站点仍是纯静态 HTML/CSS/JS。同步保留 `gh-pages` 兼容发布源，并修正站内 GitHub/README 链接到当前真实仓库名，避免上线后跳错地址。

**Tech Stack:** GitHub Pages, GitHub Actions, static HTML/CSS/JS, shell validation.

---

### Task 1: Register deployment scope and file changes

**Files:**
- Modify: `tasks/todo.md`
- Create: `docs/superpowers/plans/2026-06-23-hermeship-github-pages-deployment.md`
- Modify: `site/index.html`

- [ ] **Step 1: Record the deployment plan**

```markdown
- [x] 确认当前 GitHub Pages URL 仍返回 404。
- [x] 确认本机没有 `gh` CLI 和可用 GitHub token。
- [x] 记录 fallback：Actions workflow + `gh-pages` 兼容分支。
```

- [ ] **Step 2: Review existing site links**

```bash
rg -n "hermes-hip|hermeship" site/index.html
```

Expected: locate the repo links that still point at `hermes-hip`.

### Task 2: Publish the static site

**Files:**
- Create: `.github/workflows/pages.yml`
- Create: `site/.nojekyll`
- Modify: `site/index.html`

- [ ] **Step 1: Add the Pages workflow**

```yaml
name: Deploy Hermeship Pages
on:
  push:
    branches: [main]
    paths:
      - "site/**"
      - ".github/workflows/pages.yml"
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: false

jobs:
  deploy:
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/configure-pages@v5
      - uses: actions/upload-pages-artifact@v3
        with:
          path: ./site
      - id: deployment
        uses: actions/deploy-pages@v4
```

- [ ] **Step 2: Add the Jekyll opt-out marker**

```text
# empty file
```

- [ ] **Step 3: Fix public repo links**

```html
<a class="nav__button" href="https://github.com/zcxGGmu/hermes-hip/blob/main/README.md" target="_blank" rel="noreferrer">README</a>
<a class="nav__button nav__button--ghost" href="https://github.com/zcxGGmu/hermes-hip" target="_blank" rel="noreferrer">GitHub</a>
```

### Task 3: Verify and publish

**Files:**
- Modify: `docs/development-status.md`
- Modify: `tasks/todo.md`

- [ ] **Step 1: Verify local static structure**

```bash
git diff --check
```

Expected: exit 0.

- [ ] **Step 2: Publish compatibility branch**

```bash
git subtree split --prefix site -b codex-pages-site
git push origin codex-pages-site:gh-pages
```

Expected: remote `gh-pages` receives the site root.

- [ ] **Step 3: Verify public URL**

```bash
curl -I -L --max-time 30 https://zcxggmu.github.io/hermes-hip/
```

Expected: HTTP 200 after Pages propagation.
