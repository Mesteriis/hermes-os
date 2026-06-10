<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

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
			<div><h1>Persons</h1><p>{personList.length} persons</p></div>
			<button type="button" class="primary-button" disabled>New Person</button>
		</header>
		<div class="filter-tabs compact">
			<button type="button" class="active">All</button>
			<button type="button" disabled>People <em>532</em></button>
			<button type="button" disabled>Companies <em>110</em></button>
		</div>
		<label class="local-search"><Icon icon="tabler:search" width="17" height="17" /><input placeholder="Search persons..." /></label>
		{#each personList as person, index}
			<button type="button" class="person-row" class:active={selectedPersonIndex === index} onclick={() => onSelectPerson(index)}>
				<img src="/assets/hermes-reference-avatar.png" alt="" />
				<span><strong>{person.name}</strong><small>{person.role}</small><em>{person.company}</em></span>
				<small>{person.status ?? person.channel ?? 'Email'}</small>
			</button>
		{/each}
	</section>
</div>
