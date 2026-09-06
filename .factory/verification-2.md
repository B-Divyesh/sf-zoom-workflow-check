# Verify keyboard workflows at high zoom — independent QA

**Work order:** `zoom-workflow-check-verify-2`  
**Verdict:** **FAIL**  
**Finding count:** 4  
**Untested public claim count:** 9  
**Implementation reviewed:** `1105c80f8411f32757dd801ee88ce73f46918863`  
**Documentation base:** `4a2dae09421c771bd992bdaa9a118afc1b207090`  
**Live URL:** <https://zoom-workflow-check.sociobot.in/>  
**Date:** 6 September 2026

## Verdict

**FAIL — do not declare this candidate accepted.** The repaired CLI, package,
demo, deployment, and all six declared claim commands work. However, the live
landing page has one serious Axe violation at phone width, nine public claims
still lack complete claim-contract coverage, the demo caption overlaps its
terminal frame, and route metadata is incomplete. PASS requires zero findings
and zero untested claims.

## First screen

Before scrolling in fresh 1440×900 and 390×844 Chromium contexts:

- Job: **Check keyboard workflows at high zoom**.
- Audience: small web teams and accessibility consultants.
- First action: **Try it with sample data**.

The action was fully above the fold: its bottom was 771 CSS px on desktop and
578 CSS px on phone. The phone page had 0 px horizontal document overflow.

## Candidate and live identity

The implementation is `1105c80`. The checked-out documentation base is
`4a2dae0`; the only changes between those commits are
`.factory/repair-verification.md` and `.factory/handoff.md`.

The fresh `npm run build:site` output matched the live site byte-for-byte for
home, demo, privacy, terms, 404, service worker, robots, sitemap, icons, and
both product images. This proves the live runtime is the implementation
candidate, not a later product build.

## Declared claim commands

Each exact command from `.factory/claims.json` ran independently after
`npm ci`. All six passed.

| Claim | Exact command | Result |
| --- | --- | --- |
| Offline reload | `npm test -- --grep @claim:offline-reload` | PASS, 1 test |
| Same-origin site requests | `npm test -- --grep @claim:site-local-requests` | PASS, 1 test |
| Bundled CLI demo | `npm test -- --grep @claim:bundled-cli-demo` | PASS, 1 test |
| No account required | `npm test -- --grep @claim:no-account-required` | PASS, 1 test |
| Native desktop zoom | `npm test -- --grep @claim:native-desktop-zoom` | PASS, 1 test |
| Blocking focus finding | `npm test -- --grep @claim:focus-blocking-findings` | PASS, 1 test |

### Public claims without complete claim coverage

The live copy and README make nine additional or broader reliance-worthy
claims without the required exact manifest entry and tagged sandbox test.
Manual observations and untagged tests do not replace that contract.

1. The whole guide and sample reload offline. The declared test asserts only
   `/demo/`, although the live notice and README describe the guide more
   broadly.
2. `record` captures a real keyboard path that `check` can replay. No
   end-to-end record test is declared.
3. The runner checks expected focus order and accessible names.
4. The runner detects an outline or box-shadow focus treatment.
5. The runner detects clipping by overflow ancestors.
6. The runner detects sampled-point obstruction by another element.
7. The runner records whether the document or a container can scroll to
   expose focus.
8. The CLI adds no telemetry, upload, advertising, or product-server request
   and writes evidence only to the selected local output.
9. Scripting output follows the documented contract: exit 0 for a pass, 1 for
   findings, 2 for invalid input, and one JSON object with `--json --quiet`.

The verifier directly observed the full offline cache and all three exit
codes, and source inspection found no product-server call. These observations
show likely correct behavior, but the claims contract requires durable tagged
tests for every public claim.

## Earlier findings

| Earlier verification finding | Current disposition |
| --- | --- |
| Claims file missing | Partly fixed. Six declared claims pass, but public coverage remains incomplete (Finding 1). |
| No one-click sample or `/demo` | Fixed. The first action opens `/demo/` and `zoomcheck demo` ships in the package. |
| Clean gates and package failed | Fixed. All clean gates and consumer install passed. |
| Device emulation instead of desktop zoom | Fixed. The runner writes Chromium's native profile zoom preference; measured CSS widths and DPR values were correct. |
| Metaphorical first screen and no sample action | Fixed. Job, audience, and action are plain and above the fold. |
| 18 px methodology touch target | Fixed. Visible phone links and buttons measured at least 44×44 CSS px. |
| Missing copy, demo, and URL verification files | Fixed. All three files exist and the URL verifier passed expected 200 routes. |

## Clean checkout and installed artifact

`npm ci` installed 21 packages with zero reported vulnerabilities.

| Command | Result |
| --- | --- |
| `npm test` | PASS — 6 Rust tests and 14 Playwright tests |
| `npm run lint` | PASS — formatting and clippy with warnings denied |
| `npm run test:e2e` | PASS — at least 90% known-failure detection and 0% known-good false positives at both zoom levels |
| `npm run test:consumer` | PASS — packaged install and bundled sample in a fresh consumer root |
| `npm run build` | PASS — `dist/site` and release CLI produced |
| `cargo package -p zoomcheck` | PASS — 23.1 KiB compressed package verified |
| `npm audit --audit-level=high` | PASS — zero vulnerabilities |

