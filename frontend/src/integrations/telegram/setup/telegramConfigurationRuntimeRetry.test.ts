import { describe, expect, it, vi } from 'vitest'
import { Code, ConnectError } from '@connectrpc/connect'

import { withTelegramConfigurationRuntimeV1 } from './telegramConfigurationRuntimeRetry'

describe('withTelegramConfigurationRuntimeV1', () => {
	it('retries only the managed runtime control-channel handoff', async () => {
		const operation = vi.fn()
			.mockRejectedValueOnce(new Error('Telegram runtime control channel is unavailable'))
			.mockResolvedValue('ready')
		const wait = vi.fn().mockResolvedValue(undefined)

		await expect(withTelegramConfigurationRuntimeV1(operation, {
			attempts: 2,
			delayMillis: 25,
			wait,
		})).resolves.toBe('ready')
		expect(operation).toHaveBeenCalledTimes(2)
		expect(wait).toHaveBeenCalledWith(25)
	})

	it('does not retry provider or contract failures', async () => {
		const failure = new Error('telegram_account_setup_invalid')
		const operation = vi.fn().mockRejectedValue(failure)

		await expect(withTelegramConfigurationRuntimeV1(operation, {
			attempts: 3,
			wait: vi.fn(),
		})).rejects.toBe(failure)
		expect(operation).toHaveBeenCalledOnce()
	})

	it('retries a sanitized ConnectRPC infrastructure failure', async () => {
		const operation = vi.fn()
			.mockRejectedValueOnce(new ConnectError('HTTP 500', Code.Unknown))
			.mockResolvedValue('ready')

		await expect(withTelegramConfigurationRuntimeV1(operation, {
			attempts: 2,
			wait: vi.fn(),
		})).resolves.toBe('ready')
		expect(operation).toHaveBeenCalledTimes(2)
	})

	it('recognizes a ConnectRPC error crossing a bundled module boundary', async () => {
		const operation = vi.fn()
			.mockRejectedValueOnce({ name: 'ConnectError', code: Code.Unknown })
			.mockResolvedValue('ready')

		await expect(withTelegramConfigurationRuntimeV1(operation, {
			attempts: 2,
			wait: vi.fn(),
		})).resolves.toBe('ready')
		expect(operation).toHaveBeenCalledTimes(2)
	})
})
