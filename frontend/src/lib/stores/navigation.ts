import { writable, derived } from 'svelte/store';
import {
	communicationSectionViewId,
	type PrimaryNavId,
	type CommunicationSectionId,
	type SidebarViewId
} from '$lib/layout/sidebar-navigation';

export type AppViewId = PrimaryNavId | 'organizations' | 'settings';

export const currentView = writable<AppViewId>('home');
export const activeCommunicationSection = writable<CommunicationSectionId>('unified');
export const activeSidebarRailGroupId = writable<string | null>(null);
export const isSidebarRail = writable(false);
export const isUserMenuOpen = writable(false);
export const expandedSidebarGroupIds = writable<string[]>(['communications']);

export function navigateTo(viewId: AppViewId): void {
	currentView.set(viewId);
	activeSidebarRailGroupId.set(null);
}

export function navigateToCommunicationSection(sectionId: CommunicationSectionId): void {
	currentView.set('communications');
	activeCommunicationSection.set(sectionId);
	activeSidebarRailGroupId.set(null);
}

export function toggleUserMenu(): void {
	isUserMenuOpen.update((v) => !v);
}

export function closeUserMenu(): void {
	isUserMenuOpen.set(false);
}

type ViewCopy = Record<SidebarViewId, { title: string; subtitle: string; search: string; icon: string }>;

const viewCopy: ViewCopy = {
	home: {
		title: 'Good evening, Alex',
		subtitle: "Here's what's happening in your world today.",
		search: 'Search anything...',
		icon: 'tabler:home'
	},
	communications: {
		title: 'Communications',
		subtitle: 'All your conversations. All channels. One place.',
		search: 'Search in communications...',
		icon: 'tabler:messages'
	},
	timeline: {
		title: 'Timeline',
		subtitle: 'Chronological activity across messages, tasks, documents and meetings.',
		search: 'Search timeline...',
		icon: 'tabler:timeline-event'
	},
	persons: {
		title: 'Persons',
		subtitle: '642 persons',
		search: 'Search persons, companies, emails...',
		icon: 'tabler:address-book'
	},
	projects: {
		title: 'Hermes Hub',
		subtitle: 'Product Development',
		search: 'Search projects, documents, people...',
		icon: 'tabler:cube'
	},
	tasks: {
		title: 'Tasks',
		subtitle: 'All your tasks from connected trackers',
		search: 'Search tasks, projects, trackers, people...',
		icon: 'tabler:checkbox'
	},
	calendar: {
		title: 'Calendar',
		subtitle: 'All your events from connected calendars',
		search: 'Search events, meetings, persons...',
		icon: 'tabler:calendar'
	},
	documents: {
		title: 'Documents',
		subtitle: 'All your documents from connected sources',
		search: 'Search documents, folders, content...',
		icon: 'tabler:file-text'
	},
	notes: {
		title: 'Notes',
		subtitle: 'All your notes from connected sources',
		search: 'Search notes, content, emails...',
		icon: 'tabler:notes'
	},
	knowledge: {
		title: 'Knowledge Graph',
		subtitle: 'Explore relationships across people, projects, documents, messages and tasks.',
		search: 'Search anything in your knowledge graph...',
		icon: 'tabler:share'
	},
	review: {
		title: 'Review',
		subtitle: 'Source-backed suggested changes across relationships, decisions, obligations and contradictions.',
		search: 'Search review queue...',
		icon: 'tabler:clipboard-check'
	},
	telegram: {
		title: 'Telegram Client',
		subtitle: 'Telegram messaging, policy automation and call intelligence.',
		search: 'Search Telegram chats, policies, calls...',
		icon: 'tabler:brand-telegram'
	},
	whatsapp: {
		title: 'WhatsApp Web',
		subtitle: 'WhatsApp companion sessions, fixture ingestion and live-runtime guardrails.',
		search: 'Search WhatsApp sessions and messages...',
		icon: 'tabler:brand-whatsapp'
	},
	agents: {
		title: 'AI Agents',
		subtitle: 'Your intelligent assistants working across your data and tools',
		search: 'Search agents, capabilities, tasks...',
		icon: 'tabler:sparkles'
	},
	organizations: {
		title: 'Companies',
		subtitle: 'All companies and organizations from your communications',
		search: 'Search companies, industries, locations...',
		icon: 'tabler:building'
	},
	settings: {
		title: 'Settings',
		subtitle: 'Runtime settings and connected accounts.',
		search: 'Search settings and accounts...',
		icon: 'tabler:settings'
	}
};

export const activeWorkspaceView = derived(
	[currentView, activeCommunicationSection],
	([$currentView, $activeCommunicationSection]): SidebarViewId =>
		$currentView === 'communications'
			? communicationSectionViewId($activeCommunicationSection)
			: $currentView
);

export const activeView = derived(activeWorkspaceView, ($activeWorkspaceView) => {
	return viewCopy[$activeWorkspaceView] ?? viewCopy.home;
});

export const shellViewClass = derived(activeWorkspaceView, ($activeWorkspaceView) =>
	`view-${$activeWorkspaceView}`
);
