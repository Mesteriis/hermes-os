<script setup lang="ts">
import { computed, ref } from 'vue'
import { Badge, Button, Icon, TreeSelect } from '@/shared/ui'
import type { TreeSelectOption } from '@/shared/ui'
import '../communicationDomainElements.css'
import {
  communicationChannelDirectChatCount,
  communicationChannelProviderIconName,
  communicationChannelProviderLabel,
  type CommunicationChannelDirectChatModel,
  type CommunicationChannelDirectFolderModel,
  type CommunicationChannelRoomModel
} from '../communicationDomainElements'
import ChannelListItem from './ChannelListItem.vue'

const props = defineProps<{
  providerValue: string
  providerOptions: TreeSelectOption[]
  rooms: readonly CommunicationChannelRoomModel[]
  directChatFolders: readonly CommunicationChannelDirectFolderModel[]
}>()

const emit = defineEmits<{
  select: [room: CommunicationChannelRoomModel]
  'select-direct-chat': [chat: CommunicationChannelDirectChatModel]
}>()

const providerValue = ref(props.providerValue)
const directChatCount = computed(() => communicationChannelDirectChatCount(props.directChatFolders))
</script>

<template>
	<aside class="communication-channel-rail" aria-label="Channel rooms">
		<section class="communication-channel-rail__actions" aria-label="Channel search">
			<label class="communication-channel-search">
				<Icon icon="tabler:search" size="1rem" class="communication-channel-search__icon" />
				<span class="communication-channel-search__label">Search channels</span>
				<input
					class="communication-channel-search__input"
					type="search"
					placeholder="Search Zulip, Slack, Discord, channels"
					aria-label="Search channels"
				/>
			</label>
			<Button
				class="communication-channel-rail__tool hermes-icon-button"
				variant="outline"
				size="sm"
				icon="tabler:refresh"
				aria-label="Refresh channels"
				title="Refresh channels"
			/>
		</section>

		<header class="communication-channel-rail__header">
			<TreeSelect
				v-model="providerValue"
				class="communication-channel-provider-select"
				:options="providerOptions"
				placeholder="Select channel provider"
				aria-label="Channel provider"
				empty-label="No channel providers"
			/>
		</header>

		<div class="communication-channel-rail__sections" aria-label="Channel and direct chat groups">
			<details class="communication-channel-rail__section" open>
				<summary class="communication-channel-rail__section-summary">
					<span>
						<Icon icon="tabler:hash" size="1rem" />
						Channels
					</span>
					<Badge variant="neutral">{{ rooms.length }}</Badge>
				</summary>
				<div class="communication-channel-rail__section-body">
					<ChannelListItem
						v-for="room in rooms"
						:key="room.id"
						:room="room"
						@select="emit('select', $event)"
					/>
				</div>
			</details>

			<details class="communication-channel-rail__section" open>
				<summary class="communication-channel-rail__section-summary">
					<span>
						<Icon icon="tabler:messages" size="1rem" />
						Direct chats
					</span>
					<Badge variant="neutral">{{ directChatCount }}</Badge>
				</summary>
				<div class="communication-channel-rail__section-body communication-channel-direct-folders">
					<details
						v-for="folder in directChatFolders"
						:key="folder.id"
						class="communication-channel-direct-folder"
						:open="folder.expanded !== false"
					>
						<summary class="communication-channel-direct-folder__summary">
							<span>
								<Icon icon="tabler:folder" size="0.95rem" />
								{{ folder.label }}
							</span>
							<Badge variant="neutral">{{ folder.chats.length }}</Badge>
						</summary>
						<div class="communication-channel-direct-folder__items">
							<button
								v-for="chat in folder.chats"
								:key="chat.id"
								type="button"
								:class="[
									'communication-channel-direct-chat',
									chat.selected && 'communication-channel-direct-chat--selected'
								]"
								:aria-label="`${communicationChannelProviderLabel(chat.providerKind)}, ${chat.label}`"
								@click="emit('select-direct-chat', chat)"
							>
								<span class="communication-channel-direct-chat__lead">
									<span class="communication-channel-direct-chat__avatar">{{ chat.avatarLabel }}</span>
									<span
										class="communication-channel-direct-chat__provider"
										:aria-label="communicationChannelProviderLabel(chat.providerKind)"
									>
										<Icon :icon="communicationChannelProviderIconName(chat.providerKind)" size="0.75rem" />
									</span>
								</span>
								<span class="communication-channel-direct-chat__body">
									<strong>{{ chat.label }}</strong>
									<small>{{ chat.description }}</small>
									<span class="communication-channel-direct-chat__meta">
										<span v-if="chat.kindLabel">{{ chat.kindLabel }}</span>
										<span v-if="chat.lastActivityLabel">{{ chat.lastActivityLabel }}</span>
									</span>
								</span>
								<span class="communication-channel-room__signals">
									<span v-if="chat.unreadCount" class="communication-inbox-item__unread">{{ chat.unreadCount }}</span>
									<Badge v-if="chat.mentionCount" variant="warning">@{{ chat.mentionCount }}</Badge>
								</span>
							</button>
						</div>
					</details>
				</div>
			</details>
		</div>
	</aside>
</template>
