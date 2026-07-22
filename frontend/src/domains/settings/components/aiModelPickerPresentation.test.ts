import { describe, expect, it } from 'vitest'
import { aiModelPickerDescription } from './aiModelPickerPresentation'

describe('ai model picker presentation', () => {
  const translate = (key: string) => `translated:${key}`

  it('describes the selected provider and available model count', () => {
    expect(aiModelPickerDescription(
      { display_name: 'Local AI' },
      3,
      5,
      translate,
    )).toBe('Local AI · 3/5 translated:available')
  })

  it('describes the empty provider state', () => {
    expect(aiModelPickerDescription(null, 0, 0, translate))
      .toBe('translated:Select a provider before choosing models.')
  })
})
