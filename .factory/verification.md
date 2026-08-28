# Independent verification — FAIL

**Work order:** `zoom-workflow-check-verify-1`
**Candidate:** `cffe72f545ffb7de528b8a0dc4807a265fdc02f6` (`cffe72f docs: record release verification and boundaries`)
**Live URL checked:** <https://zoom-workflow-check.sociobot.in/>
**Date:** 2026-08-28

## Verdict

**FAIL — do not release this candidate.** The required claims contract is absent,
the required one-click CLI sample/demo does not exist, and the exact Rust test,
production-build, lint, e2e, and package commands cannot complete in a clean
install. The core runner also uses Chromium device emulation rather than real
browser zoom semantics required by the researched brief.

## Mandatory first checks

### Claims

`.factory/claims.json` is **missing**. Consequently there were no claim tests
to run from the demo entry point. This is a release-blocking failure under the
claims contract.

The live landing page and README nevertheless make reliance-worthy claims,
including local screenshots/data, no telemetry/account/cloud storage, 200% and
400% replay, focus/clipping/obstruction/scroll checks, offline reload, and
specific CLI exit semantics. None has the required claim ID and sandbox test.
`.factory/demo.md` and `.factory/copy-audit.md` are also missing.

### Cold first read of the live page

In a new browser context, the first screen said **“Your workflow, under
pressure.”** It eventually explains recording/replaying keyboard paths at 200%
and 400%, but does not plainly name the intended small web teams/accessibility
consultants. The only first-screen actions are **“Install the CLI”** and
**“Inspect a report.”** There is no **“Try it with sample data”** action.

`https://zoom-workflow-check.sociobot.in/demo` returned **404**. The public CLI
only exposes `record`, `check`, and `init`; it has no `demo`/`--demo` command.
This fails the CLI demo-sandbox requirement independently of the missing claims
file.

## Test and build evidence

Fresh setup: `npm ci` completed successfully (21 packages, 0 audit
vulnerabilities). Tests were run in `/work/repo` at the candidate commit.

| Command | Result | Evidence |
| --- | --- | --- |
| Claim tests from `.factory/claims.json` | **BLOCKED/FAIL** | Required file is absent; no tests can be enumerated or run. |
| `npm test` | **FAIL** | Its first stage, exact `cargo test --workspace`, exits 101 while compiling `headless_chrome v1.0.22`; `rustc` is killed with `SIGKILL` before any Rust test runs. `dist/site` was not produced by the command. |
| `npm run build` | **FAIL** | Vite site build completes, then release compilation of `headless_chrome` exits 101 with `SIGKILL`; no release `zoomcheck` binary is produced. |
| `cargo clippy --workspace --all-targets -- -D warnings` | **FAIL** | Exits 101 at the same `headless_chrome` compile, killed by `SIGKILL`. |
| `npm run test:e2e` | **FAIL** | Exits 1: its `cargo run` build dies with `SIGKILL`, then the script reports `expected finding exit code 1, got 101`. |
| `cargo package -p zoomcheck` | **FAIL** | Package-tarball verification exits 101 at the same compile failure. A clean-consumer install/API test therefore cannot be performed. |
| Low-memory retry (`CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_DEV_CODEGEN_UNITS=256 cargo build -p zoomcheck`) | **FAIL** | Also exits 101; `headless_chrome` `rustc` is killed. |
| `npm run build:site` | **PASS** | Vite produced `dist/site` in 286 ms. |
| `npx playwright test` | **PASS** | 6/6 local static-site tests passed: routes/axe, tab interaction, 390px fit, and offline reload. |
| `npm audit --audit-level=high` | **PASS** | 0 vulnerabilities. |

The compiler process reached roughly 2.0 GB RSS before termination. This may
describe a resource constraint in the verifier, but the product's required
clean-clone quality gates still fail here and a release binary/public CLI could
not be independently exercised.

## Live deployment verification

The deployed static site matches the candidate's successful Vite output exactly:
`index.html`, `/privacy/`, `/terms/`, JS, CSS, hero WebP, favicon, `robots.txt`,
`sitemap.xml`, and `sw.js` all matched byte-for-byte. Relevant hashes:

| Asset | SHA-256 |
| --- | --- |
| `index.html` | `6547eb5460ecf970a16695b8975f7c66c5d608ba8505849eb03de5177bb46906` |
| `assets/home-DtR5jEDT.js` | `9070d742191928d9f5205d273fd601ef220d00fdb26583ba67e1661fcb26618d` |
| `assets/style-CnXGgOHk.css` | `22d838782d98bf7ddd420fb96b1b0e94ded5923acba623154e054f734aba8deb` |
| `zoom-field-sheet.webp` | `db3b91ccdda877a97b09a97312475d902203ba7e52ba4cf031e04566ff7da8db` |

