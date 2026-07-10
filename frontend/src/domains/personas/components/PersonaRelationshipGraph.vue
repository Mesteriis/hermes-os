<script setup lang="ts">
import '@vue-flow/core/dist/style.css'
import { computed, nextTick, ref, shallowRef, watch } from 'vue'
import { VueFlow, type EdgeMouseEvent, type VueFlowStore } from '@vue-flow/core'
import { useI18n } from '../../../platform/i18n'
import { Avatar, Icon } from '../../../shared/ui'
import PersonaRelationshipEdge from './PersonaRelationshipEdge.vue'
import {
  buildRelationshipGraphEdges,
  buildRelationshipGraphNodes,
  relationshipGraphEdgeDetail,
  relationshipGraphNodeDetail,
  type RelationshipEdgeData,
  type RelationshipGraphDetail,
  type RelationshipNodeData
} from './personaRelationshipGraphModel'
import { personaInitials } from './personaWorkspaceElements'
import type { PersonaPanelProfile, Relationship } from '../types/persona'

const props = withDefaults(defineProps<{
  selectedPersona: PersonaPanelProfile
  entityLabels?: Readonly<Record<string, string>>
  relationships: readonly Relationship[]
}>(), {
  entityLabels: () => ({}),
  relationships: () => []
})

const { t } = useI18n()

const activeDetail = ref<RelationshipGraphDetail | null>(null)
const relationshipFlow = shallowRef<Pick<VueFlowStore, 'fitView' | 'zoomIn' | 'zoomOut'> | null>(null)
const graphFitViewOptions = {
  padding: 0.34,
  maxZoom: 0.94,
  duration: 240
}

const graphNodes = computed(() =>
  buildRelationshipGraphNodes({
    selectedPersona: props.selectedPersona,
    entityLabels: props.entityLabels,
    relationships: props.relationships,
    t
  })
)

const graphEdges = computed(() =>
  buildRelationshipGraphEdges({
    selectedPersona: props.selectedPersona,
    entityLabels: props.entityLabels,
    relationships: props.relationships,
    t
  })
)

watch(
  () => [props.selectedPersona.persona_id, graphNodes.value.length, graphEdges.value.length],
  () => {
    void fitGraph()
  },
  { flush: 'post' }
)

function handlePaneReady(flow: VueFlowStore): void {
  relationshipFlow.value = flow
  void fitGraph()
}

async function fitGraph(): Promise<void> {
  if (!relationshipFlow.value) return

  await nextTick()
  await relationshipFlow.value.fitView(graphFitViewOptions)
}

function zoomGraphIn(): void {
  void relationshipFlow.value?.zoomIn({ duration: 180 })
}

function zoomGraphOut(): void {
  void relationshipFlow.value?.zoomOut({ duration: 180 })
}

function showNodeDetail(data: RelationshipNodeData): void {
  activeDetail.value = relationshipGraphNodeDetail(data, t)
}

function showEdgeDetail(event: EdgeMouseEvent): void {
  event.event.stopPropagation()
  const data = event.edge.data as RelationshipEdgeData

  showEdgeData(data)
}

function showEdgeDetailById(edgeId: string): void {
  const edge = graphEdges.value.find((item) => item.id === edgeId)
  if (!edge?.data) return

  showEdgeData(edge.data)
}

function showEdgeData(data: RelationshipEdgeData): void {
  activeDetail.value = relationshipGraphEdgeDetail(data, t)
}

function hideDetail(): void {
  activeDetail.value = null
}
</script>

<template>
  <div class="personas-graph-stage" :aria-label="t('Relationship graph')">
    <VueFlow
      class="personas-relationship-flow"
      :nodes="graphNodes"
      :edges="graphEdges"
      :fit-view-on-init="false"
      :min-zoom="0.5"
      :max-zoom="1.45"
      :nodes-connectable="false"
      :edges-updatable="false"
      :select-nodes-on-drag="false"
      :pan-on-scroll="true"
      :zoom-on-double-click="false"
      @edge-click="showEdgeDetail"
      @pane-click="hideDetail"
      @pane-ready="handlePaneReady"
    >
      <template #edge-relationship="edgeProps">
        <PersonaRelationshipEdge v-bind="edgeProps" @edge-action="showEdgeDetailById" />
      </template>

      <template #node-default="{ data, selected }">
        <button
          type="button"
          class="personas-relationship-avatar-node"
          :class="{ 'is-selected': selected }"
          :aria-label="`${data.title}. ${t('Open details')}`"
          @click.stop="showNodeDetail(data)"
        >
          <Avatar size="lg" :fallback="data.initials" :alt="data.title" />
          <span>{{ data.title }}</span>
          <small>{{ data.kindLabel }}</small>
        </button>
      </template>
    </VueFlow>

    <div
      class="personas-graph-controls"
      :aria-label="t('Graph zoom controls')"
      @pointerdown.stop
      @click.stop
    >
      <button type="button" :aria-label="t('Zoom in')" @click="zoomGraphIn">
        <Icon icon="tabler:plus" :size="16" />
      </button>
      <button type="button" :aria-label="t('Zoom out')" @click="zoomGraphOut">
        <Icon icon="tabler:minus" :size="16" />
      </button>
      <button type="button" :aria-label="t('Fit graph to view')" @click="fitGraph">
        <Icon icon="tabler:focus-centered" :size="16" />
      </button>
    </div>

    <aside
      v-if="activeDetail"
      class="personas-graph-detail-popover"
      role="dialog"
      :aria-label="activeDetail.title"
    >
      <header>
        <span>{{ activeDetail.eyebrow }}</span>
        <button type="button" :aria-label="t('Close')" @click="hideDetail">
          <Icon icon="tabler:x" />
        </button>
      </header>
      <strong>{{ activeDetail.title }}</strong>
      <p>{{ activeDetail.description }}</p>
      <dl>
        <div v-for="row in activeDetail.rows" :key="row.label">
          <dt>{{ row.label }}</dt>
          <dd>{{ row.value }}</dd>
        </div>
      </dl>
    </aside>

    <div v-if="relationships.length === 0" class="personas-graph-empty">
      <strong>{{ personaInitials(selectedPersona) }}</strong>
      <span>{{ t('No relationships for selected persona') }}</span>
    </div>
  </div>
</template>
