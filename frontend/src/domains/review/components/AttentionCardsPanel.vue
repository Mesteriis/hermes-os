<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type {
	AttentionCard,
	AttentionImportance,
	AttentionRelatedEntity,
	AttentionSuggestedAction
} from '../types/review'

defineProps<{
	cards: AttentionCard[]
}>()

const importanceLabels: Record<AttentionImportance, string> = {
	low: 'Low',
	medium: 'Medium',
	high: 'High',
	critical: 'Critical'
}

const importanceIcons: Record<AttentionImportance, string> = {
	low: 'tabler:circle',
	medium: 'tabler:circle-half-2',
	high: 'tabler:alert-triangle',
	critical: 'tabler:alert-octagon'
}

function confidencePercent(value: number): string {
	return `${Math.round(value * 100)}%`
}

function relatedEntityLabel(entity: AttentionRelatedEntity): string {
	const entityRef = `${entity.entity_kind}:${entity.entity_id}`
	if (!entity.label?.trim()) return entityRef
	return `${entity.label}: ${entityRef}`
}

function suggestedActionLabel(action: AttentionSuggestedAction): string {
	if (!action.target_domain || !action.target_entity_kind) return action.label
	return `${action.label} -> ${action.target_domain}/${action.target_entity_kind}`
}

function shortId(value: string): string {
	if (value.length <= 24) return value
	return `${value.slice(0, 10)}...${value.slice(-8)}`
}
</script>

<template>
	<section class="attention-panel" aria-labelledby="attention-panel-title">
		<h3 id="attention-panel-title" class="panel-title">
			<Icon icon="tabler:radar-2" />
			Attention Cards
			<span v-if="cards.length > 0" class="panel-badge">
				{{ cards.length }}
			</span>
		</h3>

		<div v-if="cards.length === 0" class="attention-empty">
			<p>No active attention cards</p>
		</div>

		<div v-else class="attention-grid">
			<article
				v-for="card in cards"
				:key="card.id"
				class="attention-card"
				:class="`attention-card--${card.importance}`"
			>
				<header class="attention-card-header">
					<div class="attention-title-group">
						<p class="attention-title">{{ card.title }}</p>
						<p class="attention-summary">{{ card.summary }}</p>
					</div>
					<span class="importance-pill" :class="`importance-pill--${card.importance}`">
						<Icon :icon="importanceIcons[card.importance]" />
						{{ importanceLabels[card.importance] }}
					</span>
				</header>

				<div class="attention-stats" aria-label="Attention evidence summary">
					<span>
						<strong>{{ confidencePercent(card.confidence) }}</strong>
						confidence
					</span>
					<span>
						<strong>{{ card.evidence_count }}</strong>
						evidence
					</span>
					<span>
						<strong>{{ card.review_item_ids.length }}</strong>
						review
					</span>
				</div>

				<div class="attention-section">
					<p class="section-label">Why this matters</p>
					<p class="section-text">{{ card.explainability.why_this_matters }}</p>
				</div>

				<div class="attention-section">
					<p class="section-label">Source</p>
					<p class="section-text">{{ card.source_summary }}</p>
				</div>

				<div class="attention-columns">
					<div class="attention-section">
						<p class="section-label">Evidence</p>
						<ul class="compact-list">
							<li
								v-for="evidence in card.explainability.evidence"
								:key="`${evidence.observation_id}:${evidence.role}`"
							>
								<span>{{ evidence.role }}</span>
								<code :title="evidence.observation_id">{{ shortId(evidence.observation_id) }}</code>
							</li>
						</ul>
					</div>

					<div class="attention-section">
						<p class="section-label">Related objects</p>
						<ul v-if="card.explainability.related_objects.length > 0" class="compact-list">
							<li
								v-for="entity in card.explainability.related_objects"
								:key="`${entity.entity_kind}:${entity.entity_id}`"
							>
								{{ relatedEntityLabel(entity) }}
							</li>
						</ul>
						<p v-else class="section-muted">None</p>
					</div>
				</div>

				<div class="attention-section">
					<p class="section-label">Suggested actions</p>
					<div class="action-pill-row">
						<span
							v-for="action in card.explainability.suggested_actions"
							:key="`${action.action_kind}:${action.label}`"
							class="action-pill"
						>
							{{ suggestedActionLabel(action) }}
						</span>
					</div>
				</div>

				<footer class="attention-trace">
					<span>Trace</span>
					<code :title="card.trace_id">{{ shortId(card.trace_id) }}</code>
					<span>Review items</span>
					<code :title="card.review_item_ids.join(', ')">
						{{ card.review_item_ids.map(shortId).join(', ') }}
					</code>
				</footer>
			</article>
		</div>
	</section>
