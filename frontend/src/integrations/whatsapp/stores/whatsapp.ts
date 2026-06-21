import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type {
  WhatsappWebSession,
  WhatsappCapabilitiesResponse
} from '../types/whatsapp'

export interface WhatsappMessageForm {
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  chat_title: string
  sender_id: string
  sender_display_name: string
  text: string
  import_batch_id: string
  occurred_at: string
  delivery_state: string
}

export const useWhatsappStore = defineStore('whatsapp-ui', () => {
  // Data state
  const whatsappSessions = ref<WhatsappWebSession[]>([])
  const whatsappCapabilities = ref<WhatsappCapabilitiesResponse | null>(null)

  // Selection state
  const selectedWhatsappSessionId = ref('')

  // UI state
  const whatsappError = ref('')
  const whatsappActionMessage = ref('')
  const isWhatsappLoading = ref(false)
  const isWhatsappActionSubmitting = ref(false)

  // Fixture message form
  const whatsappMessageForm = ref<WhatsappMessageForm>({
    account_id: 'whatsapp-primary',
    provider_chat_id: 'wa-fixture-chat-1',
    provider_message_id: '',
    chat_title: 'WhatsApp Fixture Chat',
    sender_id: 'wa-fixture-sender-1',
    sender_display_name: 'Alice',
    text: 'WhatsApp fixture WhatsApp Web message for local memory and graph recall.',
    import_batch_id: 'whatsapp-web-fixture-ui',
    occurred_at: new Date().toISOString(),
    delivery_state: 'received'
  })

  // Derived
  const selectedWhatsappSession = computed(() =>
    whatsappSessions.value.find((s) => s.session_id === selectedWhatsappSessionId.value) ??
    whatsappSessions.value[0] ??
    null
  )

  const whatsappClosureCapabilities = computed(() =>
    whatsappCapabilities.value?.capabilities.filter((c) => c.closure_gate) ?? []
  )

  const whatsappBlockedCapabilities = computed(() =>
    whatsappCapabilities.value?.capabilities.filter((c) => c.status === 'blocked') ?? []
  )

  // Actions
  function setWhatsappData(data: {
    sessions: WhatsappWebSession[]
    capabilities: WhatsappCapabilitiesResponse | null
    selectedSessionId: string
    error: string
  }) {
    whatsappSessions.value = data.sessions
    whatsappCapabilities.value = data.capabilities
    selectedWhatsappSessionId.value = data.selectedSessionId
    whatsappError.value = data.error
  }

  function selectWhatsappSession(session: WhatsappWebSession) {
    selectedWhatsappSessionId.value = session.session_id
    whatsappMessageForm.value = {
      ...whatsappMessageForm.value,
      account_id: session.account_id
    }
  }

  function setWhatsappLoading(loading: boolean) {
    isWhatsappLoading.value = loading
  }

  function setWhatsappActionSubmitting(submitting: boolean) {
    isWhatsappActionSubmitting.value = submitting
  }

  function setWhatsappError(error: string) {
    whatsappError.value = error
  }

  function setWhatsappActionMessage(message: string) {
    whatsappActionMessage.value = message
  }

  function resetWhatsappMessageForm() {
    whatsappMessageForm.value = {
      account_id: 'whatsapp-primary',
      provider_chat_id: 'wa-fixture-chat-1',
      provider_message_id: '',
      chat_title: 'WhatsApp Fixture Chat',
      sender_id: 'wa-fixture-sender-1',
      sender_display_name: 'Alice',
      text: '',
      import_batch_id: 'whatsapp-web-fixture-ui',
      occurred_at: new Date().toISOString(),
      delivery_state: 'received'
    }
  }

  return {
    // State
    whatsappSessions,
    whatsappCapabilities,
    selectedWhatsappSessionId,
    whatsappError,
    whatsappActionMessage,
    isWhatsappLoading,
    isWhatsappActionSubmitting,
    whatsappMessageForm,
    // Derived
    selectedWhatsappSession,
    whatsappClosureCapabilities,
    whatsappBlockedCapabilities,
    // Actions
    setWhatsappData,
    selectWhatsappSession,
    setWhatsappLoading,
    setWhatsappActionSubmitting,
    setWhatsappError,
    setWhatsappActionMessage,
    resetWhatsappMessageForm
  }
})
