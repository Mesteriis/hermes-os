<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { ResolvedWidget } from '$lib/layout';
	import './widgetSettingsDrawer.css';

	const _ = (key: string) => t($currentLocale, key);

	type WidgetGridDimension = 'columns' | 'rows';
	type WidgetPanelSurfaceSetting = 'panelOpacity' | 'panelBlur';

	interface Props {
		isOpen: boolean;
		widget: ResolvedWidget | null;
		onClose: () => void;
		widgetGridValue: (widgetId: string, dimension: WidgetGridDimension) => number;
		widgetGridMin: (widgetId: string, dimension: WidgetGridDimension) => number;
		widgetGridMax: (dimension: WidgetGridDimension) => number;
		adjustWidgetGridValue: (widgetId: string, dimension: WidgetGridDimension, delta: -1 | 1) => void;
		handleWidgetGridInput: (widgetId: string, dimension: WidgetGridDimension, event: Event) => void;
		widgetPanelSurfaceValue: (widgetId: string, setting: WidgetPanelSurfaceSetting) => number;
		widgetPanelSurfaceOverrideValue: (widgetId: string, setting: WidgetPanelSurfaceSetting) => number | null;
		handleWidgetPanelSurfaceInput: (widgetId: string, setting: WidgetPanelSurfaceSetting, event: Event) => void;
		resetWidgetPanelSurface: (widgetId: string) => void;
		resetWidgetGrid: (widgetId: string) => void;
		moveWidgetInZone: (widgetId: string, direction: -1 | 1) => void;
		hideWidget: (widgetId: string) => void;
		widgetZoneTitle: (zoneId: string) => string;
	}

	let {
		isOpen,
		widget,
		onClose,
		widgetGridValue,
		widgetGridMin,
		widgetGridMax,
		adjustWidgetGridValue,
		handleWidgetGridInput,
		widgetPanelSurfaceValue,
		widgetPanelSurfaceOverrideValue,
		handleWidgetPanelSurfaceInput,
		resetWidgetPanelSurface,
		resetWidgetGrid,
		moveWidgetInZone,
		hideWidget,
		widgetZoneTitle
	}: Props = $props();
</script>

