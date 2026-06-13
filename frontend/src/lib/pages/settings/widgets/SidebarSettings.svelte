<script lang="ts">
	import Icon from '@iconify/svelte';
	import HermesSelect from '$lib/components/shared/HermesSelect.svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { SidebarSettings, SidebarNavGroup, SidebarItemId, SidebarRootItemId, ResolvedSidebarItem, ResolvedSidebarRootEntry, PrimaryNavId } from '$lib/layout';
	import { parseCommunicationSidebarItemId } from '$lib/layout';
	import './sidebarSettings.css';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		sidebarError: string;
		isSidebarSettingsSaving: boolean;
		newSidebarGroupLabel: string;
		sidebarRootEntries: ResolvedSidebarRootEntry<{ id: PrimaryNavId; label: string; icon: string; badge?: string; enabled: boolean }>[];
		sidebarHiddenNavItems: Array<{ id: SidebarItemId; label: string; icon: string }>;

		effectiveSidebarSettings: SidebarSettings;
		hasSidebarChanges: boolean;

		onCancelSidebarEditing: () => void;
		onResetSidebar: () => void;
		onSaveSidebar: () => Promise<void>;
		onAddSidebarGroup: () => void;
		onRemoveSidebarGroup: (groupId: string) => void;
		onMoveSidebarGroup: (groupId: string, direction: -1 | 1) => void;
		onMoveSidebarRootItem: (rootId: SidebarRootItemId, direction: -1 | 1) => void;
		onMoveSidebarItem: (itemId: SidebarItemId, direction: -1 | 1) => void;
		onMoveSidebarItemToGroup: (itemId: SidebarItemId, targetGroupId: string) => void;
		onToggleSidebarGroupSeparator: (groupId: string, itemId: SidebarItemId) => void;
		onToggleSidebarItemHidden: (itemId: SidebarItemId) => void;
		onUpdateSidebarGroupLabel: (groupId: string, label: string) => void;
		onUpdateNewSidebarGroupLabel: (label: string) => void;

		sidebarGroupLabelFn: (group: SidebarNavGroup, index: number) => string;
		sidebarItemLabelFn: (item: ResolvedSidebarItem<{ id: PrimaryNavId; label: string; icon: string; badge?: string; enabled: boolean }>) => string;
		sidebarGroupHasSeparatorBeforeFn: (group: SidebarNavGroup, itemId: SidebarItemId) => boolean;
		sidebarRootIndexForGroupFn: (groupId: string) => number;
		sidebarGroupIdFromLabelFn: (label: string) => string;
		sidebarConfigItemFn: (itemId: SidebarItemId) => { id: SidebarItemId; label: string; icon: string } | null;
		inputEventValueFn: (event: Event) => string;
	}

	let {
		sidebarError,
		isSidebarSettingsSaving,
		newSidebarGroupLabel,
		sidebarRootEntries,
		sidebarHiddenNavItems,
		effectiveSidebarSettings,
		hasSidebarChanges,
		onCancelSidebarEditing,
		onResetSidebar,
		onSaveSidebar,
		onAddSidebarGroup,
		onRemoveSidebarGroup,
		onMoveSidebarGroup,
		onMoveSidebarRootItem,
		onMoveSidebarItem,
		onMoveSidebarItemToGroup,
		onToggleSidebarGroupSeparator,
		onToggleSidebarItemHidden,
		onUpdateSidebarGroupLabel,
		onUpdateNewSidebarGroupLabel,
		sidebarGroupLabelFn,
		sidebarItemLabelFn,
		sidebarGroupHasSeparatorBeforeFn,
		sidebarRootIndexForGroupFn,
		sidebarGroupIdFromLabelFn,
		sidebarConfigItemFn,
		inputEventValueFn
	}: Props = $props();

	function sidebarMoveTargetOptions(includeRoot: boolean) {
		return [
			...(includeRoot ? [{ value: 'root', label: _('Root level') }] : []),
			...effectiveSidebarSettings.groups.map((group, index) => ({
				value: group.id,
				label: _(sidebarGroupLabelFn(group, index))
			}))
		];
	}
</script>

