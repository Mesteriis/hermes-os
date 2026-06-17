import { describe, expect, it } from 'vitest'
import { telegramMediaTypeForFile } from './useTelegramMediaUploadQuery'

describe('telegramMediaTypeForFile', () => {
  it('maps common desktop files to supported Telegram upload kinds', () => {
    expect(telegramMediaTypeForFile({ name: 'photo.jpg', type: 'image/jpeg' } as File)).toBe('photo')
    expect(telegramMediaTypeForFile({ name: 'clip.mp4', type: 'video/mp4' } as File)).toBe('video')
    expect(telegramMediaTypeForFile({ name: 'voice.ogg', type: 'audio/ogg' } as File)).toBe('audio')
    expect(telegramMediaTypeForFile({ name: 'fun.gif', type: 'image/gif' } as File)).toBe('animation')
    expect(telegramMediaTypeForFile({ name: 'archive.zip', type: 'application/zip' } as File)).toBe('document')
  })
})
