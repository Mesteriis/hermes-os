<script setup lang="ts">
import { onMounted } from 'vue'

import TelegramAutomationPanel from '../presentation/TelegramAutomationPanel.vue'
import { useTelegramAutomationManagement } from '../queries/useTelegramAutomationManagement'

const props = defineProps<{
	canCommand: boolean
	canQuery: boolean
}>()

const automation = useTelegramAutomationManagement({
	canCommand: () => props.canCommand,
	canQuery: () => props.canQuery,
})

onMounted(() => {
	if (props.canQuery) void automation.refresh()
})
</script>

<template>
	<TelegramAutomationPanel
		:model="automation.model.value"
		@new-policy="automation.newPolicy"
		@new-template="automation.newTemplate"
		@preview="automation.preview"
		@refresh="automation.refresh"
		@save-policy="automation.savePolicy"
		@save-template="automation.saveTemplate"
		@select-policy="automation.selectPolicy"
		@select-template="automation.selectTemplate"
		@update-policy-account-id="automation.updatePolicyAccountId"
		@update-policy-chat-ids="automation.updatePolicyChatIds"
		@update-policy-enabled="automation.updatePolicyEnabled"
		@update-policy-expires-at="automation.updatePolicyExpiresAt"
		@update-policy-id="automation.updatePolicyId"
		@update-policy-name="automation.updatePolicyName"
		@update-policy-template-id="automation.updatePolicyTemplateId"
		@update-preview-account-id="automation.updatePreviewAccountId"
		@update-preview-chat-id="automation.updatePreviewChatId"
		@update-preview-policy-id="automation.updatePreviewPolicyId"
		@update-preview-variables="automation.updatePreviewVariables"
		@update-template-body="automation.updateTemplateBody"
		@update-template-id="automation.updateTemplateId"
		@update-template-name="automation.updateTemplateName"
		@update-template-variables="automation.updateTemplateVariables"
	/>
</template>
