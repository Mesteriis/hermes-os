import type { StorybookConfig } from '@storybook/vue3-vite'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const storybookDir = dirname(fileURLToPath(import.meta.url))
const allowedStorybookHosts = ['localhost', '127.0.0.1']

const config: StorybookConfig = {
	stories: ['../stories/**/*.mdx', '../stories/**/*.stories.@(ts|tsx|js|jsx|mdx)'],
	staticDirs: ['../public'],
	addons: [
		'@storybook/addon-docs',
		'@storybook/addon-a11y',
		'@storybook/addon-themes',
		'@storybook/addon-vitest',
		'@storybook/addon-coverage',
		'@storybook/addon-designs',
		'msw-storybook-addon',
		'storybook-addon-pseudo-states',
		{
			name: 'storybook-design-token',
			options: {
				designTokenGlob: 'src/shared/ui/{foundation,styles}/**/*.css'
			}
		}
	],
	framework: {
		name: '@storybook/vue3-vite',
		options: {}
	},
	viteFinal(config) {
		config.resolve = config.resolve ?? {}
		const existingAlias = config.resolve.alias
		config.resolve.alias = {
			...(Array.isArray(existingAlias) ? {} : existingAlias),
			'@': resolve(storybookDir, '../src')
		}
		const existingAllowedHosts = config.server?.allowedHosts
		config.server = {
			...config.server,
			allowedHosts: existingAllowedHosts === true
				? true
				: Array.from(new Set([...(Array.isArray(existingAllowedHosts) ? existingAllowedHosts : []), ...allowedStorybookHosts]))
		}
		return config
	},
	docs: {
		autodocs: 'tag'
	}
}

export default config
