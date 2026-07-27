<script setup lang="ts">
import type { WhatsAppOperationalReplayModel } from './whatsAppOperationalReplayModel'
import './whatsAppOperationalReplayPanel.css'

defineProps<{ model: WhatsAppOperationalReplayModel }>()

const emit = defineEmits<{
	loadMore: []
	refresh: []
	selectAccount: [accountId: string]
}>()
</script>

<template>
	<section class="whatsapp-replay-panel" :aria-busy="model.state === 'loading'">
		<header>
			<div>
				<span>Replayable realtime</span>
				<h2>Operational event stream</h2>
				<p>Bounded WhatsApp-owned frames with an explicit retention reset boundary.</p>
			</div>
			<form @submit.prevent="emit('refresh')">
				<label for="whatsapp-replay-account">Admitted account</label>
				<div>
					<select
						id="whatsapp-replay-account"
						:value="model.selectedAccountId"
						:disabled="!model.canReplay || model.accounts.length === 0"
						@change="emit('selectAccount', ($event.target as HTMLSelectElement).value)"
					>
						<option v-if="model.accounts.length === 0" value="">No account</option>
						<option v-for="account in model.accounts" :key="account.id" :value="account.id">
							{{ account.label }}
						</option>
					</select>
					<button
						type="submit"
						:disabled="!model.canReplay || !model.selectedAccountId || model.state === 'loading'"
					>
						Refresh from current window
					</button>
				</div>
			</form>
		</header>

		<p
			v-if="model.statusMessage"
			class="whatsapp-replay-panel__status"
			:class="{ reset: model.resetRequired }"
			:role="model.state === 'error' ? 'alert' : 'status'"
		>
			{{ model.statusMessage }}
		</p>

		<dl class="whatsapp-replay-window">
			<div><dt>Earliest</dt><dd>{{ model.earliestSequence }}</dd></div>
			<div><dt>Latest</dt><dd>{{ model.latestSequence }}</dd></div>
			<div><dt>Cursor</dt><dd>{{ model.nextSequence }}</dd></div>
		</dl>

		<ol class="whatsapp-replay-frames">
			<li v-for="frame in model.frames" :key="frame.sequence">
				<code>#{{ frame.sequence }}</code>
				<strong>{{ frame.kind }}</strong>
			</li>
		</ol>

		<button
			v-if="model.hasMore"
			type="button"
			class="whatsapp-replay-more"
			:disabled="model.state === 'loading'"
			@click="emit('loadMore')"
		>
			Load more frames
		</button>
	</section>
</template>
