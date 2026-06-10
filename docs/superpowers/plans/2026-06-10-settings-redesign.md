# Settings Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current card-heavy Settings Accounts tab with an IDE-style Settings surface and an Integrations workbench backed by existing frontend account data.

**Architecture:** This is a frontend-first slice. A new integration view-model helper derives provider/integration rows from `providerAccounts` and `calendarAccounts`, the Settings store exposes those rows, and the Settings page renders a tree + work area + inspector layout. No backend API or schema changes are required for the first pass.

**Tech Stack:** Svelte 5, Svelte stores, TypeScript, Vitest, existing Hermes CSS tokens, existing i18n JSON dictionaries.

---

## Scope And Preconditions

Existing backend work-in-progress may be present in the workspace. This plan must not stage, edit or commit backend files.

Before executing implementation tasks:

- [ ] Run `git status --short`.
- [ ] Confirm any backend paths such as `backend/src/...` are unrelated and remain unstaged.
- [ ] If the executor wants isolation, create a frontend-focused worktree before editing, following the repository's worktree workflow.

## File Structure

Create:

- `frontend/src/lib/services/integrations.ts`  
  Owns `IntegrationViewModel`, service-state types, grouping and derivation from API DTOs.

- `frontend/src/lib/services/integrations.test.ts`  
  Unit tests for Gmail, iCloud, Telegram and empty WhatsApp rows.

- `frontend/src/lib/pages/settings/widgets/IntegrationsSettings.svelte`  
  Renders the selected Integrations workbench: compact table, service columns and inspector.

Modify:

- `frontend/src/lib/stores/settings.ts`  
  Expose derived `integrationViewModels` and change `SettingsSection` from `accounts` to `integrations`.

- `frontend/src/lib/stores/settings.test.ts`  
  Verify settings store derives integration rows from loaded account/calendar data.

- `frontend/src/lib/pages/settings/SettingsPage.svelte`  
  Replace horizontal tabs with IDE-style settings tree, route `integrations` to `IntegrationsSettings`, keep existing settings widgets for Application, Appearance, Sidebar and Language.

- `frontend/src/lib/pages/pages.css`  
  Add scoped Settings tree/workbench/table/inspector CSS and remove or retire unused account-card layout styles when no longer referenced.

- `frontend/src/lib/i18n/ru.json`  
  Add Russian translations for all new user-visible strings.

Do not modify:

- backend routes, migrations or DTOs;
- account setup modal behavior beyond calling the existing `openAccountDrawer('mail')`;
- global shell navigation.

---

### Task 1: Add Integration View Model

**Files:**
- Create: `frontend/src/lib/services/integrations.ts`
- Test: `frontend/src/lib/services/integrations.test.ts`

- [ ] **Step 1: Write the failing integration view-model tests**

Create `frontend/src/lib/services/integrations.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import type { CalendarAccount, ProviderAccount } from '$lib/api';
import { buildIntegrationViewModels, serviceStateLabel } from './integrations';

function providerAccount(overrides: Partial<ProviderAccount>): ProviderAccount {
	return {
		account_id: 'account-primary',
		provider_kind: 'gmail',
		display_name: 'Provider Account',
		external_account_id: 'provider@example.com',
		config: {},
		created_at: '2026-06-10T00:00:00Z',
		updated_at: '2026-06-10T10:00:00Z',
		...overrides
	};
}

function calendarAccount(overrides: Partial<CalendarAccount>): CalendarAccount {
	return {
		account_id: 'google-calendar:gmail-primary',
		provider: 'google',
		account_name: 'Google Workspace',
		email: 'gmail-primary',
		credentials_reference: 'secret:provider-account:gmail-primary:oauth_token',
		sync_status: 'idle',
		capabilities: { mail_account_id: 'gmail-primary', connected_services: ['calendar'] },
		created_at: '2026-06-10T00:00:00Z',
		updated_at: '2026-06-10T10:30:00Z',
		...overrides
	};
}

describe('integration view models', () => {
	it('derives Google mail, calendar and people service states from existing metadata', () => {
		const integrations = buildIntegrationViewModels(
			[
				providerAccount({
					account_id: 'gmail-primary',
					provider_kind: 'gmail',
					display_name: 'Google Workspace',
					external_account_id: 'gmail-primary',
					config: { connected_services: ['mail', 'calendar', 'contacts'] }
				})
			],
			[calendarAccount({ account_id: 'google-calendar:gmail-primary' })]
		);

		expect(integrations).toHaveLength(2);
		expect(integrations[0]).toMatchObject({
			integrationId: 'gmail:gmail-primary',
			providerKind: 'gmail',
			title: 'Google Workspace',
			subtitle: 'gmail-primary',
			status: 'connected'
		});
		expect(integrations[0].services.map((service) => [service.id, service.state])).toEqual([
			['mail', 'ready'],
			['calendar', 'ready'],
			['people', 'ready'],
			['messages', 'not_applicable']
		]);
	});

	it('marks requested calendar service as unknown when provider metadata exists but calendar row is missing', () => {
		const [integration] = buildIntegrationViewModels(
			[
				providerAccount({
					account_id: 'icloud-primary',
					provider_kind: 'icloud',
					display_name: 'Primary iCloud',
					external_account_id: 'user@icloud.com',
					config: { connected_services: ['mail', 'calendar', 'contacts'] }
				})
			],
			[]
		);

		expect(integration.status).toBe('partial');
		expect(integration.services.map((service) => [service.id, service.state])).toEqual([
			['mail', 'ready'],
			['calendar', 'unknown'],
			['people', 'ready'],
			['messages', 'not_applicable']
		]);
	});

	it('groups Telegram accounts into one messaging integration row', () => {
		const integrations = buildIntegrationViewModels(
			[
				providerAccount({
					account_id: '682703602_account_alexm36',
					provider_kind: 'telegram_user',
					display_name: '@AlexM36',
					external_account_id: 'telegram:682703602'
				}),
				providerAccount({
					account_id: '5499503231_account_viki_avm',
					provider_kind: 'telegram_user',
					display_name: '@viki_avm',
					external_account_id: 'telegram:5499503231'
				})
			],
			[]
		);

		const telegram = integrations.find((integration) => integration.integrationId === 'telegram');
		expect(telegram).toMatchObject({
			providerKind: 'telegram',
			title: 'Telegram',
			subtitle: '@AlexM36, @viki_avm',
			status: 'connected'
		});
		expect(telegram?.services.map((service) => [service.id, service.state])).toEqual([
			['mail', 'not_applicable'],
			['calendar', 'not_applicable'],
			['people', 'not_applicable'],
			['messages', 'ready']
		]);
		expect(telegram?.accounts).toHaveLength(2);
	});

	it('adds an empty WhatsApp row when no WhatsApp account exists', () => {
		const integrations = buildIntegrationViewModels([], []);

		expect(integrations).toEqual([
			expect.objectContaining({
				integrationId: 'whatsapp',
				providerKind: 'whatsapp_web',
				title: 'WhatsApp',
				subtitle: 'No account configured',
				status: 'empty'
			})
		]);
	});

	it('maps service state labels for table cells', () => {
		expect(serviceStateLabel('ready')).toBe('Ready');
		expect(serviceStateLabel('unknown')).toBe('Auth');
		expect(serviceStateLabel('disabled')).toBe('Disabled');
		expect(serviceStateLabel('not_applicable')).toBe('-');
	});
});
```

