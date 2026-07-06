<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '@/platform/i18n'
import { HtmlPreview, IconButton, ToggleGroup } from '@/shared/ui'
import type { CommunicationConversationMessageModel } from '../communicationDomainElements'
import { htmlToComposePlainText } from '../richComposeHtml'
import '../communicationDomainElements.css'
import MailQuotedOriginal from './MailQuotedOriginal.vue'

const { t } = useI18n()
type MailViewerBodyMode = 'translation' | 'clean' | 'original' | 'plain'
const bodyMode = ref<MailViewerBodyMode>('clean')
const recipientsOpen = ref(false)

const props = defineProps<{
  message: CommunicationConversationMessageModel
  fallbackSubject: string
}>()

type RecipientDetailRow = {
  id: string
  label: string
  value: string
}

type ContextSummaryItem = {
  id: string
  label: string
  tone?: string
}

const recipientDetailRows = computed<RecipientDetailRow[]>(() => {
  const rows: RecipientDetailRow[] = []

  if (props.message.ccLabel) rows.push({ id: 'cc', label: t('CC'), value: props.message.ccLabel })
  if (props.message.bccLabel) rows.push({ id: 'bcc', label: t('BCC'), value: props.message.bccLabel })
  if (props.message.replyToLabel) {
    rows.push({ id: 'reply-to', label: t('Reply to'), value: props.message.replyToLabel })
  }

  return rows
})
const senderLabel = computed(() => props.message.fromLabel ?? props.message.author)
const recipientLabel = computed(() => props.message.toLabel ?? 'Owner')
const hasTranslation = computed(() => Boolean(props.message.translation?.text.trim()))
const bodyModeItems = computed(() => [
  ...(hasTranslation.value
    ? [{ value: 'translation', label: t('Translation'), icon: 'tabler:language' }]
    : []),
  { value: 'clean', label: t('Clean'), icon: 'tabler:sparkles' },
  { value: 'original', label: t('Original HTML'), icon: 'tabler:code' },
  { value: 'plain', label: t('Plain text'), icon: 'tabler:file-text' }
])
const translationMeta = computed(() => {
  if (!props.message.translation) return ''
  const parts = [t('Translation'), props.message.translation.target]
  if (props.message.translation.model) parts.push(props.message.translation.model)

  return parts.join(' · ')
})
const bodyPreviewContent = computed(() => {
  if (bodyMode.value === 'translation' && props.message.translation) {
    return props.message.translation.text
  }

  if (bodyMode.value === 'plain') {
    return props.message.bodyFormat === 'html' && props.message.bodyHtml
      ? htmlToComposePlainText(props.message.bodyHtml)
      : props.message.body
  }

  return props.message.bodyFormat === 'html'
    ? (props.message.bodyHtml ?? props.message.body)
    : props.message.body
})
const bodyPreviewFormat = computed(() => bodyMode.value === 'plain' || bodyMode.value === 'translation'
  ? 'text'
  : props.message.bodyFormat === 'html' ? 'html' : 'text')
const bodyPreviewSanitized = computed(() => bodyPreviewFormat.value === 'html' && props.message.bodyHtmlSanitized === true)
const bodyPreviewClass = computed(() => [
  'communication-email-message__body-preview',
  `communication-email-message__body-preview--${bodyMode.value}`
].join(' '))
const contextSummaryItems = computed<ContextSummaryItem[]>(() => {
  const evidenceItems = props.message.evidenceItems ?? []
  const evidenceTone = evidenceItems.some((item) => item.tone === 'danger')
    ? 'danger'
    : evidenceItems.some((item) => item.tone === 'warning')
      ? 'warning'
      : 'info'
  const items: ContextSummaryItem[] = []

  for (const label of props.message.labels ?? []) {
    items.push({ id: `label-${label}`, label })
  }

  for (const marker of props.message.markers ?? []) {
    items.push({
      id: marker.id,
      label: summaryMarkerValue(marker.value),
      tone: marker.tone
    })
  }

  if (evidenceItems.length > 0) {
    items.push({
      id: 'evidence-count',
      label: t('{count} evidence', { count: evidenceItems.length }),
      tone: evidenceTone
    })
  }

  return items
})
const hasContext = computed(() => contextSummaryItems.value.length > 0)
const hasRecipientDetails = computed(() => recipientDetailRows.value.length > 0)

