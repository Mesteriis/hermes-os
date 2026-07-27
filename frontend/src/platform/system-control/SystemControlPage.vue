<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../shared/ui/Icon.vue'
import ToggleGroup from '../../shared/ui/ToggleGroup.vue'
import { clientSurfaceCatalog } from '../client-runtime/clientSurfaces'
import type { ClientSurfaceAdapterId } from '../client-runtime/clientSurfaces'
import { recoveryClientBootstrap, type ClientBootstrapSnapshot } from '../gateway/clientBootstrap'
import {
	eventComponents,
	publicModuleSettingRows,
	schedulerComponents,
	systemControlComponentRows,
} from './systemControlComponents'
import {
  systemControlAvailableSurfaceCount,
  systemControlModuleRows,
  systemControlSurfaceRows,
  systemControlSurfaceStateLabel,
} from './systemControlPresentation'
import './systemControlPage.css'

const props = withDefaults(defineProps<{
	bootstrap?: ClientBootstrapSnapshot
	routeDowngradeReason?: string
	developmentProfile?: 'disabled' | 'private-lan' | 'loopback-full-stack'
	currentLanguage?: string
	languageOptions?: readonly { value: string; label: string }[]
	compiledAdapterIds: readonly ClientSurfaceAdapterId[]
}>(), { developmentProfile: 'disabled', currentLanguage: 'ru', languageOptions: () => [] })
const emit = defineEmits<{ languageChange: [value: string] }>()
type SystemControlSection = 'system' | 'registry' | 'scheduler' | 'events' | 'composition' | 'interface'

const selectedSection = ref<SystemControlSection>('system')
const bootstrap = computed(() => props.bootstrap ?? recoveryClientBootstrap())
const availableSurfaceCount = computed(() => systemControlAvailableSurfaceCount(bootstrap.value))
const compositionRows = computed(() => systemControlSurfaceRows(
	bootstrap.value,
	props.compiledAdapterIds,
))
const moduleRows = computed(() => systemControlModuleRows(bootstrap.value.modules))
const schedulerRows = computed(() => systemControlComponentRows(schedulerComponents, bootstrap.value.systemStatus))
const eventRows = computed(() => systemControlComponentRows(eventComponents, bootstrap.value.systemStatus))
const publicSettingsRows = computed(() => publicModuleSettingRows(bootstrap.value.modules))
const developmentMode = computed(() => props.developmentProfile !== 'disabled')
const developmentProfileLabel = computed(() => {
	if (props.developmentProfile === 'private-lan') return 'Private LAN diagnostics'
	if (props.developmentProfile === 'loopback-full-stack') return 'Loopback full-stack assembly'
	return 'Authentication required'
})

</script>

