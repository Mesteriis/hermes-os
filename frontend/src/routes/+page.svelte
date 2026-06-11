<script lang="ts">
	import { onMount } from 'svelte';
	import { currentLocale, t } from '$lib/i18n';
	import { communicationSections } from '$lib/layout';

	const _ = (key: string) => t($currentLocale, key);
	import AccountSetupModal from '$lib/components/shared/AccountSetupModal.svelte';
	import ComposeDrawer from '$lib/components/shared/ComposeDrawer.svelte';
	import DraftStrip from '$lib/components/shared/DraftStrip.svelte';
	import HealthStrip from '$lib/components/shared/HealthStrip.svelte';
	import AgentsPage from '$lib/pages/agents/AgentsPage.svelte';
	import CalendarPage from '$lib/pages/calendar/CalendarPage.svelte';
	import CommunicationsEmptyPage from '$lib/pages/communications/CommunicationsEmptyPage.svelte';
	import CommunicationsPage from '$lib/pages/communications/CommunicationsPage.svelte';
	import DocumentsPage from '$lib/pages/documents/DocumentsPage.svelte';
	import HomePage from '$lib/pages/home/HomePage.svelte';
	import KnowledgePage from '$lib/pages/knowledge/KnowledgePage.svelte';
	import NotesPage from '$lib/pages/notes/NotesPage.svelte';
	import OrganizationsPage from '$lib/pages/organizations/OrganizationsPage.svelte';
	import PersonsPage from '$lib/pages/persons/PersonsPage.svelte';
	import ProjectsPage from '$lib/pages/projects/ProjectsPage.svelte';
	import SettingsPage from '$lib/pages/settings/SettingsPage.svelte';
	import TasksPage from '$lib/pages/tasks/TasksPage.svelte';
	import TelegramPage from '$lib/pages/telegram/TelegramPage.svelte';
	import TimelinePage from '$lib/pages/timeline/TimelinePage.svelte';
	import WhatsAppPage from '$lib/pages/whatsapp/WhatsAppPage.svelte';
	import { isAccountDrawerOpen } from '$lib/stores/accountWizard';
	import {
		aiAnalysisResult as communicationAiAnalysisResult,
		closeSendReview,
		composeForm as communicationComposeForm,
		composeSendError,
		composeStatusMessage,
		confirmSendMessage,
		drafts as communicationDrafts,
		handleSaveDraft as saveCommunicationDraft,
		isComposeOpen as communicationComposeOpen,
		isSendReviewOpen,
		isSendingMessage,
		loadCommunicationsWorkspace,
		mailAccountOptions,
		mailboxHealth as communicationMailboxHealth,
		openSendReview,
		openComposeForDraft,
		selectedCommunication as selectedCommunicationStore
	} from '$lib/stores/communications';
	import { isLayoutEditing, visibleWidgetIds } from '$lib/stores/layoutEditor';
	import { activeCommunicationSection, currentView, navigateTo } from '$lib/stores/navigation';
	import {
		loadSettingsWorkspace,
		selectedSettingsSection,
		settingsActionMessage
	} from '$lib/stores/settings';
	import {
		initializeUiStatePersistence,
		restoreComposeFromPersistedUiState,
		restoreUiStateFromBackendSettings
	} from '$lib/stores/uiState';
	import { apiBaseUrl } from '$lib/config';
	import {
		isGmailOAuthConnectedSearch,
		isTrustedGmailOAuthConnectedMessage,
		removeHermesOAuthSearch
	} from '$lib/services/oauth-callback';

	const notes = [
		{ title: 'Hermes Hub - Product Strategy', body: 'Основные принципы: единое пространство памяти, интеграция всех коммуникаций...', source: 'Apple Notes', tag: '#project', time: '10:42', icon: 'tabler:notes' },
		{ title: 'User Research Summary', body: 'Ключевые инсайты из интервью с пользователями...', source: 'Obsidian', tag: '#research', time: '09:15', icon: 'tabler:file-text' },
		{ title: 'Meeting with Maria - 13 May 2024', body: 'Обсудили roadmap, приоритеты и сроки запуска новых функций...', source: 'Gmail', tag: '#meeting', time: '08:27', icon: 'tabler:brand-gmail' },
		{ title: 'Quick Ideas', body: '- AI для автоматической категоризации заметок - Граф связей между проектами...', source: 'Anytype', tag: '#idea', time: '07:58', icon: 'tabler:bulb' },
		{ title: 'Integration Architecture', body: 'Схема интеграции с внешними сервисами и потоками данных...', source: 'Obsidian', tag: '#reference', time: 'May 12, 18:45', icon: 'tabler:file-text' },
		{ title: 'Email: Partnership Opportunity', body: 'Интересное предложение о партнерстве. Нужно обсудить с командой...', source: 'Outlook', tag: '#partnership', time: 'May 12, 16:20', icon: 'tabler:mail' }
	];

	const currentViewId = $derived($currentView);
	const activeCommunicationSectionId = $derived($activeCommunicationSection);
	const isCommunicationMessagesSection = $derived(
		currentViewId === 'communications' &&
			['unified', 'inbox', 'waiting', 'needs_reply', 'mail'].includes(activeCommunicationSectionId)
	);
	const activeCommunicationEmptySection = $derived(
		currentViewId === 'communications' &&
			['mentions', 'calls', 'meetings'].includes(activeCommunicationSectionId)
			? communicationSections.find((item) => item.id === activeCommunicationSectionId) ?? null
			: null
	);
	const isWidgetVisible = $derived.by(() => {
		const widgetIds = $visibleWidgetIds;
		return (widgetId: string) => widgetIds.has(widgetId);
	});

	onMount(() => {
		initializeUiStatePersistence();
		void (async () => {
			await loadSettingsWorkspace();
			restoreUiStateFromBackendSettings();
			await loadCommunicationsWorkspace();
			await restoreComposeFromPersistedUiState();
		})();

		function showConnectedAccounts() {
			navigateTo('settings');
			selectedSettingsSection.set('integrations');
			settingsActionMessage.set('Google mail connected');
			void loadSettingsWorkspace();
		}

		if (isGmailOAuthConnectedSearch(window.location.search)) {
			showConnectedAccounts();
			window.history.replaceState(null, '', removeHermesOAuthSearch(new URL(window.location.href)));
		}

		function handleOAuthMessage(event: MessageEvent) {
			if (isTrustedGmailOAuthConnectedMessage(event, apiBaseUrl)) {
				showConnectedAccounts();
			}
		}

		window.addEventListener('message', handleOAuthMessage);
		return () => window.removeEventListener('message', handleOAuthMessage);
	});