watch(() => [props.message.id, props.message.translation?.text] as const, () => {
  if (hasTranslation.value) {
    bodyMode.value = 'translation'
    return
  }

  bodyMode.value = props.message.bodyHtml ? 'original' : 'clean'
}, { immediate: true })

function setBodyMode(value: string | string[]): void {
  if (
    value === 'translation' ||
    value === 'clean' ||
    value === 'original' ||
    value === 'plain'
  ) {
    bodyMode.value = value
  }
}

function summaryMarkerValue(value: string | number): string {
  return typeof value === 'number' ? String(value) : t(value)
}

function toggleRecipients(): void {
  recipientsOpen.value = !recipientsOpen.value
}
</script>

<template>
	<section class="communication-email-viewer" :aria-label="t('Message body')">
		<header class="communication-email-envelope">
			<div class="communication-email-envelope__main">
				<div class="communication-email-message__title-block">
					<h3 class="communication-email-message__subject">{{ message.subject ?? fallbackSubject }}</h3>
					<div class="communication-email-envelope__route" :aria-label="t('Participants')">
						<span>{{ t('From') }} <strong>{{ senderLabel }}</strong></span>
						<span>{{ t('To') }} <strong>{{ recipientLabel }}</strong></span>
						<IconButton
							v-if="hasRecipientDetails"
							class="communication-email-envelope__recipient-toggle"
							:icon="recipientsOpen ? 'tabler:chevron-up' : 'tabler:chevron-down'"
							:label="recipientsOpen ? t('Hide recipient details') : t('Show recipient details')"
							size="sm"
							variant="ghost"
							@click="toggleRecipients"
						/>
					</div>
					<dl
						v-if="recipientsOpen && hasRecipientDetails"
						class="communication-email-envelope__recipient-details"
						:aria-label="t('Recipient details')"
					>
						<div
							v-for="row in recipientDetailRows"
							:key="row.id"
							class="communication-email-envelope__recipient-detail"
						>
							<dt>{{ row.label }}</dt>
							<dd>{{ row.value }}</dd>
						</div>
					</dl>
				</div>

				<div class="communication-email-envelope__tools">
					<span class="communication-email-message__time">{{ message.timestamp }}</span>
				</div>
			</div>

			<section
				v-if="hasContext"
				class="communication-email-envelope__context"
				:aria-label="t('Message markers')"
			>
				<div v-if="contextSummaryItems.length" class="communication-email-message__signals">
					<span
						v-for="item in contextSummaryItems"
						:key="item.id"
						:class="[
							'communication-email-message__summary-item',
							item.tone && `communication-email-message__summary-item--${item.tone}`
						]"
					>
						{{ item.label }}
					</span>
				</div>
			</section>
		</header>

		<div class="communication-email-viewer__mode-divider">
			<ToggleGroup
				class="communication-email-viewer__mode-toggle"
				:model-value="bodyMode"
				:aria-label="t('Body mode')"
				:items="bodyModeItems"
				@update:model-value="setBodyMode"
			/>
		</div>

		<div class="communication-email-center__body-scroll">
			<article class="communication-email-center__paper">
				<div
					v-if="bodyMode === 'translation' && translationMeta"
					class="communication-email-message__translation-meta"
				>
					{{ translationMeta }}
				</div>
				<HtmlPreview
					:class="bodyPreviewClass"
					:content="bodyPreviewContent"
					:format="bodyPreviewFormat"
					:sanitized="bodyPreviewSanitized"
					:unsafe-label="t('HTML preview requires sanitized content')"
					:empty-label="t('No message body')"
				/>

				<MailQuotedOriginal v-if="message.quotedOriginal" :original="message.quotedOriginal" />
			</article>
		</div>
	</section>
</template>
