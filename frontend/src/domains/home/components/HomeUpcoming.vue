<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
</script>

<template>
  <div class="widget-frame" data-widget-id="home-upcoming">
    <section class="panel schedule-panel" data-widget-fit-content>
      <header class="panel-title-row">
        <div>
          <h2>Upcoming</h2>
          <p>Your schedule</p>
        </div>
      </header>
      <div class="schedule-list">
        <article>
          <time>Today</time>
          <strong>14:00 Call with Acme Corp</strong>
          <span>16:30 Review Q2 Report</span>
        </article>
        <article>
          <time>Tomorrow</time>
          <strong>10:00 Project Hermes - Planning</strong>
          <span>15:00 Design Review</span>
        </article>
      </div>
      <button type="button" class="link-row" disabled>
        View full calendar <Icon icon="tabler:arrow-right" :size="15" />
      </button>
    </section>
  </div>
</template>
