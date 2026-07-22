<script setup lang="ts">
import { useI18n } from '@/platform/i18n'
import { Button, ButtonGroup, DropdownMenu, DropdownMenuItem, Spacer } from '@/shared/ui'
import type { MessengerConversationRuntimeAction } from '@/shared/communications/types/messengerRuntimeActions'
import { useMessengerActionController } from '../../queries/useMessengerActionController'
import '../communicationDomainElements.css'

const emit = defineEmits<{
  'conversation-action': [action: MessengerConversationRuntimeAction]
  'toggle-inspector': []
}>()

const props = withDefaults(defineProps<{
  inspectorVisible?: boolean
  isActionRunning?: boolean
  showInspectorToggle?: boolean
}>(), {
  inspectorVisible: true,
  isActionRunning: false,
  showInspectorToggle: true
})

const controller = useMessengerActionController(
  props,
  {
    conversationAction: (action) => emit('conversation-action', action),
    toggleInspector: () => emit('toggle-inspector'),
  },
)

const {
  handleMarkRead,
  handleMarkUnread,
  handlePin,
  handleUnpin,
  handleMute,
  handleUnmute,
  handleArchive,
  handleUnarchive,
  handleSyncLatest,
  handleSyncOlder,
  handleToggleInspector,
} = controller

const { t } = useI18n()
</script>

<template>
	<nav class="messenger-action-bar" :aria-label="t('Messenger conversation actions')">
		<ButtonGroup
			:aria-label="t('Conversation state actions')"
			class="messenger-action-bar__button-group"
		>
			<Button
				class="messenger-action-bar__button hermes-icon-button"
				variant="outline"
				size="sm"
				icon="tabler:mail-opened"
				:aria-label="t('Mark read')"
				:disabled="isActionRunning"
				:title="t('Mark read')"
				@click="handleMarkRead"
			/>
			<DropdownMenu align="start" :side-offset="8">
				<template #trigger>
					<Button
						class="messenger-action-bar__button hermes-icon-button"
						variant="outline"
						size="sm"
						icon="tabler:chevron-down"
						:aria-label="t('Open state actions')"
						:disabled="isActionRunning"
						:title="t('Open state actions')"
					/>
				</template>
				<DropdownMenuItem icon="tabler:mail" :disabled="isActionRunning" @click="handleMarkUnread">
					{{ t('Mark unread') }}
				</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:pin" :disabled="isActionRunning" @click="handlePin">
					{{ t('Pin') }}
				</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:pinned-off" :disabled="isActionRunning" @click="handleUnpin">
					{{ t('Unpin') }}
				</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:bell-off" :disabled="isActionRunning" @click="handleMute">
					{{ t('Mute') }}
				</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:bell" :disabled="isActionRunning" @click="handleUnmute">
					{{ t('Unmute') }}
				</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:archive" :disabled="isActionRunning" @click="handleArchive">
					{{ t('Archive') }}
				</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:archive-off" :disabled="isActionRunning" @click="handleUnarchive">
					{{ t('Unarchive') }}
				</DropdownMenuItem>
			</DropdownMenu>
		</ButtonGroup>

		<ButtonGroup
			:aria-label="t('Synchronization actions')"
			class="messenger-action-bar__button-group"
		>
			<Button
				class="messenger-action-bar__button hermes-icon-button"
				variant="outline"
				size="sm"
				icon="tabler:refresh"
				:aria-label="t('Sync latest history')"
				:disabled="isActionRunning"
				:title="t('Sync latest history')"
				@click="handleSyncLatest"
			/>
			<DropdownMenu align="start" :side-offset="8">
				<template #trigger>
					<Button
						class="messenger-action-bar__button hermes-icon-button"
						variant="outline"
						size="sm"
						icon="tabler:chevron-down"
						:aria-label="t('Open synchronization actions')"
						:disabled="isActionRunning"
						:title="t('Open synchronization actions')"
					/>
				</template>
					<DropdownMenuItem icon="tabler:history" :disabled="isActionRunning" @click="handleSyncOlder">
						{{ t('Sync older history') }}
					</DropdownMenuItem>
			</DropdownMenu>
		</ButtonGroup>

		<Spacer class="messenger-action-bar__spacer" />

		<Button
			v-if="showInspectorToggle"
			class="messenger-action-bar__button messenger-action-bar__inspector-toggle hermes-icon-button"
			variant="outline"
			size="sm"
			icon="tabler:layout-sidebar-right"
			:aria-label="inspectorVisible ? t('Hide Hermes inspector') : t('Show Hermes inspector')"
			:title="inspectorVisible ? t('Hide Hermes inspector') : t('Show Hermes inspector')"
			@click="handleToggleInspector"
		/>
	</nav>
</template>
