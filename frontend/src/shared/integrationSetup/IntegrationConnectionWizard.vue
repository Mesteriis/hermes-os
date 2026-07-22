<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Button from '../ui/primitives/Button.vue'
import Icon from '../ui/Icon.vue'
import Input from '../ui/primitives/Input.vue'
import Steps from '../ui/Steps.vue'
import Switch from '../ui/primitives/Switch.vue'
import type { StepsItem } from '../ui/Steps.types'
import type { SelectedIntegrationAccount } from './queries/useIntegrationConnectionWizardSurface'
import { useIntegrationConnectionWizardSurface } from './queries/useIntegrationConnectionWizardSurface'
import type { ConnectionProviderId } from '../stores/integrationConnectionWizard'
import {
  canAdvanceIntegrationConnectionWizard,
  integrationCheckIcon,
  integrationProviderIconTone,
} from './integrationConnectionWizardPresentation'

const props = defineProps<{
  open: boolean
  selectedAccount: SelectedIntegrationAccount | null
  defaultProviderId: ConnectionProviderId | null
}>()

const emit = defineEmits<{
  close: []
}>()

const currentStep = ref(1)
const surface = useIntegrationConnectionWizardSurface({
  selectedAccount: () => props.selectedAccount,
  defaultProviderId: () => props.defaultProviderId,
})

const wizardOpen = computed({
  get: () => props.open,
  set: (value: boolean) => {
    if (!value) closeWizard()
  },
})

const providerSteps = computed<StepsItem[]>(() => [
  { title: surface.t('Авторизация') },
  { title: surface.t('Проверка') },
  { title: surface.t('Сервисы') },
])

const canMoveForward = computed(() => canAdvanceIntegrationConnectionWizard(
  currentStep.value,
  surface.canSubmit.value,
  surface.selectedProvider.value.id,
  surface.showSelectedProviderChecks.value,
))
const providerTitle = computed(() => surface.t(surface.selectedProvider.value.label))

watch(() => props.open, (isOpen) => {
  if (isOpen) {
    currentStep.value = 1
  }
})

watch(
  () => [props.open, surface.selectedProvider.value.id] as const,
  ([isOpen, providerId]) => {
    if (isOpen && providerId === 'telegram') {
      void startTelegramQrStep()
    }
  },
  { flush: 'post' }
)

function closeWizard(): void {
  surface.closeWizard()
  currentStep.value = 1
  emit('close')
}

function chooseProvider(providerId: ConnectionProviderId): void {
  surface.previewProvider(providerId)
  currentStep.value = 1
  if (providerId === 'telegram') {
    void startTelegramQrStep()
  }
}

async function startTelegramQrStep(): Promise<void> {
  surface.setTelegramLoginMode('qr')
  currentStep.value = 2
  await surface.ensureTelegramQrLoginStarted()
}

async function goNext(): Promise<void> {
  if (currentStep.value === 1) {
    currentStep.value = 2
    await surface.handleSubmit()
    return
  }
  if (currentStep.value === 2) {
    currentStep.value = 3
    return
  }
  closeWizard()
}

function goBack(): void {
  if (currentStep.value > 1) {
    currentStep.value -= 1
  }
}

</script>

