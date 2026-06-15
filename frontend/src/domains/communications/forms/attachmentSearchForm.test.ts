import { describe, expect, it } from 'vitest'
import {
  attachmentSearchFormDefaults,
  attachmentSearchFormToRequest
} from './attachmentSearchForm'

describe('attachment search form', () => {
  it('builds a bounded attachment search request from normalized form values', () => {
    expect(attachmentSearchFormToRequest({
      query: ' invoice ',
      content_type: ' application/pdf ',
      scan_status: 'not_scanned'
    }, ' account-1 ')).toEqual({
      account_id: 'account-1',
      q: 'invoice',
      content_type: 'application/pdf',
      scan_status: 'not_scanned',
      limit: 50
    })
  })

  it('keeps blank optional filters out of the request', () => {
    expect(attachmentSearchFormToRequest(attachmentSearchFormDefaults(), null)).toEqual({
      limit: 50
    })
  })
})
