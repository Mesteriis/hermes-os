<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '@/platform/i18n'
import { Button, ButtonGroup, Spacer, SplitButton, ToolbarGroup } from '@/shared/ui'
import type { CommunicationMessageActionGroupModel } from '../communicationDomainElements'
import { mailActionResponseControls, mailActionToolbarSections } from './mailActions'
import '../communicationDomainElements.css'

const props = withDefaults(defineProps<{
  actionGroups?: readonly CommunicationMessageActionGroupModel[]
  inspectorVisible?: boolean
}>(), {
  inspectorVisible: true
})

const emit = defineEmits<{
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
</script>

<template>
	<nav class="communication-email-command-bar" role="toolbar" :aria-label="t('Message actions')">
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
						:disabled="control.disabled"
						:icon="control.icon"
						:items="control.items"
						:label="control.label"
						:menu-label="control.menuLabel"
						variant="outline"
						size="sm"
					/>
					<Button
						v-else
						:class="`communication-email-action-button${control.tone ? ` communication-email-action-button--${control.tone}` : ''}`"
						:disabled="control.disabled"
						:icon="control.icon"
						:title="control.label"
						variant="outline"
						size="sm"
					>
						{{ control.label }}
					</Button>
				</template>
			</ButtonGroup>
			<template
				v-for="(section, sectionIndex) in actionToolbarSections"
				:key="section.id"
			>
				<template v-if="section.id === 'danger'">
					<Spacer
						v-if="responseControls.length || sectionIndex > 0"
						class="communication-email-command-bar__spacer"
						orientation="horizontal"
					/>
					<Button
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
						/>
					</template>
				</ToolbarGroup>
			</template>
		</div>
	</nav>
</template>
