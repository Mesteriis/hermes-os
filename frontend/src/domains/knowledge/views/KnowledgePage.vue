<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useKnowledgeStore, graphNodeKindIcon, graphNodeKindLabel } from '../stores/knowledge'
import { useGraphSummaryQuery, useContradictionsQuery } from '../queries/useKnowledgeQuery'
import KnowledgeGraphCanvas from '../components/KnowledgeGraphCanvas.vue'
import KnowledgeNodeInspector from '../components/KnowledgeNodeInspector.vue'
import KnowledgePolygraphReview from '../components/KnowledgePolygraphReview.vue'
import Icon from '../../../shared/ui/Icon.vue'
import type { GraphNode } from '../types/knowledge'

const store = useKnowledgeStore()

const { data: summaryData, error: summaryError, isLoading: summaryLoading } = useGraphSummaryQuery()
const { data: contradictionsData, error: contradictionsError, isLoading: contradictionsLoading } = useContradictionsQuery(50)

onMounted(() => {
  if (summaryData.value) {
    store.setGraphSummary(summaryData.value, '')
  }
})

watch(summaryData, (val) => {
  if (val) {
    store.setGraphSummary(val, '')
  }
})
watch(summaryError, (err) => {
  if (err) {
    store.setGraphSummary(null, (err as Error)?.message || 'Unknown error')
  }
})
watch(contradictionsData, (val) => {
  if (val) {
    store.setContradictionObservations(val)
  }
})

const searchQuery = ref('')
const searchLoading = ref(false)

async function handleSearch() {
  if (!searchQuery.value.trim()) return
  searchLoading.value = true
  try {
    await store.runGraphSearch(searchQuery.value)
  } finally {
    searchLoading.value = false
  }
}

async function handleSelectSearchResult(node: GraphNode) {
  searchQuery.value = ''
  store.setGraphSearchResults([], '')
  await store.selectGraphNode(node)
}

const suggestedContradictionsCount = computed(() => {
  return store.contradictionObservations.filter((o) => o.review_state === 'suggested').length
})

async function loadGraphNodeChoices() {
  await store.loadGraphNodeChoices()
}
</script>

<template>
  <div class="knowledge-page">
    <!-- Filter tabs -->
    <div class="filter-tabs" v-if="store.graphFilterChips.length > 0">
      <button
        v-for="chip in store.graphFilterChips"
        :key="chip.kind"
        class="filter-chip"
        @click="loadGraphNodeChoices"
      >
        <Icon :icon="chip.icon" class="chip-icon" />
        <span class="chip-label">{{ chip.label }}</span>
        <span class="chip-count">{{ chip.count }}</span>
      </button>
      <button class="filter-chip rebuild-btn" @click="loadGraphNodeChoices">
        <Icon icon="tabler:refresh" class="chip-icon" />
        <span>Rebuild</span>
      </button>
    </div>

    <!-- Loading state -->
    <div v-if="summaryLoading" class="loading-banner">
      <Icon icon="tabler:loader-2" class="spin" />
      <span>Loading knowledge graph...</span>
    </div>

    <!-- Error state -->
    <div v-else-if="store.graphError && !store.graphSummary" class="error-banner">
      <Icon icon="tabler:alert-circle" />
      <span>{{ store.graphError }}</span>
    </div>

    <!-- Main layout -->
    <div v-else class="knowledge-layout">
      <!-- Workbench -->
      <div class="graph-workbench">
        <!-- Toolbar -->
        <div class="workbench-toolbar">
          <form class="search-form" @submit.prevent="handleSearch">
            <input
              v-model="searchQuery"
              type="text"
              placeholder="Search graph nodes..."
              class="search-input"
            />
            <button
              type="submit"
              class="primary-button"
              :disabled="searchLoading || !searchQuery.trim()"
            >
              <Icon icon="tabler:search" />
              Search
            </button>
          </form>
        </div>

        <!-- Search results -->
        <div v-if="store.graphSearchResults.length > 0" class="search-results">
          <div
            v-for="node in store.graphSearchResults"
            :key="node.node_id"
            class="search-result-item"
            @click="handleSelectSearchResult(node)"
          >
            <Icon :icon="graphNodeKindIcon(node.node_kind)" class="result-icon" />
            <div class="result-info">
              <span class="result-label">{{ node.label }}</span>
              <span class="result-kind">{{ graphNodeKindLabel(node.node_kind) }}</span>
            </div>
          </div>
        </div>

        <!-- Graph canvas -->
        <KnowledgeGraphCanvas />

        <!-- Status bar -->
        <div class="status-bar" v-if="store.graphSummary">
          <span>{{ store.graphSummary.node_counts.length }} node types</span>
          <span>{{ store.graphSummary.edge_counts.length }} edge types</span>
          <span>{{ store.graphSummary.evidence_count }} evidence items</span>
        </div>
      </div>

      <!-- Side rail -->
      <div class="knowledge-side-rail">
        <!-- Polygraph Review -->
        <div class="rail-section">
          <h3 class="rail-section-title">
            <Icon icon="tabler:git-compare" />
            Polygraph Review
            <span v-if="suggestedContradictionsCount > 0" class="section-badge">
              {{ suggestedContradictionsCount }}
            </span>
          </h3>
          <KnowledgePolygraphReview
            :observations="store.contradictionObservations"
            :error="(contradictionsError as Error)?.message || ''"
            :loading="contradictionsLoading"
          />
        </div>

        <!-- Node Inspector -->
        <div class="rail-section">
          <h3 class="rail-section-title">
            <Icon icon="tabler:info-circle" />
            Node Inspector
          </h3>
          <KnowledgeNodeInspector />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.knowledge-page {
  display: flex;
  flex-direction: column;
  gap: 8px;
  height: 100%;
  padding: 16px;
}

