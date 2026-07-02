import type { Preview } from '@storybook/vue3-vite'
import { withThemeByDataAttribute } from '@storybook/addon-themes'
import { initialize, mswLoader } from 'msw-storybook-addon'
import { storybookLocaleToolbarItems } from '../stories/ui/storybook-i18n'
import '../src/style.css'
import '../src/styles/surfaces.css'
import '../src/styles/theme-classes.css'
import '../src/shared/ui/styles/index.css'

initialize({
	onUnhandledRequest: 'bypass',
	serviceWorker: {
		url: '/mockServiceWorker.js'
	}
})

const preview: Preview = {
	globalTypes: {
		theme: {
			description: 'Hermes UI theme',
			defaultValue: 'light',
			toolbar: {
				title: 'Theme',
				icon: 'circlehollow',
				items: [
					{ value: 'light', title: 'Light' },
					{ value: 'dark', title: 'Dark' },
					{ value: 'hermes', title: 'Hermes' }
				],
				dynamicTitle: true
			}
		},
		locale: {
			description: 'Hermes UI locale',
			defaultValue: 'ru',
			toolbar: {
				title: 'Locale',
				icon: 'globe',
				items: storybookLocaleToolbarItems,
				dynamicTitle: true
			}
		}
	},
	decorators: [
		withThemeByDataAttribute({
			themes: {
				light: 'light',
				dark: 'dark',
				hermes: 'hermes'
			},
			defaultTheme: 'light',
			attributeName: 'data-ui-theme'
		}),
		(story, context) => ({
			components: { story },
			setup() {
				return {
					locale: context.globals.locale,
					storyHeading: [context.title, context.name].filter(Boolean).join(' / '),
					theme: context.globals.theme
				}
			},
			template: '<main :data-ui-theme="theme" :data-ui-locale="locale" :lang="locale" class="storybook-shell"><h1 class="hermes-sr-only">{{ storyHeading }}</h1><story /></main>'
		})
	],
	loaders: [mswLoader],
	parameters: {
		actions: { argTypesRegex: '^on[A-Z].*' },
		controls: {
			expanded: true,
			matchers: {
				color: /(background|color)$/i,
				date: /Date$/i
			}
		},
		backgrounds: {
			disable: true
		},
		layout: 'fullscreen',
		docs: {
			toc: true
		},
		designToken: {
			defaultTab: 'Colors'
		},
		msw: {
			handlers: []
		},
		viewport: {
			options: {
				uhd4k: {
					name: '4K',
					styles: { width: '3840px', height: '2160px' }
				},
				ipadPro1292020: {
					name: 'iPad Pro 12.9" (2020)',
					styles: { width: '1024px', height: '1366px' }
				},
				zFold7Open: {
					name: 'Samsung Z Fold 7 (open)',
					styles: { width: '1968px', height: '2184px' }
				},
				zFold7Closed: {
					name: 'Samsung Z Fold 7 (closed)',
					styles: { width: '1080px', height: '2520px' }
				},
				macbookPro14: {
					name: 'MacBook Pro 14"',
					styles: { width: '1512px', height: '982px' }
				}
			}
		},
		pseudo: {
			rootSelector: '.storybook-shell'
		}
	}
}

export default preview
