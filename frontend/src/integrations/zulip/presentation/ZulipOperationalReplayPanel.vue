<script setup lang="ts">
import type { ZulipOperationalReplayModel } from './zulipOperationalReplayModel'
import './zulipOperationalReplayPanel.css'

defineProps<{ model: ZulipOperationalReplayModel }>()

const emit = defineEmits<{
	loadMore: []
	refresh: []
	selectAccount: [accountId: string]
}>()
</script>

<template>
	<section class="zulip-replay-panel" :aria-busy="model.state === 'loading'">
		<header>
			<div>
				<span>Replayable realtime</span>
				<h2>Operational event stream</h2>
				<p>Monotonic Zulip-owned frames with an explicit retention reset boundary.</p>
			</div>
			<form @submit.prevent="emit('refresh')">
				<label for="zulip-replay-account">Admitted account</label>
				<div>
					<select
						id="zulip-replay-account"
						:value="model.selectedAccountId"
						:disabled="!model.canReplay || model.accounts.length === 0"
						@change="emit('selectAccount', ($event.target as HTMLSelectElement).value)"
					>
						<option v-if="model.accounts.length === 0" value="">No account</option>
						<option v-for="account in model.accounts" :key="account.id" :value="account.id">
							{{ account.label }}
						</option>
					</select>
					<button type="submit" :disabled="!model.canReplay || !model.selectedAccountId || model.state === 'loading'">
						Refresh current window
					</button>
				</div>
			</form>
		</header>

		<p
			v-if="model.statusMessage"
			class="zulip-replay-panel__status"
			:class="{ reset: model.resetRequired }"
			:role="model.state === 'error' ? 'alert' : 'status'"
		>
			{{ model.statusMessage }}
		</p>

		<dl class="zulip-replay-window">
			<div><dt>Earliest</dt><dd>{{ model.earliestSequence }}</dd></div>
			<div><dt>Latest</dt><dd>{{ model.latestSequence }}</dd></div>
			<div><dt>Cursor</dt><dd>{{ model.nextSequence }}</dd></div>
		</dl>

		<ol class="zulip-replay-frames">
			<li v-for="frame in model.frames" :key="frame.sequence">
				<code>#{{ frame.sequence }}</code>
				<strong>{{ frame.kind }}</strong>
				<small>{{ frame.messageId }}</small>
			</li>
		</ol>

		<button
			v-if="model.hasMore"
			type="button"
			class="zulip-replay-more"
			:disabled="model.state === 'loading'"
			@click="emit('loadMore')"
		>
			Load more frames
		</button>
	</section>
</template>
