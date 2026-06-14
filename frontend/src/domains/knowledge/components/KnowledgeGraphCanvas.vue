<script setup lang="ts">
import { useKnowledgeStore } from '../stores/knowledge'
import { graphNodeKindIcon } from '../stores/knowledge'

const store = useKnowledgeStore()
</script>

<template>
  <div class="graph-canvas">
    <template v-if="store.graphError && !store.graphNeighborhood">
      <div class="state-card error-card">
        <Icon icon="tabler:alert-circle" class="state-icon error-icon" />
        <p>{{ store.graphError }}</p>
      </div>
    </template>

    <template v-else-if="!store.graphSummary">
      <div class="state-card loading-card">
        <Icon icon="tabler:loader-2" class="state-icon spin" />
        <p>Loading graph summary...</p>
      </div>
    </template>

    <template v-else-if="store.graphSummary.is_empty">
      <div class="state-card empty-card">
        <Icon icon="tabler:graph" class="state-icon" />
        <p>No knowledge graph projection yet</p>
      </div>
    </template>

    <template v-else-if="store.graphNeighborhood && store.graphCanvasNodes.length > 0">
      <svg viewBox="0 0 100 100" preserveAspectRatio="none" class="graph-svg">
        <line
          v-for="(edge, i) in store.graphCanvasEdges"
          :key="'edge-' + i"
          :x1="edge.x1" :y1="edge.y1"
          :x2="edge.x2" :y2="edge.y2"
          :class="['graph-edge', edge.review_state === 'suggested' ? 'edge-suggested' : '']"
        />
        <text
          v-for="(edge, i) in store.graphCanvasEdges"
          :key="'edge-label-' + i"
          :x="(edge.x1 + edge.x2) / 2"
          :y="(edge.y1 + edge.y2) / 2"
          class="graph-edge-label"
        >{{ edge.label }}</text>
      </svg>
      <div class="graph-nodes-layer">
        <button
          v-for="node in store.graphCanvasNodes"
          :key="node.node_id"
          :class="[
            'graph-node-btn',
            'kind-' + node.node_kind,
            node.isSelected ? 'selected' : ''
          ]"
          :style="{ left: node.x + '%', top: node.y + '%' }"
          @click="store.selectGraphNode({ node_id: node.node_id, node_kind: node.node_kind, label: node.label, properties: {}, stable_key: '', created_at: '', updated_at: '' } as any)"
        >
          <Icon :icon="graphNodeKindIcon(node.node_kind)" class="node-icon" />
          <span class="node-label">{{ node.label }}</span>
        </button>
      </div>
    </template>

    <template v-else>
      <div class="state-card idle-card">
        <Icon icon="tabler:pointer" class="state-icon" />
        <p>Select a node from search or filter to explore</p>
      </div>
    </template>
  </div>
</template>

<style scoped>
.graph-canvas {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 400px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.graph-svg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.graph-edge {
  stroke: hsl(var(--border));
  stroke-width: 0.3;
}

.edge-suggested {
  stroke: hsl(var(--warning));
  stroke-dasharray: 1, 1;
}

.graph-edge-label {
  font-size: 2px;
  fill: hsl(var(--muted-foreground));
  text-anchor: middle;
  dominant-baseline: middle;
}

.graph-nodes-layer {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
}

.graph-node-btn {
  position: absolute;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  padding: 4px 8px;
  cursor: pointer;
  transition: box-shadow 0.15s, border-color 0.15s;
  max-width: 80px;
}

.graph-node-btn:hover {
  box-shadow: 0 2px 8px hsl(var(--shadow) / 0.15);
  border-color: hsl(var(--ring));
}

.graph-node-btn.selected {
  border-color: hsl(var(--ring));
  box-shadow: 0 0 0 2px hsl(var(--ring) / 0.3);
}

.node-icon {
  font-size: 18px;
  color: hsl(var(--foreground));
}

.node-label {
  font-size: 6px;
  color: hsl(var(--muted-foreground));
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 70px;
}

.state-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 24px;
  color: hsl(var(--muted-foreground));
}

.state-icon {
  font-size: 32px;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-icon {
  color: hsl(var(--destructive));
}
</style>