<template>
	<section class="system-control-surface">
		<header class="system-control-surface__header">
			<div>
				<span>Platform settings</span>
				<h2>System Control</h2>
				<p>Owner-neutral recovery, admission and runtime state from the Kernel bootstrap.</p>
			</div>
			<strong>{{ availableSurfaceCount }}/{{ clientSurfaceCatalog.length }} surfaces</strong>
		</header>

		<nav class="system-control-tabs" aria-label="System Control sections">
			<button type="button" :aria-pressed="selectedSection === 'system'" @click="selectedSection = 'system'"><Icon icon="tabler:heart-rate-monitor" />System</button>
			<button type="button" :aria-pressed="selectedSection === 'registry'" @click="selectedSection = 'registry'"><Icon icon="tabler:adjustments" />Registry <em>{{ moduleRows.length }}</em></button>
			<button type="button" :aria-pressed="selectedSection === 'scheduler'" @click="selectedSection = 'scheduler'"><Icon icon="tabler:calendar-time" />Scheduler</button>
			<button type="button" :aria-pressed="selectedSection === 'events'" @click="selectedSection = 'events'"><Icon icon="tabler:route" />Events</button>
			<button type="button" :aria-pressed="selectedSection === 'composition'" @click="selectedSection = 'composition'"><Icon icon="tabler:layout-grid" />Surfaces</button>
			<button type="button" :aria-pressed="selectedSection === 'interface'" @click="selectedSection = 'interface'"><Icon icon="tabler:language" />Interface</button>
		</nav>

		<section v-if="selectedSection === 'system'" class="system-control-section">
			<header class="system-control-section__header"><h3>System Control</h3></header>
			<div v-if="routeDowngradeReason" class="inline-error" role="alert">Active product surface was closed: {{ routeDowngradeReason }}</div>
			<div class="system-control-list" aria-label="Kernel operator settings"><article class="system-control-row" :class="{ disabled: !developmentMode }"><Icon icon="tabler:code" /><span><strong>Developer mode</strong><small>{{ developmentProfileLabel }}</small></span><strong>{{ developmentMode ? 'Enabled' : 'Disabled' }}</strong></article></div>
		</section>
		<section v-else-if="selectedSection === 'registry'" class="system-control-section">
			<header class="system-control-section__header"><h3>Settings registry</h3></header>
			<div v-if="publicSettingsRows.length" class="system-control-list" aria-label="Public module settings"><article v-for="setting in publicSettingsRows" :key="setting.key" class="system-control-row" :class="{ disabled: setting.blocked }"><Icon :icon="setting.editable ? 'tabler:adjustments' : 'tabler:lock'" /><span><strong>{{ setting.label }}</strong><small>{{ setting.moduleId }} · {{ setting.settingId }} · {{ setting.applyState }}</small></span><strong>{{ setting.value }}</strong></article></div>
			<div v-else class="system-control-empty-state">No public module settings</div>
		</section>
		<section v-else-if="selectedSection === 'scheduler'" class="system-control-section">
			<header class="system-control-section__header"><h3>Scheduler</h3></header>
			<div class="system-control-list" aria-label="Scheduler runtime status"><article v-for="component in schedulerRows" :key="component.id" class="system-control-row" :class="{ disabled: component.disabled }"><Icon :icon="component.icon" /><span><strong>{{ component.label }}</strong><small>{{ component.reasonCode }}</small></span><strong>{{ component.stateLabel }}</strong></article></div>
		</section>
		<section v-else-if="selectedSection === 'events'" class="system-control-section">
			<header class="system-control-section__header"><h3>Events</h3></header>
			<div class="system-control-list" aria-label="Events runtime status"><article v-for="component in eventRows" :key="component.id" class="system-control-row" :class="{ disabled: component.disabled }"><Icon :icon="component.icon" /><span><strong>{{ component.label }}</strong><small>{{ component.reasonCode }}</small></span><strong>{{ component.stateLabel }}</strong></article></div>
		</section>
		<section v-else-if="selectedSection === 'composition'" class="system-control-section">
			<header class="system-control-section__header"><h3>Client surfaces</h3></header>
			<div class="system-control-list" aria-label="Client surface admission"><article v-for="surface in compositionRows" :key="surface.routeId" class="system-control-row" :class="{ disabled: !surface.available || !surface.compiledAdapterReady }"><Icon :icon="surface.icon" /><span><strong>{{ surface.label }}</strong><small>{{ surface.available ? (surface.compiledAdapterReady ? 'Ready for compiled route load' : 'client_route_adapter_unavailable') : surface.reasonCode || 'not_admitted' }}</small></span><strong>{{ systemControlSurfaceStateLabel(surface.routeId, surface.state, surface.available && surface.compiledAdapterReady) }}</strong></article></div>
			<h4 class="system-control-subsection-title">Module Control Plane</h4>
			<div v-if="moduleRows.length" class="system-control-list" aria-label="Approved module composition"><article v-for="module in moduleRows" :key="module.registrationId" class="system-control-row" :class="{ disabled: !module.sectionsEnabled }"><Icon icon="tabler:package" /><span><strong>{{ module.moduleId }}</strong><small>{{ module.registrationId }} · grants {{ module.capabilityCount }} · epoch {{ module.grantEpoch }}<template v-if="module.reasonCode"> · {{ module.reasonCode }}</template></small></span><strong>{{ module.sectionsEnabled ? (module.applyState ?? 'current') : (module.applyState ?? 'blocked_config') }}</strong></article></div>
			<div v-else class="system-control-empty-state">No approved modules</div>
		</section>
		<section v-else class="system-control-section">
			<header class="system-control-section__header"><h3>Interface</h3></header>
			<div class="system-control-list"><article class="system-control-row"><Icon icon="tabler:language" /><span><strong>Interface language</strong></span><ToggleGroup :model-value="currentLanguage" :items="languageOptions" aria-label="Interface language" @update:model-value="(value) => !Array.isArray(value) && emit('languageChange', value)" /></article></div>
		</section>
	</section>
</template>
