import { mkdir, writeFile } from 'node:fs/promises';
import os from 'node:os';
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
const outputDir = path.join(os.tmpdir(), `hermes-layout-${mode}-${timestamp}`);

await mkdir(outputDir, { recursive: true });

const browser = await chromium.launch();

try {
	const page = await browser.newPage({ viewport: { width: 800, height: 600 } });
	const results = [];

	async function getPrimaryNavButton(label) {
		const primaryNav = page.locator('nav[aria-label="Primary"]');
		const button = primaryNav.getByRole('button').filter({ hasText: label });
		const count = await button.count();
		if (count !== 1) {
			const navLabels = await primaryNav.getByRole('button').evaluateAll((buttons) =>
				buttons.map((navButton) => (navButton.textContent ?? '').trim().replace(/\s+/g, ' '))
			);
			throw new Error(
				`Expected exactly one primary nav button containing visible text "${label}", found ${count}. ` +
					`Primary nav buttons: ${navLabels.length > 0 ? navLabels.join(', ') : '(none)'}`
			);
		}
		return button;
	}

	async function getViewportGuardDisplay(context) {
		return page.evaluate((currentContext) => {
			const guard = document.querySelector('.viewport-guard');
			if (guard === null) {
				throw new Error(`Expected .viewport-guard element to exist while ${currentContext}.`);
			}
			return getComputedStyle(guard).display;
		}, context);
	}

	page.on('console', (message) => {
		if (['warning', 'error'].includes(message.type())) {
			results.push({ type: 'console', level: message.type(), text: message.text() });
		}
	});

	await page.goto(url, { waitUntil: 'networkidle' });

	for (const [id, label] of views) {
		const button = await getPrimaryNavButton(label);
		await button.click();
		await page.waitForTimeout(100);
		const layoutState = await page.evaluate(() => {
			const outliers = [];
			const tokenNumber = (name, fallback) => {
				const probe = document.createElement('div');
				probe.style.position = 'absolute';
				probe.style.visibility = 'hidden';
				probe.style.pointerEvents = 'none';
				probe.style.height = `var(${name})`;
				document.body.append(probe);
				const value = Number.parseFloat(getComputedStyle(probe).height);
				probe.remove();
				return Number.isFinite(value) ? value : fallback;
			};
			const widgetUnit = tokenNumber('--hh-widget-unit', 24);
			const widgetRow = tokenNumber('--hh-widget-row', widgetUnit);
			const isNearTokenMultiple = (height, token) => {
				if (token <= 0) return false;
				const nearest = Math.round(height / token) * token;
				return Math.abs(height - nearest) <= 2;
			};
			const visibleWidgetFrames = [];
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
			for (const element of document.querySelectorAll('.widget-frame[data-widget-id]')) {
				const rect = element.getBoundingClientRect();
				const style = getComputedStyle(element);
				if (style.display === 'none' || style.visibility === 'hidden' || rect.width === 0 || rect.height === 0) {
					continue;
				}
				const height = Math.round(rect.height);
				visibleWidgetFrames.push({
					widgetId: element.getAttribute('data-widget-id') ?? '',
					height,
					left: Math.round(rect.left),
					top: Math.round(rect.top),
					moduleAligned: isNearTokenMultiple(height, widgetUnit),
					rowAligned: isNearTokenMultiple(height, widgetRow)
				});
			}
			const columnsByLeft = new Map();
			for (const frame of visibleWidgetFrames) {
				const columnKey = String(Math.round(frame.left / 12) * 12);
				const column = columnsByLeft.get(columnKey) ?? { left: Number(columnKey), count: 0, totalHeight: 0 };
				column.count += 1;
				column.totalHeight += frame.height;
				columnsByLeft.set(columnKey, column);
			}
			const columns = Array.from(columnsByLeft.values()).sort((left, right) => left.left - right.left);
			const columnHeights = columns.map((column) => column.totalHeight);
			const columnHeightSpread =
				columnHeights.length > 1 ? Math.max(...columnHeights) - Math.min(...columnHeights) : 0;
			return {
				h1: document.querySelector('h1')?.textContent?.trim() ?? null,
				bodyScrollWidth: document.body.scrollWidth,
				documentScrollWidth: document.documentElement.scrollWidth,
				outliers,
				widgetFrameMetrics: {
					widgetUnit,
					widgetRow,
					visibleCount: visibleWidgetFrames.length,
					nonModularHeights: visibleWidgetFrames
						.filter((frame) => !frame.moduleAligned && !frame.rowAligned)
						.slice(0, 10),
					columns,
					columnHeightSpread
				}
			};
		});
		const state = {
			h1: layoutState.h1,
			bodyScrollWidth: layoutState.bodyScrollWidth,
			documentScrollWidth: layoutState.documentScrollWidth,
			guardDisplay: await getViewportGuardDisplay(`capturing ${label} view state`),
			outliers: layoutState.outliers,
			widgetFrameMetrics: layoutState.widgetFrameMetrics
		};
		const screenshotPath = path.join(outputDir, `${id}.png`);
		await page.screenshot({ path: screenshotPath, fullPage: false });
		results.push({ type: 'view', id, label, screenshotPath, state });
	}

	await page.setViewportSize({ width: 799, height: 600 });
	await page.waitForTimeout(50);
	const widthGuard = await getViewportGuardDisplay('checking the 799px width guard');

	await page.setViewportSize({ width: 800, height: 599 });
	await page.waitForTimeout(50);
	const heightGuard = await getViewportGuardDisplay('checking the 599px height guard');

	results.push({ type: 'guard', widthGuard, heightGuard });

	await writeFile(path.join(outputDir, 'summary.json'), JSON.stringify(results, null, 2));

	const widgetSummary = results
		.filter((result) => result.type === 'view')
		.map((result) => ({
			id: result.id,
			visibleWidgets: result.state.widgetFrameMetrics.visibleCount,
			nonModularWidgets: result.state.widgetFrameMetrics.nonModularHeights.length,
			columnHeightSpread: result.state.widgetFrameMetrics.columnHeightSpread
		}));
	console.log(`Widget frame summary: ${JSON.stringify(widgetSummary)}`);
} finally {
	await browser.close();
}

console.log(outputDir);