<template>
  <Steps
    v-model:open="wizardOpen"
    v-model:step="currentStep"
    :title="surface.t('Добавить аккаунт')"
    :description="providerTitle"
    :steps="providerSteps"
    :step-count="3"
    :can-advance="canMoveForward"
    :busy="surface.isSubmitting.value"
    :next-label="currentStep === 1 ? surface.submitLabel.value : surface.t('Дальше')"
    :finish-label="surface.t('Готово')"
    :previous-label="surface.t('Назад')"
    :steps-label="surface.t('Шаги мастера')"
    size="lg"
  >
    <template #step-1>
      <section class="app-connection-wizard">
        <div class="app-connection-wizard__providers" :aria-label="surface.t('Провайдеры')">
          <button
            v-for="provider in surface.providerCards.value"
            :key="provider.id"
            type="button"
            class="app-connection-wizard__provider"
            :class="{ active: provider.id === surface.selectedProvider.value.id }"
            @click="chooseProvider(provider.id)"
          >
            <i class="app-connection-wizard__provider-icon" :class="integrationProviderIconTone(provider.id)" aria-hidden="true">
              <Icon :icon="provider.icon" />
            </i>
            <span>
              <strong>{{ surface.t(provider.label) }}</strong>
              <small>{{ surface.providerStatus(provider.id) }}</small>
            </span>
          </button>
        </div>

        <div class="app-connection-wizard__fields">
          <label v-if="surface.selectedProvider.value.id === 'mail'" class="app-connection-wizard__field">
            <span>{{ surface.t('Название аккаунта') }}</span>
            <Input
              :model-value="surface.gmailAccountLabel.value"
              :aria-label="surface.t('Название аккаунта')"
              @update:model-value="surface.gmailAccountLabel.value = $event"
            />
          </label>

          <template v-else-if="surface.selectedProvider.value.id === 'icloud'">
            <label class="app-connection-wizard__field">
              <span>{{ surface.t('Email') }}</span>
              <Input
                :model-value="surface.icloudEmail.value"
                type="email"
                :aria-label="surface.t('Email')"
                @update:model-value="surface.icloudEmail.value = $event"
              />
            </label>
            <label class="app-connection-wizard__field">
              <span>{{ surface.t('Пароль приложения') }}</span>
              <Input
                :model-value="surface.icloudPassword.value"
                type="password"
                :aria-label="surface.t('Пароль приложения')"
                @update:model-value="surface.icloudPassword.value = $event"
              />
            </label>
            <label class="app-connection-wizard__field app-connection-wizard__field--wide">
              <span>{{ surface.t('Название аккаунта') }}</span>
              <Input
                :model-value="surface.icloudAccountLabel.value"
                :aria-label="surface.t('Название аккаунта')"
                @update:model-value="surface.icloudAccountLabel.value = $event"
              />
            </label>
          </template>

          <template v-else-if="surface.selectedProvider.value.id === 'telegram'">
            <div class="app-connection-wizard__mode" role="group" :aria-label="surface.t('Способ входа Telegram')">
              <button
                type="button"
                class="app-connection-wizard__mode-button"
                :class="{ active: surface.telegramLoginMode.value === 'qr' }"
                @click="surface.setTelegramLoginMode('qr')"
              >
                <Icon icon="tabler:qrcode" />
                {{ surface.t('QR') }}
              </button>
              <button
                type="button"
                class="app-connection-wizard__mode-button"
                :class="{ active: surface.telegramLoginMode.value === 'phone' }"
                @click="surface.setTelegramLoginMode('phone')"
              >
                <Icon icon="tabler:phone" />
                {{ surface.t('Телефон') }}
              </button>
            </div>

            <template v-if="surface.telegramLoginMode.value === 'qr'">
              <div class="app-connection-wizard__method-card app-connection-wizard__field--wide">
                <Icon icon="tabler:qrcode" />
                <span>
                  <strong>{{ surface.t('QR-вход') }}</strong>
                  <small>{{ surface.t('Код появится на следующем шаге.') }}</small>
                </span>
              </div>
              <label class="app-connection-wizard__field app-connection-wizard__field--wide">
                <span>{{ surface.t('Название аккаунта') }}</span>
                <Input
                  :model-value="surface.telegramAccountLabel.value"
                  :aria-label="surface.t('Название аккаунта')"
                  @update:model-value="surface.telegramAccountLabel.value = $event"
                />
              </label>
            </template>

            <template v-else>
              <label class="app-connection-wizard__field">
                <span>{{ surface.t('Телефон') }}</span>
                <Input
                  :model-value="surface.telegramPhone.value"
                  type="tel"
                  :aria-label="surface.t('Телефон')"
                  @update:model-value="surface.telegramPhone.value = $event"
                />
              </label>
              <label class="app-connection-wizard__field">
                <span>{{ surface.t('Название аккаунта') }}</span>
                <Input
                  :model-value="surface.telegramAccountLabel.value"
                  :aria-label="surface.t('Название аккаунта')"
                  @update:model-value="surface.telegramAccountLabel.value = $event"
                />
              </label>
            </template>
          </template>

          <template v-else-if="surface.selectedProvider.value.id === 'whatsapp'">
            <label class="app-connection-wizard__field">
              <span>{{ surface.t('Устройство') }}</span>
              <Input
                :model-value="surface.whatsappDeviceName.value"
                :aria-label="surface.t('Устройство')"
                @update:model-value="surface.whatsappDeviceName.value = $event"
              />
            </label>
            <label class="app-connection-wizard__field">
              <span>{{ surface.t('Название аккаунта') }}</span>
              <Input
                :model-value="surface.whatsappAccountLabel.value"
                :aria-label="surface.t('Название аккаунта')"
                @update:model-value="surface.whatsappAccountLabel.value = $event"
              />
            </label>
          </template>
        </div>
      </section>
    </template>

    <template #step-2>
      <section
        class="app-connection-wizard"
        :class="{ 'app-connection-wizard--telegram-auth': surface.selectedProvider.value.id === 'telegram' }"
      >
        <div
          v-if="surface.wizard.guidedResult"
          class="app-connection-wizard__result"
          :class="[
            `is-${surface.wizard.guidedResult.kind}`,
            surface.selectedProvider.value.id === 'telegram'
              ? `app-connection-wizard__result--telegram-${surface.wizard.guidedResult.status || 'unknown'}`
              : null
          ]"
        >
          <strong>{{ surface.wizard.guidedResult.title }}</strong>
          <span>{{ surface.guidedResultMessage.value }}</span>
          <figure
            v-if="
              surface.guidedQrImage.value &&
              (surface.selectedProvider.value.id !== 'telegram' ||
                surface.wizard.guidedResult.status === 'waiting_qr_scan')
            "
            class="app-connection-wizard__qr"
            :class="{ 'app-connection-wizard__qr--telegram': surface.selectedProvider.value.id === 'telegram' }"
          >
            <div class="app-connection-wizard__qr-badge" aria-hidden="true">
              <Icon icon="tabler:brand-telegram" />
            </div>
            <img :src="surface.guidedQrImage.value" :alt="surface.t('Telegram QR')" >
            <figcaption v-if="surface.selectedProvider.value.id === 'telegram'">
              {{ surface.t('Сканируйте в Telegram на телефоне') }}
            </figcaption>
          </figure>
          <a
            v-if="surface.wizard.guidedResult.qrLink && surface.selectedProvider.value.id !== 'telegram'"
            :href="surface.wizard.guidedResult.qrLink"
            target="_blank"
            rel="noreferrer"
          >
            {{ surface.t('Открыть QR') }}
          </a>
          <ul v-if="surface.wizard.guidedResult.blockers?.length">
            <li v-for="blocker in surface.wizard.guidedResult.blockers" :key="blocker">
              {{ blocker }}
            </li>
          </ul>
        </div>

        <form
          v-if="surface.selectedProvider.value.id === 'telegram' && surface.telegramQrNeedsPassword.value"
          class="app-connection-wizard__password"
          @submit.prevent="surface.submitTelegramCloudPassword"
        >
          <label class="app-connection-wizard__field">
            <span>{{ surface.t('Облачный пароль Telegram') }}</span>
            <Input
              :model-value="surface.telegramCloudPassword.value"
              type="password"
              autocomplete="current-password"
              :aria-label="surface.t('Облачный пароль Telegram')"
              @update:model-value="surface.telegramCloudPassword.value = $event"
            />
          </label>
          <Button
            class="app-connection-wizard__password-submit"
            type="submit"
            icon="tabler:lock-check"
            :loading="surface.isSubmitting.value"
            :disabled="!surface.canSubmitTelegramCloudPassword.value"
          >
            {{ surface.t('Продолжить') }}
          </Button>
        </form>

        <div v-if="surface.showSelectedProviderChecks.value" class="app-connection-wizard__checks">
          <article
            v-for="check in surface.selectedProviderChecks.value"
            :key="check.label"
            class="app-connection-wizard__check"
            :class="`is-${check.status}`"
          >
              <Icon :icon="integrationCheckIcon(check.status)" />
            <span>
              <strong>{{ check.label }}</strong>
              <small>{{ check.description }}</small>
            </span>
          </article>
        </div>

        <Button
          v-if="surface.canRefreshGuidedResult.value"
          variant="secondary"
          icon="tabler:refresh"
          :loading="surface.isSubmitting.value"
          @click="surface.handleRefreshGuidedResult"
        >
          {{ surface.t('Обновить') }}
        </Button>
      </section>
    </template>

    <template #step-3>
      <section class="app-connection-wizard">
        <div class="app-connection-wizard__capabilities">
          <label
            v-for="capability in surface.selectedProviderCapabilities.value"
            :key="capability.label"
            class="app-connection-wizard__capability"
          >
            <span>
              <strong>{{ capability.label }}</strong>
              <small>{{ capability.description }}</small>
            </span>
            <Switch :model-value="capability.enabled" :disabled="capability.locked" :aria-label="capability.label" />
          </label>
        </div>
      </section>
    </template>

    <template #footer>
      <div class="hermes-steps__dock">
        <Button
          class="hermes-steps__dock-back"
          variant="ghost"
          icon="tabler:arrow-left"
          :aria-label="surface.t('Назад')"
          :disabled="currentStep === 1 || surface.isSubmitting.value"
          @click="goBack"
        />
        <nav class="hermes-steps__dots" :aria-label="surface.t('Шаги мастера')">
          <button
            v-for="item in providerSteps"
            :key="item.title"
            class="hermes-steps__dot"
            :class="{ 'hermes-steps__dot--active': providerSteps.indexOf(item) + 1 === currentStep }"
            type="button"
            :aria-current="providerSteps.indexOf(item) + 1 === currentStep ? 'step' : undefined"
            :aria-label="item.title"
            :disabled="surface.isSubmitting.value"
            @click="currentStep = providerSteps.indexOf(item) + 1"
          />
        </nav>
        <Button
          class="hermes-steps__dock-next"
          :loading="surface.isSubmitting.value"
          :disabled="!canMoveForward"
          @click="goNext"
        >
          {{ currentStep === 3 ? surface.t('Готово') : currentStep === 1 ? surface.submitLabel.value : surface.t('Дальше') }}
        </Button>
      </div>
    </template>
  </Steps>
</template>
