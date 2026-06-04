<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import {
		completeGmailOAuthSetup,
		fetchV1Status,
		setupImapAccount,
		startGmailOAuthSetup,
		type GmailOAuthStartResponse,
		type V1Status
	} from '$lib/api';

	type Provider = 'gmail' | 'icloud' | 'imap';
	type NavAction = 'account-setup';

	type NavItem = {
		label: string;
		icon: string;
		badge?: string;
		active?: boolean;
		enabled: boolean;
		action?: NavAction;
	};

	type TimelineItem = {
		time: string;
		title: string;
		description: string;
		meta: string;
		icon: string;
		tag?: string;
		tagTone?: 'amber' | 'cyan' | 'purple';
		details?: string;
	};

	type ProjectItem = {
		name: string;
		kind: string;
		progress: number;
		tasks: string;
		members: string;
		icon: string;
		tone: 'amber' | 'cyan' | 'purple' | 'mint';
	};

	const apiBaseUrl = import.meta.env.VITE_HERMES_API_BASE_URL ?? 'http://127.0.0.1:8080';
	const apiToken = import.meta.env.VITE_HERMES_LOCAL_API_TOKEN ?? 'change-me-local-api-token';
	const actorId = import.meta.env.VITE_HERMES_ACTOR_ID ?? 'desktop-shell';

	let status = $state<V1Status | null>(null);
	let errorMessage = $state('');
	let isLoading = $state(true);
	let selectedProvider = $state<Provider>('gmail');
	let setupMessage = $state('');
	let setupError = $state('');
	let isSetupSubmitting = $state(false);
	let isAccountDrawerOpen = $state(false);
	let gmailPending = $state<GmailOAuthStartResponse | null>(null);
	let gmailAuthorizationCode = $state('');
	let searchQuery = $state('');
	let selectedTimelineFilter = $state('All Events');
	let gmailForm = $state({
		account_id: 'gmail-primary',
		display_name: 'Primary Gmail',
		external_account_id: '',
		client_id: '',
		client_secret: '',
		redirect_uri: `${apiBaseUrl.replace(/\/+$/, '')}/api/v1/email-accounts/gmail/oauth/callback`
	});
	let imapForm = $state({
		account_id: 'icloud-primary',
		display_name: 'Primary iCloud',
		external_account_id: '',
		host: 'imap.mail.me.com',
		port: 993,
		tls: true,
		mailbox: 'INBOX',
		username: '',
		password: '',
		secret_kind: 'app_password' as 'app_password' | 'password'
	});

	const primaryNav: NavItem[] = [
		{ label: 'Home', icon: 'tabler:home', active: true, enabled: true },
		{ label: 'Timeline', icon: 'tabler:timeline-event', enabled: false },
		{ label: 'Communications', icon: 'tabler:messages', badge: '23', enabled: false },
		{ label: 'Contacts', icon: 'tabler:address-book', enabled: false },
		{ label: 'Projects', icon: 'tabler:briefcase', enabled: false },
		{ label: 'Tasks', icon: 'tabler:checkbox', enabled: false },
		{ label: 'Calendar', icon: 'tabler:calendar', enabled: false },
		{ label: 'Documents', icon: 'tabler:file-text', enabled: false },
		{ label: 'Notes', icon: 'tabler:notes', enabled: false },
		{ label: 'Knowledge Graph', icon: 'tabler:share', enabled: false },
		{ label: 'AI Agents', icon: 'tabler:sparkles', enabled: false }
	];

	const shortcutNav: NavItem[] = [
		{ label: 'Inbox', icon: 'tabler:inbox', badge: '12', enabled: false },
		{ label: 'Starred', icon: 'tabler:star', enabled: false },
		{ label: 'Today', icon: 'tabler:calendar-time', enabled: false },
		{ label: 'Waiting', icon: 'tabler:clock-hour-4', enabled: false },
		{ label: 'Someday', icon: 'tabler:calendar-week', enabled: false },
		{ label: 'Trash', icon: 'tabler:trash', enabled: false }
	];

	const metricCards = [
		{ label: 'Events', value: '1,247', delta: '12%', icon: 'tabler:chart-bar', tone: 'chart' },
		{ label: 'Tasks', value: '23', delta: '5', icon: 'tabler:circle-check', tone: 'check' },
		{ label: 'Projects', value: '17', delta: '2', icon: 'tabler:folder', tone: 'folder' },
		{ label: 'Contacts', value: '642', delta: '18', icon: 'tabler:users', tone: 'contacts' }
	];

	const timelineItems: TimelineItem[] = [
		{
			time: '18:42',
			title: 'Re: Project Hermes Update',
			description: 'John Carter -> You. Thanks for the update. Please prepare the final report...',
			meta: 'Email',
			icon: 'tabler:mail',
			tag: 'Important',
			tagTone: 'amber'
		},
		{
			time: '17:30',
			title: 'Telegram - @design_discussion',
			description: "Maria Petrova: I've reviewed the new mockups. Looks great!",
			meta: 'Telegram',
			icon: 'tabler:brand-telegram',
			tag: 'Work',
			tagTone: 'cyan'
		},
		{
			time: '16:15',
			title: 'Document Uploaded',
			description: 'Q2_Financial_Report.pdf',
			meta: 'Documents',
			icon: 'tabler:file-description',
			tag: 'Document',
			tagTone: 'amber',
			details: '2.4 MB · OCR completed'
		},
		{
			time: '15:09',
			title: 'Task Created',
			description: 'Prepare presentation for Acme Corp',
			meta: 'Tasks',
			icon: 'tabler:checkbox',
			details: 'Due Tomorrow · John Carter'
		},
		{
			time: '14:20',
			title: 'Meeting Completed',
			description: 'Project Hermes - Weekly Sync',
			meta: 'Calendar',
			icon: 'tabler:calendar-check',
			details: '45m · 3 participants'
		},
		{
			time: '21:17',
			title: 'WhatsApp - Business',
			description: 'Acme Corp: Could you send the latest contract?',
			meta: 'WhatsApp',
			icon: 'tabler:brand-whatsapp'
		},
		{
			time: '19:45',
			title: 'Note Created',
			description: 'Ideas for Q3 Marketing Campaign',
			meta: 'Notes',
			icon: 'tabler:note',
			tagTone: 'purple'
		}
	];

	const graphNodes = [
		{ label: 'John Carter', caption: 'Person', icon: 'tabler:user', x: 47, y: 12 },
		{ label: 'Acme Corp', caption: 'Organization', icon: 'tabler:building-bank', x: 82, y: 28 },
		{ label: 'Project Plan', caption: 'Document', icon: 'tabler:file-text', x: 88, y: 54 },
		{ label: 'Maria Petrova', caption: 'Person', icon: 'tabler:user', x: 74, y: 78 },
		{ label: 'Budget', caption: 'Document', icon: 'tabler:file-text', x: 18, y: 70 },
		{ label: 'Q2 Report', caption: 'Document', icon: 'tabler:file-text', x: 12, y: 38 }
	];

	const discoveredItems = [
		{ label: 'VAT Discussion', source: 'From Email', icon: 'tabler:mail' },
		{ label: 'Acme Contract', source: 'From WhatsApp', icon: 'tabler:brand-whatsapp' },
		{ label: 'Budget 2024', source: 'From Document', icon: 'tabler:file-text' }
	];

	const projects: ProjectItem[] = [
		{
			name: 'Hermes Hub',
			kind: 'Product Development',
			progress: 75,
			tasks: '23 tasks',
			members: '8 members',
			icon: 'tabler:hexagon-letter-h',
			tone: 'amber'
		},
		{
			name: 'Acme Integration',
			kind: 'Client Project',
			progress: 45,
			tasks: '12 tasks',
			members: '3 members',
			icon: 'tabler:cube',
			tone: 'cyan'
		},
		{
			name: 'Q3 Marketing Campaign',
			kind: 'Marketing',
			progress: 60,
			tasks: '17 tasks',
			members: '4 members',
			icon: 'tabler:hexagon-3d',
			tone: 'purple'
		},
		{
			name: 'Personal Finance',
			kind: 'Personal Project',
			progress: 30,
			tasks: '8 tasks',
			members: '1 member',
			icon: 'tabler:cash',
			tone: 'mint'
		}
	];

	const calendarDays = [
		{ day: 'Mon', date: '12', active: true },
		{ day: 'Tue', date: '13' },
		{ day: 'Wed', date: '14' },
		{ day: 'Thu', date: '15' },
		{ day: 'Fri', date: '16' },
		{ day: 'Sat', date: '17.' },
		{ day: 'Sun', date: '18' }
	];

	const calendarItems = [
		{ time: '10:00', title: 'Project Hermes - Weekly Sync', duration: '1h', badge: '+3' },
		{ time: '14:00', title: 'Call with Acme Corp', duration: '1h' },
		{ time: '16:30', title: 'Review Q2 Report', duration: '30m' }
	];

	const taskItems = [
		{ title: 'Prepare report for Acme Corp', due: 'Today', tag: 'Work' },
		{ title: 'Review Q2 financials', due: 'Jun 2', tag: 'Work' },
		{ title: 'Call with John Carter', due: 'Jun 3', tag: 'Personal' },
		{ title: 'Update project roadmap', due: 'Jun 5', tag: 'Work' },
		{ title: 'Book flights to Berlin', due: 'Jun 5', tag: 'Personal' }
	];

	const topicItems = [
		{ label: 'Hermes Project', value: 128 },
		{ label: 'Acme Corp', value: 96 },
		{ label: 'Q2 Report', value: 64 },
		{ label: 'Budget', value: 48 },
		{ label: 'VAT', value: 32 }
	];

	onMount(async () => {
		try {
			status = await fetchV1Status(apiBaseUrl, apiToken, actorId);
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : 'Unknown status error';
		} finally {
			isLoading = false;
		}
	});

	function openAccountDrawer() {
		isAccountDrawerOpen = true;
		setupMessage = '';
		setupError = '';
	}

	function closeAccountDrawer() {
		isAccountDrawerOpen = false;
	}

	function selectProvider(provider: Provider) {
		selectedProvider = provider;
		setupMessage = '';
		setupError = '';

		if (provider === 'icloud') {
			imapForm = {
				...imapForm,
				account_id: imapForm.account_id || 'icloud-primary',
				display_name: imapForm.display_name || 'Primary iCloud',
				host: 'imap.mail.me.com',
				port: 993,
				tls: true,
				mailbox: imapForm.mailbox || 'INBOX',
				secret_kind: 'app_password'
			};
		}
		if (provider === 'imap') {
			imapForm = {
				...imapForm,
				account_id: imapForm.account_id === 'icloud-primary' ? 'imap-primary' : imapForm.account_id,
				display_name:
					imapForm.display_name === 'Primary iCloud' ? 'Primary IMAP' : imapForm.display_name,
				host: imapForm.host === 'imap.mail.me.com' ? '' : imapForm.host,
				secret_kind: 'password'
			};
		}
	}

	function handleNav(item: NavItem) {
		if (!item.enabled) {
			return;
		}
		if (item.action === 'account-setup') {
			openAccountDrawer();
		}
	}

	function visibleTimelineItems() {
		const query = searchQuery.trim().toLowerCase();
		if (!query) {
			return timelineItems;
		}

		return timelineItems.filter((item) =>
			[item.title, item.description, item.meta, item.tag ?? '', item.details ?? '']
				.join(' ')
				.toLowerCase()
				.includes(query)
		);
	}

	async function startGmailSetup() {
		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';

		try {
			gmailPending = await startGmailOAuthSetup(apiBaseUrl, apiToken, actorId, {
				account_id: gmailForm.account_id,
				display_name: gmailForm.display_name,
				external_account_id: gmailForm.external_account_id,
				client_id: gmailForm.client_id,
				client_secret: gmailForm.client_secret || undefined,
				redirect_uri: gmailForm.redirect_uri
			});
			setupMessage = 'Gmail OAuth grant started';
		} catch (error) {
			setupError = error instanceof Error ? error.message : 'Gmail setup failed';
		} finally {
			isSetupSubmitting = false;
		}
	}

	async function completeGmailSetup() {
		if (!gmailPending) {
			setupError = 'Gmail OAuth grant has not been started';
			return;
		}

		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';

		try {
			const result = await completeGmailOAuthSetup(apiBaseUrl, apiToken, actorId, {
				setup_id: gmailPending.setup_id,
				state: gmailPending.state,
				authorization_code: gmailAuthorizationCode
			});
			setupMessage = `Gmail account ${result.account_id} saved`;
			gmailAuthorizationCode = '';
			gmailPending = null;
		} catch (error) {
			setupError = error instanceof Error ? error.message : 'Gmail setup failed';
		} finally {
			isSetupSubmitting = false;
		}
	}

	async function saveImapAccount() {
		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';

		try {
			const result = await setupImapAccount(apiBaseUrl, apiToken, actorId, {
				account_id: imapForm.account_id,
				provider_kind: selectedProvider === 'icloud' ? 'icloud' : 'imap',
				display_name: imapForm.display_name,
				external_account_id: imapForm.external_account_id,
				host: imapForm.host,
				port: Number(imapForm.port),
				tls: imapForm.tls,
				mailbox: imapForm.mailbox,
				username: imapForm.username,
				password: imapForm.password,
				secret_kind: imapForm.secret_kind
			});
			setupMessage = `Mail account ${result.account_id} saved`;
			imapForm = { ...imapForm, password: '' };
		} catch (error) {
			setupError = error instanceof Error ? error.message : 'Mail account setup failed';
		} finally {
			isSetupSubmitting = false;
		}
	}
