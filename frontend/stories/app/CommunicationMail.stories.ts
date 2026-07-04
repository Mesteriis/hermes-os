import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, ref } from 'vue'
import { expect, userEvent, waitFor, within } from 'storybook/test'
import { setLocale, type Locale } from '@/platform/i18n'
import { Icon, Tooltip } from '@/shared/ui'
import {
	MailAction as MailActionComponent,
	MailFooter as MailFooterComponent,
	MailFolderList as MailFolderListComponent,
	MailInspector as MailInspectorComponent,
	MailList as MailListComponent,
	MailListItem as MailListItemComponent,
	MailMessage as MailMessageComponent,
	MailReplyComposer as MailReplyComposerComponent,
	MailViewer as MailViewerComponent,
	MailWorkspace as MailWorkspaceComponent,
	mailListItemAttachmentLabel,
	mailListItemCounters,
	mailListItemHasSignal,
	mailListItemMarkerClass,
	mailListItemMarkerPresentation,
	mailListItemMarkerSummary,
	mailListItemStatus,
	mailStandardFolders,
	type CommunicationConversationMessageModel,
	type CommunicationConversationModel,
	type MailFolderModel,
	type MailInspectorModel,
	type MailListItemDensity,
	type MailListItemMarker,
	type MailListItemModel
} from '@/domains/communications/components'
import { storybookLocaleFromGlobals } from '../ui/storybook-i18n'

type MailListItemStoryCase = {
  id: string
  label: string
  note: string
  item: MailListItemModel
}

type MailListItemMarkerOption = {
  marker: MailListItemMarker
}

const mailListItems: readonly MailListItemModel[] = [
	{
		id: 'mail-security-review',
		accountLabel: 'Work',
		mailboxLabel: 'Inbox',
		providerRecordId: 'gmail-msg-security-review-001',
		fromName: 'Maya Chen',
		fromAddress: 'maya@northwind.example',
		recipients: ['owner@hermes.local', 'legal@hermes.local'],
		subject: 'Vendor security review',
		snippet: 'Updated answers attached. The only open point is data retention wording.',
		sourceKind: 'mail',
		timestampLabel: '09:42',
		workflowState: 'needs_action',
		localState: 'active',
		deliveryState: 'received',
		aiCategory: 'risk',
		importanceScore: 88,
		unreadCount: 2,
		hasOpenAction: true,
		attachmentCount: 2,
		confidence: 'low',
		labels: ['security', 'work', 'vendor'],
		hermesEntities: [
			{ kind: 'organization', title: 'Northwind Security Vendor' },
			{ kind: 'decision', title: 'Retention clause approval' },
			{ kind: 'document', title: 'Security answers' }
		],
		evidenceKinds: ['mail_thread', 'attachment', 'redline'],
		taskCandidateCount: 1,
		decisionCandidateCount: 1,
		documentCandidateCount: 2,
		deadlineCount: 1,
		riskCount: 2,
		counters: [
			{ kind: 'messages', value: 5 },
			{ kind: 'insights', value: 3 },
			{ kind: 'calendar', value: 1 }
		],
		timelineHint: 'Waiting since today 09:42',
		selected: true
	},
	{
		id: 'mail-board-pack',
		accountLabel: 'Work',
		mailboxLabel: 'Finance review',
		providerRecordId: 'gmail-msg-board-pack-017',
		fromName: 'Finance review group',
		fromAddress: 'finance-review@northwind.example',
		recipients: ['owner@hermes.local', 'board@northwind.example'],
		subject: 'Board pack edits',
		snippet: 'Legal signed off on the latest version after the disclosure note.',
		sourceKind: 'mail',
		timestampLabel: 'Yesterday',
		workflowState: 'waiting',
		localState: 'active',
		deliveryState: 'received',
		aiCategory: 'finance',
		importanceScore: 61,
		attachmentCount: 1,
		confidence: 'medium',
		labels: ['finance', 'board'],
		hermesEntities: [
			{ kind: 'organization', title: 'Northwind Board' },
			{ kind: 'document', title: 'Board pack' }
		],
		evidenceKinds: ['mail_thread', 'attachment'],
		documentCandidateCount: 1,
		counters: [
			{ kind: 'messages', value: 3 },
			{ kind: 'insights', value: 1 }
		],
		timelineHint: 'Waiting since yesterday'
	},
	{
		id: 'mail-newsletter',
		accountLabel: 'Personal',
		mailboxLabel: 'Signals',
		providerRecordId: 'gmail-msg-procurement-digest-044',
		fromName: 'Weekly industry digest',
		fromAddress: 'digest@example.news',
		recipients: ['owner@hermes.local'],
		subject: 'Procurement regulation roundup',
		snippet: 'Three articles mention the same procurement regulation update.',
		sourceKind: 'mail',
		timestampLabel: 'Mon',
		workflowState: 'reviewed',
		localState: 'active',
		deliveryState: 'received',
		aiCategory: 'digest',
		importanceScore: 32,
		confidence: 'high',
		labels: ['signals', 'newsletter'],
		hermesEntities: [
			{ kind: 'document', title: 'Procurement regulation roundup' }
		],
		evidenceKinds: ['mail_thread'],
		counters: [{ kind: 'insights', value: 3 }],
		muted: true
	}
]

const mailListItemMarkerOptions: readonly MailListItemMarkerOption[] = [
	{
		marker: 'spam'
	},
	{
		marker: 'phishing'
	},
	{
		marker: 'important'
	},
	{
		marker: 'blocked'
	},
	{
		marker: 'archived'
	}
]

const mailListItemDensityOptions: readonly MailListItemDensity[] = ['compact', 'comfortable', 'cozy']

function syncAppLocaleFromStorybook(globals: Record<string, unknown>): Locale {
	const storybookLocale = storybookLocaleFromGlobals(globals)
	const locale: Locale = storybookLocale === 'ru' ? 'ru' : 'en'
	setLocale(locale)
	return locale
}

