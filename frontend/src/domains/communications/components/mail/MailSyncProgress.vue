<script setup lang="ts">
import { Icon } from '@/shared/ui'
import type { MailSyncStatus } from '../../types/communications'
import { useMailSyncProgressController } from '../../queries/useMailSyncProgressController'
import '../communicationDomainElements.css'

const props = defineProps<{
  status?: MailSyncStatus | null
}>()

const emit = defineEmits<{
  'visibility-change': [visible: boolean]
}>()

const {
  progressClass,
  icon,
  title,
  subtitle,
  detail,
  movementLabel,
  badgeLabel,
  progressPercent,
} = useMailSyncProgressController(props, {
  visibilityChange: (visible) => emit('visibility-change', visible),
})
</script>

<template>
	<section v-if="status" :class="progressClass" aria-live="polite">
		<span class="mail-sync-progress__ambient" aria-hidden="true" />
		<div class="mail-sync-progress__top">
			<span class="mail-sync-progress__orb" aria-hidden="true">
				<Icon :icon="icon" size="1rem" class="mail-sync-progress__icon" />
			</span>
			<div class="mail-sync-progress__copy">
				<strong class="mail-sync-progress__title">{{ title }}</strong>
				<span class="mail-sync-progress__subtitle">{{ subtitle }}</span>
			</div>
			<span class="mail-sync-progress__badge">{{ badgeLabel }}</span>
		</div>
		<progress
			v-if="progressPercent !== null"
			class="mail-sync-progress__bar"
			:value="progressPercent"
			max="100"
			:aria-label="title"
		/>
		<progress
			v-else
			class="mail-sync-progress__bar"
			:aria-label="title"
		/>
		<div class="mail-sync-progress__meta">
			<span>{{ detail }}</span>
			<span>{{ movementLabel }}</span>
		</div>
	</section>
</template>