This confirms the static documentation deployment, not a runnable Rust CLI
artifact. The site exposes no product API/server endpoint, so API rate limiting
is not applicable.

## Browser, accessibility, privacy, and performance checks

- Live desktop and 390px runs: one `h1`, `lang="en"`, `main`, title, skip link,
  meaningful hero-image alt text, no horizontal overflow at 390px, and visible
  yellow 3px focus outlines plus dark offset rings while tabbing.
- Keyboard: first eight Tab stops were reachable in order (skip link, wordmark,
  nav, hero actions, copy control). The report tab changes to 400% and returns
  to 200% with ArrowLeft.
- `@axe-core/playwright` on the live home page: **0 serious/critical**
  WCAG 2A/AA/2.1AA violations. No console errors or page errors occurred.
- Reduced-motion context at 390px loaded without errors; live offline reload
  after service-worker activation kept the heading available.
- Network observation over the full landing-page flow recorded only
  `https://zoom-workflow-check.sociobot.in`; no third-party script/font/request
  was made. This is observational evidence only, not the required tested
  privacy claim.
- Headers on HTML include HSTS, `X-Content-Type-Options: nosniff`, restrictive
  same-origin CSP, referrer policy, and permissions policy. Hashed JS/CSS/WebP
  have `max-age=31536000, immutable`; HTML and `sw.js` use 30-second
  revalidation.
- Static payload budgets pass: JS 3.06 KB raw / 1.35 KB gzip, CSS 14.13 KB raw
  / 4.11 KB gzip, no fonts, hero WebP 231,982 bytes (<300 KB).
- Local `verify-url.sh` requested by the accessibility work order is absent, so
  it could not be run; the direct browser checks above covered its stated
  title/lang/main/alt/console scope.

## Defects

### Blocker

1. **Required claim contract and sandbox proof are absent.**
   `.factory/claims.json` is missing, so every claim lacks the required tagged,
   demo-entry-point test. This fails the explicit acceptance contract.

2. **No one-click, isolated sample demo for this CLI.**
   The first screen has no “Try it with sample data”; `/demo` is a 404; there is
   no persistent demo banner/reset/start-for-real behavior; and the CLI has no
   `demo` command. `.factory/demo.md` is also absent.

3. **Clean quality gates and distributable CLI fail.**
   `npm test`, `npm run build`, clippy, e2e, and package verification all fail
   before the CLI executes because `headless_chrome` compilation is killed.
   The core job-to-be-done and clean-consumer install could not be verified.

### High

4. **The runner does not use real browser zoom semantics required by the brief.**
   [`apply_browser_zoom`](../crates/zoomcheck/src/browser.rs) calls CDP
   `Emulation.setDeviceMetricsOverride` with reciprocal viewport dimensions and
   DPR. That emulates device metrics; it does not invoke Chromium desktop page
   zoom. The brief specifically constrains the product to real browser zoom
   semantics, so the central result cannot be accepted as the promised check.

### Medium

5. **First screen fails the plain-words contract.**
   “Your workflow, under pressure.” is a metaphor rather than a <=9-word
   statement of the job, does not name the audience, and offers install/report
   inspection rather than a no-setup first action.

6. **At least one touch target is below the required 44px height.**
   On desktop and 390px mobile, the “Read the full methodology” link measured
   259 x 18 CSS px.

7. **Required verification/copy artifacts are missing.**
   In addition to claims/demo documentation, there is no
   `.factory/copy-audit.md` and no repository `verify-url.sh`.

## What would be needed for a re-verification

1. Add the claims manifest and one tagged, observable sandbox test per listed
   claim; remove claims that cannot be proven.
2. Ship `zoomcheck demo` (or `--demo`) using bundled realistic sample input,
   printing the temporary output location, plus an on-page terminal recording
   and `/demo`/documentation that meet the sandbox contract.
3. Make the documented clean commands build reliably within the intended CI
   resources, then provide a package that can be installed into a fresh
   consumer for CLI verification.
4. Implement and prove actual desktop browser zoom, or explicitly change the
   product/brief contract before release.
5. Repair first-screen copy and the undersized touch target, then rerun the
   full claim, CLI, PWA, and live-site suite.
