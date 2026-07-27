<script setup lang="ts">
import type { ZulipMessageRow } from './zulipOperationalReadModel'

defineProps<{ message: ZulipMessageRow }>()
</script>

<template>
	<article class="zulip-message-row" :class="{ deleted: message.deleted }">
		<header><strong>{{ message.sender }}</strong><small>{{ message.meta }}</small></header>
		<p>{{ message.content }}</p>
		<small v-if="message.attachments">Attachments: {{ message.attachments }}</small>
		<small v-if="message.reactions">Reactions: {{ message.reactions }}</small>
	</article>
</template>
