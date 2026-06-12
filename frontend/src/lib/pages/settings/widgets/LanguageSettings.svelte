<script lang="ts">
	import Icon from '@iconify/svelte';
	import HermesSelect from '$lib/components/shared/HermesSelect.svelte';
	import { currentLocale, t, type Locale } from '$lib/i18n';
	import { saveLocaleSetting } from '$lib/stores/settings';

	const _ = (key: string) => t($currentLocale, key);
	const localeOptions = $derived([
		{ value: 'en', label: _('English') },
		{ value: 'ru', label: _('Русский') }
	]);

	async function updateLocale(value: string) {
		if (value !== 'en' && value !== 'ru') return;
		await saveLocaleSetting(value as Locale);
	}
</script>

<div class="settings-layout">
	<section class="panel settings-list-panel settings-primary-pane">
		<header class="panel-title-row">
			<div><h2>{_('Interface Language')}</h2><p>{_('Choose the display language for the Hermes Hub interface.')}</p></div>
		</header>
		<div class="settings-category-list">
			<div class="setting-row">
				<span>{_('Language')}</span>
				<div class="setting-control">
					<HermesSelect
						value={$currentLocale}
						options={localeOptions}
						placeholder={_('Language')}
						searchPlaceholder={_('Search languages...')}
						emptyLabel={_('No options')}
						ariaLabel={_('Language')}
						searchable={false}
						onChange={(nextValue) => void updateLocale(nextValue)}
					/>
				</div>
			</div>
		</div>
	</section>
	<aside class="settings-rail">
		<section class="panel info-card">
			<h2>{_('About')}</h2>
			<p>{_('Language preference is stored as a declared frontend.locale setting.')}</p>
		</section>
	</aside>
</div>
