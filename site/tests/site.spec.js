import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

for (const [path, title] of [['/', /Zoom Workflow Check/], ['/privacy/', /Privacy/], ['/terms/', /Terms/]]) {
  test(`${path} has its landmark and no serious accessibility violations`, async ({ page }) => {
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

test('report demo works with pointer and arrow keys', async ({ page }) => {
  await page.goto('/');
  const tab400 = page.getByRole('tab', { name: '400%' });
  await tab400.click();
  await expect(page.locator('#demo-result')).toHaveText('Blocked at 400%');
  await tab400.press('ArrowLeft');
  await expect(page.locator('#demo-result')).toHaveText('Passes at 200%');
  await expect(page.getByRole('tab', { name: '200%' })).toBeFocused();
});

test('mobile page keeps primary content inside the viewport', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  expect(overflow).toBeLessThanOrEqual(1);
  await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Install the CLI' })).toBeVisible();
});

test('installed field guide reloads offline', async ({ page, context }) => {
  await page.goto('/');
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.reload();
  await context.setOffline(true);
  await page.reload();
  await expect(page.getByRole('heading', { level: 1 })).toContainText('YOUR WORKFLOW');
});
