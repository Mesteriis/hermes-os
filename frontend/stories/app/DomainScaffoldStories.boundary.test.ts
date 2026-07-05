import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

type DomainScaffoldStoryExpectation = {
  fileName: string
  storyTitle: string
  modelKey: string
}

const domainScaffoldStories: readonly DomainScaffoldStoryExpectation[] = [
  {
    fileName: 'Agents.stories.ts',
    storyTitle: 'Hermes App/AI Agents/Scaffold',
    modelKey: 'agents'
  },
  {
    fileName: 'Calendar.stories.ts',
    storyTitle: 'Hermes App/Calendar/Scaffold',
    modelKey: 'calendar'
  },
  {
    fileName: 'Communications.stories.ts',
    storyTitle: 'Hermes App/Communications/Scaffold',
    modelKey: 'communications'
  },
  {
    fileName: 'Documents.stories.ts',
    storyTitle: 'Hermes App/Documents/Scaffold',
    modelKey: 'documents'
  },
  {
    fileName: 'EventTraces.stories.ts',
    storyTitle: 'Hermes App/Event Traces/Scaffold',
    modelKey: 'eventTraces'
  },
  {
    fileName: 'Home.stories.ts',
    storyTitle: 'Hermes App/Home/Scaffold',
    modelKey: 'home'
  },
  {
    fileName: 'Knowledge.stories.ts',
    storyTitle: 'Hermes App/Knowledge Graph/Scaffold',
    modelKey: 'knowledge'
  },
  {
    fileName: 'Notes.stories.ts',
    storyTitle: 'Hermes App/Notes/Scaffold',
    modelKey: 'notes'
  },
  {
    fileName: 'Organizations.stories.ts',
    storyTitle: 'Hermes App/Organizations/Scaffold',
    modelKey: 'organizations'
  },
  {
    fileName: 'Persons.stories.ts',
    storyTitle: 'Hermes App/Persons/Scaffold',
    modelKey: 'persons'
  },
  {
    fileName: 'Projects.stories.ts',
    storyTitle: 'Hermes App/Projects/Scaffold',
    modelKey: 'projects'
  },
  {
    fileName: 'Review.stories.ts',
    storyTitle: 'Hermes App/Review/Scaffold',
    modelKey: 'review'
  },
  {
    fileName: 'Settings.stories.ts',
    storyTitle: 'Hermes App/Settings/Scaffold',
    modelKey: 'settings'
  },
  {
    fileName: 'Tasks.stories.ts',
    storyTitle: 'Hermes App/Tasks/Scaffold',
    modelKey: 'tasks'
  },
  {
    fileName: 'Timeline.stories.ts',
    storyTitle: 'Hermes App/Timeline/Scaffold',
    modelKey: 'timeline'
  }
]

describe('domain scaffold Storybook coverage', () => {
  it('keeps one app Storybook scaffold per planned domain', () => {
    for (const story of domainScaffoldStories) {
      const storyUrl = new URL(`./${story.fileName}`, import.meta.url)
      expect(existsSync(storyUrl)).toBe(true)

      const source = readFileSync(storyUrl, 'utf8')
      expect(source).toContain(`title: '${story.storyTitle}'`)
      expect(source).toContain(`domainScaffoldModels.${story.modelKey}`)
      expect(source).toContain('createDomainScaffoldStory')
      expect(source).not.toContain('createDomainSurfaceStory')
    }
  })

  it('keeps Storybook scaffolds separate from TS surface facades', () => {
    const storySources = domainScaffoldStories
      .map((story) => readFileSync(new URL(`./${story.fileName}`, import.meta.url), 'utf8'))
      .join('\n')
    const helperSource = readFileSync(new URL('./domainScaffoldStory.ts', import.meta.url), 'utf8')

    expect(storySources).not.toMatch(/use[A-Z][A-Za-z]+Surface/)
    expect(storySources).not.toContain('/queries/')
    expect(storySources).not.toContain('/Surface')
    expect(helperSource).not.toContain('surfacePath')
    expect(helperSource).not.toContain('contract')
  })
})
