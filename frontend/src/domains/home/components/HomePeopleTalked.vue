<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { PersonItem } from '../types/home'

defineProps<{
  people: PersonItem[]
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="home-people-talked-to">
    <section class="panel mini-panel" data-widget-fit-content>
      <header class="panel-title-row">
        <h2>People You Talked To</h2>
        <button type="button" class="link-button" disabled>View all</button>
      </header>
      <div class="person-list">
        <article v-for="person in people" :key="person.name">
          <span class="round-icon ghost">
            <Icon icon="tabler:user" :size="20" />
          </span>
          <span>
            <strong>{{ person.name }}</strong>
            <small>{{ person.meta }}</small>
          </span>
          <Icon :icon="person.icon" :size="18" />
        </article>
      </div>
    </section>
  </div>
</template>
