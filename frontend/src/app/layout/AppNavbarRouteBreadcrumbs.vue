<script setup lang="ts">
import { computed, ref, type ComponentPublicInstance } from 'vue'
import Icon from '../../shared/ui/Icon.vue'
import { useMouseLeaveDismiss } from '../../shared/ui/useMouseLeaveDismiss'

type AppNavbarRouteBreadcrumbItem = {
	id: string
	label: string
	icon?: string
	iconTone?: AppNavbarRouteBreadcrumbIconTone
}

type AppNavbarRouteBreadcrumbLevel = {
	id: string
	label: string
	currentItem: AppNavbarRouteBreadcrumbItem
	items: readonly AppNavbarRouteBreadcrumbItem[]
}

type AppNavbarRouteBreadcrumbIconTone =
	| 'accounts'
	| 'calendar'
	| 'channels'
	| 'communication'
	| 'dashboard'
	| 'documents'
	| 'knowledge'
	| 'mail'
	| 'review'
	| 'settings'
	| 'tasks'
	| 'telegram'
	| 'whatsapp'

const props = withDefaults(defineProps<{
	levels?: readonly AppNavbarRouteBreadcrumbLevel[]
}>(), {
	levels: () => []
})

const emit = defineEmits<{
	select: [itemId: string]
}>()

const openLevelId = ref<string | null>(null)
const isMenuOpen = computed(() => openLevelId.value !== null)
const rootElement = ref<HTMLElement | null>(null)
const menuElement = ref<HTMLElement | null>(null)

const { cancelMouseLeaveDismiss, scheduleMouseLeaveDismiss } = useMouseLeaveDismiss(closeMenu, undefined, {
	isOpen: isMenuOpen,
	getBoundaryElements: () => [rootElement.value, menuElement.value]
})

function setMenuRef(value: Element | ComponentPublicInstance | null): void {
	menuElement.value = templateRefElement(value)
}

function templateRefElement(value: Element | ComponentPublicInstance | null): HTMLElement | null {
	if (value instanceof HTMLElement) return value
	if (isComponentRef(value) && value.$el instanceof HTMLElement) return value.$el

	return null
}

function isComponentRef(value: Element | ComponentPublicInstance | null): value is ComponentPublicInstance {
	return Boolean(value && !(value instanceof Element) && '$el' in value)
}

function isLevelOpen(levelId: string): boolean {
	return openLevelId.value === levelId
}

function isLastLevel(index: number): boolean {
	return index === props.levels.length - 1
}

function triggerClass(index: number): string {
	return isLastLevel(index)
		? 'app-navbar-route-breadcrumbs__trigger--labelled'
		: 'app-navbar-route-breadcrumbs__trigger--icon-only'
}

function iconToneClass(tone?: AppNavbarRouteBreadcrumbIconTone): string | undefined {
	if (!tone) return undefined

	return `app-navbar-route-breadcrumbs__item-icon--${tone}`
}

function toggleLevel(levelId: string): void {
	cancelMouseLeaveDismiss()
	openLevelId.value = isLevelOpen(levelId) ? null : levelId
}

function closeMenu(): void {
	cancelMouseLeaveDismiss()
	openLevelId.value = null
}

function selectItem(itemId: string): void {
	closeMenu()
	emit('select', itemId)
}
</script>

<template>
	<ol ref="rootElement" class="app-navbar__breadcrumbs app-navbar-route-breadcrumbs" aria-label="Navigation path">
		<li
			v-for="(level, index) in levels"
			:key="level.id"
			class="app-navbar__breadcrumb-item app-navbar-route-breadcrumbs__item"
		>
			<div
				class="app-navbar-route-breadcrumbs__dropdown"
				@mouseenter="cancelMouseLeaveDismiss"
				@mouseleave="scheduleMouseLeaveDismiss"
			>
				<button
					type="button"
					class="app-navbar__breadcrumb-button app-navbar-route-breadcrumbs__trigger"
					:class="triggerClass(index)"
					:aria-current="index === levels.length - 1 ? 'page' : undefined"
					:aria-expanded="isLevelOpen(level.id)"
					:aria-label="`${level.label}: ${level.currentItem.label}`"
					:title="level.currentItem.label"
					aria-haspopup="menu"
					@click="toggleLevel(level.id)"
				>
					<Icon
						v-if="level.currentItem.icon"
						:icon="level.currentItem.icon"
						size="18"
						class="app-navbar-route-breadcrumbs__item-icon"
						:class="iconToneClass(level.currentItem.iconTone)"
						aria-hidden="true"
					/>
					<span v-if="isLastLevel(index)" class="app-navbar-route-breadcrumbs__label">{{ level.currentItem.label }}</span>
					<Icon
						v-if="isLastLevel(index)"
						icon="tabler:chevron-down"
						size="14"
						class="app-navbar-route-breadcrumbs__chevron"
						aria-hidden="true"
					/>
				</button>

				<div
					v-if="isLevelOpen(level.id)"
					:ref="setMenuRef"
					class="app-navbar__menu app-navbar-route-breadcrumbs__menu"
					role="menu"
					:aria-label="level.label"
				>
					<button
						v-for="item in level.items"
						:key="item.id"
						type="button"
						class="app-navbar-route-breadcrumbs__option"
						role="menuitemradio"
						:aria-checked="item.id === level.currentItem.id"
						@click="selectItem(item.id)"
					>
						<Icon
							v-if="item.icon"
							:icon="item.icon"
							size="16"
							class="app-navbar-route-breadcrumbs__item-icon"
							:class="iconToneClass(item.iconTone)"
							aria-hidden="true"
						/>
						<span class="app-navbar-route-breadcrumbs__option-label">{{ item.label }}</span>
						<Icon
							v-if="item.id === level.currentItem.id"
							icon="tabler:check"
							size="16"
							class="app-navbar-route-breadcrumbs__check"
							aria-hidden="true"
						/>
					</button>
				</div>
			</div>

		</li>
	</ol>
</template>
