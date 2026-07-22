import { BrowserGatewayAuthenticator } from './browserGatewayAuth'
import { decodeBrowserCredentialId } from './browserGatewayEnrollment'
import { BrowserGatewayEnrollment } from './browserGatewayEnrollment'

export async function enrollBrowserGateway(pairingId: string): Promise<string> {
  return new BrowserGatewayEnrollment().enroll(pairingId.trim())
}

export async function authenticateBrowserGateway(credentialId: string): Promise<void> {
  await new BrowserGatewayAuthenticator().authenticate(decodeBrowserCredentialId(credentialId))
}

export function browserGatewayAccessError(reason: unknown): string {
  return reason instanceof Error ? reason.message : 'browser access failed'
}
