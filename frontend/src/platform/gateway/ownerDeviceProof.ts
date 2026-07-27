import { readBrowserGatewayCredentialId } from './browserGatewayCredential'
import { signBrowserLocalDeviceChallenge } from './browserLocalDeviceKey'

export interface OwnerDeviceProofV1 {
	sign(challenge: Uint8Array): Promise<Uint8Array>
}

/**
 * The browser profile owns only a non-extractable P-256 CryptoKey handle.
 * Application and integration code receive a signature, never private bytes.
 */
export class BrowserOwnerDeviceProofV1 implements OwnerDeviceProofV1 {
	async sign(challenge: Uint8Array): Promise<Uint8Array> {
		const credentialId = readBrowserGatewayCredentialId()
		if (!credentialId) throw new Error('owner device proof is unavailable')
		return signBrowserLocalDeviceChallenge(
			credentialId,
			challenge.buffer.slice(
				challenge.byteOffset,
				challenge.byteOffset + challenge.byteLength,
			) as ArrayBuffer,
		)
	}
}
