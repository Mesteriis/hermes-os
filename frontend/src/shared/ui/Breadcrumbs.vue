<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { NavigationItem } from './Navigation.types'

const props = withDefaults(defineProps<{
	items?: NavigationItem[]
	label?: string
	class?: string
}>(), {
	items: () => [],
	label: 'Breadcrumb'
})

const emit = defineEmits<{
	navigate: [item: NavigationItem]
}>()

const classes = computed(() => ['hermes-breadcrumbs', props.class])

function isCurrent(item: NavigationItem, index: number): boolean {
	return item.current === true || index === props.items.length - 1
}
</script>

<template>
	<nav :class="classes" :aria-label="label">
		<ol class="hermes-breadcrumbs__list">
			<li v-for="(item, index) in items" :key="item.id" class="hermes-breadcrumbs__item">
				<Icon v-if="index > 0" icon="tabler:chevron-right" size="0.875rem" class="hermes-breadcrumbs__separator" aria-hidden="true" />
				<span v-if="isCurrent(item, index)" class="hermes-breadcrumbs__current" aria-current="page">
					{{ item.label }}
				</span>
				<a
					v-else-if="item.href"
					class="hermes-breadcrumbs__link"
					:href="item.href"
					@click.prevent="emit('navigate', item)"
				>
					{{ item.label }}
				</a>
				<button v-else class="hermes-breadcrumbs__link" type="button" @click="emit('navigate', item)">
					{{ item.label }}
				</button>
			</li>
		</ol>
	</nav>
</template>
