<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../shared/ui/Icon.vue'
import ToggleGroup from '../../shared/ui/ToggleGroup.vue'
import { clientSurfaceCatalog } from '../client-runtime/clientSurfaces'
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

const props = withDefaults(defineProps<{
	bootstrap?: ClientBootstrapSnapshot
	routeDowngradeReason?: string
	developerMode?: boolean
	currentLanguage?: string
	languageOptions?: readonly { value: string; label: string }[]
}>(), { developerMode: false, currentLanguage: 'ru', languageOptions: () => [] })
const emit = defineEmits<{ languageChange: [value: string] }>()
type SystemControlSection = 'system' | 'registry' | 'scheduler' | 'events' | 'composition' | 'interface'

const selectedSection = ref<SystemControlSection>('system')
const bootstrap = computed(() => props.bootstrap ?? recoveryClientBootstrap())
const availableSurfaceCount = computed(() => systemControlAvailableSurfaceCount(bootstrap.value))
const compositionRows = computed(() => systemControlSurfaceRows(bootstrap.value))
const moduleRows = computed(() => systemControlModuleRows(bootstrap.value.modules))
const schedulerRows = computed(() => systemControlComponentRows(schedulerComponents, bootstrap.value.systemStatus))
const eventRows = computed(() => systemControlComponentRows(eventComponents, bootstrap.value.systemStatus))
const publicSettingsRows = computed(() => publicModuleSettingRows(bootstrap.value.modules))

</script>