</template>

<style scoped>
.attention-panel {
	background: hsl(var(--card));
	border: 1px solid hsl(var(--border));
	border-radius: 10px;
	padding: 16px;
}

.panel-title {
	display: flex;
	align-items: center;
	gap: 6px;
	font-size: 14px;
	font-weight: 600;
	margin: 0 0 12px;
	color: hsl(var(--foreground));
}

.panel-badge {
	font-size: 11px;
	font-weight: 600;
	padding: 1px 7px;
	border-radius: 999px;
	background: hsl(var(--primary) / 0.12);
	color: hsl(var(--primary));
	margin-left: auto;
}

.attention-empty {
	padding: 20px 0;
	text-align: center;
	color: hsl(var(--muted-foreground));
	font-size: 13px;
}

.attention-grid {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
	gap: 12px;
}

.attention-card {
	--attention-accent: hsl(var(--muted-foreground));
	display: flex;
	flex-direction: column;
	gap: 12px;
	min-width: 0;
	padding: 14px;
	border: 1px solid hsl(var(--border));
	border-left: 3px solid var(--attention-accent);
	border-radius: 8px;
	background: hsl(var(--background));
}

.attention-card--medium {
	--attention-accent: hsl(var(--primary));
}

.attention-card--high {
	--attention-accent: hsl(var(--success));
}

.attention-card--critical {
	--attention-accent: hsl(var(--destructive));
}

.attention-card-header {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 12px;
	min-width: 0;
}

.attention-title-group {
	min-width: 0;
}

.attention-title {
	margin: 0 0 4px;
	color: hsl(var(--foreground));
	font-size: 14px;
	font-weight: 700;
	line-height: 1.35;
	overflow-wrap: anywhere;
}

.attention-summary,
.section-text,
.section-muted {
	margin: 0;
	color: hsl(var(--muted-foreground));
	font-size: 12px;
	line-height: 1.45;
	overflow-wrap: anywhere;
}

.importance-pill,
.action-pill {
	display: inline-flex;
	align-items: center;
	gap: 4px;
	min-height: 24px;
	padding: 2px 8px;
	border-radius: 999px;
	border: 1px solid hsl(var(--border));
	color: hsl(var(--foreground));
	background: hsl(var(--card));
	font-size: 11px;
	font-weight: 600;
	white-space: nowrap;
}

.importance-pill--medium {
	color: hsl(var(--primary));
	background: hsl(var(--primary) / 0.08);
}

.importance-pill--high {
	color: hsl(var(--success));
	background: hsl(var(--success) / 0.08);
}

.importance-pill--critical {
	color: hsl(var(--destructive));
	background: hsl(var(--destructive) / 0.08);
}

.attention-stats {
	display: grid;
	grid-template-columns: repeat(3, minmax(0, 1fr));
	gap: 8px;
}

.attention-stats span {
	display: flex;
	flex-direction: column;
	gap: 2px;
	padding: 8px;
	border-radius: 8px;
	background: hsl(var(--muted) / 0.45);
	color: hsl(var(--muted-foreground));
	font-size: 11px;
}

.attention-stats strong {
	color: hsl(var(--foreground));
	font-size: 14px;
}

.attention-section {
	display: flex;
	flex-direction: column;
	gap: 6px;
	min-width: 0;
}

.section-label {
	margin: 0;
	color: hsl(var(--foreground));
	font-size: 11px;
	font-weight: 700;
	text-transform: uppercase;
}

.attention-columns {
	display: grid;
	grid-template-columns: repeat(2, minmax(0, 1fr));
	gap: 12px;
}

.compact-list {
	display: flex;
	flex-direction: column;
	gap: 5px;
	margin: 0;
	padding: 0;
	list-style: none;
	color: hsl(var(--muted-foreground));
	font-size: 12px;
}

.compact-list li {
	display: flex;
	flex-wrap: wrap;
	gap: 5px;
	min-width: 0;
	overflow-wrap: anywhere;
}

code {
	color: hsl(var(--foreground));
	font-size: 11px;
	overflow-wrap: anywhere;
}

.action-pill-row {
	display: flex;
	flex-wrap: wrap;
	gap: 6px;
}

.attention-trace {
	display: grid;
	grid-template-columns: max-content minmax(0, 1fr);
	gap: 4px 8px;
	padding-top: 10px;
	border-top: 1px solid hsl(var(--border));
	color: hsl(var(--muted-foreground));
	font-size: 11px;
}

@media (max-width: 760px) {
	.attention-grid,
	.attention-columns {
		grid-template-columns: 1fr;
	}

	.attention-card-header {
		flex-direction: column;
	}
}
</style>
