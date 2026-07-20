import { describe, expect, it, vi } from 'vitest'

vi.mock('./browserLocalDeviceKey', () => ({
	signBrowserLocalDeviceChallenge: vi.fn().mockResolvedValue(new Uint8Array(64)),
}))

import { BrowserGatewayAuthenticator } from './browserGatewayAuth'

describe('BrowserGatewayAuthenticator', () => {
	it('uses same-origin WebAuthn auth routes and never exposes a session identifier', async () => {
		const fetchImpl = vi.fn<typeof fetch>()
			.mockResolvedValueOnce(new Response(JSON.stringify({
				authentication_id: 'a'.repeat(64),
				public_key: {
					challenge: 'AQ',
					rpId: 'hub.local',
					allowCredentials: [{ type: 'public-key', id: 'AQ' }],
					userVerification: 'required',
				},
				browser_key_challenge: 'AQ',
			}), { status: 200 }))
			.mockResolvedValueOnce(new Response('{"authenticated":true}', { status: 200 }))
		const credentialGet = vi.fn().mockResolvedValue({
			id: 'AQ',
			rawId: Uint8Array.of(1).buffer,
			type: 'public-key',
			response: {
				authenticatorData: Uint8Array.of(2).buffer,
				clientDataJSON: Uint8Array.of(3).buffer,
				signature: Uint8Array.of(4).buffer,
				userHandle: null,
			},
		})
		const authenticator = new BrowserGatewayAuthenticator({
			origin: 'https://hub.local',
			fetchImpl,
			credentialGet,
		})

		await expect(authenticator.authenticate(Uint8Array.of(1))).resolves.toBeUndefined()

		const requestOptions = credentialGet.mock.calls[0]?.[0]
		expect(requestOptions).toEqual(expect.objectContaining({
			rpId: 'hub.local',
			userVerification: 'required',
		}))
		expect(Array.from(new Uint8Array(requestOptions?.challenge))).toEqual([1])
		expect(fetchImpl).toHaveBeenNthCalledWith(1,
			'/browser/v1/authentication/begin',
			expect.objectContaining({ credentials: 'same-origin', method: 'POST' }),
		)
		expect(fetchImpl).toHaveBeenNthCalledWith(2,
			`/browser/v1/authentication/${'a'.repeat(64)}/finish`,
			expect.objectContaining({ credentials: 'same-origin', method: 'POST' }),
		)
		expect(fetchImpl.mock.calls[1]?.[1]?.body).not.toContain('session')
		expect(fetchImpl.mock.calls[1]?.[1]?.body).toContain('browser_key_signature')
	})

	it('rejects malformed Gateway ceremonies before requesting an authenticator', async () => {
		const fetchImpl = vi.fn<typeof fetch>().mockResolvedValue(new Response('{}', { status: 200 }))
		const credentialGet = vi.fn()
		const authenticator = new BrowserGatewayAuthenticator({
			origin: 'https://hub.local',
			fetchImpl,
			credentialGet,
		})

		await expect(authenticator.authenticate(Uint8Array.of(1))).rejects.toThrow('response is invalid')
		expect(credentialGet).not.toHaveBeenCalled()
	})
})
