import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, ref } from 'vue'
import { expect, userEvent, within } from 'storybook/test'
import { setLocale, type Locale } from '@/platform/i18n'
import {
	MessengerAction as MessengerActionComponent,
	MessengerInspector as MessengerInspectorComponent,
	MessengerList as MessengerListComponent,
	MessengerListItem as MessengerListItemComponent,
	MessengerMessage as MessengerMessageComponent,
	MessengerViewer as MessengerViewerComponent,
	MessengerWorkspace as MessengerWorkspaceComponent,
	SignalMessengerRichEditor as SignalMessengerRichEditorComponent,
	TelegramMessengerRichEditor as TelegramMessengerRichEditorComponent,
	WhatsAppMessengerRichEditor as WhatsAppMessengerRichEditorComponent,
	type MessengerConversationModel,
	type MessengerInspectorModel,
	type MessengerListItemModel
} from '@/domains/communications/components'
import { storybookLocaleFromGlobals } from '../ui/storybook-i18n'

const messengerListItems: readonly MessengerListItemModel[] = [
	{
		id: 'telegram-owner-radar',
		channelKind: 'telegram',
		accountId: 'tg-1',
		accountLabel: 'TG 1',
		conversationKind: 'direct',
		title: 'Anna Petrova',
		subtitle: 'Radar summary before meeting',
		preview: 'Can you check the Radar summary before the meeting starts?',
		timestampLabel: '10:14',
		workflowState: 'needs_action',
		unreadCount: 3,
		mentionCount: 1,
		hermesSignalCount: 2,
		selected: true,
		labels: ['Radar', 'Decision'],
		profile: {
			displayName: 'Anna Petrova',
			fallback: 'AP',
			statusLabel: 'Shared Radar summary 4 minutes ago',
			storyItems: [
				{
					id: 'anna-radar',
					title: 'Radar summary',
					description: 'Asked for owner review before the meeting.',
					timestampLabel: '10:14',
					tone: 'accent'
				},
				{
					id: 'anna-date-conflict',
					title: 'Date conflict',
					description: 'Friday is stronger than Tuesday.',
					timestampLabel: '10:13',
					tone: 'warning'
				}
			]
		}
	},
	{
		id: 'whatsapp-family',
		channelKind: 'whatsapp',
		accountId: 'whatsapp-1',
		accountLabel: 'WhatsApp 1',
		conversationKind: 'group',
		title: 'Family logistics',
		subtitle: 'Dinner and calendar candidate',
		preview: 'Dinner moved to 19:30, calendar candidate is waiting.',
		timestampLabel: '09:58',
		workflowState: 'waiting',
		unreadCount: 1,
		attachmentCount: 1,
		profile: {
			displayName: 'Family logistics',
			fallback: 'FL',
			statusLabel: 'WhatsApp group active now',
			storyItems: [
				{
					id: 'family-dinner',
					title: 'Dinner moved',
					description: 'Calendar candidate changed to 19:30.',
					timestampLabel: '09:58',
					tone: 'success'
				},
				{
					id: 'family-reminder',
					title: 'Reminder',
					description: 'Keep the family note attached to the dialog.',
					timestampLabel: 'Today',
					tone: 'accent'
				}
			]
		}
	},
	{
		id: 'telegram-product',
		channelKind: 'telegram',
		accountId: 'tg-2',
		accountLabel: 'TG 2',
		conversationKind: 'group',
		title: 'Product backchannel',
		subtitle: 'Support escalation resolved',
		preview: 'The support escalation was resolved and archived.',
		timestampLabel: 'Yesterday',
		workflowState: 'reviewed',
		muted: true,
		profile: {
			displayName: 'Product backchannel',
			fallback: 'PB',
			statusLabel: 'Telegram group',
			storyItems: [
				{
					id: 'product-escalation',
					title: 'Escalation resolved',
					description: 'Support discussion was archived.',
					timestampLabel: 'Yesterday',
					tone: 'success'
				}
			]
		}
	},
	{
		id: 'signal-legal',
		channelKind: 'signal',
		accountId: 'signal-1',
		accountLabel: 'Signal 1',
		conversationKind: 'direct',
		title: 'Legal review',
		subtitle: 'Signal secure chat',
		preview: 'Keep the clause as pending until source evidence is attached.',
		timestampLabel: 'Mon',
		workflowState: 'muted',
		pinned: true,
		profile: {
			displayName: 'Legal review',
			fallback: 'LR',
			statusLabel: 'Signal secure chat',
			storyItems: [
				{
					id: 'legal-evidence',
					title: 'Evidence pending',
					description: 'Promotion waits for source evidence.',
					timestampLabel: 'Mon',
					tone: 'warning'
				}
			]
		}
	}
]

