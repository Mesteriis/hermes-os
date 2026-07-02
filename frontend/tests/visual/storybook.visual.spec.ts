import { expect, test } from '@playwright/test'
import type { APIRequestContext, Page } from '@playwright/test'

const THEMES = ['light', 'dark', 'hermes'] as const
const LOCALES = ['ru', 'en', 'es'] as const

const VIEWPORTS = [
	{ name: 'w320', width: 320, height: 900 },
	{ name: 'w375', width: 375, height: 900 },
	{ name: 'w768', width: 768, height: 900 },
	{ name: 'w1024', width: 1024, height: 900 },
	{ name: 'w1440', width: 1440, height: 900 },
	{ name: 'w1920', width: 1920, height: 1080 },
	{ name: 'w5120', width: 5120, height: 1440 }
] as const

const GENERAL_STORY_TITLE_PREFIX = 'Hermes UI/General/'

type UiThemeName = (typeof THEMES)[number]
type StorybookLocale = (typeof LOCALES)[number]

interface StorybookIndex {
	entries: Record<string, StorybookEntry>
}

interface StorybookEntry {
	id: string
	name: string
	title: string
	type: string
}

interface StorybookStory extends StorybookEntry {
	type: 'story'
}

interface StoryBucket {
	name: string
	includes: (story: StorybookStory) => boolean
}

const STORY_BUCKETS: readonly StoryBucket[] = [
	{
		name: 'domain-and-foundation',
		includes: (story) => story.title.startsWith('Hermes UI/Domain/') || story.title.startsWith('Hermes UI/Foundation/')
	},
	{
		name: 'general-a-d',
		includes: createGeneralBucketMatcher([
			'Async Select',
			'Button',
			'Button Group',
			'Cascader',
			'Checkbox',
			'Command',
			'Context Menu',
			'Data Display',
			'Date Picker',
			'Date Range Picker',
			'Dialog',
			'Drawer'
		])
	},
	{
		name: 'general-e-l',
		includes: createGeneralBucketMatcher([
			'Editor',
			'Feedback',
			'Form',
			'Grouped Select',
			'Icon Button',
			'Input',
			'Layout',
			'List'
		])
	},
	{
		name: 'general-m-o',
		includes: createGeneralBucketMatcher(['Media', 'Menu', 'Multi Select', 'Navigation', 'Overlays'])
	},
	{
		name: 'general-p-s',
		includes: createGeneralBucketMatcher([
			'Popover',
			'Radio',
			'Search Input',
			'Searchable Multi Select',
				'Searchable Select',
				'Select',
				'Slider',
				'Split Button',
				'Surface',
				'Switch'
			])
		},
	{
		name: 'general-t-z',
		includes: createGeneralBucketMatcher([
			'Table',
			'Tabs',
			'Tag Input',
			'Textarea',
			'Time Picker',
			'Timeline',
			'Toggle Group',
			'Token Input',
			'Tooltip',
			'Tree',
			'Tree Select',
			'Utility'
		])
	}
]

test.describe('Hermes UI Storybook visual regression', () => {
	test.describe.configure({ mode: 'serial' })

	test('story buckets cover all Storybook stories', async ({ request }) => {
		const stories = await loadStories(request)
		expect(stories, 'Storybook must expose at least one story for visual regression').not.toHaveLength(0)
		const uncoveredStories = stories.filter((story) => STORY_BUCKETS.every((bucket) => !bucket.includes(story)))
		const duplicatedStories = stories.filter(
			(story) => STORY_BUCKETS.filter((bucket) => bucket.includes(story)).length !== 1
		)

		expect(uncoveredStories.map(formatStoryName), 'Every Storybook story must be assigned to a visual bucket').toEqual([])
		expect(duplicatedStories.map(formatStoryName), 'Every Storybook story must match exactly one visual bucket').toEqual(
			[]
		)
	})

	for (const bucket of STORY_BUCKETS) {
		test(`${bucket.name} stories match visual baselines`, async ({ page, request }) => {
			const browserErrors: string[] = []
			page.on('console', (message) => {
				if (message.type() === 'error') {
					browserErrors.push(message.text())
				}
			})

			const stories = (await loadStories(request)).filter(bucket.includes)
			expect(stories, `Visual bucket "${bucket.name}" must include at least one story`).not.toHaveLength(0)

			for (const story of stories) {
				for (const locale of LOCALES) {
					for (const theme of THEMES) {
						for (const viewport of VIEWPORTS) {
							await test.step(
								`${story.title} / ${story.name} / ${locale} / ${theme} / ${viewport.name}`,
								async () => {
									await page.setViewportSize({
										width: viewport.width,
										height: viewport.height
									})
									await openStory(page, story, theme, locale)
									await expect(page).toHaveScreenshot(snapshotName(story, locale, theme, viewport.name))
								}
							)
						}
					}
				}
			}

			expect(browserErrors, 'Storybook stories must not emit browser console errors').toEqual([])
		})
	}
})

