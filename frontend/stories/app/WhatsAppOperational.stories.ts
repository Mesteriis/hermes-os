import type { Meta, StoryObj } from '@storybook/vue3-vite'

import WhatsAppOperationalPage from '../../src/integrations/whatsapp/presentation/WhatsAppOperationalPage.vue'
import type { WhatsAppOperationalPageModel } from '../../src/integrations/whatsapp/presentation/whatsAppOperationalPageModel'

const meta = {
	title: 'Hermes App/Communications/WhatsApp Operational',
	component: WhatsAppOperationalPage,
	parameters: { layout: 'fullscreen' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

const model: WhatsAppOperationalPageModel = {
	accountId: 'whatsapp-owner-primary',
	providerChatId: '34600000000@c.us',
	draft: 'The clean-room command boundary is ready.',
	operationId: '6a80b72f-618a-4bfa-a88e-623f88d99f98',
	busy: false,
	canSend: true,
	notice: '',
	status: {
		operationId: '6a80b72f-618a-4bfa-a88e-623f88d99f98',
		accountId: 'whatsapp-owner-primary',
		state: 'completed',
		requestedAt: 'Jul 26, 2026, 10:42',
		completedAt: 'Jul 26, 2026, 10:42',
	},
}

export const Default: Story = {
	args: { model },
}
