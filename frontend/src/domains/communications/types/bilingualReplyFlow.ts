export const bilingualReplyToneOptions = [
  'formal',
  'business',
  'friendly',
  'short',
  'detailed'
] as const

export type BilingualReplyTone = typeof bilingualReplyToneOptions[number]

export type BilingualReplyFlowRequest = {
  reply_text_ru: string
  tone: BilingualReplyTone
}

export type BilingualOriginal = {
  language: string
  confidence: number
  text: string
}

export type BilingualTranslationStep = {
  target: string
  translated: boolean
  text: string | null
  model: string | null
  reason: string | null
}

export type BilingualReplyDraft = {
  language: 'ru'
  tone: BilingualReplyTone
  text: string
}

export type BilingualReplyFlowResponse = {
  message_id: string
  subject: string
  tone: BilingualReplyTone
  reply_language: 'ru'
  send_ready: boolean
  original: BilingualOriginal
  translation: BilingualTranslationStep
  reply: BilingualReplyDraft
  back_translation: BilingualTranslationStep
}
