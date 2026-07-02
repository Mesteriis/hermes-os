<script setup lang="ts">
import { computed, useSlots } from 'vue'

type AppLayoutDensity = 'compact' | 'default'
type AppLayoutMode = 'workspace' | 'focus'

const props = withDefaults(defineProps<{
	as?: string
	density?: AppLayoutDensity
	mode?: AppLayoutMode
	rail?: boolean
	sidebar?: boolean
	inspector?: boolean
	footer?: boolean
	class?: string
}>(), {
	as: 'div',
	density: 'default',
	mode: 'workspace',
	rail: true,
	sidebar: true,
	inspector: true,
	footer: true
})

const slots = useSlots()

const hasRail = computed(() => props.rail && Boolean(slots.rail))
const hasSidebar = computed(() => props.sidebar && props.mode !== 'focus' && Boolean(slots.sidebar))
const hasInspector = computed(() => props.inspector && Boolean(slots.inspector))
const hasTopbar = computed(() => Boolean(slots.topbar))
const hasFooter = computed(() => props.footer && Boolean(slots.footer))
</script>

<template>
	<component
		:is="props.as"
		:class="[
			'hermes-app-layout',
			`hermes-app-layout--${props.density}`,
			`hermes-app-layout--${props.mode}`,
			{
				'hermes-app-layout--has-rail': hasRail,
				'hermes-app-layout--has-sidebar': hasSidebar,
				'hermes-app-layout--has-inspector': hasInspector,
				'hermes-app-layout--has-topbar': hasTopbar,
				'hermes-app-layout--has-footer': hasFooter
			},
			props.class
		]"
	>
		<div v-if="hasRail" class="hermes-app-layout__rail">
			<slot name="rail" />
		</div>

		<aside v-if="hasSidebar" class="hermes-app-layout__sidebar">
			<slot name="sidebar" />
		</aside>

		<section class="hermes-app-layout__workspace">
			<header v-if="hasTopbar" class="hermes-app-layout__topbar">
				<slot name="topbar" />
			</header>

			<main class="hermes-app-layout__main">
				<slot />
			</main>

			<footer v-if="hasFooter" class="hermes-app-layout__footer">
				<slot name="footer" />
			</footer>
		</section>

		<aside v-if="hasInspector" class="hermes-app-layout__inspector">
			<slot name="inspector" />
		</aside>
	</component>
</template>
