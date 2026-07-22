<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { MaintenanceSettingsSurface } from '../queries/useMaintenanceSettingsSurface'
import { useMaintenanceSettingsPanelController } from '../queries/useMaintenanceSettingsPanelController'

const props = defineProps<{
  surface: MaintenanceSettingsSurface
}>()

const { t } = useI18n()

const {
  handleRefresh,
  handleSelectAction,
  handleConfirmationInput,
  handleRunSelectedAction,
} = useMaintenanceSettingsPanelController({
  surface: props.surface,
})
</script>

<template>
  <section class="settings-section settings-maintenance-section">
    <header class="settings-section-toolbar">
      <div>
        <h3>{{ t('Maintenance') }}</h3>
        <p>{{ t('Local storage sizes, backups, cleanup and restore guardrails.') }}</p>
      </div>
      <button
        type="button"
        class="icon-button"
        :title="t('Refresh maintenance overview')"
        :aria-label="t('Refresh maintenance overview')"
        @click="handleRefresh()"
      >
        <Icon icon="tabler:refresh" />
      </button>
    </header>

    <div v-if="surface.isLoading.value" class="settings-empty-state">
      <Icon icon="tabler:loader-2" />
      <strong>{{ t('Loading maintenance data') }}</strong>
    </div>

    <div v-else-if="surface.errorMessage.value" class="settings-empty-state">
      <Icon icon="tabler:alert-circle" />
      <strong>{{ t('Maintenance unavailable') }}</strong>
      <span>{{ surface.errorMessage.value }}</span>
    </div>

    <template v-else>
      <section class="settings-maintenance-summary" :aria-label="t('Maintenance summary')">
        <article
          v-for="tile in surface.summaryTiles.value"
          :key="tile.id"
          class="settings-maintenance-summary-tile"
          :class="`tone-${tile.tone}`"
        >
          <Icon :icon="tile.icon" />
          <span>{{ t(tile.label) }}</span>
          <strong>{{ tile.value }}</strong>
          <small>{{ t(tile.detail) }}</small>
        </article>
      </section>

      <section class="settings-maintenance-panel" :aria-label="t('Storage inventory')">
        <header class="settings-maintenance-panel__header">
          <div>
            <span>{{ t('Inventory') }}</span>
            <strong>{{ t('Database, logs and local storage') }}</strong>
          </div>
          <small>{{ t('Payload bodies stay in local storage; this view reports sizes without exposing private contents.') }}</small>
        </header>

        <div class="settings-maintenance-table-scroll">
          <table class="settings-maintenance-table">
            <thead>
              <tr>
                <th scope="col">{{ t('Surface') }}</th>
                <th scope="col">{{ t('Path') }}</th>
                <th scope="col">{{ t('Size') }}</th>
                <th scope="col">{{ t('Files') }}</th>
                <th scope="col">{{ t('Status') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="row in surface.inventoryRows.value"
                :key="row.id"
                :class="`tone-${row.tone}`"
              >
                <td>
                  <span class="settings-maintenance-identity">
                    <Icon :icon="row.icon" />
                    <span>
                      <strong>{{ t(row.label) }}</strong>
                      <small>{{ t(row.description) }}</small>
                    </span>
                  </span>
                </td>
                <td><code>{{ row.path_label }}</code></td>
                <td>{{ row.sizeLabel }}</td>
                <td>{{ row.fileCountLabel }}</td>
                <td>
                  <span class="settings-maintenance-state" :class="`tone-${row.tone}`">{{ t(row.status) }}</span>
                  <small>{{ t(row.detail) }}</small>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <section class="settings-maintenance-grid">
        <section class="settings-maintenance-panel" :aria-label="t('Backups')">
          <header class="settings-maintenance-panel__header settings-maintenance-panel__header--compact">
            <div>
              <span>{{ t('Backups') }}</span>
              <strong>{{ t('Recent snapshots') }}</strong>
            </div>
            <small>{{ t('Database/vault backups come from the repository backup script; storage backups copy local blob roots.') }}</small>
          </header>

          <div v-if="surface.backupRows.value.length === 0" class="settings-empty-state">
            <Icon icon="tabler:archive-off" />
            <strong>{{ t('No backups found') }}</strong>
          </div>

          <div v-else class="settings-maintenance-backup-list">
            <article
              v-for="backup in surface.backupRows.value"
              :key="backup.id"
              class="settings-maintenance-backup"
            >
              <header>
                <strong>{{ backup.label }}</strong>
                <span>{{ backup.sizeLabel }}</span>
              </header>
              <small>{{ backup.path_label }}</small>
              <footer>
                <code>{{ backup.contentsLabel }}</code>
                <span>{{ backup.createdAtLabel }}</span>
              </footer>
            </article>
          </div>
        </section>

        <section class="settings-maintenance-panel" :aria-label="t('Maintenance actions')">
          <header class="settings-maintenance-panel__header settings-maintenance-panel__header--compact">
            <div>
              <span>{{ t('Actions') }}</span>
              <strong>{{ t('Cleanup, backup and restore') }}</strong>
            </div>
            <small>{{ t('Destructive operations require an exact confirmation phrase.') }}</small>
          </header>

          <div class="settings-maintenance-action-list">
            <button
              v-for="action in surface.actionRows.value"
              :key="action.id"
              type="button"
              class="settings-maintenance-action"
              :class="[`tone-${action.tone}`, { active: surface.selectedActionId.value === action.id }]"
              :disabled="surface.isBusy.value"
              @click="handleSelectAction(action.id)"
            >
              <Icon :icon="action.icon" />
              <span>
                <strong>{{ t(action.label) }}</strong>
                <small>{{ t(action.description) }}</small>
              </span>
              <em>{{ t(action.availabilityLabel) }}</em>
            </button>
          </div>

          <form
            v-if="surface.selectedAction.value"
            class="settings-maintenance-confirm"
            @submit.prevent="handleRunSelectedAction()"
          >
            <label>
              <span>{{ t('Confirmation') }}</span>
              <input
                type="text"
                :value="surface.confirmationDraft.value"
                :placeholder="surface.selectedAction.value.confirmation_phrase ?? t('No confirmation required')"
                :disabled="!surface.selectedAction.value.enabled || surface.isBusy.value"
                autocomplete="off"
                @input="handleConfirmationInput"
              >
            </label>
            <button
              type="submit"
              :class="surface.selectedAction.value.destructive ? 'danger-button' : 'secondary-button'"
              :disabled="!surface.canRunSelectedAction.value"
            >
              <Icon :icon="surface.isBusy.value ? 'tabler:loader-2' : 'tabler:player-play'" />
              {{ t(surface.selectedAction.value.buttonLabel) }}
            </button>
            <small v-if="surface.selectedAction.value.disabled_reason">
              {{ t(surface.selectedAction.value.disabled_reason) }}
            </small>
          </form>
        </section>
      </section>
    </template>
  </section>
</template>
