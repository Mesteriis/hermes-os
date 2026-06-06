import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { chromium } from 'playwright';

const views = [
	['home', 'Home'],
	['communications', 'Communications'],
	['timeline', 'Timeline'],
	['contacts', 'Contacts'],
	['projects', 'Projects'],
	['tasks', 'Tasks'],
	['calendar', 'Calendar'],
	['documents', 'Documents'],
	['notes', 'Notes'],
	['knowledge-graph', 'Knowledge Graph'],
	['telegram', 'Telegram'],
	['whatsapp', 'WhatsApp'],
	['ai-agents', 'AI Agents'],
	['settings', 'Settings']
];

const mode = process.argv[2] ?? 'baseline';
if (!['baseline', 'after'].includes(mode)) {
	console.error('Usage: node scripts/capture-layout-screenshots.mjs baseline|after [url]');
	process.exit(1);
}

const url = process.argv[3] ?? 'http://localhost:5174/';
const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
const outputDir = path.join('/tmp', `hermes-layout-${mode}-${timestamp}`);

await mkdir(outputDir, { recursive: true });

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 800, height: 600 } });
const results = [];

page.on('console', (message) => {
	if (['warning', 'error'].includes(message.type())) {
		results.push({ type: 'console', level: message.type(), text: message.text() });
	}
});

await page.goto(url, { waitUntil: 'networkidle' });

for (const [id, label] of views) {
	const button = page.getByRole('button', { name: label, exact: true });
	await button.click();
	await page.waitForTimeout(100);
	const state = await page.evaluate(() => {
		const outliers = [];
		for (const element of document.querySelectorAll('body *')) {
			const rect = element.getBoundingClientRect();
			const style = getComputedStyle(element);
			if (style.display === 'none' || style.visibility === 'hidden' || rect.width === 0 || rect.height === 0) {
				continue;
			}
			if (rect.left < -1 || rect.right > window.innerWidth + 1) {
				outliers.push({
					tag: element.tagName.toLowerCase(),
					className: typeof element.className === 'string' ? element.className : '',
					left: Math.round(rect.left),
					right: Math.round(rect.right),
					text: (element.textContent ?? '').trim().replace(/\s+/g, ' ').slice(0, 80)
				});
			}
			if (outliers.length >= 10) break;
		}
		return {
			h1: document.querySelector('h1')?.textContent?.trim() ?? null,
			bodyScrollWidth: document.body.scrollWidth,
			documentScrollWidth: document.documentElement.scrollWidth,
			guardDisplay: getComputedStyle(document.querySelector('.viewport-guard')).display,
			outliers
		};
	});
	const screenshotPath = path.join(outputDir, `${id}.png`);
	await page.screenshot({ path: screenshotPath, fullPage: false });
	results.push({ type: 'view', id, label, screenshotPath, state });
}

await page.setViewportSize({ width: 799, height: 600 });
await page.waitForTimeout(50);
const widthGuard = await page.evaluate(() => getComputedStyle(document.querySelector('.viewport-guard')).display);

await page.setViewportSize({ width: 800, height: 599 });
await page.waitForTimeout(50);
const heightGuard = await page.evaluate(() => getComputedStyle(document.querySelector('.viewport-guard')).display);

results.push({ type: 'guard', widthGuard, heightGuard });

await writeFile(path.join(outputDir, 'summary.json'), JSON.stringify(results, null, 2));
await browser.close();

console.log(outputDir);