function mailStoryText(globals: Record<string, unknown>) {
	return syncAppLocaleFromStorybook(globals) === 'ru'
		? {
			analyze: 'Анализировать',
			applied: 'Применить',
			attachments: 'вложения',
			aiReply: 'ИИ-ответ',
			aiReplyVariants: 'Варианты ИИ-ответа',
			builder: 'Конструктор поиска',
			bilingualReplyFlow: 'Двуязычный ответ',
			bulkAction: 'Массовое действие',
			contains: 'содержит',
			compose: 'Написать',
			entity: 'сущность',
			expandFolder: 'Развернуть папку',
			from: 'от',
			filters: 'Фильтры',
			filterName: 'Название поискового фильтра',
			forward: 'Переслать',
			forwardEml: 'Переслать EML',
			forwardingActions: 'Пересылка',
			hermesActions: 'Действия Hermes',
			hideHermesInspector: 'Скрыть инспектор Hermes',
			emailIntelligence: 'Интеллект письма',
			extractedEntities: 'Извлечённые сущности',
			mailboxes: 'Почтовые папки',
			mailInspector: 'Инспектор письма',
			mailList: 'Список писем',
			mailView: 'Представление почты',
			mailSearchValue: 'Значение поиска по почте',
			mailAttrs: 'Атрибуты письма',
			mailFolders: 'Папки почты',
			mailListDensity: 'Плотность списка писем',
			openDestructiveActions: 'Открыть опасные действия',
			openEvidenceActions: 'Открыть действия доказательств',
			openForwardingActions: 'Открыть действия пересылки',
			openHermesActions: 'Открыть действия Hermes',
			openMessage: 'Открытое письмо',
			openOrganizationActions: 'Открыть действия организации',
			openReplyActions: 'Открыть действия ответа',
			openStateActions: 'Открыть действия состояния',
			inbox: 'Входящие',
			other: 'Прочие',
			archived: 'Архив',
			markSpam: 'Пометить как спам',
			markUnread: 'Отметить непрочитанным',
			removeLabel: 'Убрать метку',
			restoreTrash: 'Восстановить из корзины',
			redirect: 'Перенаправить',
			replyAll: 'Ответить всем',
			refresh: 'Обновить',
			saveFilter: 'Сохранить фильтр',
			savedFilters: 'Сохранённые фильтры',
			smartCc: 'Умные CC',
			showHermesInspector: 'Показать инспектор Hermes',
			suggestedActions: 'Предложенные действия',
			relatedContext: 'Связанный контекст',
			settings: 'Настройки',
			spfDkim: 'SPF/DKIM',
			updateAiState: 'Обновить AI-состояние',
			workFolder: 'Работа',
			spam: 'Спам',
			trash: 'Корзина',
			collapseFolder: 'Свернуть папку',
			compact: 'Компактно',
			comfortable: 'Удобно',
			cozy: 'Просторно',
			deleteProvider: 'Удалить у провайдера',
			status: 'статус',
			value: 'Значение условия поиска',
			vendorReviewFilter: 'Проверка поставщика'
		}
		: {
			analyze: 'Analyze',
			applied: 'Apply',
			attachments: 'attachments',
			aiReply: 'AI Reply',
			aiReplyVariants: 'AI Reply variants',
			builder: 'Search builder',
			bilingualReplyFlow: 'Bilingual reply flow',
			bulkAction: 'Bulk message action',
			contains: 'contains',
			compose: 'Compose',
			entity: 'entity',
			expandFolder: 'Expand folder',
			from: 'from',
			filters: 'Filters',
			filterName: 'Search filter name',
			forward: 'Forward',
			forwardEml: 'Forward EML',
			forwardingActions: 'Forwarding actions',
			hermesActions: 'Hermes actions',
			hideHermesInspector: 'Hide Hermes inspector',
			emailIntelligence: 'Email Intelligence',
			extractedEntities: 'Extracted entities',
			mailboxes: 'Mailboxes',
			mailInspector: 'Mail inspector',
			mailList: 'Mail list',
			mailView: 'Mail view',
			mailSearchValue: 'Mail search value',
			mailAttrs: 'Mail attrs',
			mailFolders: 'Mail folders',
			mailListDensity: 'Mail list density',
			openDestructiveActions: 'Open destructive actions',
			openEvidenceActions: 'Open evidence actions',
			openForwardingActions: 'Open forwarding actions',
			openHermesActions: 'Open Hermes actions',
			openMessage: 'Open message',
			openOrganizationActions: 'Open organization actions',
			openReplyActions: 'Open reply actions',
			openStateActions: 'Open state actions',
			inbox: 'Inbox',
			other: 'Other',
			archived: 'Archived',
			markSpam: 'Mark spam',
			markUnread: 'Mark unread',
			removeLabel: 'Remove label',
			restoreTrash: 'Restore from trash',
			redirect: 'Redirect',
			replyAll: 'Reply all',
			refresh: 'Refresh',
			saveFilter: 'Save filter',
			savedFilters: 'Saved filters',
			smartCc: 'Smart CC',
			showHermesInspector: 'Show Hermes inspector',
			suggestedActions: 'Suggested actions',
			relatedContext: 'Related context',
			settings: 'Settings',
			spfDkim: 'SPF/DKIM',
			updateAiState: 'Update AI state',
			workFolder: 'Work',
			spam: 'Spam',
			trash: 'Trash',
			collapseFolder: 'Collapse folder',
			compact: 'Compact',
			comfortable: 'Comfortable',
			cozy: 'Cozy',
			deleteProvider: 'Delete from provider',
			status: 'status',
			value: 'Builder search value',
			vendorReviewFilter: 'Vendor review'
		}
}

const mailFolderMetrics: Record<string, Pick<MailFolderModel, 'count' | 'unreadCount'>> = {
	inbox: { count: 42, unreadCount: 7 },
	sent: { count: 128 },
	drafts: { count: 4 },
	outbox: { count: 2, unreadCount: 2 },
	archive: { count: 318 },
	spam: { count: 11, unreadCount: 3 },
	trash: { count: 8 },
	all: { count: 511 }
}

const mailFolderChildrenById: Record<string, readonly MailFolderModel[]> = {
	archive: [
		{ id: 'archive-2026', kind: 'custom', label: '2026', count: 96 },
		{ id: 'archive-vendors', kind: 'custom', label: 'Vendors', count: 34 }
	],
	inbox: [
		{ id: 'inbox-work', kind: 'custom', label: 'Work', count: 24, unreadCount: 5 },
		{ id: 'inbox-personal', kind: 'custom', label: 'Personal', count: 9, unreadCount: 1 },
		{ id: 'inbox-finance', kind: 'custom', label: 'Finance', count: 6, unreadCount: 1 }
	]
}

const mailFolderItems: readonly MailFolderModel[] = mailStandardFolders.map((folder): MailFolderModel => {
	return {
		...folder,
		...mailFolderMetrics[folder.id],
		...(mailFolderChildrenById[folder.id] ? { children: mailFolderChildrenById[folder.id] } : {})
	}
})

const mailListItemDynamicFrames: readonly MailListItemStoryCase[] = [
	{
		id: 'observed',
		label: 'Observed',
		note: 'New unread mail arrives before review.',
		item: {
			...mailListItems[0],
			id: 'mail-dynamic-observed',
			workflowState: 'new',
			unreadCount: 1,
			hasOpenAction: false,
			confidence: 'medium',
			selected: false
		}
	},
	{
		id: 'needs-action',
		label: 'Needs action',
		note: 'Hermes found an owner decision inside the thread.',
		item: {
			...mailListItems[0],
			id: 'mail-dynamic-needs-action',
			workflowState: 'needs_action',
			unreadCount: 2,
			hasOpenAction: true,
			confidence: 'low',
			selected: true
		}
	},
	{
		id: 'waiting',
		label: 'Waiting',
		note: 'Reply was sent, evidence remains attached.',
		item: {
			...mailListItems[0],
			id: 'mail-dynamic-waiting',
			timestampLabel: '10:08',
			workflowState: 'waiting',
			unreadCount: undefined,
			hasOpenAction: false,
			attachmentCount: 1,
			confidence: 'medium',
			timelineHint: 'Waiting since today 10:08',
			selected: true
		}
	},
	{
		id: 'reviewed',
		label: 'Reviewed',
		note: 'The thread is no longer active but stays traceable.',
		item: {
			...mailListItems[0],
			id: 'mail-dynamic-reviewed',
			timestampLabel: '10:18',
			workflowState: 'reviewed',
			unreadCount: undefined,
			hasOpenAction: false,
			attachmentCount: 2,
			confidence: 'high',
			timelineHint: 'Reviewed today 10:18',
			selected: false
		}
	}
]

