<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '@/platform/i18n'
import { Badge, Button, EntityIcon, Icon, ScoreGauge } from '@/shared/ui'
import '../communicationDomainElements.css'
import type { MessengerInspectorModel } from './messengerElements'
import type { MessengerConversationRuntimeActionRunner } from '@/shared/communications/types/messengerRuntimeActions'
import { communicationInspectorScoreUnit } from '../communicationInspectorScorePresentation'

const props = defineProps<{
  model: MessengerInspectorModel
  runtimeActionRunner?: MessengerConversationRuntimeActionRunner
}>()

const { t } = useI18n()
const scoreUnit = computed(() => communicationInspectorScoreUnit(props.model.intelligence.maxScore))
</script>

<template>
	<aside class="communication-workspace-panel communication-workspace-panel--inspector messenger-inspector" :aria-label="t('Messenger inspector')">
		<div class="communication-workspace-panel__body">
			<div class="messenger-inspector__content">
				<section class="messenger-inspector-section messenger-inspector-section--intelligence">
					<header class="messenger-inspector-section__header">
						<div class="messenger-inspector-section__title-row">
							<Icon icon="tabler:sparkles" size="1rem" class="messenger-inspector-section__title-icon" />
							<h2 class="messenger-inspector-section__title">{{ t('Messenger Intelligence') }}</h2>
						</div>
						<p class="messenger-inspector-section__summary">{{ model.intelligence.summary }}</p>
					</header>
					<div class="messenger-inspector-score">
						<div class="messenger-inspector-score__meter">
							<ScoreGauge
								:value="model.intelligence.score"
								:max="model.intelligence.maxScore"
								tone="success"
								size="lg"
								:label="t(model.intelligence.label)"
								:unit="scoreUnit"
							/>
						</div>
						<div class="messenger-inspector-checks">
							<div
								v-for="check in model.intelligence.checks"
								:key="check.id"
								:class="['messenger-inspector-check', check.tone && `messenger-inspector-check--${check.tone}`]"
							>
								<Icon :icon="check.icon ?? 'tabler:circle-check'" size="0.95rem" />
								<div class="messenger-inspector-check__body">
									<strong>{{ check.label }}</strong>
									<span>{{ check.description }}</span>
								</div>
							</div>
						</div>
					</div>
				</section>

				<section class="messenger-inspector-section">
					<header class="messenger-inspector-section__header">
						<div class="messenger-inspector-section__title-row">
							<Icon icon="tabler:circles-relation" size="1rem" class="messenger-inspector-section__title-icon" />
							<h2 class="messenger-inspector-section__title">{{ t('Extracted entities') }}</h2>
						</div>
					</header>
					<div class="messenger-inspector-entity-groups">
						<section
							v-for="group in model.entityGroups"
							:key="group.id"
							class="messenger-inspector-entity-group"
						>
							<h3 class="messenger-inspector-entity-group__title">{{ t(group.title) }}</h3>
							<article
								v-for="item in group.items"
								:key="item.id"
								class="messenger-inspector-card"
							>
								<EntityIcon :entity="item.entity" :label="item.title" class="messenger-inspector-card__icon" />
								<div class="messenger-inspector-card__body">
									<strong>{{ item.title }}</strong>
									<span>{{ item.description }}</span>
									<small v-if="item.evidenceLabel">{{ item.evidenceLabel }}</small>
								</div>
								<Badge v-if="item.tone" :variant="item.tone">{{ t('Candidate') }}</Badge>
							</article>
						</section>
					</div>
				</section>

				<section class="messenger-inspector-section">
					<header class="messenger-inspector-section__header">
						<div class="messenger-inspector-section__title-row">
							<Icon icon="tabler:wand" size="1rem" class="messenger-inspector-section__title-icon" />
							<h2 class="messenger-inspector-section__title">{{ t('Suggested actions') }}</h2>
						</div>
					</header>
					<div class="messenger-inspector-list">
						<article
							v-for="action in model.suggestedActions"
							:key="action.id"
							class="messenger-inspector-row"
						>
							<span :class="['messenger-inspector-row__icon', action.tone && `messenger-inspector-row__icon--${action.tone}`]">
								<Icon :icon="action.icon" size="1rem" />
							</span>
							<div class="messenger-inspector-row__body">
								<strong>{{ action.label }}</strong>
								<span>{{ action.description }}</span>
							</div>
							<Button
								class="messenger-inspector-row__button hermes-icon-button"
								variant="ghost"
								size="sm"
								icon="tabler:plus"
								:aria-label="action.label"
								:title="action.contract"
							/>
						</article>
					</div>
				</section>

				<section class="messenger-inspector-section">
					<header class="messenger-inspector-section__header">
						<div class="messenger-inspector-section__title-row">
							<Icon icon="tabler:link" size="1rem" class="messenger-inspector-section__title-icon" />
							<h2 class="messenger-inspector-section__title">{{ t('Related context') }}</h2>
						</div>
					</header>
					<div class="messenger-inspector-list">
						<article
							v-for="context in model.relatedContext"
							:key="context.id"
							class="messenger-inspector-row"
						>
							<span :class="['messenger-inspector-row__icon', context.tone && `messenger-inspector-row__icon--${context.tone}`]">
								<Icon :icon="context.icon" size="1rem" />
							</span>
							<div class="messenger-inspector-row__body">
								<strong>{{ context.title }}</strong>
								<span>{{ context.description }}</span>
							</div>
							<Button
								class="messenger-inspector-row__button hermes-icon-button"
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
