<script lang="ts">
	import { onMount } from 'svelte';
	import { communicationSections } from '$lib/layout';
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
		composeForm as communicationComposeForm,
		drafts as communicationDrafts,
		handleSaveDraft as saveCommunicationDraft,
		isComposeOpen as communicationComposeOpen,
		loadCommunicationsWorkspace,
		mailboxHealth as communicationMailboxHealth,
		openComposeForDraft,
		selectedCommunication as selectedCommunicationStore
	} from '$lib/stores/communications';
	import { isLayoutEditing, visibleWidgetIds } from '$lib/stores/layoutEditor';
	import { activeCommunicationSection, currentView } from '$lib/stores/navigation';
	import { loadSettingsWorkspace } from '$lib/stores/settings';

	const notes = [
		{ title: 'Hermes Hub - Product Strategy', body: 'Основные принципы: единое пространство памяти, интеграция всех коммуникаций...', source: 'Apple Notes', tag: '#project', time: '10:42', icon: 'tabler:notes' },
		{ title: 'User Research Summary', body: 'Ключевые инсайты из интервью с пользователями...', source: 'Obsidian', tag: '#research', time: '09:15', icon: 'tabler:file-text' },
		{ title: 'Meeting with Maria - 13 May 2024', body: 'Обсудили roadmap, приоритеты и сроки запуска новых функций...', source: 'Gmail', tag: '#meeting', time: '08:27', icon: 'tabler:brand-gmail' },
		{ title: 'Quick Ideas', body: '- AI для автоматической категоризации заметок - Граф связей между проектами...', source: 'Anytype', tag: '#idea', time: '07:58', icon: 'tabler:bulb' },
		{ title: 'Integration Architecture', body: 'Схема интеграции с внешними сервисами и потоками данных...', source: 'Obsidian', tag: '#reference', time: 'May 12, 18:45', icon: 'tabler:file-text' },
		{ title: 'Email: Partnership Opportunity', body: 'Интересное предложение о партнерстве. Нужно обсудить с командой...', source: 'Outlook', tag: '#partnership', time: 'May 12, 16:20', icon: 'tabler:mail' }
	];

	const isCommunicationMessagesSection = $derived(
		$currentView === 'communications' &&
			['unified', 'inbox', 'waiting', 'needs_reply', 'mail'].includes($activeCommunicationSection)
	);
	const activeCommunicationEmptySection = $derived(
		$currentView === 'communications' &&
			['mentions', 'calls', 'meetings'].includes($activeCommunicationSection)
			? communicationSections.find((item) => item.id === $activeCommunicationSection) ?? null
			: null
	);
	const isWidgetVisible = $derived.by(() => {
		const widgetIds = $visibleWidgetIds;
		return (widgetId: string) => widgetIds.has(widgetId);
	});

	onMount(() => {
		void loadCommunicationsWorkspace();
		void loadSettingsWorkspace();
	});
</script>

<svelte:head>
	<title>Hermes Hub</title>
	<meta name="description" content="Hermes Hub desktop personal OS dashboard." />
</svelte:head>

{#if $currentView === 'home'}
	<HomePage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if isCommunicationMessagesSection}
	<CommunicationsPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if activeCommunicationEmptySection}
	<CommunicationsEmptyPage activeCommunicationEmptySection={activeCommunicationEmptySection} />
{:else if $currentView === 'persons'}
	<PersonsPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if $currentView === 'projects'}
	<ProjectsPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if $currentView === 'tasks'}
	<TasksPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if $currentView === 'calendar'}
	<CalendarPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if $currentView === 'documents'}
	<DocumentsPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if $currentView === 'notes'}
	<NotesPage {notes} isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if $currentView === 'knowledge'}
	<KnowledgePage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if $currentView === 'communications' && $activeCommunicationSection === 'telegram'}
	<TelegramPage
		isLayoutEditing={$isLayoutEditing}
		{isWidgetVisible}
		aiAnalysisResult={$communicationAiAnalysisResult}
		selectedCommunication={$selectedCommunicationStore}
	/>
{:else if $currentView === 'communications' && $activeCommunicationSection === 'whatsapp'}
	<WhatsAppPage
		isLayoutEditing={$isLayoutEditing}
		{isWidgetVisible}
		aiAnalysisResult={$communicationAiAnalysisResult}
		selectedCommunication={$selectedCommunicationStore}
	/>
{:else if $currentView === 'settings'}
	<section class="settings-page">
		<SettingsPage />
	</section>
{:else if $currentView === 'agents'}
	<AgentsPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if $currentView === 'organizations'}
	<OrganizationsPage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{:else if $currentView === 'timeline'}
	<TimelinePage isLayoutEditing={$isLayoutEditing} {isWidgetVisible} />
{/if}

<ComposeDrawer
	isOpen={$communicationComposeOpen}
	bind:form={$communicationComposeForm}
	onClose={() => communicationComposeOpen.set(false)}
	onSaveDraft={saveCommunicationDraft}
/>
<DraftStrip drafts={$communicationDrafts} onOpenCompose={(draft) => openComposeForDraft(draft as never)} />
{#if $communicationMailboxHealth}<HealthStrip health={$communicationMailboxHealth} />{/if}

<AccountSetupModal
	bind:isOpen={$isAccountDrawerOpen}
	telegramCapabilities={null}
	onAccountSaved={loadSettingsWorkspace}
/>
