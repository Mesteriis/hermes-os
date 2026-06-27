<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { WhatsAppProviderCommand } from '../../../shared/communications/types/whatsapp'
import {
	canDeadLetterCommand,
	canRetryCommand,
	commandStatusTone,
	commandTimestamp,
	providerTargetLabel,
} from '../views/WhatsAppRuntimePanel.helpers'

defineProps<{
	providerCommands: WhatsAppProviderCommand[]
	isRuntimeBusy: boolean
}>()

const emit = defineEmits<{
	(event: 'retry', commandId: string): void
	(event: 'dead-letter', commandId: string): void
}>()

const { t } = useI18n()
</script>

<template>
	<section class="panel runtime-card">
		<header>
			<h2>{{ t('Command Audit') }}</h2>
			<span>{{ providerCommands.length }}</span>
		</header>
		<div v-if="providerCommands.length" class="command-list">
			<article
				v-for="command in providerCommands"
				:key="command.command_id"
				class="command-row"
				:data-tone="commandStatusTone(command)"
			>
				<div class="command-head">
					<strong>{{ command.command_kind }}</strong>
					<em>{{ command.status }}</em>
				</div>
				<p class="command-target">{{ providerTargetLabel(command) }}</p>
				<dl class="runtime-details compact">
					<div><dt>{{ t('Capability') }}</dt><dd>{{ command.capability_state }}</dd></div>
					<div><dt>{{ t('Reconciliation') }}</dt><dd>{{ command.reconciliation_status }}</dd></div>
					<div><dt>{{ t('Attempts') }}</dt><dd>{{ command.retry_count }} / {{ command.max_retries }}</dd></div>
					<div><dt>{{ t('Updated') }}</dt><dd>{{ commandTimestamp(command) }}</dd></div>
				</dl>
				<p v-if="command.last_error" class="command-error">{{ command.last_error }}</p>
				<div class="runtime-actions compact">
					<button
						type="button"
						:disabled="isRuntimeBusy || !canRetryCommand(command)"
						@click="emit('retry', command.command_id)"
					>
						<Icon icon="tabler:reload" width="16" height="16" />{{ t('Retry') }}
					</button>
					<button
						type="button"
						:disabled="isRuntimeBusy || !canDeadLetterCommand(command)"
						@click="emit('dead-letter', command.command_id)"
					>
						<Icon icon="tabler:archive-off" width="16" height="16" />{{ t('Dead-letter') }}
					</button>
				</div>
			</article>
		</div>
		<p v-else class="empty-state">{{ t('No WhatsApp provider commands recorded for this account yet.') }}</p>
	</section>
</template>