- [ ] **Step 2: Run the tests and verify they fail**

Run:

```bash
cd frontend && pnpm test:unit -- src/lib/services/integrations.test.ts
```

Expected: FAIL because `./integrations` does not exist.

- [ ] **Step 3: Implement the integration view-model helper**

Create `frontend/src/lib/services/integrations.ts`:

```ts
import type { CalendarAccount, ProviderAccount } from '$lib/api';
import { accountProviderIcon, accountProviderLabel, accountUpdatedLabel } from './accounts';

export type IntegrationServiceId = 'mail' | 'calendar' | 'people' | 'messages';
export type IntegrationServiceState = 'ready' | 'auth' | 'disabled' | 'not_applicable' | 'unknown';
export type IntegrationStatus = 'connected' | 'needs_action' | 'empty' | 'partial';

export type IntegrationService = {
	id: IntegrationServiceId;
	label: string;
	state: IntegrationServiceState;
	description: string;
};

export type IntegrationViewModel = {
	integrationId: string;
	providerKind: string;
	title: string;
	subtitle: string;
	icon: string;
	updatedAt: string | null;
	updatedLabel: string;
	status: IntegrationStatus;
	services: IntegrationService[];
	accounts: ProviderAccount[];
	calendarAccounts: CalendarAccount[];
	metadata: Record<string, string>;
};

const serviceOrder: IntegrationServiceId[] = ['mail', 'calendar', 'people', 'messages'];

export function buildIntegrationViewModels(
	providerAccounts: ProviderAccount[],
	calendarAccounts: CalendarAccount[]
): IntegrationViewModel[] {
	const integrations = [
		...providerAccounts
			.filter((account) => ['gmail', 'icloud', 'imap'].includes(account.provider_kind))
			.map((account) => buildMailIntegration(account, calendarAccounts)),
		buildTelegramIntegration(
			providerAccounts.filter((account) =>
				['telegram_user', 'telegram_bot'].includes(account.provider_kind)
			)
		),
		buildWhatsappIntegration(
			providerAccounts.filter((account) => account.provider_kind === 'whatsapp_web')
		)
	];

	return integrations
		.filter((integration): integration is IntegrationViewModel => integration !== null)
		.sort(integrationSort);
}

export function serviceStateLabel(state: IntegrationServiceState): string {
	switch (state) {
		case 'ready':
			return 'Ready';
		case 'auth':
		case 'unknown':
			return 'Auth';
		case 'disabled':
			return 'Disabled';
		case 'not_applicable':
			return '-';
		default:
			return 'Auth';
	}
}

export function integrationStatusLabel(status: IntegrationStatus): string {
	switch (status) {
		case 'connected':
			return 'Connected';
		case 'needs_action':
			return 'Need action';
		case 'empty':
			return 'Empty';
		case 'partial':
			return 'Partial';
		default:
			return 'Partial';
	}
}

function buildMailIntegration(
	account: ProviderAccount,
	calendarAccounts: CalendarAccount[]
): IntegrationViewModel {
	const connectedServices = connectedServiceSet(account);
	const linkedCalendars = calendarAccounts.filter((calendar) =>
		calendarBelongsToProviderAccount(calendar, account)
	);
	const hasMail = ['gmail', 'icloud', 'imap'].includes(account.provider_kind);
	const requestedCalendar = connectedServices.has('calendar');
	const requestedPeople = connectedServices.has('contacts') || connectedServices.has('people');
	const services: IntegrationService[] = [
		service('mail', hasMail ? 'ready' : 'not_applicable', `${accountProviderLabel(account.provider_kind)} mail account`),
		service(
			'calendar',
			requestedCalendar ? (linkedCalendars.length > 0 ? 'ready' : 'unknown') : 'not_applicable',
			requestedCalendar ? 'Calendar account metadata' : 'Calendar is not configured for this integration'
		),
		service(
			'people',
			requestedPeople ? 'ready' : 'not_applicable',
			requestedPeople ? 'Contacts source metadata' : 'Contacts are not configured for this integration'
		),
		service('messages', 'not_applicable', 'Messaging is not provided by this integration')
	];

	return {
		integrationId: `${account.provider_kind}:${account.account_id}`,
		providerKind: account.provider_kind,
		title: account.display_name || accountProviderLabel(account.provider_kind),
		subtitle: account.external_account_id || account.account_id,
		icon: accountProviderIcon(account.provider_kind),
		updatedAt: latestTimestamp([account.updated_at, ...linkedCalendars.map((calendar) => calendar.updated_at)]),
		updatedLabel: accountUpdatedLabel(account),
		status: statusFromServices(services),
		services,
		accounts: [account],
		calendarAccounts: linkedCalendars,
		metadata: {
			'Provider': accountProviderLabel(account.provider_kind),
			'Account ID': account.account_id,
			'External ID': account.external_account_id || account.account_id
		}
	};
}

function buildTelegramIntegration(accounts: ProviderAccount[]): IntegrationViewModel | null {
	if (accounts.length === 0) {
		return null;
	}
	const services = [
		service('mail', 'not_applicable', 'Mail is not provided by Telegram'),
		service('calendar', 'not_applicable', 'Calendar is not provided by Telegram'),
		service('people', 'not_applicable', 'People sync is not provided by Telegram'),
		service('messages', 'ready', `${accounts.length} Telegram account records`)
	];
	return {
		integrationId: 'telegram',
		providerKind: 'telegram',
		title: 'Telegram',
		subtitle: accounts.map((account) => account.display_name).join(', '),
		icon: accountProviderIcon(accounts[0].provider_kind),
		updatedAt: latestTimestamp(accounts.map((account) => account.updated_at)),
		updatedLabel: accountUpdatedLabel(accounts[0]),
		status: 'connected',
		services,
		accounts,
		calendarAccounts: [],
		metadata: {
			'Provider': 'Telegram',
			'Accounts': String(accounts.length)
		}
	};
}

function buildWhatsappIntegration(accounts: ProviderAccount[]): IntegrationViewModel {
	const hasAccount = accounts.length > 0;
	const services = [
		service('mail', 'not_applicable', 'Mail is not provided by WhatsApp'),
		service('calendar', 'not_applicable', 'Calendar is not provided by WhatsApp'),
		service('people', 'not_applicable', 'People sync is not provided by WhatsApp'),
		service('messages', hasAccount ? 'ready' : 'not_applicable', hasAccount ? 'WhatsApp account record' : 'No WhatsApp account configured')
	];
	return {
		integrationId: 'whatsapp',
		providerKind: 'whatsapp_web',
		title: 'WhatsApp',
		subtitle: hasAccount ? accounts.map((account) => account.display_name).join(', ') : 'No account configured',
		icon: 'tabler:brand-whatsapp',
		updatedAt: latestTimestamp(accounts.map((account) => account.updated_at)),
		updatedLabel: hasAccount ? accountUpdatedLabel(accounts[0]) : 'never',
		status: hasAccount ? 'connected' : 'empty',
		services,
		accounts,
		calendarAccounts: [],
		metadata: {
			'Provider': 'WhatsApp Web',
			'Accounts': String(accounts.length)
		}
	};
}

function service(
	id: IntegrationServiceId,
	state: IntegrationServiceState,
	description: string
): IntegrationService {
	return {
		id,
		label: serviceLabel(id),
		state,
		description
	};
}

function serviceLabel(id: IntegrationServiceId): string {
	switch (id) {
		case 'mail':
			return 'Mail';
		case 'calendar':
			return 'Calendar';
		case 'people':
			return 'People';
		case 'messages':
			return 'Messages';
		default:
			return id;
	}
}

function connectedServiceSet(account: ProviderAccount): Set<string> {
	const services = account.config.connected_services;
	if (!Array.isArray(services)) {
		return new Set(['mail']);
	}
	return new Set(services.filter((service): service is string => typeof service === 'string'));
}

function calendarBelongsToProviderAccount(
	calendar: CalendarAccount,
	account: ProviderAccount
): boolean {
	const mailAccountId = calendar.capabilities.mail_account_id;
	return (
		mailAccountId === account.account_id ||
		calendar.email === account.external_account_id ||
		calendar.email === account.account_id ||
		calendar.account_id.endsWith(`:${account.account_id}`)
	);
}

function latestTimestamp(values: Array<string | null | undefined>): string | null {
	const timestamps = values.filter((value): value is string => Boolean(value));
	if (timestamps.length === 0) {
		return null;
	}
	return timestamps.sort().at(-1) ?? null;
}

function statusFromServices(services: IntegrationService[]): IntegrationStatus {
	const active = services.filter((service) => service.state !== 'not_applicable');
	if (active.length === 0) {
		return 'empty';
	}
	if (active.some((service) => ['auth', 'unknown'].includes(service.state))) {
		return active.some((service) => service.state === 'ready') ? 'partial' : 'needs_action';
	}
	return 'connected';
}

function integrationSort(a: IntegrationViewModel, b: IntegrationViewModel): number {
	const priority = new Map([
		['gmail', 10],
		['icloud', 20],
		['imap', 30],
		['telegram', 40],
		['whatsapp_web', 50]
	]);
	return (priority.get(a.providerKind) ?? 100) - (priority.get(b.providerKind) ?? 100);
}
```

