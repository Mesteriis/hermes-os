<script lang="ts">
	import './sidebar.css';
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import { currentView, activeCommunicationSection, isSidebarRail, activeSidebarRailGroupId } from '$lib/stores/navigation';
	import type { ResolvedSidebarItem, ResolvedSidebarRootEntry, SidebarNavGroup, SidebarItemId, PrimaryNavId } from '$lib/layout/sidebar-navigation';

	const _ = (key: string) => t($currentLocale, key);

	type NavItem = { id: PrimaryNavId; label: string; icon: string; badge?: string; enabled: boolean };

	interface Props {
		sidebarRootEntries: ResolvedSidebarRootEntry<NavItem>[];
		expandedSidebarGroupIds: string[];
		onSelectItem: (item: ResolvedSidebarItem<NavItem>) => void;
		onToggleGroup: (group: SidebarNavGroup) => void;
		onToggleRail: () => void;
		onSettings: () => void;
	}

	let {
		sidebarRootEntries,
		expandedSidebarGroupIds,
		onSelectItem,
		onToggleGroup,
		onToggleRail,
		onSettings
	}: Props = $props();

	function sidebarGroupLabel(group: SidebarNavGroup, index: number) {
		return group.label || (group.id === 'communications' ? 'Communications' : `Group ${index + 1}`);
	}

	function sidebarItemLabel(item: ResolvedSidebarItem<NavItem>) {
		return item.kind === 'primary' ? item.primary.label : item.section.label;
	}

	function sidebarItemIcon(item: ResolvedSidebarItem<NavItem>) {
		return item.kind === 'primary' ? item.primary.icon : item.section.icon;
	}

	function sidebarItemBadge(item: ResolvedSidebarItem<NavItem>) {
		return item.kind === 'primary' ? item.primary.badge : undefined;
	}

	function sidebarItemTitle(item: ResolvedSidebarItem<NavItem>) {
		return item.kind === 'primary' && !item.primary.enabled
			? `${item.primary.label} is not available in the current desktop scope`
			: sidebarItemLabel(item);
	}

	function isSidebarItemDisabled(item: ResolvedSidebarItem<NavItem>) {
		return item.kind === 'primary' && !item.primary.enabled;
	}

	function isSidebarItemActive(item: ResolvedSidebarItem<NavItem>) {
		return item.kind === 'primary'
			? $currentView === item.primary.id
			: $currentView === 'communications' && $activeCommunicationSection === item.section.id;
	}

	function isSidebarGroupExpanded(groupId: string) {
		return expandedSidebarGroupIds.includes(groupId);
	}

	function sidebarGroupHasActiveItem(group: SidebarNavGroup, $cv: string, $acs: string) {
		return group.itemIds.some((itemId) => {
			const communicationSectionId = itemId.startsWith('communications.') ? itemId.replace('communications.', '') : null;
			if (communicationSectionId) {
				return $cv === 'communications' && $acs === communicationSectionId;
			}
			return $cv === itemId;
		});
	}

	function sidebarGroupHasSeparatorBefore(group: SidebarNavGroup, itemId: SidebarItemId) {
		return group.itemIds.indexOf(itemId) > 0 && group.separatorBeforeItemIds.includes(itemId);
	}
</script>

