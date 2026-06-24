# Hermeship Website Maturity Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve the published Hermeship static website so it more closely matches the depth and polish of `gajae-code.com`, while keeping all capability claims accurate.

**Architecture:** Keep a pure static site under `site/`. Add richer homepage sections and a lightweight docs hub page without adding a frontend build system. Use existing brand and diagram assets, and keep the public narrative Hermeship-native.

**Tech Stack:** Static HTML, CSS, vanilla JavaScript, GitHub Pages.

---

### Task 1: Planning Records

**Files:**
- Create: `docs/superpowers/specs/2026-06-24-hermeship-website-maturity-refresh-design.md`
- Create: `docs/superpowers/plans/2026-06-24-hermeship-website-maturity-refresh.md`
- Modify: `tasks/todo.md`

- [ ] Record the design scope and implementation checklist.
- [ ] Explicitly state that Rust feature code is out of scope.
- [ ] Explicitly state the capability boundaries that must not be overstated.

### Task 2: Homepage Narrative

**Files:**
- Modify: `site/index.html`

- [ ] Update navigation to include What’s New, Method, Evidence, and Docs.
- [ ] Rework hero so `hermeship-icon.png` is centered in the first viewport.
- [ ] Add a What’s New section with current status cards.
- [ ] Add a Method section using Observe / Scrub / Deliver.
- [ ] Keep Quickstart, Workflow, Capability, Architecture, and Boundary content accurate.
- [ ] Add a Docs preview section linking to `./docs/`.

### Task 3: Docs Hub

**Files:**
- Create: `site/docs/index.html`

- [ ] Add a static docs index with links to README, Architecture, Operations, Event Contract, Live Verification, and Observer Plugin.
- [ ] Keep the docs page visually consistent with the homepage.
- [ ] Keep all external links pointed at `https://github.com/zcxGGmu/hermes-hip`.

### Task 4: Styling

**Files:**
- Modify: `site/css/styles.css`

- [ ] Add styles for centered hero brand mark.
- [ ] Add styles for status cards, method cards, docs cards, and evidence timeline.
- [ ] Preserve mobile nav behavior and no-horizontal-overflow constraints.
- [ ] Ensure all new interactive targets remain at least 44px high.

### Task 5: Verification and Publish

**Files:**
- Modify: `tasks/todo.md`

- [ ] Run required content checks with `rg`.
- [ ] Run `git diff --check`.
- [ ] Run a local static server smoke check for `/` and `/docs/`.
- [ ] Verify desktop and mobile rendering with browser automation.
- [ ] Commit, push `codex/milestone-1-cli`, push `main`, update `gh-pages`, and verify the public URL.

