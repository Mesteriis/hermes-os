import { describe, expect, it } from 'vitest'
import {
	communicationSectionWorkflowState,
	communicationWorkflowStateSectionId
} from './communications'

describe('communication section workflow mapping', () => {
	it('maps UI section ids to backend workflow states', () => {
		expect(communicationSectionWorkflowState('unified')).toBe('')
		expect(communicationSectionWorkflowState('inbox')).toBe('new')
		expect(communicationSectionWorkflowState('needs_reply')).toBe('needs_action')
		expect(communicationSectionWorkflowState('waiting')).toBe('waiting')
		expect(communicationSectionWorkflowState('done')).toBe('done')
		expect(communicationSectionWorkflowState('archived')).toBe('archived')
	})

	it('maps backend workflow states back to UI section ids', () => {
		expect(communicationWorkflowStateSectionId('')).toBe('unified')
		expect(communicationWorkflowStateSectionId('new')).toBe('inbox')
		expect(communicationWorkflowStateSectionId('needs_action')).toBe('needs_reply')
		expect(communicationWorkflowStateSectionId('waiting')).toBe('waiting')
		expect(communicationWorkflowStateSectionId('done')).toBe('done')
		expect(communicationWorkflowStateSectionId('archived')).toBe('archived')
	})
})