function createGeneralBucketMatcher(componentNames: readonly string[]): (story: StorybookStory) => boolean {
	const allowedComponentNames = new Set(componentNames)
	return (story) => {
		const componentName = getGeneralComponentName(story)
		return componentName !== null && allowedComponentNames.has(componentName)
	}
}

function getGeneralComponentName(story: StorybookStory): string | null {
	if (!story.title.startsWith(GENERAL_STORY_TITLE_PREFIX)) {
		return null
	}

	return story.title.slice(GENERAL_STORY_TITLE_PREFIX.length).split('/')[0] ?? null
}

function formatStoryName(story: StorybookStory): string {
	return `${story.title} / ${story.name}`
}

async function loadStories(request: APIRequestContext): Promise<StorybookStory[]> {
	const response = await request.get('/index.json')
	expect(response.ok(), 'Storybook index.json must be reachable').toBe(true)
	const index = (await response.json()) as StorybookIndex
	return Object.values(index.entries)
		.filter(isStory)
		.sort((left, right) => left.id.localeCompare(right.id))
}

function isStory(entry: StorybookEntry): entry is StorybookStory {
	return entry.type === 'story'
}

async function openStory(page: Page, story: StorybookStory, theme: UiThemeName, locale: StorybookLocale): Promise<void> {
	const storyUrl = new URL('/iframe.html', 'http://127.0.0.1')
	storyUrl.searchParams.set('id', story.id)
	storyUrl.searchParams.set('viewMode', 'story')
	storyUrl.searchParams.set('globals', `theme:${theme};locale:${locale}`)

	await page.goto(`${storyUrl.pathname}${storyUrl.search}`, { waitUntil: 'load' })
	await expect(page.locator('#storybook-root')).toBeAttached()
	const shell = page.locator('.storybook-shell')
	await expect(shell).toBeVisible()
	await expect(shell).toHaveAttribute('data-ui-theme', theme)
	await expect(shell).toHaveAttribute('data-ui-locale', locale)
	await expect(shell).toHaveAttribute('lang', locale)
	await waitForStableStoryFrame(page)
	await assertNoStorybookError(page)
}

async function waitForStableStoryFrame(page: Page): Promise<void> {
	await page.evaluate(async () => {
		await document.fonts.ready
		await new Promise<void>((resolve) => {
			requestAnimationFrame(() => {
				requestAnimationFrame(() => resolve())
			})
		})
		window.scrollTo(0, 0)
	})
}

async function assertNoStorybookError(page: Page): Promise<void> {
	const bodyText = (await page.locator('body').textContent()) ?? ''
	expect(bodyText).not.toContain('Internal server error')
	expect(bodyText).not.toContain('Failed to fetch dynamically imported module')
	expect(bodyText).not.toContain('Cannot find module')
}

function snapshotName(story: StorybookStory, locale: StorybookLocale, theme: UiThemeName, viewportName: string): string {
	return `${safeSnapshotPart(story.id)}--${locale}--${theme}--${viewportName}.png`
}

function safeSnapshotPart(value: string): string {
	return value.replace(/[^a-z0-9-]+/gi, '-').toLowerCase()
}