<aside class="sidebar" class:rail={$isSidebarRail} aria-label="Hermes Hub navigation">
	<div class="brand">
		<button
			type="button"
			class="brand-mark-button"
			aria-label={$isSidebarRail ? 'Expand sidebar' : 'Collapse sidebar'}
			aria-pressed={$isSidebarRail}
			title={$isSidebarRail ? 'Expand sidebar' : 'Collapse sidebar'}
			onclick={onToggleRail}
		>
			<img src="/assets/hermes-logo-mark.png" alt="" class="brand-mark" />
		</button>
		<div class="brand-copy">
			<p class="brand-name">{_('Hermes Hub')}</p>
			<p class="brand-subtitle">{_('Personal OS')}</p>
		</div>
	</div>

	<nav class="nav-group primary-nav" aria-label="Primary workspaces">
		{#each sidebarRootEntries as entry, entryIndex}
			{#if entry.kind === 'item'}
				{@const item = entry.item}
				<div class="nav-entry">
					<button
						type="button"
						class:active={isSidebarItemActive(item)}
						class:disabled={isSidebarItemDisabled(item)}
						disabled={isSidebarItemDisabled(item)}
						aria-current={isSidebarItemActive(item) ? 'page' : undefined}
						title={sidebarItemTitle(item)}
						onclick={() => onSelectItem(item)}
					>
						<Icon icon={sidebarItemIcon(item)} width="18" height="18" />
						<span>{_(sidebarItemLabel(item))}</span>
						{#if sidebarItemBadge(item)}
							<em>{sidebarItemBadge(item)}</em>
						{/if}
					</button>
				</div>
			{:else}
				{@const group = entry.group}
				<div class="nav-entry">
					<button
						type="button"
						class:active={sidebarGroupHasActiveItem(group, $currentView, $activeCommunicationSection)}
						class:has-subnav={true}
						aria-current={sidebarGroupHasActiveItem(group, $currentView, $activeCommunicationSection) ? 'page' : undefined}
						aria-expanded={$isSidebarRail ? $activeSidebarRailGroupId === group.id : isSidebarGroupExpanded(group.id)}
						aria-controls={`sidebar-group-${group.id}-sections`}
						aria-haspopup={$isSidebarRail ? 'menu' : undefined}
						title={_(sidebarGroupLabel(group, entryIndex))}
						onclick={() => onToggleGroup(group)}
					>
						<Icon icon={group.icon} width="18" height="18" />
						<span>{_(sidebarGroupLabel(group, entryIndex))}</span>
						{#if !$isSidebarRail}
							<Icon class="nav-disclosure" icon={isSidebarGroupExpanded(group.id) ? 'tabler:chevron-up' : 'tabler:chevron-down'} width="15" height="15" />
						{/if}
					</button>
					{#if $activeSidebarRailGroupId === group.id && $isSidebarRail}
						<div
							id={`sidebar-group-${group.id}-sections`}
							class="communications-rail-dropdown"
							aria-label={`${_(sidebarGroupLabel(group, entryIndex))} sections`}
						>
							{#each group.items as item}
								{#if sidebarGroupHasSeparatorBefore(group, item.itemId)}
									<div class="subnav-separator" aria-hidden="true"></div>
								{/if}
								<button
									type="button"
									class="subnav-item"
									class:active={isSidebarItemActive(item)}
									aria-current={isSidebarItemActive(item) ? 'page' : undefined}
									title={sidebarItemTitle(item)}
									disabled={isSidebarItemDisabled(item)}
									onclick={() => onSelectItem(item)}
								>
									<Icon icon={sidebarItemIcon(item)} width="16" height="16" />
									<span>{_(sidebarItemLabel(item))}</span>
									{#if sidebarItemBadge(item)}
										<em>{sidebarItemBadge(item)}</em>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
					{#if isSidebarGroupExpanded(group.id) && !$isSidebarRail}
						<div
							id={`sidebar-group-${group.id}-sections`}
							class="communications-subnav"
							aria-label={`${_(sidebarGroupLabel(group, entryIndex))} sections`}
						>
							{#each group.items as item}
								{#if sidebarGroupHasSeparatorBefore(group, item.itemId)}
									<div class="subnav-separator" aria-hidden="true"></div>
								{/if}
								<button
									type="button"
									class="subnav-item"
									class:active={isSidebarItemActive(item)}
									aria-current={isSidebarItemActive(item) ? 'page' : undefined}
									title={sidebarItemTitle(item)}
									disabled={isSidebarItemDisabled(item)}
									onclick={() => onSelectItem(item)}
								>
									<Icon icon={sidebarItemIcon(item)} width="16" height="16" />
									<span>{_(sidebarItemLabel(item))}</span>
									{#if sidebarItemBadge(item)}
										<em>{sidebarItemBadge(item)}</em>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
				</div>
			{/if}
		{/each}
	</nav>

	<div class="sidebar-tools" aria-label="System navigation">
		<button type="button" class="settings-link" class:active={$currentView === 'settings'} title="Open settings" onclick={onSettings}>
			<Icon icon="tabler:settings" width="18" height="18" />
			<span>{_('Settings')}</span>
		</button>
	</div>
</aside>