- [ ] **Step 4: Run the integration view-model tests and verify they pass**

Run:

```bash
cd frontend && pnpm test:unit -- src/lib/services/integrations.test.ts
```

Expected: PASS for all tests in `integrations.test.ts`.

- [ ] **Step 5: Commit Task 1**

Run:

```bash
git add frontend/src/lib/services/integrations.ts frontend/src/lib/services/integrations.test.ts
git commit -m "Add settings integration view models"
```

Expected: commit includes only the new integration service and tests.

---

### Task 2: Expose Integrations From Settings Store

**Files:**
- Modify: `frontend/src/lib/stores/settings.ts`
- Modify: `frontend/src/lib/stores/settings.test.ts`

- [ ] **Step 1: Write the failing settings store assertion**

In `frontend/src/lib/stores/settings.test.ts`, add assertions inside the existing `loads workspace settings and synchronizes shell stores` test after the `contactsProviderAccounts` expectation:

```ts
		expect(get(settingsStore.integrationViewModels).map((integration) => integration.integrationId)).toEqual([
			'gmail:gmail-primary',
			'icloud:icloud-primary',
			'telegram',
			'whatsapp'
		]);
		expect(get(settingsStore.integrationViewModels)[0].services.map((service) => service.state)).toEqual([
			'ready',
			'ready',
			'ready',
			'not_applicable'
		]);
```

