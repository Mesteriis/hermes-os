export type ProviderBrandTone =
  | 'anthropic'
  | 'api'
  | 'codex'
  | 'deepseek'
  | 'gmail'
  | 'icloud'
  | 'imap'
  | 'ollama'
  | 'omniroute'
  | 'openai'
  | 'telegram'
  | 'whatsapp'
  | 'yandex'
  | 'zoom'
  | 'zulip'

export interface ProviderBrand {
  icon: string
  tone: ProviderBrandTone
}

const DEFAULT_PROVIDER_BRAND: ProviderBrand = {
  icon: 'tabler:plug-connected',
  tone: 'api',
}

const COMMUNICATION_PROVIDER_BRANDS: Record<string, ProviderBrand> = {
  gmail: { icon: 'simple-icons:gmail', tone: 'gmail' },
  icloud: { icon: 'simple-icons:icloud', tone: 'icloud' },
  imap: { icon: 'tabler:server', tone: 'imap' },
  telegram_user: { icon: 'simple-icons:telegram', tone: 'telegram' },
  telegram_bot: { icon: 'simple-icons:telegram', tone: 'telegram' },
  whatsapp_web: { icon: 'simple-icons:whatsapp', tone: 'whatsapp' },
  zoom_user: { icon: 'simple-icons:zoom', tone: 'zoom' },
  zoom_server_to_server: { icon: 'simple-icons:zoom', tone: 'zoom' },
  yandex_telemost_user: { icon: 'simple-icons:yandexcloud', tone: 'yandex' },
  zulip_bot: { icon: 'simple-icons:zulip', tone: 'zulip' },
}

const API_PROVIDER_BRANDS: Record<string, ProviderBrand> = {
  anthropic: { icon: 'simple-icons:anthropic', tone: 'anthropic' },
  deepseek: { icon: 'simple-icons:deepseek', tone: 'deepseek' },
  google: { icon: 'simple-icons:googlegemini', tone: 'api' },
  gemini: { icon: 'simple-icons:googlegemini', tone: 'api' },
  ollama: { icon: 'simple-icons:ollama', tone: 'ollama' },
  omniroute: { icon: 'tabler:route-square', tone: 'omniroute' },
  openai: { icon: 'simple-icons:openai', tone: 'openai' },
  openrouter: { icon: 'simple-icons:openrouter', tone: 'api' },
  raw: { icon: 'tabler:braces', tone: 'api' },
}

export function communicationProviderBrand(providerKind: string): ProviderBrand {
  return COMMUNICATION_PROVIDER_BRANDS[normalizeProviderKey(providerKind)] ?? DEFAULT_PROVIDER_BRAND
}

export function aiProviderBrand(providerKind: string, providerKey?: string): ProviderBrand {
  const normalizedKind = normalizeProviderKey(providerKind)
  const normalizedKey = normalizeProviderKey(providerKey)

  if (normalizedKind === 'built_in') {
    if (normalizedKey.includes('ollama')) return API_PROVIDER_BRANDS.ollama
    return { icon: 'tabler:device-desktop', tone: 'api' }
  }

  if (normalizedKind === 'cli') {
    if (normalizedKey.includes('claude')) {
      return { icon: 'simple-icons:anthropic', tone: 'anthropic' }
    }
    if (normalizedKey.includes('codex')) {
      return { icon: 'simple-icons:openai', tone: 'codex' }
    }
    return { icon: 'tabler:terminal-2', tone: 'api' }
  }

  return API_PROVIDER_BRANDS[normalizedKey] ?? DEFAULT_PROVIDER_BRAND
}

export function providerBrandClass(brand: ProviderBrand): string {
  return `provider-brand-${brand.tone}`
}

function normalizeProviderKey(value: string | undefined): string {
  return (value ?? '').trim().toLowerCase()
}
