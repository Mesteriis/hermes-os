import { BrowserGatewayFetch } from './browserGatewayFetch'
import type { BrowserGatewayFetchOptions } from './browserGatewayFetch'
import { signBrowserLocalDeviceChallenge } from './browserLocalDeviceKey'

type BrowserAuthenticationCeremony = {
	authentication_id: string
	public_key: BrowserAuthenticationOptions
	browser_key_challenge: string
}

type BrowserAuthenticationOptions = {
	challenge: string
	timeout?: number
	rpId: string
	allowCredentials: BrowserCredentialDescriptor[]
	userVerification: UserVerificationRequirement
}

type BrowserCredentialDescriptor = {
	type: PublicKeyCredentialType
	id: string
	transports?: AuthenticatorTransport[]
}

type BrowserAssertionResponse = {
	authenticatorData: ArrayBuffer
	clientDataJSON: ArrayBuffer
	signature: ArrayBuffer
	userHandle: ArrayBuffer | null
}

export type BrowserAssertionCredential = {
	id: string
	rawId: ArrayBuffer
	type: PublicKeyCredentialType
	response: BrowserAssertionResponse
}

export type BrowserCredentialGetter = (
	options: PublicKeyCredentialRequestOptions,
) => Promise<BrowserAssertionCredential | null>

export type BrowserGatewayAuthenticatorOptions = BrowserGatewayFetchOptions & {
	credentialGet?: BrowserCredentialGetter
}

/**
 * Browser-only WebAuthn session bootstrap. It exposes neither a bearer token
 * nor the resulting opaque Gateway session identifier.
 */
export class BrowserGatewayAuthenticator {
	private readonly gateway: BrowserGatewayFetch
	private readonly credentialGet: BrowserCredentialGetter

	constructor(options: BrowserGatewayAuthenticatorOptions = {}) {
		const { credentialGet, ...fetchOptions } = options
		this.gateway = new BrowserGatewayFetch(fetchOptions)
		this.credentialGet = credentialGet ?? browserCredentialGet
	}

	async authenticate(credentialId: Uint8Array): Promise<void> {
		if (credentialId.byteLength === 0 || credentialId.byteLength > 1_024) {
			throw new Error('browser credential is invalid')
		}
		const ceremony = await this.begin(credentialId)
		const credential = await this.credentialGet(toRequestOptions(ceremony.public_key))
		if (!credential || !isAssertionCredential(credential)) {
			throw new Error('browser WebAuthn assertion is unavailable')
		}
		const browserKeySignature = await signBrowserLocalDeviceChallenge(
			base64UrlEncode(credentialId),
			base64UrlDecode(ceremony.browser_key_challenge),
		)
		const response = await this.gateway.fetch(
			`/browser/v1/authentication/${ceremony.authentication_id}/finish`,
			{
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					credential: assertionPayload(credential),
					browser_key_signature: base64UrlEncode(browserKeySignature),
				}),
			},
		)
		if (!response.ok) throw new Error('browser authentication was rejected')
	}

	private async begin(credentialId: Uint8Array): Promise<BrowserAuthenticationCeremony> {
		const response = await this.gateway.fetch('/browser/v1/authentication/begin', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ credential_id: base64UrlEncode(credentialId) }),
		})
		if (!response.ok) throw new Error('browser authentication is unavailable')
		return parseCeremony(await response.json())
	}
}

const browserCredentialGet: BrowserCredentialGetter = async (publicKey) => {
	const credential = await globalThis.navigator?.credentials?.get({ publicKey })
	return isAssertionCredential(credential) ? credential : null
}

function parseCeremony(value: unknown): BrowserAuthenticationCeremony {
	if (!isRecord(value) || typeof value.authentication_id !== 'string' || !isAuthenticationId(value.authentication_id) || typeof value.browser_key_challenge !== 'string') {
		throw new Error('browser authentication response is invalid')
	}
	const options = parseAuthenticationOptions(value.public_key)
	base64UrlDecode(value.browser_key_challenge)
	return { authentication_id: value.authentication_id, public_key: options, browser_key_challenge: value.browser_key_challenge }
}

