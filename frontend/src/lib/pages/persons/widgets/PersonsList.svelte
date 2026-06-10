<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface PersonItem {
		name: string;
		role: string;
		company: string;
		channel?: string;
		status?: string;
	}

	interface Props {
		personList: PersonItem[];
		selectedPersonIndex: number;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onSelectPerson: (index: number) => void;
	}

	let { personList, selectedPersonIndex, isLayoutEditing, isWidgetVisible, onSelectPerson }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-list" data-widget-hidden={!isWidgetVisible('persons-list')}>
	<WidgetEditChrome widgetId="persons-list" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel persons-list-panel">
		<header>
			<div><h1>{_('Persons')}</h1><p>{personList.length} {_('persons')}</p></div>
			<button type="button" class="primary-button" disabled>{_('New Person')}</button>
		</header>
		<div class="filter-tabs compact">
			<button type="button" class="active">{_('All')}</button>
			<button type="button" disabled>{_('People')} <em>532</em></button>
			<button type="button" disabled>{_('Companies')} <em>110</em></button>
		</div>
		<label class="local-search"><Icon icon="tabler:search" width="17" height="17" /><input placeholder={_('Search persons...')} /></label>
		{#each personList as person, index}
			<button type="button" class="person-row" class:active={selectedPersonIndex === index} onclick={() => onSelectPerson(index)}>
				<img src="/assets/hermes-reference-avatar.png" alt="" />
				<span><strong>{person.name}</strong><small>{person.role}</small><em>{person.company}</em></span>
				<small>{person.status ?? person.channel ?? _('Email')}</small>
			</button>
		{/each}
	</section>
</div>
