<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '@/platform/i18n'
import { Button, ButtonGroup, Spacer, SplitButton, ToolbarGroup } from '@/shared/ui'
import type { CommunicationMessageActionGroupModel } from '../communicationDomainElements'
import {
  mailActionGroupDefaultAction,
  mailActionResponseControls,
  mailActionToolbarSections,
  type MailActionMenuGroup
} from './mailActions'
import '../communicationDomainElements.css'

const props = withDefaults(defineProps<{
  actionGroups?: readonly CommunicationMessageActionGroupModel[]
  inspectorVisible?: boolean
  isRunning?: boolean
  showInspectorToggle?: boolean
}>(), {
  inspectorVisible: true,
  isRunning: false,
  showInspectorToggle: true
})

const emit = defineEmits<{
  'select-action': [actionId: string]
  'toggle-inspector': []
}>()

const { t } = useI18n()
const responseControls = computed(() => mailActionResponseControls(props.actionGroups, t))
const actionToolbarSections = computed(() => mailActionToolbarSections(props.actionGroups, t))
const inspectorToggleLabel = computed(() => props.inspectorVisible ? t('Hide Hermes inspector') : t('Show Hermes inspector'))
const inspectorToggleIcon = computed(() => props.inspectorVisible
  ? 'tabler:layout-sidebar-right-collapse'
  : 'tabler:layout-sidebar-right-expand')

function handleToggleInspector(): void {
  emit('toggle-inspector')
}

function handleSelectAction(actionId: string | undefined): void {
  if (!actionId || props.isRunning) return
  emit('select-action', actionId)
}

function handleSelectGroupAction(group: MailActionMenuGroup): void {
  handleSelectAction(mailActionGroupDefaultAction(group))
}
</script>

<template>
	<nav
		class="communication-email-command-bar"
		role="toolbar"
		:aria-busy="isRunning"
		:aria-label="t('Message actions')"
	>
		<div v-if="responseControls.length || actionToolbarSections.length" class="communication-email-command-bar__scroll">
			<ButtonGroup
				v-if="responseControls.length"
				class="communication-email-command-bar__button-group communication-email-response-group"
				:aria-label="t('Respond and forward')"
			>
				<template v-for="control in responseControls" :key="control.id">
					<SplitButton
						v-if="control.kind === 'split'"
						:class="`communication-email-action-split communication-email-action-split--grouped${control.tone ? ` communication-email-action-split--${control.tone}` : ''}`"
						:disabled="isRunning || control.disabled"
						:icon="control.icon"
						:items="control.items"
						:label="control.label"
						:menu-label="control.menuLabel"
						variant="outline"
						size="sm"
						@click="handleSelectAction(control.id)"
						@select="handleSelectAction($event.id)"
					/>
					<Button
						v-else
						:class="`communication-email-action-button hermes-icon-button${control.tone ? ` communication-email-action-button--${control.tone}` : ''}`"
						:aria-label="control.label"
						:disabled="isRunning || control.disabled"
						:icon="control.icon"
						:title="control.label"
						variant="outline"
						size="sm"
						@click="handleSelectAction(control.id)"
					/>
				</template>
			</ButtonGroup>
			<template
				v-for="(section, sectionIndex) in actionToolbarSections"
				:key="section.id"
			>
				<template v-if="section.id === 'danger'">
					<Spacer
						v-if="showInspectorToggle && (responseControls.length || sectionIndex > 0)"
						class="communication-email-command-bar__spacer"
						orientation="horizontal"
					/>
					<Button
						v-if="showInspectorToggle"
						class="hermes-icon-button communication-email-command-bar__inspector-toggle"
						:aria-label="inspectorToggleLabel"
						:aria-pressed="inspectorVisible"
						:icon="inspectorToggleIcon"
						:title="inspectorToggleLabel"
						variant="outline"
						size="sm"
						@click="handleToggleInspector"
					/>
				</template>
				<Spacer
					v-if="responseControls.length || sectionIndex > 0"
					class="communication-email-command-bar__spacer"
					orientation="horizontal"
				/>
				<ToolbarGroup
					:class="section.groups.length > 1
						? 'communication-email-command-bar__group communication-email-command-bar__group--spread'
						: 'communication-email-command-bar__group'"
					:label="section.label"
				>
					<template v-for="(group, groupIndex) in section.groups" :key="group.id">
						<Spacer
							v-if="groupIndex > 0"
							class="communication-email-command-bar__inner-spacer"
							orientation="horizontal"
						/>
						<SplitButton
							:class="`communication-email-action-split${group.tone ? ` communication-email-action-split--${group.tone}` : ''}`"
							:icon="group.icon"
							:items="group.items"
							:label="group.label"
							:menu-label="group.menuLabel"
							variant="outline"
							size="sm"
							:disabled="isRunning"
							@click="handleSelectGroupAction(group)"
							@select="handleSelectAction($event.id)"
						/>
					</template>
				</ToolbarGroup>
			</template>
		</div>
	</nav>
</template>
