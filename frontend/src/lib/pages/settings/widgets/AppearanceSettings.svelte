<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import {
		selectShellBackground,
		updateShellBrightness,
		updateGlobalPanelOpacity,
		updateGlobalPanelBlur,
		selectShellAccent,
		resetThemeSettingsToDefault,
		shellBackgroundLabel,
		shellBackgroundPreviewClass,
		shellAccentSwatchClass,
		shellAccentLabel,
		effectiveThemeSettings,
		themeError,
		isThemeSettingsSaving
	} from '$lib/stores/theme';
	import {
		shellBackgroundOptions,
		shellAccentColorOptions,
		shellBackgroundBrightnessValues,
		panelOpacityValues,
		panelBlurValues
	} from '$lib/layout';

	const _ = (key: string) => t($currentLocale, key);
</script>

<div class="settings-layout appearance-settings-layout">
	<section class="panel settings-list-panel settings-primary-pane appearance-settings-panel">
		<header class="panel-title-row">
			<div>
				<h2>{_('Interface Appearance')}</h2>
				<p>{_('Choose shell background, brightness and application accent color.')}</p>
			</div>
			<div class="appearance-settings-actions">
				<button type="button" onclick={resetThemeSettingsToDefault} disabled={$isThemeSettingsSaving}>
					<Icon icon="tabler:restore" width="16" height="16" />{_('Default')}
				</button>
				<span class="appearance-save-state">{$isThemeSettingsSaving ? _('Saving') : _('Auto-save')}</span>
			</div>
		</header>

		{#if $themeError}
			<p class="inline-error">{$themeError}</p>
		{/if}

		<div class="appearance-config">
			<section class="appearance-section">
				<header>
					<div>
						<h3>{_('Shell Background')}</h3>
						<p>{_('Background image for the desktop shell.')}</p>
					</div>
					<strong>{_(shellBackgroundLabel($effectiveThemeSettings.shellBackground))}</strong>
				</header>
				<div class="background-option-grid">
					{#each shellBackgroundOptions as option}
						<button
							type="button"
							class:active={$effectiveThemeSettings.shellBackground === option.id}
							aria-pressed={$effectiveThemeSettings.shellBackground === option.id}
							onclick={() => selectShellBackground(option.id)}
						>
							<span class={shellBackgroundPreviewClass(option.id)}></span>
							<span>{_(option.label)}</span>
						</button>
					{/each}
				</div>
			</section>

			<section class="appearance-section">
				<header>
					<div>
						<h3>{_('Background Brightness')}</h3>
						<p>{_('Controls image brightness without changing layout geometry.')}</p>
					</div>
					<strong>{$effectiveThemeSettings.backgroundBrightness}%</strong>
				</header>
				<div class="brightness-control">
					<input
						type="range"
						min="30"
						max="100"
						step="10"
						value={$effectiveThemeSettings.backgroundBrightness}
						list="shell-background-brightness-values"
						aria-label={_('Background Brightness')}
						oninput={updateShellBrightness}
					/>
					<datalist id="shell-background-brightness-values">
						{#each shellBackgroundBrightnessValues as value}
							<option value={value}></option>
						{/each}
					</datalist>
				</div>
			</section>

			<section class="appearance-section">
				<header>
					<div>
						<h3>{_('Panel Transparency')}</h3>
						<p>{_('Global opacity for widget panels and cards.')}</p>
					</div>
					<strong>{$effectiveThemeSettings.panelOpacity}%</strong>
				</header>
				<div class="brightness-control">
					<input
						type="range"
						min="40"
						max="100"
						step="10"
						value={$effectiveThemeSettings.panelOpacity}
						list="panel-opacity-values"
						aria-label={_('Panel Transparency')}
						oninput={updateGlobalPanelOpacity}
					/>
					<datalist id="panel-opacity-values">
						{#each panelOpacityValues as value}
							<option value={value}></option>
						{/each}
					</datalist>
				</div>
			</section>

			<section class="appearance-section">
				<header>
					<div>
						<h3>{_('Panel Blur')}</h3>
						<p>{_('Global backdrop blur for widget panels and cards.')}</p>
					</div>
					<strong>{$effectiveThemeSettings.panelBlur}px</strong>
				</header>
				<div class="brightness-control">
					<input
						type="range"
						min="0"
						max="24"
						step="4"
						value={$effectiveThemeSettings.panelBlur}
						list="panel-blur-values"
						aria-label={_('Panel Blur')}
						oninput={updateGlobalPanelBlur}
					/>
					<datalist id="panel-blur-values">
						{#each panelBlurValues as value}
							<option value={value}></option>
						{/each}
					</datalist>
				</div>
			</section>

			<section class="appearance-section">
				<header>
					<div>
						<h3>{_('Application Color')}</h3>
						<p>{_('Accent color for controls, borders and active states.')}</p>
					</div>
					<strong>{_(shellAccentLabel($effectiveThemeSettings.accentColor))}</strong>
				</header>
				<div class="accent-swatch-grid">
					{#each shellAccentColorOptions as option}
						<button
							type="button"
							class={shellAccentSwatchClass(option.id)}
							class:active={$effectiveThemeSettings.accentColor === option.id}
							aria-pressed={$effectiveThemeSettings.accentColor === option.id}
							onclick={() => selectShellAccent(option.id)}
						>
							<span class="accent-swatch-dot"></span>
							<span>{_(option.label)}</span>
						</button>
					{/each}
				</div>
			</section>
		</div>
	</section>

	<aside class="settings-rail appearance-preview-rail">
		<section class="panel info-card">
			<h2>{_('Current Appearance')}</h2>
			<div class="health-row"><span>{_('Background')}</span><strong>{_(shellBackgroundLabel($effectiveThemeSettings.shellBackground))}</strong></div>
			<div class="health-row"><span>{_('Brightness')}</span><strong>{$effectiveThemeSettings.backgroundBrightness}%</strong></div>
			<div class="health-row"><span>{_('Color')}</span><strong>{_(shellAccentLabel($effectiveThemeSettings.accentColor))}</strong></div>
			<div class="health-row"><span>{_('Panel Transparency')}</span><strong>{$effectiveThemeSettings.panelOpacity}%</strong></div>
			<div class="health-row"><span>{_('Panel Blur')}</span><strong>{$effectiveThemeSettings.panelBlur}px</strong></div>
		</section>
		<section class="panel info-card">
			<h2>{_('Storage')}</h2>
			<ul class="detail-list">
				<li>{_('Stored as declared frontend.theme setting')}<em>JSON</em></li>
				<li>{_('No private content or secrets')}<em>ADR-0054</em></li>
				<li>{_('No inline styles')}<em>CSS</em></li>
			</ul>
		</section>
	</aside>
</div>
