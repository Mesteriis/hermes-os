import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'
import {
	CallAction,
	CallInspector,
	CallList,
	CallListItem,
	CallMessage,
	CallViewer,
	CallWorkspace,
	CommunicationCallsSurface,
	type CommunicationCallsSurfaceModel
} from '@/domains/communications/components'

const callsSurface: CommunicationCallsSurfaceModel = {
	title: 'Calls',
	providerValue: 'calls:all',
	providerOptions: [
		{ value: 'calls:all', label: 'All call providers', icon: 'tabler:phone-call' },
		{
			value: 'calls:zoom',
			label: 'Zoom',
			icon: 'tabler:brand-zoom',
			children: [
				{ value: 'calls:zoom:owner', label: 'Owner Zoom', icon: 'tabler:user-circle' },
				{ value: 'calls:zoom:work', label: 'Work Zoom', icon: 'tabler:building' }
			]
		},
		{
			value: 'calls:telemost',
			label: 'Yandex Telemost',
			icon: 'tabler:video',
			children: [
				{ value: 'calls:telemost:personal', label: 'Personal Telemost', icon: 'tabler:user' },
				{ value: 'calls:telemost:team', label: 'Team Telemost', icon: 'tabler:users' }
			]
		},
		{
			value: 'calls:zulip',
			label: 'Zulip',
			icon: 'tabler:messages',
			children: [
				{ value: 'calls:zulip:hermes', label: 'Hermes Zulip', icon: 'tabler:hash' },
				{ value: 'calls:zulip:ops', label: 'Ops Zulip', icon: 'tabler:users-group' }
			]
		}
	],
	permanentMeetings: [
		{
			id: 'permanent-zoom-product-review',
			providerKind: 'zoom',
			providerLabel: 'Zoom',
			title: 'Product review room',
			description: 'Owner Zoom link',
			href: '#zoom-product-review-room',
			statusLabel: 'Permanent',
			tone: 'info'
		},
		{
			id: 'permanent-telemost-ops',
			providerKind: 'telemost',
			providerLabel: 'Yandex Telemost',
			title: 'Ops incident room',
			description: 'Team Telemost link',
			href: '#telemost-ops-incident-room',
			statusLabel: 'Always open',
			tone: 'success'
		}
	],
	calls: [
		{
			id: 'call-retention-review',
			providerKind: 'zoom',
			kind: 'scheduled',
			dateGroupLabel: 'Today',
			sortKey: '2026-07-05T12:20:00',
			title: 'Retention review',
			subtitle: 'Security vendor meeting',
			providerLabel: 'Zoom',
			participantsLabel: '4 participants',
			startedAtLabel: 'Today, 12:20',
			durationLabel: '38 min',
			state: 'transcribing',
			summary: 'Owner approved the short retention window, but legal confirmation is still required.',
			avatarLabel: 'RR',
			recordingCount: 2,
			transcriptStateLabel: 'Transcript ready',
			unreadCount: 2,
			selected: true
		},
		{
			id: 'call-product-weekly-review',
			providerKind: 'telemost',
			kind: 'scheduled',
			dateGroupLabel: 'Tomorrow',
			sortKey: '2026-07-06T10:00:00',
			title: 'Product weekly review',
			subtitle: 'Standing roadmap meeting',
			providerLabel: 'Yandex Telemost',
			participantsLabel: 'Core team',
			startedAtLabel: 'Tomorrow, 10:00',
			durationLabel: '50 min',
			state: 'scheduled',
			summary: 'Standing meeting where roadmap decisions and owner obligations are captured.',
			avatarLabel: 'PR',
			recordingCount: 12,
			transcriptStateLabel: 'Auto transcript',
			recurrenceLabel: 'Weekly'
		},
		{
			id: 'call-zulip-risk-huddle',
			providerKind: 'zulip',
			kind: 'recording',
			dateGroupLabel: 'Today',
			sortKey: '2026-07-05T11:04:00',
			title: 'Risk review huddle',
			subtitle: 'Zulip channel call',
			providerLabel: 'Zulip',
			participantsLabel: '#risk-review',
			startedAtLabel: 'Today, 11:04',
			durationLabel: '16 min',
			state: 'recording',
			summary: 'Zulip channel call linked to Export SLA topic and source-evidence review.',
			avatarLabel: 'ZH',
			recordingCount: 1,
			transcriptStateLabel: 'Channel transcript',
			recurrenceLabel: 'Channel call'
		},
		{
			id: 'call-sales-sync',
			providerKind: 'zoom',
			kind: 'recording',
			dateGroupLabel: 'Yesterday',
			sortKey: '2026-07-04T16:15:00',
			title: 'Partner sync recording',
			subtitle: 'Recorded conversation',
			providerLabel: 'Zoom',
			participantsLabel: 'Alex Johnson, Owner',
			startedAtLabel: 'Yesterday',
			durationLabel: '24 min',
			state: 'completed',
			summary: 'Partnership proposal was discussed and converted into context candidates.',
			avatarLabel: 'PS',
			recordingCount: 1,
			transcriptStateLabel: 'Reviewed'
		}
	],
	activeCall: {
		id: 'call-retention-review',
		title: 'Retention review',
		subtitle: 'Security vendor meeting',
		statusLabel: 'Transcribing',
		statusTone: 'warning',
		providerKind: 'zoom',
		providerLabel: 'Zoom',
		startedAtLabel: 'Today, 12:20',
		durationLabel: '38 min',
		participantCountLabel: '4 participants',
		recurrenceLabel: 'Created from calendar event',
		recordingStatusLabel: '2 recordings attached',
		transcriptStatusLabel: 'Transcript indexed',
		summary: 'Calls stay as evidence until transcript, recordings and review candidates agree.',
		recordings: [
			{
				id: 'recording-main',
				title: 'retention-review-main.mp4',
				meta: 'Video · 184 MB',
				statusLabel: 'Indexed',
				icon: 'tabler:video',
				tone: 'info'
			},
			{
				id: 'recording-audio',
				title: 'retention-review-audio.m4a',
				meta: 'Audio · 32 MB',
				statusLabel: 'Transcript source',
				icon: 'tabler:wave-sine',
				tone: 'success'
			}
		],
		moments: [
			{
				id: 'moment-1',
				timestamp: '03:12',
				speaker: 'Maya',
				text: 'The retention window is acceptable if exported reports are kept to thirty days.',
				tone: 'info',
				evidenceLabel: 'Transcript segment · recording-main'
			},
			{
				id: 'moment-2',
				timestamp: '09:40',
				speaker: 'Owner',
				text: 'Please do not treat this as final until legal replies in writing.',
				tone: 'warning',
				evidenceLabel: 'Decision blocker · owner instruction'
			},
			{
				id: 'moment-3',
				timestamp: '21:08',
				speaker: 'Hermes',
				text: 'Candidate obligation extracted: request written confirmation from legal before promotion.',
				tone: 'neutral',
				evidenceLabel: 'AI candidate · needs review'
			}
		]
	},
	actionGroups: [
		{
			id: 'meeting',
			title: 'Meeting actions',
			icon: 'tabler:calendar-plus',
			menuLabel: 'Open meeting actions',
			actions: [
				{
					id: 'create-meeting',
					label: 'Create meeting',
					description: 'Schedule a one-off meeting.',
					icon: 'tabler:calendar-plus',
					contract: 'communications.calls.meeting.create'
				},
				{
					id: 'create-recurring-room',
					label: 'Create recurring room',
					description: 'Create a permanent or recurring call room.',
					icon: 'tabler:repeat',
					contract: 'communications.calls.room.create'
				},
				{
					id: 'join-room',
					label: 'Join active room',
					description: 'Open the provider room.',
					icon: 'tabler:login-2',
					contract: 'communications.calls.room.join'
				}
			]
		},
		{
			id: 'recording',
			title: 'Recording and transcript',
			icon: 'tabler:record-mail',
			menuLabel: 'Open recording actions',
			actions: [
				{
					id: 'start-recording',
					label: 'Start recording',
					description: 'Request provider recording if capability is available.',
					icon: 'tabler:player-record',
					contract: 'communications.calls.recording.start'
				},
				{
					id: 'attach-recording',
					label: 'Attach recording',
					description: 'Attach a local or provider recording as evidence.',
					icon: 'tabler:paperclip',
					contract: 'communications.calls.recording.attach'
				},
				{
					id: 'retranscribe',
					label: 'Retranscribe',
					description: 'Regenerate transcript from selected recording.',
					icon: 'tabler:file-text-ai',
					contract: 'communications.calls.transcript.rebuild'
				}
			]
		},
		{
			id: 'intelligence',
			title: 'Hermes intelligence',
			icon: 'tabler:sparkles',
			menuLabel: 'Open call intelligence actions',
			actions: [
				{
					id: 'extract-decisions',
					label: 'Extract decisions',
					description: 'Create review candidates from transcript evidence.',
					icon: 'tabler:git-branch',
					contract: 'communications.calls.ai.extract_decisions'
				},
				{
					id: 'create-summary',
					label: 'Create call summary',
					description: 'Build a cited summary from transcript and recordings.',
					icon: 'tabler:file-analytics',
					contract: 'communications.calls.summary.create'
				},
				{
					id: 'promote-actions',
					label: 'Promote reviewed actions',
					description: 'Promote approved candidates into durable domains.',
					icon: 'tabler:circle-check',
					contract: 'communications.calls.review.promote'
				}
			]
		}
	],
	inspector: {
		intelligence: {
			label: 'Call evidence confidence',
			score: 84,
			maxScore: 100,
			summary: 'Recording and transcript agree, but legal confirmation is missing before promotion.',
			checks: [
				{
					id: 'recording-linked',
					label: 'Recording linked',
					description: 'Video and audio sources are attached as evidence.',
					tone: 'success',
					icon: 'tabler:record-mail'
				},
				{
					id: 'speaker-map',
					label: 'Speaker map stable',
					description: 'Four speakers matched to known Personas and owner context.',
					tone: 'success',
					icon: 'tabler:user-check'
				},
				{
					id: 'legal-blocker',
					label: 'Legal blocker',
					description: 'Owner explicitly requested written confirmation before final decision.',
					tone: 'warning',
					icon: 'tabler:alert-triangle'
				}
			]
		},
		entityGroups: [
			{
				id: 'personas',
				title: 'Personas',
				items: [
					{
						id: 'maya',
						entity: 'persona',
						title: 'Maya Chen',
						description: 'Security vendor participant',
						evidenceLabel: 'Speaker segments 03:12, 12:44',
						tone: 'info'
					},
					{
						id: 'legal',
						entity: 'organization',
						title: 'Legal desk',
						description: 'Required approver before promotion',
						evidenceLabel: 'Owner instruction 09:40',
						tone: 'warning'
					}
				]
			},
			{
				id: 'candidates',
				title: 'Hermes candidates',
				items: [
					{
						id: 'written-confirmation',
						entity: 'task',
						title: 'Request written confirmation',
						description: 'Task candidate created from transcript evidence.',
						evidenceLabel: 'Moment 09:40',
						tone: 'warning'
					},
					{
						id: 'retention-decision',
						entity: 'decision',
						title: 'Retention window decision',
						description: 'Decision candidate remains provisional.',
						evidenceLabel: 'Moment 03:12 + recording-main',
						tone: 'info'
					}
				]
			}
		],
		topics: [
			{ id: 'security', label: 'Security', tone: 'info' },
			{ id: 'retention', label: 'Retention', tone: 'warning' },
			{ id: 'recording', label: 'Recording evidence', tone: 'success' }
		],
		semanticFacts: [
			{
				id: 'intent',
				label: 'Intent',
				value: 'Approve retention wording after legal confirmation.',
				tone: 'warning'
			},
			{
				id: 'evidence',
				label: 'Evidence path',
				value: 'recording-main + transcript segment 09:40',
				tone: 'info'
			}
		],
		suggestedActions: [
			{
				id: 'create-task',
				label: 'Create task',
				description: 'Ask legal for written confirmation.',
				icon: 'tabler:checkbox',
				tone: 'warning',
				contract: 'tasks.create_from_call_candidate'
			},
			{
				id: 'schedule-followup',
				label: 'Schedule follow-up',
				description: 'Create a follow-up meeting if legal does not reply.',
				icon: 'tabler:calendar-plus',
				tone: 'info',
				contract: 'calendar.event.create_from_call'
			},
			{
				id: 'save-summary',
				label: 'Save summary',
				description: 'Promote the cited call summary into a context pack.',
				icon: 'tabler:file-plus',
				tone: 'success',
				contract: 'context_packs.promote_call_summary'
			}
		],
		relatedContext: [
			{
				id: 'security-review',
				title: 'Vendor security review',
				description: 'Previous mail thread with matching retention clause.',
				icon: 'tabler:mail',
				tone: 'info'
			},
			{
				id: 'recording-audit',
				title: 'Recording audit trail',
				description: 'Local recording evidence and transcript rebuild history.',
				icon: 'tabler:history',
				tone: 'success'
			}
		]
	}
}

