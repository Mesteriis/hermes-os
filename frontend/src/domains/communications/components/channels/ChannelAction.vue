<script setup lang="ts">
import { Button, ButtonGroup, DropdownMenu, DropdownMenuItem, Spacer } from '@/shared/ui'
import type { CommunicationChannelActionGroupModel } from '../communicationDomainElements'
import '../communicationDomainElements.css'

withDefaults(defineProps<{
  actionGroups?: readonly CommunicationChannelActionGroupModel[]
  inspectorVisible?: boolean
  showInspectorToggle?: boolean
}>(), {
  actionGroups: () => [],
  inspectorVisible: true,
  showInspectorToggle: true
})

const emit = defineEmits<{
  'toggle-inspector': []
}>()
</script>

<template>
	<nav class="channel-action-bar" aria-label="Channel actions">
		<ButtonGroup aria-label="Channel navigation actions" class="channel-action-bar__button-group">
			<Button
				class="channel-action-bar__button hermes-icon-button"
				variant="outline"
				size="sm"
				icon="tabler:arrow-left"
				aria-label="Back to channel list"
				title="Back to channel list"
			/>
			<Button
				class="channel-action-bar__button hermes-icon-button"
				variant="outline"
				size="sm"
				icon="tabler:pinned"
				aria-label="Pinned messages"
				title="Pinned messages"
			/>
		</ButtonGroup>

		<ButtonGroup
			v-for="group in actionGroups"
			:key="group.id"
			:aria-label="group.title"
			class="channel-action-bar__button-group"
		>
			<Button
				class="channel-action-bar__button hermes-icon-button"
				variant="outline"
				size="sm"
				:icon="group.icon"
				:aria-label="group.title"
				:title="group.title"
			/>
			<DropdownMenu align="start" :side-offset="8">
				<template #trigger>
					<Button
						class="channel-action-bar__button hermes-icon-button"
						variant="outline"
						size="sm"
						icon="tabler:chevron-down"
						:aria-label="group.menuLabel"
						:title="group.menuLabel"
					/>
				</template>
				<DropdownMenuItem
					v-for="action in group.actions"
					:key="action.id"
					:icon="action.icon"
					:disabled="action.disabled"
					:title="action.contract ?? action.description"
				>
					{{ action.label }}
				</DropdownMenuItem>
			</DropdownMenu>
		</ButtonGroup>

		<Spacer class="channel-action-bar__spacer" />

		<Button
			v-if="showInspectorToggle"
			class="channel-action-bar__button channel-action-bar__inspector-toggle hermes-icon-button"
			variant="outline"
			size="sm"
			icon="tabler:layout-sidebar-right"
			:aria-label="inspectorVisible ? 'Hide Hermes inspector' : 'Show Hermes inspector'"
			:title="inspectorVisible ? 'Hide Hermes inspector' : 'Show Hermes inspector'"
			@click="emit('toggle-inspector')"
		/>
	</nav>
</template>
