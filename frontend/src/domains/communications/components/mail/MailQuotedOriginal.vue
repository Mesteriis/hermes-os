<script setup lang="ts">
import type { CommunicationEmailQuotedOriginalModel } from '../communicationDomainElements'
import '../communicationDomainElements.css'

defineProps<{
  original: CommunicationEmailQuotedOriginalModel
}>()
</script>

<template>
	<section class="communication-email-quote">
		<h4 class="communication-email-quote__title">Original message</h4>
		<p class="communication-email-quote__meta">
			{{ original.author }} · {{ original.timestamp }} · {{ original.subject }}
		</p>
		<blockquote class="communication-email-quote__body">{{ original.body }}</blockquote>
	</section>
</template>
