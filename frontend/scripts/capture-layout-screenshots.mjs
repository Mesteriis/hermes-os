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

const viewports = [
	{ id: '800x600', width: 800, height: 600, expectMultiColumn: false },
	{ id: '901x768', width: 901, height: 768, expectMultiColumn: false },
	{ id: '1024x768', width: 1024, height: 768, expectMultiColumn: false },
	{ id: '1200x768', width: 1200, height: 768, expectMultiColumn: false },
	{ id: '1366x768', width: 1366, height: 768, expectMultiColumn: true }
];

const desktopColumnSpreadTolerance = 2;

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
const results = [];
const failures = [];

async function getPrimaryNavButton(page, label) {
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

async function getViewportGuardDisplay(page, context) {
	return page.evaluate((currentContext) => {
		const guard = document.querySelector('.viewport-guard');
		if (guard === null) {
			throw new Error(`Expected .viewport-guard element to exist while ${currentContext}.`);
		}
		return getComputedStyle(guard).display;
	}, context);
}

function trackConsoleIssues(page, consoleIssues) {
	page.on('console', (message) => {
		if (['warning', 'error'].includes(message.type())) {
			consoleIssues.push({ level: message.type(), text: message.text() });
		}
	});
}

async function captureViewport(viewport) {
	const viewportDir = path.join(outputDir, viewport.id);
	await mkdir(viewportDir, { recursive: true });

	const page = await browser.newPage({ viewport: { width: viewport.width, height: viewport.height } });
	const consoleIssues = [];
	trackConsoleIssues(page, consoleIssues);

	try {
		await page.goto(url, { waitUntil: 'networkidle' });

		for (const [id, label] of views) {
			const button = await getPrimaryNavButton(page, label);
			await button.click();
			await page.waitForTimeout(100);
			const layoutState = await page.evaluate(() => {
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
				const isElementVisible = (element) => {
					const rect = element.getBoundingClientRect();
					const style = getComputedStyle(element);
					return {
						rect,
						visible:
							style.display !== 'none' &&
							style.visibility !== 'hidden' &&
							rect.width > 0 &&
							rect.height > 0
					};
				};
				const widgetUnit = tokenNumber('--hh-widget-unit', 24);
				const widgetRow = tokenNumber('--hh-widget-row', widgetUnit);
				const isNearTokenMultiple = (height, token) => {
					if (token <= 0) return false;
					const nearest = Math.round(height / token) * token;
					return Math.abs(height - nearest) <= 2;
				};
				const outliers = [];
				for (const element of document.querySelectorAll('body *')) {
					const { rect, visible } = isElementVisible(element);
					if (!visible) continue;
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

				const visibleWidgetFrames = [];
				for (const element of document.querySelectorAll('.widget-frame[data-widget-id]')) {
					const { rect, visible } = isElementVisible(element);
					if (!visible) continue;
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

				const layoutColumnMetrics = [];
				const layoutSelectors = [
					'.dashboard-grid',
					'.three-pane',
					'.contacts-layout',
					'.docs-layout',
					'.notes-layout',
					'.project-dashboard-grid',
					'.tasks-layout',
					'.calendar-layout',
					'.knowledge-layout',
					'.agents-layout',
					'.settings-layout',
					'.timeline-layout'
				];
				for (const container of document.querySelectorAll(layoutSelectors.join(','))) {
					const { rect: containerRect, visible: containerVisible } = isElementVisible(container);
					if (!containerVisible) continue;

					const columnsByLeft = new Map();
					for (const child of Array.from(container.children)) {
						const { rect, visible } = isElementVisible(child);
						if (!visible) continue;
						const left = Math.round((rect.left - containerRect.left) / 8) * 8;
						const column = columnsByLeft.get(left) ?? {
							left,
							count: 0,
							top: Number.POSITIVE_INFINITY,
							bottom: Number.NEGATIVE_INFINITY,
							contentTop: Number.POSITIVE_INFINITY,
							contentBottom: Number.NEGATIVE_INFINITY
						};
						column.count += 1;
						column.top = Math.min(column.top, Math.round(rect.top - containerRect.top));
						column.bottom = Math.max(column.bottom, Math.round(rect.bottom - containerRect.top));
						column.contentTop = Math.min(column.contentTop, Math.round(rect.top - containerRect.top));
						column.contentBottom = Math.max(column.contentBottom, Math.round(rect.bottom - containerRect.top));
						columnsByLeft.set(left, column);
					}

					if (columnsByLeft.size < 2) continue;

					const columns = Array.from(columnsByLeft.values())
						.map((column) => ({
							...column,
							height: column.bottom - column.top,
							contentHeight: column.contentBottom - column.contentTop
						}))
						.sort((left, right) => left.left - right.left);
					const columnHeights = columns.map((column) => column.height);
					layoutColumnMetrics.push({
						selector: layoutSelectors.find((selector) => container.matches(selector)) ?? '',
						columnCount: columns.length,
						columns,
						columnHeightSpread: Math.max(...columnHeights) - Math.min(...columnHeights)
					});
				}

				return {
					h1: document.querySelector('h1')?.textContent?.trim() ?? null,
					bodyScrollWidth: document.body.scrollWidth,
					documentScrollWidth: document.documentElement.scrollWidth,
					outliers,
					widgetFrameMetrics: {
						widgetUnit,
						widgetRow,
						visibleCount: visibleWidgetFrames.length,
						nonModularHeights: visibleWidgetFrames.filter((frame) => !frame.moduleAligned && !frame.rowAligned)
					},
					layoutColumnMetrics
				};
			});
			const state = {
				h1: layoutState.h1,
				bodyScrollWidth: layoutState.bodyScrollWidth,
				documentScrollWidth: layoutState.documentScrollWidth,
				guardDisplay: await getViewportGuardDisplay(page, `capturing ${label} view state at ${viewport.id}`),
				outliers: layoutState.outliers,
				widgetFrameMetrics: layoutState.widgetFrameMetrics,
				layoutColumnMetrics: layoutState.layoutColumnMetrics
			};
			const screenshotPath = path.join(viewportDir, `${id}.png`);
			await page.screenshot({ path: screenshotPath, fullPage: false });
			results.push({ type: 'view', viewport, id, label, screenshotPath, state });

			if (
				state.bodyScrollWidth > viewport.width + 1 ||
				state.documentScrollWidth > viewport.width + 1 ||
				state.outliers.length > 0
			) {
				failures.push({
					type: 'horizontal-overflow',
					viewport: viewport.id,
					view: id,
					bodyScrollWidth: state.bodyScrollWidth,
					documentScrollWidth: state.documentScrollWidth,
					outliers: state.outliers
				});
			}
			if (state.widgetFrameMetrics.nonModularHeights.length > 0) {
				failures.push({
					type: 'non-modular-widget-heights',
					viewport: viewport.id,
					view: id,
					widgets: state.widgetFrameMetrics.nonModularHeights
				});
			}
			if (viewport.expectMultiColumn) {
				const spreadOutliers = state.layoutColumnMetrics.filter(
					(metric) => metric.columnHeightSpread > desktopColumnSpreadTolerance
				);
				if (spreadOutliers.length > 0) {
					failures.push({
						type: 'desktop-column-spread',
						viewport: viewport.id,
						view: id,
						tolerance: desktopColumnSpreadTolerance,
						layouts: spreadOutliers
					});
				}
			}
		}

		results.push({ type: 'console', viewport, issues: consoleIssues });
	} finally {
		await page.close();
	}
}

async function captureViewportGuard() {
	const page = await browser.newPage({ viewport: { width: 800, height: 600 } });
	try {
		await page.goto(url, { waitUntil: 'networkidle' });

		await page.setViewportSize({ width: 799, height: 600 });
		await page.waitForTimeout(50);
		const widthGuard = await getViewportGuardDisplay(page, 'checking the 799px width guard');

		await page.setViewportSize({ width: 800, height: 599 });
		await page.waitForTimeout(50);
		const heightGuard = await getViewportGuardDisplay(page, 'checking the 599px height guard');

		results.push({ type: 'guard', widthGuard, heightGuard });
		if (widthGuard !== 'grid' || heightGuard !== 'grid') {
			failures.push({ type: 'viewport-guard', widthGuard, heightGuard });
		}
	} finally {
		await page.close();
	}
}

try {
	for (const viewport of viewports) {
		await captureViewport(viewport);
	}
	await captureViewportGuard();

	await writeFile(path.join(outputDir, 'summary.json'), JSON.stringify(results, null, 2));

	const widgetSummary = results
		.filter((result) => result.type === 'view')
		.map((result) => ({
			viewport: result.viewport.id,
			id: result.id,
			visibleWidgets: result.state.widgetFrameMetrics.visibleCount,
			widgetRow: result.state.widgetFrameMetrics.widgetRow,
			nonModularWidgets: result.state.widgetFrameMetrics.nonModularHeights.length,
			horizontalOutliers: result.state.outliers.length,
			columnSpreads: result.state.layoutColumnMetrics.map((metric) => ({
				selector: metric.selector,
				spread: metric.columnHeightSpread
			}))
		}));
	console.log(`Widget frame summary: ${JSON.stringify(widgetSummary)}`);
	if (failures.length > 0) {
		await writeFile(path.join(outputDir, 'failures.json'), JSON.stringify(failures, null, 2));
		console.error(`Layout capture failed: ${JSON.stringify(failures.slice(0, 5))}`);
		console.log(outputDir);
		process.exit(1);
	}
} finally {
	await browser.close();
}

console.log(outputDir);