{#if isOpen && widget}
	<div class="layout-widget-drawer" role="dialog" aria-label={_('Widget settings')}>
		<header>
			<div>
				<p>{_('Widget settings')}</p>
				<h2>{_(widget.definition.title)}</h2>
			</div>
			<button
				type="button"
				class="icon-button"
				onclick={onClose}
				title={_('Close widget settings')}
				aria-label={_('Close widget settings')}
			>
				<Icon icon="tabler:x" width="16" height="16" />
			</button>
		</header>

		<div class="layout-widget-meta">
			<span>{_('Widget ID')}<strong>{widget.widgetId}</strong></span>
			<span>{_('Zone')}<strong>{_(widgetZoneTitle(widget.zoneId))}</strong></span>
		</div>

		<section class="layout-widget-size-panel" aria-label={_('Widget grid size')}>
			<div class="layout-widget-grid-row">
				<div class="widget-grid-field">
					<span>{_('Cols')}</span>
					<div class="widget-grid-stepper">
						<button
							type="button"
							title={_('Decrease columns')}
							aria-label={_('Decrease columns')}
							disabled={widgetGridValue(widget.widgetId, 'columns') <= widgetGridMin(widget.widgetId, 'columns')}
							onclick={() => adjustWidgetGridValue(widget.widgetId, 'columns', -1)}
						>
							<Icon icon="tabler:minus" width="12" height="12" />
						</button>
						<input
							type="number"
							min={widgetGridMin(widget.widgetId, 'columns')}
							max={widgetGridMax('columns')}
							value={widgetGridValue(widget.widgetId, 'columns')}
							aria-label={_('Widget columns')}
							onchange={(event) => handleWidgetGridInput(widget.widgetId, 'columns', event)}
						/>
						<button
							type="button"
							title={_('Increase columns')}
							aria-label={_('Increase columns')}
							disabled={widgetGridValue(widget.widgetId, 'columns') >= widgetGridMax('columns')}
							onclick={() => adjustWidgetGridValue(widget.widgetId, 'columns', 1)}
						>
							<Icon icon="tabler:plus" width="12" height="12" />
						</button>
					</div>
					<small>{widgetGridMin(widget.widgetId, 'columns')} - {widgetGridMax('columns')}</small>
				</div>

				<div class="widget-grid-field">
					<span>{_('Rows')}</span>
					<div class="widget-grid-stepper">
						<button
							type="button"
							title={_('Decrease rows')}
							aria-label={_('Decrease rows')}
							disabled={widgetGridValue(widget.widgetId, 'rows') <= widgetGridMin(widget.widgetId, 'rows')}
							onclick={() => adjustWidgetGridValue(widget.widgetId, 'rows', -1)}
						>
							<Icon icon="tabler:minus" width="12" height="12" />
						</button>
						<input
							type="number"
							min={widgetGridMin(widget.widgetId, 'rows')}
							max={widgetGridMax('rows')}
							value={widgetGridValue(widget.widgetId, 'rows')}
							aria-label={_('Widget rows')}
							onchange={(event) => handleWidgetGridInput(widget.widgetId, 'rows', event)}
						/>
						<button
							type="button"
							title={_('Increase rows')}
							aria-label={_('Increase rows')}
							disabled={widgetGridValue(widget.widgetId, 'rows') >= widgetGridMax('rows')}
							onclick={() => adjustWidgetGridValue(widget.widgetId, 'rows', 1)}
						>
							<Icon icon="tabler:plus" width="12" height="12" />
						</button>
					</div>
					<small>{widgetGridMin(widget.widgetId, 'rows')} - {widgetGridMax('rows')}</small>
				</div>
			</div>
		</section>

		<section class="layout-widget-surface-panel" aria-label={_('Widget panel surface')}>
			<header>
				<div>
					<h3>{_('Panel Surface')}</h3>
					<p>{_('Override transparency and blur for this widget only.')}</p>
				</div>
				<button
					type="button"
					onclick={() => resetWidgetPanelSurface(widget.widgetId)}
					disabled={
						widgetPanelSurfaceOverrideValue(widget.widgetId, 'panelOpacity') === null &&
						widgetPanelSurfaceOverrideValue(widget.widgetId, 'panelBlur') === null
					}
				>
					<Icon icon="tabler:restore" width="14" height="14" />{_('Reset')}
				</button>
			</header>

			<div class="layout-widget-grid-row">
				<div class="widget-grid-field">
					<span>{_('Opacity')}</span>
					<div class="widget-surface-slider">
						<input
							type="range"
							min="40"
							max="100"
							step="10"
							value={widgetPanelSurfaceValue(widget.widgetId, 'panelOpacity')}
							aria-label={_('Widget panel opacity')}
							oninput={(event) => handleWidgetPanelSurfaceInput(widget.widgetId, 'panelOpacity', event)}
						/>
					</div>
					<small>
						{widgetPanelSurfaceValue(widget.widgetId, 'panelOpacity')}%
						{widgetPanelSurfaceOverrideValue(widget.widgetId, 'panelOpacity') === null ? ` ${_('Global')}` : ` ${_('Override')}`}
					</small>
				</div>

				<div class="widget-grid-field">
					<span>{_('Blur')}</span>
					<div class="widget-surface-slider">
						<input
							type="range"
							min="0"
							max="24"
							step="4"
							value={widgetPanelSurfaceValue(widget.widgetId, 'panelBlur')}
							aria-label={_('Widget panel blur')}
							oninput={(event) => handleWidgetPanelSurfaceInput(widget.widgetId, 'panelBlur', event)}
						/>
					</div>
					<small>
						{widgetPanelSurfaceValue(widget.widgetId, 'panelBlur')}px
						{widgetPanelSurfaceOverrideValue(widget.widgetId, 'panelBlur') === null ? ` ${_('Global')}` : ` ${_('Override')}`}
					</small>
				</div>
			</div>
		</section>

		<div class="layout-widget-actions">
			<button type="button" onclick={() => moveWidgetInZone(widget.widgetId, -1)}>
				<Icon icon="tabler:arrow-up" width="14" height="14" />{_('Move widget up')}
			</button>
			<button type="button" onclick={() => moveWidgetInZone(widget.widgetId, 1)}>
				<Icon icon="tabler:arrow-down" width="14" height="14" />{_('Move widget down')}
			</button>
			<button type="button" onclick={() => resetWidgetGrid(widget.widgetId)}>
				<Icon icon="tabler:restore" width="14" height="14" />{_('Reset widget size')}
			</button>
			<button type="button" class="danger" onclick={() => hideWidget(widget.widgetId)}>
				<Icon icon="tabler:eye-off" width="14" height="14" />{_('Hide widget')}
			</button>
		</div>
	</div>
{/if}
