<script setup lang="ts">
import { Icon } from '@/shared/ui'
import type { MailMessageFlagModel } from './mailMessageFlagModel'
import MailMessageFlagActions from './MailMessageFlagActions.vue'
import type { MailMessageLocationModel } from './mailMessageLocationModel'
import MailMessageLocationActions from './MailMessageLocationActions.vue'
import type { MailMessagePermanentDeleteModel } from './mailMessagePermanentDeleteModel'
import MailMessagePermanentDeleteActions from './MailMessagePermanentDeleteActions.vue'
import type { MailMessageDetailCard } from './mailOperationalReadModel'

defineProps<{
	detail: MailMessageDetailCard | null
	flagModel: MailMessageFlagModel
	locationModel: MailMessageLocationModel
	permanentDeleteModel: MailMessagePermanentDeleteModel
}>()

const emit = defineEmits<{
	close: []
	flagRefreshStatus: []
	flagSetRead: [targetValue: boolean]
	flagSetStarred: [targetValue: boolean]
	locationArchive: []
	locationMove: []
	locationRefreshStatus: []
	locationRestore: []
	locationSelectTargetFolder: [folderId: string]
	locationTrash: []
	permanentDelete: []
	permanentDeleteRefreshStatus: []
	permanentDeleteSetConfirmed: [confirmed: boolean]
}>()
</script>

<template>
	<aside class="mail-workspace-inspector" aria-label="Mail details">
		<header>
			<div>
				<h2>Details</h2>
				<p>{{ detail?.subject || 'No message selected' }}</p>
			</div>
			<button type="button" title="Close" @click="emit('close')">
				<Icon icon="tabler:x" size="1rem" />
			</button>
		</header>

		<nav class="mail-inspector-tabs" aria-label="Inspector sections">
			<button type="button" class="active">Context</button>
			<button type="button" disabled>Signals</button>
			<button type="button" disabled>Activity</button>
		</nav>

		<div class="mail-inspector-body">
			<section class="mail-inspector-card">
				<h3>Message context</h3>
				<dl>
					<div><dt>Sender</dt><dd>{{ detail?.sender || '—' }}</dd></div>
					<div><dt>Recipients</dt><dd>{{ detail?.recipients || '—' }}</dd></div>
					<div><dt>Folders</dt><dd>{{ detail?.folders || '—' }}</dd></div>
					<div><dt>Evidence</dt><dd>{{ detail?.evidenceState || '—' }}</dd></div>
				</dl>
			</section>

			<MailMessageFlagActions
				:model="flagModel"
				@refresh-status="emit('flagRefreshStatus')"
				@set-read="emit('flagSetRead', $event)"
				@set-starred="emit('flagSetStarred', $event)"
			/>
			<MailMessageLocationActions
				:model="locationModel"
				@archive="emit('locationArchive')"
				@move="emit('locationMove')"
				@refresh-status="emit('locationRefreshStatus')"
				@restore="emit('locationRestore')"
				@select-target-folder="emit('locationSelectTargetFolder', $event)"
				@trash="emit('locationTrash')"
			/>
			<MailMessagePermanentDeleteActions
				:model="permanentDeleteModel"
				@delete="emit('permanentDelete')"
				@refresh-status="emit('permanentDeleteRefreshStatus')"
				@set-confirmed="emit('permanentDeleteSetConfirmed', $event)"
			/>
		</div>
	</aside>
</template>
