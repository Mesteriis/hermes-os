import { BrowserGatewayFetch } from './browserGatewayFetch'
import {
	createBrowserLocalDeviceKey,
	deleteBrowserLocalDeviceKey,
	saveBrowserLocalDeviceKey,
} from './browserLocalDeviceKey'

const PAIRING_ID = /^[a-f\d]{64}$/i

export class BrowserGatewayEnrollment {
	private readonly gateway = new BrowserGatewayFetch()

	async enroll(pairingId: string): Promise<string> {
		if (!PAIRING_ID.test(pairingId)) throw new Error('pairing code is invalid')
		const begin = await this.gateway.fetch(`/browser/v1/pairing/${pairingId}/registration`, { method: 'GET' })
		if (!begin.ok) throw new Error('pairing code is unavailable')
		const ceremony = parseCeremony(await begin.json())
		const credential = await navigator.credentials.create({ publicKey: ceremony.options })
		if (!(credential instanceof PublicKeyCredential)) throw new Error('browser WebAuthn credential is unavailable')
		const credentialId = base64UrlEncode(new Uint8Array(credential.rawId))
		const localKey = await createBrowserLocalDeviceKey()
		await saveBrowserLocalDeviceKey(credentialId, localKey.privateKey)
		try {
			const finish = await this.gateway.fetch(`/browser/v1/pairing/${pairingId}/registration/finish`, {
				method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({
					credential: registrationPayload(credential),
					browser_key_public_key: base64UrlEncode(localKey.publicKey),
				}),
			})
			if (!finish.ok) throw new Error('browser pairing was rejected')
			return credentialId
		} catch (error) {
			await deleteBrowserLocalDeviceKey(credentialId).catch(() => undefined)
			throw error
		}
	}
}

type RegistrationOptions = PublicKeyCredentialCreationOptions

function parseCeremony(value: unknown): { options: RegistrationOptions } {
	if (!record(value) || !record(value.public_key)) throw new Error('browser pairing response is invalid')
	// protobuf JSON wraps the generated `public_key` message in its oneof field
	// name (`publicKey`); retain support for the direct form used by the browser
	// contract fixtures as well.
	const options = record(value.public_key.publicKey) ? value.public_key.publicKey : value.public_key
	if (!record(options.rp) || !record(options.user) || typeof options.challenge !== 'string' || typeof options.user.id !== 'string' || !Array.isArray(options.pubKeyCredParams)) {
		throw new Error('browser pairing response is invalid')
	}
	return { options: {
		challenge: base64UrlDecode(options.challenge), rp: { name: string(options.rp.name), id: optionalString(options.rp.id) },
		user: { id: base64UrlDecode(options.user.id), name: string(options.user.name), displayName: string(options.user.displayName) },
		pubKeyCredParams: options.pubKeyCredParams.map((item) => ({ type: 'public-key' as const, alg: integer(record(item) ? item.alg : undefined) })),
		timeout: optionalInteger(options.timeout), attestation: optionalAttestation(options.attestation),
		excludeCredentials: Array.isArray(options.excludeCredentials) ? options.excludeCredentials.map(descriptor) : undefined,
		authenticatorSelection: authenticatorSelection(options.authenticatorSelection),
	} }
}

function descriptor(value: unknown): PublicKeyCredentialDescriptor {
	if (!record(value) || value.type !== 'public-key' || typeof value.id !== 'string') throw new Error('browser pairing response is invalid')
	return { type: 'public-key', id: base64UrlDecode(value.id) }
}

function registrationPayload(credential: PublicKeyCredential): object {
	const response = credential.response as AuthenticatorAttestationResponse
	return { id: credential.id, rawId: base64UrlEncode(new Uint8Array(credential.rawId)), type: credential.type, response: {
		attestationObject: base64UrlEncode(new Uint8Array(response.attestationObject)), clientDataJSON: base64UrlEncode(new Uint8Array(response.clientDataJSON)),
	} }
}

function base64UrlEncode(value: Uint8Array): string { return btoa(String.fromCharCode(...value)).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/, '') }
export function decodeBrowserCredentialId(value: string): Uint8Array {
	if (!/^[A-Za-z0-9_-]+$/.test(value)) throw new Error('browser credential is invalid')
	const normalized = value.replaceAll('-', '+').replaceAll('_', '/')
	const binary = atob(normalized.padEnd(Math.ceil(normalized.length / 4) * 4, '='))
	return Uint8Array.from(binary, (char) => char.charCodeAt(0))
}
function base64UrlDecode(value: string): ArrayBuffer { return decodeBrowserCredentialId(value).buffer as ArrayBuffer }
function record(value: unknown): value is Record<string, unknown> { return typeof value === 'object' && value !== null }
function string(value: unknown): string { if (typeof value !== 'string' || !value) throw new Error('browser pairing response is invalid'); return value }
function optionalString(value: unknown): string | undefined { return value === undefined ? undefined : string(value) }
function integer(value: unknown): number { if (typeof value !== 'number' || !Number.isInteger(value)) throw new Error('browser pairing response is invalid'); return value }
function optionalInteger(value: unknown): number | undefined { return value === undefined ? undefined : integer(value) }
function optionalAttestation(value: unknown): AttestationConveyancePreference | undefined { return value === undefined ? undefined : value === 'none' || value === 'indirect' || value === 'direct' || value === 'enterprise' ? value : undefined }
function authenticatorSelection(value: unknown): AuthenticatorSelectionCriteria | undefined {
	if (!record(value) || value.userVerification === undefined) return undefined
	const userVerification = value.userVerification === 'required' || value.userVerification === 'preferred' || value.userVerification === 'discouraged' ? value.userVerification : undefined
	if (!userVerification) throw new Error('browser pairing response is invalid')
	return { userVerification }
}
