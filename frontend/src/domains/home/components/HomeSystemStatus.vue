<script setup lang="ts">
defineProps<{
  statusError: string
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="home-system-status">
    <section class="panel mini-panel" data-widget-fit-content>
      <header class="panel-title-row">
        <h2>System Status</h2>
      </header>
      <ul class="status-list">
        <li>All systems operational</li>
        <li>AI Agents online <span>5/5</span></li>
        <li>Data synchronized <span>2m ago</span></li>
        <li>Local AI models <span>Ready</span></li>
      </ul>
      <p v-if="statusError" class="inline-error">{{ statusError }}</p>
    </section>
  </div>
</template>
