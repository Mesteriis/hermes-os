<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type {
	WhatsappCapabilitiesResponse,
	WhatsappProviderShape,
	WhatsappProviderShapeStatus,
	WhatsappWebProviderKind,
} from '../../../shared/communications/types/whatsapp'

const props = defineProps<{
	capabilities: WhatsappCapabilitiesResponse | null
	liveAccountProviderKind: WhatsappWebProviderKind
	liveAccountShape: WhatsappProviderShape
	liveAccountId: string
	liveAccountDisplayName: string
	liveAccountExternalId: string
	liveAccountDeviceName: string
	liveAccountLocalStatePath: string
	liveAccountSupportsDeviceFields: boolean
	selectedProviderShapeMeta: WhatsappProviderShapeStatus | null
	liveAccountSessionMode: string
	isSubmitting: boolean
}>()

const emit = defineEmits<{
	(event: 'update:liveAccountShape', value: WhatsappProviderShape): void
	(event: 'update:liveAccountId', value: string): void
	(event: 'update:liveAccountDisplayName', value: string): void
	(event: 'update:liveAccountExternalId', value: string): void
	(event: 'update:liveAccountDeviceName', value: string): void
	(event: 'update:liveAccountLocalStatePath', value: string): void
	(event: 'create-live-account'): void
}>()

const { t } = useI18n()

const liveAccountShapeModel = computed({
	get: () => props.liveAccountShape,
	set: (value: WhatsappProviderShape) => emit('update:liveAccountShape', value),
})
const liveAccountIdModel = computed({
	get: () => props.liveAccountId,
	set: (value: string) => emit('update:liveAccountId', value),
})
const liveAccountDisplayNameModel = computed({
	get: () => props.liveAccountDisplayName,
	set: (value: string) => emit('update:liveAccountDisplayName', value),
})
const liveAccountExternalIdModel = computed({
	get: () => props.liveAccountExternalId,
	set: (value: string) => emit('update:liveAccountExternalId', value),
})
const liveAccountDeviceNameModel = computed({
	get: () => props.liveAccountDeviceName,
	set: (value: string) => emit('update:liveAccountDeviceName', value),
})
const liveAccountLocalStatePathModel = computed({
	get: () => props.liveAccountLocalStatePath,
	set: (value: string) => emit('update:liveAccountLocalStatePath', value),
})
</script>

<template>
	<section class="panel runtime-card">
		<header>
			<h2>{{ t('Account Provisioning') }}</h2>
			<span>{{ liveAccountProviderKind }}</span>
		</header>
		<div class="provisioning-grid">
			<label class="runtime-field compact">
				<span>{{ t('Provider shape') }}</span>
				<select v-model="liveAccountShapeModel">
					<option
						v-for="shape in capabilities?.provider_shapes ?? []"
						:key="shape.provider_shape"
						:value="shape.provider_shape"
					>
						{{ shape.provider_shape }}
					</option>
				</select>
			</label>
			<label class="runtime-field compact">
				<span>{{ t('Account id') }}</span>
				<input v-model="liveAccountIdModel" autocomplete="off" />
			</label>
			<label class="runtime-field compact">
				<span>{{ t('Display name') }}</span>
				<input v-model="liveAccountDisplayNameModel" autocomplete="off" />
			</label>
			<label class="runtime-field compact">
				<span>{{ t('External account id') }}</span>
				<input v-model="liveAccountExternalIdModel" autocomplete="off" />
			</label>
			<label v-if="liveAccountSupportsDeviceFields" class="runtime-field compact">
				<span>{{ t('Device name') }}</span>
				<input v-model="liveAccountDeviceNameModel" autocomplete="off" />
			</label>
			<label class="runtime-field compact">
				<span>{{ t('Local state path') }}</span>
				<input v-model="liveAccountLocalStatePathModel" autocomplete="off" />
			</label>
		</div>
		<div class="evidence-row">
			<strong>{{ t('Provider posture') }}</strong>
			<p>{{ selectedProviderShapeMeta?.reason ?? t('No provider-shape metadata available') }}</p>
		</div>
		<dl class="runtime-details compact">
			<div><dt>{{ t('Runtime mode') }}</dt><dd>{{ liveAccountSessionMode }}</dd></div>
			<div><dt>{{ t('Capability status') }}</dt><dd>{{ selectedProviderShapeMeta?.status ?? '-' }}</dd></div>
		</dl>
		<div class="runtime-actions">
			<button type="button" :disabled="isSubmitting" @click="emit('create-live-account')">
				<Icon icon="tabler:user-plus" width="16" height="16" />{{ t('Create Live Account') }}
			</button>
		</div>
	</section>
</template>
