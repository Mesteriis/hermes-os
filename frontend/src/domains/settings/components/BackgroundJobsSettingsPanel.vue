<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { BackgroundJobsSettingsSurface } from '../queries/useBackgroundJobsSettingsSurface'

defineProps<{
  surface: BackgroundJobsSettingsSurface
}>()

const { t } = useI18n()
</script>

<template>
  <section class="settings-section settings-background-section">
    <header class="settings-section-toolbar">
      <div>
        <h3>{{ t('Background Jobs') }}</h3>
        <p>{{ t('Mail sync, provider runtimes, Signal Hub dispatchers and projection workers.') }}</p>
      </div>
      <button
        type="button"
        class="icon-button"
        :title="t('Refresh job status')"
        :aria-label="t('Refresh job status')"
        @click="surface.handleRefresh()"
      >
        <Icon icon="tabler:refresh" />
      </button>
    </header>

    <section class="settings-background-summary" :aria-label="t('Background Jobs summary')">
      <article
        v-for="tile in surface.summaryTiles.value"
        :key="tile.id"
        class="settings-background-summary-tile"
        :class="`tone-${tile.tone}`"
      >
        <Icon :icon="tile.icon" />
        <span>{{ t(tile.label) }}</span>
        <strong>{{ tile.value }}</strong>
        <small>{{ t(tile.detail) }}</small>
      </article>
    </section>

    <section class="settings-background-panel" :aria-label="t('Background job inventory')">
      <header class="settings-background-panel__header">
        <div>
          <span>{{ t('Inventory') }}</span>
          <strong>{{ t('Schedulers and workers') }}</strong>
        </div>
        <small>{{ t('Live status is shown when mail sync or Signal Hub runtime APIs expose it.') }}</small>
        <nav class="settings-background-tabs" :aria-label="t('Background job categories')">
          <button
            v-for="tab in surface.backgroundJobTabs.value"
            :key="tab.id"
            type="button"
            class="settings-background-tab"
            :class="{ active: surface.activeJobFilter.value === tab.id }"
            :aria-pressed="surface.activeJobFilter.value === tab.id"
            @click="surface.handleSelectJobFilter(tab.id)"
          >
            <span>{{ t(tab.label) }}</span>
            <strong>{{ tab.count }}</strong>
          </button>
        </nav>
      </header>

      <div v-if="surface.isLoading.value" class="settings-empty-state">
        <Icon icon="tabler:loader-2" />
        <strong>{{ t('Loading background jobs') }}</strong>
      </div>

      <div v-else class="settings-background-job-list">
        <article
          v-for="job in surface.filteredBackgroundJobRows.value"
          :key="job.id"
          class="settings-background-job"
          :class="`tone-${job.tone}`"
        >
          <header>
            <i class="settings-provider-icon" aria-hidden="true">
              <Icon :icon="job.icon" />
            </i>
            <div>
              <span>{{ t(job.groupLabel) }}</span>
              <strong>{{ t(job.label) }}</strong>
            </div>
            <em :class="`tone-${job.tone}`">{{ t(job.statusLabel) }}</em>
            <button
              v-if="job.controlSection"
              type="button"
              class="icon-button"
              :title="t('Open control surface')"
              :aria-label="`${t('Open control surface')}: ${t(job.label)}`"
              @click="surface.handleOpenControl(job.controlSection)"
            >
              <Icon icon="tabler:arrow-right" />
            </button>
          </header>

          <p>{{ t(job.description) }}</p>

          <dl class="settings-background-job-facts">
            <div>
              <dt>{{ t('Status') }}</dt>
              <dd>{{ t(job.statusDetail) }}</dd>
            </div>
            <div>
              <dt>{{ t('Cadence') }}</dt>
              <dd>{{ t(job.cadence) }}</dd>
            </div>
            <div>
              <dt>{{ t('Metric') }}</dt>
              <dd>{{ job.metric }}</dd>
            </div>
            <div>
              <dt>{{ t('Last activity') }}</dt>
              <dd>{{ job.lastActivityLabel }}</dd>
            </div>
            <div>
              <dt>{{ t('Next run') }}</dt>
              <dd>{{ job.nextRunLabel }}</dd>
            </div>
            <div>
              <dt>{{ t('Evidence') }}</dt>
              <dd>{{ job.evidence }}</dd>
            </div>
          </dl>

          <footer>
            <code v-for="runtime in job.runtimeKinds" :key="runtime">{{ runtime }}</code>
            <span v-if="job.runtimeKinds.length === 0">{{ t('No backend runtime kind') }}</span>
          </footer>
        </article>
      </div>
    </section>

    <section class="settings-background-panel settings-background-mail-panel" :aria-label="t('Mail sync accounts')">
      <header class="settings-background-panel__header settings-background-panel__header--compact">
        <div>
          <span>{{ t('Mail') }}</span>
          <strong>{{ t('Account sync status') }}</strong>
        </div>
        <small>{{ t('Per-account status from /api/v1/integrations/mail/accounts/sync-status.') }}</small>
      </header>

      <div v-if="surface.mailStatusRows.value.length === 0" class="settings-empty-state">
        <Icon icon="tabler:mail-off" />
        <strong>{{ t('No mail sync statuses') }}</strong>
        <span>{{ t('Mail account sync statuses will appear here after the backend reports them.') }}</span>
      </div>

      <div v-else class="settings-background-mail-table-scroll">
        <table class="settings-background-mail-table">
          <thead>
            <tr>
              <th scope="col">{{ t('Account') }}</th>
              <th scope="col">{{ t('Status') }}</th>
              <th scope="col">{{ t('Progress') }}</th>
              <th scope="col">{{ t('Last activity') }}</th>
              <th scope="col">{{ t('Next run') }}</th>
              <th scope="col">{{ t('Result') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in surface.mailStatusRows.value" :key="row.accountId">
              <td><code>{{ row.accountId }}</code></td>
              <td>
                <span class="settings-background-state" :class="`tone-${row.tone}`">{{ row.status }}</span>
                <small>{{ row.phase }}</small>
              </td>
              <td>{{ row.progressLabel }}</td>
              <td>{{ row.lastActivityLabel }}</td>
              <td>{{ row.nextRunLabel }}</td>
              <td>
                <span>{{ row.throughputLabel }}</span>
                <small v-if="row.errorLabel">{{ row.errorLabel }}</small>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </section>
</template>
