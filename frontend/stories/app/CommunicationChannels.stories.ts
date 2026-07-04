import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'
import {
	CommunicationChannelWorkspace,
	type CommunicationChannelWorkspaceModel
} from '@/domains/communications/components'

const workspace: CommunicationChannelWorkspaceModel = {
	title: 'Operations workspace',
	subtitle: 'Slack-like channel surface for persistent group communication.',
	activeRoomLabel: 'risk-review',
	rooms: [
		{
			id: 'room-risk',
			label: 'risk-review',
			description: 'Open incidents and review candidates',
			unreadCount: 6,
			selected: true
		},
		{
			id: 'room-planning',
			label: 'planning',
			description: 'Decision notes and weekly planning'
		},
		{
			id: 'room-evidence',
			label: 'evidence-log',
			description: 'Pinned source records and contradictions',
			unreadCount: 2
		}
	],
	messages: [
		{
			id: 'channel-message-1',
			author: 'Nadia',
			body: 'The account health check moved from degraded to active, but the export blocker is still open.',
			timestamp: '11:02',
			direction: 'inbound',
			meta: 'thread root',
			tone: 'warning'
		},
		{
			id: 'channel-message-2',
			author: 'Hermes',
			body: 'I linked the blocker to the same decision candidate from yesterday. Evidence confidence is medium.',
			timestamp: '11:04',
			direction: 'system',
			meta: 'context link'
		},
		{
			id: 'channel-message-3',
			author: 'Owner',
			body: 'Keep this in review. Do not promote the obligation until the export owner confirms the SLA.',
			timestamp: '11:08',
			direction: 'outbound',
			meta: 'owner instruction'
		}
	],
	inspectorSections: [
		{
			id: 'channel-thread-candidates',
			title: 'Thread candidates',
			items: [
				{
					id: 'candidate-sla',
					entity: 'obligation',
					title: 'Export SLA confirmation',
					description: 'Candidate obligation waiting for source confirmation.',
					evidenceLabel: 'Mentioned in active channel thread',
					tone: 'warning'
				},
				{
					id: 'candidate-blocker',
					entity: 'project',
					title: 'Export blocker',
					description: 'Related project context from previous risk review.',
					evidenceLabel: 'Linked by channel thread and yesterday decision note',
					tone: 'info'
				}
			]
		}
	]
}

const meta = {
	title: 'Hermes App/Communications/Channels',
	component: CommunicationChannelWorkspace
} satisfies Meta<typeof CommunicationChannelWorkspace>

export default meta
type Story = StoryObj<typeof meta>

export const WorkspaceChannels: Story = {
	render: () => ({
		components: { CommunicationChannelWorkspace },
		data() {
			return { workspace }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<CommunicationChannelWorkspace :workspace="workspace" />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('complementary', { name: 'Channel rooms' })).toBeVisible()
		await expect(canvas.getByText('#risk-review')).toBeVisible()
		await expect(canvas.getByText('Export SLA confirmation')).toBeVisible()
	}
}
