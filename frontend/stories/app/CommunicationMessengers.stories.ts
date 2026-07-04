import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'
import {
	CommunicationWorkspaceShell,
	type CommunicationConversationModel,
	type CommunicationHermesInspectorSectionModel,
	type CommunicationInboxItemModel
} from '@/domains/communications/components'

const messengerInboxItems: readonly CommunicationInboxItemModel[] = [
	{
		id: 'telegram-owner-radar',
		kind: 'direct_chat',
		channelKind: 'telegram',
		title: 'Anna Petrova',
		subtitle: 'Telegram · direct chat',
		preview: 'Can you check the Radar summary before the meeting starts?',
		timestamp: '10:14',
		workflowState: 'needs_action',
		unreadCount: 3,
		hasOpenAction: true,
		selected: true
	},
	{
		id: 'whatsapp-family',
		kind: 'group_chat',
		channelKind: 'whatsapp',
		title: 'Family logistics',
		subtitle: 'WhatsApp · group',
		preview: 'Dinner moved to 19:30, calendar candidate is waiting.',
		timestamp: '09:58',
		workflowState: 'waiting',
		unreadCount: 1
	},
	{
		id: 'telegram-product',
		kind: 'group_chat',
		channelKind: 'telegram',
		title: 'Product backchannel',
		subtitle: 'Telegram · group',
		preview: 'The support escalation was resolved and archived.',
		timestamp: 'Yesterday',
		workflowState: 'reviewed',
		muted: true
	}
]

const messengerConversation: CommunicationConversationModel = {
	id: 'telegram-owner-radar',
	channelKind: 'telegram',
	title: 'Anna Petrova',
	subtitle: 'Telegram direct chat · realtime-style surface',
	workflowState: 'needs_action',
	facts: [
		{ label: 'Unread', value: 3 },
		{ label: 'Signals', value: 1 },
		{ label: 'Provider', value: 'Telegram' }
	],
	messages: [
		{
			id: 'msg-telegram-1',
			author: 'Anna',
			body: 'Can you check the Radar summary before the meeting starts?',
			timestamp: '10:12',
			direction: 'inbound',
			tone: 'warning',
			meta: 'Telegram'
		},
		{
			id: 'msg-telegram-2',
			author: 'Hermes',
			body: 'I found two conflicting dates in the thread. The Friday slot has stronger evidence.',
			timestamp: '10:13',
			direction: 'system',
			meta: 'Candidate explanation'
		},
		{
			id: 'msg-telegram-3',
			author: 'Owner',
			body: 'Use Friday as the working assumption and keep the Tuesday note as low confidence.',
			timestamp: '10:14',
			direction: 'outbound',
			meta: 'Drafted reply'
		}
	],
	draftPreview: 'Friday is the safer date. I will keep Tuesday as a low confidence note until we get confirmation.'
}

const messengerInspectorSections: readonly CommunicationHermesInspectorSectionModel[] = [
	{
		id: 'messenger-candidates',
		title: 'Candidates',
		items: [
			{
				id: 'candidate-meeting',
				entity: 'event',
				title: 'Meeting date clarification',
				description: 'Potential calendar context from conflicting chat evidence.',
				evidenceLabel: '3 Telegram messages mention Friday and Tuesday',
				tone: 'warning'
			},
			{
				id: 'candidate-anna',
				entity: 'person',
				title: 'Anna Petrova',
				description: 'Known relationship context attached to the direct chat.',
				evidenceLabel: 'Existing person match with fresh message evidence',
				tone: 'info'
			}
		]
	}
]

const meta = {
	title: 'Hermes App/Communications/Messengers',
	component: CommunicationWorkspaceShell
} satisfies Meta<typeof CommunicationWorkspaceShell>

export default meta
type Story = StoryObj<typeof meta>

export const DirectChats: Story = {
	render: () => ({
		components: { CommunicationWorkspaceShell },
		data() {
			return {
				inboxItems: messengerInboxItems,
				conversation: messengerConversation,
				inspectorSections: messengerInspectorSections
			}
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<CommunicationWorkspaceShell
					:inbox-items="inboxItems"
					:conversation="conversation"
					:inspector-sections="inspectorSections"
				/>
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('heading', { name: 'Anna Petrova', level: 2 })).toBeVisible()
		await expect(canvas.getAllByText('Can you check the Radar summary before the meeting starts?')[0]).toBeVisible()
		await expect(canvas.getByLabelText('Reply')).toBeVisible()
		await expect(canvas.getByText('Meeting date clarification')).toBeVisible()
	}
}
