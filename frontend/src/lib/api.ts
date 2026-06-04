export type V1Status = {
	version: string;
	surfaces: {
		messages: boolean;
		contacts: boolean;
		search: boolean;
		documents: boolean;
	};
};

export async function fetchV1Status(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<V1Status> {
	const response = await fetch(`${baseUrl}/api/v1/status`, {
		headers: {
			Authorization: `Bearer ${token}`,
			'X-Hermes-Actor-Id': actorId
		}
	});

	if (!response.ok) {
		throw new Error(`V1 status request failed: ${response.status}`);
	}

	return (await response.json()) as V1Status;
}
