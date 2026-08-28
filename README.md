# Zoom Workflow Check

Zoom Workflow Check is a local-first CLI for small web teams and accessibility
consultants. It records a short keyboard journey once, replays it in Chromium
at 200% and 400% browser zoom, and produces a visual HTML report of focus order,
focus visibility, clipping, obstruction, and scroll reachability.

It is an engineering check, not a WCAG certification tool. Screenshots and
workflow data stay on the machine or CI runner where the command runs.

## Install

Download a release binary, or build it with a current Rust toolchain:

```sh
cargo install --path crates/zoomcheck
zoomcheck --help
```

The CLI looks for Chromium through `--browser`, `ZOOMCHECK_BROWSER`, common
system locations, and `PLAYWRIGHT_BROWSERS_PATH`, in that order.

## Usage

Record the checkout path in a visible browser. Finish with `Alt+Shift+S`:

```sh
zoomcheck record https://shop.example.test/cart \
  --name "Complete checkout" \
  --out .zoomcheck/checkout.json
```

Replay it at the default zoom levels and open the local report:

```sh
zoomcheck check .zoomcheck/checkout.json --out zoomcheck-report
open zoomcheck-report/index.html
```

CI can consume one JSON object from stdout. Exit code `0` means no blocking
findings, `1` means the workflow completed with one or more failures, and `2`
means the input or browser run was invalid.

```sh
zoomcheck check .zoomcheck/checkout.json \
  --json --quiet --out artifacts/zoomcheck
```

Useful options:

```text
zoomcheck record <URL> --name <NAME> [--out <FILE>] [--browser <PATH>]
zoomcheck check [WORKFLOW] [--zoom <PERCENT>...] [--out <DIR>]
                [--browser <PATH>] [--headful] [--json] [--quiet]
zoomcheck init [--out <FILE>]
```

`zoomcheck init` writes a small, editable workflow when recording a remote site
is inconvenient. A workflow contains a URL and keyboard steps (`Tab`,
`Shift+Tab`, `Enter`, `Space`, arrow keys, `Escape`). Optional `expect` selectors
lock important focus stops without turning the format into a test framework.

## What gets checked

After every keyboard step, the runner records the focused element and checks:

- expected focus target and a useful accessible name;
- visible focus treatment;
- intersection with the zoomed viewport and clipping ancestors;
- whether another element obscures the control;
- available document and container scrolling.

The report puts 200% and 400% evidence side by side and links findings to the
step that produced them. Password values are never recorded. Treat screenshots
as potentially sensitive and keep report directories out of public artifacts.

## Develop and verify

Requirements: Rust 1.85+, Node 20+, and Chromium. Playwright's Chromium is
supported when `PLAYWRIGHT_BROWSERS_PATH` is set.

```sh
npm install
npm test
npm run build
```

`npm test` runs Rust unit/integration tests and the site checks. `npm run build`
creates the release binary in `target/release/zoomcheck` and the deployable docs
site in `dist/site/`. To work on the site alone, use `npm run dev` or
`npm run build:site`.

To exercise the browser runner against the seeded failure fixtures:

```sh
npm run test:e2e
```

## Deploy and publish

The static host should publish `dist/site` at
<https://zoom-workflow-check.sociobot.in>. The factory owns registry and release
credentials; this repository does not publish automatically. Validate the Rust
package with `cargo package -p zoomcheck`.

## Project policy

There is no telemetry, account, cloud report storage, or third-party runtime
script. See [CHANGELOG.md](CHANGELOG.md), [SECURITY.md](SECURITY.md), and the
[MIT license](LICENSE).
