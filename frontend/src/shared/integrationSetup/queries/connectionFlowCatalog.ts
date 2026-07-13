import type {
  ConnectionFlowPattern,
  ConnectionProviderOption,
} from '../../stores/integrationConnectionWizard'

export interface ConnectionFlowCard {
  id: ConnectionFlowPattern
  label: string
  icon: string
  summary: string
  promise: string
  recovery: string
  providers: ConnectionProviderOption[]
}

export const FLOW_CATALOG: Array<Omit<ConnectionFlowCard, 'providers'>> = [
  {
    id: 'browser_callback',
    label: 'Browser callback',
    icon: 'tabler:browser-share',
    summary: 'Launch provider auth in a secure browser tab and return to Hermes automatically.',
    promise: 'Primary route for browser-authorized providers',
    recovery: 'OAuth fields stay out of Settings.',
  },
  {
    id: 'qr_companion',
    label: 'QR companion',
    icon: 'tabler:qrcode',
    summary: 'Start a hidden provider runtime; pairing artifacts stay outside the settings surface.',
    promise: 'Managed hidden runtime',
    recovery: 'Phone and session material never render here.',
  },
  {
    id: 'managed_surface',
    label: 'Exception route',
    icon: 'tabler:route-alt-left',
    summary: 'Shown only when a provider cannot complete callback or QR onboarding from Settings.',
    promise: 'Hidden unless required',
    recovery: 'Manual recovery stays explicit and exceptional.',
  },
]

export const FLOW_ICONS: Record<ConnectionFlowPattern, string> = {
  browser_callback: 'tabler:browser-share',
  qr_companion: 'tabler:qrcode',
  managed_surface: 'tabler:route-alt-left',
}
