<script setup lang="ts">
import type { MailSyncHealthModel } from './mailSyncHealthModel'
import './mailSyncHealthPanel.css'

defineProps<{ model: MailSyncHealthModel }>()

const emit = defineEmits<{
	loadMore: []
	refresh: []
	selectConnection: [connectionId: string]
}>()
</script>

<template>
	<section class="mail-sync-health" aria-label="Mail sync health">
		<header class="mail-sync-health__header">
			<div>
				<span>Persisted provider evidence</span>
				<h2>Sync health</h2>
				<p>Restart-safe Mail-owned status and bounded run history for the selected account.</p>
			</div>
			<div class="mail-sync-health__actions">
				<label v-if="model.connections.length > 1">
					Connection
					<select
						:value="model.selectedConnectionId"
						:disabled="model.state === 'loading'"
						@change="emit('selectConnection', ($event.target as HTMLSelectElement).value)"
					>
						<option
							v-for="connection in model.connections"
							:key="connection.id"
							:value="connection.id"
						>
							{{ connection.label }}
						</option>
					</select>
				</label>
				<button
					type="button"
					:disabled="!model.canQuery || !model.selectedConnectionId || model.state === 'loading'"
					@click="emit('refresh')"
				>
					{{ model.state === 'loading' ? 'Loading…' : 'Refresh health' }}
				</button>
			</div>
		</header>

		<p
			v-if="model.statusMessage"
			class="mail-sync-health__notice"
			:class="{ error: model.state === 'error' }"
			role="status"
		>
			{{ model.statusMessage }}
		</p>

		<div v-if="model.state === 'ready' || model.runs.length > 0" class="mail-sync-health__content">
			<div class="mail-sync-health__summary">
				<article>
					<span>Provider path</span>
					<strong :class="`tone-${model.readinessTone}`">{{ model.readiness }}</strong>
				</article>
				<article>
					<span>Latest outcome</span>
					<strong :class="`tone-${model.latestOutcomeTone}`">{{ model.latestOutcome }}</strong>
				</article>
				<article>
					<span>Last success</span>
					<strong>{{ model.lastSuccessAt }}</strong>
				</article>
				<article>
					<span>Consecutive failures</span>
					<strong>{{ model.consecutiveFailures }}</strong>
				</article>
				<article>
					<span>Projection revision</span>
					<strong>{{ model.projectionRevision }}</strong>
				</article>
			</div>

			<section class="mail-sync-health__history">
				<header>
					<div>
						<span>Terminal and interrupted evidence</span>
						<h3>Run history</h3>
					</div>
					<small>{{ model.runs.length }} loaded</small>
				</header>
				<p v-if="model.runs.length === 0" class="mail-sync-health__empty">
					No persisted sync runs yet.
				</p>
				<ol v-else>
					<li v-for="run in model.runs" :key="run.operationId">
						<div class="mail-sync-health__run-heading">
							<div>
								<strong>{{ run.trigger }}</strong>
								<code>{{ run.operationId }}</code>
							</div>
							<span :class="`tone-${run.outcomeTone}`">{{ run.outcome }}</span>
						</div>
						<dl>
							<div><dt>Started</dt><dd>{{ run.startedAt }}</dd></div>
							<div><dt>Completed</dt><dd>{{ run.completedAt }}</dd></div>
							<div><dt>Observed</dt><dd>{{ run.observedMessages }} messages</dd></div>
							<div><dt>Failure</dt><dd>{{ run.failure }}</dd></div>
							<div><dt>Runtime generation</dt><dd>{{ run.runtimeGeneration }}</dd></div>
							<div><dt>Revision</dt><dd>{{ run.projectionRevision }}</dd></div>
						</dl>
					</li>
				</ol>
				<button
					v-if="model.hasMoreRuns"
					type="button"
					class="mail-sync-health__more"
					:disabled="model.state === 'loading'"
					@click="emit('loadMore')"
				>
					Load more runs
				</button>
			</section>
		</div>
	</section>
</template>