const messengerConversation: MessengerConversationModel = {
	id: 'telegram-owner-radar',
	channelKind: 'telegram',
	kind: 'direct',
	title: 'Anna Petrova',
	subtitle: 'Telegram direct chat',
	workflowState: 'needs_action',
	participantsLabel: '2 participants',
	lastSeenLabel: 'Active 4 minutes ago',
	facts: [
		{ label: 'Unread', value: 3, tone: 'accent' },
		{ label: 'Signals', value: 2, tone: 'warning' },
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
			meta: 'Drafted reply',
			attachments: [
				{
					id: 'radar-summary',
					name: 'radar-summary.md',
					meta: 'Evidence note',
					icon: 'tabler:file-text',
					tone: 'info'
				}
			]
		}
	],
	draftPreview: 'Friday is the safer date. I will keep Tuesday as a low confidence note until we get confirmation.'
}

const whatsAppConversation: MessengerConversationModel = {
	...messengerConversation,
	id: 'whatsapp-family',
	channelKind: 'whatsapp',
	kind: 'group',
	title: 'Family logistics',
	subtitle: 'WhatsApp group',
	workflowState: 'waiting',
	participantsLabel: '5 participants',
	lastSeenLabel: 'Active now',
	facts: [
		{ label: 'Unread', value: 1, tone: 'accent' },
		{ label: 'Provider', value: 'WhatsApp' }
	],
	messages: [
		{
			id: 'msg-whatsapp-1',
			author: 'Mila',
			body: 'Dinner moved to 19:30. Can we keep the calendar note updated?',
			timestamp: '09:58',
			direction: 'inbound',
			meta: 'WhatsApp'
		}
	],
	draftPreview: '19:30 works for me. I will keep the calendar candidate aligned.'
}

const signalConversation: MessengerConversationModel = {
	...messengerConversation,
	id: 'signal-legal',
	channelKind: 'signal',
	kind: 'direct',
	title: 'Legal review',
	subtitle: 'Signal secure chat',
	workflowState: 'needs_action',
	participantsLabel: '2 participants',
	lastSeenLabel: 'Last seen today',
	facts: [
		{ label: 'Signals', value: 1, tone: 'warning' },
		{ label: 'Provider', value: 'Signal' }
	],
	messages: [
		{
			id: 'msg-signal-1',
			author: 'Legal',
			body: 'Keep the clause pending until source evidence is attached.',
			timestamp: 'Mon',
			direction: 'inbound',
			meta: 'Signal'
		}
	],
	draftPreview: 'I will keep it pending and attach source evidence before promotion.'
}

const productBackchannelConversation: MessengerConversationModel = {
	...messengerConversation,
	id: 'telegram-product',
	channelKind: 'telegram',
	kind: 'group',
	title: 'Product backchannel',
	subtitle: 'Telegram group',
	workflowState: 'reviewed',
	participantsLabel: '8 participants',
	lastSeenLabel: 'Yesterday',
	facts: [
		{ label: 'Provider', value: 'Telegram' }
	],
	messages: [
		{
			id: 'msg-product-1',
			author: 'Support',
			body: 'The support escalation was resolved and archived.',
			timestamp: 'Yesterday',
			direction: 'inbound',
			meta: 'Telegram'
		}
	],
	draftPreview: 'Glad this is resolved. I will keep the context attached.'
}

