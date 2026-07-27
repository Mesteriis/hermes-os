const OWNER_OPERATION_ID_LENGTH = 16

export function resolveOwnerOperationIdV1(value?: Uint8Array): Uint8Array {
	const operationId = value?.slice()
		?? crypto.getRandomValues(new Uint8Array(OWNER_OPERATION_ID_LENGTH))
	if (!isOwnerOperationIdV1(operationId)) {
		throw new Error('owner operation id is invalid')
	}
	return operationId
}

export function isOwnerOperationIdV1(value: Uint8Array): boolean {
	return value.byteLength === OWNER_OPERATION_ID_LENGTH
		&& value.some((byte) => byte !== 0)
}

export function sameOwnerOperationIdV1(
	left: Uint8Array,
	right: Uint8Array,
): boolean {
	return left.byteLength === right.byteLength
		&& left.every((byte, index) => byte === right[index])
}
