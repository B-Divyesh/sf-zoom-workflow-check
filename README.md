# Zoom Workflow Check

Zoom Workflow Check checks a real keyboard workflow at 200% and 400% Chromium
desktop zoom. It is for small web teams and accessibility consultants who need
evidence when focused controls become clipped, covered, or unreachable.

First, run `zoomcheck demo` to create a local report from the bundled checkout
sample. The sample is isolated from your workflow files and intentionally exits
`1` after finding known blocked controls.

The CLI uses Chromium’s native desktop page zoom setting, not CSS zoom or
device-metrics emulation. It writes reports and screenshots to the machine or
CI runner that executes it. No account is required.

## Install

Build from a checkout with Rust 1.85+:

```sh
cargo install --path crates/zoomcheck
zoomcheck --help
```

The CLI looks for Chromium through `--browser`, `ZOOMCHECK_BROWSER`, common
system locations, and `PLAYWRIGHT_BROWSERS_PATH`, in that order.

## Try the bundled sample

```sh
zoomcheck demo
```

The command copies a checkout-flyout fixture into a new temporary
`zoomcheck-demo-*` folder. It writes `report.json`, `index.html`, two
screenshots, and the copied sample there, then prints the folder path. The
fixture has a payment button beyond a fixed flyout edge, so exit code `1` means
the demonstration found its intended blocking findings. Use `--out` when a
specific disposable report directory is needed.

```sh
zoomcheck demo --out artifacts/zoomcheck-demo
```

## Record and check a workflow

Record the checkout path in a visible Chromium window. Finish with
`Alt+Shift+S`:

```sh
zoomcheck record https://shop.example.test/cart \
  --name "Complete checkout" \
  --out .zoomcheck/checkout.json
```

Replay at the default zoom levels and open the local HTML report:

```sh
zoomcheck check .zoomcheck/checkout.json --out zoomcheck-report
open zoomcheck-report/index.html
```

CI can consume one JSON object from stdout. Exit code `0` means no blocking
findings, `1` means the workflow completed with one or more blocking findings,
and `2` means the input or browser run was invalid.

```sh
zoomcheck check .zoomcheck/checkout.json \
  --json --quiet --out artifacts/zoomcheck
```

```text
zoomcheck record <URL> --name <NAME> [--out <FILE>] [--browser <PATH>]
zoomcheck check [WORKFLOW] [--zoom <PERCENT>...] [--out <DIR>]
                [--browser <PATH>] [--headful] [--json] [--quiet]
zoomcheck demo [--out <DIR>] [--zoom <PERCENT>...] [--browser <PATH>]
               [--headful] [--json] [--quiet]
zoomcheck init [--out <FILE>]
```

`zoomcheck init` writes an editable workflow when recording a test site is
inconvenient. A workflow contains a URL and keyboard steps: `Tab`, `Shift+Tab`,
`Enter`, `Space`, arrow keys, `Escape`, `Home`, and `End`.

## What the report checks

After every keyboard step, the runner records the focused element and checks:

- expected focus target and accessible name;
- outline or box-shadow focus treatment;
- viewport and overflow-ancestor clipping;
- sampled-point obstruction; and
- document or container scroll availability.

The report puts one run per selected zoom level alongside local screenshots.
This is engineering evidence, not WCAG conformance certification. Screenshots
can contain sensitive interface content, so review them before sharing.

## Develop and verify

Requirements: Rust 1.85+, Node 20+, and Chromium. Playwright’s Chromium is
supported when `PLAYWRIGHT_BROWSERS_PATH` is set.

```sh
npm ci
npm test
npm run lint
npm run test:e2e
npm run test:consumer
npm run build
cargo package -p zoomcheck
```

`npm test` runs Rust tests, builds the site, and runs browser checks including
the public claims in `.factory/claims.json`. `npm run test:e2e` exercises the
seeded suite at native zoom. `npm run test:consumer` installs the packaged CLI
into a temporary consumer root and runs its bundled sample. `npm run build`
creates `target/release/zoomcheck` and the deployable documentation site in
`dist/site/`.

The static documentation deploys from `dist/site` to
<https://zoom-workflow-check.sociobot.in>. The factory owns registry and
release credentials; this repository does not publish automatically.

## Privacy and license

The documentation site makes no third-party runtime request. After its first
visit, its service worker can reload the public guide offline. See
[Privacy](https://zoom-workflow-check.sociobot.in/privacy/) and
[Terms](https://zoom-workflow-check.sociobot.in/terms/). The project is MIT
licensed; see [LICENSE](LICENSE).
