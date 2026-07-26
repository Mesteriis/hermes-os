import { computed, ref } from 'vue'

import type {
	AccountSummaryV1,
	CommunicationSearchHitV1,
	ConversationSummaryV1,
	MessageSummaryV1,
} from '../../../gen/hermes/communications/query/v1/query_pb'
import {
	buildCanonicalAccountRows,
	buildCanonicalConversationRows,
	buildCanonicalMessageRows,
	buildCanonicalSearchRows,
	bytesKey,
	type CanonicalCommunicationsPageModel,
	type CanonicalCommunicationsPageStatus,
	type CanonicalCommunicationsSearchStatus,
} from '../presentation/canonicalCommunicationsPageModel'
import {
	listCanonicalCommunicationAccounts,
	listCanonicalConversationMessages,
	listCanonicalConversations,
} from './canonicalCommunicationsRead'
import { searchCanonicalCommunications } from './canonicalCommunicationsSearch'

export function useCanonicalCommunicationsPage() {
	const accounts = ref<readonly AccountSummaryV1[]>([])
	const conversations = ref<readonly ConversationSummaryV1[]>([])
	const messages = ref<readonly MessageSummaryV1[]>([])
	const searchResults = ref<readonly CommunicationSearchHitV1[]>([])
	const selectedAccountKey = ref('')
	const selectedConversationKey = ref('')
	const searchText = ref('')
	const status = ref<CanonicalCommunicationsPageStatus>('loading')
	const statusMessage = ref('Loading canonical evidence…')
	const searchStatus = ref<CanonicalCommunicationsSearchStatus>('idle')
	const searchMessage = ref('Search uses exact tokens in the owner-local derived index.')
	let accountRequestGeneration = 0
	let conversationRequestGeneration = 0

	const model = computed<CanonicalCommunicationsPageModel>(() => ({
		status: status.value,
		statusMessage: statusMessage.value,
		accounts: buildCanonicalAccountRows(accounts.value, selectedAccountKey.value),
		conversations: buildCanonicalConversationRows(
			conversations.value,
			selectedConversationKey.value,
		),
		messages: buildCanonicalMessageRows(messages.value),
		searchText: searchText.value,
		searchStatus: searchStatus.value,
		searchMessage: searchMessage.value,
		searchResults: buildCanonicalSearchRows(searchResults.value),
	}))

	async function load(): Promise<void> {
		const generation = ++accountRequestGeneration
		status.value = 'loading'
		statusMessage.value = 'Loading canonical evidence…'
		try {
			const nextAccounts = await listCanonicalCommunicationAccounts()
			if (generation !== accountRequestGeneration) return
			accounts.value = nextAccounts
			if (nextAccounts.length === 0) {
				clearConversationState()
				status.value = 'empty'
				statusMessage.value = 'No canonical communication evidence has been observed yet.'
				return
			}
			await selectAccount(bytesKey(nextAccounts[0]!.accountId))
		} catch {
			if (generation !== accountRequestGeneration) return
			clearAllState()
			status.value = 'error'
			statusMessage.value = 'Canonical Communications is temporarily unavailable.'
		}
	}

	async function selectAccount(accountKey: string): Promise<void> {
		const account = accounts.value.find((candidate) => bytesKey(candidate.accountId) === accountKey)
		if (!account) return
		selectedAccountKey.value = accountKey
		clearConversationState()
		status.value = 'loading'
		statusMessage.value = 'Loading canonical conversations…'
		const generation = ++conversationRequestGeneration
		try {
			const nextConversations = await listCanonicalConversations(account.accountCursorSha256)
			if (generation !== conversationRequestGeneration) return
			conversations.value = nextConversations
			status.value = 'ready'
			statusMessage.value = nextConversations.length === 0
				? 'This source has no canonical conversations yet.'
				: ''
			if (nextConversations[0]) {
				await selectConversation(bytesKey(nextConversations[0].conversationId))
			}
		} catch {
			if (generation !== conversationRequestGeneration) return
			conversations.value = []
			messages.value = []
			status.value = 'error'
			statusMessage.value = 'Canonical conversations are temporarily unavailable.'
		}
	}

	async function selectConversation(conversationKey: string): Promise<void> {
		const conversation = conversations.value.find(
			(candidate) => bytesKey(candidate.conversationId) === conversationKey,
		)
		if (!conversation) return
		selectedConversationKey.value = conversationKey
		messages.value = []
		const generation = ++conversationRequestGeneration
		try {
			const nextMessages = await listCanonicalConversationMessages(conversation.conversationId)
			if (generation !== conversationRequestGeneration) return
			messages.value = nextMessages
			status.value = 'ready'
			statusMessage.value = nextMessages.length === 0
				? 'This conversation has no canonical messages yet.'
				: ''
		} catch {
			if (generation !== conversationRequestGeneration) return
			messages.value = []
			status.value = 'error'
			statusMessage.value = 'Canonical messages are temporarily unavailable.'
		}
	}

	async function search(): Promise<void> {
		const query = searchText.value.trim()
		if (!query) {
			searchStatus.value = 'idle'
			searchResults.value = []
			searchMessage.value = 'Enter at least one exact token.'
			return
		}
		searchStatus.value = 'loading'
		searchMessage.value = 'Searching canonical evidence…'
		try {
			searchResults.value = await searchCanonicalCommunications(query)
			searchStatus.value = 'ready'
			searchMessage.value = searchResults.value.length === 0
				? 'No canonical evidence matched those exact tokens.'
				: ''
		} catch {
			searchResults.value = []
			searchStatus.value = 'error'
			searchMessage.value = 'Canonical search is temporarily unavailable.'
		}
	}

	function updateSearchText(value: string): void {
		searchText.value = value
	}

	function clearAllState(): void {
		accounts.value = []
		selectedAccountKey.value = ''
		clearConversationState()
	}

	function clearConversationState(): void {
		conversations.value = []
		messages.value = []
		selectedConversationKey.value = ''
		conversationRequestGeneration += 1
	}

	return {
		load,
		model,
		search,
		selectAccount,
		selectConversation,
		updateSearchText,
	}
}