const messengerConversationsById: Record<string, MessengerConversationModel> = {
	'telegram-owner-radar': messengerConversation,
	'whatsapp-family': whatsAppConversation,
	'telegram-product': productBackchannelConversation,
	'signal-legal': signalConversation
}

const messengerInspectorModel: MessengerInspectorModel = {
	intelligence: {
		score: 88,
		maxScore: 100,
		label: 'Conversation confidence',
		summary: 'Friday is the stronger meeting candidate, but Tuesday remains contradictory evidence.',
		checks: [
			{
				id: 'known-contact',
				label: 'Known contact',
				description: 'Anna is linked to an existing person record.',
				icon: 'tabler:user-check',
				tone: 'success'
			},
			{
				id: 'conflict',
				label: 'Conflicting dates',
				description: 'Two candidate dates need owner confirmation.',
				icon: 'tabler:alert-triangle',
				tone: 'warning'
			},
			{
				id: 'provider',
				label: 'Provider context',
				description: 'Telegram runtime data remains integration-owned.',
				icon: 'tabler:brand-telegram',
				tone: 'accent'
			}
		]
	},
	entityGroups: [
		{
			id: 'people',
			title: 'People',
			items: [
				{
					id: 'anna',
					entity: 'person',
					title: 'Anna Petrova',
					description: 'Direct chat participant',
					evidenceLabel: '3 fresh Telegram messages',
					tone: 'info'
				}
			]
		},
		{
			id: 'candidates',
			title: 'Hermes candidates',
			items: [
				{
					id: 'meeting',
					entity: 'event',
					title: 'Meeting date clarification',
					description: 'Calendar candidate with conflicting evidence.',
					evidenceLabel: 'Friday stronger than Tuesday',
					tone: 'warning'
				},
				{
					id: 'decision',
					entity: 'decision',
					title: 'Use Friday as working assumption',
					description: 'Decision candidate waiting for owner review.',
					evidenceLabel: 'Owner reply draft',
					tone: 'accent'
				}
			]
		}
	],
	suggestedActions: [
		{
			id: 'create-event',
			label: 'Create event',
			description: 'Draft calendar event using Friday as the working date.',
			icon: 'tabler:calendar-plus',
			tone: 'warning',
			contract: 'runWorkflowAction:create_event'
		},
		{
			id: 'create-task',
			label: 'Create task',
			description: 'Ask owner to resolve the Tuesday contradiction.',
			icon: 'tabler:checkbox',
			tone: 'accent',
			contract: 'runWorkflowAction:create_task'
		},
		{
			id: 'save-context',
			label: 'Save context',
			description: 'Promote the Radar summary into a context pack.',
			icon: 'tabler:package-export',
			tone: 'info',
			contract: 'createContextPackCandidate'
		}
	],
	relatedContext: [
		{
			id: 'person-context',
			title: 'Anna Petrova',
			description: 'Existing person memory and relationship notes.',
			icon: 'tabler:user',
			tone: 'info'
		},
		{
			id: 'radar-context',
			title: 'Radar meeting summary',
			description: 'Evidence-backed summary attached to the chat.',
			icon: 'tabler:radar',
			tone: 'accent'
		},
		{
			id: 'calendar-context',
			title: 'Calendar candidates',
			description: 'Friday and Tuesday slots detected from messages.',
			icon: 'tabler:calendar',
			tone: 'warning'
		}
	]
}

function syncAppLocaleFromStorybook(globals: Record<string, unknown>): Locale {
	const storybookLocale = storybookLocaleFromGlobals(globals)
	const locale: Locale = storybookLocale === 'ru' ? 'ru' : 'en'
	setLocale(locale)
	return locale
}

