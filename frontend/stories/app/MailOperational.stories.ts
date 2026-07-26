import type { Meta, StoryObj } from '@storybook/vue3-vite'

import MailOperationalPage from '../../src/integrations/mail/presentation/MailOperationalPage.vue'
import type { MailOperationalPageModel } from '../../src/integrations/mail/presentation/mailOperationalPageModel'

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

export const Default: Story = {
	args: { model },
}