I also installed the packaged crate into a second fresh temporary consumer
root and exercised these paths:

| Path | Result |
| --- | --- |
| `zoomcheck --help` | Exit 0 with commands and purpose |
| Bundled demo with JSON | Exit 1; one valid object, 4 findings, 2 runs, report and screenshots present |
| Ten known-good controls | Exit 0; `passed=true`, 0 failures, 2 runs |
| Missing workflow | Exit 2 with a clear file error |
| Unsupported 225% zoom | Exit 2 with the accepted levels |
| `init`, then `init` over the same file | Exit 0, then exit 2 without overwrite |
| Empty workflow | Exit 2 with the next action |
| 201-step workflow | Exit 2 and states the 200-step maximum |

The generated HTML report had one title, one H1, one main landmark, alt text on
both screenshots, no document overflow, no page errors, and zero serious or
critical Axe findings at 1280 px and 390 px.

## Live browser evidence

### Demo and storage

- `/demo/` returned 200 and immediately showed two populated zoom runs, the
  three-step path, and **4 blocking findings**.
- **Demo — sample data, nothing is saved** remained visible after reset.
- A seeded `zoomcheck:real-workflow=keep-me` local-storage value was unchanged
  after entering and resetting the demo.
- Reset worked by pointer and keyboard. **Start for real** opened `/#install`.

### Routes, links, offline, and privacy

- Home, demo, privacy, and terms returned 200 with their own titles, one H1,
  one main landmark, header, footer, skip link, and no console or page errors.
- `/no-such-page` deliberately returned HTTP 404 with the designed page and a
  working home action. The browser's failed-document console message is the
  expected result of that deliberate 404, not a defect.
- Every internal link resolved. The external source link returned 200.
- A fresh service-worker-controlled context reloaded home, demo, privacy,
  terms, and the designed 404 offline.
- The landing/demo/reset flow requested only
  `https://zoom-workflow-check.sociobot.in`.
- Reduced-motion media queries matched and reduced animation and transition
  durations to 0.01 ms.
- Security headers included CSP, HSTS, `nosniff`, referrer policy, and a
  permissions policy. Hashed assets and the hero image use immutable caching.

### Accessibility and performance

- Keyboard order started with skip link, wordmark, navigation, then the sample
  action. Focus used a 3 px yellow outline and dark separation ring.
- Default-width Axe checks found zero serious or critical issues on every
  route. The generated CLI report also passed at desktop and phone widths.
- At exactly 390 px, Axe found the serious issue in Finding 2.
- Visible phone links and buttons were at least 44×44 CSS px.
- Static payloads passed budget: 1.77 KiB gzip total JS, 4.96 KiB gzip CSS, no
  font download, and a 231,982-byte hero image.
- Lighthouse mobile output recorded performance 99, accessibility 100, best
  practices 100, SEO 100, LCP 2.0 s, CLS 0, and TBT 0 ms. Lighthouse emitted a
  tab-crash warning after writing the complete result; direct browser and Axe
  checks supplied the authoritative functional evidence.

## Findings

### 1. High — public claim coverage is incomplete

Nine public claims listed above lack a complete `.factory/claims.json` entry
and exact tagged sandbox assertion. This violates the attached claims contract
even though the six declared commands pass and several behaviors were observed
manually.

### 2. High — the phone command block is not keyboard-scrollable

At 390×844 on the live home page, Axe reports
`scrollable-region-focusable` with serious impact on the record/check `<pre>`.
The block has horizontal overflow but is neither focusable nor contains a
focusable control. A keyboard user cannot reach its hidden command text. The
desktop-only route Axe test and the existing phone size test do not combine
phone width with Axe, so both pass while this defect remains.

### 3. Low — demo caption text overlaps the terminal frame

On both fresh desktop and phone screenshots, the sentence beginning “Run
`zoomcheck demo` after installing” starts directly under the terminal's border
and 9 px orange shadow. The frame crosses the top of the text. The caption is
still readable, but the collision is visible in the primary sample flow.

### 4. Low — route social metadata is incomplete

Demo, privacy, and terms declare `twitter:card` but omit route-specific Twitter
title, description, and image fields. The designed 404 also omits canonical,
Open Graph, Twitter, apple-touch-icon metadata, and its product image. This does
not break the routes, but it does not meet the attached every-route metadata
contract.

## Not applicable

This is a static documentation site plus local CLI. It has no product backend,
tenant storage, payment flow, account system, shared database, live API rate
limit, or restart-persistence requirement. No AI feature is useful for the
narrow deterministic job in the brief.

## Required next work

1. Register and tag tests for every public claim, or remove the unsupported
   copy. Expand the offline claim test to the promised guide scope.
2. Make the phone command region keyboard-focusable or wrap its content without
   horizontal scrolling, then add a 390 px Axe regression test.
3. Add spacing below the terminal frame and complete route metadata.
4. Rerun every exact claim command, the clean consumer checks, phone Axe, and
   live byte comparison before requesting another independent verification.
