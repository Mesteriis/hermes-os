<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from './Icon.vue'
import type { NavigationItem } from './Navigation.types'

const props = withDefaults(defineProps<{
	items?: NavigationItem[]
	label?: string
	openLabel?: string
	defaultOpen?: boolean
	class?: string
}>(), {
	items: () => [],
	label: 'Context menu',
	openLabel: 'Open context menu',
	defaultOpen: false
})

const emit = defineEmits<{
	select: [item: NavigationItem]
}>()

const isOpen = ref(props.defaultOpen)
const classes = computed(() => ['hermes-context-menu', props.class])

function openMenu(event?: Event): void {
	event?.preventDefault()
	isOpen.value = true
}

function selectItem(item: NavigationItem): void {
	if (item.disabled) {
		return
	}
	emit('select', item)
	isOpen.value = false
}
</script>

<template>
	<div :class="classes" @keydown.escape="isOpen = false">
		<div class="hermes-context-menu__trigger" @contextmenu="openMenu">
			<slot name="trigger">
				<button class="hermes-context-menu__button" type="button" @click="openMenu">
					{{ openLabel }}
				</button>
			</slot>
		</div>
		<div v-if="isOpen" class="hermes-context-menu__content" role="menu" :aria-label="label">
			<button
				v-for="item in items"
				:key="item.id"
				class="hermes-context-menu__item"
				role="menuitem"
				type="button"
				:disabled="item.disabled"
				@click="selectItem(item)"
			>
				<Icon v-if="item.icon" :icon="item.icon" size="1rem" class="hermes-context-menu__icon" aria-hidden="true" />
				<span>{{ item.label }}</span>
			</button>
		</div>
	</div>
</template>
