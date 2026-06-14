<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { FeedItem } from '../types/home'

defineProps<{
  items: FeedItem[]
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="home-whats-new">
    <section class="panel feed-panel" data-widget-fit-content>
      <header class="panel-title-row">
        <div>
          <h2>What's New</h2>
          <p>Key changes and important updates</p>
        </div>
        <button type="button" class="ghost-button" disabled>All Types</button>
      </header>
      <div class="feed-list">
        <article v-for="item in items" :key="item.title + item.time" class="feed-row">
          <span :class="['round-icon', item.tone ?? '']">
            <Icon :icon="item.icon" :size="22" />
          </span>
          <div>
            <strong>{{ item.title }}</strong>
            <p>{{ item.meta }}</p>
            <em v-if="item.tag">{{ item.tag }}</em>
          </div>
          <time>{{ item.time }}</time>
        </article>
      </div>
      <button type="button" class="link-row" disabled>
        View all events <Icon icon="tabler:arrow-right" :size="15" />
      </button>
    </section>
  </div>
</template>
