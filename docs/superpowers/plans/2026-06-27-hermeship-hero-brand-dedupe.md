# Hermeship Hero Brand Dedupe Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove repeated project identity marks from the published website opening area and keep one centered project icon in the homepage hero.

**Architecture:** Keep the static site structure unchanged. Update only homepage hero/nav markup and the small set of CSS selectors that control brand identity rendering. Do not modify Rust code, docs content, or capability claims.

**Tech Stack:** Static HTML, CSS, vanilla JavaScript, GitHub Pages.

---

### Task 1: Planning Record

**Files:**
- Modify: `tasks/todo.md`
- Create: `docs/superpowers/plans/2026-06-27-hermeship-hero-brand-dedupe.md`

- [x] Record the scope: homepage opening area only.
- [x] Record the success criteria: one centered project icon in the hero, no lockup image, no repeated hero title mark.

### Task 2: Homepage Markup

**Files:**
- Modify: `site/index.html`

- [x] Remove the hero lockup image from the homepage hero identity block.
- [x] Remove the duplicate homepage hero `h1` visual title.
- [x] Keep the centered `hero__icon-frame` as the only project icon in the homepage hero.
- [x] Change the homepage nav brand to text-only so the opening area does not show another project icon.
- [x] Keep accessible names and page title intact.

### Task 3: Styling

**Files:**
- Modify: `site/css/styles.css`

- [x] Add a text-only homepage brand modifier.
- [x] Remove unused homepage lockup styling if no longer referenced.
- [x] Adjust hero icon spacing so the single centered mark feels intentional.
- [x] Preserve mobile layout and no-horizontal-overflow constraints.

### Task 4: Verification and Publish

**Files:**
- Modify: `tasks/todo.md`

- [x] Run `rg` checks confirming homepage has exactly one `hero__icon-frame`, no `hero__lockup`, and no `brand__icon`.
- [x] Run `git diff --check`.
- [x] Run local static smoke checks for `/`.
- [x] Commit, push `codex/milestone-1-cli`, push `main`, update `gh-pages`, and verify the public URL.