const mailListItemInvariantCases: readonly MailListItemStoryCase[] = [
	{
		id: 'selected-signal',
		label: 'Selected with signal',
		note: 'Selected state and signal edge can coexist.',
		item: mailListItems[0]
	},
	{
		id: 'single-attachment',
		label: 'Single attachment',
		note: 'Attachment copy stays singular.',
		item: mailListItems[1]
	},
	{
		id: 'muted-reviewed',
		label: 'Muted reviewed',
		note: 'Muted mail still shows mailbox and account.',
		item: mailListItems[2]
	},
	{
		id: 'spam-risk',
		label: 'Spam and risk',
		note: 'Spam, phishing and blocked markers stay explicit on the item.',
		item: {
			id: 'mail-invariant-spam-risk',
			accountLabel: 'Personal',
			mailboxLabel: 'Spam',
			fromName: 'Billing Security Desk',
			fromAddress: 'billing-alert@example-risk.test',
			subject: 'Urgent account verification required',
			snippet: 'The message is isolated as spam and kept as evidence without becoming trusted business context.',
			sourceKind: 'mail',
			timestampLabel: '08:12',
			workflowState: 'spam',
			confidence: 'low',
			counters: [{ kind: 'messages', value: 1 }],
			markers: ['spam', 'phishing', 'blocked'],
			unreadCount: 1
		}
	},
	{
		id: 'no-address',
		label: 'No address',
		note: 'Sender address is optional.',
		item: {
			id: 'mail-invariant-no-address',
			accountLabel: 'Legal',
			mailboxLabel: 'Archive',
			fromName: 'Outside counsel',
			subject: 'Signed procurement addendum',
			snippet: 'The signed addendum is stored as evidence and does not require a reply.',
			sourceKind: 'mail',
			timestampLabel: 'Fri',
			workflowState: 'reviewed',
			attachmentCount: 1,
			confidence: 'high'
		}
	},
	{
		id: 'long-copy',
		label: 'Long copy',
		note: 'Long sender, subject and snippet must not resize the row.',
		item: {
			id: 'mail-invariant-long-copy',
			accountLabel: 'Operations',
			mailboxLabel: 'Evidence review',
			fromName: 'International vendor compliance coordination group',
			fromAddress: 'very-long-compliance-distribution-list@example.internal',
			subject: 'Follow up on the procurement retention clause, delegated approval and exported report retention language',
			snippet:
				'This message intentionally carries a long preview so the list item proves it can clamp content without pushing badges or timestamps out of the stable surface.',
			sourceKind: 'mail',
			timestampLabel: 'Yesterday',
			workflowState: 'open',
			unreadCount: 12,
			attachmentCount: 3,
			confidence: 'medium',
			counters: [
				{ kind: 'messages', value: 12 },
				{ kind: 'insights', value: 4 }
			]
		}
	},
	{
		id: 'no-signal',
		label: 'Quiet inbox',
		note: 'No unread count and no action means no signal edge.',
		item: {
			id: 'mail-invariant-no-signal',
			accountLabel: 'Personal',
			mailboxLabel: 'Receipts',
			fromName: 'Travel desk',
			fromAddress: 'receipts@example.travel',
			subject: 'Receipt for Madrid train booking',
			snippet: 'Stored for evidence. No review action was detected.',
			sourceKind: 'mail',
			timestampLabel: 'Jun 29',
			workflowState: 'open',
			confidence: 'high'
		}
	}
]

const mailMessages: readonly CommunicationConversationMessageModel[] = [
	{
		id: 'mail-message-1',
		author: 'Maya Chen',
		fromLabel: 'Maya Chen <maya@northwind.example>',
		toLabel: 'Owner',
		subject: 'Vendor security review',
		body: 'I attached the updated security answers. Could you confirm whether the retention clause is acceptable before we send the final pack?',
		timestamp: 'Today, 08:55',
		direction: 'inbound',
		attachments: [
			{
				id: 'att-answers',
				name: 'security-answers.pdf',
				meta: 'PDF · 420 KB',
				icon: 'tabler:file-text',
				tone: 'info'
			}
		]
	},
	{
		id: 'mail-message-2',
		author: 'Owner',
		fromLabel: 'Owner',
		toLabel: 'Maya Chen',
		subject: 'Re: Vendor security review',
		body: 'I am okay with the answers, but let us ask them to narrow retention from 90 days to 30 days for exported reports.',
		timestamp: 'Today, 09:11',
		direction: 'outbound',
		quotedOriginal: {
			author: 'Maya Chen',
			timestamp: 'Today, 08:55',
			subject: 'Vendor security review',
			body: 'Could you confirm whether the retention clause is acceptable before we send the final pack?'
		}
	},
	{
		id: 'mail-message-3',
		author: 'Maya Chen',
		fromLabel: 'Maya Chen <maya@northwind.example>',
		toLabel: 'Owner',
		ccLabel: 'legal@hermes.local',
		bccLabel: 'audit-safe@hermes.local',
		replyToLabel: 'security-review@northwind.example',
		subject: 'Re: Vendor security review',
		body: 'That makes sense. I drafted the revised wording below and marked the clause that needs explicit approval.',
		timestamp: 'Today, 09:42',
		direction: 'inbound',
		tone: 'warning',
		attachments: [
			{
				id: 'att-redline',
				name: 'retention-clause-redline.docx',
				meta: 'Document · 88 KB',
				icon: 'tabler:file-description',
				tone: 'warning'
			}
		],
		quotedOriginal: {
			author: 'Owner',
			timestamp: 'Today, 09:11',
			subject: 'Re: Vendor security review',
			body: 'Let us ask them to narrow retention from 90 days to 30 days for exported reports.'
		}
	}
]

const mailConversation: CommunicationConversationModel = {
	id: 'mail-security-review',
	channelKind: 'mail',
	title: 'Re: Vendor security review',
	subtitle: 'Classic mail thread · 3 messages · waiting for owner decision',
	workflowState: 'needs_action',
	facts: [
		{ label: 'Attachments', value: 2 },
		{ label: 'Open action', value: 'retention clause' },
		{ label: 'Evidence', value: 'mail thread' }
	],
	messages: mailMessages,
	draftPreview: 'Approved with the 30 day retention wording. Please send the final pack after legal confirms the redline.',
	replyOriginal: {
		author: 'Maya Chen',
		timestamp: 'Today, 09:42',
		subject: 'Re: Vendor security review',
		body: 'I drafted the revised wording below and marked the clause that needs explicit approval.'
	}
}

