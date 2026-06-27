<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type {
	WhatsAppRuntimeHealth,
	WhatsAppRuntimeStatus,
	WhatsAppWebCompanionManifest,
	WhatsappAccountSummary,
	WhatsappCapabilitiesResponse,
} from '../../../shared/communications/types/whatsapp'
import {
	runtimeHealthCheckDetail,
	runtimeHealthCheckStatus,
	snapshotTimestamp,
} from '../views/WhatsAppRuntimePanel.helpers'

type RuntimeAction = 'start' | 'stop' | 'revoke' | 'relink' | 'rotate' | 'remove'

defineProps<{
	selectedAccountId: string | null
	selectedAccountSummary: WhatsappAccountSummary | null
	runtimeStatus: WhatsAppRuntimeStatus | null
	runtimeCapabilities: WhatsappCapabilitiesResponse | null
	runtimeHealth: WhatsAppRuntimeHealth | null
	runtimeHealthChecks: Array<[string, unknown]>
	companionOpenManifest: WhatsAppWebCompanionManifest | null
	canOpenWebCompanion: boolean
	isRuntimeBusy: boolean
}>()

const emit = defineEmits<{
	(event: 'open-companion'): void
	(event: 'set-runtime-state', action: RuntimeAction): void
}>()

const { t } = useI18n()
</script>

<template>
	<section class="panel runtime-card">
		<header>
			<h2>{{ t('Runtime Control') }}</h2>
			<span>{{ selectedAccountId ?? '-' }}</span>
		</header>
		<div v-if="selectedAccountSummary" class="evidence-row">
			<strong>{{ selectedAccountSummary.display_name }}</strong>
			<p>
				{{ selectedAccountSummary.provider_shape ?? selectedAccountSummary.provider_kind }}
				· {{ selectedAccountSummary.runtime ?? 'unknown' }}
				· {{ selectedAccountSummary.lifecycle_state ?? 'unknown' }}
			</p>
		</div>
		<div class="runtime-actions">
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId || !canOpenWebCompanion" @click="emit('open-companion')">
				<Icon icon="tabler:brand-whatsapp" width="16" height="16" />{{ t('Open Companion') }}
			</button>
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId" @click="emit('set-runtime-state', 'start')">
				<Icon icon="tabler:player-play" width="16" height="16" />{{ t('Start') }}
			</button>
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId" @click="emit('set-runtime-state', 'stop')">
				<Icon icon="tabler:player-stop" width="16" height="16" />{{ t('Stop') }}
			</button>
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId" @click="emit('set-runtime-state', 'revoke')">
				<Icon icon="tabler:shield-x" width="16" height="16" />{{ t('Revoke') }}
			</button>
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId" @click="emit('set-runtime-state', 'relink')">
				<Icon icon="tabler:link-plus" width="16" height="16" />{{ t('Relink') }}
			</button>
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId" @click="emit('set-runtime-state', 'rotate')">
				<Icon icon="tabler:rotate-2" width="16" height="16" />{{ t('Rotate') }}
			</button>
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId" @click="emit('set-runtime-state', 'remove')">
				<Icon icon="tabler:trash" width="16" height="16" />{{ t('Remove') }}
			</button>
		</div>
		<dl class="runtime-details">
			<div><dt>{{ t('Lifecycle') }}</dt><dd>{{ runtimeStatus?.status ?? '-' }}</dd></div>
			<div><dt>{{ t('Provider shape') }}</dt><dd>{{ runtimeStatus?.provider_shape ?? runtimeCapabilities?.account_scope?.provider_shape ?? '-' }}</dd></div>
			<div><dt>{{ t('Runtime kind') }}</dt><dd>{{ runtimeStatus?.runtime_kind ?? runtimeCapabilities?.runtime_mode ?? '-' }}</dd></div>
			<div><dt>{{ t('Restore') }}</dt><dd>{{ runtimeStatus?.session_restore_available ? t('available') : t('blocked') }}</dd></div>
			<div><dt>{{ t('Health') }}</dt><dd>{{ runtimeHealth?.healthy ? t('healthy') : runtimeHealth?.status ?? '-' }}</dd></div>
			<div><dt>{{ t('Last error') }}</dt><dd>{{ runtimeStatus?.last_error ?? '-' }}</dd></div>
		</dl>
		<div v-if="runtimeStatus?.runtime_blockers?.length" class="evidence-row">
			<strong>{{ t('Runtime blockers') }}</strong>
			<p>{{ runtimeStatus.runtime_blockers.join(', ') }}</p>
		</div>
		<div v-if="companionOpenManifest" class="evidence-row">
			<strong>{{ t('WebView companion') }}</strong>
			<p>
				{{ companionOpenManifest.window_label }}
				· {{ companionOpenManifest.event_extractor.relay_channel }}
				· {{ companionOpenManifest.event_extractor.runtime_bridge_dispatch }}
			</p>
		</div>
		<div v-if="runtimeHealthChecks.length" class="evidence-row">
			<strong>{{ t('Health diagnostics') }}</strong>
			<small>{{ snapshotTimestamp(runtimeHealth?.checked_at) }}</small>
			<ul class="detail-list">
				<li v-for="[checkName, checkValue] in runtimeHealthChecks" :key="checkName">
					<span>{{ checkName }}</span>
					<em>{{ runtimeHealthCheckStatus(checkValue) }}</em>
					<small v-if="runtimeHealthCheckDetail(checkValue)">{{ runtimeHealthCheckDetail(checkValue) }}</small>
				</li>
			</ul>
		</div>
	</section>
</template>
