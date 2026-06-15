<script setup lang="ts">
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import type { TemplateRecipientVariableMapping } from './templateLibrary'

const props = defineProps<{
  templateVariables: string[]
  mapping: TemplateRecipientVariableMapping
  summary: string
}>()

const emit = defineEmits<{
  'update:mapping': [mapping: TemplateRecipientVariableMapping]
  fill: []
  buildPreview: []
}>()

function updateMappingField(
  key: keyof TemplateRecipientVariableMapping,
  value: string
): void {
  emit('update:mapping', {
    ...props.mapping,
    [key]: value
  })
}
</script>

<template>
  <div class="template-recipient-mapping">
    <div class="template-recipient-mapping-header">
      <h4>Recipient mapping</h4>
      <span class="template-recipient-mapping-summary">{{ summary }}</span>
    </div>
    <div class="template-recipient-mapping-grid">
      <label>
        <span>To variable</span>
        <select :value="mapping.toVariable" @change="updateMappingField('toVariable', ($event.target as HTMLSelectElement).value)">
          <option value="">Not mapped</option>
          <option v-for="variable in templateVariables" :key="`to:${variable}`" :value="variable">
            {{ variable }}
          </option>
        </select>
      </label>
      <label>
        <span>CC variable</span>
        <select :value="mapping.ccVariable" @change="updateMappingField('ccVariable', ($event.target as HTMLSelectElement).value)">
          <option value="">Not mapped</option>
          <option v-for="variable in templateVariables" :key="`cc:${variable}`" :value="variable">
            {{ variable }}
          </option>
        </select>
      </label>
      <label>
        <span>BCC variable</span>
        <select :value="mapping.bccVariable" @change="updateMappingField('bccVariable', ($event.target as HTMLSelectElement).value)">
          <option value="">Not mapped</option>
          <option v-for="variable in templateVariables" :key="`bcc:${variable}`" :value="variable">
            {{ variable }}
          </option>
        </select>
      </label>
    </div>
    <div class="template-recipient-mapping-actions">
      <Button type="button" variant="ghost" size="sm" @click="emit('fill')">
        <Icon icon="tabler:user-share" size="16" />
        Fill mapped variables
      </Button>
      <Button type="button" variant="ghost" size="sm" @click="emit('buildPreview')">
        <Icon icon="tabler:users-group" size="16" />
        Build rows from To
      </Button>
    </div>
  </div>
</template>
