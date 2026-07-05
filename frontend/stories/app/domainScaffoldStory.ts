import type { Component } from 'vue'
import { Icon } from '@/shared/ui'
import './domainScaffoldStory.css'

export type DomainScaffoldNavItem = {
  label: string
  count?: number
  selected?: boolean
}

export type DomainScaffoldRecord = {
  title: string
  summary: string
  meta: string
  icon: string
  selected?: boolean
}

export type DomainScaffoldPreview = {
  title: string
  meta: string
  icon: string
  chips: readonly string[]
  body: readonly string[]
}

export type DomainScaffoldInspectorSection = {
  title: string
  items: readonly {
    label: string
    value: string
    icon: string
  }[]
}

export type DomainScaffoldModel = {
  title: string
  subtitle: string
  icon: string
  actionLabel: string
  searchPlaceholder: string
  navItems: readonly DomainScaffoldNavItem[]
  records: readonly DomainScaffoldRecord[]
  preview: DomainScaffoldPreview
  inspectorTitle: string
  inspectorSummary: string
  inspectorSections: readonly DomainScaffoldInspectorSection[]
}

export function createDomainScaffoldStory(model: DomainScaffoldModel): Component {
  return {
    components: { Icon },
    setup() {
      return { model }
    },
    template: `
      <section class="storybook-canvas storybook-canvas--wide">
        <article class="domain-scaffold-story" :aria-label="model.title">
          <aside class="domain-scaffold-story__panel">
            <div class="domain-scaffold-story__toolbar">
              <div class="domain-scaffold-story__action-row">
                <button class="domain-scaffold-story__action" type="button">{{ model.actionLabel }}</button>
                <button class="domain-scaffold-story__icon-button" type="button" aria-label="Настройки списка">
                  <Icon icon="tabler:settings" size="18" />
                </button>
              </div>
              <div class="domain-scaffold-story__search" role="search">
                <Icon icon="tabler:search" size="18" />
                <span>{{ model.searchPlaceholder }}</span>
              </div>
            </div>

            <nav class="domain-scaffold-story__nav" aria-label="Представления домена">
              <span
                v-for="item in model.navItems"
                :key="item.label"
                :class="[
                  'domain-scaffold-story__nav-item',
                  { 'domain-scaffold-story__nav-item--selected': item.selected }
                ]"
              >
                {{ item.label }}
                <span v-if="item.count !== undefined" class="domain-scaffold-story__count">{{ item.count }}</span>
              </span>
            </nav>

            <div class="domain-scaffold-story__list" role="list">
              <button
                v-for="record in model.records"
                :key="record.title"
                :class="[
                  'domain-scaffold-story__list-item',
                  { 'domain-scaffold-story__list-item--selected': record.selected }
                ]"
                type="button"
                role="listitem"
              >
                <span class="domain-scaffold-story__avatar">
                  <Icon :icon="record.icon" size="18" />
                </span>
                <span class="domain-scaffold-story__item-main">
                  <span class="domain-scaffold-story__item-title">{{ record.title }}</span>
                  <span class="domain-scaffold-story__item-summary">{{ record.summary }}</span>
                </span>
                <span class="domain-scaffold-story__item-meta">{{ record.meta }}</span>
              </button>
            </div>
          </aside>

          <main class="domain-scaffold-story__panel">
            <header class="domain-scaffold-story__header">
              <div class="domain-scaffold-story__title-block">
                <span class="domain-scaffold-story__hero-icon">
                  <Icon :icon="model.preview.icon" size="28" />
                </span>
                <div class="domain-scaffold-story__preview-heading">
                  <h1 class="domain-scaffold-story__preview-title">{{ model.preview.title }}</h1>
                  <p class="domain-scaffold-story__preview-meta">{{ model.preview.meta }}</p>
                </div>
              </div>
              <button class="domain-scaffold-story__icon-button" type="button" aria-label="Открыть действия">
                <Icon icon="tabler:dots" size="18" />
              </button>
            </header>

            <div class="domain-scaffold-story__chip-row" aria-label="Контекст">
              <span v-for="chip in model.preview.chips" :key="chip" class="domain-scaffold-story__chip">
                {{ chip }}
              </span>
            </div>

            <section class="domain-scaffold-story__body" aria-label="Рабочая область">
              <p v-for="paragraph in model.preview.body" :key="paragraph">{{ paragraph }}</p>
            </section>
          </main>

          <aside class="domain-scaffold-story__panel domain-scaffold-story__panel--inspector">
            <header class="domain-scaffold-story__inspector-header">
              <h2 class="domain-scaffold-story__inspector-title">{{ model.inspectorTitle }}</h2>
              <p class="domain-scaffold-story__inspector-summary">{{ model.inspectorSummary }}</p>
            </header>

            <div class="domain-scaffold-story__inspector-body">
              <section
                v-for="section in model.inspectorSections"
                :key="section.title"
                class="domain-scaffold-story__section"
              >
                <h3 class="domain-scaffold-story__section-title">{{ section.title }}</h3>
                <article
                  v-for="item in section.items"
                  :key="item.label"
                  class="domain-scaffold-story__section-item"
                >
                  <span class="domain-scaffold-story__section-icon">
                    <Icon :icon="item.icon" size="16" />
                  </span>
                  <span>
                    <p class="domain-scaffold-story__section-label">{{ item.label }}</p>
                    <p class="domain-scaffold-story__section-value">{{ item.value }}</p>
                  </span>
                </article>
              </section>
            </div>
          </aside>
        </article>
      </section>
    `
  }
}
