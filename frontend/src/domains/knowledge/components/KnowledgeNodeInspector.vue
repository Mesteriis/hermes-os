<script setup lang="ts">
import { useKnowledgeStore, graphNodeKindLabel } from '../stores/knowledge'
import Icon from '../../../shared/ui/Icon.vue'

const store = useKnowledgeStore()
</script>

<template>
  <div class="inspector-panels">
    <!-- Selected Node -->
    <div class="inspector-section" v-if="store.selectedGraphNode">
      <h4 class="section-title">Selected Node</h4>
      <div class="property-list">
        <div class="property-row">
          <span class="prop-key">Kind</span>
          <span class="prop-value">{{ graphNodeKindLabel(store.selectedGraphNode.node_kind) }}</span>
        </div>
        <div class="property-row">
          <span class="prop-key">Label</span>
          <span class="prop-value">{{ store.selectedGraphNode.label }}</span>
        </div>
        <div class="property-row" v-if="store.selectedGraphNode.stable_key">
          <span class="prop-key">Stable Key</span>
          <span class="prop-value mono">{{ store.selectedGraphNode.stable_key }}</span>
        </div>
        <div
          v-for="prop in store.selectedGraphProperties"
          :key="prop.key"
          class="property-row"
        >
          <span class="prop-key">{{ prop.key }}</span>
          <span class="prop-value">{{ String(prop.value) }}</span>
        </div>
        <div class="property-row">
          <span class="prop-key">Created</span>
          <span class="prop-value">{{ new Date(store.selectedGraphNode.created_at).toLocaleDateString() }}</span>
        </div>
        <div class="property-row">
          <span class="prop-key">Updated</span>
          <span class="prop-value">{{ new Date(store.selectedGraphNode.updated_at).toLocaleDateString() }}</span>
        </div>
      </div>
    </div>

    <!-- Connections -->
    <div class="inspector-section" v-if="store.graphNeighborCounts.length > 0">
      <h4 class="section-title">Connections</h4>
      <div class="property-list">
        <div
          v-for="nc in store.graphNeighborCounts"
          :key="nc.kind"
          class="property-row"
        >
          <span class="prop-key">{{ graphNodeKindLabel(nc.kind) }}</span>
          <span class="prop-value count">{{ nc.count }}</span>
        </div>
      </div>
    </div>

    <!-- Evidence -->
    <div class="inspector-section" v-if="store.graphNeighborhood && store.graphNeighborhood.evidence.length > 0">
      <h4 class="section-title">Evidence</h4>
      <div
        v-for="ev in store.graphNeighborhood.evidence.slice(0, 5)"
        :key="ev.edge_id"
        class="evidence-item"
      >
        <p class="evidence-excerpt">{{ ev.excerpt || 'No excerpt' }}</p>
        <p class="evidence-meta">{{ ev.source_kind }} · {{ ev.source_id }}</p>
      </div>
    </div>

    <!-- Graph Statistics -->
    <div class="inspector-section" v-if="store.graphSummary">
      <h4 class="section-title">Graph Statistics</h4>
      <div class="property-list">
        <div class="property-row">
          <span class="prop-key">Nodes</span>
          <span class="prop-value count">
            {{ store.graphSummary.node_counts.reduce((acc, c) => acc + c.count, 0) }}
          </span>
        </div>
        <div class="property-row">
          <span class="prop-key">Connections</span>
          <span class="prop-value count">
            {{ store.graphSummary.edge_counts.reduce((acc, c) => acc + c.count, 0) }}
          </span>
        </div>
        <div class="property-row">
          <span class="prop-key">Evidence</span>
          <span class="prop-value count">{{ store.graphSummary.evidence_count }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.inspector-panels {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.inspector-section {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  padding: 12px;
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  margin: 0 0 8px;
  color: hsl(var(--foreground));
}

.property-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.property-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}

.prop-key {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  white-space: nowrap;
}

.prop-value {
  font-size: 12px;
  color: hsl(var(--foreground));
  text-align: right;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 180px;
}

.prop-value.mono {
  font-family: ui-monospace, monospace;
  font-size: 11px;
}

.prop-value.count {
  font-weight: 600;
}

.evidence-item {
  padding: 8px 0;
  border-bottom: 1px solid hsl(var(--border));
}

.evidence-item:last-child {
  border-bottom: none;
}

.evidence-excerpt {
  font-size: 12px;
  color: hsl(var(--foreground));
  margin: 0 0 4px;
  line-height: 1.4;
}

.evidence-meta {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  margin: 0;
}
</style>
