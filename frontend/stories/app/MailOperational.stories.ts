import type { Meta, StoryObj } from '@storybook/vue3-vite'

import MailOperationalPage from '../../src/integrations/mail/presentation/MailOperationalPage.vue'
import type { MailOperationalPageModel } from '../../src/integrations/mail/presentation/mailOperationalPageModel'
import type { MailOperationalReadModel } from '../../src/integrations/mail/presentation/mailOperationalReadModel'

const meta = {
	title: 'Hermes App/Communications/Mail Operational',
	component: MailOperationalPage,
	parameters: { layout: 'fullscreen' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

const model: MailOperationalPageModel = {
	recipients: 'owner@example.com',
	subject: 'Clean-room delivery boundary',
	textBody: 'Mail owns provider delivery. Communications receives durable evidence.',
	providerConversationId: '',
	operationId: '170d768c-d956-4963-9603-2a0f578a2db4',
	busyAction: null,
	canDeliver: true,
	canSync: true,
	notice: '',
	syncSummary: '18 messages observed by sync-17.',
	status: {
		operationId: '170d768c-d956-4963-9603-2a0f578a2db4',
		connectionId: 'gmail-primary',
		outcome: 'accepted',
		requestedAt: 'Jul 26, 2026, 10:42',
		completedAt: 'Jul 26, 2026, 10:42',
		responseCode: '202',
	},
}

const readModel: MailOperationalReadModel = {
	canQuery: true,
	status: 'ready',
	statusMessage: '',
	connections: [{ id: 'gmail-primary', label: 'gmail-primary' }],
	selectedConnectionId: 'gmail-primary',
	folders: [
		{ id: 'inbox', label: 'Inbox', meta: '4 unread · 18 total', selected: true },
		{ id: 'sent', label: 'Sent', meta: '0 unread · 9 total', selected: false },
		{ id: 'archive', label: 'Archive', meta: '0 unread · 31 total', selected: false },
	],
	threads: [
		{
			id: 'thread-1',
			subject: 'Clean-room boundary',
			snippet: 'The managed route now returns bounded provider evidence.',
			meta: 'Jul 26, 2026, 10:42 · 3 messages',
			selected: true,
			unread: true,
		},
		{
			id: 'thread-2',
			subject: 'Release evidence',
			snippet: 'Managed conformance passed on the disposable host contour.',
			meta: 'Jul 26, 2026, 09:18 · 2 messages',
			selected: false,
			unread: false,
		},
	],
	messages: [
		{
			id: 'message-1',
			subject: 'Clean-room boundary',
			sender: 'owner@example.com',
			snippet: 'The managed route now returns bounded provider evidence.',
			meta: 'Jul 26, 2026, 10:42',
			selected: true,
			unread: true,
			hasAttachments: true,
		},
		{
			id: 'message-2',
			subject: 'Re: Clean-room boundary',
			sender: 'team@example.com',
			snippet: 'Confirmed. Communications remains the canonical evidence owner.',
			meta: 'Jul 26, 2026, 10:37',
			selected: false,
			unread: false,
			hasAttachments: false,
		},
	],
	detail: {
		id: 'message-1',
		subject: 'Clean-room boundary',
		sender: 'owner@example.com',
		recipients: 'team@example.com',
		snippet: 'The managed route now returns bounded provider evidence.',
		meta: 'Jul 26, 2026, 10:42 · revision 7',
		folders: 'inbox',
		flags: 'Starred',
		evidenceState: 'Canonical evidence linked',
		contentState: 'Authorized body content is Communications-owned and is not part of this Mail projection.',
	},
	hasMoreFolders: false,
	hasMoreThreads: true,
	hasMoreMessages: false,
}

export const Default: Story = {
	args: { model, readModel },
}
