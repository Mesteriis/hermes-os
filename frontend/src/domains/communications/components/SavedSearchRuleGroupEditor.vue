<script setup lang="ts">
defineOptions({ name: 'SavedSearchRuleGroupEditor' })

import {
  createSavedSearchRuleCondition,
  createSavedSearchRuleGroup,
  type SavedSearchRuleGroup,
  type SavedSearchRuleNode
} from '../forms/savedSearchForm'
import {
  savedSearchRuleGroupDepthLabel,
  savedSearchRuleGroupSummary
} from './savedSearchRuleTreePresentation'

const props = defineProps<{
  group: SavedSearchRuleGroup
  isRoot?: boolean
  depth?: number
}>()

const emit = defineEmits<{
  removeGroup: []
}>()

function addRule() {
  props.group.children.push(createSavedSearchRuleCondition())
}

function addGroup() {
  props.group.children.push(createSavedSearchRuleGroup('all', [createSavedSearchRuleCondition()]))
}

function removeNode(nodeId: string) {
  props.group.children = props.group.children.filter((child) => child.id !== nodeId)
}

function removeNestedGroup(node: SavedSearchRuleNode) {
  if (node.kind !== 'group') return
  removeNode(node.id)
}

function nextDepth(): number {
  return (props.depth ?? 0) + 1
}
</script>

<template>
  <div class="saved-search-group-builder" :class="{ root: isRoot }">
    <div class="saved-search-group-builder-header">
      <div class="saved-search-group-builder-summary">
        <span class="saved-search-group-builder-depth">{{ savedSearchRuleGroupDepthLabel(depth ?? 0) }}</span>
        <span class="saved-search-group-builder-description">{{ savedSearchRuleGroupSummary(group) }}</span>
      </div>
      <label class="saved-search-field">
        <span>{{ isRoot ? 'Match' : 'Group match' }}</span>
        <select v-model="group.matchMode">
          <option value="all">All conditions</option>
          <option value="any">Any condition</option>
        </select>
      </label>
      <div class="saved-search-group-builder-actions">
        <button class="saved-search-rule-add" type="button" @click="addRule">+ Rule</button>
        <button class="saved-search-rule-add" type="button" @click="addGroup">+ Group</button>
        <button
          v-if="!isRoot"
          class="saved-search-rule-remove"
          type="button"
          @click="emit('removeGroup')"
        >
          Remove group
        </button>
      </div>
    </div>

    <div v-if="group.children.length" class="saved-search-rules saved-search-rules-tree">
      <template v-for="node in group.children" :key="node.id">
        <label v-if="node.kind === 'rule'" class="saved-search-rule-row">
          <select v-model="node.field">
            <option value="subject">Subject</option>
            <option value="body">Body</option>
            <option value="sender">Sender</option>
            <option value="all">All</option>
          </select>
          <select v-model="node.operator">
            <option value=":">Contains</option>
            <option value="=">Equals</option>
          </select>
          <input v-model="node.value" type="text" autocomplete="off" />
          <button class="saved-search-rule-remove" type="button" @click="removeNode(node.id)">Remove</button>
        </label>
        <SavedSearchRuleGroupEditor
          v-else
          :group="node"
          :depth="nextDepth()"
          @remove-group="removeNestedGroup(node)"
        />
      </template>
    </div>
    <div v-else class="saved-search-rule-empty">No rules yet</div>
  </div>
</template>
