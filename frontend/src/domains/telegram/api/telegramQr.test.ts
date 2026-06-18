import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  cancelTelegramQrLogin,
  getTelegramQrLoginStatus,
  startTelegramQrLogin,
  submitTelegramQrPassword,
} from './telegram'

describe('telegram QR login API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('calls start, status, password and cancel QR routes', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ setup_id: 'qr-1', account_id: 'acc-1', status: 'waiting_qr_scan', qr_link: null, qr_svg: '<svg />', telegram_user_id: null, telegram_username: null, suggested_account_id: null, suggested_display_name: null, suggested_external_account_id: null, expires_at: null, poll_after_ms: 1500, message: null }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ setup_id: 'qr-1', account_id: 'acc-1', status: 'waiting_password', qr_link: null, qr_svg: null, telegram_user_id: null, telegram_username: null, suggested_account_id: null, suggested_display_name: null, suggested_external_account_id: null, expires_at: null, poll_after_ms: 1500, message: null }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ setup_id: 'qr-1', account_id: 'acc-1', status: 'ready', qr_link: null, qr_svg: null, telegram_user_id: '123', telegram_username: 'demo', suggested_account_id: 'acc-1-ready', suggested_display_name: 'Demo', suggested_external_account_id: 'telegram:123', expires_at: null, poll_after_ms: 1500, message: null }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ setup_id: 'qr-1', cancelled: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await startTelegramQrLogin({
      account_id: 'acc-1',
      display_name: 'Demo',
      external_account_id: 'telegram:123',
      tdlib_data_path: '/tmp/telegram-demo',
      transcription_enabled: false,
    })
    await getTelegramQrLoginStatus('qr-1')
    await submitTelegramQrPassword('qr-1', { password: 'secret' })
    await cancelTelegramQrLogin('qr-1')

    expect(fetchMock).toHaveBeenCalledTimes(4)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/telegram/login/qr/start')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/telegram/login/qr/qr-1')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/telegram/login/qr/qr-1/password')
    expect(fetchMock.mock.calls[3][0]).toContain('/api/v1/telegram/login/qr/qr-1')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(fetchMock.mock.calls[1][1].method).toBe('GET')
    expect(fetchMock.mock.calls[2][1].method).toBe('POST')
    expect(fetchMock.mock.calls[3][1].method).toBe('DELETE')
  })
})
