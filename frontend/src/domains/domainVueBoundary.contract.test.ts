import { readdirSync, readFileSync } from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const domainsRoot = new URL('./', import.meta.url)
const sourceRoot = path.resolve(new URL('../', import.meta.url).pathname)

function collectVueFiles(rootPath: string): string[] {
  const entries = readdirSync(rootPath, { withFileTypes: true })
  const files: string[] = []

  for (const entry of entries) {
    const entryPath = path.join(rootPath, entry.name)
    if (entry.isDirectory()) {
      files.push(...collectVueFiles(entryPath))
      continue
    }
    if (entry.isFile() && entry.name.endsWith('.vue')) {
      files.push(entryPath)
    }
  }

  return files.sort()
}

function getScriptSection(source: string): string {
  const start = source.indexOf('<script')
  if (start === -1) return ''
  const openEnd = source.indexOf('>', start)
  if (openEnd === -1) return ''
  const end = source.indexOf('</script>', openEnd)
  if (end === -1) return ''
  return source.slice(openEnd + 1, end)
}

describe('Domain Vue boundary contract', () => {
  it('keeps domain and integration Vue components presentation-only', () => {
    const domainVueFiles = collectVueFiles(path.join(sourceRoot, 'domains'))
    const integrationVueFiles = collectVueFiles(path.join(sourceRoot, 'integrations'))
    const allVueFiles = [...domainVueFiles, ...integrationVueFiles].map((filePath) =>
      filePath.split(path.sep).join('/')
    )

    expect(allVueFiles).toContain(
      `${sourceRoot}/domains/communications/views/CommunicationsEmptyPage.vue`.split(path.sep).join('/')
    )
    expect(allVueFiles).toContain(
      `${sourceRoot}/domains/settings/views/SettingsPage.vue`.split(path.sep).join('/')
    )

    for (const filePath of allVueFiles) {
      const source = readFileSync(filePath, 'utf8')
      const scriptSource = getScriptSection(source)

      expect(scriptSource).not.toMatch(/from ['"][^'"]*\/api\/[^'"]*['"]/)
      expect(scriptSource).not.toMatch(/from ['"][^'"]*\/connect\/[^'"]*['"]/)
      expect(scriptSource).not.toMatch(/from ['"][^'"]*\/(model|mappers|policies|services|helpers|lib)\/[^'"]*['"]/)
      expect(scriptSource).not.toMatch(/fetch\s*\(/)
      expect(scriptSource).not.toMatch(/client\.(get|post|put|delete|patch)\s*\(/)
      expect(scriptSource).not.toMatch(/localStorage\./)
      expect(scriptSource).not.toMatch(/sessionStorage\./)
      expect(scriptSource).not.toMatch(/queryClient\./)
      expect(scriptSource).not.toMatch(/invalidateQueries\s*\(/)
      expect(scriptSource).not.toMatch(/\.(map|filter|reduce|sort)\s*\(/)
      expect(scriptSource).not.toMatch(/\bflatMap\s*\(/)
      expect(scriptSource).not.toMatch(/\bnew\s+(Set|Map)\s*\(/)
    }

    const settingsPageSource = readFileSync(
      new URL('./settings/views/SettingsPage.vue', domainsRoot),
      'utf8'
    )
    const communicationsEmptyPageSource = readFileSync(
      new URL('./communications/views/CommunicationsEmptyPage.vue', domainsRoot),
      'utf8'
    )

    expect(settingsPageSource).toContain("useSettingsPageSurface")
    expect(settingsPageSource).toContain("store.selectSection(item.id)")
    expect(settingsPageSource).not.toContain("useSettingsStore")
    expect(settingsPageSource).not.toContain("useApplicationSettingsQuery")

    expect(communicationsEmptyPageSource).toContain("communications.empty.title")
    expect(communicationsEmptyPageSource).not.toContain('fetch(')
  })
})
