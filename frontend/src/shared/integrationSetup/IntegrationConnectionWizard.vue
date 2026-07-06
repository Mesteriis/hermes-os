<script setup lang="ts">
import Icon from '../ui/Icon.vue'
import type { SelectedIntegrationAccount } from './queries/useIntegrationConnectionWizardSurface'
import { useIntegrationConnectionWizardSurface } from './queries/useIntegrationConnectionWizardSurface'
import type { ConnectionFlowPattern, ConnectionProviderId } from '../stores/integrationConnectionWizard'

const props = defineProps<{
  open: boolean
  selectedAccount: SelectedIntegrationAccount | null
  defaultProviderId: ConnectionProviderId | null
}>()

const emit = defineEmits<{
  close: []
}>()

const surface = useIntegrationConnectionWizardSurface({
  selectedAccount: () => props.selectedAccount,
  defaultProviderId: () => props.defaultProviderId,
})

function chooseFlow(flowId: ConnectionFlowPattern) {
  surface.setActiveFlow(flowId)
}

function closeWizard() {
  surface.closeWizard()
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="integration-connection-wizard"
      role="dialog"
      aria-modal="true"
      :aria-label="surface.t('Add account')"
    >
      <button
        type="button"
        class="integration-connection-wizard__scrim"
        :aria-label="surface.t('Close')"
        @click="closeWizard"
      />

      <section class="integration-connection-wizard__panel">
        <header class="integration-connection-wizard__header">
          <div>
            <span>{{ surface.t('Account wizard') }}</span>
            <h2>{{ surface.t('Add account') }}</h2>
            <p>{{ surface.t('Choose an existing backend-managed onboarding route.') }}</p>
          </div>
          <button
            type="button"
            class="icon-button"
            :aria-label="surface.t('Close')"
            @click="closeWizard"
          >
            <Icon icon="tabler:x" />
          </button>
        </header>

        <div class="integration-connection-wizard__body">
          <nav class="integration-connection-wizard__flows" :aria-label="surface.t('Connection methods')">
            <button
              v-for="flow in surface.primaryFlowCards.value"
              :key="flow.id"
              type="button"
              :class="{ active: surface.activeFlowId.value === flow.id }"
              @click="chooseFlow(flow.id)"
            >
              <Icon :icon="flow.icon" />
              <span>
                <strong>{{ surface.t(flow.label) }}</strong>
                <small>{{ surface.t(flow.promise) }}</small>
              </span>
            </button>
            <button
              v-if="surface.exceptionFlowCard.value"
              type="button"
              :class="{ active: surface.activeFlowId.value === surface.exceptionFlowCard.value.id }"
              @click="chooseFlow(surface.exceptionFlowCard.value.id)"
            >
              <Icon :icon="surface.exceptionFlowCard.value.icon" />
              <span>
                <strong>{{ surface.t(surface.exceptionFlowCard.value.label) }}</strong>
                <small>{{ surface.t(surface.exceptionFlowCard.value.promise) }}</small>
              </span>
            </button>
          </nav>

          <section class="integration-connection-wizard__providers">
            <article
              v-for="provider in surface.activeFlow.value.providers"
              :key="provider.id"
              class="integration-connection-wizard__provider"
              :class="{ active: provider.id === surface.selectedProvider.value.id }"
            >
              <button type="button" @click="surface.wizard.previewProvider(provider.id)">
                <Icon :icon="provider.icon" />
                <span>
                  <strong>{{ surface.t(provider.label) }}</strong>
                  <small>{{ surface.providerStatus(provider.id) }}</small>
                </span>
              </button>
            </article>
          </section>

          <section class="integration-connection-wizard__details">
            <div class="integration-connection-wizard__selected">
              <Icon :icon="surface.activeFlowIcon.value" />
              <div>
                <span>{{ surface.t(surface.selectedProvider.value.flowLabel) }}</span>
                <h3>{{ surface.t(surface.selectedProvider.value.label) }}</h3>
                <p>{{ surface.t(surface.selectedProvider.value.guidance) }}</p>
              </div>
            </div>

            <ol class="integration-connection-wizard__steps">
              <li v-for="step in surface.launchSteps.value" :key="step.title">
                <strong>{{ surface.t(step.title) }}</strong>
                <span>{{ surface.t(step.detail) }}</span>
              </li>
            </ol>

            <div
              v-if="surface.wizard.guidedResult"
              class="integration-connection-wizard__result"
              :class="`is-${surface.wizard.guidedResult.kind}`"
            >
              <strong>{{ surface.wizard.guidedResult.title }}</strong>
              <p>{{ surface.wizard.guidedResult.message }}</p>
              <figure v-if="surface.guidedQrImage.value" class="integration-connection-wizard__qr">
                <img :src="surface.guidedQrImage.value" :alt="surface.t('Telegram QR login')" >
                <figcaption>
                  {{ surface.t('Scan this QR code with Telegram to finish account linking.') }}
                </figcaption>
              </figure>
              <a
                v-if="surface.wizard.guidedResult.qrLink"
                :href="surface.wizard.guidedResult.qrLink"
                target="_blank"
                rel="noreferrer"
              >
                {{ surface.t('Open QR login link') }}
              </a>
              <ul v-if="surface.wizard.guidedResult.blockers?.length">
                <li v-for="blocker in surface.wizard.guidedResult.blockers" :key="blocker">
                  {{ blocker }}
                </li>
              </ul>
              <button
                v-if="surface.canRefreshGuidedResult.value"
                type="button"
                class="secondary-button"
                @click="surface.handleRefreshGuidedResult"
              >
                <Icon icon="tabler:refresh" />
                {{ surface.t('Refresh status') }}
              </button>
            </div>

            <div v-else-if="surface.wizard.errorMessage" class="integration-connection-wizard__result is-error">
              <strong>{{ surface.t('Connection failed') }}</strong>
              <p>{{ surface.wizard.errorMessage }}</p>
            </div>

            <footer class="integration-connection-wizard__footer">
              <span v-if="surface.selectedAccountLabel.value">
                {{ surface.t('Selected account') }}: {{ surface.selectedAccountLabel.value }}
              </span>
              <span v-else>{{ surface.t('No existing account selected') }}</span>
              <button
                type="button"
                class="primary-button"
                :disabled="!surface.canSubmit.value"
                @click="surface.handleSubmit"
              >
                <Icon :icon="surface.isSubmitting.value ? 'tabler:loader-2' : surface.selectedProvider.value.icon" />
                {{ surface.submitLabel.value }}
              </button>
            </footer>
          </section>
        </div>
      </section>
    </div>
  </Teleport>
</template>
