import { watch } from 'vue'
import { useForm } from 'vee-validate'
import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type { ComposeFormModel } from '../types/communications'

const MAX_EMAIL_SUBJECT_LENGTH = 998
const MAX_EMAIL_BODY_LENGTH = 1_000_000
const EMAIL_ADDRESS_PATTERN = /^[^\s@<>]+@[^\s@<>]+\.[^\s@<>]+$/

export type ComposeValidationValues = {
  accountId: string
  toText: string
  ccText: string
  bccText: string
  subject: string
  body: string
  inReplyTo: string | null
}

export function splitComposeRecipients(value: string): string[] {
  return value
    .split(',')
    .map((recipient) => normalizeRecipientAddress(recipient))
    .filter(Boolean)
}

export function toComposeValidationValues(form: ComposeFormModel): ComposeValidationValues {
  return {
    accountId: form.accountId,
    toText: form.toText,
    ccText: form.ccText,
    bccText: form.bccText,
    subject: form.subject,
    body: form.body,
    inReplyTo: form.inReplyTo
  }
}

const requiredRecipientsSchema = z
  .string()
  .trim()
  .refine((value) => splitComposeRecipients(value).length > 0, {
    message: 'At least one recipient is required'
  })
  .refine((value) => recipientsAreValid(value), {
    message: 'Recipient list contains an invalid email address'
  })

const optionalRecipientsSchema = z
  .string()
  .trim()
  .refine((value) => value === '' || recipientsAreValid(value), {
    message: 'Recipient list contains an invalid email address'
  })

export const composeSendSchema = z.object({
  accountId: z.string().trim().min(1, 'Mail account is required'),
  toText: requiredRecipientsSchema,
  ccText: optionalRecipientsSchema,
  bccText: optionalRecipientsSchema,
  subject: z.string().max(MAX_EMAIL_SUBJECT_LENGTH, 'Subject is too long'),
  body: z.string().max(MAX_EMAIL_BODY_LENGTH, 'Message body is too long'),
  inReplyTo: z.string().nullable()
})

export const composeVeeValidationSchema = toTypedSchema(composeSendSchema)

export function useComposeValidation(formSource: () => ComposeFormModel) {
  const { errors, setValues, validate } = useForm<ComposeValidationValues>({
    validationSchema: composeVeeValidationSchema,
    initialValues: toComposeValidationValues(formSource()),
    validateOnMount: false
  })

  watch(
    () => toComposeValidationValues(formSource()),
    (values) => setValues(values, false),
    { deep: true }
  )

  async function validateForSend(): Promise<boolean> {
    const result = await validate()
    return result.valid
  }

  return {
    errors,
    validateForSend
  }
}

function recipientsAreValid(value: string): boolean {
  const recipients = splitComposeRecipients(value)
  return recipients.length > 0 && recipients.every((recipient) => EMAIL_ADDRESS_PATTERN.test(recipient))
}

function normalizeRecipientAddress(value: string): string {
  const trimmed = value.trim()
  const angleAddress = trimmed.match(/<([^<>]+)>$/)
  return (angleAddress?.[1] ?? trimmed).trim()
}
