<script setup lang="ts">
import { computed } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import {
	publicModuleSettingRows,
	publicModuleSettingsReasonCode,
	settingsApplyStateLabel,
} from '../../../platform/gateway/publicModuleSettings'
import ModuleSettingsPanel from '../../../shared/ui/settings/ModuleSettingsPanel.vue'
import type { ModuleSettingsPanelModel } from '../../../shared/ui/settings/ModuleSettingsPanelModel'
import MailAccountManagementPanel from './MailAccountManagementPanel.vue'
import MailAccountSetupPanel from './MailAccountSetupPanel.vue'
import MailPortabilityPanel from './MailPortabilityPanel.vue'
import MailGmailPermanentDeleteAuthorizationPanel from './MailGmailPermanentDeleteAuthorizationPanel.vue'

const MAIL_MODULE_ID = 'hermes-mail-runtime'
const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const model = computed<ModuleSettingsPanelModel>(() => {
	const owned = props.module?.moduleId === MAIL_MODULE_ID ? props.module : null
	const settings = owned?.settings
	return {
		title: 'Mail',
		description: 'Mail owns provider accounts, synchronization and outbound delivery settings.',
		icon: 'tabler:mail',
		tone: 'mail',
		moduleId: MAIL_MODULE_ID,
		registered: Boolean(owned),
		applyState: settings ? settingsApplyStateLabel(settings.applyState) : 'No schema',
		revision: settings ? `${settings.effectiveRevision}/${settings.desiredRevision}` : '—',
		reasonCode: publicModuleSettingsReasonCode(owned),
		settings: publicModuleSettingRows(owned ? [owned] : []),
	}
})
</script>

<template>
	<div class="mail-settings-owner">
		<ModuleSettingsPanel :model="model" />
		<MailAccountSetupPanel :module="module" />
		<MailAccountManagementPanel :module="module" />
		<MailGmailPermanentDeleteAuthorizationPanel />
		<MailPortabilityPanel :module="module" />
	</div>
</template>
