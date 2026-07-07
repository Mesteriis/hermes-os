<script setup lang="ts">
import { DialogRoot, DialogTrigger, DialogPortal, DialogOverlay, DialogContent, DialogTitle, DialogDescription, DialogClose } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  open?: boolean
  title?: string
  description?: string
  closeLabel?: string
  showClose?: boolean
  class?: string
  contentClass?: string
}>(), {
  open: false,
  closeLabel: 'Close dialog',
  showClose: true
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const contentClasses = computed(() => ['hermes-dialog-content', props.contentClass])
</script>

<template>
  <DialogRoot :open="open" @update:open="(val) => emit('update:open', val)">
    <DialogTrigger v-if="$slots.trigger" as-child>
      <slot name="trigger" />
    </DialogTrigger>
    <DialogPortal>
      <DialogOverlay class="hermes-dialog-overlay">
        <DialogContent :class="contentClasses">
          <div class="hermes-dialog-header">
            <DialogTitle v-if="title" class="hermes-dialog-title">{{ title }}</DialogTitle>
            <DialogDescription v-if="description" class="hermes-dialog-description">{{ description }}</DialogDescription>
            <slot name="header" />
          </div>
          <div class="hermes-dialog-body">
            <slot />
          </div>
          <div v-if="$slots.footer" class="hermes-dialog-footer">
            <slot name="footer" />
          </div>
          <DialogClose v-if="showClose" class="hermes-dialog-close" as-child>
            <button class="hermes-dialog-close-btn" type="button" :aria-label="closeLabel">
              <Icon icon="tabler:x" size="1.125rem" />
            </button>
          </DialogClose>
        </DialogContent>
      </DialogOverlay>
    </DialogPortal>
  </DialogRoot>
</template>
