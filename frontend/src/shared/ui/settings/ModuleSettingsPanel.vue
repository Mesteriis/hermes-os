<script setup lang="ts">
import Icon from '../Icon.vue'
import type { ModuleSettingsPanelModel } from './ModuleSettingsPanelModel'
import './moduleSettingsPanel.css'

defineProps<{ model: ModuleSettingsPanelModel }>()
</script>

<template>
	<section class="module-settings-panel" :data-provider-tone="model.tone">
		<header class="module-settings-panel__header">
			<span class="module-settings-panel__icon"><Icon :icon="model.icon" /></span>
			<div>
				<span>Integration settings</span>
				<h2>{{ model.title }}</h2>
				<p>{{ model.description }}</p>
			</div>
			<strong>{{ model.registered ? model.applyState : 'Not admitted' }}</strong>
		</header>

		<div class="module-settings-panel__metadata">
			<span><small>Module</small><strong>{{ model.moduleId }}</strong></span>
			<span><small>Revision</small><strong>{{ model.revision }}</strong></span>
			<span><small>Reason</small><strong>{{ model.reasonCode || 'current' }}</strong></span>
		</div>

		<div v-if="model.settings.length" class="module-settings-panel__list">
			<article
				v-for="setting in model.settings"
				:key="setting.key"
				:class="{ blocked: setting.blocked }"
			>
				<Icon :icon="setting.editable ? 'tabler:adjustments' : 'tabler:lock'" />
				<span>
					<strong>{{ setting.label }}</strong>
					<small>{{ setting.settingId }} · {{ setting.editable ? 'Owner-editable' : 'Read only' }}</small>
				</span>
				<strong>{{ setting.value }}</strong>
			</article>
		</div>
		<div v-else class="module-settings-panel__empty">
			<Icon icon="tabler:settings-off" />
			<strong>No public settings admitted</strong>
			<p>
				Only sanitized values from this module's Settings Registry projection can appear
				here. Secrets, provider sessions, cursors and checkpoints remain excluded.
			</p>
		</div>
	</section>
</template>
