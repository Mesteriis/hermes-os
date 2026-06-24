import { createRouter, createWebHashHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

import HomeView from './views/HomeView.vue'
import CommunicationsView from './views/CommunicationsView.vue'
import TimelineView from './views/TimelineView.vue'
import PersonsView from './views/PersonsView.vue'
import ProjectsView from './views/ProjectsView.vue'
import TasksView from './views/TasksView.vue'
import CalendarView from './views/CalendarView.vue'
import DocumentsView from './views/DocumentsView.vue'
import NotesView from './views/NotesView.vue'
import KnowledgeView from './views/KnowledgeView.vue'
import ReviewView from './views/ReviewView.vue'
import SettingsView from './views/SettingsView.vue'
import AgentsView from './views/AgentsView.vue'
import OrganizationsView from './views/OrganizationsView.vue'
import EventTracingView from './views/EventTracingView.vue'

const routes: RouteRecordRaw[] = [
  { path: '/', redirect: '/home' },
  { path: '/home', name: 'home', component: HomeView },
  { path: '/communications', name: 'communications', component: CommunicationsView },
  { path: '/timeline', name: 'timeline', component: TimelineView },
  { path: '/persons', name: 'persons', component: PersonsView },
  { path: '/projects', name: 'projects', component: ProjectsView },
  { path: '/tasks', name: 'tasks', component: TasksView },
  { path: '/calendar', name: 'calendar', component: CalendarView },
  { path: '/documents', name: 'documents', component: DocumentsView },
  { path: '/notes', name: 'notes', component: NotesView },
  { path: '/knowledge', name: 'knowledge', component: KnowledgeView },
  { path: '/review', name: 'review', component: ReviewView },
  { path: '/event-tracing', name: 'event-tracing', component: EventTracingView },
  { path: '/settings', name: 'settings', component: SettingsView },
  { path: '/agents', name: 'agents', component: AgentsView },
  { path: '/organizations', name: 'organizations', component: OrganizationsView }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

export default router
