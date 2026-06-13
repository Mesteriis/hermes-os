<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import * as personsService from '$lib/services/persons';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { PersonIdentity } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	type PersonaOption = {
		person_id: string;
		name: string;
		company: string;
	};

	interface Props {
		identityTraces: PersonIdentity[];
		persons: PersonaOption[];
		selectedPersonaId: string | null;
		isLoading: boolean;
		error: string;
		assigningIdentityTraceId: string | null;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onReload: () => Promise<void>;
		onAssign: (trace: PersonIdentity, personId: string) => Promise<void>;
	}

	let {
		identityTraces,
		persons,
		selectedPersonaId,
		isLoading,
		error,
		assigningIdentityTraceId,
		isLayoutEditing,
		isWidgetVisible,
		onReload,
		onAssign
	}: Props = $props();

	let selectedPersonaByTrace = $state<Record<string, string>>({});

	let pendingIdentityTraces = $derived(identityTraces.filter((trace) => trace.person_id === null));

	function targetPersonaId(trace: PersonIdentity): string {
		return selectedPersonaByTrace[trace.id] ?? selectedPersonaId ?? persons[0]?.person_id ?? '';
	}

	function personaLabel(person: PersonaOption): string {
		return person.company ? `${person.name} · ${person.company}` : person.name;
	}
</script>

<div
	class="widget-frame"
	class:editing={isLayoutEditing}
	data-widget-id="persons-identity-trace-review"
	data-widget-hidden={!isWidgetVisible('persons-identity-trace-review')}
>
	<WidgetEditChrome
		widgetId="persons-identity-trace-review"
		{isLayoutEditing}
		isSelected={false}
		onConfigure={() => {}}
	/>
	<section class="panel info-card relationship-review-panel" aria-busy={isLoading}>
		<header>
			<div>
				<span class="panel-kicker">{_('Identity Resolution')}</span>
				<h2>{_('Unattached Traces')}</h2>
			</div>
			<button type="button" title={_('Reload identity traces')} onclick={() => void onReload()} disabled={isLoading}>
				<Icon icon="tabler:refresh" width="15" height="15" />
			</button>
		</header>

		{#if error}
			<div class="relationship-review-state error">
				<span>{error}</span>
				<button type="button" onclick={() => void onReload()} disabled={isLoading}>{_('Retry')}</button>
			</div>
		{:else if isLoading}
			<div class="relationship-review-state">
				<span>{_('Loading identity traces')}</span>
			</div>
		{:else if pendingIdentityTraces.length === 0}
			<div class="relationship-review-state">
				<span>{_('No unattached identity traces')}</span>
			</div>
		{:else}
			<div class="relationship-review-list">
				{#each pendingIdentityTraces as trace}
					<article class="relationship-review-item">
						<div>
							<strong>{personsService.formatIdentityTraceKind(trace.identity_type)}</strong>
							<p>{personsService.formatIdentityTraceValue(trace)}</p>
							<small>
								{_('Source')}: {trace.source}
								· {_('Confidence')}: {personsService.identityTraceConfidence(trace)}
							</small>
						</div>
						<div class="identity-trace-target">
							<select
								value={targetPersonaId(trace)}
								aria-label={_('Target Persona')}
								onchange={(event) => {
									selectedPersonaByTrace[trace.id] = event.currentTarget.value;
								}}
							>
								{#each persons as person}
									<option value={person.person_id}>{personaLabel(person)}</option>
								{/each}
							</select>
							<button
								type="button"
								disabled={assigningIdentityTraceId === trace.id || persons.length === 0}
								onclick={() => void onAssign(trace, targetPersonaId(trace))}
							>
								<Icon icon="tabler:link" width="14" height="14" />
								{_('Assign')}
							</button>
						</div>
					</article>
				{/each}
			</div>
		{/if}
	</section>
</div>