const mailOpenMessage: CommunicationConversationMessageModel = {
	...mailMessages[2],
	bodyFormat: 'html',
	bodyHtmlSanitized: true,
	bodyHtml:
		'<p>That makes sense. I drafted the revised wording below and marked the clause that needs explicit approval.</p><p><strong>Proposed wording:</strong> exported reports are retained for 30 days unless legal approves a longer audit window.</p><ul><li>Owner approval is still required.</li><li>The redline is attached as source evidence.</li></ul>',
	labels: ['security', 'vendor', 'legal-review'],
	markers: [
		{ id: 'workflow-marker', label: 'Workflow state', value: 'needs_action', tone: 'warning' },
		{ id: 'local-marker', label: 'Local state', value: 'active', tone: 'success' },
		{ id: 'delivery-marker', label: 'Delivery state', value: 'received', tone: 'info' }
	],
	evidenceItems: [
		{ id: 'evidence-thread', label: 'Evidence', value: 'mail_thread', tone: 'info' },
		{ id: 'evidence-attachment', label: 'Evidence', value: 'attachment:redline', tone: 'warning' }
	],
	attributeGroups: [
		{
			id: 'identity',
			title: 'Message identity',
			items: [
				{ id: 'message-id', label: 'Message ID', value: 'msg_mail_security_review_003', mono: true },
				{ id: 'provider-record', label: 'Provider record', value: 'gmail-msg-security-review-001', mono: true },
				{ id: 'raw-record', label: 'Raw record', value: 'raw_mail_7f31', mono: true },
				{ id: 'observation', label: 'Observation', value: 'obs_comm_29ca', mono: true },
				{ id: 'account', label: 'Account label', value: 'Work' },
				{ id: 'conversation', label: 'Conversation', value: 'thread_vendor_security_review', mono: true },
				{ id: 'channel', label: 'Channel', value: 'mail' }
			]
		},
		{
			id: 'participants',
			title: 'Participants',
			items: [
				{ id: 'sender-display', label: 'Sender display', value: 'Maya Chen' },
				{ id: 'sender-address', label: 'Sender address', value: 'maya@northwind.example', mono: true },
				{ id: 'recipients', label: 'Recipients', value: 'owner@hermes.local, legal@hermes.local' },
				{ id: 'reply-to', label: 'Reply to', value: 'security-review@northwind.example', mono: true }
			]
		},
		{
			id: 'state',
			title: 'State and routing',
			items: [
				{ id: 'workflow', label: 'Workflow state', value: 'needs_action', tone: 'warning' },
				{ id: 'local', label: 'Local state', value: 'active', tone: 'success' },
				{ id: 'local-reason', label: 'Local state reason', value: 'owner_review' },
				{ id: 'delivery', label: 'Delivery state', value: 'received', tone: 'info' },
				{ id: 'occurred', label: 'Occurred', value: '2026-07-03 09:42' },
				{ id: 'projected', label: 'Projected', value: '2026-07-03 09:43' },
				{ id: 'importance', label: 'Importance score', value: 88, tone: 'warning' }
			]
		},
		{
			id: 'ai',
			title: 'Hermes analysis',
			items: [
				{ id: 'ai-category', label: 'AI category', value: 'risk' },
				{ id: 'ai-summary', label: 'AI summary', value: 'Owner decision needed on retention wording.' },
				{ id: 'ai-generated', label: 'Summary generated', value: '2026-07-03 09:44' },
				{ id: 'metadata', label: 'Metadata', value: 'labels=security, source=gmail, has_redline=true' }
			]
		}
	],
	actionGroups: [
		{
			id: 'message-actions',
			title: 'Message actions',
			actions: [
				{ id: 'reply', label: 'Reply', description: 'Open a reply draft with quoted original.', icon: 'tabler:corner-up-left', tone: 'accent', contract: 'workflow.reply / createDraft' },
				{ id: 'reply-all', label: 'Reply all', description: 'Open a reply-all draft with source recipients.', icon: 'tabler:corner-up-left-double', contract: 'post_v1_reply_all' },
				{ id: 'mark-read', label: 'Mark read', description: 'Mark this message as read.', icon: 'tabler:mail-opened', contract: 'markMessageRead' },
				{ id: 'mark-unread', label: 'Mark unread', description: 'Return the message to unread state.', icon: 'tabler:mail', contract: 'bulkMessageAction:mark_unread' },
				{ id: 'mark-spam', label: 'Mark spam', description: 'Move workflow state to spam.', icon: 'tabler:mail-x', tone: 'warning', contract: 'transitionMessageWorkflowState:spam' },
				{ id: 'archive', label: 'Archive', description: 'Move workflow state to archived.', icon: 'tabler:archive', contract: 'bulkMessageAction:archive' },
				{ id: 'restore-trash', label: 'Restore from trash', description: 'Restore the local message record from trash.', icon: 'tabler:restore', contract: 'restoreMessage' },
				{ id: 'bulk-actions', label: 'Bulk message action', description: 'Run a provider-neutral bulk action for this message selection.', icon: 'tabler:list-check', contract: 'bulkMessageAction' },
				{ id: 'trash', label: 'Trash', description: 'Move local state to trash.', icon: 'tabler:trash', tone: 'danger', contract: 'trashMessage' }
			]
		},
		{
			id: 'forwarding-actions',
			title: 'Forwarding actions',
			actions: [
				{ id: 'forward', label: 'Forward', description: 'Forward the message with an owner note.', icon: 'tabler:send', contract: 'post_v1_forward' },
				{ id: 'forward-eml', label: 'Forward EML', description: 'Forward the original message as EML evidence.', icon: 'tabler:file-arrow-right', contract: 'post_v1_forward_eml' },
				{ id: 'redirect', label: 'Redirect', description: 'Redirect the original message preserving sender context.', icon: 'tabler:route', contract: 'redirectMessage' }
			]
		},
		{
			id: 'organization-actions',
			title: 'Message organization',
			actions: [
				{ id: 'pin', label: 'Pin', description: 'Keep this message visible in owner context.', icon: 'tabler:pin', contract: 'toggleMessagePin' },
				{ id: 'important', label: 'Mark important', description: 'Promote the importance marker.', icon: 'tabler:star', tone: 'warning', contract: 'toggleMessageImportant' },
				{ id: 'mute', label: 'Mute', description: 'Mute low-value follow-up noise.', icon: 'tabler:bell-off', contract: 'toggleMessageMute' },
				{ id: 'snooze', label: 'Snooze', description: 'Hide until a chosen follow-up time.', icon: 'tabler:clock-pause', contract: 'snoozeMessage' },
				{ id: 'add-label', label: 'Add label', description: 'Attach a local mail label.', icon: 'tabler:tag', contract: 'addMessageLabel' },
				{ id: 'remove-label', label: 'Remove label', description: 'Detach a local mail label from this message.', icon: 'tabler:tag-off', contract: 'bulkMessageAction:remove_label' },
				{ id: 'move-folder', label: 'Move to folder', description: 'Move this message to a custom folder.', icon: 'tabler:folder-symlink', contract: 'moveMessageToFolder' },
				{ id: 'copy-folder', label: 'Copy to folder', description: 'Copy this message to another folder.', icon: 'tabler:copy', contract: 'copyMessageToFolder' },
				{ id: 'export-json', label: 'Export JSON', description: 'Export message detail with provenance.', icon: 'tabler:braces', contract: 'exportMessage:json' }
			]
		},
		{
			id: 'hermes-actions',
			title: 'Hermes actions',
			actions: [
				{ id: 'analyze', label: 'Analyze', description: 'Refresh category, summary and importance.', icon: 'tabler:sparkles', tone: 'accent', contract: 'analyzeMessage' },
				{ id: 'explain', label: 'Explain', description: 'Explain why this message needs attention.', icon: 'tabler:message-2-question', tone: 'info', contract: 'fetchMessageExplain' },
				{ id: 'smart-cc', label: 'Smart CC', description: 'Suggest recipients for the owner reply.', icon: 'tabler:users-plus', contract: 'fetchMessageSmartCc' },
				{ id: 'ai-reply', label: 'AI Reply', description: 'Generate owner-reviewed reply candidates.', icon: 'tabler:robot', tone: 'accent', contract: 'generateAiReplyVariants' },
				{ id: 'ai-reply-variants', label: 'AI Reply variants', description: 'Generate multiple tone/language reply candidates.', icon: 'tabler:messages', tone: 'accent', contract: 'generateAiReplyVariants' },
				{ id: 'bilingual-reply-flow', label: 'Bilingual reply flow', description: 'Prepare a translated reply flow with owner review.', icon: 'tabler:language-hiragana', tone: 'accent', contract: 'post_v1_bilingual_reply_flow' },
				{ id: 'update-ai-state', label: 'Update AI state', description: 'Persist an owner-reviewed AI state transition.', icon: 'tabler:brain', tone: 'accent', contract: 'updateMessageAiState' },
				{ id: 'translate', label: 'Translate', description: 'Translate body while preserving source.', icon: 'tabler:language', contract: 'translateMessage' },
				{ id: 'extract-tasks', label: 'Extract Tasks', description: 'Create task candidates, not durable tasks.', icon: 'tabler:checkbox', tone: 'warning', contract: 'extractMessageTasks' },
				{ id: 'extract-notes', label: 'Extract Notes', description: 'Create note candidates from evidence.', icon: 'tabler:notes', contract: 'extractMessageNotes' },
				{ id: 'create-task', label: 'Create task', description: 'Promote a reviewed candidate to a task.', icon: 'tabler:circle-check', tone: 'success', contract: 'runWorkflowAction:create_task' },
				{ id: 'create-document', label: 'Create document', description: 'Promote attachment context to document.', icon: 'tabler:file-plus', contract: 'runWorkflowAction:create_document' },
				{ id: 'create-event', label: 'Create event', description: 'Promote deadline signal to calendar event.', icon: 'tabler:calendar-plus', contract: 'runWorkflowAction:create_event' },
				{ id: 'create-contact', label: 'Create contact', description: 'Promote sender into a reviewed contact.', icon: 'tabler:user-plus', contract: 'runWorkflowAction:create_contact' },
				{ id: 'archive-response', label: 'Archive response', description: 'Archive after owner decision is captured.', icon: 'tabler:archive-filled', contract: 'runWorkflowAction:archive' }
			]
		},
		{
			id: 'evidence-actions',
			title: 'Evidence and safety',
			actions: [
				{ id: 'auth-check', label: 'Auth check', description: 'Inspect SPF/DKIM/DMARC style evidence.', icon: 'tabler:shield-check', contract: 'fetchMessageAuth' },
				{ id: 'spf-dkim', label: 'SPF/DKIM', description: 'Inspect provider SPF and DKIM authentication details.', icon: 'tabler:shield-half', contract: 'get_v1_spf_dkim' },
				{ id: 'signature', label: 'Signature', description: 'Detect signature and disclaimer blocks.', icon: 'tabler:signature', contract: 'fetchMessageSignature' },
				{ id: 'language', label: 'Detect language', description: 'Detect source language before translation.', icon: 'tabler:alphabet-latin', contract: 'detectMessageLanguage' },
				{ id: 'export-md', label: 'Export Markdown', description: 'Export readable markdown evidence.', icon: 'tabler:markdown', contract: 'exportMessage:md' },
				{ id: 'export-eml', label: 'Export EML', description: 'Export original mail evidence.', icon: 'tabler:file-export', contract: 'exportMessage:eml' },
				{ id: 'delete-provider', label: 'Delete from provider', description: 'Provider write, requires explicit confirmation.', icon: 'tabler:trash-x', tone: 'danger', contract: 'deleteMessageFromProvider' }
			]
		}
	],
	hermesEntities: [
		{
			id: 'entity-northwind',
			entity: 'organization',
			title: 'Northwind Security Vendor',
			description: 'Organization candidate detected from sender domain and thread context.',
			evidenceLabel: '3 messages, 2 attachments, sender domain',
			tone: 'info'
		},
		{
			id: 'entity-retention-decision',
			entity: 'decision',
			title: 'Retention clause approval',
			description: 'Decision candidate requiring owner review before reply.',
			evidenceLabel: 'Quoted reply plus redline attachment',
			tone: 'warning'
		},
		{
			id: 'entity-follow-up-task',
			entity: 'task',
			title: 'Confirm legal wording',
			description: 'Task candidate derived from current open action.',
			evidenceLabel: 'Needs action state with low confidence',
			tone: 'neutral'
		}
	]
}