function parseAuthenticationOptions(value: unknown): BrowserAuthenticationOptions {
	if (!isRecord(value)
		|| typeof value.challenge !== 'string'
		|| typeof value.rpId !== 'string'
		|| !Array.isArray(value.allowCredentials)
		|| !isUserVerification(value.userVerification)) {
		throw new Error('browser authentication response is invalid')
	}
	const timeout = optionalNonNegativeInteger(value.timeout)
	return {
		challenge: value.challenge,
		timeout,
		rpId: value.rpId,
		allowCredentials: value.allowCredentials.map(parseCredentialDescriptor),
		userVerification: value.userVerification,
	}
}

function parseCredentialDescriptor(value: unknown): BrowserCredentialDescriptor {
	if (!isRecord(value) || value.type !== 'public-key' || typeof value.id !== 'string') {
		throw new Error('browser authentication response is invalid')
	}
	const rawTransports = value.transports
	if (rawTransports !== undefined && !Array.isArray(rawTransports)) {
		throw new Error('browser authentication response is invalid')
	}
	const transports = rawTransports?.map((transport) => {
		if (!isAuthenticatorTransport(transport)) throw new Error('browser authentication response is invalid')
		return transport
	})
	return { type: value.type, id: value.id, transports }
}

function toRequestOptions(options: BrowserAuthenticationOptions): PublicKeyCredentialRequestOptions {
	return {
		challenge: base64UrlDecode(options.challenge),
		timeout: options.timeout,
		rpId: options.rpId,
		allowCredentials: options.allowCredentials.map((credential) => ({
			type: credential.type,
			id: base64UrlDecode(credential.id),
			transports: credential.transports,
		})),
		userVerification: options.userVerification,
	}
}

function assertionPayload(credential: BrowserAssertionCredential): object {
	return {
		id: credential.id,
		rawId: base64UrlEncode(new Uint8Array(credential.rawId)),
		type: credential.type,
		response: {
			authenticatorData: base64UrlEncode(new Uint8Array(credential.response.authenticatorData)),
			clientDataJSON: base64UrlEncode(new Uint8Array(credential.response.clientDataJSON)),
			signature: base64UrlEncode(new Uint8Array(credential.response.signature)),
			userHandle: credential.response.userHandle
				? base64UrlEncode(new Uint8Array(credential.response.userHandle))
				: null,
		},
	}
}

function isAssertionCredential(value: unknown): value is BrowserAssertionCredential {
	return isRecord(value)
		&& typeof value.id === 'string'
		&& value.type === 'public-key'
		&& value.rawId instanceof ArrayBuffer
		&& isAssertionResponse(value.response)
}

function isAssertionResponse(value: unknown): value is BrowserAssertionResponse {
	return isRecord(value)
		&& value.authenticatorData instanceof ArrayBuffer
		&& value.clientDataJSON instanceof ArrayBuffer
		&& value.signature instanceof ArrayBuffer
		&& (value.userHandle === null || value.userHandle instanceof ArrayBuffer)
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null
}

function isAuthenticationId(value: string): boolean {
	return /^[a-f\d]{64}$/i.test(value)
}

function isUserVerification(value: unknown): value is UserVerificationRequirement {
	return value === 'required' || value === 'preferred' || value === 'discouraged'
}

function isAuthenticatorTransport(value: unknown): value is AuthenticatorTransport {
	return value === 'usb' || value === 'nfc' || value === 'ble' || value === 'internal' || value === 'hybrid'
}

function optionalNonNegativeInteger(value: unknown): number | undefined {
	if (value === undefined) return undefined
	if (typeof value === 'number' && Number.isInteger(value) && value >= 0) return value
	throw new Error('browser authentication response is invalid')
}

function base64UrlEncode(value: Uint8Array): string {
	let binary = ''
	for (const byte of value) binary += String.fromCharCode(byte)
	return globalThis.btoa(binary).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/, '')
}

function base64UrlDecode(value: string): ArrayBuffer {
	if (!/^[A-Za-z0-9_-]+$/.test(value)) throw new Error('browser authentication response is invalid')
	const normalized = value.replaceAll('-', '+').replaceAll('_', '/')
	const binary = globalThis.atob(normalized.padEnd(Math.ceil(normalized.length / 4) * 4, '='))
	const bytes = new Uint8Array(binary.length)
	for (let index = 0; index < binary.length; index += 1) {
		bytes[index] = binary.charCodeAt(index)
	}
	return bytes.buffer
}
