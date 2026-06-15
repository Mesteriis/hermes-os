import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import {
  bilingualReplyToneOptions,
  type BilingualReplyFlowRequest,
  type BilingualReplyTone
} from '../types/bilingualReplyFlow'

export { bilingualReplyToneOptions }

export const bilingualReplyFlowFormSchema = z.object({
  replyTextRu: z
    .string()
    .trim()
    .min(1, 'Russian reply is required')
    .max(64_000, 'Russian reply is too long'),
  tone: z.enum(bilingualReplyToneOptions)
})

export type BilingualReplyFlowFormValues = z.infer<typeof bilingualReplyFlowFormSchema>

export const bilingualReplyFlowVeeValidationSchema = toTypedSchema(bilingualReplyFlowFormSchema)

export function bilingualReplyFlowFormDefaults(): BilingualReplyFlowFormValues {
  return {
    replyTextRu: '',
    tone: 'business'
  }
}

export function bilingualReplyFlowFormToRequest(
  values: BilingualReplyFlowFormValues
): BilingualReplyFlowRequest {
  const parsed = bilingualReplyFlowFormSchema.parse(values)
  return {
    reply_text_ru: parsed.replyTextRu,
    tone: parsed.tone as BilingualReplyTone
  }
}
