import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'
import {
	ChannelAction as ChannelActionComponent,
	ChannelInspector as ChannelInspectorComponent,
	ChannelList as ChannelListComponent,
	ChannelListItem as ChannelListItemComponent,
	ChannelMessage as ChannelMessageComponent,
	ChannelViewer as ChannelViewerComponent,
	ChannelWorkspace as ChannelWorkspaceComponent,
	CommunicationChannelWorkspace,
	channelActionGroupsFromSubSurface,
	channelComposerCapabilitiesFromSubSurface,
	channelProviderOptionsFromSubSurfaces,
	type CommunicationChannelWorkspaceModel
} from '@/domains/communications/components'
import { useCommunicationsWorkspaceSurface } from '@/domains/communications/queries/useCommunicationsWorkspaceSurface'

const communicationsSurface = useCommunicationsWorkspaceSurface()
const activeChannelSubSurface = communicationsSurface.zulip

const workspace: CommunicationChannelWorkspaceModel = {
	title: 'Channels',
	providerValue: 'channels:zulip',
	providerOptions: channelProviderOptionsFromSubSurfaces(communicationsSurface.subSurfaces),
	activeProviderKind: 'zulip',
	activeProviderLabel: 'Zulip',
	activeAccountLabel: 'Hermes Zulip',
	activeRoomLabel: 'risk-review',
	activeRoomDescription: 'Operational stream for review candidates, source evidence and owner decisions.',
	activeTopicLabel: 'Export SLA',
	rooms: [
		{
			id: 'room-risk',
			providerKind: 'zulip',
			accountId: 'hermes-main',
			label: 'risk-review',
			description: 'Open incidents and review candidates',
			topicCountLabel: '7 topics',
			lastActivityLabel: '11:08',
			unreadCount: 6,
			mentionCount: 1,
			selected: true
		},
		{
			id: 'room-planning',
			providerKind: 'zulip',
			accountId: 'hermes-main',
			label: 'planning',
			description: 'Decision notes and weekly planning',
			topicCountLabel: '4 topics',
			lastActivityLabel: 'Yesterday'
		},
		{
			id: 'room-evidence',
			providerKind: 'zulip',
			accountId: 'hermes-main',
			label: 'evidence-log',
			description: 'Pinned source records and contradictions',
			topicCountLabel: '12 topics',
			lastActivityLabel: 'Mon',
			unreadCount: 2
		},
		{
			id: 'room-release',
			providerKind: 'zulip',
			accountId: 'lab',
			label: 'release-radar',
			description: 'Signals promoted from research workspace',
			topicCountLabel: '3 topics',
			lastActivityLabel: '2d'
		}
	],
	directChatFolders: [
		{
			id: 'folder-owner',
			label: 'Owner conversations',
			expanded: true,
			chats: [
				{
					id: 'dm-nadia-owner',
					providerKind: 'zulip',
					accountId: 'hermes-main',
					label: 'Nadia Ivanova',
					description: 'Export owner clarification',
					avatarLabel: 'NI',
					kindLabel: 'Direct chat',
					lastActivityLabel: '10:58',
					unreadCount: 1
				},
				{
					id: 'dm-legal-owner',
					providerKind: 'zulip',
					accountId: 'hermes-main',
					label: 'Legal review',
					description: 'Retention wording source evidence',
					avatarLabel: 'LR',
					kindLabel: 'Direct chat',
					lastActivityLabel: 'Yesterday'
				}
			]
		},
		{
			id: 'folder-provider-ops',
			label: 'Provider ops',
			expanded: true,
			chats: [
				{
					id: 'dm-sync-bot',
					providerKind: 'zulip',
					accountId: 'lab',
					label: 'Sync monitor',
					description: 'Webhook retries and delivery diagnostics',
					avatarLabel: 'SM',
					kindLabel: 'Bot chat',
					lastActivityLabel: 'Mon',
					mentionCount: 1
				}
			]
		}
	],
	topics: [
		{
			id: 'topic-export-sla',
			label: 'Export SLA',
			summary: 'Owner confirmation before obligation promotion',
			messageCountLabel: '18',
			tone: 'warning',
			selected: true
		},
		{
			id: 'topic-provider-health',
			label: 'Provider health',
			summary: 'Zulip sync latency and retries',
			messageCountLabel: '9',
			tone: 'info'
		},
		{
			id: 'topic-review-queue',
			label: 'Review queue',
			summary: 'Candidates waiting for human review',
			messageCountLabel: '14',
			tone: 'neutral'
		}
	],
	messages: [
		{
			id: 'channel-message-1',
			author: 'Nadia',
			body: 'The account health check moved from degraded to active, but the export blocker is still open.',
			timestamp: '11:02',
			direction: 'inbound',
			meta: 'Zulip · risk-review · Export SLA',
			tone: 'warning'
		},
		{
			id: 'channel-message-2',
			author: 'Hermes',
			body: 'I linked the blocker to the same decision candidate from yesterday. Evidence confidence is medium.',
			timestamp: '11:04',
			direction: 'system',
			meta: 'candidate explanation'
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
	composerPlaceholder: 'Reply in #risk-review · Export SLA',
	composerCapabilities: channelComposerCapabilitiesFromSubSurface(activeChannelSubSurface),
	actionGroups: channelActionGroupsFromSubSurface(activeChannelSubSurface),
	inspector: {
		intelligence: {
			score: 76,
			maxScore: 100,
			label: 'Channel confidence',
			summary: 'Export SLA has enough source context to stay in review, but owner confirmation is still missing.',
			checks: [
				{
					id: 'source-evidence',
					label: 'Source evidence linked',
					description: 'Topic is tied to mail_thread and attachment:redline.',
					tone: 'success',
					icon: 'tabler:shield-check'
				},
				{
					id: 'owner-confirmation',
					label: 'Owner confirmation missing',
					description: 'Do not promote the obligation until the export owner confirms the SLA.',
					tone: 'warning',
					icon: 'tabler:alert-triangle'
				},
				{
					id: 'provider-context',
					label: 'Provider context preserved',
					description: 'Zulip stream, topic, account and message provenance are retained.',
					tone: 'info',
					icon: 'tabler:messages'
				}
			]
		},
		entityGroups: [
			{
				id: 'personas',
				title: 'Personas',
				items: [
					{
						id: 'entity-nadia',
						entity: 'persona',
						title: 'Nadia Ivanova',
						description: 'Reporter in the active channel topic.',
						evidenceLabel: 'Mentioned in risk-review · Export SLA',
						tone: 'info'
					},
					{
						id: 'entity-owner',
						entity: 'persona',
						title: 'Export owner',
						description: 'Likely approver before durable promotion.',
						evidenceLabel: 'Appears in two channel topics and one mail thread',
						tone: 'success'
					}
				]
			},
			{
				id: 'work',
				title: 'Work objects',
				items: [
					{
						id: 'candidate-sla',
						entity: 'obligation',
						title: 'Export SLA confirmation',
						description: 'Candidate obligation waiting for source confirmation.',
						evidenceLabel: 'Mentioned in active Zulip topic',
						tone: 'warning'
					},
					{
						id: 'candidate-blocker',
						entity: 'project',
						title: 'Export blocker',
						description: 'Related project context from previous risk review.',
						evidenceLabel: 'Linked by channel topic and yesterday decision note',
						tone: 'info'
					}
				]
			},
			{
				id: 'evidence',
				title: 'Evidence',
				items: [
					{
						id: 'evidence-redline',
						entity: 'document',
						title: 'retention-clause-redline.docx',
						description: 'Attachment evidence referenced by the channel discussion.',
						evidenceLabel: 'attachment:redline',
						tone: 'warning'
					}
				]
			}
		],
		topics: [
			{ id: 'topic-export-sla', label: 'Export SLA', tone: 'warning' },
			{ id: 'topic-review-queue', label: 'Review queue', tone: 'info' },
			{ id: 'topic-source-evidence', label: 'Source evidence', tone: 'success' }
		],
		semanticFacts: [
			{
				id: 'intent',
				label: 'Intent',
				value: 'Keep obligation candidate in review until owner confirmation.',
				tone: 'warning'
			},
			{
				id: 'review-state',
				label: 'Review state',
				value: 'Candidate, not durable business truth.',
				tone: 'info'
			},
			{
				id: 'evidence-path',
				label: 'Evidence path',
				value: 'zulip:risk-review/export-sla + mail_thread + attachment:redline.'
			}
		],
		suggestedActions: [
			{
				id: 'create-task',
				label: 'Create task',
				description: 'Ask export owner to confirm SLA wording.',
				icon: 'tabler:checkbox',
				tone: 'warning',
				contract: 'review.create_task_from_channel'
			},
			{
				id: 'ask-direct-chat',
				label: 'Ask in direct chat',
				description: 'Move the clarification to a Zulip direct chat with the owner.',
				icon: 'tabler:message-forward',
				tone: 'info',
				contract: 'communication.channel.direct_message'
			},
			{
				id: 'attach-evidence',
				label: 'Attach evidence',
				description: 'Bind redline attachment and topic messages to the candidate.',
				icon: 'tabler:paperclip',
				tone: 'success',
				contract: 'evidence.attach_source'
			},
			{
				id: 'promote-after-review',
				label: 'Promote after review',
				description: 'Create durable obligation only after source confirmation.',
				icon: 'tabler:scale',
				tone: 'accent',
				contract: 'review.promote_obligation'
			}
		],
		relatedContext: [
			{
				id: 'context-mail-thread',
				title: 'Vendor security review',
				description: 'Mail thread containing retention wording and redline.',
				icon: 'tabler:mail',
				tone: 'info'
			},
			{
				id: 'context-decision',
				title: 'Retention clause approval',
				description: 'Decision candidate waiting for legal and owner review.',
				icon: 'tabler:git-branch',
				tone: 'warning'
			},
			{
				id: 'context-direct-chat',
				title: 'Nadia Ivanova direct chat',
				description: 'Owner clarification can move into a Zulip direct conversation.',
				icon: 'tabler:message',
				tone: 'accent'
			}
		]
	}
}

const selectedRoom = workspace.rooms[0] as (typeof workspace.rooms)[number]

const meta = {
	title: 'Hermes App/Communications/Channels',
	component: CommunicationChannelWorkspace
} satisfies Meta<typeof CommunicationChannelWorkspace>

export default meta
type Story = StoryObj<typeof meta>

export const ChannelListItem: Story = {
	render: () => ({
		components: { ChannelListItemComponent },
		data() {
			return { selectedRoom }
		},
		template: `
			<section class="storybook-canvas">
				<div style="width: 360px">
					<ChannelListItemComponent :room="selectedRoom" />
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('button', { name: /Zulip, #risk-review/ })).toBeVisible()
	}
}

export const ChannelList: Story = {
	render: () => ({
		components: { ChannelListComponent },
		data() {
			return { workspace }
		},
		template: `
			<section class="storybook-canvas">
				<div style="width: 380px; height: 620px">
					<ChannelListComponent
						:provider-value="workspace.providerValue"
						:provider-options="workspace.providerOptions"
						:rooms="workspace.rooms"
						:direct-chat-folders="workspace.directChatFolders"
					/>
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('combobox', { name: 'Channel provider' })).toBeVisible()
		await expect(canvas.getByText('#risk-review')).toBeVisible()
		await expect(canvas.getByText('Direct chats')).toBeVisible()
		await expect(canvas.getByText('Nadia Ivanova')).toBeVisible()
	}
}

export const ChannelAction: Story = {
	render: () => ({
		components: { ChannelActionComponent },
		data() {
			return { actionGroups: workspace.actionGroups }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<ChannelActionComponent :action-groups="actionGroups" />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('navigation', { name: 'Channel actions' })).toBeVisible()
		await expect(canvas.getByRole('button', { name: 'Outbound commands' })).toBeVisible()
	}
}

export const ChannelViewer: Story = {
	render: () => ({
		components: { ChannelViewerComponent },
		data() {
			return { workspace }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<div style="height: 680px">
					<ChannelViewerComponent :workspace="workspace" />
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('region', { name: 'Channel stream' })).toBeVisible()
		await expect(canvas.getAllByText('Export SLA')[0]).toBeVisible()
		await expect(canvas.getByRole('textbox', { name: 'Channel message' })).toBeVisible()
	}
}

export const ChannelInspector: Story = {
	render: () => ({
		components: { ChannelInspectorComponent },
		data() {
			return { model: workspace.inspector }
		},
		template: `
			<section class="storybook-canvas">
				<div style="width: 360px">
					<ChannelInspectorComponent :model="model" />
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByText('Channel Intelligence')).toBeVisible()
		await expect(canvas.getByText('Export SLA confirmation')).toBeVisible()
		await expect(canvas.getByText('Ask in direct chat')).toBeVisible()
	}
}

export const ChannelMessage: Story = {
	render: () => ({
		components: { ChannelMessageComponent },
		data() {
			return { workspace }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<div style="height: 720px">
					<ChannelMessageComponent :workspace="workspace" />
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('navigation', { name: 'Channel actions' })).toBeVisible()
		await expect(canvas.getByText('#risk-review')).toBeVisible()
	}
}

export const WorkspaceChannels: Story = {
	render: () => ({
		components: { ChannelWorkspaceComponent },
		data() {
			return { workspace }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide storybook-canvas--workspace">
				<ChannelWorkspaceComponent :workspace="workspace" />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('region', { name: 'Channels workspace' })).toBeVisible()
		await expect(canvas.getByText('Channel Intelligence')).toBeVisible()
	}
}