</script>

<svelte:head>
	<title>{_('Hermes Hub')}</title>
	<meta name="description" content={_('Hermes Hub desktop personal OS dashboard.')} />
</svelte:head>

{#if currentViewId === 'home'}
	<HomePage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if isCommunicationMessagesSection}
	<CommunicationsPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if activeCommunicationEmptySection}
	<CommunicationsEmptyPage activeCommunicationEmptySection={activeCommunicationEmptySection} />
{:else if currentViewId === 'persons'}
	<PersonsPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if currentViewId === 'projects'}
	<ProjectsPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if currentViewId === 'tasks'}
	<TasksPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if currentViewId === 'calendar'}
	<CalendarPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if currentViewId === 'documents'}
	<DocumentsPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if currentViewId === 'notes'}
	<NotesPage {notes} isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if currentViewId === 'knowledge'}
	<KnowledgePage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if currentViewId === 'communications' && activeCommunicationSectionId === 'telegram'}
	<TelegramPage
		isLayoutEditing={$isLayoutEditing}
		{isWidgetVisible}
		aiAnalysisResult={$communicationAiAnalysisResult}
		selectedCommunication={$selectedCommunicationStore}
	/>
{:else if currentViewId === 'communications' && activeCommunicationSectionId === 'whatsapp'}
	<WhatsAppPage
		isLayoutEditing={$isLayoutEditing}
		{isWidgetVisible}
		aiAnalysisResult={$communicationAiAnalysisResult}
		selectedCommunication={$selectedCommunicationStore}
	/>
{:else if currentViewId === 'settings'}
	<section class="settings-page">
		<SettingsPage />
	</section>
{:else if currentViewId === 'agents'}
	<AgentsPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if currentViewId === 'organizations'}
	<OrganizationsPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if currentViewId === 'timeline'}
	<TimelinePage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{/if}

<ComposeDrawer
	isOpen={$communicationComposeOpen}
	bind:form={$communicationComposeForm}
	accountOptions={$mailAccountOptions}
	isSending={$isSendingMessage}
	sendError={$composeSendError}
	statusMessage={$composeStatusMessage}
	isSendReviewOpen={$isSendReviewOpen}
	onClose={() => communicationComposeOpen.set(false)}
	onSaveDraft={saveCommunicationDraft}
	onOpenSendReview={openSendReview}
	onCloseSendReview={closeSendReview}
	onConfirmSend={confirmSendMessage}
/>
<DraftStrip drafts={$communicationDrafts} onOpenCompose={(draft) => openComposeForDraft(draft as never)} />
{#if $communicationMailboxHealth}<HealthStrip health={$communicationMailboxHealth} />{/if}

<AccountSetupModal
	bind:isOpen={$isAccountDrawerOpen}
	telegramCapabilities={null}
	onAccountSaved={loadSettingsWorkspace}
/>
