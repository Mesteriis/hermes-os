<script setup lang="ts">
import type { CommunicationEmailQuotedOriginalModel } from '../communicationDomainElements'
import '../communicationDomainElements.css'
import MailQuotedOriginal from './MailQuotedOriginal.vue'

defineProps<{
  draftPreview: string
  replyOriginal?: CommunicationEmailQuotedOriginalModel
}>()
</script>

<template>
	<div class="communication-email-reply">
		<p class="communication-email-reply__label">Reply draft</p>
		<div
			class="communication-email-reply__editor"
			role="textbox"
			tabindex="0"
			aria-label="Reply draft"
		>
			{{ draftPreview }}
		</div>
		<MailQuotedOriginal
			v-if="replyOriginal"
			class="communication-email-quote--reply"
			:original="replyOriginal"
		/>
	</div>
</template>
