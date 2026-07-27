<script setup lang="ts">
import type { MailSyncModel } from './mailSyncModel'

defineProps<{ model: MailSyncModel }>()
const emit = defineEmits<{ sync: [] }>()
</script>

<template>
	<section class="mail-sync-card">
		<div>
			<strong>Inbox sync</strong>
			<small>{{ model.summary || 'No sync run in this session.' }}</small>
			<small v-if="model.notice" role="status">{{ model.notice }}</small>
		</div>
		<button type="button" :disabled="!model.canSync || model.busy" @click="emit('sync')">
			{{ model.busy ? 'Syncing…' : 'Sync now' }}
		</button>
	</section>
</template>