<div class="settings-layout sidebar-settings-layout">
	<section class="panel settings-list-panel settings-primary-pane sidebar-settings-panel">
		<header class="panel-title-row">
			<div>
				<h2>{_('Sidebar Navigation')}</h2>
				<p>{_('Configure workspace groups, order and hidden domains. Communications sources stay inside Communications.')}</p>
			</div>
			<div class="sidebar-settings-actions">
				<button type="button" onclick={onCancelSidebarEditing} disabled={!hasSidebarChanges || isSidebarSettingsSaving}>
					<Icon icon="tabler:arrow-back-up" width="16" height="16" />{_('Cancel')}
				</button>
				<button type="button" onclick={onResetSidebar} disabled={isSidebarSettingsSaving}>
					<Icon icon="tabler:restore" width="16" height="16" />{_('Default')}
				</button>
				<button type="button" class="primary-button" onclick={() => void onSaveSidebar()} disabled={!hasSidebarChanges || isSidebarSettingsSaving}>
					<Icon icon="tabler:device-floppy" width="16" height="16" />{isSidebarSettingsSaving ? _('Saving') : _('Save')}
				</button>
			</div>
		</header>

		{#if sidebarError}
			<p class="inline-error">{sidebarError}</p>
		{/if}

		<form class="sidebar-group-create" onsubmit={(event) => { event.preventDefault(); onAddSidebarGroup(); }}>
			<label>
				<span>{_('New group')}</span>
				<input value={newSidebarGroupLabel} placeholder={_('Focus, Library, Planning')} autocomplete="off" oninput={(event) => onUpdateNewSidebarGroupLabel(inputEventValueFn(event))} />
			</label>
			<button type="submit">
				<Icon icon="tabler:plus" width="16" height="16" />{_('Create Group')}
			</button>
		</form>

		<div class="sidebar-config-list">
			<section class="sidebar-config-group">
				<header>
					<label>
						<span>{_('Root level')}</span>
						<input value={_('Sidebar root')} disabled autocomplete="off" />
					</label>
				</header>
				<div class="sidebar-config-items">
					{#each effectiveSidebarSettings.rootItemIds as rootId, rootIndex}
						{@const groupId = sidebarGroupIdFromLabelFn(rootId)}
						{#if groupId}
							{@const group = effectiveSidebarSettings.groups.find((item) => item.id === groupId)}
							{#if group}
								<div class="sidebar-config-item group-node">
									<div class="sidebar-config-item-main">
										<span class="round-icon green"><Icon icon={group.icon} width="18" height="18" /></span>
										<div>
											<strong>{_(sidebarGroupLabelFn(group, rootIndex))}</strong>
											<small>{_('Expandable group')} · {group.itemIds.length} {_('items')}</small>
										</div>
									</div>
									<div class="sidebar-config-item-controls">
										<button type="button" aria-label={_('Move {label} up').replace('{label}', _(sidebarGroupLabelFn(group, rootIndex)))} title={_('Move group up')} onclick={() => onMoveSidebarGroup(group.id, -1)} disabled={rootIndex === 0}>
											<Icon icon="tabler:arrow-up" width="16" height="16" />
										</button>
										<button type="button" aria-label={_('Move {label} down').replace('{label}', _(sidebarGroupLabelFn(group, rootIndex)))} title={_('Move group down')} onclick={() => onMoveSidebarGroup(group.id, 1)} disabled={rootIndex === effectiveSidebarSettings.rootItemIds.length - 1}>
											<Icon icon="tabler:arrow-down" width="16" height="16" />
										</button>
										<button type="button" aria-label={_('Remove {label} group').replace('{label}', _(sidebarGroupLabelFn(group, rootIndex)))} title={_('Remove group')} onclick={() => onRemoveSidebarGroup(group.id)} disabled={group.id === 'communications'}>
											<Icon icon="tabler:trash" width="16" height="16" />
										</button>
									</div>
								</div>
							{/if}
						{:else}
							{@const item = sidebarConfigItemFn(rootId as SidebarItemId)}
							{#if item}
								{@const isHidden = effectiveSidebarSettings.hiddenItemIds.includes(item.id)}
								<div class="sidebar-config-item" class:hidden={isHidden}>
									<div class="sidebar-config-item-main">
										<span class="round-icon cyan"><Icon icon={item.icon} width="18" height="18" /></span>
										<div>
											<strong>{item.label}</strong>
											<small>{isHidden ? _('Hidden from sidebar') : _('Root domain')}</small>
										</div>
									</div>
									<div class="sidebar-config-item-controls">
										<HermesSelect
											value="root"
											options={sidebarMoveTargetOptions(true)}
											placeholder={_('Move to group')}
											searchPlaceholder={_('Search groups...')}
											emptyLabel={_('No options')}
											ariaLabel={_('Move {label} to group').replace('{label}', item.label)}
											onChange={(nextValue) => onMoveSidebarItemToGroup(item.id, nextValue)}
										/>
										<button type="button" aria-label={_('Move {label} up').replace('{label}', item.label)} title={_('Move item up')} onclick={() => onMoveSidebarRootItem(rootId, -1)} disabled={rootIndex === 0}>
											<Icon icon="tabler:arrow-up" width="16" height="16" />
										</button>
										<button type="button" aria-label={_('Move {label} down').replace('{label}', item.label)} title={_('Move item down')} onclick={() => onMoveSidebarRootItem(rootId, 1)} disabled={rootIndex === effectiveSidebarSettings.rootItemIds.length - 1}>
											<Icon icon="tabler:arrow-down" width="16" height="16" />
										</button>
										<button type="button" class:active={!isHidden} aria-pressed={!isHidden} onclick={() => onToggleSidebarItemHidden(item.id)}>
											<Icon icon={isHidden ? 'tabler:eye' : 'tabler:eye-off'} width="16" height="16" />{isHidden ? _('Show') : _('Hide')}
										</button>
									</div>
								</div>
							{/if}
						{/if}
					{/each}
				</div>
			</section>

			{#each effectiveSidebarSettings.groups as group, groupIndex}
				<section class="sidebar-config-group">
					<header>
						<label>
							<span>{_('Group label')}</span>
							<input value={group.label} placeholder={groupIndex === 0 ? _('Primary') : _('Group {n}').replace('{n}', String(groupIndex + 1))} autocomplete="off" oninput={(event) => onUpdateSidebarGroupLabel(group.id, inputEventValueFn(event))} />
						</label>
						<div class="sidebar-config-group-actions">
							<button type="button" aria-label={_('Move {label} up').replace('{label}', _(sidebarGroupLabelFn(group, groupIndex)))} title={_('Move group up')} onclick={() => onMoveSidebarGroup(group.id, -1)} disabled={sidebarRootIndexForGroupFn(group.id) <= 0}>
								<Icon icon="tabler:arrow-up" width="16" height="16" />
							</button>
							<button type="button" aria-label={_('Move {label} down').replace('{label}', _(sidebarGroupLabelFn(group, groupIndex)))} title={_('Move group down')} onclick={() => onMoveSidebarGroup(group.id, 1)} disabled={sidebarRootIndexForGroupFn(group.id) === effectiveSidebarSettings.rootItemIds.length - 1}>
								<Icon icon="tabler:arrow-down" width="16" height="16" />
							</button>
							<button type="button" aria-label={_('Remove {label} group').replace('{label}', _(sidebarGroupLabelFn(group, groupIndex)))} title={_('Remove group')} onclick={() => onRemoveSidebarGroup(group.id)} disabled={group.id === 'communications'}>
								<Icon icon="tabler:trash" width="16" height="16" />
							</button>
						</div>
					</header>
					<div class="sidebar-config-items">
						{#if group.itemIds.length === 0}
							<div class="empty-panel">{_('No items in this group.')}</div>
						{:else}
							{#each group.itemIds as itemId, itemIndex}
								{@const item = sidebarConfigItemFn(itemId)}
								{#if item}
									{@const isHidden = effectiveSidebarSettings.hiddenItemIds.includes(item.id)}
									{@const hasSeparator = sidebarGroupHasSeparatorBeforeFn(group, item.id)}
									<div class="sidebar-config-item" class:hidden={isHidden}>
										<div class="sidebar-config-item-main">
											<span class="round-icon cyan"><Icon icon={item.icon} width="18" height="18" /></span>
											<div>
												<strong>{item.label}</strong>
												<small>{isHidden ? _('Hidden from sidebar') : _('Visible domain')}</small>
											</div>
										</div>
										<div class="sidebar-config-item-controls">
											<HermesSelect
												value={group.id}
												options={sidebarMoveTargetOptions(!parseCommunicationSidebarItemId(item.id))}
												placeholder={_('Move to group')}
												searchPlaceholder={_('Search groups...')}
												emptyLabel={_('No options')}
												ariaLabel={_('Move {label} to group').replace('{label}', item.label)}
												onChange={(nextValue) => onMoveSidebarItemToGroup(item.id, nextValue)}
											/>
											<button type="button" aria-label={_('Move {label} up').replace('{label}', item.label)} title={_('Move item up')} onclick={() => onMoveSidebarItem(item.id, -1)}>
												<Icon icon="tabler:arrow-up" width="16" height="16" />
											</button>
											<button type="button" aria-label={_('Move {label} down').replace('{label}', item.label)} title={_('Move item down')} onclick={() => onMoveSidebarItem(item.id, 1)}>
												<Icon icon="tabler:arrow-down" width="16" height="16" />
											</button>
											<button type="button" class="separator-toggle" class:active={hasSeparator} aria-pressed={hasSeparator} aria-label={hasSeparator ? _('Remove divider before {label}').replace('{label}', item.label) : _('Add divider before {label}').replace('{label}', item.label)} title={hasSeparator ? _('Remove divider before {label}').replace('{label}', item.label) : _('Add divider before {label}').replace('{label}', item.label)} onclick={() => onToggleSidebarGroupSeparator(group.id, item.id)} disabled={itemIndex === 0}>
												<Icon icon="tabler:separator-horizontal" width="16" height="16" />{_('Divider')}
											</button>
											<button type="button" class:active={!isHidden} aria-pressed={!isHidden} onclick={() => onToggleSidebarItemHidden(item.id)}>
												<Icon icon={isHidden ? 'tabler:eye' : 'tabler:eye-off'} width="16" height="16" />{isHidden ? _('Show') : _('Hide')}
											</button>
										</div>
									</div>
								{/if}
							{/each}
						{/if}
					</div>
				</section>
			{/each}
		</div>
	</section>

	<aside class="settings-rail sidebar-settings-summary">
		<section class="panel info-card">
			<h2>{_('Preview')}</h2>
			<ul class="sidebar-preview-list">
				{#each sidebarRootEntries as entry, entryIndex}
					<li>
						{#if entry.kind === 'group'}
							<strong>{_(sidebarGroupLabelFn(entry.group, entryIndex))}</strong>
							<span>{entry.group.items.map((item) => sidebarItemLabelFn(item)).join(', ') || _('Empty group')}</span>
						{:else}
							<strong>{_(sidebarItemLabelFn(entry.item))}</strong>
							<span>{_('Root domain')}</span>
						{/if}
					</li>
				{/each}
			</ul>
		</section>
		<section class="panel info-card">
			<h2>{_('Hidden')}</h2>
			{#if sidebarHiddenNavItems.length === 0}
				<p>{_('No domains are hidden.')}</p>
			{:else}
				<ul class="detail-list">
					{#each sidebarHiddenNavItems as item}
						<li>{item.label}<button type="button" onclick={() => onToggleSidebarItemHidden(item.id)}>{_('Show')}</button></li>
					{/each}
				</ul>
			{/if}
		</section>
		<section class="panel info-card">
			<h2>{_('Rules')}</h2>
			<ul class="detail-list">
				<li>{_('Default keeps the current sidebar order')}<em>{_('Preset')}</em></li>
				<li>{_('Communications sources stay nested')}<em>{_('Context')}</em></li>
				<li>{_('Hidden domains stay recoverable here')}<em>{_('Safe')}</em></li>
				<li>{_('Settings store no message content')}<em>{_('Privacy')}</em></li>
			</ul>
		</section>
	</aside>
</div>
