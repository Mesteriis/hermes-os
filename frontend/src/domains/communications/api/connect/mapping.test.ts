import { describe, expect, it } from 'vitest'
import { parseJsonObject } from './mapping'

describe('communications Connect mapping', () => {
  it('returns an object only for object-shaped JSON metadata', () => {
    expect(parseJsonObject('{"source":"mail","retry_count":2}')).toEqual({
      source: 'mail',
      retry_count: 2
    })
    expect(parseJsonObject('["not-an-object"]')).toEqual({})
    expect(parseJsonObject('invalid-json')).toEqual({})
  })
})
