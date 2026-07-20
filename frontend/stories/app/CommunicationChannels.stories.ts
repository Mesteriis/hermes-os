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
import type { CommunicationSubSurface } from '@/domains/communications/queries/communicationChannelSurface'

// Storybook uses an explicit fixture catalog: route admission must never create
// a legacy provider/query runtime merely to render an unavailable page.
const channelFixtureSurfaces: readonly CommunicationSubSurface[] = [
	{
		channelId: 'zulip',
		labelKey: 'Zulip',
		status: 'active',
		businessQueryRoot: ['communications', 'channels'],
		runtimeQueryRoot: ['integrations', 'zulip', 'runtime'],
		capabilityNotes: [
			'Zulip stream, topic and direct-message UI is represented as the Channels sub-surface.',
			'Provider writes remain provider commands and inbound events remain raw-to-accepted Communications evidence.'
		],
		capabilityGroups: [
			{
				id: 'zulip-outbound', labelKey: 'Outbound commands', menuLabelKey: 'Open Zulip outbound commands', icon: 'tabler:send', status: 'available',
				capabilities: [
					{ id: 'send-stream-message', labelKey: 'Send stream message', descriptionKey: 'Compose a message into a Zulip stream and topic.', icon: 'tabler:message-share', status: 'available', kind: 'command', contract: 'send_stream_message' },
					{ id: 'send-direct-message', labelKey: 'Send direct message', descriptionKey: 'Send to resolved recipient emails or Zulip user ids.', icon: 'tabler:message', status: 'available', kind: 'command', contract: 'send_direct_message' },
					{ id: 'upload-file', labelKey: 'Upload file', descriptionKey: 'Prepare a provider upload for later stream or direct message composition.', icon: 'tabler:paperclip', status: 'available', kind: 'command', contract: 'upload_file' },
					{ id: 'send-stream-message-with-upload', labelKey: 'Send stream message with upload', descriptionKey: 'Attach a prepared upload and reference it from stream content.', icon: 'tabler:file-upload', status: 'available', kind: 'command', contract: 'send_stream_message_with_upload' },
					{ id: 'send-direct-message-with-upload', labelKey: 'Send direct message with upload', descriptionKey: 'Attach a prepared upload to a direct Zulip message.', icon: 'tabler:message-up', status: 'available', kind: 'command', contract: 'send_direct_message_with_upload' }
				]
			},
			{
				id: 'zulip-message-lifecycle', labelKey: 'Message lifecycle', menuLabelKey: 'Open Zulip message lifecycle actions', icon: 'tabler:activity', status: 'available',
				capabilities: [
					{ id: 'update-message', labelKey: 'Update message content or topic', descriptionKey: 'Supports content, topic, stream id and propagate mode in one lifecycle action.', icon: 'tabler:edit', status: 'available', kind: 'command', contract: 'update_message' },
					{ id: 'delete-message', labelKey: 'Delete message', descriptionKey: 'Deletion is shown as provider action plus local tombstone/evidence state.', icon: 'tabler:trash', status: 'available', kind: 'command', contract: 'delete_message' },
					{ id: 'add-reaction', labelKey: 'Add reaction', descriptionKey: 'Add a Zulip reaction while preserving provider event reconciliation.', icon: 'tabler:mood-plus', status: 'available', kind: 'command', contract: 'add_reaction' },
					{ id: 'remove-reaction', labelKey: 'Remove reaction', descriptionKey: 'Remove a Zulip reaction and reconcile the resulting reaction event.', icon: 'tabler:mood-minus', status: 'available', kind: 'command', contract: 'remove_reaction' }
				]
			},
			{
				id: 'zulip-event-projection', labelKey: 'Event ingest and projection', menuLabelKey: 'Open Zulip event trace actions', icon: 'tabler:route', status: 'available',
				capabilities: [
					{ id: 'raw-message-observed', labelKey: 'Raw message observed', descriptionKey: 'Raw provider message with stream, topic, sender and content provenance.', icon: 'tabler:message-circle', status: 'available', kind: 'projection', contract: 'signal.raw.zulip.message.observed' },
					{ id: 'raw-reaction-observed', labelKey: 'Raw reaction observed', descriptionKey: 'Raw reaction events preserve emoji, operation and provider message id.', icon: 'tabler:mood-smile', status: 'available', kind: 'projection', contract: 'signal.raw.zulip.reaction.observed' },
					{ id: 'raw-message-update-observed', labelKey: 'Raw message update observed', descriptionKey: 'Edited content and previous topic/content are kept as source evidence.', icon: 'tabler:message-2-share', status: 'available', kind: 'projection', contract: 'signal.raw.zulip.message_update.observed' },
					{ id: 'raw-message-delete-observed', labelKey: 'Raw message delete observed', descriptionKey: 'Delete events produce tombstone-style channel state.', icon: 'tabler:tombstone', status: 'available', kind: 'projection', contract: 'signal.raw.zulip.message_delete.observed' },
					{ id: 'accepted-zulip-message', labelKey: 'Accepted Zulip message', descriptionKey: 'Accepted Zulip messages become provider-neutral Communications channel evidence.', icon: 'tabler:database-import', status: 'available', kind: 'projection', contract: 'signal.accepted.zulip.message' }
				]
			},
			{
				id: 'zulip-composer', labelKey: 'Composer', menuLabelKey: 'Open Zulip composer tools', icon: 'tabler:edit', status: 'available',
				capabilities: [
					{ id: 'change-topic', labelKey: 'Change topic', descriptionKey: 'Move the draft to another Zulip topic before sending.', icon: 'tabler:message-circle-2', status: 'available', kind: 'composer' },
					{ id: 'mention-participant', labelKey: 'Mention participant', descriptionKey: 'Insert a Zulip mention into the channel draft.', icon: 'tabler:at', status: 'available', kind: 'composer' },
					{ id: 'attach-evidence', labelKey: 'Attach evidence', descriptionKey: 'Attach source evidence through the prepared upload path.', icon: 'tabler:paperclip', status: 'available', kind: 'composer', contract: 'ZulipPreparedUpload' },
					{ id: 'code-block', labelKey: 'Code block', descriptionKey: 'Insert a code block into Zulip message content.', icon: 'tabler:code', status: 'available', kind: 'composer' },
					{ id: 'create-poll', labelKey: 'Create poll', descriptionKey: 'Prepare a poll-style prompt for channels that support it.', icon: 'tabler:list-check', status: 'partial', kind: 'composer' },
					{ id: 'scheduled-send', labelKey: 'Schedule send', descriptionKey: 'Represent scheduled send intent before provider support is finalized.', icon: 'tabler:clock', status: 'facade', kind: 'composer', disabled: true }
				]
			}
		]
	},
	...(['slack', 'discord', 'mattermost'] as const).map((channelId) => ({
		channelId,
		labelKey: channelId === 'slack' ? 'Slack' : channelId === 'discord' ? 'Discord' : 'Mattermost',
		status: 'facade' as const,
		businessQueryRoot: ['communications', 'channels'] as const,
		capabilityNotes: ['Provider surface is present in the compiled UI but is not admitted.'],
		capabilityGroups: []
	}))
]
const activeChannelSubSurface = channelFixtureSurfaces[0]!

const workspace: CommunicationChannelWorkspaceModel = {
	title: 'Channels',
	providerValue: 'channels:zulip',
	providerOptions: channelProviderOptionsFromSubSurfaces(channelFixtureSurfaces),
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
