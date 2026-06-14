<script setup lang="ts">
import type { TaskItem } from '../types/home'

defineProps<{
  tasks: TaskItem[]
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="home-priorities">
    <section class="panel priorities-panel" data-widget-fit-content>
      <header class="panel-title-row">
        <div>
          <h2>Today's Priorities</h2>
          <p>Focus on what matters most</p>
        </div>
      </header>
      <div class="task-stack">
        <label v-for="task in tasks.slice(0, 5)" :key="task.title">
          <input type="checkbox" />
          <span>
            <strong>{{ task.title }}</strong>
            <small>{{ task.assignee }} · {{ task.due }}</small>
          </span>
          <em :class="{ high: task.priority === 'High' }">{{ task.priority }}</em>
        </label>
      </div>
      <button type="button" class="link-row" disabled>
        View all tasks
      </button>
    </section>
  </div>
</template>
