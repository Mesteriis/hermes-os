import type { ThreadMessage } from '../types/communications'

export type ThreadMessageBodySegments = {
  mainText: string
  quotedText: string
}

export function splitThreadMessageBody(bodyText: string): ThreadMessageBodySegments {
  const normalized = bodyText.trim()
  if (!normalized) {
    return {
      mainText: '',
      quotedText: ''
    }
  }

  const lines = normalized.split('\n')
  const quotedStart = lines.findIndex((line, index) => {
    const trimmed = line.trim()
    if (trimmed.startsWith('>')) return true
    if (index === 0) return false
    return /^On .+ wrote:$/i.test(trimmed)
  })

  if (quotedStart <= 0) {
    return {
      mainText: normalized,
      quotedText: quotedStart === 0 ? normalized : ''
    }
  }

  return {
    mainText: lines.slice(0, quotedStart).join('\n').trim(),
    quotedText: lines.slice(quotedStart).join('\n').trim()
  }
}

export function previewThreadMessageBody(message: ThreadMessage, expanded: boolean): string {
  const { mainText, quotedText } = splitThreadMessageBody(message.body_text)
  const source = expanded ? (mainText || quotedText) : message.body_text
  const compact = source.trim().replace(/\s+/g, ' ')
  return compact.length > 220 ? `${compact.slice(0, 220)}...` : compact
}
