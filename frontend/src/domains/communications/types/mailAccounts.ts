export type EmailAccountCapabilities = {
  read: boolean
  sync: boolean
  send: boolean
  oauth: boolean
  imap: boolean
  smtp: boolean
  mutate_flags: boolean
  mutate_mailboxes: boolean
  server_delete: boolean
  provider_folders: boolean
  local_trash: boolean
}

export type EmailProviderAccount = {
  account_id: string
  provider_kind: string
  display_name: string
  external_account_id: string
  config: Record<string, unknown>
  created_at: string
  updated_at: string
  email?: string | null
  label?: string | null
  is_active?: boolean
  is_authenticated?: boolean
  last_sync_at?: string | null
  deleted_at?: string | null
}

export type EmailAccountView = {
  account: EmailProviderAccount
  capabilities: EmailAccountCapabilities
}

export type EmailAccountListResponse = {
  items: EmailAccountView[]
}
