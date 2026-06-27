<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type {
	WhatsAppCallSyncItem,
	WhatsAppChatSyncItem,
	WhatsAppContactSyncItem,
	WhatsAppMediaSyncItem,
	WhatsAppMembersSyncItem,
	WhatsAppPresenceSyncItem,
	WhatsappWebMessage,
} from '../../../shared/communications/types/whatsapp'
import {
	callLabel,
	chatLabel,
	chatMeta,
	contactLabel,
	historyLabel,
	mediaLabel,
	memberLabel,
	presenceLabel,
	snapshotTimestamp,
	statusLabel,
	statusPreview,
} from '../views/WhatsAppRuntimePanel.helpers'

const props = defineProps<{
	selectedAccountId: string | null
	selectedSyncChatIdResolved: string | null
	chatItems: WhatsAppChatSyncItem[]
	historyItems: WhatsappWebMessage[]
	memberItems: WhatsAppMembersSyncItem[]
	statusItems: WhatsappWebMessage[]
	presenceItems: WhatsAppPresenceSyncItem[]
	callItems: WhatsAppCallSyncItem[]
	contactItems: WhatsAppContactSyncItem[]
	mediaItems: WhatsAppMediaSyncItem[]
	statusPublishText: string
	isRuntimeBusy: boolean
}>()

const emit = defineEmits<{
	(event: 'select-chat', providerChatId: string): void
	(event: 'update:statusPublishText', value: string): void
	(event: 'publish-status'): void
}>()

const { t } = useI18n()

const statusText = computed({
	get: () => props.statusPublishText,
	set: (value: string) => emit('update:statusPublishText', value),
})
</script>

<template>
	<section class="panel runtime-card">
		<header>
			<h2>{{ t('Projected Snapshots') }}</h2>
			<span>{{ selectedAccountId ?? '-' }}</span>
		</header>
		<div class="snapshot-grid">
			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('Chats') }}</strong>
					<span>{{ chatItems.length }}</span>
				</div>
				<ul v-if="chatItems.length" class="detail-stack compact">
					<li
						v-for="item in chatItems"
						:key="item.provider_chat_id"
						:class="{ selected: item.provider_chat_id === selectedSyncChatIdResolved }"
					>
						<button
							type="button"
							class="snapshot-select"
							:disabled="isRuntimeBusy"
							@click="emit('select-chat', item.provider_chat_id)"
						>
							<strong>{{ chatLabel(item) }}</strong>
							<span>{{ chatMeta(item) }}</span>
						</button>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('No projected chats yet.') }}</p>
			</section>

			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('History') }}</strong>
					<span>{{ historyItems.length }}</span>
				</div>
				<ul v-if="historyItems.length" class="detail-stack compact">
					<li v-for="item in historyItems" :key="item.message_id">
						<strong>{{ historyLabel(item) }}</strong>
						<span>{{ statusPreview(item) }} · {{ snapshotTimestamp(item.occurred_at ?? item.projected_at) }}</span>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('Select a synced chat to inspect recent history.') }}</p>
			</section>

			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('Members') }}</strong>
					<span>{{ memberItems.length }}</span>
				</div>
				<ul v-if="memberItems.length" class="detail-stack compact">
					<li v-for="item in memberItems" :key="item.participant_id">
						<strong>{{ memberLabel(item) }}</strong>
						<span>{{ item.role }}<template v-if="item.status"> · {{ item.status }}</template></span>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('Select a synced chat to inspect roster members.') }}</p>
			</section>

			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('Statuses') }}</strong>
					<span>{{ statusItems.length }}</span>
				</div>
				<label class="runtime-field compact">
					<span>{{ t('Publish text status') }}</span>
					<textarea
						v-model="statusText"
						rows="3"
						maxlength="700"
						:placeholder="t('Share a short status update')"
					/>
				</label>
				<div class="runtime-actions compact">
					<button
						type="button"
						:disabled="isRuntimeBusy || !selectedAccountId || !statusText.trim()"
						@click="emit('publish-status')"
					>
						<Icon icon="tabler:send" width="16" height="16" />{{ t('Publish Status') }}
					</button>
				</div>
				<ul v-if="statusItems.length" class="detail-stack compact">
					<li v-for="item in statusItems" :key="item.message_id">
						<strong>{{ statusLabel(item) }}</strong>
						<span>{{ statusPreview(item) }} · {{ snapshotTimestamp(item.occurred_at ?? item.projected_at) }}</span>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('No projected statuses yet.') }}</p>
			</section>

			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('Presence') }}</strong>
					<span>{{ presenceItems.length }}</span>
				</div>
				<ul v-if="presenceItems.length" class="detail-stack compact">
					<li v-for="item in presenceItems" :key="item.identity_id">
						<strong>{{ presenceLabel(item) }}</strong>
						<span>{{ item.presence_state }} · {{ snapshotTimestamp(item.observed_at) }}</span>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('No projected presence for the selected synced chat yet.') }}</p>
			</section>

			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('Calls') }}</strong>
					<span>{{ callItems.length }}</span>
				</div>
				<ul v-if="callItems.length" class="detail-stack compact">
					<li v-for="item in callItems" :key="item.call_id">
						<strong>{{ item.provider_chat_id }}</strong>
						<span>{{ callLabel(item) }} · {{ snapshotTimestamp(item.started_at ?? item.observed_at) }}</span>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('No projected calls for the selected synced chat yet.') }}</p>
			</section>

			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('Contacts') }}</strong>
					<span>{{ contactItems.length }}</span>
				</div>
				<ul v-if="contactItems.length" class="detail-stack compact">
					<li v-for="item in contactItems" :key="item.identity_id">
						<strong>{{ contactLabel(item) }}</strong>
						<span>{{ item.identity_kind }} · {{ item.address ?? item.provider_identity_id }}</span>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('No projected contacts yet.') }}</p>
			</section>

			<section class="snapshot-card">
				<div class="snapshot-head">
					<strong>{{ t('Media') }}</strong>
					<span>{{ mediaItems.length }}</span>
				</div>
				<ul v-if="mediaItems.length" class="detail-stack compact">
					<li v-for="item in mediaItems" :key="item.attachment_id">
						<strong>{{ mediaLabel(item) }}</strong>
						<span>{{ item.content_type }} · {{ item.provider_chat_id ?? t('unknown') }}</span>
					</li>
				</ul>
				<p v-else class="empty-state compact">{{ t('No projected media for the selected synced chat yet.') }}</p>
			</section>
		</div>
	</section>
</template>
