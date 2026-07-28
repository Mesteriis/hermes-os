import { describe, expect, it } from 'vitest'

import {
	legacyRecoveryOperationIdV1,
	legacyRecoveryOperationKeyV1,
} from './legacyRecoveryOperationId'

describe('legacyRecoveryOperationIdV1', () => {
	it('derives stable non-zero operation identities without account PII', async () => {
		const fingerprint = 'a'.repeat(64)
		const handle = 'b'.repeat(64)
		const first = await legacyRecoveryOperationIdV1(fingerprint, handle, 'mail_create_target')
		const repeated = await legacyRecoveryOperationIdV1(fingerprint, handle, 'mail_create_target')
		const other = await legacyRecoveryOperationIdV1(fingerprint, handle, 'mail_apply_settings')

		expect(first).toEqual(repeated)
		expect(first).not.toEqual(other)
		expect(first).toHaveLength(16)
		expect(first.some((byte) => byte !== 0)).toBe(true)
		expect(await legacyRecoveryOperationKeyV1(
			fingerprint,
			handle,
			'mail_gmail_oauth',
		)).toMatch(/^legacy-recovery-[0-9a-f]{32}$/)
	})

	it('rejects unbounded or non-opaque recovery identity input', async () => {
		await expect(legacyRecoveryOperationIdV1(
			'not-a-fingerprint',
			'b'.repeat(64),
			'mail_create_target',
		)).rejects.toThrow('legacy recovery operation identity is invalid')
	})
})