- [ ] **Step 2: Run the store test and verify it fails**

Run:

```bash
cd frontend && pnpm test:unit -- src/lib/stores/settings.test.ts
```

Expected: FAIL because `integrationViewModels` is not exported.

- [ ] **Step 3: Add the derived integration store**

Modify `frontend/src/lib/stores/settings.ts`:

```ts
import { buildIntegrationViewModels } from '$lib/services/integrations';
```

Change the section type:

```ts
export type SettingsSection = 'application' | 'language' | 'appearance' | 'sidebar' | 'integrations';
```

Keep the default selected section:

```ts
export const selectedSettingsSection = writable<SettingsSection>('appearance');
```

Add the derived store after `contactsProviderAccounts`:

```ts
export const integrationViewModels = derived(
	[providerAccounts, calendarAccounts],
	([$providerAccounts, $calendarAccounts]) =>
		buildIntegrationViewModels($providerAccounts, $calendarAccounts)
);
```

- [ ] **Step 4: Update references from `accounts` section to `integrations`**

Search:

```bash
rg -n "selectedSettingsSection|=== 'accounts'|set\\('accounts'\\)|SettingsSection" frontend/src/lib frontend/src/routes
```

Expected files to update:

- `frontend/src/lib/pages/settings/SettingsPage.svelte`
- `frontend/src/lib/stores/settings.ts`

Do not update unrelated account setup terms; the product still has provider accounts.

- [ ] **Step 5: Run the store test and verify it passes**

Run:

```bash
cd frontend && pnpm test:unit -- src/lib/stores/settings.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit Task 2**

Run:

```bash
git add frontend/src/lib/stores/settings.ts frontend/src/lib/stores/settings.test.ts
git commit -m "Expose settings integrations store"
```

Expected: commit includes only settings store/test changes.

---

### Task 3: Build Integrations Settings Widget

**Files:**
- Create: `frontend/src/lib/pages/settings/widgets/IntegrationsSettings.svelte`

- [ ] **Step 1: Create the component with table and inspector**

Create `frontend/src/lib/pages/settings/widgets/IntegrationsSettings.svelte`:

```svelte
<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import {
		integrationStatusLabel,
		serviceStateLabel,
		type IntegrationService,
		type IntegrationViewModel
	} from '$lib/services/integrations';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		integrations: IntegrationViewModel[];
		selectedIntegrationId: string | null;
		onSelectIntegration: (integrationId: string) => void;
		onOpenAccountDrawer: (target?: string) => void;
		formatDateTimeFn: (value: string | null) => string;
	}

	let {
		integrations,
		selectedIntegrationId,
		onSelectIntegration,
		onOpenAccountDrawer,
		formatDateTimeFn
	}: Props = $props();

	const tableServices = ['mail', 'calendar', 'people'] as const;

	let selectedIntegration = $derived(
		integrations.find((integration) => integration.integrationId === selectedIntegrationId) ??
			integrations[0] ??
			null
	);

	function serviceFor(
		integration: IntegrationViewModel,
		serviceId: (typeof tableServices)[number]
	): IntegrationService | null {
		return integration.services.find((service) => service.id === serviceId) ?? null;
	}

	function serviceClass(service: IntegrationService | null): string {
		if (!service) return 'off';
		if (service.state === 'ready') return 'ready';
		if (service.state === 'not_applicable') return 'off';
		return 'warn';
	}

	function statusClass(integration: IntegrationViewModel): string {
		if (integration.status === 'connected') return 'ready';
		if (integration.status === 'empty') return 'muted';
		return 'warn';
	}

	function integrationActionTarget(integration: IntegrationViewModel | null): string {
		if (!integration) return 'mail';
		if (integration.providerKind === 'telegram') return 'telegram';
		if (integration.providerKind === 'whatsapp_web') return 'whatsapp';
		if (['gmail', 'icloud', 'imap'].includes(integration.providerKind)) return integration.providerKind;
		return 'mail';
	}
</script>

