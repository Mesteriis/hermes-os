import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'
import {
	CommunicationCallsSurface,
	type CommunicationCallsSurfaceModel
} from '@/domains/communications/components'

const callsSurface: CommunicationCallsSurfaceModel = {
	title: 'Calls',
	subtitle: 'Phone, voice note and meeting call evidence before promotion.',
	calls: [
		{
			id: 'call-budget-review',
			channelKind: 'whatsapp',
			title: 'Budget review call',
			participants: 'Maya Chen, Owner',
			startedAt: 'Today, 12:20',
			durationLabel: '18 min',
			state: 'transcribing',
			summary: 'Retention wording approved, but budget owner asked for a written confirmation.',
			selected: true
		},
		{
			id: 'call-project-checkin',
			channelKind: 'telegram',
			title: 'Project check-in',
			participants: 'Operations workspace',
			startedAt: 'Yesterday',
			durationLabel: '42 min',
			state: 'completed',
			summary: 'Two follow-up candidates were reviewed and one was dismissed.'
		},
		{
			id: 'call-missed-legal',
			channelKind: 'mail',
			title: 'Legal callback',
			participants: 'Legal desk',
			startedAt: 'Mon',
			durationLabel: 'missed',
			state: 'missed',
			summary: 'Missed call remains a signal until matched with email evidence.'
		}
	],
	moments: [
		{
			id: 'moment-1',
			timestamp: '03:12',
			speaker: 'Maya',
			text: 'The retention window is acceptable if we keep exported reports to thirty days.',
			tone: 'info'
		},
		{
			id: 'moment-2',
			timestamp: '09:40',
			speaker: 'Owner',
			text: 'Please do not treat this as final until legal replies in writing.',
			tone: 'warning'
		},
		{
			id: 'moment-3',
			timestamp: '15:08',
			speaker: 'Hermes',
			text: 'Candidate obligation extracted: request written confirmation from legal.',
			tone: 'neutral'
		}
	],
	inspectorSections: [
		{
			id: 'call-candidates',
			title: 'Extracted candidates',
			items: [
				{
					id: 'candidate-written-confirmation',
					entity: 'task',
					title: 'Request written confirmation',
					description: 'Task candidate requires review because the source is a call transcript.',
					evidenceLabel: 'Call transcript moment 09:40',
					tone: 'warning'
				},
				{
					id: 'candidate-retention-decision',
					entity: 'decision',
					title: 'Retention window decision',
					description: 'Decision candidate stays provisional until email evidence arrives.',
					evidenceLabel: 'Call transcript moment 03:12',
					tone: 'info'
				}
			]
		}
	]
}

const meta = {
	title: 'Hermes App/Communications/Calls',
	component: CommunicationCallsSurface
} satisfies Meta<typeof CommunicationCallsSurface>

export default meta
type Story = StoryObj<typeof meta>

export const CallReview: Story = {
	render: () => ({
		components: { CommunicationCallsSurface },
		data() {
			return { callsSurface }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<CommunicationCallsSurface :surface="callsSurface" />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('complementary', { name: 'Calls' })).toBeVisible()
		await expect(canvas.getByText('Call transcript')).toBeVisible()
		await expect(canvas.getByText('Request written confirmation')).toBeVisible()
	}
}
