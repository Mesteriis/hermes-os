import { describe, expect, it } from 'vitest'
import {
  buildMaintenanceActionRows,
  buildMaintenanceBackupRows,
  buildMaintenanceInventoryRows,
  buildMaintenanceSummaryTiles,
  formatBytes,
  totalInventoryBytes
} from './maintenanceSettingsPresentation'
import type {
  MaintenanceActionDescriptor,
  MaintenanceBackupItem,
  MaintenanceInventoryItem
} from '../types/maintenance'

describe('maintenanceSettingsPresentation', () => {
  it('formats inventory, backups and guarded actions for settings UI', () => {
    const inventory = sampleInventory()
    const backups = sampleBackups()
    const actions = sampleActions()

    expect(formatBytes(1536)).toBe('1.5 KB')
    expect(totalInventoryBytes(inventory)).toBe(3072)
    expect(buildMaintenanceSummaryTiles(inventory, backups).map((tile) => tile.id)).toEqual([
      'database',
      'mail',
      'logs',
      'backups'
    ])
    expect(buildMaintenanceInventoryRows(inventory)[0]).toMatchObject({
      id: 'database',
      sizeLabel: '2 KB',
      tone: 'good',
      icon: 'tabler:database'
    })
    expect(buildMaintenanceBackupRows(backups)[0]).toMatchObject({
      contentsLabel: 'DB + Vault + Storage',
      sizeLabel: '4 KB'
    })
    expect(buildMaintenanceActionRows(actions)[1]).toMatchObject({
      id: 'restore_database',
      tone: 'neutral',
      availabilityLabel: 'Stop Hermes and run make vault-restore'
    })
  })
})

function sampleInventory(): MaintenanceInventoryItem[] {
  return [
    {
      id: 'database',
      label: 'PostgreSQL database',
      description: 'DB',
      kind: 'database',
      path_label: 'configured database',
      exists: true,
      size_bytes: 2048,
      file_count: null,
      status: 'ok',
      detail: 'pg_database_size(current_database())'
    },
    {
      id: 'mail_blobs',
      label: 'Mail blob store',
      description: 'Mail',
      kind: 'storage',
      path_label: 'docker/data/mail',
      exists: true,
      size_bytes: 1024,
      file_count: 2,
      status: 'ok',
      detail: 'inspected'
    }
  ]
}

function sampleBackups(): MaintenanceBackupItem[] {
  return [
    {
      id: 'backups/2026-07-08/20260708T090000Z',
      label: '20260708T090000Z',
      created_at: '2026-07-08T09:00:00Z',
      path_label: 'backups/2026-07-08/20260708T090000Z',
      size_bytes: 4096,
      file_count: 3,
      has_database_dump: true,
      has_vault_snapshot: true,
      has_storage_snapshot: true,
      manifest_present: true
    }
  ]
}

function sampleActions(): MaintenanceActionDescriptor[] {
  return [
    {
      id: 'backup_database',
      label: 'Backup DB + vault',
      description: 'Backup',
      icon: 'tabler:database-export',
      destructive: false,
      enabled: true,
      requires_confirmation: true,
      confirmation_phrase: 'BACKUP DB',
      disabled_reason: null
    },
    {
      id: 'restore_database',
      label: 'Restore DB + vault',
      description: 'Restore',
      icon: 'tabler:database-import',
      destructive: true,
      enabled: false,
      requires_confirmation: true,
      confirmation_phrase: 'RESTORE',
      disabled_reason: 'Stop Hermes and run make vault-restore'
    }
  ]
}
