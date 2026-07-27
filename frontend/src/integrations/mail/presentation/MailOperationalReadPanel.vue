<script setup lang="ts">
import type { MailOperationalReadModel } from './mailOperationalReadModel'
import type { MailMessageFlagModel } from './mailMessageFlagModel'
import MailMessageFlagActions from './MailMessageFlagActions.vue'
import './mailOperationalReadPanel.css'

defineProps<{
	flagModel: MailMessageFlagModel
	model: MailOperationalReadModel
}>()

const emit = defineEmits<{
	loadMoreFolders: []
	loadMoreMessages: []
	loadMoreThreads: []
	refresh: []
	selectConnection: [connectionId: string]
	selectFolder: [folderId: string]
	selectMessage: [providerMessageId: string]
	selectThread: [providerThreadId: string]
	flagRefreshStatus: []
	flagSetRead: [targetValue: boolean]
	flagSetStarred: [targetValue: boolean]
}>()
</script>

<template>
	<section class="mail-read-panel" :aria-busy="model.status === 'loading'">
		<header class="mail-read-panel__header">
			<div>
				<span>Operational projection</span>
				<h2>Mailbox</h2>
				<p>Mail-owned folders, exact provider threads and bounded message evidence.</p>
			</div>
			<form @submit.prevent="emit('refresh')">
				<label for="mail-connection">Admitted connection</label>
				<div>
					<select
						id="mail-connection"
						:value="model.selectedConnectionId"
						:disabled="!model.canQuery || model.connections.length === 0"
						@change="emit('selectConnection', ($event.target as HTMLSelectElement).value)"
					>
						<option v-if="model.connections.length === 0" value="">No connection</option>
						<option v-for="connection in model.connections" :key="connection.id" :value="connection.id">
							{{ connection.label }}
						</option>
					</select>
					<button type="submit" :disabled="!model.canQuery || !model.selectedConnectionId || model.status === 'loading'">
						{{ model.status === 'loading' ? 'Loading…' : 'Refresh' }}
					</button>
				</div>
			</form>
		</header>

		<p
			v-if="model.statusMessage"
			class="mail-read-panel__status"
			:role="model.status === 'error' ? 'alert' : 'status'"
		>
			{{ model.statusMessage }}
		</p>

		<div class="mail-read-workbench">
			<aside class="mail-read-pane">
				<header><h3>Folders</h3><span>{{ model.folders.length }}</span></header>
				<button
					v-for="folder in model.folders"
					:key="folder.id"
					type="button"
					class="mail-read-row"
					:class="{ selected: folder.selected }"
					:aria-pressed="folder.selected"
					@click="emit('selectFolder', folder.id)"
				>
					<strong>{{ folder.label }}</strong>
					<small>{{ folder.meta }}</small>
				</button>
				<button v-if="model.hasMoreFolders" type="button" class="mail-read-more" @click="emit('loadMoreFolders')">
					Load more folders
				</button>
			</aside>

			<section class="mail-read-pane">
				<header><h3>Threads</h3><span>{{ model.threads.length }}</span></header>
				<button
					v-for="thread in model.threads"
					:key="thread.id"
					type="button"
					class="mail-read-row"
					:class="{ selected: thread.selected, unread: thread.unread }"
					:aria-pressed="thread.selected"
					@click="emit('selectThread', thread.id)"
				>
					<strong>{{ thread.subject }}</strong>
					<p>{{ thread.snippet }}</p>
					<small>{{ thread.meta }}</small>
				</button>
				<button v-if="model.hasMoreThreads" type="button" class="mail-read-more" @click="emit('loadMoreThreads')">
					Load more threads
				</button>
			</section>

			<section class="mail-read-pane">
				<header><h3>Messages</h3><span>{{ model.messages.length }}</span></header>
				<button
					v-for="message in model.messages"
					:key="message.id"
					type="button"
					class="mail-read-row"
					:class="{ selected: message.selected, unread: message.unread }"
					:aria-pressed="message.selected"
					@click="emit('selectMessage', message.id)"
				>
					<div><strong>{{ message.sender }}</strong><small>{{ message.meta }}</small></div>
					<b>{{ message.subject }}</b>
					<p>{{ message.snippet }}</p>
					<small v-if="message.hasAttachments">Has attachments</small>
				</button>
				<button v-if="model.hasMoreMessages" type="button" class="mail-read-more" @click="emit('loadMoreMessages')">
					Load more messages
				</button>
			</section>
		</div>

		<article v-if="model.detail" class="mail-message-detail">
			<header>
				<div><span>Selected provider evidence</span><h3>{{ model.detail.subject }}</h3></div>
				<small>{{ model.detail.meta }}</small>
			</header>
			<dl>
				<div><dt>From</dt><dd>{{ model.detail.sender }}</dd></div>
				<div><dt>To</dt><dd>{{ model.detail.recipients }}</dd></div>
				<div><dt>Folders</dt><dd>{{ model.detail.folders }}</dd></div>
				<div><dt>Flags</dt><dd>{{ model.detail.flags }}</dd></div>
				<div><dt>Evidence</dt><dd>{{ model.detail.evidenceState }}</dd></div>
			</dl>
			<p>{{ model.detail.snippet }}</p>
			<small>{{ model.detail.contentState }}</small>
			<MailMessageFlagActions
				:model="flagModel"
				@refresh-status="emit('flagRefreshStatus')"
				@set-read="emit('flagSetRead', $event)"
				@set-starred="emit('flagSetStarred', $event)"
			/>
		</article>
	</section>
</template>
