# Hermeship Website Maturity Refresh Design

## Goal

Upgrade the published Hermeship static site so it feels closer in maturity to `gajae-code.com`: stronger product narrative, clearer next steps, a docs hub, evidence-oriented trust language, and a centered project icon in the first viewport.

## Scope

- Update `site/index.html`.
- Update `site/css/styles.css`.
- Add `site/docs/index.html`.
- Keep the project as static HTML/CSS/JS with no npm, Vite, Astro, Next, or generated frontend bundle.
- Keep public content Hermeship-native.
- Do not modify Rust feature code.
- Do not claim Slack sink support, real GitHub API polling, observer auto-enable, or real Discord/Hermes live verification pass.

## Design Direction

The refreshed homepage should keep the current dark developer-tool visual language, but move from a compact project overview toward a product-quality landing page:

- Hero: project icon centered in the overall first viewport, followed by concise headline, proof badges, and focused CTAs.
- What’s New: show current project state without overstating capability.
- Method: turn the internal six-step flow into a memorable public method: Observe, Scrub, Deliver.
- Quickstart: keep the copyable local smoke path.
- Capabilities: keep the truthful status matrix.
- Architecture: keep existing diagrams and explain their purpose.
- Evidence: reframe boundary language as proof and verification instead of only limitations.
- Docs Hub: provide a static `/docs/` index with deep links into existing repository docs.

## Information Architecture

Homepage order:

1. Hero with centered icon and brand lockup.
2. What’s New.
3. Method: Observe / Scrub / Deliver.
4. Quickstart.
5. Detailed workflow.
6. Capabilities matrix.
7. Architecture diagrams.
8. Evidence and boundaries.
9. Docs hub preview.
10. Footer.

Docs hub page order:

1. Header and navigation back to homepage.
2. Getting started links.
3. Architecture and event contract links.
4. Operations, live verification, observer plugin links.
5. Boundary note.

## Visual Requirements

- Project icon must be visually centered in the hero as an independent brand mark.
- Hero text should center on desktop and mobile before the two-column proof/demo content begins.
- Preserve 8px-or-less radius convention already used by the site.
- No decorative orbs, no one-note color expansion, no new frontend framework.
- Keep body text readable, mobile line lengths controlled, and no horizontal overflow.
- Use existing raster assets only; do not add new generated imagery in this pass.

## Verification

- `rg` checks for required sections and forbidden capability claims.
- `git diff --check`.
- Local static server content checks.
- Browser verification on desktop and mobile:
  - project icon centered in hero,
  - no horizontal overflow,
  - nav and copy button still work,
  - docs hub loads,
  - key assets render.
- Push to `main` and `gh-pages`, then verify `https://zcxggmu.github.io/hermes-hip/`.

