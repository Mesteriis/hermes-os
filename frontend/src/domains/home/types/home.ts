export interface StatCard {
  label: string
  value: string
  delta: string
  icon: string
  tone?: string
}

export interface FeedItem {
  icon: string
  title: string
  meta: string
  time: string
  tag?: string
  tone?: string
}

export interface TaskItem {
  title: string
  assignee: string
  due: string
  priority: string
}

export interface PersonItem {
  name: string
  meta: string
  icon: string
}

export interface ProjectItem {
  name: string
  kind: string
  progress: number
  icon: string
  tone: string
}

export interface SystemStatusItem {
  label: string
  value: string
  status: 'ok' | 'warning' | 'error'
}
