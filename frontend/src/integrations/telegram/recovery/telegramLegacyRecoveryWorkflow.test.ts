import { describe, expect, it, vi } from 'vitest'
import { Code, ConnectError } from '@connectrpc/connect'

import { TelegramLegacyRecoveryWorkflowV1 } from './telegramLegacyRecoveryWorkflow'

describe('TelegramLegacyRecoveryWorkflowV1', () => {
	it('uses native API hash and a fresh native session key before user-only provisioning', async () => {
		const sourceHandle = 'c'.repeat(64)
		const sealSource = vi.fn().mockResolvedValue({})
		const provisionCustodied = vi.fn().mockImplementation(async (input, seal) => {
			await seal({
				hostSessionId: 'host-session',
				operationId: input.operationId,
				action: input.action,
				secretClass: input.secretClass,
				authorized: {},
			})
			return { secretRevision: 1n }
		})
		const provision = vi.fn()
			.mockRejectedValueOnce(new ConnectError('credential unavailable', Code.FailedPrecondition))
			.mockResolvedValue({ accountId: 'telegram-account' })
		const apply = vi.fn().mockResolvedValue({})
		const workflow = new TelegramLegacyRecoveryWorkflowV1({
			source: {
				source: vi.fn().mockResolvedValue({
					kind: 'telegram_user',
					sourceHandle,
					accountId: 'telegram-account',
					displayName: 'Personal Telegram',
					externalAccountId: '',
					apiId: 123n,
				}),
				sealSource,
			},
			configuration: {
				apply,
			},
			vault: { provisionCustodied },
			lifecycle: { list: vi.fn().mockResolvedValue([]), provision },
		} as never)
		const plan = {
			schemaRevision: 1 as const,
			recoverySessionId: 'a'.repeat(32),
			bundleFingerprintSha256: 'b'.repeat(64),
			counts: {
				gmailActive: 1 as const,
				icloudActive: 1 as const,
				telegramUserActive: 1 as const,
				gmailDeleted: 2 as const,
			},
			candidates: [],
		}

		const result = await workflow.recover({
			registrationId: 'telegram-registration',
			expectedDesiredRevision: 1n,
			plan,
			candidate: {
				sourceHandle,
				kind: 'telegram_user',
				state: 'qr_authorization_required',
			},
		})

		expect(result.state).toBe('qr_authorization_required')
		expect(sealSource.mock.calls.map((call) => call[0].secretPurpose)).toEqual([
			'telegram_api_hash',
			'generated_telegram_session_store_key',
		])
		expect(provisionCustodied.mock.calls.map((call) => call[0].purposeId)).toEqual([
			'telegram_api_hash',
			'telegram_session_store_key',
		])
		expect(provision).toHaveBeenCalledWith(expect.objectContaining({
			accountId: 'telegram-account',
			credentials: [
				{ purpose: 'telegram_api_hash', revision: 1n },
				{ purpose: 'telegram_session_encryption_key', revision: 1n },
			],
		}))
		expect(provision).toHaveBeenCalledTimes(2)
		expect(apply.mock.calls[0]?.[0].values).toEqual(expect.arrayContaining([
			expect.objectContaining({ settingId: 'telegram.account_id' }),
			expect.objectContaining({ settingId: 'telegram.api_id' }),
		]))
		expect(apply).toHaveBeenCalledWith(expect.objectContaining({
			configurationInstanceId: 'telegram-registration',
			expectedDesiredRevision: 1n,
		}))
	})

	it('reuses an already-provisioned fresh session key without generating another', async () => {
		const sourceHandle = 'c'.repeat(64)
		const sealSource = vi.fn().mockResolvedValue({})
		const provisionCustodied = vi.fn().mockResolvedValue({ secretRevision: 1n })
		const provision = vi.fn().mockResolvedValue({ accountId: 'telegram-account' })
		const workflow = new TelegramLegacyRecoveryWorkflowV1({
			source: {
				source: vi.fn().mockResolvedValue({
					kind: 'telegram_user',
					sourceHandle,
					accountId: 'telegram-account',
					displayName: 'Personal Telegram',
					externalAccountId: '',
					apiId: 123n,
				}),
				sealSource,
			},
			configuration: { apply: vi.fn().mockResolvedValue({}) },
			vault: { provisionCustodied },
			lifecycle: { list: vi.fn().mockResolvedValue([]), provision },
		} as never)

		await workflow.recover({
			registrationId: 'telegram-registration',
			expectedDesiredRevision: 1n,
			plan: {
				schemaRevision: 1,
				recoverySessionId: 'a'.repeat(32),
				bundleFingerprintSha256: 'b'.repeat(64),
				counts: {
					gmailActive: 1,
					icloudActive: 1,
					telegramUserActive: 1,
					gmailDeleted: 2,
				},
				candidates: [],
			},
			candidate: {
				sourceHandle,
				kind: 'telegram_user',
				state: 'qr_authorization_required',
			},
		})

		expect(provisionCustodied).toHaveBeenCalledOnce()
		expect(provisionCustodied).toHaveBeenCalledWith(
			expect.objectContaining({ purposeId: 'telegram_api_hash' }),
			expect.any(Function),
		)
		expect(provision).toHaveBeenCalledWith(expect.objectContaining({
			credentials: [
				{ purpose: 'telegram_api_hash', revision: 1n },
				{ purpose: 'telegram_session_encryption_key', revision: 1n },
			],
		}))
	})

	it('reconciles an existing account without provisioning duplicate secrets', async () => {
		const provisionCustodied = vi.fn()
		const provision = vi.fn()
		const apply = vi.fn().mockResolvedValue({})
		const workflow = new TelegramLegacyRecoveryWorkflowV1({
			source: {
				source: vi.fn().mockResolvedValue({
					kind: 'telegram_user',
					sourceHandle: 'c'.repeat(64),
					accountId: 'telegram-account',
					displayName: 'Personal Telegram',
					externalAccountId: '',
					apiId: 123n,
				}),
			},
			configuration: { apply },
			vault: { provisionCustodied },
			lifecycle: {
				list: vi.fn().mockResolvedValue([{ accountId: 'telegram-account' }]),
				provision,
			},
		} as never)

		await workflow.recover({
			registrationId: 'telegram-registration',
			expectedDesiredRevision: 1n,
			plan: {
				schemaRevision: 1,
				recoverySessionId: 'a'.repeat(32),
				bundleFingerprintSha256: 'b'.repeat(64),
				counts: {
					gmailActive: 1,
					icloudActive: 1,
					telegramUserActive: 1,
					gmailDeleted: 2,
				},
				candidates: [],
			},
			candidate: {
				sourceHandle: 'c'.repeat(64),
				kind: 'telegram_user',
				state: 'qr_authorization_required',
			},
		})

		expect(apply).toHaveBeenCalledOnce()
		expect(provisionCustodied).not.toHaveBeenCalled()
		expect(provision).not.toHaveBeenCalled()
	})

	it('binds Settings operation identities to the exact CAS revisions', async () => {
		const apply = vi.fn().mockResolvedValue({})
		const workflow = new TelegramLegacyRecoveryWorkflowV1({
			source: {
				source: vi.fn().mockResolvedValue({
					kind: 'telegram_user',
					sourceHandle: 'c'.repeat(64),
					accountId: 'telegram-account',
					displayName: 'Personal Telegram',
					externalAccountId: '',
					apiId: 123n,
				}),
			},
			configuration: { apply },
			vault: { provisionCustodied: vi.fn() },
			lifecycle: {
				list: vi.fn().mockResolvedValue([{ accountId: 'telegram-account' }]),
				provision: vi.fn(),
			},
		} as never)
		const input = {
			registrationId: 'telegram-registration',
			plan: {
				schemaRevision: 1 as const,
				recoverySessionId: 'a'.repeat(32),
				bundleFingerprintSha256: 'b'.repeat(64),
				counts: {
					gmailActive: 1 as const,
					icloudActive: 1 as const,
					telegramUserActive: 1 as const,
					gmailDeleted: 2 as const,
				},
				candidates: [],
			},
			candidate: {
				sourceHandle: 'c'.repeat(64),
				kind: 'telegram_user' as const,
				state: 'qr_authorization_required' as const,
			},
		}

		await workflow.recover({ ...input, expectedDesiredRevision: 1n })
		await workflow.recover({ ...input, expectedDesiredRevision: 1n })
		await workflow.recover({ ...input, expectedDesiredRevision: 2n })

		const first = apply.mock.calls[0]?.[0]
		const replay = apply.mock.calls[1]?.[0]
		const successor = apply.mock.calls[2]?.[0]
		expect(first.updateOperationId).toEqual(replay.updateOperationId)
		expect(first.applyOperationId).toEqual(replay.applyOperationId)
		expect(successor.updateOperationId).not.toEqual(first.updateOperationId)
		expect(successor.applyOperationId).not.toEqual(first.applyOperationId)
	})
})
