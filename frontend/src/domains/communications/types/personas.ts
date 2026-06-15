export type EmailPersona = {
  persona_id: string
  account_id: string
  name: string
  display_name: string
  signature: string
  default_language: string | null
  default_tone: string | null
  is_default: boolean
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}
