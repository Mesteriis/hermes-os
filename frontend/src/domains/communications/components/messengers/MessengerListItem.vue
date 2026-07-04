<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '@/platform/i18n'
import { Badge, Icon, Popover } from '@/shared/ui'
import '../communicationDomainElements.css'
import type { MessengerListItemDensity, MessengerListItemModel } from './messengerElements'
import {
  messengerListItemAriaLabel,
  messengerListItemHasSignal,
  messengerListItemProfile,
  messengerWorkflowStatusPresentation
} from './messengerElements'

const props = withDefaults(defineProps<{
  item: MessengerListItemModel
  density?: MessengerListItemDensity
  selected?: boolean
}>(), {
  density: 'comfortable'
})

const emit = defineEmits<{
  select: [item: MessengerListItemModel]
}>()

const { t } = useI18n()
const avatarStoryOpen = ref(false)
const status = computed(() => messengerWorkflowStatusPresentation(props.item.workflowState))
const profile = computed(() => messengerListItemProfile(props.item))
const profileStoryItems = computed(() => profile.value.storyItems ?? [])
const isSelected = computed(() => props.selected ?? Boolean(props.item.selected))
const itemClasses = computed(() => [
  'messenger-list-item',
  `messenger-list-item--${props.density}`,
  isSelected.value && 'messenger-list-item--selected',
  messengerListItemHasSignal(props.item) && 'messenger-list-item--signal'
])

const avatarAriaLabel = computed(() => `${t('Open avatar story')}: ${profile.value.displayName}`)
</script>

<template>
	<article :class="itemClasses">
		<div class="messenger-list-item__lead">
			<Popover
				v-model:open="avatarStoryOpen"
				side="right"
				align="start"
				:side-offset="8"
				class="messenger-avatar-popover"
				:close-label="t('Close avatar story')"
			>
				<template #trigger>
					<button
						type="button"
						class="messenger-list-item__avatar-trigger"
						:aria-label="avatarAriaLabel"
						:title="avatarAriaLabel"
						@click="avatarStoryOpen = true"
					>
						<span class="messenger-list-item__avatar" aria-hidden="true">
							<img v-if="profile.src" :src="profile.src" :alt="profile.displayName" />
							<span v-else>{{ profile.fallback }}</span>
						</span>
					</button>
				</template>
				<section class="messenger-avatar-story" :aria-label="`${t('Avatar story')}: ${profile.displayName}`">
					<div class="messenger-avatar-story__hero">
						<span class="messenger-avatar-story__avatar" aria-hidden="true">
							<img v-if="profile.src" :src="profile.src" :alt="profile.displayName" />
							<span v-else>{{ profile.fallback }}</span>
						</span>
						<div class="messenger-avatar-story__identity">
							<strong>{{ profile.displayName }}</strong>
							<span v-if="profile.statusLabel">{{ profile.statusLabel }}</span>
						</div>
					</div>
					<div v-if="profileStoryItems.length" class="messenger-avatar-story__track" :aria-label="t('Profile stories')">
						<article
							v-for="story in profileStoryItems"
							:key="story.id"
							:class="['messenger-avatar-story__item', story.tone && `messenger-avatar-story__item--${story.tone}`]"
						>
							<strong>{{ story.title }}</strong>
							<span v-if="story.description">{{ story.description }}</span>
							<time v-if="story.timestampLabel">{{ story.timestampLabel }}</time>
						</article>
					</div>
					<p v-else class="messenger-avatar-story__empty">{{ t('No profile stories') }}</p>
				</section>
			</Popover>
		</div>
		<button
			type="button"
			class="messenger-list-item__body"
			:aria-label="messengerListItemAriaLabel(item)"
			@click="emit('select', item)"
		>
			<div class="messenger-list-item__top">
				<strong class="messenger-list-item__title">{{ item.title }}</strong>
				<span v-if="item.unreadCount" class="messenger-list-item__unread">{{ item.unreadCount }}</span>
				<span class="messenger-list-item__time">{{ item.timestampLabel }}</span>
			</div>
			<p class="messenger-list-item__subject">{{ item.subtitle }}</p>
			<p class="messenger-list-item__preview">{{ item.preview }}</p>
			<div v-if="density !== 'compact'" class="messenger-list-item__signals" :aria-label="t('Messenger signals')">
				<span class="messenger-list-item__status">
					<Icon :icon="status.icon" size="0.9rem" />
					<span v-if="density === 'cozy'">{{ t(status.label) }}</span>
				</span>
				<span v-if="item.mentionCount" class="messenger-list-item__signal">
					<Icon icon="tabler:at" size="0.9rem" />
					<span>{{ item.mentionCount }}</span>
				</span>
				<span v-if="item.attachmentCount" class="messenger-list-item__signal">
					<Icon icon="tabler:paperclip" size="0.9rem" />
					<span>{{ item.attachmentCount }}</span>
				</span>
				<span v-if="item.hermesSignalCount" class="messenger-list-item__signal messenger-list-item__signal--hermes">
					<Icon icon="tabler:sparkles" size="0.9rem" />
					<span>{{ item.hermesSignalCount }}</span>
				</span>
				<Badge v-if="item.pinned" variant="info">{{ t('Pinned') }}</Badge>
				<Badge v-if="item.muted" variant="neutral">{{ t('Muted') }}</Badge>
			</div>
		</button>
	</article>
</template>