const meta = {
	title: 'Hermes App/Communications/Calls',
	component: CommunicationCallsSurface
} satisfies Meta<typeof CommunicationCallsSurface>

export default meta
type Story = StoryObj<typeof meta>

export const CallListItemStory: Story = {
	name: 'Call List Item',
	render: () => ({
		components: { CallListItem },
		data() {
			return { item: callsSurface.calls[0] }
		},
		template: `
			<section class="storybook-canvas">
				<CallListItem :item="item" />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByText('Retention review')).toBeVisible()
		await expect(canvas.getByText('Zoom')).toBeVisible()
	}
}

export const CallListStory: Story = {
	name: 'Call List',
	render: () => ({
		components: { CallList },
		data() {
			return { callsSurface }
		},
		template: `
			<section class="storybook-canvas">
				<CallList
					:provider-value="callsSurface.providerValue"
					:provider-options="callsSurface.providerOptions"
					:permanent-meetings="callsSurface.permanentMeetings"
					:calls="callsSurface.calls"
				/>
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByText('Permanent meetings')).toBeVisible()
		await expect(canvas.getByText('Today')).toBeVisible()
		await expect(canvas.getByText('Yesterday')).toBeVisible()
		await expect(canvas.getByText('Yandex Telemost')).toBeVisible()
	}
}

