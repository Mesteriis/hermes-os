import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ToggleGroup } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Hermes UI/General/Toggle Group',
	component: ToggleGroup,
	render: (_args, context) => ({
		components: { ToggleGroup },
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return { copy, selected: 'review', selectedMany: ['review', 'evidence'] }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.controls.toggleGroup }}</h2>
					<ToggleGroup v-model="selected" :aria-label="copy.controls.toggleGroup" :items="copy.toggles" />
					<ToggleGroup v-model="selectedMany" multiple :aria-label="copy.controls.toggleGroup" :items="copy.toggles" />
				</div>
			</section>
		`
	})
} satisfies Meta<typeof ToggleGroup>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
