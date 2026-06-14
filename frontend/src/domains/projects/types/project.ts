export type ProjectStatus = 'planning' | 'active' | 'on_hold' | 'completed' | 'archived'

export interface ProjectRecord {
  project_id: string
  name: string
  kind: string
  status: ProjectStatus
  description: string
  owner_display_name: string
  progress_percent: number
  start_date: string | null
  target_date: string | null
  created_at: string
  updated_at: string
}

export interface ProjectStats {
  message_count: number
  document_count: number
  people_count: number
  graph_connection_count: number
  latest_activity_at: string | null
}

export interface ProjectSummary {
  project: ProjectRecord
  stats: ProjectStats
  graph_node_id: string
}

export interface ProjectTimelineItem {
  item_kind: string
  item_id: string
  title: string
  subtitle: string
  occurred_at: string
}

export interface ProjectPersonSummary {
  display_name: string
  email_address: string
  interaction_count: number
  last_interaction_at: string | null
}

export interface ProjectMessageSummary {
  message_id: string
  subject: string
  sender: string
  occurred_at: string
}

export interface ProjectDocumentSummary {
  document_id: string
  document_kind: string
  title: string
  imported_at: string
}

export interface ProjectDetail {
  project: ProjectRecord
  stats: ProjectStats
  graph_node_id: string
  timeline: ProjectTimelineItem[]
  key_people: ProjectPersonSummary[]
  recent_messages: ProjectMessageSummary[]
  documents: ProjectDocumentSummary[]
}

export interface ProjectListResponse {
  items: ProjectSummary[]
}
