<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import type { WhatsappCapabilitiesResponse } from '../../../shared/communications/types/whatsapp'

defineProps<{
	runtimeCapabilities: WhatsappCapabilitiesResponse | null
}>()

const { t } = useI18n()
</script>

<template>
	<section class="panel runtime-card">
		<header>
			<h2>{{ t('Capabilities') }}</h2>
			<span>{{ runtimeCapabilities?.version ?? '-' }}</span>
		</header>
		<div v-if="runtimeCapabilities?.provider_shapes?.length" class="shape-grid">
			<article v-for="shape in runtimeCapabilities.provider_shapes" :key="shape.provider_shape" class="shape-card">
				<strong>{{ shape.provider_shape }}</strong>
				<span>{{ shape.status }}</span>
				<small>{{ shape.reason }}</small>
			</article>
		</div>
		<ul v-if="runtimeCapabilities?.capabilities?.length" class="detail-list">
			<li v-for="capability in runtimeCapabilities.capabilities" :key="capability.capability">
				<span>{{ capability.capability }}</span>
				<em>{{ capability.status }}</em>
			</li>
		</ul>
	</section>
</template>
