<script setup lang="ts">
import { computed } from 'vue'
import type { DataTableColumn, DataTableRow } from './DataDisplay.types'

const props = withDefaults(defineProps<{
  columns: DataTableColumn[]
  rows: DataTableRow[]
  caption?: string
  emptyText?: string
  density?: 'compact' | 'regular'
  class?: string
}>(), {
  emptyText: 'No rows',
  density: 'regular'
})

const classes = computed(() => [
  'hermes-table-shell',
  `hermes-table-shell--${props.density}`,
  props.class
])

function cellValue(row: DataTableRow, key: string): string {
  const value = row[key]
  return value === null || value === undefined || value === '' ? '-' : String(value)
}
</script>

<template>
  <div :class="classes">
    <table class="hermes-table">
      <caption v-if="caption" class="hermes-table-caption">{{ caption }}</caption>
      <thead>
        <tr>
          <th
            v-for="column in columns"
            :key="column.key"
            scope="col"
            :class="['hermes-table-heading', `hermes-table-cell--${column.align ?? 'left'}`]"
          >
            {{ column.label }}
          </th>
        </tr>
      </thead>
      <tbody v-if="rows.length > 0">
        <tr v-for="(row, rowIndex) in rows" :key="String(row.id ?? rowIndex)" class="hermes-table-row">
          <td
            v-for="column in columns"
            :key="column.key"
            :class="['hermes-table-cell', `hermes-table-cell--${column.align ?? 'left'}`]"
          >
            <slot :name="`cell-${column.key}`" :row="row" :column="column" :value="cellValue(row, column.key)">
              {{ cellValue(row, column.key) }}
            </slot>
          </td>
        </tr>
      </tbody>
      <tbody v-else>
        <tr>
          <td class="hermes-table-empty" :colspan="Math.max(columns.length, 1)">
            {{ emptyText }}
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
