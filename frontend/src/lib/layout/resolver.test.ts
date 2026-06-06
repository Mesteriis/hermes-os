import { describe, expect, it } from 'vitest';
import { LAYOUT_SCHEMA_VERSION } from './types';

describe('layout domain exports', () => {
	it('uses schema version 1 for the first persisted layout setting', () => {
		expect(LAYOUT_SCHEMA_VERSION).toBe(1);
	});
});
