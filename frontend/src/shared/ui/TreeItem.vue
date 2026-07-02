<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { TreeItemData } from './Navigation.types'

defineOptions({ name: 'TreeItem' })

const props = withDefaults(defineProps<{
	item: TreeItemData
	selectedId?: string
	expanded?: string[]
	level?: number
}>(), {
	selectedId: '',
	expanded: () => [],
	level: 1
})

const emit = defineEmits<{
	select: [item: TreeItemData]
	toggle: [item: TreeItemData]
}>()

const hasChildren = computed(() => Boolean(props.item.children?.length))
const isExpanded = computed(() => props.expanded.includes(props.item.id))
const isSelected = computed(() => props.selectedId === props.item.id)

function handleClick(): void {
	if (props.item.disabled) {
		return
	}
	if (hasChildren.value) {
		emit('toggle', props.item)
	}
	emit('select', props.item)
}

function handleKeydown(event: KeyboardEvent): void {
	if (['Enter', ' '].includes(event.key)) {
		event.preventDefault()
		handleClick()
	}
	if (event.key === 'ArrowRight' && hasChildren.value && !isExpanded.value) {
		event.preventDefault()
		emit('toggle', props.item)
	}
	if (event.key === 'ArrowLeft' && hasChildren.value && isExpanded.value) {
		event.preventDefault()
		emit('toggle', props.item)
	}
}
</script>

<template>
	<li
		class="hermes-tree-item"
		role="treeitem"
		:aria-disabled="item.disabled"
		:aria-expanded="hasChildren ? isExpanded : undefined"
		:aria-level="level"
		:aria-selected="isSelected"
	>
		<button
			class="hermes-tree-item__button"
			type="button"
			:disabled="item.disabled"
			:tabindex="isSelected ? 0 : -1"
			@click="handleClick"
			@keydown="handleKeydown"
		>
			<Icon
				v-if="hasChildren"
				:icon="isExpanded ? 'tabler:chevron-down' : 'tabler:chevron-right'"
				size="0.875rem"
				class="hermes-tree-item__chevron"
				aria-hidden="true"
			/>
			<span v-else class="hermes-tree-item__spacer" aria-hidden="true"></span>
			<Icon v-if="item.icon" :icon="item.icon" size="1rem" class="hermes-tree-item__icon" aria-hidden="true" />
			<span>{{ item.label }}</span>
		</button>
		<ul v-if="hasChildren && isExpanded" class="hermes-tree-item__children" role="group">
			<TreeItem
				v-for="child in item.children"
				:key="child.id"
				:item="child"
				:expanded="expanded"
				:level="level + 1"
				:selected-id="selectedId"
				@select="emit('select', $event)"
				@toggle="emit('toggle', $event)"
			/>
		</ul>
	</li>
</template>
