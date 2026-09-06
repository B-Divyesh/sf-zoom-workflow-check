# Repair verification — PASS

**Implementation SHA:** `1105c80f8411f32757dd801ee88ce73f46918863`
**Implementation date:** 2026-09-06
**Live URL:** <https://zoom-workflow-check.sociobot.in/>

## Commands

All passed after `npm ci`:

- `npm test` — 6 Rust tests and 14 browser checks, including all claim tags.
- `npm run lint` — formatting and clippy with warnings denied.
- `npm run test:e2e` — seeded high-zoom fixture detected at least 90% of
  known failures with zero known-good false positives.
- `npm run test:consumer` — package was installed into a fresh temporary
  consumer root and its bundled sample generated a report.
- `npm run build` — produced `dist/site` and `target/release/zoomcheck`.
- `cargo package -p zoomcheck` — passed package verification.
- `npm audit --audit-level=high` — zero vulnerabilities.

Each exact command listed in `.factory/claims.json` was also run and passed.

## Live evidence

Fresh desktop and phone browser contexts confirmed the job, audience, and
sample action on first screen. The phone action bottom was 577.9px in an 844px
viewport and horizontal overflow was 0px. `/demo/` showed the persistent demo
banner, four populated blocking findings, and a working reset action. A
service-worker-controlled offline reload retained the home heading.

`scripts/verify-url.sh` passed for `/` and `/demo/`. Playwright Axe on the
live home found no serious or critical WCAG 2A/AA/2.1AA violations. The browser
recorded no console errors and no third-party request during the landing/demo
flow. `/no-such-page` returned the expected HTTP 404 and `/privacy/` and
`/terms/` returned 200.
