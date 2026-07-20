<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../Icon.vue'

const props = withDefaults(defineProps<{
  title?: string
  description?: string
  icon?: string
  class?: string
}>(), {
  title: 'Nothing here yet',
  icon: 'tabler:inbox'
})

const classes = computed(() => ['hermes-state', 'hermes-state--empty', props.class])
</script>

<template>
  <section :class="classes">
    <Icon :icon="icon" size="2rem" class="hermes-state-icon" />
    <div class="hermes-state-copy">
      <h3 class="hermes-state-title">{{ title }}</h3>
      <p v-if="description" class="hermes-state-description">{{ description }}</p>
    </div>
    <div v-if="$slots.action" class="hermes-state-action">
      <slot name="action" />
    </div>
  </section>
</template>
