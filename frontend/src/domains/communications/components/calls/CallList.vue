<script setup lang="ts">
import { computed, ref } from 'vue'
import { Badge, Button, Icon, TreeSelect } from '@/shared/ui'
import type { TreeSelectOption } from '@/shared/ui'
import '../communicationDomainElements.css'
import {
  communicationCallProviderIconName,
  createCommunicationCallDateGroups,
  type CommunicationCallItemModel,
  type CommunicationPermanentCallLinkModel
} from '../communicationDomainElements'
import CallListItem from './CallListItem.vue'

const props = defineProps<{
  providerValue: string
  providerOptions: TreeSelectOption[]
  permanentMeetings?: readonly CommunicationPermanentCallLinkModel[]
  calls: readonly CommunicationCallItemModel[]
}>()

const emit = defineEmits<{
  select: [item: CommunicationCallItemModel]
}>()

const providerValue = ref(props.providerValue)

const permanentMeetingLinks = computed(() => props.permanentMeetings ?? [])

const dateGroups = computed(() => createCommunicationCallDateGroups(props.calls))
</script>

<template>
	<aside class="communication-calls-list" aria-label="Calls">
		<section class="communication-calls-list__actions" aria-label="Call search">
			<label class="communication-channel-search">
				<Icon icon="tabler:search" size="1rem" class="communication-channel-search__icon" />
				<span class="communication-channel-search__label">Search calls</span>
				<input
					class="communication-channel-search__input"
					type="search"
					placeholder="Search calls, recordings, transcripts"
					aria-label="Search calls"
				/>
			</label>
			<Button
				class="communication-calls-list__tool hermes-icon-button"
				variant="outline"
				size="sm"
				icon="tabler:refresh"
				aria-label="Refresh calls"
				title="Refresh calls"
			/>
		</section>

		<header class="communication-calls-list__header">
			<TreeSelect
				v-model="providerValue"
				class="communication-call-provider-select"
				:options="providerOptions"
				placeholder="Select call provider"
				aria-label="Call provider"
				empty-label="No call providers"
			/>
		</header>

		<div
			:class="[
				'communication-calls-list__body',
				!permanentMeetingLinks.length && 'communication-calls-list__body--only-sections'
			]"
		>
			<section
				v-if="permanentMeetingLinks.length"
				class="communication-calls-list__permanent"
				aria-label="Permanent meetings"
			>
				<header class="communication-calls-list__permanent-header">
					<span>
						<Icon icon="tabler:pin" size="1rem" />
						Permanent meetings
					</span>
					<Badge variant="neutral">{{ permanentMeetingLinks.length }}</Badge>
				</header>
				<div class="communication-calls-list__permanent-links">
					<a
						v-for="meeting in permanentMeetingLinks"
						:key="meeting.id"
						class="communication-permanent-call-link"
						:href="meeting.href"
						:aria-label="`${meeting.providerLabel}: ${meeting.title}`"
					>
						<span class="communication-permanent-call-link__provider" :aria-label="meeting.providerLabel">
							<Icon :icon="communicationCallProviderIconName(meeting.providerKind)" size="0.95rem" />
						</span>
						<span class="communication-permanent-call-link__body">
							<strong>{{ meeting.title }}</strong>
							<small>{{ meeting.providerLabel }} · {{ meeting.description }}</small>
						</span>
						<Badge v-if="meeting.statusLabel" :variant="meeting.tone ?? 'info'">{{ meeting.statusLabel }}</Badge>
					</a>
				</div>
			</section>

			<div class="communication-calls-list__sections" aria-label="Calls by date">
				<section v-for="group in dateGroups" :key="group.id" class="communication-calls-list__date-section">
					<header class="communication-calls-list__date-header">
						<span>{{ group.label }}</span>
						<Badge variant="neutral">{{ group.calls.length }}</Badge>
					</header>
					<div class="communication-calls-list__date-body">
						<CallListItem
							v-for="call in group.calls"
							:key="call.id"
							:item="call"
							@select="emit('select', $event)"
						/>
					</div>
				</section>
			</div>
		</div>
	</aside>
</template>
