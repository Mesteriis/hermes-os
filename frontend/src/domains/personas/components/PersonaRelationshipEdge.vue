<script setup lang="ts">
import { computed } from 'vue'
import { BaseEdge, EdgeLabelRenderer, getSmoothStepPath, type EdgeProps } from '@vue-flow/core'
import { Icon } from '../../../shared/ui'

type RelationshipEdgeData = {
  icon: string
  iconLabel: string
}

const props = defineProps<EdgeProps<RelationshipEdgeData>>()
const emit = defineEmits<{
  edgeAction: [edgeId: string]
}>()

const edgePathParams = computed(() =>
  getSmoothStepPath({
    sourceX: props.sourceX,
    sourceY: props.sourceY,
    sourcePosition: props.sourcePosition,
    targetX: props.targetX,
    targetY: props.targetY,
    targetPosition: props.targetPosition
  })
)

const edgePath = computed(() => edgePathParams.value[0])
const labelX = computed(() => edgePathParams.value[1])
const labelY = computed(() => edgePathParams.value[2])
const labelStyle = computed(() => ({
  transform: `translate(-50%, -50%) translate(${labelX.value}px, ${labelY.value}px)`
}))

function showDetails(event: MouseEvent): void {
  event.stopPropagation()
  emit('edgeAction', props.id)
}
</script>

<template>
  <BaseEdge
    :id="id"
    :path="edgePath"
    :marker-start="markerStart"
    :marker-end="markerEnd"
    :interaction-width="interactionWidth"
  />
  <EdgeLabelRenderer>
    <button
      type="button"
      class="personas-relationship-edge-action nodrag nopan"
      :style="labelStyle"
      :aria-label="data.iconLabel"
      @pointerdown.stop
      @click="showDetails"
    >
      <Icon :icon="data.icon" :size="16" />
    </button>
  </EdgeLabelRenderer>
</template>
