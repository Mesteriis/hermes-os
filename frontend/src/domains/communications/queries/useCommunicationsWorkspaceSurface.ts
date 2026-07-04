import { useMailCommunicationsSurface } from './useMailCommunicationsSurface'
import { useTelegramCommunicationsSurface } from './useTelegramCommunicationsSurface'
import { useWhatsappCommunicationsSurface } from './useWhatsappCommunicationsSurface'

export function useCommunicationsWorkspaceSurface() {
  const mail = useMailCommunicationsSurface()
  const telegram = useTelegramCommunicationsSurface()
  const whatsapp = useWhatsappCommunicationsSurface()

  const childSurfaces = [
    {
      id: mail.channelId,
      labelKey: mail.labelKey,
      status: mail.status,
      surfacePath: mail.surfacePath
    },
    {
      id: telegram.channelId,
      labelKey: telegram.labelKey,
      status: telegram.status,
      surfacePath: telegram.surfacePath
    },
    {
      id: whatsapp.channelId,
      labelKey: whatsapp.labelKey,
      status: whatsapp.status,
      surfacePath: whatsapp.surfacePath
    }
  ] as const

  return {
    surfaceId: 'communications',
    mail,
    telegram,
    whatsapp,
    childSurfaces
  }
}