<div class="settings-integrations-layout">
	<section class="settings-integrations-main">
		<header class="settings-workbench-header">
			<div>
				<h2>{_('Integrations')}</h2>
				<p>{_('Connected providers, service coverage and account-level actions.')}</p>
			</div>
			<button type="button" class="primary-button" onclick={() => onOpenAccountDrawer('mail')}>
				<Icon icon="tabler:plus" width="16" height="16" />{_('Add integration')}
			</button>
		</header>

		{#if integrations.length === 0}
			<div class="empty-panel fill">{_('No integrations configured.')}</div>
		{:else}
			<div class="integrations-table" role="table" aria-label={_('Integrations')}>
				<div class="integrations-table-head" role="row">
					<span>{_('Integration')}</span>
					<span>{_('Mail')}</span>
					<span>{_('Calendar')}</span>
					<span>{_('People')}</span>
					<span>{_('Updated')}</span>
					<span>{_('Status')}</span>
				</div>
				{#each integrations as integration}
					<button
						type="button"
						class="integrations-table-row"
						class:selected={selectedIntegration?.integrationId === integration.integrationId}
						role="row"
						onclick={() => onSelectIntegration(integration.integrationId)}
					>
						<span class="integration-primary-cell">
							<span class="round-icon cyan">
								<Icon icon={integration.icon} width="20" height="20" />
							</span>
							<span>
								<strong>{integration.title}</strong>
								<small>{integration.subtitle}</small>
							</span>
						</span>
						{#each tableServices as serviceId}
							{@const service = serviceFor(integration, serviceId)}
							<span class={`integration-service-state ${serviceClass(service)}`}>
								{_(serviceStateLabel(service?.state ?? 'not_applicable'))}
							</span>
						{/each}
						<span class="integration-updated">{formatDateTimeFn(integration.updatedAt)}</span>
						<span class={`integration-status ${statusClass(integration)}`}>
							{_(integrationStatusLabel(integration.status))}
						</span>
					</button>
				{/each}
			</div>
		{/if}
	</section>

	<aside class="settings-integration-inspector">
		{#if selectedIntegration}
			<header>
				<h3>{selectedIntegration.title}</h3>
				<p>{selectedIntegration.subtitle}</p>
			</header>

			<section class="integration-inspector-section">
				<h4>{_('Services')}</h4>
				{#each selectedIntegration.services as service}
					<div class="integration-service-line">
						<div>
							<strong>{_(service.label)}</strong>
							<small>{_(service.description)}</small>
						</div>
						<span class={`integration-service-state ${serviceClass(service)}`}>
							{_(serviceStateLabel(service.state))}
						</span>
					</div>
				{/each}
			</section>

			<section class="integration-inspector-section">
				<h4>{_('Actions')}</h4>
				<div class="integration-action-stack">
					<button type="button" class="primary-button" onclick={() => onOpenAccountDrawer(integrationActionTarget(selectedIntegration))}>
						{_('Reconnect')}
					</button>
					<button type="button" class="ghost-button" disabled>{_('Run sync now')}</button>
					<button type="button" class="ghost-button" disabled>{_('View vault binding')}</button>
					<button type="button" class="danger-button" disabled>{_('Remove integration')}</button>
				</div>
			</section>

			<section class="integration-inspector-section">
				<h4>{_('Metadata')}</h4>
				<ul class="integration-metadata-list">
					{#each Object.entries(selectedIntegration.metadata) as [label, value]}
						<li><span>{_(label)}</span><code>{value}</code></li>
					{/each}
				</ul>
			</section>
		{:else}
			<div class="empty-panel fill">{_('Select an integration.')}</div>
		{/if}
	</aside>
</div>
```

- [ ] **Step 2: Run Svelte check and capture expected styling failures**

Run:

```bash
cd frontend && pnpm lint:ts
```

Expected: PASS for TypeScript/Svelte syntax. CSS classes are not styled yet, so the app will still look unfinished until Task 5.

- [ ] **Step 3: Commit Task 3**

Run:

```bash
git add frontend/src/lib/pages/settings/widgets/IntegrationsSettings.svelte
git commit -m "Add settings integrations widget"
```

Expected: commit includes only the new Svelte component.

---

### Task 4: Replace Settings Tabs With IDE Tree

**Files:**
- Modify: `frontend/src/lib/pages/settings/SettingsPage.svelte`

- [ ] **Step 1: Update imports and local state**

In `frontend/src/lib/pages/settings/SettingsPage.svelte`, replace:

```ts
import AccountsSettings from './widgets/AccountsSettings.svelte';
```

with:

```ts
import IntegrationsSettings from './widgets/IntegrationsSettings.svelte';
```

Add `integrationViewModels` to the store import list:

```ts
integrationViewModels,
```

Add local selected integration state after the `openAccountWizard` function:

```ts
let selectedIntegrationId = $state<string | null>(null);

const settingsTreeGroups = [
	{
		label: 'General',
		items: [
			{ id: 'application', label: 'Application', icon: 'tabler:adjustments-horizontal' },
			{ id: 'language', label: 'Language', icon: 'tabler:language' }
		]
	},
	{
		label: 'Interface',
		items: [
			{ id: 'appearance', label: 'Appearance', icon: 'tabler:palette' },
			{ id: 'sidebar', label: 'Sidebar', icon: 'tabler:layout-sidebar' }
		]
	},
	{
		label: 'Sources',
		items: [
			{ id: 'integrations', label: 'Integrations', icon: 'tabler:plug-connected' }
		]
	}
] as const;
```

Add the derived selected integration effect:

```ts
$effect(() => {
	if ($integrationViewModels.length === 0) {
		selectedIntegrationId = null;
		return;
	}
	if (!selectedIntegrationId || !$integrationViewModels.some((item) => item.integrationId === selectedIntegrationId)) {
		selectedIntegrationId = $integrationViewModels[0].integrationId;
	}
});

function selectIntegration(integrationId: string) {
	selectedIntegrationId = integrationId;
}
```

- [ ] **Step 2: Replace horizontal Settings tabs markup**

Delete the current `<div class="section-tabs settings-tabs" ...>` block.

Wrap the section content in:

```svelte
<div class="settings-workbench">
	<nav class="settings-tree" aria-label={_('Settings sections')}>
		{#each settingsTreeGroups as group}
			<section class="settings-tree-group">
				<h2>{_(group.label)}</h2>
				{#each group.items as item}
					<button
						type="button"
						class:active={$selectedSettingsSection === item.id}
						onclick={() => selectedSettingsSection.set(item.id)}
					>
						<Icon icon={item.icon} width="16" height="16" />
						<span>{_(item.label)}</span>
						{#if item.id === 'integrations'}
							<em>{$integrationViewModels.length}</em>
						{/if}
					</button>
				{/each}
			</section>
		{/each}
	</nav>

	<div class="settings-workbench-content">
		<!-- move the existing selected section conditional here -->
	</div>
</div>
```

- [ ] **Step 3: Replace AccountsSettings branch**

Replace the final `{:else}` branch that renders `AccountsSettings` with:

```svelte
{:else}
	<IntegrationsSettings
		integrations={$integrationViewModels}
		{selectedIntegrationId}
		onSelectIntegration={selectIntegration}
		onOpenAccountDrawer={openAccountWizard}
		formatDateTimeFn={formatDateTime}
	/>
{/if}
```

The previous props for raw account groups are no longer needed in `SettingsPage.svelte`. Remove unused imports after `pnpm lint:ts` reports them.

- [ ] **Step 4: Run Svelte check and fix compile errors**

Run:

```bash
cd frontend && pnpm lint:ts
```

Expected: PASS. If it fails with unused imports from `$lib/stores/settings`, remove only the unused account grouping imports from `SettingsPage.svelte`; do not remove store exports used by other pages.

- [ ] **Step 5: Commit Task 4**

Run:

```bash
git add frontend/src/lib/pages/settings/SettingsPage.svelte
git commit -m "Convert settings page to IDE workbench"
```

Expected: commit includes only Settings page wiring.

---

### Task 5: Add Settings Workbench CSS

**Files:**
- Modify: `frontend/src/lib/pages/pages.css`

- [ ] **Step 1: Add scoped CSS for the workbench**

In `frontend/src/lib/pages/pages.css`, replace the `.settings-tabs` and `.settings-account-layout` dependent layout behavior with a workbench layout. Keep existing `.settings-layout` styles for Application, Appearance, Sidebar and Language content.

Add after the existing `.settings-page` block:

```css
.settings-workbench {
	display: grid;
	grid-template-columns: 220px minmax(0, 1fr);
	gap: var(--hh-layout-gap);
	flex: 1 1 auto;
	min-width: 0;
	min-height: 0;
	height: 100%;
	overflow: hidden;
}

.settings-tree {
	display: grid;
	align-content: start;
	gap: 14px;
	min-width: 0;
	min-height: 0;
	overflow-x: hidden;
	overflow-y: auto;
	border: 1px solid var(--hh-border-muted);
	border-radius: var(--hh-radius-md);
	background: rgba(4, 18, 20, 0.72);
	padding: 12px 8px;
	scrollbar-color: rgba(45, 240, 206, 0.25) transparent;
}

.settings-tree-group {
	display: grid;
	gap: 5px;
}

.settings-tree-group h2 {
	color: var(--hh-color-text-muted);
	font-size: 10px;
	font-weight: 760;
	text-transform: uppercase;
}

.settings-tree button {
	display: grid;
	grid-template-columns: 18px minmax(0, 1fr) auto;
	align-items: center;
	gap: 8px;
	min-height: 32px;
	border: 1px solid transparent;
	border-radius: var(--hh-radius-control);
	background: transparent;
	color: var(--hh-color-text-soft);
	font-size: 12px;
	font-weight: 650;
	padding: 0 8px;
	text-align: left;
}

.settings-tree button:hover,
.settings-tree button:focus-visible {
	border-color: var(--hh-border-accent-soft);
	background: rgba(45, 240, 206, 0.06);
}

.settings-tree button.active {
	border-color: var(--hh-border-accent);
	background: var(--hh-accent-tint);
	color: var(--hh-color-accent);
}

.settings-tree button span {
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.settings-tree button em {
	border-radius: var(--hh-radius-pill);
	background: rgba(124, 156, 156, 0.14);
	color: var(--hh-color-text-muted);
	font-size: 10px;
	font-style: normal;
	padding: 1px 7px;
}

.settings-workbench-content {
	min-width: 0;
	min-height: 0;
	overflow: hidden;
}
```

- [ ] **Step 2: Add Integrations table and inspector CSS**

Add near the existing account styles:

```css
.settings-integrations-layout {
	display: grid;
	grid-template-columns: minmax(0, 1fr) minmax(280px, 320px);
	gap: var(--hh-layout-gap);
	width: 100%;
	min-width: 0;
	height: 100%;
	min-height: 0;
	overflow: hidden;
}

.settings-integrations-main,
.settings-integration-inspector {
	min-width: 0;
	min-height: 0;
	border: 1px solid var(--hh-border-muted);
	border-radius: var(--hh-radius-md);
	background: rgba(4, 21, 24, 0.56);
	overflow: hidden;
}

.settings-integrations-main {
	display: grid;
	grid-template-rows: auto minmax(0, 1fr);
}

.settings-workbench-header {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 12px;
	min-height: 58px;
	border-bottom: 1px solid var(--hh-border-muted);
	padding: 0 14px;
}

.settings-workbench-header h2,
.settings-integration-inspector h3 {
	color: var(--hh-color-text);
	font-size: 16px;
	font-weight: 680;
}

.settings-workbench-header p,
.settings-integration-inspector header p {
	margin-top: 4px;
	color: var(--hh-color-text-muted);
	font-size: 12px;
}

.integrations-table {
	min-height: 0;
	overflow-x: hidden;
	overflow-y: auto;
	scrollbar-color: rgba(45, 240, 206, 0.25) transparent;
}

.integrations-table-head,
.integrations-table-row {
	display: grid;
	grid-template-columns: minmax(220px, 1.3fr) 76px 76px 76px 92px 104px;
	align-items: center;
	gap: 0;
	width: 100%;
	min-width: 0;
	border-bottom: 1px solid rgba(102, 189, 180, 0.08);
}

.integrations-table-head {
	min-height: 34px;
	background: rgba(3, 16, 18, 0.72);
	color: var(--hh-color-text-muted);
	font-size: 10px;
	font-weight: 760;
	text-transform: uppercase;
}

.integrations-table-head span,
.integrations-table-row > span {
	min-width: 0;
	padding: 0 10px;
}

.integrations-table-row {
	min-height: 62px;
	background: transparent;
	color: var(--hh-color-text-soft);
	text-align: left;
}

.integrations-table-row:hover,
.integrations-table-row:focus-visible {
	background: rgba(45, 240, 206, 0.05);
}

.integrations-table-row.selected {
	background: rgba(45, 240, 206, 0.07);
	box-shadow: inset 3px 0 0 var(--hh-color-accent);
}

.integration-primary-cell {
	display: grid;
	grid-template-columns: 36px minmax(0, 1fr);
	align-items: center;
	gap: 10px;
}

.integration-primary-cell strong,
.integration-primary-cell small {
	display: block;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.integration-primary-cell strong {
	color: var(--hh-color-text);
	font-size: 13px;
}

.integration-primary-cell small,
.integration-updated {
	color: var(--hh-color-text-muted);
	font-size: 11px;
}

.integration-service-state,
.integration-status {
	display: inline-flex;
	justify-content: center;
	max-width: 72px;
	border-radius: var(--hh-radius-sm);
	font-size: 11px;
	font-weight: 720;
	line-height: 22px;
	padding: 0 8px;
	white-space: nowrap;
}

.integration-service-state.ready,
.integration-status.ready {
	background: var(--hh-status-success-surface);
	color: var(--hh-status-success-text);
}

.integration-service-state.warn,
.integration-status.warn {
	background: var(--hh-status-warning-surface);
	color: var(--hh-status-warning-text);
}

.integration-service-state.off,
.integration-status.muted {
	background: var(--hh-status-neutral-surface);
	color: var(--hh-status-neutral-text);
}

.settings-integration-inspector {
	display: grid;
	align-content: start;
	gap: 16px;
	overflow-x: hidden;
	overflow-y: auto;
	padding: 14px;
	scrollbar-color: rgba(45, 240, 206, 0.25) transparent;
}

.integration-inspector-section {
	display: grid;
	gap: 8px;
}

.integration-inspector-section h4 {
	color: var(--hh-color-text-soft);
	font-size: 12px;
	font-weight: 720;
}

.integration-service-line {
	display: grid;
	grid-template-columns: minmax(0, 1fr) auto;
	align-items: center;
	gap: 10px;
	min-height: 34px;
	border-bottom: 1px solid rgba(102, 189, 180, 0.08);
}

.integration-service-line strong,
.integration-service-line small {
	display: block;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.integration-service-line strong {
	color: var(--hh-color-text);
	font-size: 12px;
}

.integration-service-line small {
	color: var(--hh-color-text-muted);
	font-size: 10px;
}

.integration-action-stack {
	display: grid;
	gap: 8px;
}

.integration-action-stack button {
	min-height: 32px;
	border-radius: var(--hh-radius-control);
	font-size: 12px;
	font-weight: 720;
}

.danger-button {
	border: 1px solid rgba(255, 171, 171, 0.24);
	background: var(--hh-danger-tint);
	color: var(--hh-color-danger);
}

.integration-metadata-list {
	display: grid;
	gap: 6px;
	margin: 0;
	padding: 0;
	list-style: none;
}

.integration-metadata-list li {
	display: grid;
	grid-template-columns: minmax(0, 0.72fr) minmax(0, 1fr);
	gap: 8px;
	align-items: center;
	color: var(--hh-color-text-muted);
	font-size: 11px;
}

.integration-metadata-list code {
	overflow: hidden;
	border: 1px solid rgba(111, 205, 195, 0.1);
	border-radius: var(--hh-radius-pill);
	background: rgba(2, 9, 11, 0.56);
	color: #8fece1;
	font-size: 10px;
	padding: 3px 7px;
	text-overflow: ellipsis;
	white-space: nowrap;
}
```

- [ ] **Step 3: Add desktop resize behavior**

Add to the existing `@media (max-width: 1359px)` block:

```css
.integrations-table-head,
.integrations-table-row {
	grid-template-columns: minmax(190px, 1.2fr) 68px 68px 68px 82px 92px;
}
```

Add to the existing `@media (max-width: 900px)` block, even though mobile is not supported, to keep narrow desktop windows usable:

```css
.settings-workbench {
	grid-template-columns: 180px minmax(0, 1fr);
}

.settings-integrations-layout {
	grid-template-columns: minmax(0, 1fr);
	overflow-x: hidden;
	overflow-y: auto;
}

.settings-integration-inspector {
	min-height: 260px;
}
```

- [ ] **Step 4: Run style and Svelte checks**

Run:

```bash
cd frontend && pnpm lint
```

Expected: PASS. If `check-no-inline-styles.mjs` fails, remove any inline `style=` accidentally introduced in Svelte files.

- [ ] **Step 5: Commit Task 5**

Run:

```bash
git add frontend/src/lib/pages/pages.css
git commit -m "Style settings integrations workbench"
```

Expected: commit includes only CSS changes.

---

### Task 6: Add i18n Coverage

**Files:**
- Modify: `frontend/src/lib/i18n/ru.json`

- [ ] **Step 1: Add Russian translations for new Settings strings**

Add these entries to `frontend/src/lib/i18n/ru.json`, keeping JSON sorted near related Settings strings where practical:

```json
	"Actions": "Действия",
	"Add integration": "Добавить интеграцию",
	"Advanced": "Дополнительно",
	"Auth": "Авторизация",
	"Connected": "Подключено",
	"Connected providers, service coverage and account-level actions.": "Подключённые провайдеры, покрытие сервисов и действия на уровне аккаунта.",
	"Developer": "Разработчик",
	"Disabled": "Отключено",
	"Empty": "Пусто",
	"External ID": "Внешний ID",
	"Integration": "Интеграция",
	"Integrations": "Интеграции",
	"Need action": "Требует действия",
	"No account configured": "Аккаунт не настроен",
	"No integrations configured.": "Интеграции не настроены.",
	"Partial": "Частично",
	"Reconnect": "Переподключить",
	"Remove integration": "Удалить интеграцию",
	"Run sync now": "Запустить синхронизацию",
	"Select an integration.": "Выберите интеграцию.",
	"Services": "Сервисы",
	"Sync": "Синхронизация",
	"Updated": "Обновлено",
	"Vault": "Хранилище",
	"View vault binding": "Показать привязку хранилища"
```

If a key already exists, do not duplicate it. Keep valid JSON commas.

- [ ] **Step 2: Run frontend check**

Run:

```bash
cd frontend && pnpm check
```

Expected: PASS for style, Svelte/TypeScript and Vitest tests.

- [ ] **Step 3: Commit Task 6**

Run:

```bash
git add frontend/src/lib/i18n/ru.json
git commit -m "Add settings integrations translations"
```

Expected: commit includes only translation changes.

---

### Task 7: Remove Dead Account Dashboard Wiring

**Files:**
- Modify or delete: `frontend/src/lib/pages/settings/widgets/AccountsSettings.svelte`
- Modify: `frontend/src/lib/pages/settings/SettingsPage.svelte`
- Modify: `frontend/src/lib/pages/pages.css`

- [ ] **Step 1: Search for remaining references**

Run:

```bash
rg -n "AccountsSettings|settings-account-layout|account-section|account-card-grid|account-card" frontend/src
```

Expected: references should be limited to `AccountsSettings.svelte` and obsolete CSS after Tasks 3-5.

- [ ] **Step 2: Delete the old AccountsSettings component when unreferenced**

If `AccountsSettings.svelte` has no imports or route references, delete it:

```bash
rm frontend/src/lib/pages/settings/widgets/AccountsSettings.svelte
```

If repository policy prefers non-shell deletion through patch tooling, remove the file with the patch tool instead of `rm`.

- [ ] **Step 3: Remove obsolete account dashboard CSS**

From `frontend/src/lib/pages/pages.css`, remove selectors that only supported the old card dashboard and are no longer referenced:

```css
.settings-account-layout { ... }
.account-section { ... }
.account-card-grid { ... }
.account-card { ... }
.account-card div { ... }
.account-card strong { ... }
.account-card p,
.account-card small { ... }
.account-card p { ... }
.account-card small { ... }
.account-card code { ... }
```

Do not remove shared styles still used elsewhere, such as `.setting-copy code`, `.round-icon`, `.empty-panel`, `.primary-button` or `.panel-title-row`.

- [ ] **Step 4: Run reference search again**

Run:

```bash
rg -n "AccountsSettings|settings-account-layout|account-section|account-card-grid|account-card" frontend/src
```

Expected: no matches, or only matches intentionally retained for shared non-settings UI.

- [ ] **Step 5: Run frontend check**

Run:

```bash
cd frontend && pnpm check
```

Expected: PASS.

- [ ] **Step 6: Commit Task 7**

Run:

```bash
git add frontend/src/lib/pages/settings/widgets/AccountsSettings.svelte frontend/src/lib/pages/pages.css frontend/src/lib/pages/settings/SettingsPage.svelte
git commit -m "Remove old settings account cards"
```

Expected: commit removes old account dashboard component/CSS and keeps new Integrations UI.

---

### Task 8: Live Smoke And Final Validation

**Files:**
- No expected source edits unless smoke reveals a bug.

- [ ] **Step 1: Run full frontend validation**

Run:

```bash
cd frontend && pnpm check
```

Expected: PASS.

- [ ] **Step 2: Run diff whitespace validation**

Run:

```bash
git diff --check
```

Expected: no output and exit code 0.

- [ ] **Step 3: Start or reuse dev stack**

If `make dev` is already running, reuse it. Otherwise run:

```bash
make dev
```

Expected:

- backend available on `http://127.0.0.1:8080`;
- frontend available on `http://127.0.0.1:5174`.

- [ ] **Step 4: Manual browser smoke**

Open `http://127.0.0.1:5174`, navigate to Settings, and verify:

- left Settings tree is visible;
- `Sources -> Integrations` shows count and opens the integrations workbench;
- Google/iCloud rows appear once each, not repeated as Mail/Calendar/Contacts cards;
- Mail, Calendar and People service columns render;
- selecting a row changes the inspector;
- `Add integration` opens the existing account setup modal at provider selection;
- no text overlaps at the current desktop viewport;
- browser console has no new Svelte/runtime errors.

- [ ] **Step 5: Inspect final staged scope**

Run:

```bash
git status --short
```

Expected:

- frontend changes from this implementation are present;
- unrelated backend WIP remains unstaged if it existed before;
- `.superpowers/` remains ignored.

- [ ] **Step 6: Final commit**

Run:

```bash
git add frontend/src/lib/services/integrations.ts \
	frontend/src/lib/services/integrations.test.ts \
	frontend/src/lib/stores/settings.ts \
	frontend/src/lib/stores/settings.test.ts \
	frontend/src/lib/pages/settings/SettingsPage.svelte \
	frontend/src/lib/pages/settings/widgets/IntegrationsSettings.svelte \
	frontend/src/lib/pages/settings/widgets/AccountsSettings.svelte \
	frontend/src/lib/pages/pages.css \
	frontend/src/lib/i18n/ru.json
git commit -m "Redesign settings integrations workbench"
```

Expected: final commit contains only frontend Settings redesign changes. If Task 7 already deleted `AccountsSettings.svelte`, `git add` stages that deletion.

---

## Self-Review Checklist

- Spec coverage:
  - IDE-style tree: Tasks 4 and 5.
  - Integrations as primary object: Tasks 1, 3 and 4.
  - Service columns instead of badges: Tasks 3 and 5.
  - Inspector actions and metadata: Task 3.
  - Frontend-only view model: Tasks 1 and 2.
  - i18n: Task 6.
  - Old duplicate cards removed: Task 7.
  - Validation and smoke: Task 8.

- Scope guard:
  - No backend files are part of this plan.
  - No new sync backend behavior is introduced.
  - No mobile UI validation is required.

- Type consistency:
  - `IntegrationServiceState`, `IntegrationStatus`, `IntegrationService` and `IntegrationViewModel` are introduced in Task 1 and reused by Tasks 2 and 3.
  - Store export is `integrationViewModels`.
  - Selected Settings section is `integrations`, not `accounts`.
