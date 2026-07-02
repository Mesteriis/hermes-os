import { describe, expect, it } from 'vitest'
import { readdirSync, readFileSync } from 'node:fs'
import type { Dirent } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'

describe('Hermes UI Storybook visual coverage boundary', () => {
	it('exports every shared UI component through the kit barrel', () => {
		const uiDir = fileURLToPath(new URL('.', import.meta.url))
		const componentNames = readdirSync(uiDir)
			.filter((fileName) => fileName.endsWith('.vue'))
			.map((fileName) => fileName.replace(/\.vue$/, ''))
			.sort()
		const exportedNames = exportedComponentNames(readFileSync(new URL('./index.ts', import.meta.url), 'utf8'))

		expect(exportedNames).toEqual(componentNames)
	})

	it('keeps every exported shared UI component represented in Storybook', () => {
		const storiesDir = fileURLToPath(new URL('../../../stories/ui/', import.meta.url))
		const storySources = readdirSync(storiesDir)
			.filter((fileName) => fileName.endsWith('.stories.ts'))
			.map((fileName) => readFileSync(join(storiesDir, fileName), 'utf8'))
			.join('\n')
		const exportedNames = exportedComponentNames(readFileSync(new URL('./index.ts', import.meta.url), 'utf8'))
		const storyImports = storybookUiImports(storySources)

		expect(exportedNames.filter((componentName) => !storyImports.includes(componentName))).toEqual([])
	})

	it('keeps Storybook configured as the Hermes UI lab surface', () => {
		const frontendRoot = fileURLToPath(new URL('../../../', import.meta.url))
		const mainConfig = readFileSync(join(frontendRoot, '.storybook/main.ts'), 'utf8')
		const previewConfig = readFileSync(join(frontendRoot, '.storybook/preview.ts'), 'utf8')
		const requiredAddons = [
			'@storybook/addon-docs',
			'@storybook/addon-a11y',
			'@storybook/addon-themes',
			'@storybook/addon-vitest',
			'@storybook/addon-coverage',
			'@storybook/addon-designs',
			'msw-storybook-addon',
			'storybook-addon-pseudo-states',
			'storybook-design-token'
		]

		for (const addonName of requiredAddons) {
			expect(mainConfig).toContain(addonName)
		}
		expect(mainConfig).toContain("staticDirs: ['../public']")
		expect(mainConfig).toContain("const allowedStorybookHosts = ['localhost', '127.0.0.1']")
		expect(mainConfig).toContain('allowedHosts:')
		expect(mainConfig).toContain("designTokenGlob: 'src/shared/ui/styles/**/*.css'")
		expect(previewConfig).toContain('withThemeByDataAttribute')
		expect(previewConfig).toContain('initialize({')
		expect(previewConfig).toContain('loaders: [mswLoader]')
		expect(previewConfig).toContain('storybookLocaleToolbarItems')
	})

	it('keeps Storybook stories localized for Russian, English and Spanish', () => {
		const storiesDir = fileURLToPath(new URL('../../../stories/ui/', import.meta.url))
		const localeSource = readFileSync(join(storiesDir, 'storybook-i18n.ts'), 'utf8')
		const storySources = readdirSync(storiesDir)
			.filter((fileName) => fileName.endsWith('.stories.ts'))
			.map((fileName) => `${fileName}\n${readFileSync(join(storiesDir, fileName), 'utf8')}`)

		expect(localeSource).toContain("export const storybookLocales = ['ru', 'en', 'es'] as const")
		expect(localeSource).toContain("'Русский'")
		expect(localeSource).toContain("'English'")
		expect(localeSource).toContain("'Español'")
		for (const source of storySources) {
			expect(source).toContain("from './storybook-i18n'")
		}
	})

	it('keeps standard controls under the General Storybook hierarchy', () => {
		const storiesDir = fileURLToPath(new URL('../../../stories/ui/', import.meta.url))
		const storySources = readdirSync(storiesDir)
			.filter((fileName) => fileName.endsWith('.stories.ts'))
			.map((fileName) => `${fileName}\n${readFileSync(join(storiesDir, fileName), 'utf8')}`)
			.join('\n')
		const requiredGeneralTitles = [
			'Hermes UI/General/Button',
			'Hermes UI/General/Button Group',
			'Hermes UI/General/Icon Button',
			'Hermes UI/General/Split Button',
			'Hermes UI/General/Toggle Group',
			'Hermes UI/General/Select',
			'Hermes UI/General/Searchable Select',
			'Hermes UI/General/Multi Select',
			'Hermes UI/General/Searchable Multi Select',
			'Hermes UI/General/Grouped Select',
			'Hermes UI/General/Tree Select',
			'Hermes UI/General/Cascader',
			'Hermes UI/General/Async Select',
			'Hermes UI/General/Input',
			'Hermes UI/General/Textarea',
			'Hermes UI/General/Search Input',
			'Hermes UI/General/Token Input',
			'Hermes UI/General/Tag Input',
			'Hermes UI/General/Checkbox',
			'Hermes UI/General/Radio',
			'Hermes UI/General/Switch',
			'Hermes UI/General/Slider',
			'Hermes UI/General/Date Picker',
			'Hermes UI/General/Date Range Picker',
			'Hermes UI/General/Time Picker',
			'Hermes UI/General/Menu',
			'Hermes UI/General/Context Menu',
			'Hermes UI/General/Command',
			'Hermes UI/General/Tabs',
			'Hermes UI/General/Dialog',
			'Hermes UI/General/Drawer',
			'Hermes UI/General/Tooltip',
			'Hermes UI/General/Popover',
			'Hermes UI/General/Surface',
			'Hermes UI/General/Table',
			'Hermes UI/General/List',
			'Hermes UI/General/Tree',
			'Hermes UI/General/Timeline',
			'Hermes UI/General/Media',
			'Hermes UI/General/Editor',
			'Hermes UI/General/Feedback',
			'Hermes UI/General/Layout',
			'Hermes UI/General/Utility'
		]
		const requiredDomainTitles = [
			'Hermes UI/Domain/Communications'
		]
		const requiredFoundationTitles = [
			'Hermes UI/Foundation/Tokens',
			'Hermes UI/Foundation/Themes',
			'Hermes UI/Foundation/Typography',
			'Hermes UI/Foundation/Icons',
			'Hermes UI/Foundation/Spacing'
		]
		const forbiddenLegacyTopLevelTitles = [
			'Hermes UI/Command',
			'Hermes UI/Communication',
			'Hermes UI/Data Display',
			'Hermes UI/Editor',
			'Hermes UI/Feedback',
			'Hermes UI/Foundation',
			'Hermes UI/Layout',
			'Hermes UI/Media',
			'Hermes UI/Navigation',
			'Hermes UI/Overlays',
			'Hermes UI/Primitives',
			'Hermes UI/Themes',
			'Hermes UI/Utility'
		]

		expect(storySources).not.toContain('Hermes UI/Controls/')
		for (const title of [...requiredGeneralTitles, ...requiredDomainTitles, ...requiredFoundationTitles]) {
			expect(storySources).toContain(`title: '${title}'`)
		}
		for (const title of forbiddenLegacyTopLevelTitles) {
			expect(storySources).not.toContain(`title: '${title}'`)
		}
	})

	it('screenshots all Storybook stories across Hermes themes, locales and responsive widths', () => {
		const visualSpec = readFileSync(
			new URL('../../../tests/visual/storybook.visual.spec.ts', import.meta.url),
			'utf8'
		)

		expect(visualSpec).toContain("const THEMES = ['light', 'dark', 'hermes'] as const")
		expect(visualSpec).toContain("const LOCALES = ['ru', 'en', 'es'] as const")
		for (const width of [320, 375, 768, 1024, 1440, 1920, 5120]) {
			expect(visualSpec).toContain(`width: ${width}`)
		}
		expect(visualSpec).toContain("request.get('/index.json')")
		expect(visualSpec).toContain("entry.type === 'story'")
		expect(visualSpec).toContain('data-ui-locale')
		expect(visualSpec).toContain('toHaveScreenshot')
	})

	it('keeps Storybook visual regression wired into CI as a compare-only gate', () => {
		const frontendRoot = fileURLToPath(new URL('../../../', import.meta.url))
		const repoRoot = fileURLToPath(new URL('../../../../', import.meta.url))
		const makefile = readFileSync(join(repoRoot, 'Makefile'), 'utf8')
		const ciWorkflow = readFileSync(join(repoRoot, '.github/workflows/ci.yml'), 'utf8')

		expect(makefile).toContain('frontend-visual:\n\t@cd frontend && pnpm test:visual')
		expect(makefile).toContain('frontend-visual-update:\n\t@cd frontend && pnpm test:visual:update')
		expect(makefile).toContain('frontend-validate: frontend-lint frontend-test frontend-visual frontend-build')
		expect(ciWorkflow).toContain('pull_request:')
		expect(ciWorkflow).toContain('push:')
		expect(ciWorkflow).toContain('frontend-visual:')
		expect(ciWorkflow).toContain('runs-on: macos-latest')
		expect(ciWorkflow).toContain('run: make frontend-visual')
		expect(ciWorkflow).toContain('frontend-visual-regression')
		expect(ciWorkflow).not.toContain('run: make frontend-visual-update')
		const playwrightConfig = readFileSync(join(frontendRoot, 'playwright.config.ts'), 'utf8')
		expect(playwrightConfig).toContain("process.env.HERMES_STORYBOOK_HOST ?? 'localhost'")
		expect(playwrightConfig).toContain('pnpm exec storybook build --quiet --test --output-dir storybook-static')
		expect(playwrightConfig).toContain('pnpm storybook:serve')
		expect(playwrightConfig).toContain('reuseExistingServer: false')
		const packageJson = readFileSync(join(frontendRoot, 'package.json'), 'utf8')
		expect(packageJson).toContain('"storybook:serve": "node scripts/serve-storybook-static.mjs"')
		expect(packageJson).toContain('test-storybook --url http://localhost:6006')
	})

	it('keeps vendor UI primitives behind the Hermes UI kit boundary', () => {
		const frontendRoot = fileURLToPath(new URL('../../../', import.meta.url))
		const checkedRoots = ['src', 'stories', '.storybook']
		const forbiddenImports = /from ['"](reka-ui|shadcn-vue|@radix-ui\/[^'"]+|lucide(?:-[^'"]+)?)['"]/g
		const violations = checkedRoots
			.flatMap((root) => sourceFiles(join(frontendRoot, root)))
			.filter((filePath) => !filePath.includes('/src/shared/ui/'))
			.flatMap((filePath) => {
				const source = readFileSync(filePath, 'utf8')
				return Array.from(source.matchAll(forbiddenImports)).map((match) => `${filePath}: ${match[1]}`)
			})

		expect(violations).toEqual([])
	})
})

function exportedComponentNames(source: string): string[] {
	return Array.from(source.matchAll(/export \{ default as (\w+) \} from '\.\/[^']+\.vue'/g))
		.map((match) => match[1])
		.sort()
}

function storybookUiImports(source: string): string[] {
	return Array.from(source.matchAll(/import \{([^}]+)\} from '@\/shared\/ui'/g))
		.flatMap((match) => match[1].split(','))
		.map((name) => name.trim())
		.filter(Boolean)
		.sort()
}

function sourceFiles(root: string): string[] {
	return readdirSync(root, { withFileTypes: true }).flatMap((entry: Dirent) => {
		const entryPath = join(root, entry.name)
		if (entry.isDirectory()) {
			return sourceFiles(entryPath)
		}
		return /\.(ts|vue)$/.test(entry.name) ? [entryPath] : []
	})
}
