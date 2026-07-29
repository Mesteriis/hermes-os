<script setup lang="ts">
import { Icon } from '@/shared/ui'
import type { MailOperationalReadModel } from './mailOperationalReadModel'
import type { MailSyncModel } from './mailSyncModel'

defineProps<{
	readModel: MailOperationalReadModel
	searchQuery: string
	syncModel: MailSyncModel
}>()

const emit = defineEmits<{
	compose: []
	refresh: []
	selectConnection: [connectionId: string]
	showSyncHealth: []
	sync: []
	toggleInspector: []
	updateSearch: [value: string]
}>()
</script>

<template>
	<header class="mail-workspace-toolbar">
		<div class="mail-workspace-toolbar__title">
			<h1>Communications <span>/</span> Mail</h1>
			<p>{{ readModel.selectedConnectionId || 'No admitted Mail account' }}</p>
		</div>

		<label class="mail-workspace-search">
			<Icon icon="tabler:search" size="1rem" />
			<input
				:value="searchQuery"
				placeholder="Search mail…"
				autocomplete="off"
				@input="emit('updateSearch', ($event.target as HTMLInputElement).value)"
			>
		</label>

		<label class="mail-workspace-account">
			<span>Account</span>
			<select
				:value="readModel.selectedConnectionId"
				:disabled="!readModel.canQuery || readModel.connections.length === 0"
				@change="emit('selectConnection', ($event.target as HTMLSelectElement).value)"
			>
				<option v-if="readModel.connections.length === 0" value="">No account</option>
				<option v-for="connection in readModel.connections" :key="connection.id" :value="connection.id">
					{{ connection.label }}
				</option>
			</select>
		</label>

		<div class="mail-workspace-toolbar__actions">
			<button type="button" title="Refresh" :disabled="readModel.status === 'loading'" @click="emit('refresh')">
				<Icon icon="tabler:refresh" size="1rem" />
				<span class="mail-workspace-toolbar__action-label">Refresh</span>
			</button>
			<button type="button" title="Sync health" @click="emit('showSyncHealth')">
				<Icon icon="tabler:activity-heartbeat" size="1rem" />
				<span class="mail-workspace-toolbar__action-label">Sync health</span>
			</button>
			<button type="button" title="Details" @click="emit('toggleInspector')">
				<Icon icon="tabler:layout-sidebar-right" size="1rem" />
				<span class="mail-workspace-toolbar__action-label">Details</span>
			</button>
			<button
				type="button"
				title="Sync now"
				:disabled="!syncModel.canSync || syncModel.busy"
				@click="emit('sync')"
			>
				<Icon icon="tabler:cloud-download" size="1rem" />
				<span class="mail-workspace-toolbar__action-label">{{ syncModel.busy ? 'Syncing…' : 'Sync' }}</span>
			</button>
			<button type="button" class="mail-workspace-toolbar__compose" @click="emit('compose')">
				<Icon icon="tabler:edit" size="1rem" />
				Compose
			</button>
		</div>
	</header>
</template>
