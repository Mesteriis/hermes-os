<script setup lang="ts">
import { Badge, Icon } from '@/shared/ui'
import {
  communicationChannelProviderIconName,
  communicationChannelProviderLabel,
  type CommunicationChannelRoomModel
} from '../communicationDomainElements'
import '../communicationDomainElements.css'

defineProps<{
  room: CommunicationChannelRoomModel
}>()

const emit = defineEmits<{
  select: [room: CommunicationChannelRoomModel]
}>()
</script>

<template>
	<button
		type="button"
		:class="['communication-channel-room', room.selected && 'communication-channel-room--selected']"
		:aria-label="`${communicationChannelProviderLabel(room.providerKind)}, #${room.label}`"
		@click="emit('select', room)"
	>
		<span class="communication-channel-room__provider" :aria-label="communicationChannelProviderLabel(room.providerKind)">
			<Icon :icon="communicationChannelProviderIconName(room.providerKind)" size="1rem" />
		</span>
		<span class="communication-channel-room__body">
			<strong>#{{ room.label }}</strong>
			<small>{{ room.description }}</small>
			<span class="communication-channel-room__meta">
				<span v-if="room.topicCountLabel">{{ room.topicCountLabel }}</span>
				<span v-if="room.lastActivityLabel">{{ room.lastActivityLabel }}</span>
			</span>
		</span>
		<span class="communication-channel-room__signals">
			<span v-if="room.unreadCount" class="communication-inbox-item__unread">{{ room.unreadCount }}</span>
			<Badge v-if="room.mentionCount" variant="warning">@{{ room.mentionCount }}</Badge>
		</span>
	</button>
</template>
