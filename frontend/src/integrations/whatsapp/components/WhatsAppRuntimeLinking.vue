<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type {
	WhatsAppPairCodeSession,
	WhatsAppQrLinkSession,
	WhatsAppRuntimeStatus,
} from '../../../shared/communications/types/whatsapp'

const props = defineProps<{
	runtimeStatus: WhatsAppRuntimeStatus | null
	selectedAccountId: string | null
	isRuntimeBusy: boolean
	pairCodePhoneNumber: string
	activeQrSession: WhatsAppQrLinkSession | null
	activePairCodeSession: WhatsAppPairCodeSession | null
}>()

const emit = defineEmits<{
	(event: 'update:pairCodePhoneNumber', value: string): void
	(event: 'set-runtime-state', action: 'qr' | 'pair_code'): void
}>()

const { t } = useI18n()

const pairCodePhoneNumberModel = computed({
	get: () => props.pairCodePhoneNumber,
	set: (value: string) => emit('update:pairCodePhoneNumber', value),
})
</script>

<template>
	<section class="panel runtime-card">
		<header>
			<h2>{{ t('Linking') }}</h2>
			<span>{{ runtimeStatus?.status ?? '-' }}</span>
		</header>
		<div class="runtime-actions">
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId" @click="emit('set-runtime-state', 'qr')">
				<Icon icon="tabler:qrcode" width="16" height="16" />{{ t('Start QR Link') }}
			</button>
			<button type="button" :disabled="isRuntimeBusy || !selectedAccountId || !pairCodePhoneNumberModel.trim()" @click="emit('set-runtime-state', 'pair_code')">
				<Icon icon="tabler:device-mobile-message" width="16" height="16" />{{ t('Start Pair Code') }}
			</button>
		</div>
		<label class="runtime-field">
			<span>{{ t('Phone number') }}</span>
			<input v-model="pairCodePhoneNumberModel" autocomplete="off" placeholder="+34..." />
		</label>
		<div v-if="activeQrSession" class="evidence-row">
			<strong>{{ t('QR session') }}</strong>
			<p>{{ activeQrSession.status }} · {{ activeQrSession.setup_id }}</p>
			<div v-if="activeQrSession.qr_svg" class="qr-preview" v-html="activeQrSession.qr_svg"></div>
		</div>
		<div v-if="activePairCodeSession" class="evidence-row">
			<strong>{{ t('Pair code') }}</strong>
			<p>{{ activePairCodeSession.pair_code ?? t('blocked') }} · {{ activePairCodeSession.phone_number }}</p>
		</div>
	</section>
</template>
