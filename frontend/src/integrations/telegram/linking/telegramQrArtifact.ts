import QRCode from 'qrcode'

const MAX_TELEGRAM_QR_LINK_BYTES = 4096

export async function telegramQrDataUrl(qrLink: string): Promise<string> {
	const exactLink = qrLink.trim()
	if (!validTelegramQrLink(exactLink)) {
		throw new RangeError('telegram_qr_link_invalid')
	}
	return QRCode.toDataURL(exactLink, {
		errorCorrectionLevel: 'M',
		margin: 2,
		width: 280,
		color: {
			dark: '#0b1220',
			light: '#ffffff',
		},
	})
}

function validTelegramQrLink(value: string): boolean {
	if (!value || new TextEncoder().encode(value).byteLength > MAX_TELEGRAM_QR_LINK_BYTES) {
		return false
	}
	try {
		const parsed = new URL(value)
		return parsed.protocol === 'tg:'
			&& parsed.hostname === 'login'
			&& Boolean(parsed.searchParams.get('token'))
	} catch {
		return false
	}
}
