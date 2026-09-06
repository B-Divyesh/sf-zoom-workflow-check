import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { access, mkdtemp, readFile, rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const repo = resolve(import.meta.dirname, '../..');
const binary = join(repo, 'target', 'debug', process.platform === 'win32' ? 'zoomcheck.exe' : 'zoomcheck');

async function temporaryDirectory(prefix) {
  return mkdtemp(join(tmpdir(), prefix));
}

function runCli(args) {
  return spawnSync(binary, args, {
    cwd: repo,
    encoding: 'utf8',
    timeout: 60_000,
    env: process.env
  });
}

for (const [path, title] of [
  ['/', /Zoom Workflow Check — check keyboard workflows/],
  ['/demo/', /Demo — Zoom Workflow Check/],
  ['/privacy/', /Privacy — Zoom Workflow Check/],
  ['/terms/', /Terms — Zoom Workflow Check/],
  ['/404.html', /Page not found — Zoom Workflow Check/]
]) {
  test(`${path} has its landmark, route title, and no serious accessibility violations`, async ({ page }) => {
    const errors = [];
    page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()); });
    await page.goto(path);
    await expect(page).toHaveTitle(title);
    await expect(page.locator('main')).toBeVisible();
    await expect(page.locator('h1')).toHaveCount(1);
    const results = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
    expect(results.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact))).toEqual([]);
    expect(errors).toEqual([]);
  });
}

test('landing page states the job, audience, and first action before scrolling', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Check keyboard workflows at high zoom');
  await expect(page.getByText('For small web teams and accessibility consultants, find blocked keyboard controls before users report them.')).toBeVisible();
  const action = page.getByRole('link', { name: 'Try it with sample data' });
  await expect(action).toBeVisible();
  expect(await action.boundingBox()).toMatchObject({ y: expect.any(Number) });
  await action.click();
  await expect(page).toHaveURL(/\/demo\/$/);
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Check a sample keyboard path');
});

test('demo reset keeps real browser storage unchanged and retains the populated report', async ({ page }) => {
  await page.goto('/');
  await page.evaluate(() => localStorage.setItem('zoomcheck:real-workflow', 'keep-me'));
  await page.getByRole('link', { name: 'Try it with sample data' }).click();
  await expect(page.getByText('4 blocking findings')).toBeVisible();
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(page.getByText('4 blocking findings')).toBeVisible();
  await expect.poll(() => page.evaluate(() => localStorage.getItem('zoomcheck:real-workflow'))).toBe('keep-me');
});

test('phone layouts keep visible interactive targets inside the viewport', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  for (const path of ['/', '/demo/']) {
    await page.goto(path);
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
    expect(overflow).toBeLessThanOrEqual(1);
    const undersized = await page.locator('a:visible, button:visible').evaluateAll((elements) => elements
      .filter((element) => {
        const rect = element.getBoundingClientRect();
        return rect.width < 44 || rect.height < 44;
      })
      .map((element) => element.textContent?.trim()));
    expect(undersized).toEqual([]);
  }
});

test('@claim:offline-reload Documentation reloads offline after the first visit', async ({ browser }) => {
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.goto('/demo/');
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.reload();
  await context.setOffline(true);
  await page.reload();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Check a sample keyboard path');
  await context.close();
});

test('@claim:site-local-requests The sample page uses no third-party network request', async ({ page }) => {
  const requests = [];
  page.on('request', (request) => requests.push(request.url()));
  await page.goto('/');
  await page.getByRole('link', { name: 'Try it with sample data' }).click();
  await page.getByRole('button', { name: 'Reset demo' }).click();
  expect(requests).not.toEqual([]);
  expect(requests.every((url) => new URL(url).origin === 'http://127.0.0.1:4173')).toBe(true);
});

test('@claim:bundled-cli-demo The CLI sample writes a report, screenshots, and sample data in its own folder', async () => {
  const output = await temporaryDirectory('zoomcheck-claim-demo-');
  try {
    const run = runCli(['demo', '--out', output, '--quiet']);
    expect(run.status).toBe(1);
    const report = JSON.parse(await readFile(join(output, 'report.json'), 'utf8'));
    expect(report.workflow).toBe('Checkout flyout sample');
    expect(report.failures).toBeGreaterThan(0);
    await access(join(output, 'index.html'));
    await access(join(output, 'zoom-200.png'));
    await access(join(output, 'zoom-400.png'));
    await access(join(output, 'sample', 'checkout-flyout.html'));
    await access(join(output, 'sample', 'workflow.json'));
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});

test('@claim:no-account-required The bundled CLI sample runs without account setup', async () => {
  const output = await temporaryDirectory('zoomcheck-claim-no-account-');
  try {
    const run = runCli(['demo', '--out', output, '--quiet']);
    expect(run.status).toBe(1);
    await access(join(output, 'report.json'));
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});

test('@claim:native-desktop-zoom The bundled sample reports native 200% and 400% desktop zoom viewports', async () => {
  const output = await temporaryDirectory('zoomcheck-claim-zoom-');
  try {
    const run = runCli(['demo', '--out', output, '--quiet']);
    expect(run.status).toBe(1);
    const report = JSON.parse(await readFile(join(output, 'report.json'), 'utf8'));
    expect(report.runs.map((item) => item.zoom)).toEqual([200, 400]);
    const dimensions = report.runs.map((item) => item.viewportCss.match(/([\d.]+) × ([\d.]+) CSS px · ([\d.]+) dppx/).slice(1).map(Number));
    expect(dimensions[0][0]).toBeGreaterThan(620);
    expect(dimensions[0][0]).toBeLessThan(660);
    expect(dimensions[0][2]).toBeCloseTo(2, 1);
    expect(dimensions[1][0]).toBeGreaterThan(300);
    expect(dimensions[1][0]).toBeLessThan(340);
    expect(dimensions[1][2]).toBeCloseTo(4, 1);
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});

test('@claim:focus-blocking-findings The sample identifies viewport clipping at high zoom', async () => {
  const output = await temporaryDirectory('zoomcheck-claim-findings-');
  try {
    const run = runCli(['demo', '--out', output, '--quiet']);
    expect(run.status).toBe(1);
    const report = JSON.parse(await readFile(join(output, 'report.json'), 'utf8'));
    const highZoom = report.runs.find((item) => item.zoom === 400);
    expect(highZoom.failures).toBeGreaterThanOrEqual(3);
    expect(highZoom.steps.some((step) => step.findings.some((finding) => finding.kind === 'viewport_clipping' && finding.severity === 'failure'))).toBe(true);
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});
