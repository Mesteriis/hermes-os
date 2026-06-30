import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type {
  WhatsappWebSession,
  WhatsappCapabilitiesResponse
} from '../types/whatsapp'

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

  return {
    // State
    whatsappSessions,
    whatsappCapabilities,
    selectedWhatsappSessionId,
    whatsappError,
    whatsappActionMessage,
    isWhatsappLoading,
    isWhatsappActionSubmitting,
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
    setWhatsappActionMessage
  }
})