const mailWorkspaceConversation: CommunicationConversationModel = {
	...mailConversation,
	messages: [mailMessages[0], mailMessages[1], mailOpenMessage]
}

const mailInspectorModel: MailInspectorModel = {
	intelligence: {
		score: 92,
		maxScore: 100,
		label: 'Email intelligence score',
		summary: 'Owner review is needed before the retention wording is approved.',
		checks: [
			{
				id: 'authentic-sender',
				label: 'Authentic sender',
				description: 'SPF, DKIM and DMARC passed.',
				tone: 'success',
				icon: 'tabler:shield-check'
			},
			{
				id: 'no-spam',
				label: 'No spam indicators',
				description: 'Clean content and headers.',
				tone: 'success',
				icon: 'tabler:circle-check'
			},
			{
				id: 'safe-attachments',
				label: 'Safe attachments',
				description: '2 files scanned, no threats.',
				tone: 'success',
				icon: 'tabler:paperclip'
			}
		]
	},
	entityGroups: [
		{
			id: 'people',
			title: 'People',
			items: [
				{
					id: 'maya-chen',
					entity: 'person',
					title: 'Maya Chen',
					description: 'maya@northwind.example',
					evidenceLabel: 'Sender, 3 thread messages',
					tone: 'info'
				}
			]
		},
		{
			id: 'organizations',
			title: 'Organizations',
			items: [
				{
					id: 'northwind-security',
					entity: 'organization',
					title: 'Northwind Security Vendor',
					description: 'Vendor in current security review',
					evidenceLabel: 'Sender domain plus redline attachment',
					tone: 'info'
				}
			]
		},
		{
			id: 'work-items',
			title: 'Hermes candidates',
			items: [
				{
					id: 'retention-decision',
					entity: 'decision',
					title: 'Retention clause approval',
					description: 'Decision candidate waiting for owner review.',
					evidenceLabel: 'Quoted reply and attachment:redline',
					tone: 'warning'
				},
				{
					id: 'legal-task',
					entity: 'task',
					title: 'Confirm legal wording',
					description: 'Task candidate, not durable truth yet.',
					evidenceLabel: 'Open action state: needs_action',
					tone: 'neutral'
				}
			]
		}
	],
	topics: [
		{ id: 'security', label: 'Security', tone: 'info' },
		{ id: 'vendor', label: 'Vendor', tone: 'neutral' },
		{ id: 'legal-review', label: 'Legal review', tone: 'warning' }
	],
	semanticFacts: [
		{ id: 'intent', label: 'Intent', value: 'Approve retention wording after legal review.', tone: 'warning' },
		{ id: 'tone', label: 'Tone', value: 'Professional, explicit approval requested.' },
		{ id: 'evidence', label: 'Evidence path', value: 'mail_thread + attachment:redline', tone: 'info' }
	],
	suggestedActions: [
		{
			id: 'create-task',
			label: 'Create task',
			description: 'Follow up with legal on retention wording.',
			icon: 'tabler:checkbox',
			tone: 'warning',
			contract: 'runWorkflowAction:create_task'
		},
		{
			id: 'ai-reply',
			label: 'Draft AI reply',
			description: 'Prepare owner-reviewed reply variants.',
			icon: 'tabler:robot',
			tone: 'accent',
			contract: 'generateAiReplyVariants'
		},
		{
			id: 'smart-cc',
			label: 'Smart CC',
			description: 'Suggest Legal as a recipient before sending.',
			icon: 'tabler:users-plus',
			tone: 'info',
			contract: 'fetchMessageSmartCc'
		},
		{
			id: 'create-document',
			label: 'Create document',
			description: 'Promote the redline attachment as reviewed evidence.',
			icon: 'tabler:file-plus',
			tone: 'neutral',
			contract: 'runWorkflowAction:create_document'
		}
	],
	relatedContext: [
		{
			id: 'organization-context',
			title: 'Northwind Security Vendor',
			description: 'Organization context and previous reviews.',
			icon: 'tabler:building',
			tone: 'info'
		},
		{
			id: 'decision-context',
			title: 'Retention clause approval',
			description: 'Decision candidate with source evidence.',
			icon: 'tabler:git-branch',
			tone: 'warning'
		},
		{
			id: 'document-context',
			title: 'retention-clause-redline.docx',
			description: 'Attachment evidence, 88 KB.',
			icon: 'tabler:file-text',
			tone: 'neutral'
		}
	]
}