</script>

<svelte:head>
	<title>Hermes Hub</title>
	<meta name="description" content="Hermes Hub desktop personal OS dashboard." />
</svelte:head>

<main class="desktop-shell">
	<aside class="sidebar" aria-label="Hermes Hub navigation">
		<div class="brand">
			<img src="/assets/hermes-logo-mark.png" alt="" class="brand-mark" />
			<div>
				<p class="brand-name">Hermes Hub</p>
				<p class="brand-subtitle">Personal OS</p>
			</div>
		</div>

		<nav class="nav-group" aria-label="Primary">
			{#each primaryNav as item}
				<button
					type="button"
					class:active={item.active}
					class:disabled={!item.enabled}
					disabled={!item.enabled}
					title={item.enabled ? item.label : `${item.label} is not available in the current desktop scope`}
					onclick={() => handleNav(item)}
				>
					<Icon icon={item.icon} width="18" height="18" />
					<span>{item.label}</span>
					{#if item.badge}
						<em>{item.badge}</em>
					{/if}
				</button>
			{/each}
		</nav>

		<div class="nav-separator"></div>

		<section class="shortcuts" aria-label="Shortcuts">
			<p>Shortcuts</p>
			<nav class="nav-group">
				{#each shortcutNav as item}
					<button
						type="button"
						class:disabled={!item.enabled}
						disabled={!item.enabled}
						title={`${item.label} is not available in the current desktop scope`}
					>
						<Icon icon={item.icon} width="18" height="18" />
						<span>{item.label}</span>
						{#if item.badge}
							<em>{item.badge}</em>
						{/if}
					</button>
				{/each}
			</nav>
		</section>

		<div class="profile-card">
			<img src="/assets/hermes-reference-avatar.png" alt="Alex Morgan" />
			<div>
				<strong>Alex Morgan</strong>
				<span>Focus Mode</span>
			</div>
			<Icon icon="tabler:chevron-down" width="16" height="16" />
		</div>

		<div class="sidebar-tools" aria-label="Settings shortcuts">
			<button type="button" disabled title="Settings are not available yet">
				<Icon icon="tabler:settings" width="18" height="18" />
			</button>
			<button type="button" disabled title="Help is not available yet">
				<Icon icon="tabler:help-circle" width="18" height="18" />
			</button>
			<button type="button" disabled title="Apps are not available yet">
				<Icon icon="tabler:layout-grid" width="18" height="18" />
			</button>
		</div>
	</aside>

	<section class="workspace" aria-label="Hermes Hub dashboard">
		<header class="topbar">
			<label class="search-box">
				<Icon icon="tabler:search" width="18" height="18" />
				<input bind:value={searchQuery} placeholder="Search anything..." aria-label="Search timeline" />
				<span class="kbd">⌘ K</span>
			</label>
		</header>

		<section class="hero-row" aria-labelledby="dashboard-heading">
			<div class="greeting">
				<div class="hero-mark">
					<img src="/assets/hermes-logo-mark.png" alt="" />
				</div>
				<div>
					<h1 id="dashboard-heading">Good evening, Alex</h1>
					<p>Here's what happened today</p>
				</div>
			</div>

			<div class="metric-grid" aria-label="Daily metrics">
				{#each metricCards as metric}
					<article class="metric-card {metric.tone}">
						<span>{metric.label}</span>
						<div>
							<strong>{metric.value}</strong>
							<Icon icon={metric.icon} width="28" height="28" />
						</div>
						<small>↑ {metric.delta}</small>
					</article>
				{/each}
				<article class="metric-card focus">
					<span>Focus Score</span>
					<div class="score-ring">
						<strong>78</strong>
					</div>
					<small>Good</small>
				</article>
			</div>
		</section>

		<div class="main-grid">
			<section class="panel timeline-panel" aria-labelledby="timeline-heading">
				<header class="panel-header">
					<div class="panel-tabs" role="tablist" aria-label="Timeline views">
						<button type="button" class="active" role="tab" aria-selected="true" id="timeline-heading">
							Timeline
						</button>
						<button type="button" role="tab" disabled title="Highlights are not implemented yet">
							Highlights
						</button>
						<button type="button" role="tab" disabled title="My Day is not implemented yet">My Day</button>
					</div>
					<button type="button" class="icon-button" disabled title="Share is not implemented yet">
						<Icon icon="tabler:share-3" width="17" height="17" />
					</button>
				</header>

				<div class="timeline-toolbar">
					<span class="today-dot">Today</span>
					<label>
						<select bind:value={selectedTimelineFilter} aria-label="Timeline filter">
							<option>All Events</option>
							<option>Email</option>
							<option>Documents</option>
							<option>Tasks</option>
						</select>
					</label>
					<button type="button" class="icon-button" disabled title="Timeline settings are not implemented yet">
						<Icon icon="tabler:adjustments-horizontal" width="17" height="17" />
					</button>
				</div>

				<div class="timeline-list">
					{#each visibleTimelineItems().slice(0, 5) as item, index}
						<article class="timeline-item">
							<div class="time">{item.time}</div>
							<div class="rail">
								<span class="rail-dot"></span>
							</div>
							<div class="event-icon tone-{index % 4}">
								<Icon icon={item.icon} width="20" height="20" />
							</div>
							<div class="event-body">
								<header>
									<strong>{item.title}</strong>
									<span>{item.meta}</span>
								</header>
								<p>{item.description}</p>
								{#if item.details}
									<small>{item.details}</small>
								{/if}
								{#if item.tag}
									<em class="tag {item.tagTone ?? 'cyan'}">{item.tag}</em>
								{/if}
							</div>
						</article>
					{/each}

					<div class="yesterday-label">Yesterday</div>

					{#each visibleTimelineItems().slice(5) as item, index}
						<article class="timeline-item muted">
							<div class="time">{item.time}</div>
							<div class="rail">
								<span class="rail-dot"></span>
							</div>
							<div class="event-icon tone-{index + 2}">
								<Icon icon={item.icon} width="20" height="20" />
							</div>
							<div class="event-body">
								<header>
									<strong>{item.title}</strong>
									<span>{item.meta}</span>
								</header>
								<p>{item.description}</p>
							</div>
						</article>
					{/each}
				</div>

				<button type="button" class="load-button" disabled>Load more events</button>
			</section>

			<div class="center-column">
				<section class="panel graph-panel" aria-labelledby="graph-heading">
					<header class="panel-title-row">
						<h2 id="graph-heading">Knowledge Graph</h2>
						<div>
							<button type="button" class="ghost-button" disabled>Explore Graph</button>
							<button type="button" class="icon-button" disabled title="Graph fullscreen is not implemented yet">
								<Icon icon="tabler:arrows-maximize" width="17" height="17" />
							</button>
						</div>
					</header>

					<div class="graph-canvas" aria-label="Knowledge graph preview">
						<div class="graph-line one"></div>
						<div class="graph-line two"></div>
						<div class="graph-line three"></div>
						<div class="graph-line four"></div>
						<div class="graph-core">
							<Icon icon="tabler:cube" width="32" height="32" />
							<span>Hermes Project</span>
						</div>
						{#each graphNodes as node}
							<div class="graph-node" style={`left: ${node.x}%; top: ${node.y}%;`}>
								<span>
									<Icon icon={node.icon} width="18" height="18" />
								</span>
								<strong>{node.label}</strong>
								<small>{node.caption}</small>
							</div>
						{/each}
					</div>

					<div class="discovered">
						<p>Recently Discovered</p>
						<div>
							{#each discoveredItems as item}
								<button type="button" disabled>
									<Icon icon={item.icon} width="18" height="18" />
									<span>
										<strong>{item.label}</strong>
										<small>{item.source}</small>
									</span>
								</button>
							{/each}
						</div>
					</div>
				</section>

				<section class="panel projects-panel" aria-labelledby="projects-heading">
					<header class="panel-title-row">
						<h2 id="projects-heading">Active Projects</h2>
						<button type="button" class="link-button" disabled>View all projects →</button>
					</header>

					<div class="project-list">
						{#each projects as project}
							<article class="project-row">
								<div class="project-icon {project.tone}">
									<Icon icon={project.icon} width="20" height="20" />
								</div>
								<div class="project-main">
									<strong>{project.name}</strong>
									<span>{project.kind}</span>
								</div>
								<div class="progress" aria-label={`${project.progress}%`}>
									<span style={`width: ${project.progress}%`}></span>
								</div>
								<strong class="progress-value">{project.progress}%</strong>
								<div class="project-meta">
									<span>{project.tasks}</span>
									<span>{project.members}</span>
								</div>
							</article>
						{/each}
					</div>
				</section>
			</div>
		</div>

		<footer class="quick-command" aria-label="Quick command">
			<Icon icon="tabler:command" width="20" height="20" />
			<input bind:value={searchQuery} placeholder="Quick command..." aria-label="Quick command" />
			<span class="kbd">⌘ K</span>
			<button type="button" disabled title="Notes are not implemented yet">
				<Icon icon="tabler:note" width="16" height="16" />
				New Note
			</button>
			<button type="button" disabled title="Tasks are not implemented yet">
				<Icon icon="tabler:checkbox" width="16" height="16" />
				New Task
			</button>
			<button type="button" disabled title="Calendar events are not implemented yet">
				<Icon icon="tabler:calendar-plus" width="16" height="16" />
				New Event
			</button>
			<button type="button" onclick={openAccountDrawer}>
				<Icon icon="tabler:mail-plus" width="16" height="16" />
				Add Account
			</button>
		</footer>
	</section>

	<aside class="right-rail" aria-label="Dashboard side panels">
		<header class="rail-actions">
			<button type="button" disabled title="Command palette is not implemented yet">
				<Icon icon="tabler:terminal-2" width="16" height="16" />
				Command Palette
				<span class="kbd">⌘ P</span>
			</button>
			<button type="button" class="icon-button" disabled title="Notifications are not implemented yet">
				<Icon icon="tabler:bell" width="18" height="18" />
			</button>
			<button type="button" class="avatar-button" onclick={openAccountDrawer} title="Open account setup">
				<img src="/assets/hermes-logo-mark.png" alt="" />
			</button>
		</header>

		<section class="panel calendar-card" aria-labelledby="calendar-heading">
			<header class="panel-title-row">
				<h2 id="calendar-heading">Calendar</h2>
				<button type="button" class="link-button" disabled>View full calendar ›</button>
			</header>
			<h3>May 12 - 18, 2024</h3>
			<div class="calendar-strip">
				{#each calendarDays as day}
					<button type="button" class:active={day.active} disabled={!day.active}>
						<span>{day.day}</span>
						<strong>{day.date}</strong>
					</button>
				{/each}
			</div>
			<div class="agenda">
				{#each calendarItems as item}
					<article>
						<div>
							<strong>{item.time}</strong>
							<span>{item.duration}</span>
						</div>
						<p>{item.title}</p>
						{#if item.badge}
							<em>{item.badge}</em>
						{/if}
					</article>
				{/each}
			</div>
		</section>

		<section class="panel tasks-card" aria-labelledby="tasks-heading">
			<header class="panel-title-row">
				<h2 id="tasks-heading">Tasks</h2>
				<button type="button" class="link-button" disabled>View all tasks ›</button>
			</header>
			<div class="small-tabs">
				<button type="button" class="active">My Tasks</button>
				<button type="button" disabled>Today <span>7</span></button>
				<button type="button" disabled>Upcoming</button>
				<button type="button" disabled>Waiting</button>
			</div>
			<div class="task-list">
				{#each taskItems as item}
					<label>
						<input type="checkbox" disabled />
						<span>{item.title}</span>
						<em>{item.due}</em>
						<strong>{item.tag}</strong>
					</label>
				{/each}
			</div>
		</section>

		<section class="panel insights-card" aria-labelledby="insights-heading">
			<header class="panel-title-row">
				<h2 id="insights-heading">Insights</h2>
				<button type="button" class="link-button" disabled>View all insights ›</button>
			</header>
			<div class="insight-grid">
				<article>
					<strong>Activity Summary</strong>
					<span>Last 7 days</span>
					<div class="bar-chart" aria-label="Activity chart">
						<i style="height: 36%"></i>
						<i style="height: 58%"></i>
						<i style="height: 48%"></i>
						<i style="height: 78%"></i>
						<i style="height: 52%"></i>
						<i style="height: 64%"></i>
						<i style="height: 92%"></i>
						<i style="height: 72%"></i>
					</div>
				</article>
				<article>
					<strong>Top Topics</strong>
					<ul>
						{#each topicItems as topic}
							<li><span>{topic.label}</span><em>{topic.value}</em></li>
						{/each}
					</ul>
				</article>
			</div>
		</section>

		<section class="panel assistant-card" aria-labelledby="assistant-heading">
			<header class="panel-title-row">
				<h2 id="assistant-heading">AI Assistant</h2>
				<button type="button" class="link-button" disabled>
					<Icon icon="tabler:message-plus" width="15" height="15" />
					New Chat
				</button>
			</header>
			<label>
				<input placeholder="Ask HESTIA anything..." disabled />
				<Icon icon="tabler:sparkles" width="18" height="18" />
			</label>
			<div>
				<button type="button" disabled>Summarize last 7 days</button>
				<button type="button" disabled>What did I promise?</button>
				<button type="button" disabled>Show project updates</button>
			</div>
		</section>

		<section class="panel system-card" aria-labelledby="system-heading">
			<div>
				<h2 id="system-heading">System Status</h2>
				<p class:online={status} class:error={errorMessage}>
					{#if status}
						All systems operational
					{:else if errorMessage}
						Backend unavailable
					{:else if isLoading}
						Checking local API
					{/if}
				</p>
			</div>
			<div class="sparkline" aria-hidden="true">
				<span></span>
			</div>
		</section>
	</aside>
</main>

{#if isAccountDrawerOpen}
	<button
		type="button"
		class="drawer-backdrop"
		aria-label="Close account setup"
		onclick={closeAccountDrawer}
	></button>
	<aside class="account-drawer" aria-labelledby="account-setup-heading">
		<header>
			<div>
				<p>Provider Accounts</p>
				<h2 id="account-setup-heading">Add Account</h2>
			</div>
			<button type="button" class="icon-button" onclick={closeAccountDrawer} aria-label="Close">
				<Icon icon="tabler:x" width="18" height="18" />
			</button>
		</header>

		<div class="provider-tabs" aria-label="Account provider">
			<button
				type="button"
				class:active={selectedProvider === 'gmail'}
				onclick={() => selectProvider('gmail')}>Gmail</button
			>
			<button
				type="button"
				class:active={selectedProvider === 'icloud'}
				onclick={() => selectProvider('icloud')}>iCloud</button
			>
			<button
				type="button"
				class:active={selectedProvider === 'imap'}
				onclick={() => selectProvider('imap')}>Raw IMAP</button
			>
		</div>

		{#if selectedProvider === 'gmail'}
			<form class="setup-form" onsubmit={(event) => event.preventDefault()}>
				<label>
					<span>Account ID</span>
					<input bind:value={gmailForm.account_id} autocomplete="off" />
				</label>
				<label>
					<span>Display name</span>
					<input bind:value={gmailForm.display_name} autocomplete="off" />
				</label>
				<label>
					<span>Gmail address</span>
					<input bind:value={gmailForm.external_account_id} autocomplete="email" />
				</label>
				<label>
					<span>OAuth client ID</span>
					<input bind:value={gmailForm.client_id} autocomplete="off" />
				</label>
				<label>
					<span>OAuth client secret</span>
					<input bind:value={gmailForm.client_secret} type="password" autocomplete="off" />
				</label>
				<label class="wide">
					<span>Redirect URI</span>
					<input bind:value={gmailForm.redirect_uri} autocomplete="off" />
				</label>
				<div class="form-actions wide">
					<button type="button" onclick={startGmailSetup} disabled={isSetupSubmitting}>
						Start OAuth
					</button>
				</div>
			</form>

			{#if gmailPending}
				<div class="oauth-box">
					<a href={gmailPending.authorization_url} target="_blank" rel="noreferrer">
						Open Google consent
					</a>
					<label>
						<span>Authorization code</span>
						<input bind:value={gmailAuthorizationCode} autocomplete="off" />
					</label>
					<button type="button" onclick={completeGmailSetup} disabled={isSetupSubmitting}>
						Complete Gmail
					</button>
				</div>
			{/if}
		{:else}
			<form class="setup-form" onsubmit={(event) => event.preventDefault()}>
				<label>
					<span>Account ID</span>
					<input bind:value={imapForm.account_id} autocomplete="off" />
				</label>
				<label>
					<span>Display name</span>
					<input bind:value={imapForm.display_name} autocomplete="off" />
				</label>
				<label>
					<span>Email address</span>
					<input bind:value={imapForm.external_account_id} autocomplete="email" />
				</label>
				<label>
					<span>Username</span>
					<input bind:value={imapForm.username} autocomplete="username" />
				</label>
				<label>
					<span>Host</span>
					<input bind:value={imapForm.host} autocomplete="off" />
				</label>
				<label>
					<span>Port</span>
					<input bind:value={imapForm.port} type="number" min="1" max="65535" />
				</label>
				<label>
					<span>Mailbox</span>
					<input bind:value={imapForm.mailbox} autocomplete="off" />
				</label>
				<label>
					<span>Password</span>
					<input bind:value={imapForm.password} type="password" autocomplete="current-password" />
				</label>
				<label class="checkbox-row">
					<input bind:checked={imapForm.tls} type="checkbox" />
					<span>TLS</span>
				</label>
				<div class="form-actions">
					<button type="button" onclick={saveImapAccount} disabled={isSetupSubmitting}>
						Save Account
					</button>
				</div>
			</form>
		{/if}

		{#if setupMessage}
			<p class="setup-state success">{setupMessage}</p>
		{/if}
		{#if setupError}
			<p class="setup-state error">{setupError}</p>
		{/if}
	</aside>
{/if}

<style>
	:global(*) {
		box-sizing: border-box;
	}

	:global(body) {
		margin: 0;
		min-width: 1024px;
		background: #02090b;
		color: #eefefb;
		font-family:
			Inter, 'SF Pro Display', ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont,
			'Segoe UI', sans-serif;
		letter-spacing: 0;
	}

	:global(button),
	:global(input),
	:global(select) {
		font: inherit;
		letter-spacing: 0;
	}

	:global(button) {
		border: 0;
		cursor: pointer;
	}

	:global(button:disabled) {
		cursor: not-allowed;
	}

	.desktop-shell {
		display: grid;
		grid-template-columns: 224px minmax(750px, 1fr) 354px;
		gap: 20px;
		height: 100vh;
		min-height: 720px;
		overflow: hidden;
		padding: 16px 14px 16px 0;
		background:
			linear-gradient(180deg, rgba(9, 31, 34, 0.76), rgba(2, 9, 11, 0.96) 42%),
			#02090b;
	}

	.sidebar {
		position: sticky;
		top: 0;
		display: grid;
		grid-template-rows: auto auto auto 1fr auto auto;
		align-self: stretch;
		min-height: calc(100vh - 32px);
		padding: 20px 13px 14px;
		border: 1px solid rgba(37, 224, 197, 0.14);
		border-left: 0;
		border-radius: 0 18px 34px 0;
		background:
			linear-gradient(180deg, rgba(4, 26, 29, 0.96), rgba(2, 13, 16, 0.98)),
			#020d10;
		box-shadow: inset -1px 0 0 rgba(255, 255, 255, 0.03), 18px 0 48px rgba(0, 0, 0, 0.28);
	}

	.brand,
	.greeting,
	.profile-card,
	.rail-actions,
	.panel-title-row,
	.metric-card div,
	.task-list label,
	.project-row,
	.system-card {
		display: flex;
		align-items: center;
	}

	.brand {
		gap: 12px;
		padding: 2px 8px 24px;
	}

	.brand-mark,
	.hero-mark img,
	.avatar-button img {
		display: block;
		object-fit: contain;
	}

	.brand-mark {
		width: 42px;
		height: 42px;
		filter: drop-shadow(0 0 16px rgba(37, 224, 197, 0.55));
	}

	.brand-name,
	.brand-subtitle,
	.shortcuts p,
	.panel-title-row h2,
	.calendar-card h3,
	.discovered p,
	.greeting h1,
	.greeting p,
	.setup-state,
	.account-drawer h2,
	.account-drawer p {
		margin: 0;
	}

	.brand-name {
		color: #f2fffd;
		font-size: 15px;
		font-weight: 600;
		text-transform: uppercase;
	}

	.brand-subtitle {
		margin-top: 2px;
		color: #849ca0;
		font-size: 10px;
		font-weight: 700;
		text-transform: uppercase;
	}

	.nav-group {
		display: grid;
		gap: 5px;
	}

	.nav-group button {
		display: grid;
		grid-template-columns: 22px 1fr auto;
		align-items: center;
		gap: 10px;
		width: 100%;
		min-height: 38px;
		border: 1px solid transparent;
		border-radius: 7px;
		background: transparent;
		color: #c6d7d7;
		padding: 0 9px;
		text-align: left;
	}

	.nav-group button.active {
		border-color: rgba(40, 236, 205, 0.45);
		background: linear-gradient(90deg, rgba(12, 112, 93, 0.62), rgba(8, 54, 54, 0.42));
		box-shadow: inset 0 0 24px rgba(28, 221, 188, 0.14);
		color: #40f3d1;
	}

	.nav-group button.disabled {
		opacity: 0.76;
	}

	.nav-group button em,
	.task-list strong,
	.kbd {
		border-radius: 999px;
		font-style: normal;
	}

	.nav-group button em {
		background: rgba(33, 218, 183, 0.16);
		color: #39f2d0;
		font-size: 11px;
		padding: 3px 8px;
	}

	.nav-separator {
		height: 1px;
		margin: 20px 0 18px;
		background: rgba(129, 202, 194, 0.11);
	}

	.shortcuts {
		padding: 0 8px;
	}

	.shortcuts p,
	.discovered p {
		color: #8ea4a6;
		font-size: 11px;
		font-weight: 700;
		text-transform: uppercase;
	}

	.shortcuts .nav-group {
		margin-top: 12px;
	}

	.profile-card {
		gap: 11px;
		margin: 22px -13px 0;
		padding: 12px 22px;
		border-top: 1px solid rgba(78, 157, 151, 0.15);
		border-bottom: 1px solid rgba(78, 157, 151, 0.1);
		background: rgba(9, 30, 33, 0.58);
	}

	.profile-card img {
		width: 42px;
		height: 42px;
		border-radius: 50%;
		object-fit: cover;
	}

	.profile-card div {
		min-width: 0;
		flex: 1;
	}

	.profile-card strong {
		display: block;
		color: #f2fffd;
		font-size: 13px;
		font-weight: 600;
	}

	.profile-card span {
		color: #29e2c2;
		font-size: 12px;
	}

	.sidebar-tools {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 8px;
		padding-top: 14px;
	}

	.sidebar-tools button,
	.icon-button {
		display: inline-grid;
		place-items: center;
		width: 34px;
		height: 34px;
		border: 1px solid rgba(130, 211, 205, 0.15);
		border-radius: 8px;
		background: rgba(5, 24, 27, 0.72);
		color: #b6cdcc;
	}

	.workspace {
		display: grid;
		grid-template-rows: auto auto 1fr auto;
		gap: 14px;
		min-height: 0;
		min-width: 0;
		padding-top: 0;
	}

	.topbar,
	.rail-actions {
		height: 38px;
	}

	.topbar {
		display: flex;
		align-items: center;
	}

	.search-box,
	.quick-command,
	.assistant-card label {
		display: flex;
		align-items: center;
		border: 1px solid rgba(111, 205, 195, 0.17);
		background: rgba(9, 31, 35, 0.76);
		color: #8ba4a5;
	}

	.search-box {
		width: min(540px, 62vw);
		height: 38px;
		border-radius: 8px;
		padding: 0 10px;
		box-shadow: inset 0 0 28px rgba(24, 189, 164, 0.04);
	}

	.search-box input,
	.quick-command input,
	.assistant-card input,
	.setup-form input {
		min-width: 0;
		flex: 1;
		border: 0;
		outline: 0;
		background: transparent;
		color: #edfffc;
	}

	.search-box input,
	.quick-command input,
	.assistant-card input {
		padding: 0 10px;
		font-size: 13px;
	}

	.kbd {
		border: 1px solid rgba(145, 214, 206, 0.12);
		background: rgba(255, 255, 255, 0.03);
		color: #90a6a6;
		font-size: 11px;
		line-height: 1;
		padding: 5px 7px;
	}

	.hero-row {
		display: grid;
		grid-template-columns: minmax(285px, 1fr) minmax(525px, 604px);
		align-items: center;
		gap: 14px;
		min-height: 98px;
	}

	.greeting {
		gap: 16px;
		min-width: 0;
	}

	.hero-mark {
		display: grid;
		place-items: center;
		width: 66px;
		height: 66px;
		border: 1px solid rgba(40, 236, 205, 0.38);
		border-radius: 50%;
		background:
			linear-gradient(180deg, rgba(13, 84, 78, 0.5), rgba(5, 29, 31, 0.5)),
			#041214;
		box-shadow: 0 0 36px rgba(30, 230, 196, 0.2);
	}

	.hero-mark img {
		width: 58px;
		height: 58px;
	}

	.greeting h1 {
		color: #ffffff;
		font-size: 23px;
		font-weight: 500;
		line-height: 1.18;
	}

	.greeting p {
		margin-top: 9px;
		color: #9bb1b0;
		font-size: 13px;
	}

	.metric-grid {
		display: grid;
		grid-template-columns: repeat(5, minmax(94px, 1fr));
		gap: 9px;
	}

	.metric-card,
	.panel {
		border: 1px solid rgba(82, 204, 190, 0.16);
		background:
			linear-gradient(180deg, rgba(8, 29, 33, 0.94), rgba(5, 22, 25, 0.9)),
			#06181b;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.035);
	}

	.metric-card {
		position: relative;
		min-height: 82px;
		padding: 13px 12px;
		border-radius: 8px;
		overflow: hidden;
	}

	.metric-card span {
		display: block;
		color: #93a9a9;
		font-size: 10px;
		text-transform: uppercase;
	}

	.metric-card div {
		justify-content: space-between;
		margin-top: 10px;
		color: #25e0c5;
	}

	.metric-card strong {
		color: #ffffff;
		font-size: 23px;
		font-weight: 500;
	}

	.metric-card small {
		display: block;
		margin-top: 5px;
		color: #2ef1cd;
		font-size: 11px;
	}

	.metric-card.focus {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 4px 10px;
		border-color: rgba(45, 235, 204, 0.32);
	}

	.metric-card.focus span,
	.metric-card.focus small {
		grid-column: 1;
	}

	.score-ring {
		grid-column: 2;
		grid-row: 1 / 3;
		display: grid !important;
		place-items: center;
		width: 48px;
		height: 48px;
		margin: 0 !important;
		border: 3px solid rgba(45, 235, 204, 0.8);
		border-right-color: rgba(45, 235, 204, 0.22);
		border-radius: 50%;
	}

	.score-ring strong {
		font-size: 14px;
	}

	.main-grid {
		display: grid;
		grid-template-columns: minmax(430px, 1.18fr) minmax(390px, 0.94fr);
		gap: 12px;
		min-height: 0;
		overflow: hidden;
	}

	.center-column {
		display: grid;
		gap: 12px;
		min-height: 0;
		min-width: 0;
	}

	.panel {
		border-radius: 8px;
		overflow: hidden;
	}

	.panel-header,
	.panel-title-row {
		min-height: 48px;
		border-bottom: 1px solid rgba(102, 189, 180, 0.12);
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 16px;
	}

	.panel-title-row {
		justify-content: space-between;
		gap: 12px;
		padding: 0 16px;
	}

	.panel-title-row h2 {
		color: #f6fffe;
		font-size: 16px;
		font-weight: 500;
	}

	.panel-title-row > div {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.panel-tabs,
	.small-tabs,
	.provider-tabs {
		display: flex;
		align-items: center;
		gap: 20px;
	}

	.panel-tabs button,
	.small-tabs button {
		position: relative;
		height: 48px;
		background: transparent;
		color: #91a7a7;
		font-size: 13px;
	}

	.panel-tabs button.active,
	.small-tabs button.active {
		color: #ffffff;
	}

	.panel-tabs button.active::after,
	.small-tabs button.active::after {
		position: absolute;
		right: 0;
		bottom: 0;
		left: 0;
		height: 2px;
		background: #2df0ce;
		content: '';
	}

	.timeline-toolbar {
		display: grid;
		grid-template-columns: 1fr auto auto;
		align-items: center;
		gap: 10px;
		padding: 14px 18px 5px;
	}

	.today-dot {
		display: inline-flex;
		align-items: center;
		gap: 10px;
		color: #2deac9;
		font-size: 11px;
		font-weight: 700;
		text-transform: uppercase;
	}

	.today-dot::before {
		width: 12px;
		height: 12px;
		border-radius: 50%;
		background: #2df0ce;
		box-shadow: 0 0 16px rgba(45, 240, 206, 0.8);
		content: '';
	}

	.timeline-toolbar select {
		height: 32px;
		border: 1px solid rgba(125, 204, 197, 0.16);
		border-radius: 7px;
		background: rgba(3, 20, 23, 0.76);
		color: #9fb6b5;
		padding: 0 30px 0 12px;
	}

	.timeline-list {
		display: grid;
		min-height: 0;
		overflow: hidden;
		padding: 0 16px 2px;
	}

	.timeline-panel {
		display: grid;
		grid-template-rows: auto auto 1fr auto;
		min-height: 0;
	}

	.timeline-item {
		display: grid;
		grid-template-columns: 44px 20px 38px 1fr;
		gap: 9px;
		min-height: 84px;
		padding: 10px 0;
		border-bottom: 1px solid rgba(113, 189, 181, 0.08);
	}

	.timeline-item.muted {
		min-height: 68px;
	}

	.time {
		align-self: start;
		color: #36e5c9;
		font-size: 12px;
		padding-top: 12px;
		text-align: right;
	}

	.rail {
		position: relative;
		display: flex;
		justify-content: center;
	}

	.rail::before {
		position: absolute;
		top: -16px;
		bottom: -16px;
		width: 1px;
		background: rgba(42, 225, 198, 0.55);
		content: '';
	}

	.rail-dot {
		z-index: 1;
		width: 7px;
		height: 7px;
		margin-top: 17px;
		border-radius: 50%;
		background: #2df0ce;
		box-shadow: 0 0 12px rgba(45, 240, 206, 0.85);
	}

	.event-icon,
	.project-icon {
		display: grid;
		place-items: center;
		border-radius: 50%;
	}

	.event-icon {
		width: 32px;
		height: 32px;
		margin-top: 4px;
		background: rgba(28, 199, 197, 0.17);
		color: #2eeed0;
	}

	.event-icon.tone-1 {
		background: rgba(31, 138, 214, 0.2);
		color: #30d8ff;
	}

	.event-icon.tone-2 {
		background: rgba(45, 240, 206, 0.14);
	}

	.event-icon.tone-3 {
		background: rgba(161, 87, 220, 0.22);
		color: #cf9dff;
	}

	.event-body {
		min-width: 0;
	}

	.event-body header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 14px;
	}

	.event-body strong {
		color: #f5fffe;
		font-size: 13px;
		font-weight: 600;
	}

	.event-body header span {
		flex: 0 0 auto;
		color: #8ba2a1;
		font-size: 11px;
	}

	.event-body p,
	.event-body small,
	.project-main span,
	.project-meta span,
	.discovered small,
	.calendar-card h3,
	.agenda article div span,
	.insight-grid span,
	.account-drawer p,
	.setup-form span {
		color: #91a8a8;
	}

	.event-body p {
		margin: 7px 0 0;
		font-size: 12px;
		line-height: 1.35;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.event-body small {
		display: block;
		margin-top: 6px;
		font-size: 11px;
	}

	.tag {
		display: inline-block;
		margin-top: 7px;
		border-radius: 4px;
		font-size: 10px;
		font-style: normal;
		padding: 3px 6px;
	}

	.tag.amber {
		border: 1px solid rgba(236, 183, 70, 0.28);
		background: rgba(236, 183, 70, 0.16);
		color: #eeb84b;
	}

	.tag.cyan {
		background: rgba(43, 221, 194, 0.15);
		color: #2df0ce;
	}

	.yesterday-label {
		color: #96aeb0;
		font-size: 11px;
		font-weight: 700;
		letter-spacing: 0;
		padding: 14px 0 3px 4px;
		text-transform: uppercase;
	}

	.load-button {
		display: block;
		width: 204px;
		height: 32px;
		margin: 7px auto 8px;
		border: 1px solid rgba(89, 217, 202, 0.2);
		border-radius: 7px;
		background: rgba(11, 43, 45, 0.48);
		color: #43e9cf;
		font-size: 12px;
	}

	.graph-panel {
		min-height: 432px;
	}

	.graph-canvas {
		position: relative;
		height: 290px;
		margin: 0 16px;
		overflow: hidden;
		border-bottom: 1px solid rgba(102, 189, 180, 0.1);
		background-image:
			linear-gradient(rgba(45, 240, 206, 0.04) 1px, transparent 1px),
			linear-gradient(90deg, rgba(45, 240, 206, 0.035) 1px, transparent 1px);
		background-size: 32px 32px;
	}

	.graph-canvas::before {
		position: absolute;
		inset: 28px 34px;
		border: 1px dashed rgba(45, 240, 206, 0.08);
		border-radius: 50%;
		content: '';
	}

	.graph-core {
		position: absolute;
		top: 50%;
		left: 52%;
		z-index: 2;
		display: grid;
		place-items: center;
		width: 62px;
		height: 62px;
		border: 1px solid rgba(45, 240, 206, 0.84);
		border-radius: 50%;
		background: rgba(13, 126, 113, 0.78);
		box-shadow: 0 0 34px rgba(45, 240, 206, 0.42);
		color: #dffffa;
		transform: translate(-50%, -50%);
	}

	.graph-core span {
		position: absolute;
		top: calc(100% + 8px);
		width: 110px;
		color: #f8fffe;
		font-size: 12px;
		text-align: center;
	}

	.graph-node {
		position: absolute;
		display: grid;
		grid-template-columns: 36px max-content;
		grid-template-rows: 18px 16px;
		column-gap: 7px;
		align-items: center;
		transform: translate(-50%, -50%);
	}

	.graph-node > span {
		grid-row: 1 / 3;
		display: grid;
		place-items: center;
		width: 34px;
		height: 34px;
		border: 1px solid rgba(45, 240, 206, 0.32);
		border-radius: 50%;
		background: rgba(16, 110, 101, 0.66);
		color: #40f2d4;
		box-shadow: 0 0 20px rgba(45, 240, 206, 0.22);
	}

	.graph-node strong,
	.graph-node small {
		white-space: nowrap;
	}

	.graph-node strong {
		color: #ffffff;
		font-size: 11px;
	}

	.graph-node small {
		color: #90a7a7;
		font-size: 10px;
	}

	.graph-line {
		position: absolute;
		top: 50%;
		left: 52%;
		width: 178px;
		height: 1px;
		background: rgba(45, 240, 206, 0.48);
		transform-origin: left;
	}

	.graph-line.one {
		transform: rotate(-61deg);
	}

	.graph-line.two {
		transform: rotate(-25deg);
	}

	.graph-line.three {
		transform: rotate(33deg);
	}

	.graph-line.four {
		transform: rotate(147deg);
	}

	.discovered {
		padding: 13px 16px 12px;
	}

	.discovered > div {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 8px;
		margin-top: 10px;
	}

	.discovered button {
		display: grid;
		grid-template-columns: 28px 1fr;
		gap: 8px;
		align-items: center;
		min-height: 48px;
		border: 1px solid rgba(111, 205, 195, 0.13);
		border-radius: 7px;
		background: rgba(10, 42, 45, 0.52);
		color: #35e9cc;
		text-align: left;
	}

	.discovered strong,
	.discovered small {
		display: block;
	}

	.discovered strong {
		color: #eefefb;
		font-size: 10px;
		font-weight: 500;
	}

	.discovered small {
		margin-top: 2px;
		font-size: 9px;
	}

	.projects-panel {
		min-height: 298px;
	}

	.project-list {
		display: grid;
		padding: 10px 14px;
	}

	.project-row {
		display: grid;
		grid-template-columns: 34px minmax(118px, 1fr) minmax(76px, 120px) 42px 70px;
		gap: 12px;
		min-height: 52px;
		border-bottom: 1px solid rgba(102, 189, 180, 0.09);
	}

	.project-row:last-child {
		border-bottom: 0;
	}

	.project-icon {
		width: 32px;
		height: 32px;
	}

	.project-icon.amber {
		background: rgba(226, 170, 45, 0.18);
		color: #f3b63f;
	}

	.project-icon.cyan {
		background: rgba(29, 168, 220, 0.2);
		color: #2ed4ff;
	}

	.project-icon.purple {
		background: rgba(175, 88, 215, 0.22);
		color: #d893ff;
	}

	.project-icon.mint {
		background: rgba(30, 218, 176, 0.15);
		color: #2df0ce;
	}

	.project-main {
		min-width: 0;
	}

	.project-main strong {
		display: block;
		color: #f7fffd;
		font-size: 12px;
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.project-main span,
	.project-meta span {
		display: block;
		font-size: 10px;
		margin-top: 3px;
	}

	.progress {
		height: 6px;
		border-radius: 999px;
		background: rgba(66, 130, 126, 0.22);
		overflow: hidden;
	}

	.progress span {
		display: block;
		height: 100%;
		border-radius: inherit;
		background: linear-gradient(90deg, #27d0b3, #54f0d4);
	}

	.progress-value {
		color: #55f4d8;
		font-size: 11px;
		font-weight: 500;
	}

	.project-meta {
		color: #a6bbbb;
		font-size: 10px;
	}

	.quick-command {
		height: 50px;
		border-radius: 10px;
		padding: 0 10px;
	}

	.quick-command button {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		height: 36px;
		border-left: 1px solid rgba(111, 205, 195, 0.1);
		background: transparent;
		color: #d8efed;
		padding: 0 12px;
		font-size: 12px;
	}

	.quick-command button:last-child {
		border: 1px solid rgba(45, 240, 206, 0.28);
		border-radius: 7px;
		background: rgba(39, 198, 171, 0.13);
		color: #45f3d4;
	}

	.right-rail {
		display: grid;
		grid-template-rows: 38px auto auto auto auto 1fr;
		align-content: start;
		gap: 12px;
		min-height: 0;
		min-width: 0;
		overflow: hidden;
	}

	.rail-actions {
		justify-content: end;
		gap: 9px;
	}

	.rail-actions > button:not(.icon-button):not(.avatar-button) {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		height: 38px;
		border: 1px solid rgba(111, 205, 195, 0.16);
		border-radius: 8px;
		background: rgba(6, 23, 27, 0.8);
		color: #ccdedd;
		padding: 0 10px;
		font-size: 12px;
	}

	.avatar-button {
		display: grid;
		place-items: center;
		width: 38px;
		height: 38px;
		border: 1px solid rgba(45, 240, 206, 0.2);
		border-radius: 50%;
		background: rgba(28, 174, 151, 0.16);
	}

	.avatar-button img {
		width: 31px;
		height: 31px;
	}

	.calendar-card {
		min-height: 274px;
	}

	.calendar-card h3 {
		padding: 14px 16px 8px;
		font-size: 13px;
		font-weight: 600;
	}

	.calendar-strip {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		gap: 6px;
		padding: 0 12px 10px;
	}

	.calendar-strip button {
		display: grid;
		place-items: center;
		gap: 3px;
		min-height: 42px;
		border-radius: 7px;
		background: transparent;
		color: #dcefed;
	}

	.calendar-strip button.active {
		border: 1px solid rgba(45, 240, 206, 0.48);
		background: rgba(23, 134, 119, 0.62);
		color: #ffffff;
	}

	.calendar-strip span {
		font-size: 9px;
		text-transform: uppercase;
	}

	.calendar-strip strong {
		font-size: 14px;
		font-weight: 500;
	}

	.agenda {
		display: grid;
		gap: 7px;
		padding: 0 10px 10px;
	}

	.agenda article {
		display: grid;
		grid-template-columns: 42px 1fr auto;
		gap: 10px;
		align-items: center;
		min-height: 38px;
		border-radius: 7px;
		background: rgba(13, 44, 47, 0.6);
		padding: 7px 9px;
	}

	.agenda article div strong,
	.agenda article div span {
		display: block;
	}

	.agenda article div strong {
		color: #e9fffc;
		font-size: 11px;
		font-weight: 500;
	}

	.agenda article div span {
		margin-top: 2px;
		font-size: 10px;
	}

	.agenda article p {
		margin: 0;
		color: #f6fffe;
		font-size: 12px;
		line-height: 1.25;
	}

	.agenda article em {
		border-radius: 999px;
		background: rgba(45, 240, 206, 0.13);
		color: #87fff0;
		font-size: 10px;
		font-style: normal;
		padding: 4px 7px;
	}

	.tasks-card {
		min-height: 232px;
	}

	.small-tabs {
		gap: 18px;
		height: 39px;
		padding: 0 14px;
		border-bottom: 1px solid rgba(102, 189, 180, 0.1);
	}

	.small-tabs button {
		height: 39px;
		font-size: 11px;
	}

	.small-tabs span {
		border-radius: 999px;
		background: rgba(135, 170, 170, 0.18);
		padding: 1px 5px;
	}

	.task-list {
		display: grid;
		gap: 7px;
		padding: 10px 12px 14px;
	}

	.task-list label {
		display: grid;
		grid-template-columns: 17px 1fr 44px 64px;
		gap: 8px;
		color: #d9eeec;
		font-size: 11px;
	}

	.task-list input {
		width: 13px;
		height: 13px;
		margin: 0;
		accent-color: #27d8bd;
	}

	.task-list span {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.task-list em,
	.task-list strong {
		font-size: 10px;
		font-style: normal;
		text-align: center;
	}

	.task-list em {
		color: #dcefed;
	}

	.task-list strong {
		background: rgba(31, 203, 170, 0.14);
		color: #2ff1cd;
		padding: 3px 5px;
	}

	.insights-card {
		min-height: 188px;
	}

	.insight-grid {
		display: grid;
		grid-template-columns: 1.1fr 0.9fr;
		gap: 8px;
		padding: 10px;
	}

	.insight-grid article {
		min-height: 120px;
		border: 1px solid rgba(111, 205, 195, 0.12);
		border-radius: 7px;
		background: rgba(12, 40, 44, 0.5);
		padding: 11px;
	}

	.insight-grid strong {
		display: block;
		color: #f4fffe;
		font-size: 11px;
		font-weight: 600;
	}

	.insight-grid span {
		display: block;
		margin-top: 4px;
		font-size: 10px;
	}

	.bar-chart {
		display: grid;
		grid-template-columns: repeat(8, 1fr);
		align-items: end;
		gap: 8px;
		height: 66px;
		margin-top: 12px;
	}

	.bar-chart i {
		display: block;
		border-radius: 2px 2px 0 0;
		background: linear-gradient(180deg, #30efd0, rgba(25, 129, 116, 0.75));
		box-shadow: 0 0 16px rgba(45, 240, 206, 0.18);
	}

	.insight-grid ul {
		display: grid;
		gap: 7px;
		margin: 10px 0 0;
		padding: 0;
		list-style: none;
	}

	.insight-grid li {
		display: flex;
		justify-content: space-between;
		gap: 12px;
		color: #dcefed;
		font-size: 10px;
	}

	.insight-grid li em {
		color: #ffffff;
		font-style: normal;
	}

	.assistant-card {
		min-height: 112px;
		padding-bottom: 10px;
	}

	.assistant-card label {
		height: 36px;
		margin: 10px;
		border-radius: 8px;
		padding: 0 9px 0 0;
	}

	.assistant-card > div {
		display: flex;
		gap: 6px;
		padding: 0 10px;
	}

	.assistant-card > div button {
		flex: 1;
		min-height: 26px;
		border: 1px solid rgba(45, 240, 206, 0.12);
		border-radius: 6px;
		background: rgba(30, 154, 134, 0.13);
		color: #38eccd;
		font-size: 10px;
	}

	.system-card {
		justify-content: space-between;
		min-height: 72px;
		padding: 12px 14px;
	}

	.system-card h2 {
		margin: 0 0 8px;
		color: #f4fffe;
		font-size: 13px;
		font-weight: 500;
	}

	.system-card p {
		position: relative;
		margin: 0;
		padding-left: 12px;
		color: #8fa8a7;
		font-size: 11px;
	}

	.system-card p::before {
		position: absolute;
		top: 5px;
		left: 0;
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: #879b9b;
		content: '';
	}

	.system-card p.online {
		color: #2df0ce;
	}

	.system-card p.online::before {
		background: #2df0ce;
	}

	.system-card p.error {
		color: #ff8d8d;
	}

	.system-card p.error::before {
		background: #ff6d6d;
	}

	.sparkline {
		position: relative;
		width: 164px;
		height: 46px;
		overflow: hidden;
	}

	.sparkline span {
		position: absolute;
		inset: 8px 0 0;
		border-top: 1px solid #2df0ce;
		border-radius: 50%;
		transform: rotate(-8deg);
		filter: drop-shadow(0 0 10px rgba(45, 240, 206, 0.55));
	}

	.link-button,
	.ghost-button {
		background: transparent;
		color: #9bb1b0;
		font-size: 11px;
	}

	.ghost-button {
		height: 26px;
		border: 1px solid rgba(45, 240, 206, 0.18);
		border-radius: 6px;
		background: rgba(33, 167, 144, 0.1);
		color: #3ae9cb;
		padding: 0 10px;
	}

	.drawer-backdrop {
		position: fixed;
		inset: 0;
		z-index: 20;
		background: rgba(0, 0, 0, 0.48);
	}

	.account-drawer {
		position: fixed;
		top: 18px;
		right: 18px;
		bottom: 18px;
		z-index: 21;
		display: grid;
		grid-template-rows: auto auto auto 1fr;
		gap: 16px;
		width: min(560px, calc(100vw - 36px));
		overflow: auto;
		border: 1px solid rgba(45, 240, 206, 0.24);
		border-radius: 14px;
		background:
			linear-gradient(180deg, rgba(8, 31, 35, 0.98), rgba(4, 18, 21, 0.98)),
			#041215;
		box-shadow: 0 24px 80px rgba(0, 0, 0, 0.55);
		padding: 18px;
	}

	.account-drawer > header {
		display: flex;
		justify-content: space-between;
		gap: 18px;
		align-items: start;
	}

	.account-drawer p {
		color: #37e8c9;
		font-size: 11px;
		font-weight: 700;
		text-transform: uppercase;
	}

	.account-drawer h2 {
		margin-top: 6px;
		color: #ffffff;
		font-size: 22px;
		font-weight: 500;
	}

	.provider-tabs {
		gap: 6px;
		padding: 4px;
		border: 1px solid rgba(111, 205, 195, 0.14);
		border-radius: 8px;
		background: rgba(4, 21, 24, 0.72);
	}

	.provider-tabs button {
		flex: 1;
		height: 34px;
		border-radius: 6px;
		background: transparent;
		color: #9bb1b0;
	}

	.provider-tabs button.active {
		background: rgba(36, 207, 178, 0.16);
		color: #39f0d0;
	}

	.setup-form {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 12px;
		align-content: start;
	}

	.setup-form label {
		display: grid;
		gap: 6px;
		min-width: 0;
	}

	.setup-form label.wide,
	.form-actions.wide {
		grid-column: 1 / -1;
	}

	.setup-form span {
		font-size: 11px;
		font-weight: 600;
	}

	.setup-form input {
		height: 38px;
		border: 1px solid rgba(111, 205, 195, 0.18);
		border-radius: 7px;
		background: rgba(4, 21, 24, 0.76);
		padding: 0 10px;
	}

	.checkbox-row {
		display: flex !important;
		align-items: center;
		gap: 8px !important;
		padding-top: 18px;
	}

	.checkbox-row input {
		width: 16px;
		height: 16px;
		flex: 0 0 auto;
		padding: 0;
		accent-color: #2deac9;
	}

	.form-actions {
		display: flex;
		align-items: end;
	}

	.form-actions button,
	.oauth-box button {
		height: 38px;
		border-radius: 7px;
		background: #25d8bd;
		color: #02201f;
		font-weight: 700;
		padding: 0 15px;
	}

	.oauth-box {
		display: grid;
		gap: 12px;
		border: 1px solid rgba(45, 240, 206, 0.18);
		border-radius: 9px;
		background: rgba(10, 44, 47, 0.58);
		padding: 12px;
	}

	.oauth-box a {
		color: #41f3d3;
	}

	.setup-state {
		border-radius: 8px;
		font-size: 12px;
		padding: 10px 12px;
	}

	.setup-state.success {
		border: 1px solid rgba(45, 240, 206, 0.25);
		background: rgba(37, 216, 189, 0.12);
		color: #51f7d9;
	}

	.setup-state.error {
		border: 1px solid rgba(255, 110, 110, 0.3);
		background: rgba(128, 32, 40, 0.26);
		color: #ffabab;
	}

	@media (max-width: 1360px) {
		.desktop-shell {
			grid-template-columns: 210px minmax(720px, 1fr) 330px;
			gap: 14px;
		}

		.metric-grid {
			grid-template-columns: repeat(5, minmax(88px, 1fr));
		}

		.quick-command button {
			padding: 0 8px;
		}
	}

	@media (max-width: 1200px) {
		.desktop-shell {
			grid-template-columns: 204px minmax(0, 1fr) 296px;
			gap: 10px;
			padding: 10px 8px 10px 0;
		}

		.sidebar {
			min-height: calc(100vh - 20px);
			padding-inline: 8px;
		}

		.nav-group button {
			gap: 8px;
			padding-inline: 7px;
		}

		.workspace {
			gap: 10px;
		}

		.search-box {
			width: 100%;
		}

		.hero-row {
			grid-template-columns: 1fr;
			align-items: start;
			gap: 10px;
		}

		.metric-grid {
			grid-template-columns: repeat(5, minmax(74px, 1fr));
			gap: 7px;
		}

		.metric-card {
			min-height: 74px;
			padding: 10px 9px;
		}

		.metric-card strong {
			font-size: 20px;
		}

		.metric-card div {
			margin-top: 8px;
		}

		.main-grid {
			grid-template-columns: 1fr;
			gap: 10px;
		}

		.timeline-item {
			grid-template-columns: 42px 18px 36px 1fr;
			gap: 7px;
		}

		.graph-panel {
			min-height: 392px;
		}

		.graph-canvas {
			height: 242px;
		}

		.project-row {
			grid-template-columns: 34px minmax(112px, 1fr) minmax(70px, 112px) 38px 66px;
			gap: 9px;
		}

		.right-rail {
			gap: 10px;
		}

		.rail-actions > button:not(.icon-button):not(.avatar-button) {
			width: 168px;
			overflow: hidden;
			justify-content: center;
			white-space: nowrap;
		}

		.calendar-strip {
			gap: 4px;
			padding-inline: 8px;
		}

		.insight-grid {
			grid-template-columns: 1fr;
		}

		.assistant-card > div {
			display: grid;
			grid-template-columns: 1fr;
		}

		.quick-command {
			display: grid;
			grid-template-columns: 24px minmax(0, 1fr) auto auto;
			gap: 8px;
		}

		.quick-command button:not(:last-child) {
			display: none;
		}
	}
</style>
