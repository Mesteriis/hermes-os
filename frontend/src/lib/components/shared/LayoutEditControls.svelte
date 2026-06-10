<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		isLayoutEditing: boolean;
		isSaving: boolean;
		hasChanges: boolean;
		onAddWidget: () => void;
		onCancel: () => void;
		onReset: () => void;
		onSave: () => void;
	}

	let { isLayoutEditing, isSaving, hasChanges, onAddWidget, onCancel, onReset, onSave }: Props = $props();
</script>

{#if isLayoutEditing}
	<div class="layout-edit-controls" role="group" aria-label={_('Widget layout controls')}>
		<button type="button" class="ghost-button" onclick={onAddWidget}>
			<Icon icon="tabler:plus" width="16" height="16" />
			{_('Add widget')}
		</button>
		<button type="button" class="ghost-button" onclick={onCancel}>{_('Cancel')}</button>
		<button type="button" class="ghost-button" onclick={onReset}>{_('Reset')}</button>
		<button
			type="button"
			class="primary-button"
			disabled={isSaving || !hasChanges}
			onclick={onSave}
		>
			{isSaving ? _('Saving') : _('Save')}
		</button>
	</div>
{/if}
