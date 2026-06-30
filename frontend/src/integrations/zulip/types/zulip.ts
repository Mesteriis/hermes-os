export interface ZulipCredentialBinding {
  secret_purpose: 'zulip_api_key' | string
  secret_ref: string
  secret_kind: 'api_token' | string
  store_kind: 'host_vault' | string
}

export interface ZulipAccountSetupRequest {
  account_id: string
  display_name: string
  external_account_id: string
  base_url: string
  api_key: string
}

export interface ZulipAccountSetupResponse {
  account_id: string
  provider_kind: 'zulip_bot'
  display_name: string
  external_account_id: string
  base_url: string
  credential_binding: ZulipCredentialBinding
}

export interface ZulipUploadAttachmentRef {
  attachment_id?: string
  blob_id?: string
  filename?: string
}

export interface ZulipStreamUploadCommandRequest extends ZulipUploadAttachmentRef {
  command_id?: string
  idempotency_key?: string
  stream: string
  topic: string
  content: string
  actor_id?: string
}

export interface ZulipDirectUploadCommandRequest extends ZulipUploadAttachmentRef {
  command_id?: string
  idempotency_key?: string
  recipients: string[]
  content: string
  actor_id?: string
}

export interface ZulipUploadCommandRequest extends ZulipUploadAttachmentRef {
  command_id?: string
  idempotency_key?: string
  actor_id?: string
}

export interface ZulipCommandEnqueueResponse {
  command_id: string
  account_id: string
  channel_kind: 'zulip'
  command_kind: 'upload_file' | 'send_stream_message_with_upload' | 'send_direct_message_with_upload' | string
  idempotency_key: string
  status: string
  reconciliation_status: string
  provider_conversation_id?: string | null
  payload: Record<string, unknown>
}