const meta = {
	title: 'Hermes App/Communications/Mail',
	component: MailWorkspaceComponent
} satisfies Meta<typeof MailWorkspaceComponent>

export default meta
type Story = StoryObj<typeof meta>

export const MailListItem: Story = {
	render: () => ({
		components: { MailListItemComponent },
		data() {
			return { item: mailListItems[0] }
		},
		template: `
			<section class="storybook-canvas">
				<MailListItemComponent :item="item" />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('button', { name: /Vendor security review/ })).toBeVisible()
		await expect(canvas.getByText('Maya Chen')).toBeVisible()
		await expect(canvas.getByText('2 files')).toBeVisible()
		await expect(canvas.getByText('Updated answers attached. The only open point is data retention wording.')).toBeVisible()
	}
}

export const MailListItemDynamics: Story = {
	render: () => ({
		components: { Icon, MailListItemComponent, Tooltip },
		setup() {
			const frames = mailListItemDynamicFrames
			const activeIndex = ref(1)
			const activeDensity = ref<MailListItemDensity>('comfortable')
			const activeMarkers = ref<MailListItemMarker[]>(['spam'])
			const activeCase = computed(() => frames[activeIndex.value])
			const activeItem = computed<MailListItemModel>(() => ({
				...activeCase.value.item,
				markers: activeMarkers.value
			}))
			const activeMarkerLabels = computed(() => mailListItemMarkerSummary(activeItem.value))
			const activeCounterCount = computed(() => mailListItemCounters(activeItem.value).length)

			function setFrame(index: number): void {
				activeIndex.value = index
			}

			function setDensity(density: MailListItemDensity): void {
				activeDensity.value = density
			}

			function toggleMarker(marker: MailListItemMarker): void {
				if (activeMarkers.value.includes(marker)) {
					activeMarkers.value = activeMarkers.value.filter((activeMarker) => activeMarker !== marker)
					return
				}

				activeMarkers.value = [...activeMarkers.value, marker]
			}

			return {
				activeCase,
				activeCounterCount,
				activeDensity,
				activeIndex,
				activeItem,
				activeMarkerLabels,
				activeMarkers,
				frames,
				mailListItemDensityOptions,
				mailListItemAttachmentLabel,
				mailListItemCounters,
				mailListItemHasSignal,
				mailListItemMarkerClass,
				mailListItemMarkerOptions,
				mailListItemMarkerPresentation,
				mailListItemMarkerSummary,
				mailListItemStatus,
				setDensity,
				setFrame,
				toggleMarker
			}
		},
		template: `
			<section class="storybook-canvas">
				<section class="storybook-section storybook-mail-state-panel" aria-label="Mail list item dynamics">
					<div class="storybook-mail-density-bar" role="group" aria-label="Density">
						<button
							v-for="densityOption in mailListItemDensityOptions"
							:key="densityOption"
							type="button"
							class="storybook-mail-phase-button"
							:class="{ 'storybook-mail-phase-button--active': activeDensity === densityOption }"
							:aria-pressed="activeDensity === densityOption"
							@click="setDensity(densityOption)"
						>
							{{ densityOption }}
						</button>
					</div>
					<div class="storybook-mail-phase-bar" role="group" aria-label="Mail item lifecycle">
						<button
							v-for="(frame, index) in frames"
							:key="frame.id"
							type="button"
							class="storybook-mail-phase-button"
							:class="{ 'storybook-mail-phase-button--active': activeIndex === index }"
							:aria-pressed="activeIndex === index"
							@click="setFrame(index)"
						>
							{{ frame.label }}
						</button>
					</div>
					<div class="storybook-mail-marker-bar" role="group" aria-label="Mail markers">
						<Tooltip
							v-for="markerOption in mailListItemMarkerOptions"
							:key="markerOption.marker"
							:content="mailListItemMarkerPresentation(markerOption.marker).label"
						>
							<template #trigger>
								<button
									type="button"
									:class="[
										'storybook-mail-marker-button',
										mailListItemMarkerClass(markerOption.marker),
										activeMarkers.includes(markerOption.marker) && 'storybook-mail-marker-button--active'
									]"
									:aria-label="mailListItemMarkerPresentation(markerOption.marker).label"
									:aria-pressed="activeMarkers.includes(markerOption.marker)"
									@click="toggleMarker(markerOption.marker)"
								>
									<Icon :icon="mailListItemMarkerPresentation(markerOption.marker).icon" size="1.2rem" />
								</button>
							</template>
						</Tooltip>
					</div>
					<MailListItemComponent :item="activeItem" :density="activeDensity" />
					<div class="storybook-mail-metrics" aria-label="Current mail item contract">
						<div class="storybook-mail-metric">
							<span>Status</span>
							<strong>{{ mailListItemStatus(activeItem).label }}</strong>
						</div>
						<div class="storybook-mail-metric">
							<span>Signal</span>
							<strong>{{ mailListItemHasSignal(activeItem) ? 'on' : 'off' }}</strong>
						</div>
						<div class="storybook-mail-metric">
							<span>Unread</span>
							<strong>{{ activeItem.unreadCount ?? 0 }}</strong>
						</div>
						<div class="storybook-mail-metric">
							<span>Indicators</span>
							<strong>{{ activeCounterCount }}</strong>
						</div>
						<div class="storybook-mail-metric">
							<span>Selection</span>
							<strong>{{ activeItem.selected ? 'selected' : 'idle' }}</strong>
						</div>
						<div class="storybook-mail-metric">
							<span>Markers</span>
							<strong>{{ activeMarkerLabels }}</strong>
						</div>
					</div>
					<p class="storybook-mail-note">{{ activeCase.note }}</p>
				</section>
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('region', { name: 'Mail list item dynamics' })).toBeVisible()
		await expect(canvas.getByRole('button', { name: 'Maya Chen: Vendor security review' })).toBeVisible()
		await expect(canvas.getByRole('button', { name: 'compact' })).toBeVisible()
		await expect(canvas.getByRole('button', { name: 'Needs action' })).toBeVisible()
		await expect(canvas.getByRole('button', { name: 'Spam' })).toBeVisible()
		await expect(canvas.getByRole('button', { name: 'Phishing' })).toBeVisible()
		await expect(canvas.getByText('Signal')).toBeVisible()
	}
}

