<script setup lang="ts">
import Icon from '../Icon.vue'
import './integrationAccountSetupCard.css'

defineProps<{
	eyebrow: string
	title: string
	description: string
	tone: 'mail' | 'telegram' | 'whatsapp' | 'zulip'
	icon: string
	accountState: string
	submitLabel: string
	busy: boolean
	disabled: boolean
	message?: string
	messageTone?: 'neutral' | 'success' | 'error'
	expanded?: boolean
}>()

defineEmits<{ submit: [] }>()
</script>

<template>
	<section class="integration-account-setup" :data-provider-tone="tone">
		<header class="integration-account-setup__header">
			<span class="integration-account-setup__icon"><Icon :icon="icon" /></span>
			<div>
				<small>{{ eyebrow }}</small>
				<h3>{{ title }}</h3>
				<p>{{ description }}</p>
			</div>
			<strong>{{ accountState }}</strong>
		</header>

		<details :open="expanded">
			<summary>
				<span><Icon icon="tabler:user-plus" /> Add account</span>
				<Icon icon="tabler:chevron-down" />
			</summary>
			<form class="integration-account-setup__form" @submit.prevent="$emit('submit')">
				<div class="integration-account-setup__fields">
					<slot />
				</div>
				<footer>
					<p
						v-if="message"
						:class="`integration-account-setup__message--${messageTone ?? 'neutral'}`"
						aria-live="polite"
					>
						{{ message }}
					</p>
					<button type="submit" :disabled="disabled || busy">
						<Icon :icon="busy ? 'tabler:loader-2' : 'tabler:plug-connected'" />
						{{ busy ? 'Applying…' : submitLabel }}
					</button>
				</footer>
			</form>
		</details>
	</section>
</template>
