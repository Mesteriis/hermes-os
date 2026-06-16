export type TelegramAutomationTemplate = {
  template_id: string
  name: string
  body_template: string
  required_variables: string[]
  created_at: string
  updated_at: string
}

export type TelegramAutomationPolicy = {
  policy_id: string
  template_id: string
  name: string
  enabled: boolean
  account_id: string
  allowed_chat_ids: string[]
  trigger_kind: string
  max_sends_per_hour: number
  quiet_hours: unknown
  expires_at: string | null
  conditions: unknown
  created_at: string
  updated_at: string
}

export type TelegramAutomationTemplateListResponse = {
  items: TelegramAutomationTemplate[]
}

export type TelegramAutomationPolicyListResponse = {
  items: TelegramAutomationPolicy[]
}

export type TelegramSendDryRunRequest = {
  command_id: string
  policy_id: string
  provider_chat_id: string
  variables: Record<string, string>
  source_context?: Record<string, string>
}

export type TelegramSendDryRunResponse = {
  outbound_message_id: string
  policy_id: string
  template_id: string
  account_id: string
  provider_chat_id: string
  rendered_text: string
  rendered_preview_hash: string
  status: string
  event_id: string
}