export const MailListItemDensityModes: Story = {
	render: () => ({
		components: { MailListItemComponent },
		setup() {
			return {
				densities: mailListItemDensityOptions,
				item: mailListItems[0]
			}
		},
		template: `
			<section class="storybook-canvas">
				<section class="storybook-section" aria-label="Mail list item density modes">
					<div class="storybook-mail-density-stack">
						<article v-for="densityOption in densities" :key="densityOption" class="storybook-mail-density-example">
							<h3>{{ densityOption }}</h3>
							<MailListItemComponent :item="item" :density="densityOption" />
						</article>
					</div>
				</section>
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('region', { name: 'Mail list item density modes' })).toBeVisible()
		await expect(canvas.getByText('compact')).toBeVisible()
		await expect(canvas.getByText('comfortable')).toBeVisible()
		await expect(canvas.getByText('cozy')).toBeVisible()
	}
}

export const MailListItemInvariants: Story = {
	render: () => ({
		components: { MailListItemComponent },
		setup() {
			return {
				cases: mailListItemInvariantCases,
				mailListItemAttachmentLabel,
				mailListItemHasSignal,
				mailListItemMarkerSummary,
				mailListItemStatus
			}
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<section class="storybook-section" aria-label="Mail list item invariants">
					<div class="storybook-mail-invariant-grid">
						<article v-for="invariantCase in cases" :key="invariantCase.id" class="storybook-mail-invariant-case">
							<header class="storybook-mail-invariant-header">
								<h3>{{ invariantCase.label }}</h3>
								<p>{{ invariantCase.note }}</p>
							</header>
							<MailListItemComponent :item="invariantCase.item" />
							<div class="storybook-mail-contract-list" aria-label="Invariant values">
								<span>{{ mailListItemStatus(invariantCase.item).label }}</span>
								<span>{{ mailListItemHasSignal(invariantCase.item) ? 'signal edge' : 'no signal edge' }}</span>
								<span>{{ invariantCase.item.attachmentCount ? mailListItemAttachmentLabel(invariantCase.item) : 'No files' }}</span>
								<span>{{ invariantCase.item.fromAddress ? 'address visible' : 'address optional' }}</span>
								<span>{{ mailListItemMarkerSummary(invariantCase.item) }}</span>
							</div>
						</article>
					</div>
				</section>
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('region', { name: 'Mail list item invariants' })).toBeVisible()
		await expect(canvas.getByText('Selected with signal')).toBeVisible()
		await expect(canvas.getByText('Long copy')).toBeVisible()
		await expect(canvas.getByText('Quiet inbox')).toBeVisible()
		await expect(canvas.getByText('Spam and risk')).toBeVisible()
		await expect(canvas.getAllByText('1 file')[0]).toBeVisible()
	}
}

export const MailFolders: Story = {
	render: (_args, context) => ({
		components: { MailFolderListComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			const activeFolderId = ref('inbox')

			function selectFolder(folder: MailFolderModel): void {
				activeFolderId.value = folder.id
			}

			return {
				activeFolderId,
				folders: mailFolderItems,
				selectFolder
			}
		},
		template: `
			<section class="storybook-canvas">
				<section class="storybook-section storybook-mail-folder-stage">
					<MailFolderListComponent
						:active-folder-id="activeFolderId"
						:folders="folders"
						@select="selectFolder"
					/>
				</section>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const text = mailStoryText(globals)
		await expect(canvas.getByRole('navigation', { name: text.mailFolders })).toBeVisible()
		await expect(canvas.getByRole('tree')).toBeVisible()
		const inboxTreeItem = canvas.getByRole('treeitem', { name: new RegExp(text.inbox) })
		await expect(inboxTreeItem).toHaveAttribute('aria-current', 'page')
		await expect(inboxTreeItem).toHaveAttribute('aria-expanded', 'true')
		await expect(canvas.getByRole('treeitem', { name: new RegExp(text.workFolder) })).toHaveAttribute('aria-level', '2')
		await expect(canvas.getByRole('treeitem', { name: new RegExp(text.spam) })).toBeVisible()
		await expect(canvas.getByRole('treeitem', { name: new RegExp(text.trash) })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: `${text.collapseFolder}: ${text.inbox}` }))
		await expect(canvas.getByRole('treeitem', { name: new RegExp(text.inbox) })).toHaveAttribute('aria-expanded', 'false')
		await expect(canvas.queryByRole('treeitem', { name: new RegExp(text.workFolder) })).not.toBeInTheDocument()
		await userEvent.click(canvas.getByRole('button', { name: `${text.expandFolder}: ${text.inbox}` }))
		await expect(canvas.getByRole('treeitem', { name: new RegExp(text.workFolder) })).toHaveAttribute('aria-level', '2')
		await userEvent.click(canvas.getByRole('button', { name: new RegExp(text.workFolder) }))
		await expect(canvas.getByRole('treeitem', { name: new RegExp(text.workFolder) })).toHaveAttribute('aria-current', 'page')
		await userEvent.click(canvas.getByRole('button', { name: new RegExp(text.spam) }))
		await expect(canvas.getByRole('treeitem', { name: new RegExp(text.spam) })).toHaveAttribute('aria-current', 'page')
	}
}

export const MailList: Story = {
	render: (_args, context) => ({
		components: { MailListComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			return { items: mailListItems }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<MailListComponent :items="items" />
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const body = within(canvasElement.ownerDocument.body)
		const text = mailStoryText(globals)
		await expect(canvas.getByRole('region', { name: text.mailList })).toBeVisible()
		await expect(canvas.queryByText('Email threads with attachments, quoted replies and review signals.')).not.toBeInTheDocument()
		await expect(canvas.getByRole('button', { name: text.builder })).toBeVisible()
		await expect(canvas.getByRole('button', { name: text.compose })).toBeVisible()
		await expect(canvas.getByRole('button', { name: text.refresh })).toBeVisible()
		await expect(canvas.getByRole('button', { name: text.settings })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.settings }))
		await expect(body.getByRole('menuitem', { name: text.compact })).toBeVisible()
		await expect(body.getByRole('menuitem', { name: text.comfortable })).toBeVisible()
		await expect(body.getByRole('menuitem', { name: text.cozy })).toBeVisible()
		await userEvent.keyboard('{Escape}')
		await expect(canvas.getByRole('combobox', { name: text.mailView })).toHaveTextContent(text.inbox)
		await userEvent.click(canvas.getByRole('combobox', { name: text.mailView }))
		await expect(canvas.getByRole('treeitem', { name: new RegExp(text.mailboxes) })).toBeVisible()
		await expect(canvas.getByRole('treeitem', { name: new RegExp(text.savedFilters) })).toBeVisible()
		await userEvent.keyboard('{Escape}')
		await expect(canvas.getByText('Board pack edits')).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.builder }))
		await waitFor(() => expect(body.getByRole('textbox', { name: text.value })).toBeVisible())
		await userEvent.type(body.getByRole('textbox', { name: text.value }), 'Maya')
		await userEvent.type(body.getByRole('textbox', { name: text.filterName }), text.vendorReviewFilter)
		await expect(body.getByRole('button', { name: text.from })).toBeVisible()
		await expect(body.getByRole('button', { name: text.contains })).toBeVisible()
		await expect(body.getByText(text.mailAttrs)).toBeVisible()
		await expect(body.getByRole('button', { name: text.status })).toBeVisible()
		await expect(body.getByRole('button', { name: text.attachments })).toBeVisible()
		await expect(body.getByText('Hermes')).toBeVisible()
		await expect(body.getByRole('button', { name: text.entity })).toBeVisible()
		await expect(body.getByText('Maya')).toBeVisible()
		await userEvent.click(body.getByRole('button', { name: text.saveFilter }))
		await expect(canvas.getByRole('combobox', { name: text.mailView })).toHaveTextContent(text.vendorReviewFilter)
		await userEvent.click(body.getByRole('button', { name: text.applied }))
		await expect(canvas.getByText('Maya')).toBeVisible()
		await userEvent.click(canvas.getByRole('combobox', { name: text.mailView }))
		await expect(canvas.getByRole('treeitem', { name: new RegExp(text.vendorReviewFilter) })).toBeVisible()
		await expect(canvas.getByText('Vendor security review')).toBeVisible()
		await expect(canvas.queryByText('Board pack edits')).not.toBeInTheDocument()
	}
}

