<script setup lang="ts">
import { useI18n } from '@/platform/i18n'
import { Button, ButtonGroup, DropdownMenu, DropdownMenuItem, Spacer } from '@/shared/ui'
import type { TelegramConversationRuntimeAction } from '@/shared/communications/types/telegramRuntimeActions'
import '../communicationDomainElements.css'

withDefaults(defineProps<{
  channelKind?: 'signal' | 'telegram' | 'whatsapp'
  inspectorVisible?: boolean
  isActionRunning?: boolean
  showInspectorToggle?: boolean
}>(), {
  channelKind: 'signal',
  inspectorVisible: true,
  isActionRunning: false,
  showInspectorToggle: true
})

const emit = defineEmits<{
  'conversation-action': [action: TelegramConversationRuntimeAction]
  'toggle-inspector': []
}>()

const { t } = useI18n()
</script>

<template>
	<nav class="messenger-action-bar" :aria-label="t('Messenger conversation actions')">
		<ButtonGroup
			v-if="channelKind === 'telegram'"
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
				@click="emit('conversation-action', 'mark_read')"
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
				<DropdownMenuItem icon="tabler:mail" :disabled="isActionRunning" @click="emit('conversation-action', 'mark_unread')">
					{{ t('Mark unread') }}
				</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:pin" :disabled="isActionRunning" @click="emit('conversation-action', 'pin')">
					{{ t('Pin') }}
				</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:pinned-off" :disabled="isActionRunning" @click="emit('conversation-action', 'unpin')">
					{{ t('Unpin') }}
				</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:bell-off" :disabled="isActionRunning" @click="emit('conversation-action', 'mute')">
					{{ t('Mute') }}
				</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:bell" :disabled="isActionRunning" @click="emit('conversation-action', 'unmute')">
					{{ t('Unmute') }}
				</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:archive" :disabled="isActionRunning" @click="emit('conversation-action', 'archive')">
					{{ t('Archive') }}
				</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:archive-off" :disabled="isActionRunning" @click="emit('conversation-action', 'unarchive')">
					{{ t('Unarchive') }}
				</DropdownMenuItem>
			</DropdownMenu>
		</ButtonGroup>

		<ButtonGroup
			v-if="channelKind === 'telegram'"
			:aria-label="t('Telegram synchronization actions')"
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
				@click="emit('conversation-action', 'sync_latest')"
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
				<DropdownMenuItem icon="tabler:history" :disabled="isActionRunning" @click="emit('conversation-action', 'sync_older')">
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
			@click="emit('toggle-inspector')"
		/>
	</nav>
</template>