.filter-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  font-size: 12px;
  border-radius: 999px;
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  cursor: pointer;
  transition: background 0.15s;
}

.filter-chip:hover {
  background: hsl(var(--accent));
}

.chip-icon {
  font-size: 14px;
  color: hsl(var(--muted-foreground));
}

.chip-label {
  color: hsl(var(--foreground));
}

.chip-count {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted) / 0.3);
  padding: 0 6px;
  border-radius: 999px;
}

.rebuild-btn {
  margin-left: auto;
}

.loading-banner,
.error-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  font-size: 14px;
}

.loading-banner {
  color: hsl(var(--muted-foreground));
}

.error-banner {
  color: hsl(var(--destructive));
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.knowledge-layout {
  display: flex;
  gap: 12px;
  flex: 1;
  min-height: 0;
}

.graph-workbench {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}

.workbench-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
}

.search-form {
  display: flex;
  gap: 6px;
  flex: 1;
}

.search-input {
  flex: 1;
  padding: 6px 12px;
  font-size: 13px;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  background: hsl(var(--background));
  color: hsl(var(--foreground));
  outline: none;
}

.search-input:focus {
  border-color: hsl(var(--ring));
}

.primary-button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 14px;
  font-size: 13px;
  border-radius: 6px;
  border: none;
  background: hsl(var(--primary));
  color: hsl(var(--primary-foreground));
  cursor: pointer;
}

.primary-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.search-results {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 8px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
}

.search-result-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.search-result-item:hover {
  background: hsl(var(--accent));
}

.result-icon {
  font-size: 14px;
  color: hsl(var(--muted-foreground));
}

.result-info {
  display: flex;
  flex-direction: column;
}

.result-label {
  font-size: 12px;
  font-weight: 500;
  color: hsl(var(--foreground));
}

.result-kind {
  font-size: 10px;
  color: hsl(var(--muted-foreground));
}

.status-bar {
  display: flex;
  gap: 16px;
  padding: 6px 12px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
}

.knowledge-side-rail {
  width: 320px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
}

.rail-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.rail-section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  font-weight: 600;
  margin: 0;
  color: hsl(var(--foreground));
}

.section-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 1px 7px;
  border-radius: 999px;
  background: hsl(var(--destructive) / 0.15);
  color: hsl(var(--destructive));
}
</style>
