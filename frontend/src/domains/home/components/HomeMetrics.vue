<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { StatCard } from '../types/home'

defineProps<{
  stats: StatCard[]
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="home-metrics">
    <div class="metric-grid home-metrics" data-widget-fit-content>
      <article v-for="metric in stats" :key="metric.label" class="metric-card">
        <span>{{ metric.label }}</span>
        <div>
          <strong>{{ metric.value }}</strong>
          <Icon :icon="metric.icon" :size="26" />
        </div>
        <small>↑ {{ metric.delta }}</small>
      </article>
      <article class="metric-card focus-card">
        <span>Focus Score</span>
        <div class="score-ring"><strong>78</strong></div>
        <small>Good ↑ 5</small>
      </article>
    </div>
  </div>
</template>

<style scoped>
.home-metrics {
  grid-template-columns: repeat(6, minmax(110px, 1fr));
}

.metric-grid {
  display: grid;
  gap: 12px;
  padding: 18px;
}

.metric-card {
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  padding: 16px;
  display: grid;
  gap: 10px;
  min-height: 100px;
}

.metric-card span {
  color: var(--hh-color-text-muted);
  font-size: 12px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.metric-card > div {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.metric-card strong {
  font-size: 28px;
  font-weight: 700;
  color: var(--hh-color-text-bright);
}

.metric-card small {
  color: var(--hh-color-accent);
  font-size: 12px;
  font-weight: 500;
}

.focus-card {
  background: linear-gradient(135deg, rgba(45, 235, 204, 0.08), rgba(42, 139, 167, 0.12));
  border-color: rgba(45, 235, 204, 0.25);
}

.score-ring {
  display: grid;
  place-items: center;
  width: 48px;
  height: 48px;
  border: 4px solid rgba(45, 235, 204, 0.82);
  border-right-color: rgba(42, 139, 167, 0.8);
  border-bottom-color: rgba(236, 183, 70, 0.8);
  border-radius: var(--hh-radius-round);
  margin: 0 !important;
}

.score-ring strong {
  font-size: 14px;
}
</style>
