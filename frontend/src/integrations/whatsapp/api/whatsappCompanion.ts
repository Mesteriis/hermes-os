import { invoke } from '@tauri-apps/api/core'
import type {
  WhatsAppWebCompanionManifest,
  WhatsAppWebCompanionRelayObservationReceipt,
  WhatsAppWebCompanionRelayObservationRequest,
  WhatsAppWebCompanionCommandResultRequest,
} from '../../../shared/communications/types/whatsapp'

type WhatsAppWebCompanionRequest = {
  account_id: string
}

export async function getWhatsappWebCompanionManifest(
  accountId: string
): Promise<WhatsAppWebCompanionManifest> {
  return invoke<WhatsAppWebCompanionManifest>(
    'whatsapp_web_companion_manifest',
    companionRequest(accountId)
  )
}

export async function startHiddenWhatsappWebview(
  accountId: string
): Promise<WhatsAppWebCompanionManifest> {
  return invoke<WhatsAppWebCompanionManifest>(
    'start_hidden_whatsapp_webview',
    companionRequest(accountId)
  )
}

export async function relayWhatsappWebCompanionObservation(
  accountId: string,
  observation: Omit<WhatsAppWebCompanionRelayObservationRequest, 'account_id'>
): Promise<WhatsAppWebCompanionRelayObservationReceipt> {
  return invoke<WhatsAppWebCompanionRelayObservationReceipt>(
    'whatsapp_web_companion_relay_observation',
    {
      request: {
        ...observation,
        account_id: companionAccountId(accountId),
      },
    }
  )
}

export async function reportWhatsappWebCompanionCommandResult(
  accountId: string,
  result: Omit<WhatsAppWebCompanionCommandResultRequest, 'account_id'>
): Promise<string> {
  return invoke<string>('whatsapp_web_companion_report_command_result', {
    request: {
      ...result,
      account_id: companionAccountId(accountId),
    },
  })
}

function companionRequest(accountId: string): { request: WhatsAppWebCompanionRequest } {
  return {
    request: {
      account_id: companionAccountId(accountId),
    },
  }
}

function companionAccountId(accountId: string): string {
  const trimmed = accountId.trim()
  if (!trimmed) {
    throw new Error('account_id is required for WhatsApp Web companion')
  }
  return trimmed
}
