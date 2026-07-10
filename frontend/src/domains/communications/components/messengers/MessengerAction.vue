<script setup lang="ts">
import { useI18n } from '@/platform/i18n'
import { Button, ButtonGroup, DropdownMenu, DropdownMenuItem, Spacer } from '@/shared/ui'
import '../communicationDomainElements.css'

withDefaults(defineProps<{
  inspectorVisible?: boolean
  showInspectorToggle?: boolean
}>(), {
  inspectorVisible: true,
  showInspectorToggle: true
})

const emit = defineEmits<{
  'toggle-inspector': []
}>()

const { t } = useI18n()
</script>

<template>
	<nav class="messenger-action-bar" :aria-label="t('Messenger conversation actions')">
		<ButtonGroup :aria-label="t('Reply actions')" class="messenger-action-bar__button-group">
			<Button
				class="messenger-action-bar__button hermes-icon-button"
				variant="outline"
				size="sm"
				icon="tabler:message-reply"
				:aria-label="t('Reply')"
				:title="t('Reply')"
			/>
			<DropdownMenu align="start" :side-offset="8">
				<template #trigger>
					<Button
						class="messenger-action-bar__button hermes-icon-button"
						variant="outline"
						size="sm"
						icon="tabler:chevron-down"
						:aria-label="t('Open reply actions')"
						:title="t('Open reply actions')"
					/>
				</template>
				<DropdownMenuItem icon="tabler:sparkles">{{ t('AI Reply') }}</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:message-2-code">{{ t('AI Reply variants') }}</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:language">{{ t('Bilingual reply flow') }}</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:users-plus">{{ t('Smart recipients') }}</DropdownMenuItem>
			</DropdownMenu>
		</ButtonGroup>

		<ButtonGroup :aria-label="t('Conversation state actions')" class="messenger-action-bar__button-group">
			<Button
				class="messenger-action-bar__button hermes-icon-button"
				variant="outline"
				size="sm"
				icon="tabler:mail-opened"
				:aria-label="t('Mark read')"
				:title="t('Mark read')"
			/>
			<DropdownMenu align="start" :side-offset="8">
				<template #trigger>
					<Button
						class="messenger-action-bar__button hermes-icon-button"
						variant="outline"
						size="sm"
						icon="tabler:chevron-down"
						:aria-label="t('Open state actions')"
						:title="t('Open state actions')"
					/>
				</template>
				<DropdownMenuItem icon="tabler:mail">{{ t('Mark unread') }}</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:pin">{{ t('Pin') }}</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:bell-off">{{ t('Mute') }}</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:archive">{{ t('Archive') }}</DropdownMenuItem>
			</DropdownMenu>
		</ButtonGroup>

		<Spacer class="messenger-action-bar__spacer" />

		<ButtonGroup :aria-label="t('Hermes actions')" class="messenger-action-bar__button-group">
			<Button
				class="messenger-action-bar__button hermes-icon-button"
				variant="outline"
				size="sm"
				icon="tabler:sparkles"
				:aria-label="t('Analyze')"
				:title="t('Analyze')"
			/>
			<DropdownMenu align="start" :side-offset="8">
				<template #trigger>
					<Button
						class="messenger-action-bar__button hermes-icon-button"
						variant="outline"
						size="sm"
						icon="tabler:chevron-down"
						:aria-label="t('Open Hermes actions')"
						:title="t('Open Hermes actions')"
					/>
				</template>
				<DropdownMenuItem icon="tabler:checkbox">{{ t('Create task') }}</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:calendar-plus">{{ t('Create event') }}</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:user-plus">{{ t('Create persona') }}</DropdownMenuItem>
				<DropdownMenuItem icon="tabler:file-plus">{{ t('Create document') }}</DropdownMenuItem>
			</DropdownMenu>
		</ButtonGroup>

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
