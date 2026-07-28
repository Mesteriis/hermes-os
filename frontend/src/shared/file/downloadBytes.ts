export function downloadBytesFile(
	filename: string,
	bytes: Uint8Array,
	contentType: string,
): void {
	const buffer = new ArrayBuffer(bytes.byteLength)
	new Uint8Array(buffer).set(bytes)
	const blobUrl = URL.createObjectURL(new Blob([buffer], { type: contentType }))
	const anchor = document.createElement('a')
	anchor.href = blobUrl
	anchor.download = filename
	anchor.rel = 'noopener'
	anchor.click()
	URL.revokeObjectURL(blobUrl)
}
