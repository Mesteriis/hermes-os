import type {
	AuthorizedProvisioningHostInputV1,
	SealedProvisioningHostCommandV1,
} from '../vault'

export type LegacyProviderCandidateKindV1 = 'gmail' | 'icloud' | 'telegram_user'
export type LegacyProviderRecoveryStateV1 =
	| 'ready_to_apply'
	| 'reauthorization_required'
	| 'qr_authorization_required'
export type LegacyProviderRecoverySecretPurposeV1 =
	| 'icloud_imap_password'
	| 'telegram_api_hash'
	| 'generated_telegram_session_store_key'

export type LegacyProviderRecoveryCandidateV1 = {
	sourceHandle: string
	kind: LegacyProviderCandidateKindV1
	state: LegacyProviderRecoveryStateV1
}

export type LegacyProviderRecoveryPlanV1 = {
	schemaRevision: 1
	recoverySessionId: string
	bundleFingerprintSha256: string
	counts: {
		gmailActive: 1
		icloudActive: 1
		telegramUserActive: 1
		gmailDeleted: 2
	}
	candidates: readonly LegacyProviderRecoveryCandidateV1[]
}

export type LegacyMailRecoverySourceV1 =
	| {
		kind: 'gmail'
		sourceHandle: string
		accountId: string
		displayName: string
		email: string
		oauthClientId: string
		oauthRedirectUri: string
	}
	| {
		kind: 'icloud'
		sourceHandle: string
		accountId: string
		displayName: string
		email: string
		imapHost: string
		imapPort: number
		username: string
	}

export type LegacyTelegramRecoverySourceV1 = {
	kind: 'telegram_user'
	sourceHandle: string
	accountId: string
	displayName: string
	externalAccountId: string
	apiId: bigint
}

export type LegacyProviderRecoverySourceV1 =
	| LegacyMailRecoverySourceV1
	| LegacyTelegramRecoverySourceV1

export type SealLegacyProviderRecoverySourceInputV1 = {
	recoverySessionId: string
	sourceHandle: string
	secretPurpose: LegacyProviderRecoverySecretPurposeV1
	hostSessionId: string
	operationId: Uint8Array
	action: number
	secretClass: number
	authorized: AuthorizedProvisioningHostInputV1
}

export interface LegacyProviderRecoveryHostV1 {
	start(): Promise<LegacyProviderRecoveryPlanV1>
	source(
		recoverySessionId: string,
		sourceHandle: string,
	): Promise<LegacyProviderRecoverySourceV1>
	sealSource(
		input: SealLegacyProviderRecoverySourceInputV1,
	): Promise<SealedProvisioningHostCommandV1>
	cancel(recoverySessionId: string): Promise<void>
}
