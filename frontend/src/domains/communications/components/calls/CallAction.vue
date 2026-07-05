<script setup lang="ts">
import { Button, ButtonGroup, DropdownMenu, DropdownMenuItem, Spacer } from '@/shared/ui'
import '../communicationDomainElements.css'
import type { CommunicationCallActionGroupModel } from '../communicationDomainElements'

withDefaults(defineProps<{
  actionGroups?: readonly CommunicationCallActionGroupModel[]
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
	<nav class="call-action-bar" aria-label="Call actions">
		<ButtonGroup aria-label="Call navigation actions" class="call-action-bar__button-group">
			<Button
				class="call-action-bar__button hermes-icon-button"
				variant="outline"
				size="sm"
				icon="tabler:arrow-left"
				aria-label="Back to call list"
				title="Back to call list"
			/>
			<Button
				class="call-action-bar__button hermes-icon-button"
				variant="outline"
				size="sm"
				icon="tabler:calendar-plus"
				aria-label="Create meeting"
				title="Create meeting"
			/>
		</ButtonGroup>

		<ButtonGroup
			v-for="group in actionGroups"
			:key="group.id"
			:aria-label="group.title"
			class="call-action-bar__button-group"
		>
			<Button
				class="call-action-bar__button hermes-icon-button"
				variant="outline"
				size="sm"
				:icon="group.icon"
				:aria-label="group.title"
				:title="group.title"
			/>
			<DropdownMenu align="start" :side-offset="8">
				<template #trigger>
					<Button
						class="call-action-bar__button hermes-icon-button"
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

		<Spacer class="call-action-bar__spacer" />

		<Button
			v-if="showInspectorToggle"
			class="call-action-bar__button call-action-bar__inspector-toggle hermes-icon-button"
			variant="outline"
			size="sm"
			icon="tabler:layout-sidebar-right"
			:aria-label="inspectorVisible ? 'Hide Hermes inspector' : 'Show Hermes inspector'"
			:title="inspectorVisible ? 'Hide Hermes inspector' : 'Show Hermes inspector'"
			@click="emit('toggle-inspector')"
		/>
	</nav>
</template>
