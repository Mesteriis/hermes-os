<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useNotesStore } from '../stores/notes'
import { useNotesQuery } from '../queries/useNotesQuery'
import type { NoteItem } from '../types/notes'
import NotesSourceFilters from '../components/NotesSourceFilters.vue'
import NotesList from '../components/NotesList.vue'
import NotesInsights from '../components/NotesInsights.vue'

const { t } = useI18n()
const store = useNotesStore()

const { data: notesData } = useNotesQuery()

const fallbackNotes: NoteItem[] = [
  { title: 'Welcome to Notes', body: 'This is your personal notes workspace. Notes from connected sources will appear here.', source: 'Hermes Hub', tag: '#reference', time: new Date().toISOString(), icon: 'tabler:notes' },
  { title: 'Meeting Notes Template', body: 'Use this template to capture key decisions, action items, and follow-ups from meetings.', source: 'Hermes Hub', tag: '#meeting', time: new Date().toISOString(), icon: 'tabler:clipboard-list' },
  { title: 'Project Ideas', body: 'A collection of project ideas and brainstorming notes for future development.', source: 'Hermes Hub', tag: '#idea', time: new Date().toISOString(), icon: 'tabler:lightbulb' },
  { title: 'Research Notes', body: 'Research findings and references organized by topic for easy retrieval.', source: 'Hermes Hub', tag: '#research', time: new Date().toISOString(), icon: 'tabler:books' }
]

const notes = computed<NoteItem[]>(() => notesData.value?.items ?? fallbackNotes)
</script>

<template>
  <section class="notes-page">
    <div class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small"><Icon icon="tabler:notes" :size="28" /></span>
        <div>
          <h1>{{ t('Notes') }}</h1>
          <p>{{ t('All your notes from connected sources') }}</p>
        </div>
      </div>
    </div>
    <div class="notes-layout">
      <NotesSourceFilters
        :active-sources="store.activeSources"
        :active-tags="store.activeTags"
        @toggle-source="store.toggleSource"
        @toggle-tag="store.toggleTag"
      />
      <NotesList
        :notes="notes"
        :search-query="store.searchQuery"
        @update:search-query="store.setSearchQuery"
      />
      <aside class="stacked-rail">
        <NotesInsights />
      </aside>
    </div>
  </section>
</template>
