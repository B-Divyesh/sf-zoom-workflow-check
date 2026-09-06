#!/usr/bin/env bash
set -euo pipefail

url=${1:?usage: scripts/verify-url.sh <url>}

node --input-type=module - "$url" <<'NODE'
import { chromium } from '@playwright/test';

const url = process.argv[2];
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
const consoleErrors = [];
const pageErrors = [];
page.on('console', (message) => { if (message.type() === 'error') consoleErrors.push(message.text()); });
page.on('pageerror', (error) => pageErrors.push(error.message));
await page.goto(url, { waitUntil: 'networkidle' });
const result = await page.evaluate(() => ({
  title: document.title,
  lang: document.documentElement.lang,
  mainCount: document.querySelectorAll('main').length,
  h1Count: document.querySelectorAll('h1').length,
  missingAlt: [...document.images].filter((image) => !image.hasAttribute('alt')).map((image) => image.currentSrc)
}));
await browser.close();
if (!result.title || !result.lang || result.mainCount !== 1 || result.h1Count !== 1 || result.missingAlt.length || consoleErrors.length || pageErrors.length) {
  console.error(JSON.stringify({ ...result, consoleErrors, pageErrors }, null, 2));
  process.exit(1);
}
console.log(JSON.stringify({ url, ...result, consoleErrors: 0, pageErrors: 0 }));
NODE
