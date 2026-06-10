<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';
	import HomeMetrics from './widgets/HomeMetrics.svelte';
	import HomeWhatsNew from './widgets/HomeWhatsNew.svelte';
	import HomePriorities from './widgets/HomePriorities.svelte';
	import HomeUpcoming from './widgets/HomeUpcoming.svelte';
	import HomePeopleTalked from './widgets/HomePeopleTalked.svelte';
	import HomeSystemStatus from './widgets/HomeSystemStatus.svelte';
	import HomeActiveProjects from './widgets/HomeActiveProjects.svelte';
	import { communicationMessages, mailboxHealth, communicationProjects, communicationTasks } from '$lib/stores/communications';
	import { vaultStatusError } from '$lib/stores/vault';
	import { navigateTo } from '$lib/stores/navigation';

	const _ = (key: string) => t($currentLocale, key);

	type StatCard = { label: string; value: string; delta: string; icon: string; tone?: string };
	type FeedItem = { icon: string; title: string; meta: string; time: string; tag?: string; tone?: string };
	type TaskItem = { title: string; assignee: string; due: string; priority: string };
	type PersonItem = { name: string; meta: string; icon: string };
	type ProjectItem = { name: string; kind: string; progress: number; icon: string; tone: string };

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();

	const homeStats = $derived.by(() => {
		const stats: StatCard[] = [];
			if ($mailboxHealth) {
				stats.push({ label: _('Messages'), value: String($mailboxHealth.total_messages), delta: `+${$mailboxHealth.unread}`, icon: 'tabler:mail' });
				stats.push({ label: _('Needs attention'), value: String($mailboxHealth.needs_action), delta: `+${$mailboxHealth.important}`, icon: 'tabler:alert-circle' });
				stats.push({ label: _('Waiting'), value: String($mailboxHealth.waiting), delta: `${$mailboxHealth.done} ${_('done')}`, icon: 'tabler:message-reply' });
			}
		stats.push({ label: _('Projects'), value: '—', delta: _('active'), icon: 'tabler:briefcase' });
		stats.push({ label: _('Persons'), value: '—', delta: _('enriched'), icon: 'tabler:user-plus' });
		return stats;
	});

	const whatsNew = $derived.by(() => {
		const items: FeedItem[] = [];
		const channelIcons: Record<string, string> = {
			email: 'tabler:mail',
			gmail: 'tabler:brand-gmail',
			icloud: 'tabler:cloud',
			imap: 'tabler:server',
			telegram_user: 'tabler:brand-telegram',
			telegram_bot: 'tabler:brand-telegram',
			whatsapp_web: 'tabler:brand-whatsapp'
		};
			for (const msg of $communicationMessages.slice(0, 5)) {
			const sender = msg.sender_display_name || msg.sender || _('Unknown');
			items.push({
				icon: channelIcons[msg.channel_kind] || 'tabler:message',
				title: _('New message from {sender}').replace('{sender}', sender),
				meta: msg.subject || msg.body_text_preview,
				time: msg.occurred_at || msg.projected_at,
				tone: 'blue'
			});
		}
		return items;
	});

	const peopleTalked = $derived.by(() => {
		const seen = new Set<string>();
		const result: PersonItem[] = [];
			for (const msg of $communicationMessages) {
			const sender = msg.sender_display_name || msg.sender || _('Unknown');
			if (seen.has(sender)) continue;
			seen.add(sender);
			result.push({
				name: sender,
				meta: msg.subject || msg.body_text_preview,
				icon: 'tabler:message'
			});
			if (result.length >= 5) break;
		}
		return result;
	});
</script>

<section class="home-page">
	<div class="hero-row">
		<HomeMetrics stats={homeStats} {isLayoutEditing} {isWidgetVisible} />
	</div>

	<div class="dashboard-grid">
		<HomeWhatsNew items={whatsNew} {isLayoutEditing} {isWidgetVisible} />
			<HomePriorities tasks={$communicationTasks as TaskItem[]} {isLayoutEditing} {isWidgetVisible} />
		<HomeUpcoming {isLayoutEditing} {isWidgetVisible} />

		<aside class="stacked-rail">
			<HomePeopleTalked people={peopleTalked} {isLayoutEditing} {isWidgetVisible} />
				<HomeSystemStatus statusError={$vaultStatusError} {isLayoutEditing} {isWidgetVisible} />
		</aside>
	</div>

		<HomeActiveProjects projects={$communicationProjects as ProjectItem[]} {isLayoutEditing} {isWidgetVisible} onNavigateToProjects={() => navigateTo('projects')} />
	</section>
