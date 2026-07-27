<script setup lang="ts">
import Icon from '../Icon.vue'
import './integrationAccountLifecycleCard.css'

defineProps<{
	eyebrow: string
	title: string
	description: string
	tone: 'mail' | 'telegram' | 'whatsapp' | 'zulip'
	icon: string
	accountState: string
	busy: boolean
	message?: string
	messageTone?: 'neutral' | 'success' | 'error'
}>()
</script>

<template>
	<section class="integration-account-lifecycle" :data-provider-tone="tone">
		<header class="integration-account-lifecycle__header">
			<span class="integration-account-lifecycle__icon"><Icon :icon="icon" /></span>
			<div>
				<small>{{ eyebrow }}</small>
				<h3>{{ title }}</h3>
				<p>{{ description }}</p>
			</div>
			<strong>{{ accountState }}</strong>
		</header>

		<div class="integration-account-lifecycle__summary">
			<slot name="summary" />
		</div>

		<div v-if="$slots.default" class="integration-account-lifecycle__controls">
			<slot />
		</div>

		<footer>
			<p
				v-if="message"
				:class="`integration-account-lifecycle__message--${messageTone ?? 'neutral'}`"
				aria-live="polite"
			>
				{{ message }}
			</p>
			<span v-else />
			<div class="integration-account-lifecycle__actions">
				<slot name="actions" />
			</div>
		</footer>
	</section>
</template>
