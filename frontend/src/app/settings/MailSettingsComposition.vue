<script setup lang="ts">
import { computed, onMounted } from 'vue'

import type { ClientModuleBootstrapV1 } from '../../gen/hermes/gateway/v1/client_bootstrap_pb'
import MailSettingsPanel from '../../integrations/mail/presentation/MailSettingsPanel.vue'
import { useMailAccountConnections } from '../../integrations/mail/queries/useMailAccountConnections'
import MailContactsSyncSettingsPanel from '../../workflows/mail-contacts-sync/presentation/MailContactsSyncSettingsPanel.vue'
import { mailContactsSyncAccountChoices } from './mailSettingsComposition'

const props = defineProps<{
	modules: readonly ClientModuleBootstrapV1[]
	mailModule: ClientModuleBootstrapV1 | null
	syncModule: ClientModuleBootstrapV1 | null
}>()

const connections = useMailAccountConnections({
	canQuery: () => Boolean(props.mailModule?.capabilityIds.includes('mail.account.catalog.query.v1')),
	modules: () => props.modules,
})
const accountChoices = computed(() => mailContactsSyncAccountChoices(connections.connections.value))

onMounted(() => void connections.refresh().catch(() => undefined))
</script>

<template>
	<div class="mail-settings-owner">
		<MailSettingsPanel :module="mailModule" />
		<MailContactsSyncSettingsPanel :module="syncModule" :accounts="accountChoices" />
	</div>
</template>
