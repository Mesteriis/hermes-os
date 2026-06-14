<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { ProjectItem } from '../types/home'

defineProps<{
  projects: ProjectItem[]
}>()

const emit = defineEmits<{
  navigateToProjects: []
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="home-active-projects">
    <section class="panel full-band">
      <header class="panel-title-row">
        <h2>Active Projects</h2>
        <button type="button" class="link-button" @click="emit('navigateToProjects')">
          View all projects
        </button>
      </header>
      <div class="project-card-row" data-widget-fit-content>
        <template v-if="projects.length > 0">
          <article v-for="project in projects" :key="project.name" class="compact-project">
            <span :class="['round-icon', project.tone]">
              <Icon :icon="project.icon" :size="20" />
            </span>
            <div>
              <strong>{{ project.name }}</strong>
              <small>{{ project.kind }}</small>
            </div>
            <progress class="progress" max="100" :value="project.progress" :aria-label="`${project.name} progress`">
              {{ project.progress }}%
            </progress>
            <em>{{ project.progress }}%</em>
          </article>
        </template>
        <button type="button" class="new-tile" disabled>
          <Icon icon="tabler:plus" :size="22" />
          New Project
        </button>
      </div>
    </section>
  </div>
</template>
