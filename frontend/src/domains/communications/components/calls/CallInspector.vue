<script setup lang="ts">
import { computed } from 'vue'
import { Badge, Button, EntityIcon, Icon, ScoreGauge } from '@/shared/ui'
import '../communicationDomainElements.css'
import type { CommunicationCallInspectorModel } from '../communicationDomainElements'
import {
  communicationInspectorScoreTone,
  communicationInspectorScoreUnit,
} from '../communicationInspectorScorePresentation'

const props = defineProps<{
  model: CommunicationCallInspectorModel
}>()

const scoreUnit = computed(() => communicationInspectorScoreUnit(props.model.intelligence.maxScore))
const scoreTone = computed(() => communicationInspectorScoreTone(
  props.model.intelligence.score,
  props.model.intelligence.maxScore,
))
</script>

<template>
	<aside
		class="communication-workspace-panel communication-workspace-panel--inspector mail-inspector call-inspector"
		aria-label="Call inspector"
	>
		<div class="communication-workspace-panel__body">
			<div class="mail-inspector__content call-inspector__content">
				<section class="mail-inspector-section mail-inspector-section--intelligence call-inspector-section">
					<header class="mail-inspector-section__header">
						<div class="mail-inspector-section__title-row">
							<Icon icon="tabler:sparkles" size="1rem" class="mail-inspector-section__title-icon" />
							<h2 class="mail-inspector-section__title">Call Intelligence</h2>
						</div>
						<p class="mail-inspector-section__summary">{{ model.intelligence.summary }}</p>
					</header>
					<div class="mail-inspector-score">
						<div class="mail-inspector-score__meter">
							<ScoreGauge
								:value="model.intelligence.score"
								:max="model.intelligence.maxScore"
								:tone="scoreTone"
								size="lg"
								:label="model.intelligence.label"
								:unit="scoreUnit"
							/>
						</div>
						<div class="mail-inspector-checks">
							<div
								v-for="check in model.intelligence.checks"
								:key="check.id"
								:class="['mail-inspector-check', check.tone && `mail-inspector-check--${check.tone}`]"
							>
								<Icon :icon="check.icon ?? 'tabler:circle-check'" size="0.95rem" />
								<div class="mail-inspector-check__body">
									<strong>{{ check.label }}</strong>
									<span>{{ check.description }}</span>
								</div>
							</div>
						</div>
					</div>
				</section>

				<section class="mail-inspector-section call-inspector-section">
					<header class="mail-inspector-section__header">
						<div class="mail-inspector-section__title-row">
							<Icon icon="tabler:circles-relation" size="1rem" class="mail-inspector-section__title-icon" />
							<h2 class="mail-inspector-section__title">Extracted entities</h2>
						</div>
					</header>
					<div class="mail-inspector-entity-groups">
						<section
							v-for="group in model.entityGroups"
							:key="group.id"
							class="mail-inspector-entity-group"
						>
							<h3 class="mail-inspector-entity-group__title">{{ group.title }}</h3>
							<article
								v-for="item in group.items"
								:key="item.id"
								class="mail-inspector-entity-card"
							>
								<EntityIcon :entity="item.entity" :label="item.title" class="mail-inspector-entity-card__icon" />
								<div class="mail-inspector-entity-card__body">
									<strong>{{ item.title }}</strong>
									<span>{{ item.description }}</span>
									<small v-if="item.evidenceLabel">{{ item.evidenceLabel }}</small>
								</div>
								<Badge v-if="item.tone" :variant="item.tone">Candidate</Badge>
							</article>
						</section>
					</div>
					<div v-if="model.topics.length" class="mail-inspector-topic-row" aria-label="Call topics">
						<Badge
							v-for="topic in model.topics"
							:key="topic.id"
							:variant="topic.tone ?? 'neutral'"
						>
							{{ topic.label }}
						</Badge>
					</div>
					<dl v-if="model.semanticFacts.length" class="mail-inspector-facts">
						<div
							v-for="fact in model.semanticFacts"
							:key="fact.id"
							:class="['mail-inspector-fact', fact.tone && `mail-inspector-fact--${fact.tone}`]"
						>
							<dt>{{ fact.label }}</dt>
							<dd>{{ fact.value }}</dd>
						</div>
					</dl>
				</section>

				<section class="mail-inspector-section call-inspector-section">
					<header class="mail-inspector-section__header">
						<div class="mail-inspector-section__title-row">
							<Icon icon="tabler:wand" size="1rem" class="mail-inspector-section__title-icon" />
							<h2 class="mail-inspector-section__title">Suggested actions</h2>
						</div>
					</header>
					<div class="mail-inspector-action-list">
						<article
							v-for="action in model.suggestedActions"
							:key="action.id"
							class="mail-inspector-action"
						>
							<span :class="['mail-inspector-action__icon', action.tone && `mail-inspector-action__icon--${action.tone}`]">
								<Icon :icon="action.icon" size="1rem" />
							</span>
							<div class="mail-inspector-action__body">
								<strong>{{ action.label }}</strong>
								<span>{{ action.description }}</span>
							</div>
							<Button
								class="mail-inspector-action__button hermes-icon-button"
								variant="ghost"
								size="sm"
								icon="tabler:plus"
								:aria-label="action.label"
								:title="action.contract"
							/>
						</article>
					</div>
				</section>

				<section class="mail-inspector-section call-inspector-section">
					<header class="mail-inspector-section__header">
						<div class="mail-inspector-section__title-row">
							<Icon icon="tabler:link" size="1rem" class="mail-inspector-section__title-icon" />
							<h2 class="mail-inspector-section__title">Related context</h2>
						</div>
					</header>
					<div class="mail-inspector-context-list">
						<article
							v-for="context in model.relatedContext"
							:key="context.id"
							class="mail-inspector-context"
						>
							<span :class="['mail-inspector-context__icon', context.tone && `mail-inspector-context__icon--${context.tone}`]">
								<Icon :icon="context.icon" size="1rem" />
							</span>
							<div class="mail-inspector-context__body">
								<strong>{{ context.title }}</strong>
								<span>{{ context.description }}</span>
							</div>
							<Button
								class="mail-inspector-context__button hermes-icon-button"
								variant="ghost"
								size="sm"
								icon="tabler:external-link"
								:aria-label="context.title"
							/>
						</article>
					</div>
				</section>
			</div>
		</div>
	</aside>
</template>
