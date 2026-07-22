<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '@/platform/i18n'
import { HtmlPreview, IconButton, ToggleGroup } from '@/shared/ui'
import type { CommunicationConversationMessageModel } from '../communicationDomainElements'
import '../communicationDomainElements.css'
import MailQuotedOriginal from './MailQuotedOriginal.vue'
import {
  mailViewerBodyModeItems,
  mailViewerBodyPreviewContent,
  mailViewerBodyPreviewClass,
  mailViewerBodyPreviewFormat,
  mailViewerBodyPreviewIsSanitized,
  mailViewerContextSummaryItems,
  mailViewerHasTranslation,
  mailViewerInitialBodyMode,
  isMailViewerBodyMode,
  mailViewerRecipientDetailRows,
  mailViewerRecipientLabel,
  mailViewerSenderLabel,
  mailViewerTranslationMeta,
  type ContextSummaryItem,
  type MailViewerBodyMode,
  type RecipientDetailRow,
} from './mailViewerPresentation'

const { t } = useI18n()
const bodyMode = ref<MailViewerBodyMode>('clean')
const recipientsOpen = ref(false)

const props = defineProps<{
  message: CommunicationConversationMessageModel
  fallbackSubject: string
}>()

const recipientDetailRows = computed<RecipientDetailRow[]>(() => mailViewerRecipientDetailRows(props.message, t))
const senderLabel = computed(() => mailViewerSenderLabel(props.message))
const recipientLabel = computed(() => mailViewerRecipientLabel(props.message))
const hasTranslation = computed(() => mailViewerHasTranslation(props.message))
const bodyModeItems = computed(() => mailViewerBodyModeItems(props.message, t))
const translationMeta = computed(() => mailViewerTranslationMeta(props.message, t))
const bodyPreviewContent = computed(() => mailViewerBodyPreviewContent(props.message, bodyMode.value))
const bodyPreviewFormat = computed(() => mailViewerBodyPreviewFormat(props.message, bodyMode.value))
const bodyPreviewSanitized = computed(() => mailViewerBodyPreviewIsSanitized(props.message, bodyMode.value))
const bodyPreviewClass = computed(() => mailViewerBodyPreviewClass(bodyMode.value))
const contextSummaryItems = computed<ContextSummaryItem[]>(() => mailViewerContextSummaryItems(props.message, t))
const hasContext = computed(() => contextSummaryItems.value.length > 0)
const hasRecipientDetails = computed(() => recipientDetailRows.value.length > 0)

watch(() => [props.message.id, props.message.translation?.text] as const, () => {
  bodyMode.value = mailViewerInitialBodyMode(props.message)
}, { immediate: true })

function setBodyMode(value: string | string[]): void {
  if (isMailViewerBodyMode(value)) bodyMode.value = value
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
					:isolated="bodyPreviewFormat === 'html'"
					:unsafe-label="t('HTML preview requires sanitized content')"
					:empty-label="t('No message body')"
				/>

				<MailQuotedOriginal v-if="message.quotedOriginal" :original="message.quotedOriginal" />
			</article>
		</div>
	</section>
</template>
