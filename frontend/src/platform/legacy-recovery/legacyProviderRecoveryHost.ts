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

export type LegacyProviderRecoveryTerminalStateV1 =
	| 'completed'
	| 'reauthorization_required'
	| 'qr_authorization_required'
	| 'blocked_source'
	| 'blocked_config'
	| 'outcome_unknown'

export type LegacyProviderRecoveryStepIdentifierV1 =
	| 'mail_gmail_create_target'
	| 'mail_gmail_update_settings'
	| 'mail_gmail_apply_settings'
	| `mail_gmail_oauth_start_revision_${bigint}`
	| 'mail_icloud_create_target'
	| 'mail_icloud_update_settings'
	| 'mail_icloud_apply_settings'
	| 'mail_icloud_provision_imap_password'
	| 'mail_icloud_bind_imap_password'
	| `telegram_update_settings_revision_${bigint}`
	| `telegram_apply_settings_revision_${bigint}`
	| 'telegram_provision_api_hash'
	| 'telegram_provision_session_store_key'
	| 'telegram_provision_account'

export type LegacyProviderRecoveryStepV1 = {
	disposition: 'execute' | 'completed' | 'outcome_unknown'
	operationId: Uint8Array
	targetConfigurationInstanceId?: string
	publicRevision?: bigint
}

export type BeginLegacyProviderRecoveryStepInputV1 = {
	recoverySessionId: string
	sourceHandle: string
	stepIdentifier: LegacyProviderRecoveryStepIdentifierV1
	targetConfigurationInstanceId?: string
	explicitRetry: boolean
}

export type CompleteLegacyProviderRecoveryStepInputV1 = {
	recoverySessionId: string
	sourceHandle: string
	stepIdentifier: LegacyProviderRecoveryStepIdentifierV1
	operationId: Uint8Array
	targetConfigurationInstanceId?: string
	publicRevision?: bigint
}

export type FinishLegacyProviderRecoveryCandidateInputV1 = {
	recoverySessionId: string
	sourceHandle: string
	targetConfigurationInstanceId: string
	terminalState: LegacyProviderRecoveryTerminalStateV1
}

export class LegacyProviderRecoveryOutcomeUnknownErrorV1 extends Error {
	constructor() {
		super('legacy provider recovery step outcome is unknown')
		this.name = 'LegacyProviderRecoveryOutcomeUnknownErrorV1'
	}
}

export type LegacyProviderRecoveryCandidateV1 = {
	sourceHandle: string
	kind: LegacyProviderCandidateKindV1
	state: LegacyProviderRecoveryStateV1
	receiptTerminalState?: LegacyProviderRecoveryTerminalStateV1
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
	beginStep(
		input: BeginLegacyProviderRecoveryStepInputV1,
	): Promise<LegacyProviderRecoveryStepV1>
	completeStep(input: CompleteLegacyProviderRecoveryStepInputV1): Promise<void>
	finishCandidate(input: FinishLegacyProviderRecoveryCandidateInputV1): Promise<void>
	cancel(recoverySessionId: string): Promise<void>
}
