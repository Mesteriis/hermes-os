import type {
  MaintenanceActionDescriptor,
  MaintenanceBackupItem,
  MaintenanceInventoryItem,
  MaintenanceTone
} from '../types/maintenance'

export interface MaintenanceSummaryTile {
  id: string
  label: string
  value: string
  detail: string
  icon: string
  tone: MaintenanceTone
}

export interface MaintenanceInventoryRow extends MaintenanceInventoryItem {
  sizeLabel: string
  fileCountLabel: string
  tone: MaintenanceTone
  icon: string
}

export interface MaintenanceBackupRow extends MaintenanceBackupItem {
  sizeLabel: string
  createdAtLabel: string
  contentsLabel: string
}

export interface MaintenanceActionRow extends MaintenanceActionDescriptor {
  tone: MaintenanceTone
  buttonLabel: string
  availabilityLabel: string
}

export function buildMaintenanceSummaryTiles(
  inventory: MaintenanceInventoryItem[],
  backups: MaintenanceBackupItem[]
): MaintenanceSummaryTile[] {
  const database = findInventory(inventory, 'database')
  const mail = findInventory(inventory, 'mail_blobs')
  const logs = findInventory(inventory, 'dev_logs')
  const backupBytes = backups.reduce((sum, backup) => sum + backup.size_bytes, 0)
  return [
    {
      id: 'database',
      label: 'Database',
      value: formatBytes(database?.size_bytes ?? null),
      detail: database?.detail ?? 'Database size unavailable',
      icon: 'tabler:database',
      tone: toneFromStatus(database?.status)
    },
    {
      id: 'mail',
      label: 'Mail blobs',
      value: formatBytes(mail?.size_bytes ?? null),
      detail: `${mail?.file_count ?? 0} files`,
      icon: 'tabler:mail',
      tone: toneFromStatus(mail?.status)
    },
    {
      id: 'logs',
      label: 'Logs',
      value: formatBytes(logs?.size_bytes ?? null),
      detail: `${logs?.file_count ?? 0} files`,
      icon: 'tabler:file-analytics',
      tone: toneFromStatus(logs?.status)
    },
    {
      id: 'backups',
      label: 'Backups',
      value: String(backups.length),
      detail: formatBytes(backupBytes),
      icon: 'tabler:archive',
      tone: backups.length > 0 ? 'good' : 'neutral'
    }
  ]
}

export function buildMaintenanceInventoryRows(items: MaintenanceInventoryItem[]): MaintenanceInventoryRow[] {
  return items.map((item) => ({
    ...item,
    sizeLabel: formatBytes(item.size_bytes),
    fileCountLabel: item.file_count === null ? 'n/a' : String(item.file_count),
    tone: toneFromStatus(item.status),
    icon: inventoryIcon(item)
  }))
}

export function buildMaintenanceBackupRows(items: MaintenanceBackupItem[]): MaintenanceBackupRow[] {
  return items.map((item) => ({
    ...item,
    sizeLabel: formatBytes(item.size_bytes),
    createdAtLabel: formatDate(item.created_at),
    contentsLabel: backupContentsLabel(item)
  }))
}

export function buildMaintenanceActionRows(items: MaintenanceActionDescriptor[]): MaintenanceActionRow[] {
  return items.map((item) => ({
    ...item,
    tone: actionTone(item),
    buttonLabel: item.destructive ? 'Run guarded action' : 'Run action',
    availabilityLabel: item.enabled ? 'Available' : item.disabled_reason ?? 'Unavailable'
  }))
}

export function totalInventoryBytes(items: MaintenanceInventoryItem[]): number {
  return items.reduce((sum, item) => sum + (item.size_bytes ?? 0), 0)
}

export function formatBytes(value: number | null | undefined): string {
  if (value === null || value === undefined) return 'n/a'
  if (!Number.isFinite(value) || value < 0) return 'n/a'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let amount = value
  let unitIndex = 0
  while (amount >= 1024 && unitIndex < units.length - 1) {
    amount /= 1024
    unitIndex += 1
  }
  const precision = Number.isInteger(amount) || amount >= 10 || unitIndex === 0 ? 0 : 1
  return `${amount.toFixed(precision)} ${units[unitIndex]}`
}

function findInventory(
  items: MaintenanceInventoryItem[],
  id: string
): MaintenanceInventoryItem | null {
  return items.find((item) => item.id === id) ?? null
}

function toneFromStatus(status: string | undefined): MaintenanceTone {
  if (status === 'ok') return 'good'
  if (status === 'missing') return 'neutral'
  if (status === 'unavailable') return 'warn'
  return 'neutral'
}

function inventoryIcon(item: MaintenanceInventoryItem): string {
  if (item.kind === 'database') return 'tabler:database'
  if (item.kind === 'logs') return 'tabler:file-analytics'
  if (item.kind === 'backup') return 'tabler:archive'
  if (item.kind === 'vault') return 'tabler:lock'
  return 'tabler:folder'
}

function actionTone(item: MaintenanceActionDescriptor): MaintenanceTone {
  if (!item.enabled) return 'neutral'
  return item.destructive ? 'bad' : 'good'
}

function backupContentsLabel(item: MaintenanceBackupItem): string {
  const parts = []
  if (item.has_database_dump) parts.push('DB')
  if (item.has_vault_snapshot) parts.push('Vault')
  if (item.has_storage_snapshot) parts.push('Storage')
  if (parts.length === 0) return 'Manifest only'
  return parts.join(' + ')
}

function formatDate(value: string | null): string {
  if (!value) return 'n/a'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toISOString().replace(/\.\d{3}Z$/, 'Z')
}