function messengerStoryText(globals: Record<string, unknown>) {
	return syncAppLocaleFromStorybook(globals) === 'ru'
		? {
			aiReply: 'ИИ-ответ',
			allDialogs: 'Все диалоги',
			avatarStory: 'История аватара',
			compact: 'Компактно',
			comfortable: 'Удобно',
			createEvent: 'Создать событие',
			directMessages: 'Личные диалоги',
			hideHermesInspector: 'Скрыть инспектор Hermes',
			messengerInspector: 'Инспектор диалога',
			messengerIntelligence: 'Интеллект диалога',
			messengerList: 'Список диалогов',
			messengerView: 'Представление диалогов',
			openDialog: 'Открытый диалог',
			openAvatarStory: 'Открыть историю аватара',
			openHermesActions: 'Открыть действия Hermes',
			openReplyActions: 'Открыть действия ответа',
			refresh: 'Обновить',
			reply: 'Ответить',
			safetyNumber: 'Номер безопасности',
			scheduleSend: 'Запланировать отправку',
			searchMessengers: 'Поиск по мессенджерам',
			settings: 'Настройки',
			showHermesInspector: 'Показать инспектор Hermes',
			silentSend: 'Тихая отправка',
			templateReply: 'Шаблон ответа',
			telegramReplyInput: 'Написать ответ в Telegram',
			telegramRichEditor: 'Telegram-редактор',
			whatsAppRichEditor: 'WhatsApp-редактор',
			signalRichEditor: 'Signal-редактор'
		}
		: {
			aiReply: 'AI Reply',
			allDialogs: 'All dialogs',
			avatarStory: 'Avatar story',
			compact: 'Compact',
			comfortable: 'Comfortable',
			createEvent: 'Create event',
			directMessages: 'Direct messages',
			hideHermesInspector: 'Hide Hermes inspector',
			messengerInspector: 'Messenger inspector',
			messengerIntelligence: 'Messenger Intelligence',
			messengerList: 'Messenger list',
			messengerView: 'Messenger view',
			openDialog: 'Open dialog',
			openAvatarStory: 'Open avatar story',
			openHermesActions: 'Open Hermes actions',
			openReplyActions: 'Open reply actions',
			refresh: 'Refresh',
			reply: 'Reply',
			safetyNumber: 'Safety number',
			scheduleSend: 'Schedule send',
			searchMessengers: 'Search messengers',
			settings: 'Settings',
			showHermesInspector: 'Show Hermes inspector',
			silentSend: 'Silent send',
			templateReply: 'Template reply',
			telegramReplyInput: 'Write a Telegram reply',
			telegramRichEditor: 'Telegram rich editor',
			whatsAppRichEditor: 'WhatsApp rich editor',
			signalRichEditor: 'Signal rich editor'
		}
}

const meta = {
	title: 'Hermes App/Communications/Messengers',
	component: MessengerWorkspaceComponent
} satisfies Meta<typeof MessengerWorkspaceComponent>

export default meta
type Story = StoryObj<typeof meta>

