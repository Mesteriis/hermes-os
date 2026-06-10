<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, setLocale, t } from '$lib/i18n';
	import { saveFrontendLocaleSetting } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);
</script>

<div class="settings-layout">
	<section class="panel settings-list-panel settings-primary-pane">
		<header class="panel-title-row">
			<div><h2>Interface Language</h2><p>Choose the display language for the Hermes Hub interface.</p></div>
		</header>
		<div class="settings-category-list">
			<div class="setting-row">
				<span>Language</span>
				<div class="setting-control">
					<select value={$currentLocale} onchange={async (event) => { const el = event.target; if (el instanceof HTMLSelectElement) { const loc = el.value as "en" | "ru"; setLocale(loc); try { await saveFrontendLocaleSetting(loc); } catch { /* ignore */ } } }}>
						<option value="en">English</option>
						<option value="ru">Русский</option>
					</select>
				</div>
			</div>
		</div>
	</section>
	<aside class="settings-rail">
		<section class="panel info-card">
			<h2>About</h2>
			<p>Language preference is stored in memory for the current session. A persistent locale setting can be added later.</p>
		</section>
	</aside>
</div>