export const MailAction: Story = {
	render: (_args, context) => ({
		components: { MailActionComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			const inspectorVisible = ref(true)
			return { actionGroups: mailOpenMessage.actionGroups, inspectorVisible }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<MailActionComponent
					:action-groups="actionGroups"
					:inspector-visible="inspectorVisible"
					@toggle-inspector="inspectorVisible = !inspectorVisible"
				/>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const body = within(canvasElement.ownerDocument.body)
		const text = mailStoryText(globals)
		await expect(canvas.getByRole('button', { name: text.hermesActions })).toBeVisible()
		await expect(canvas.getByRole('button', { name: text.forward })).toBeVisible()
		await expect(canvas.getByRole('button', { name: text.replyAll })).toBeVisible()
		await expect(canvas.getByRole('button', { name: text.hideHermesInspector })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.hideHermesInspector }))
		await expect(canvas.getByRole('button', { name: text.showHermesInspector })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.openReplyActions }))
		await expect(body.getByRole('menuitem', { name: text.aiReply })).toBeVisible()
		await expect(body.getByRole('menuitem', { name: text.aiReplyVariants })).toBeVisible()
		await expect(body.getByRole('menuitem', { name: text.bilingualReplyFlow })).toBeVisible()
		await expect(body.getByRole('menuitem', { name: text.smartCc })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.openForwardingActions }))
		await expect(body.getByRole('menuitem', { name: text.forwardEml })).toBeVisible()
		await expect(body.getByRole('menuitem', { name: text.redirect })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.openStateActions }))
		await expect(body.getByRole('menuitem', { name: text.markSpam })).toBeVisible()
		await expect(body.getByRole('menuitem', { name: text.markUnread })).toBeVisible()
		await expect(body.getByRole('menuitem', { name: text.restoreTrash })).toBeVisible()
		await expect(body.getByRole('menuitem', { name: text.bulkAction })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.openOrganizationActions }))
		await expect(body.getByRole('menuitem', { name: text.removeLabel })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.openHermesActions }))
		await expect(body.getByRole('menuitem', { name: text.analyze })).toBeVisible()
		await expect(body.getByRole('menuitem', { name: text.updateAiState })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.openEvidenceActions }))
		await expect(body.getByRole('menuitem', { name: text.spfDkim })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.openDestructiveActions }))
		await expect(body.getByRole('menuitem', { name: text.deleteProvider })).toBeVisible()
	}
}

export const MailViewer: Story = {
	render: (_args, context) => ({
		components: { MailViewerComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			return {
				message: mailOpenMessage,
				fallbackSubject: mailConversation.title
			}
		},
		template: `
			<section class="storybook-canvas">
				<MailViewerComponent :message="message" :fallback-subject="fallbackSubject" />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.queryByText('retention-clause-redline.docx')).toBeNull()
		await expect(canvas.getByText('Original message')).toBeVisible()
		await expect(canvas.getByText(/требует действия|needs action/)).toBeVisible()
		await expect(canvas.queryByText('needs_action')).toBeNull()
		await expect(canvas.getByRole('group', { name: /Body mode|Режим тела письма/ })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: /Show recipient details|Показать детали получателей/ }))
		await expect(canvas.getByText('legal@hermes.local')).toBeVisible()
		await expect(canvas.getByText('audit-safe@hermes.local')).toBeVisible()
		await expect(canvas.getByText('security-review@northwind.example')).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: /Hide recipient details|Скрыть детали получателей/ }))
		await expect(canvas.queryByRole('button', { name: /Message details|Детали письма/ })).toBeNull()
	}
}

export const MailFooter: Story = {
	render: () => ({
		components: { MailFooterComponent },
		template: `
			<section class="storybook-canvas">
				<MailFooterComponent />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		await expect(canvasElement.querySelector('.communication-email-footer')).not.toBeNull()
	}
}

export const MailInspector: Story = {
	render: (_args, context) => ({
		components: { MailInspectorComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			return { model: mailInspectorModel }
		},
		template: `
			<section class="storybook-canvas">
				<MailInspectorComponent :model="model" />
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const text = mailStoryText(globals)
		await expect(canvas.getByRole('complementary', { name: text.mailInspector })).toBeVisible()
		await expect(canvas.getByRole('heading', { name: text.emailIntelligence })).toBeVisible()
		await expect(canvas.getByRole('heading', { name: text.extractedEntities })).toBeVisible()
		await expect(canvas.getByRole('heading', { name: text.suggestedActions })).toBeVisible()
		await expect(canvas.getByRole('heading', { name: text.relatedContext })).toBeVisible()
		await expect(canvas.getByText('92')).toBeVisible()
		await expect(canvas.getByText('Authentic sender')).toBeVisible()
		await expect(canvas.getByText('Retention clause approval')).toBeVisible()
		await expect(canvas.getByText('Draft AI reply')).toBeVisible()
		await expect(canvas.getByText('retention-clause-redline.docx')).toBeVisible()
	}
}

export const MailMessage: Story = {
	render: (_args, context) => ({
		components: { MailMessageComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			return {
				message: mailOpenMessage,
				fallbackSubject: mailConversation.title
			}
		},
		template: `
			<section class="storybook-canvas">
				<MailMessageComponent :message="message" :fallback-subject="fallbackSubject" />
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const body = within(canvasElement.ownerDocument.body)
		const text = mailStoryText(globals)
		await expect(canvas.getByText('retention-clause-redline.docx')).toBeVisible()
		await expect(canvas.getByText('Original message')).toBeVisible()
		await expect(canvas.getByRole('button', { name: text.hermesActions })).toBeVisible()
		await expect(canvas.getByRole('button', { name: text.replyAll })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.openReplyActions }))
		await expect(body.getByRole('menuitem', { name: text.aiReply })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.openForwardingActions }))
		await expect(body.getByRole('menuitem', { name: text.forwardEml })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.openStateActions }))
		await expect(body.getByRole('menuitem', { name: text.markSpam })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.openHermesActions }))
		await expect(body.getByRole('menuitem', { name: text.analyze })).toBeVisible()
		await expect(canvasElement.querySelector('.communication-email-command-bar')).not.toBeNull()
		await expect(canvasElement.querySelector('.communication-email-preview')).not.toBeNull()
		await expect(canvasElement.querySelector('.communication-email-center')).toBeNull()
		await userEvent.click(canvas.getByRole('button', { name: text.openDestructiveActions }))
		await expect(body.getByRole('menuitem', { name: text.deleteProvider })).toBeVisible()
	}
}

export const MailReplyComposer: Story = {
	render: () => ({
		components: { MailReplyComposerComponent },
		data() {
			return {
				draftPreview: mailConversation.draftPreview,
				replyOriginal: mailConversation.replyOriginal
			}
		},
		template: `
			<section class="storybook-canvas">
				<MailReplyComposerComponent :draft-preview="draftPreview" :reply-original="replyOriginal" />
			</section>
		`
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		await expect(canvas.getByLabelText('Reply draft')).toBeVisible()
		await expect(canvas.getByText('Original message')).toBeVisible()
	}
}

export const MailWorkspace: Story = {
	render: (_args, context) => ({
		components: { MailWorkspaceComponent },
		setup() {
			syncAppLocaleFromStorybook(context.globals)
			return {
				items: mailListItems,
				conversation: mailWorkspaceConversation,
				inspector: mailInspectorModel
			}
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<MailWorkspaceComponent
					:items="items"
					:conversation="conversation"
					:inspector="inspector"
				/>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const canvas = within(canvasElement)
		const text = mailStoryText(globals)
		await expect(canvas.getByRole('region', { name: 'Mail list' })).toBeVisible()
		await expect(canvas.getByRole('region', { name: text.openMessage })).toBeVisible()
		await expect(canvas.getByRole('complementary', { name: text.mailInspector })).toBeVisible()
		await userEvent.click(canvas.getByRole('button', { name: text.hideHermesInspector }))
		await expect(canvas.queryByRole('complementary', { name: text.mailInspector })).not.toBeInTheDocument()
		await userEvent.click(canvas.getByRole('button', { name: text.showHermesInspector }))
		await expect(canvas.getByRole('complementary', { name: text.mailInspector })).toBeVisible()
	}
}
