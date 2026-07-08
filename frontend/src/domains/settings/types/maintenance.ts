export type MaintenanceTone = 'good' | 'warn' | 'bad' | 'neutral'

export interface MaintenanceOverview {
  generated_at: string
  inventory: MaintenanceInventoryItem[]
  backups: MaintenanceBackupItem[]
  actions: MaintenanceActionDescriptor[]
}

export interface MaintenanceInventoryItem {
  id: string
  label: string
  description: string
  kind: string
  path_label: string
  exists: boolean
  size_bytes: number | null
  file_count: number | null
  status: string
  detail: string
}

export interface MaintenanceBackupItem {
  id: string
  label: string
  created_at: string | null
  path_label: string
  size_bytes: number
  file_count: number
  has_database_dump: boolean
  has_vault_snapshot: boolean
  has_storage_snapshot: boolean
  manifest_present: boolean
}

export interface MaintenanceActionDescriptor {
  id: string
  label: string
  description: string
  icon: string
  destructive: boolean
  enabled: boolean
  requires_confirmation: boolean
  confirmation_phrase: string | null
  disabled_reason: string | null
}

export interface MaintenanceActionRequest {
  confirmation?: string
  backup_id?: string
}

export interface MaintenanceActionResponse {
  action_id: string
  status: string
  message: string
  completed_at: string
  backup: MaintenanceBackupItem | null
}
