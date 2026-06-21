<script setup lang="ts">
import { computed, ref } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type {
  CommunicationArchitectureBlocker,
  SenderStats,
  SubscriptionSource
} from '../types/communications'

const props = defineProps<{
  subscriptions: SubscriptionSource[]
  topSenders: SenderStats[]
  blockers: CommunicationArchitectureBlocker[]
  isLoading: boolean
  hasMoreSubscriptions: boolean
  isLoadingMoreSubscriptions: boolean
  hasMoreTopSenders: boolean
  isLoadingMoreTopSenders: boolean
}>()

const emit = defineEmits<{
  loadMoreSubscriptions: []
  loadMoreTopSenders: []
}>()

const subscriptionScrollRef = ref<HTMLDivElement | null>(null)
const subscriptionVirtualizer = useVirtualizer(computed(() => ({
  count: props.subscriptions.length,
  getScrollElement: () => subscriptionScrollRef.value,
  estimateSize: () => 28,
  overscan: 6
})))
const virtualSubscriptionRows = computed(() => subscriptionVirtualizer.value.getVirtualItems())
const subscriptionTotalSize = computed(() => subscriptionVirtualizer.value.getTotalSize())

const topSenderScrollRef = ref<HTMLDivElement | null>(null)
const topSenderVirtualizer = useVirtualizer(computed(() => ({
  count: props.topSenders.length,
  getScrollElement: () => topSenderScrollRef.value,
  estimateSize: () => 28,
  overscan: 6
})))
const virtualTopSenderRows = computed(() => topSenderVirtualizer.value.getVirtualItems())
const topSenderTotalSize = computed(() => topSenderVirtualizer.value.getTotalSize())
</script>

<template>
  <section class="mail-resource-strip" aria-label="Mailbox resources">
    <article class="resource-group">
      <div class="resource-heading">
        <Icon icon="tabler:mail-bolt" class="resource-icon" />
        <span>Newsletters</span>
      </div>
      <div v-if="isLoading && subscriptions.length === 0" class="resource-muted">Loading...</div>
      <div v-else-if="subscriptions.length === 0" class="resource-muted">No sources</div>
      <div v-else ref="subscriptionScrollRef" class="resource-virtual-list">
        <div class="resource-list-track" :style="{ height: `${subscriptionTotalSize}px` }">
          <span
            v-for="virtualRow in virtualSubscriptionRows"
            :key="String(virtualRow.key)"
            class="resource-chip virtual-resource-chip"
            :style="{
              height: `${virtualRow.size}px`,
              transform: `translateY(${virtualRow.start}px)`
            }"
          >
            {{ subscriptions[virtualRow.index]?.sender ?? '' }} · {{ subscriptions[virtualRow.index]?.message_count ?? 0 }}
          </span>
        </div>
      </div>
      <Button
        v-if="hasMoreSubscriptions"
        class="resource-load-more"
        type="button"
        variant="ghost"
        size="sm"
        :disabled="isLoadingMoreSubscriptions"
        @click="emit('loadMoreSubscriptions')"
      >
        <Icon icon="tabler:chevron-down" />
        <span>{{ isLoadingMoreSubscriptions ? 'Loading...' : 'More newsletters' }}</span>
      </Button>
    </article>

    <article class="resource-group">
      <div class="resource-heading">
        <Icon icon="tabler:user-star" class="resource-icon" />
        <span>Top senders</span>
      </div>
      <div v-if="isLoading && topSenders.length === 0" class="resource-muted">Loading...</div>
      <div v-else-if="topSenders.length === 0" class="resource-muted">No senders</div>
      <div v-else ref="topSenderScrollRef" class="resource-virtual-list">
        <div class="resource-list-track" :style="{ height: `${topSenderTotalSize}px` }">
          <span
            v-for="virtualRow in virtualTopSenderRows"
            :key="String(virtualRow.key)"
            class="resource-chip virtual-resource-chip"
            :style="{
              height: `${virtualRow.size}px`,
              transform: `translateY(${virtualRow.start}px)`
            }"
          >
            {{ topSenders[virtualRow.index]?.sender ?? '' }} · {{ topSenders[virtualRow.index]?.message_count ?? 0 }}
          </span>
        </div>
      </div>
      <Button
        v-if="hasMoreTopSenders"
        class="resource-load-more"
        type="button"
        variant="ghost"
        size="sm"
        :disabled="isLoadingMoreTopSenders"
        @click="emit('loadMoreTopSenders')"
      >
        <Icon icon="tabler:chevron-down" />
        <span>{{ isLoadingMoreTopSenders ? 'Loading...' : 'More senders' }}</span>
      </Button>
    </article>

    <article class="resource-group">
      <div class="resource-heading">
        <Icon icon="tabler:road-sign" class="resource-icon" />
        <span>Blockers</span>
      </div>
      <div v-if="isLoading" class="resource-muted">Loading...</div>
      <div v-else-if="blockers.length === 0" class="resource-muted">No blockers</div>
      <div v-else class="resource-list">
        <span v-for="blocker in blockers.slice(0, 2)" :key="`${blocker.section}-${blocker.feature}`" class="resource-chip warning">
          {{ blocker.feature }}
        </span>
      </div>
    </article>
  </section>
</template>

<style scoped>
.mail-resource-strip {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 86%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.resource-group {
  min-width: 0;
  display: grid;
  gap: 0.375rem;
}

.resource-heading {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  min-width: 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.resource-icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  color: var(--hh-accent, #2563eb);
}

.resource-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  min-width: 0;
}

.resource-virtual-list {
  position: relative;
  min-width: 0;
  min-height: 1.75rem;
  max-height: 5.25rem;
  overflow: auto;
}

.resource-list-track {
  position: relative;
  width: 100%;
}

.resource-chip,
.resource-muted {
  min-width: 0;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
}

.resource-chip {
  padding: 0.125rem 0.375rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 999px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 72%, transparent);
}

.virtual-resource-chip {
  position: absolute;
  top: 0;
  left: 0;
  display: inline-flex;
  align-items: center;
  max-width: calc(100% - 0.25rem);
}

.resource-chip.warning {
  color: var(--hh-text-error, #ef4444);
}

.resource-load-more {
  justify-content: flex-start;
  min-width: 0;
  width: fit-content;
  max-width: 100%;
  color: var(--hh-accent, #2563eb);
  font-size: 0.6875rem;
  gap: 0.25rem;
  padding-inline: 0;
}

.resource-load-more:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

@media (max-width: 900px) {
  .mail-resource-strip {
    grid-template-columns: 1fr;
  }
}
</style>
