import { getCurrentScope, onScopeDispose } from 'vue'
import type { ComposeFormModel } from '../types/communications'
import { splitComposeRecipients } from './composeValidation'

const DEFAULT_AUTOSAVE_DELAY_MS = 2000

export type ComposeDraftPayload = {
	draft_id: string
	account_id: string
	to_recipients: string[]
	cc_recipients: string[]
	bcc_recipients: string[]
	subject: string
	body_text: string
	body_html: string | null
	in_reply_to: string | null
	attachment_ids: string[]
	scheduled_send_at: string | null
	status: 'draft'
	metadata: {
		compose_mode: ComposeFormModel['mode']
	}
}

export type ComposeDraftAutosaveOptions = {
	delayMs?: number
	formSource: () => ComposeFormModel
	saveDraft: (payload: ComposeDraftPayload) => Promise<unknown>
	onSaved?: () => void
	onError?: (error: unknown) => void
}

export function buildComposeDraftPayload(form: ComposeFormModel): ComposeDraftPayload {
	return {
		draft_id: form.draftId,
		account_id: form.accountId,
		to_recipients: splitComposeRecipients(form.toText),
		cc_recipients: splitComposeRecipients(form.ccText),
		bcc_recipients: splitComposeRecipients(form.bccText),
		subject: form.subject,
		body_text: form.body,
		body_html: form.bodyFormat === 'html' ? form.bodyHtml : null,
		in_reply_to: form.inReplyTo,
		attachment_ids: form.attachments
			.filter((attachment) =>
				attachment.attachmentId &&
				attachment.uploadStatus !== 'uploading' &&
				attachment.uploadStatus !== 'failed'
			)
			.map((attachment) => attachment.attachmentId),
		scheduled_send_at: datetimeLocalToIso(form.scheduledSendAt),
		status: 'draft',
		metadata: { compose_mode: form.mode }
	}
}

export function composeDraftHasAutosaveContent(form: ComposeFormModel): boolean {
	return Boolean(
		form.toText.trim() ||
			form.ccText.trim() ||
			form.bccText.trim() ||
			form.subject.trim() ||
			form.body.trim() ||
			Boolean(form.bodyHtml?.trim()) ||
			form.attachments.length > 0 ||
			Boolean(form.scheduledSendAt.trim())
	)
}

export function datetimeLocalToIso(value: string): string | null {
	const trimmed = value.trim()
	if (!trimmed) return null
	const date = new Date(trimmed)
	return Number.isFinite(date.getTime()) ? date.toISOString() : null
}

export function useComposeDraftAutosave(options: ComposeDraftAutosaveOptions) {
	let timer: ReturnType<typeof setTimeout> | null = null
	const delayMs = options.delayMs ?? DEFAULT_AUTOSAVE_DELAY_MS

	function clearTimer(): void {
		if (!timer) return
		clearTimeout(timer)
		timer = null
	}

	function canSave(): boolean {
		const form = options.formSource()
		return Boolean(
			form.draftId.trim() &&
				form.accountId.trim() &&
				composeDraftHasAutosaveContent(form)
		)
	}

	async function saveNow(): Promise<void> {
		if (!canSave()) return

		try {
			await options.saveDraft(buildComposeDraftPayload(options.formSource()))
			options.onSaved?.()
		} catch (error) {
			options.onError?.(error)
		}
	}

	function schedule(): void {
		clearTimer()
		if (!canSave()) return
		timer = setTimeout(() => {
			void saveNow()
		}, delayMs)
	}

	async function flush(): Promise<void> {
		clearTimer()
		await saveNow()
	}

	function cancel(): void {
		clearTimer()
	}

	if (getCurrentScope()) {
		onScopeDispose(cancel)
	}

	return {
		cancel,
		flush,
		saveNow,
		schedule
	}
}
