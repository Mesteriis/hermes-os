<script setup lang="ts">
import { Badge, EntityIcon, Icon } from '@/shared/ui'
import type { CommunicationHermesInspectorSectionModel } from './communicationDomainElements'
import './communicationDomainElements.css'

defineProps<{
  sections: readonly CommunicationHermesInspectorSectionModel[]
}>()
</script>

<template>
	<aside class="communication-workspace-panel communication-workspace-panel--inspector" aria-label="Hermes inspector">
		<header class="communication-workspace-panel__header">
			<div class="communication-workspace-panel__title-row">
				<h2 class="communication-workspace-panel__title">Hermes inspector</h2>
				<Icon icon="tabler:sparkles" size="1rem" />
			</div>
			<p class="communication-workspace-panel__meta">Entities stay candidates until the owner promotes them.</p>
		</header>
		<div class="communication-workspace-panel__body">
			<div class="communication-inspector">
				<section
					v-for="section in sections"
					:key="section.id"
					class="communication-inspector-section"
				>
					<h3 class="communication-inspector-section__title">{{ section.title }}</h3>
					<article
						v-for="item in section.items"
						:key="item.id"
						class="communication-inspector-entity"
					>
						<div class="communication-inspector-entity__top">
							<div class="communication-inspector-entity__identity">
								<EntityIcon :entity="item.entity" :label="item.title" />
								<div class="communication-inspector-entity__body">
									<h4 class="communication-inspector-entity__title">{{ item.title }}</h4>
									<p class="communication-inspector-entity__description">{{ item.description }}</p>
								</div>
							</div>
							<Badge :variant="item.tone ?? 'neutral'">Candidate</Badge>
						</div>
						<p class="communication-inspector-entity__evidence">{{ item.evidenceLabel }}</p>
					</article>
				</section>
			</div>
		</div>
	</aside>
</template>
