import type { Meta, StoryObj } from '@storybook/vue3-vite'

import TelegramOperationalPage from '../../src/integrations/telegram/presentation/TelegramOperationalPage.vue'
import type { TelegramOperationalPageModel } from '../../src/integrations/telegram/presentation/telegramOperationalPageModel'

const meta = {
	title: 'Hermes App/Communications/Telegram Operational',
	component: TelegramOperationalPage,
	parameters: { layout: 'fullscreen' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

const model: TelegramOperationalPageModel = {
	accountId: 'telegram-owner-primary',
	status: 'ready',
	statusMessage: '',
	chats: [
		{ id: 'chat-architecture', title: 'Hermes Architecture', detail: '@hermes_arch · supergroup', selected: true },
		{ id: 'chat-operations', title: 'Operations', detail: 'private group', selected: false },
	],
	messages: [
		{ id: 'message-1', sender: 'Alex', body: 'Provider boundary is admitted independently.', meta: 'Jul 26, 2026, 10:42 · received', outgoing: false },
		{ id: 'message-2', sender: 'You', body: 'Great. The domain only sees neutral evidence.', meta: 'Jul 26, 2026, 10:44 · delivered', outgoing: true },
	],
	selectedChatId: 'chat-architecture',
	selectedChatTitle: 'Hermes Architecture',
	draft: 'Ship the clean-room provider surface.',
	sendPending: false,
	sendMessage: 'Accepted means queued; provider completion remains asynchronous.',
	canSend: true,
}

export const Default: Story = {
	args: { model },
}
