# Zoom Workflow Check — build handoff

Work order: `zoom-workflow-check-build-1`

Version: `0.1.0`

Completed: 2026-08-28

## What shipped

- A Rust `zoomcheck` CLI with `record`, `check`, and `init` commands, helpful
  `--help`, deterministic exit codes (`0` pass, `1` findings, `2` invalid/run
  error), and `--json --quiet` CI output.
- A visible Chromium recorder that captures supported keyboard steps and stable
  focus selectors. Its browser binding persists across same-tab navigations;
  `Alt+Shift+S` saves the local workflow JSON.
- Chromium replay at 200% and 400% by default. It applies the browser-equivalent
  desktop layout model at the renderer boundary: reciprocal CSS viewport plus
  matching DPR (1280 px canvas → 640 CSS px at 200%, 320 CSS px at 400%). This
  exercises media queries, fixed layouts, overflow, focus scrolling, and
  high-density rendering; it does not use the CSS `zoom` property.
- Per-step checks for expected focus order, accessible name, detectable focus
  treatment, viewport clipping, ancestor clipping, obstruction, and available
  scroll paths.
- Local `report.json`, 200%/400% screenshots, and a responsive side-by-side HTML
  evidence report. No result or screenshot is uploaded.
- A Vite static documentation site in `dist/site`, including an interactive
  keyboard-operable report example, offline service worker, privacy and terms,
  security headers/config, and clear non-certification language.
- A product-specific risograph visual system and original generated hero asset.
  Source, generation metadata, prompt, provenance, and design tokens are in
  `.factory/assets/` and `.factory/design.md`.

## Verification

- `npm test` — passed: 6 Rust tests and 6 Playwright tests. Browser tests cover
  all public routes, one-h1/landmark expectations, serious/critical axe rules,
  keyboard tab behavior, 390 px horizontal fit, no console errors, and offline
  reload.
- `npm run test:e2e` — passed against the seeded Chromium fixture at both zoom
  levels: 10/10 known clipped or covered controls detected per run (100%) and
  0/10 known-good controls failed (0% false-positive failures). It also asserts
  the measured CSS viewports are approximately 640 px and 320 px respectively.
- `npm run build` — passed. Static output is exactly `dist/site/index.html`;
  release binary is `target/release/zoomcheck` (13 MB in this environment).
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo package -p zoomcheck` — passed and verified; package artifact is 25 KB
  compressed. The factory can publish later; no registry action was taken.
- `npm audit --audit-level=high` — passed with 0 vulnerabilities.
- Generated report visual inspection — passed at desktop width; the generated
  report also has zero serious/critical axe violations.

### Lighthouse mobile (local production preview)

- Performance: **99**
- Accessibility: **100**
- Best practices: **100**
- SEO: **100**
- FCP: **1.0 s**; LCP: **2.3 s**; CLS: **0**; TBT: **0 ms**

### Asset budgets

- Initial JS: 3.06 KB raw / 1.35 KB gzip (budget ≤200 KB)
- CSS: 14.13 KB raw / 4.11 KB gzip (budget ≤50 KB)
- Fonts: 0 KB; system stacks only (budget ≤120 KB)
- Hero WebP: 231,982 bytes / 227 KB (budget ≤300 KB)

## Run it

```sh
npm install
npm test
npm run test:e2e
npm run build
cargo run -p zoomcheck -- --help
```

Deploy `dist/site`. To prepare the CLI release artifact, run
`cargo package -p zoomcheck`; registry credentials and publishing remain with
the factory.

## Known v1 boundaries

- The recorder captures navigation/activation keys, not free-form text or
  pointer clicks. This keeps password and form values out of workflow files.
- Tests use a fresh Chromium profile. Authenticated checks should target a test
  environment whose URL establishes its own session; importing browser storage
  is intentionally not part of v1.
- Focus-indicator detection recognizes computed outlines and box shadows. Novel
  indicator techniques can appear as review warnings and should be checked in
  the screenshot.
- The HTML report captures the final viewport for each zoom plus structured
  geometry for every step, rather than one screenshot per keystroke, to limit
  sensitive artifact volume.
