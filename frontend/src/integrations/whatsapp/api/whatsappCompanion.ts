import { invoke } from '@tauri-apps/api/core'
import type { WhatsAppWebCompanionManifest } from '../../../shared/communications/types/whatsapp'

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

export async function openWhatsappWebCompanionForPairing(
  accountId: string
): Promise<WhatsAppWebCompanionManifest> {
  return invoke<WhatsAppWebCompanionManifest>(
    'open_whatsapp_web_companion',
    companionRequest(accountId)
  )
}

export async function hideWhatsappWebCompanion(
  accountId: string
): Promise<WhatsAppWebCompanionManifest> {
  return invoke<WhatsAppWebCompanionManifest>(
    'hide_whatsapp_web_companion',
    companionRequest(accountId)
  )
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
