# Zoom Workflow Check — repair handoff

## Release result

Repair candidate implementation: `1105c80f8411f32757dd801ee88ce73f46918863`
(`fix: ship native zoom CLI demo`).

The static product was deployed with the product-specific Static Web Apps CLI
on 6 September 2026. The live HTTPS home now serves the repaired title and
**Try it with sample data** action; `/demo/` returns 200. The command generated
a local credential file, which was removed without being read or committed.

This product is a free CLI for small web teams and accessibility consultants.
Its job is to replay a named keyboard workflow at 200% and 400% Chromium
desktop zoom and save local focus evidence. The first action is **Try it with
sample data** on the landing page or `zoomcheck demo` after installation.

## What changed

- Replaced `headless_chrome` with a compact direct DevTools client. The old
  dependency was the source of the clean-build SIGKILL.
- Sets Chromium's persisted `partition.default_zoom_level` before launch. This
  is native desktop page zoom—the same browser setting changed by Chrome's
  zoom control—not CDP device-metrics emulation or CSS `zoom`.
- Added `zoomcheck demo`, bundled checkout-flyout input, local temporary
  report output, screenshots, and an expected finding exit code.
- Added `/demo/`, a persistent **Demo — sample data, nothing is saved** label,
  reset and start-for-real actions, a populated report, route metadata, a
  styled 404 page, direct route titles, offline caching, and 44px mobile
  interactive targets.
- Added the required claims manifest, claim-tagged observable browser/CLI
  checks, demo documentation, copy audit, local URL verifier, consumer package
  check, catalog description, share image, and device icon.

## Earlier verification findings

| Earlier finding | Current disposition |
| --- | --- |
| `claims.json` missing | Fixed. `.factory/claims.json` has six runnable, tagged claim checks. |
| No one-click CLI sample or `/demo` | Fixed. `zoomcheck demo` ships the input and `/demo/` shows its populated outcome. |
| Heavy CLI build was SIGKILLed | Fixed. Clean test, lint, release build, package, and consumer install complete without `headless_chrome`. |
| Device-metrics emulation was not real zoom | Fixed. The runner writes Chromium's native desktop zoom preference; its 200%/400% measured CSS viewports and DPR are asserted. |
| Metaphorical first screen and no sample action | Fixed. The live title states the job, names the audience, and shows the sample action before scrolling on desktop and phone. |
| 18px methodology touch target | Fixed. Interactive targets are checked at 390px and are at least 44px. |
| Missing copy/demo/URL verification artifacts | Fixed. `.factory/copy-audit.md`, `.factory/demo.md`, and `scripts/verify-url.sh` are present and exercised. |

The original independent failure remains preserved in
`.factory/verification.md` as historical evidence.

## Verification run from a clean setup

```sh
npm ci
npm test                 # 6 Rust tests + 14 browser/claim checks
npm run lint             # cargo fmt + clippy -D warnings
npm run test:e2e         # seeded suite: >=90% detection, 0% false positives
npm run test:consumer    # packaged CLI installed into a temporary consumer root
npm run build            # dist/site + target/release/zoomcheck
cargo package -p zoomcheck
npm audit --audit-level=high
```

All passed. `npm test -- --grep @claim:<id>` was also run for every entry in
`.factory/claims.json`. The package is 23.1 KB compressed in this environment.
The static build has 1.08 KB gzip home JS, 4.96 KB gzip CSS, no downloaded
fonts, and a 227 KB hero image.

Live cold checks on `https://zoom-workflow-check.sociobot.in` passed in fresh
1440px desktop and 390px phone contexts: job/audience/first action are above
the fold, `/demo/` has its persistent sample label and populated findings,
reset works, phone overflow is 0px, offline reload works after first visit,
no console errors occurred, and all observed requests were same-origin. Live
Axe WCAG 2A/AA/2.1AA found zero serious or critical violations. The local URL
verifier passed both `/` and `/demo/`. `/no-such-page` deliberately returns
404; the host serves the designed 404 page.

## Known boundaries and next steps

- `record` needs a visible desktop Chromium session and records supported
  keyboard actions only. It does not record text entry, pointer activity, or
  passwords.
- Workflow screenshots can contain sensitive page information. Keep reports in
  appropriate local or CI access controls.
- Registry publishing remains a factory operation. `cargo package -p
  zoomcheck` produces the ready-to-publish artifact; no registry credentials
  are stored here.
- This remains an engineering check, not accessibility certification or legal
  advice. A passing workflow does not prove all paths or assistive technology
  combinations work.
