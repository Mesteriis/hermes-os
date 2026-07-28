<script setup lang="ts">
import type { CanonicalCommunicationContentModel } from './canonicalCommunicationContentModel'
import './canonicalCommunicationContent.css'

defineProps<{ model: CanonicalCommunicationContentModel }>()
</script>

<template>
	<section
		v-if="model.status !== 'idle'"
		class="canonical-communication-content"
		:aria-busy="model.status === 'loading'"
		aria-labelledby="canonical-communication-content-title"
	>
		<header>
			<div>
				<small>Canonical content</small>
				<h3 id="canonical-communication-content-title">Message body</h3>
			</div>
			<strong>{{ model.status }}</strong>
		</header>
		<p v-if="model.statusMessage" role="status">{{ model.statusMessage }}</p>
		<pre v-if="model.status === 'ready'">{{ model.bodyText }}</pre>
	</section>
</template>