<template>
	<section class="settings-page system-control-page">
		<div class="settings-workbench">
			<nav class="settings-tree" aria-label="System Control sections">
				<header class="settings-tree-header"><span>Settings</span><strong>System Control</strong></header>
				<section class="settings-tree-group">
					<h2>Platform</h2>
					<button type="button" :class="{ active: selectedSection === 'system' }" @click="selectedSection = 'system'"><Icon class="tree-icon" icon="tabler:heart-rate-monitor" /><span class="settings-tree-copy"><strong>System status</strong></span></button>
					<button type="button" :class="{ active: selectedSection === 'registry' }" @click="selectedSection = 'registry'"><Icon class="tree-icon" icon="tabler:adjustments" /><span class="settings-tree-copy"><strong>Settings registry</strong></span><em>{{ publicSettingsRows.length }}</em></button>
					<button type="button" :class="{ active: selectedSection === 'scheduler' }" @click="selectedSection = 'scheduler'"><Icon class="tree-icon" icon="tabler:calendar-time" /><span class="settings-tree-copy"><strong>Scheduler</strong></span></button>
					<button type="button" :class="{ active: selectedSection === 'events' }" @click="selectedSection = 'events'"><Icon class="tree-icon" icon="tabler:route" /><span class="settings-tree-copy"><strong>Events</strong></span></button>
					<button type="button" :class="{ active: selectedSection === 'composition' }" @click="selectedSection = 'composition'"><Icon class="tree-icon" icon="tabler:layout-grid" /><span class="settings-tree-copy"><strong>Client surfaces</strong></span><em>{{ availableSurfaceCount }}/{{ clientSurfaceCatalog.length }}</em></button>
					<button type="button" :class="{ active: selectedSection === 'interface' }" @click="selectedSection = 'interface'"><Icon class="tree-icon" icon="tabler:language" /><span class="settings-tree-copy"><strong>Interface</strong></span></button>
				</section>
			</nav>

			<main class="settings-workbench-content">
				<section v-if="selectedSection === 'system'" class="settings-section">
					<header class="settings-section-toolbar"><h3>System Control</h3></header>
					<div v-if="routeDowngradeReason" class="inline-error" role="alert">Active product surface was closed: {{ routeDowngradeReason }}</div>
					<div class="settings-service-list" aria-label="Kernel operator settings"><article class="settings-service-row" :class="{ disabled: !developerMode }"><Icon icon="tabler:code" /><span><strong>Developer mode</strong><small>{{ developerMode ? 'Private LAN' : 'Authentication required' }}</small></span><strong>{{ developerMode ? 'Enabled' : 'Disabled' }}</strong></article></div>
				</section>
				<section v-else-if="selectedSection === 'registry'" class="settings-section">
					<header class="settings-section-toolbar"><h3>Settings registry</h3></header>
					<div v-if="publicSettingsRows.length" class="settings-service-list" aria-label="Public module settings"><article v-for="setting in publicSettingsRows" :key="setting.key" class="settings-service-row" :class="{ disabled: setting.blocked }"><Icon :icon="setting.editable ? 'tabler:adjustments' : 'tabler:lock'" /><span><strong>{{ setting.label }}</strong><small>{{ setting.moduleId }} · {{ setting.settingId }} · {{ setting.applyState }}</small></span><strong>{{ setting.value }}</strong></article></div>
					<div v-else class="settings-empty-state">No public module settings</div>
				</section>
				<section v-else-if="selectedSection === 'scheduler'" class="settings-section">
					<header class="settings-section-toolbar"><h3>Scheduler</h3></header>
					<div class="settings-service-list" aria-label="Scheduler runtime status"><article v-for="component in schedulerRows" :key="component.id" class="settings-service-row" :class="{ disabled: component.disabled }"><Icon :icon="component.icon" /><span><strong>{{ component.label }}</strong><small>{{ component.reasonCode }}</small></span><strong>{{ component.stateLabel }}</strong></article></div>
				</section>
				<section v-else-if="selectedSection === 'events'" class="settings-section">
					<header class="settings-section-toolbar"><h3>Events</h3></header>
					<div class="settings-service-list" aria-label="Events runtime status"><article v-for="component in eventRows" :key="component.id" class="settings-service-row" :class="{ disabled: component.disabled }"><Icon :icon="component.icon" /><span><strong>{{ component.label }}</strong><small>{{ component.reasonCode }}</small></span><strong>{{ component.stateLabel }}</strong></article></div>
				</section>
				<section v-else-if="selectedSection === 'composition'" class="settings-section">
					<header class="settings-section-toolbar"><h3>Client surfaces</h3></header>
					<div class="settings-service-list" aria-label="Client surface admission"><article v-for="surface in compositionRows" :key="surface.routeId" class="settings-service-row" :class="{ disabled: !surface.available || !surface.compiledAdapterReady }"><Icon :icon="surface.icon" /><span><strong>{{ surface.label }}</strong><small>{{ surface.available ? (surface.compiledAdapterReady ? 'Ready for compiled route load' : 'client_route_adapter_unavailable') : surface.reasonCode || 'not_admitted' }}</small></span><strong>{{ systemControlSurfaceStateLabel(surface.routeId, surface.state, surface.available && surface.compiledAdapterReady) }}</strong></article></div>
					<h4 class="settings-subsection-title">Module Control Plane</h4>
					<div v-if="moduleRows.length" class="settings-service-list" aria-label="Approved module composition"><article v-for="module in moduleRows" :key="module.registrationId" class="settings-service-row" :class="{ disabled: !module.sectionsEnabled }"><Icon icon="tabler:package" /><span><strong>{{ module.moduleId }}</strong><small>{{ module.registrationId }} · grants {{ module.capabilityCount }} · epoch {{ module.grantEpoch }}<template v-if="module.reasonCode"> · {{ module.reasonCode }}</template></small></span><strong>{{ module.sectionsEnabled ? (module.applyState ?? 'current') : (module.applyState ?? 'blocked_config') }}</strong></article></div>
					<div v-else class="settings-empty-state">No approved modules</div>
				</section>
				<section v-else class="settings-section">
					<header class="settings-section-toolbar"><h3>Interface</h3></header>
					<div class="settings-service-list"><article class="settings-service-row"><Icon icon="tabler:language" /><span><strong>Interface language</strong></span><ToggleGroup :model-value="currentLanguage" :items="languageOptions" aria-label="Interface language" @update:model-value="(value) => !Array.isArray(value) && emit('languageChange', value)" /></article></div>
				</section>
			</main>
		</div>
	</section>
</template>
