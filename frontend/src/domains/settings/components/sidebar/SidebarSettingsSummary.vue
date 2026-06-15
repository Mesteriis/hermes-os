<script setup lang="ts">
import type {
  ResolvedSidebarRootEntry,
  SidebarItemId
} from '../../../../shared/stores/sidebar'

defineProps<{
  entries: ResolvedSidebarRootEntry[]
  hiddenItemIds: SidebarItemId[]
  itemLabels: Record<SidebarItemId, { label: string; icon: string }>
  previewLabel: string
  hiddenLabel: string
  rulesLabel: string
  rootDomainLabel: string
  emptyGroupLabel: string
  noHiddenLabel: string
  showLabel: string
  rules: Array<{ text: string; badge: string }>
}>()

defineEmits<{
  toggleHidden: [itemId: SidebarItemId]
}>()
</script>

<template>
  <aside class="settings-rail sidebar-settings-summary">
    <section class="panel info-card">
      <h2>{{ previewLabel }}</h2>
      <ul class="sidebar-preview-list">
        <li v-for="entry in entries" :key="entry.rootId">
          <template v-if="entry.kind === 'group'">
            <strong>{{ entry.group.label }}</strong>
            <span>{{ entry.group.items.map((item) => item.label).join(', ') || emptyGroupLabel }}</span>
          </template>
          <template v-else>
            <strong>{{ entry.item.label }}</strong>
            <span>{{ rootDomainLabel }}</span>
          </template>
        </li>
      </ul>
    </section>
    <section class="panel info-card">
      <h2>{{ hiddenLabel }}</h2>
      <p v-if="hiddenItemIds.length === 0">{{ noHiddenLabel }}</p>
      <ul v-else class="detail-list">
        <li v-for="itemId in hiddenItemIds" :key="itemId">
          {{ itemLabels[itemId]?.label ?? itemId }}
          <button
            type="button"
            class="hermes-btn hermes-btn--ghost"
            @click="$emit('toggleHidden', itemId)"
          >
            {{ showLabel }}
          </button>
        </li>
      </ul>
    </section>
    <section class="panel info-card">
      <h2>{{ rulesLabel }}</h2>
      <ul class="detail-list">
        <li v-for="rule in rules" :key="rule.text">
          {{ rule.text }}
          <em>{{ rule.badge }}</em>
        </li>
      </ul>
    </section>
  </aside>
</template>

<style scoped>
.settings-rail {
  display: grid;
  gap: 12px;
  align-content: start;
  min-width: 0;
  min-height: 0;
  max-height: 100%;
  overflow-x: hidden;
  overflow-y: auto;
}

.sidebar-preview-list,
.detail-list {
  display: grid;
  gap: 6px;
  list-style: none;
  padding: 0;
  margin: 0;
}

.sidebar-preview-list li {
  display: grid;
  gap: 2px;
}

.sidebar-preview-list li strong {
  font-size: 12px;
  font-weight: 620;
  color: var(--hh-text-primary);
}

.sidebar-preview-list li span {
  font-size: 10px;
  color: var(--hh-text-muted);
}

.detail-list li {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  color: var(--hh-text-secondary);
}

.detail-list li em {
  font-size: 9px;
  font-weight: 720;
  color: var(--hh-accent);
  font-style: normal;
  text-transform: uppercase;
}
</style>