export const CallActionStory: Story = {
	name: 'Call Action',
	render: () => ({
		components: { CallAction },
		data() {
			return { callsSurface }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<CallAction :action-groups="callsSurface.actionGroups" />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('navigation', { name: 'Call actions' })).toBeVisible()
	}
}

export const CallViewerStory: Story = {
	name: 'Call Viewer',
	render: () => ({
		components: { CallViewer },
		data() {
			return { callsSurface }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<CallViewer :active-call="callsSurface.activeCall" />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByLabelText('Call transcript')).toBeVisible()
		await expect(canvas.getByText('retention-review-main.mp4')).toBeVisible()
	}
}

export const CallInspectorStory: Story = {
	name: 'Call Inspector',
	render: () => ({
		components: { CallInspector },
		data() {
			return { callsSurface }
		},
		template: `
			<section class="storybook-canvas">
				<CallInspector :model="callsSurface.inspector" />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByText('Call Intelligence')).toBeVisible()
		await expect(canvas.getByText('Recording linked')).toBeVisible()
	}
}

export const CallMessageStory: Story = {
	name: 'Call Message',
	render: () => ({
		components: { CallMessage },
		data() {
			return { callsSurface }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<CallMessage :surface="callsSurface" />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByText('Retention review')).toBeVisible()
		await expect(canvas.getByRole('navigation', { name: 'Call actions' })).toBeVisible()
	}
}

export const CallWorkspaceStory: Story = {
	name: 'Call Workspace',
	render: () => ({
		components: { CallWorkspace },
		data() {
			return { callsSurface }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--workspace">
				<CallWorkspace :surface="callsSurface" />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByLabelText('Calls workspace')).toBeVisible()
		await expect(canvas.getByText('Call Intelligence')).toBeVisible()
	}
}

export const CallReview: Story = {
	render: () => ({
		components: { CommunicationCallsSurface },
		data() {
			return { callsSurface }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--workspace">
				<CommunicationCallsSurface :surface="callsSurface" />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByLabelText('Calls workspace')).toBeVisible()
		await expect(canvas.getByText('Zoom')).toBeVisible()
		await expect(canvas.getByText('Yandex Telemost')).toBeVisible()
		await expect(canvas.getByText('Zulip')).toBeVisible()
		await expect(canvas.getByText('Request written confirmation')).toBeVisible()
	}
}
