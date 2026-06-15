export type ThreadTranslationItem = {
  message_id: string
  original_language: string
  confidence: number
  translated: boolean
  text: string | null
  target: string
  model: string | null
  reason: string | null
}

export type ThreadTranslationResponse = {
  account_id: string
  subject: string
  target_language: string
  items: ThreadTranslationItem[]
}