export const MessengerListItem: Story = {
	render: (_args, context) => ({
		components: { MessengerListItemComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			return { item: messengerListItems[0] }
		},
		template: `
			<section class="storybook-canvas">
				<MessengerListItemComponent :item="item" />
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const body = within(canvasElement.ownerDocument.body)
		const text = messengerStoryText(globals)
		await expect(canvas.getByRole('button', { name: /Telegram, Anna Petrova/ })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: `${text.openAvatarStory}: Anna Petrova` }))
		await expect(body.getByText('Radar summary')).toBeVisible()
		await expect(canvas.getByText('Radar summary before meeting')).toBeVisible()
	}
}

export const MessengerList: Story = {
	render: (_args, context) => ({
		components: { MessengerListComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			return { items: messengerListItems }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<MessengerListComponent :items="items" />
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const body = within(canvasElement.ownerDocument.body)
		const text = messengerStoryText(globals)
		await expect(canvas.getByRole('region', { name: text.messengerList })).toBeVisible()
		const search = canvas.getByRole('searchbox', { name: text.searchMessengers })
		await expect(search).toBeVisible()
		await userEvent.type(search, 'WhatsApp')
		await expect(canvas.getByText('Family logistics')).toBeVisible()
		await expect(canvas.queryByText('Anna Petrova')).not.toBeInTheDocument()
		await userEvent.click(canvas.getByRole('button', { name: `${text.openAvatarStory}: Family logistics` }))
		await expect(body.getByText('Dinner moved')).toBeVisible()
		await userEvent.keyboard('{Escape}')
		await userEvent.clear(search)
		await expect(canvas.getByRole('button', { name: text.refresh })).toBeVisible()
		await expect(canvas.getByRole('button', { name: text.settings })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.settings }))
		await expect(body.getByRole('menuitem', { name: text.compact })).toBeVisible()
		await expect(body.getByRole('menuitem', { name: text.comfortable })).toBeVisible()
		await userEvent.keyboard('{Escape}')
		await expect(canvas.getByRole('combobox', { name: text.messengerView })).toHaveTextContent(text.allDialogs)
		await userEvent.click(canvas.getByRole('combobox', { name: text.messengerView }))
		await expect(canvas.getByRole('treeitem', { name: 'Telegram' })).toBeVisible()
		await userEvent.click(canvas.getByRole('treeitem', { name: 'Telegram' }))
		await expect(canvas.getByRole('treeitem', { name: /TG 1/ })).toBeVisible()
		await expect(canvas.getByRole('treeitem', { name: /TG 2/ })).toBeVisible()
	}
}

export const MessengerAction: Story = {
	render: (_args, context) => ({
		components: { MessengerActionComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			const inspectorVisible = ref(true)
			return { inspectorVisible }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<MessengerActionComponent
					:inspector-visible="inspectorVisible"
					@toggle-inspector="inspectorVisible = !inspectorVisible"
				/>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const body = within(canvasElement.ownerDocument.body)
		const text = messengerStoryText(globals)
		await expect(canvas.getByRole('button', { name: text.hideHermesInspector })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.hideHermesInspector }))
		await expect(canvas.getByRole('button', { name: text.showHermesInspector })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.openReplyActions }))
		await expect(body.getByRole('menuitem', { name: text.aiReply })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.openHermesActions }))
		await expect(body.getByRole('menuitem', { name: text.createEvent })).toBeVisible()
	}
}

export const MessengerViewer: Story = {
	render: (_args, context) => ({
		components: { MessengerViewerComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			return { conversation: messengerConversation }
		},
		template: `
			<section class="storybook-canvas">
				<MessengerViewerComponent :conversation="conversation" />
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const text = messengerStoryText(globals)
		await expect(canvas.getByRole('region', { name: text.openDialog })).toBeVisible()
		await expect(canvas.getByRole('heading', { name: 'Anna Petrova', level: 2 })).toBeVisible()
		await expect(canvas.getByText('I found two conflicting dates in the thread. The Friday slot has stronger evidence.')).toBeVisible()
		await expect(canvas.getByRole('textbox', { name: text.telegramReplyInput })).toBeVisible()
		await expect(canvas.getByRole('button', { name: text.silentSend })).toBeVisible()
		await expect(canvas.getByRole('button', { name: text.scheduleSend })).toBeVisible()
	}
}

export const TelegramRichEditor: Story = {
	render: (_args, context) => ({
		components: { TelegramMessengerRichEditorComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			const draft = ref('<p>Friday is safer.</p>')
			return { conversation: messengerConversation, draft }
		},
		template: `
			<section class="storybook-canvas">
				<TelegramMessengerRichEditorComponent
					v-model="draft"
					:conversation="conversation"
				/>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const text = messengerStoryText(globals)
		await expect(canvas.getByRole('button', { name: text.silentSend })).toBeVisible()
		await expect(canvas.getByRole('button', { name: text.scheduleSend })).toBeVisible()
		await expect(canvas.queryByRole('button', { name: text.templateReply })).not.toBeInTheDocument()
	}
}

export const WhatsAppRichEditor: Story = {
	render: (_args, context) => ({
		components: { WhatsAppMessengerRichEditorComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			const draft = ref('<p>19:30 works.</p>')
			return { conversation: whatsAppConversation, draft }
		},
		template: `
			<section class="storybook-canvas">
				<WhatsAppMessengerRichEditorComponent
					v-model="draft"
					:conversation="conversation"
				/>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const text = messengerStoryText(globals)
		await expect(canvas.getByRole('button', { name: text.templateReply })).toBeVisible()
		await expect(canvas.queryByRole('button', { name: text.scheduleSend })).not.toBeInTheDocument()
	}
}

export const SignalRichEditor: Story = {
	render: (_args, context) => ({
		components: { SignalMessengerRichEditorComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			const draft = ref('<p>Evidence stays pending.</p>')
			return { conversation: signalConversation, draft }
		},
		template: `
			<section class="storybook-canvas">
				<SignalMessengerRichEditorComponent
					v-model="draft"
					:conversation="conversation"
				/>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const text = messengerStoryText(globals)
		await expect(canvas.getByRole('button', { name: text.safetyNumber })).toBeVisible()
		await expect(canvas.queryByRole('button', { name: text.scheduleSend })).not.toBeInTheDocument()
	}
}

export const MessengerInspector: Story = {
	render: (_args, context) => ({
		components: { MessengerInspectorComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			return { model: messengerInspectorModel }
		},
		template: `
			<section class="storybook-canvas">
				<MessengerInspectorComponent :model="model" />
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const text = messengerStoryText(globals)
		await expect(canvas.getByRole('complementary', { name: text.messengerInspector })).toBeVisible()
		await expect(canvas.getByRole('heading', { name: text.messengerIntelligence })).toBeVisible()
		await expect(canvas.getByText('88')).toBeVisible()
		await expect(canvas.getByText('Meeting date clarification')).toBeVisible()
		await expect(canvas.getByText('Create event')).toBeVisible()
	}
}

export const MessengerMessage: Story = {
	render: (_args, context) => ({
		components: { MessengerMessageComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			return { conversation: messengerConversation }
		},
		template: `
			<section class="storybook-canvas">
				<MessengerMessageComponent :conversation="conversation" />
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const text = messengerStoryText(globals)
		await expect(canvas.getByRole('button', { name: text.openReplyActions })).toBeVisible()
		await expect(canvas.getByRole('region', { name: text.openDialog })).toBeVisible()
		await expect(canvasElement.querySelector('.messenger-action-bar')).not.toBeNull()
		await expect(canvasElement.querySelector('.messenger-viewer')).not.toBeNull()
	}
}

export const MessengerWorkspace: Story = {
	render: (_args, context) => ({
		components: { MessengerWorkspaceComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			const activeConversationId = ref(messengerConversation.id)
			const activeConversation = computed(() =>
				messengerConversationsById[activeConversationId.value] ?? messengerConversation
			)
			const activeItems = computed(() => messengerListItems.map((item) => ({
				...item,
				selected: item.id === activeConversationId.value
			})))

			return {
				activeConversationId,
				items: activeItems,
				conversation: activeConversation,
				inspector: messengerInspectorModel
			}
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide storybook-canvas--workspace">
				<MessengerWorkspaceComponent
					:items="items"
					:conversation="conversation"
					:inspector="inspector"
					@select-conversation="activeConversationId = $event.id"
				/>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const text = messengerStoryText(globals)
		await expect(canvas.getByRole('region', { name: text.messengerList })).toBeVisible()
		await expect(canvas.getByRole('region', { name: text.openDialog })).toBeVisible()
		await expect(canvas.getByRole('complementary', { name: text.messengerInspector })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: /WhatsApp, Family logistics/ }))
		await expect(canvas.getByRole('heading', { name: 'Family logistics', level: 2 })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.hideHermesInspector }))
		await expect(canvas.queryByRole('complementary', { name: text.messengerInspector })).not.toBeInTheDocument()
	}
}

export const DirectChats: Story = MessengerWorkspace
